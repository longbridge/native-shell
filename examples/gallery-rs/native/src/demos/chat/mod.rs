//! AI Chat (as the TSX gallery's `demos/Chat.tsx` and `demos/chat/`): a
//! simulated assistant whose replies stream token by token (batched per
//! frame), think first, call tools, fail and recover; every scenario
//! comes from local scripts and timers.
//!
//! The conversation is the chat's state; its rows are a virtualized list.
//! The reply being written is [`Live`], a global only [`LiveReply`] reads:
//! a frame of tokens renders that row again and nothing else, and its
//! Markdown blocks are components, so only the block being written does.

mod glyphs;
mod markdown;
mod parts;
mod scripts;

use std::{
    rc::Rc,
    time::{Duration, Instant},
};

use nsh_core::{
    LayoutAnimation, Size,
    ui::{AnyElement, ScrollEvent, ScrollOffset, prelude::*},
};
use nsh_ui_mobile::{KeyboardAvoidingView, KeyboardAvoidingViewProps};

use self::{
    glyphs::{GlyphName, glyph},
    markdown::{Block, MarkdownEvent, markdown, parse},
    parts::{
        action_button, avatar, caret, date_separator, error_row, file_chip, photo, spinner,
        tool_card,
    },
    scripts::{
        Assistant, FileRef, MODELS, Model, Msg, NOTES, PHOTO, Rating, Reasoning, Scenario, Status,
        Step, Tool, ToolState, freeform, history, retried, scenarios,
    },
};
use crate::ui::{
    Badge, Finish, Finished, IconName, IconTile, PressProps, RoundButton, Segment, Segmented,
    SegmentedEvent, SegmentedProps, SegmentedSize, Tone, appear, colors, icon, motion, press_with,
    radius, rgb_a,
};

/// The reply being written. Finished replies are plain data in the
/// chat's messages.
#[derive(Default)]
pub(crate) struct Live {
    id: Option<String>,
    model: Model,
    reason: String,
    /// When the thinking started, and its seconds once it ended.
    thinking: Option<(Instant, Option<u64>)>,
    tools: Vec<Tool>,
    blocks: Vec<Block>,
    /// Ticks once a second while thinking.
    now: Option<Instant>,
}

/// What the rows ask of the chat.
#[derive(Debug, Clone, PartialEq)]
enum Action {
    Copy(String, String),
    Regenerate(String),
    Retry(String),
    Rate(String, Rating),
    Reasoning(String),
}

/// The connection banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Banner {
    #[default]
    Offline,
    Online,
}

/// The features the empty chat offers: scenario, icon, note.
const FEATURED: [(&str, IconName, &str); 4] = [
    ("guide", IconName::Text, "Markdown, a table, code"),
    ("reason", IconName::Spark, "Reasoning first"),
    ("search", IconName::Search, "A tool call"),
    ("vision", IconName::Image, "A photo and a file"),
];

/// A reply being run.
struct Run {
    id: String,
    model: Model,
    script: &'static str,
    md: String,
    reason: String,
    think_start: Instant,
    think_seconds: Option<u64>,
    /// Tokens collect here and reach [`Live`] once per frame.
    pending_text: String,
    pending_reason: String,
    frame: Option<Task<()>>,
    ticker: Option<Task<()>>,
    /// The run's steps: dropping it (stopping, freezing) ends them.
    _task: Option<Task<()>>,
}

/// Where streamed tokens go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stream {
    Text,
    Reason,
}

/// The chat.
pub(crate) struct Chat {
    ids: u32,
    messages: Rc<Vec<Msg>>,
    model: Model,
    draft: String,
    input_height: f32,
    /// The photo and the notes are attached.
    attachment: bool,
    menu: bool,
    banner: Option<Banner>,
    banner_kind: Banner,
    /// The copied message or code block, for a moment.
    copied: Option<String>,
    copy_timer: Option<Task<()>>,
    /// Whether the user is at the bottom (the list then follows the
    /// stream natively); scrolling up clears it.
    following: bool,
    dragging: bool,
    list: Option<ElementRef>,
    run: Option<Run>,
    offline: Option<Task<()>>,
    settle: Option<Task<()>>,
    /// The jump button, the menu and the banner: 0 hidden, 1 shown.
    shown: Option<[NativeValue; 3]>,
}

impl Default for Chat {
    fn default() -> Self {
        Self {
            ids: 0,
            messages: Rc::default(),
            model: Model::Swift,
            draft: String::new(),
            input_height: 20.0,
            attachment: false,
            menu: false,
            banner: None,
            banner_kind: Banner::Offline,
            copied: None,
            copy_timer: None,
            following: true,
            dragging: false,
            list: None,
            run: None,
            offline: None,
            settle: None,
            shown: None,
        }
    }
}

/// Streams `text` at about `rate` tokens a second, with jitter; false
/// when the chat is gone.
async fn stream(
    this: &WeakComponent<Chat>,
    cx: &AsyncCx,
    text: &str,
    rate: f64,
    into: Stream,
) -> bool {
    let mut random = Jitter::new();
    for token in tokens(text) {
        let delay = 1.0 / rate * (0.55 + random.next() * 0.9);
        cx.timer(Duration::from_secs_f64(delay)).await;
        let added = this
            .update(move |chat: &mut Chat, cx| chat.token(token, into, cx))
            .await;
        if added.is_none() {
            return false;
        }
    }
    this.update(|chat: &mut Chat, cx| chat.flush(cx))
        .await
        .is_some()
}

/// Numbers in 0..1 (xorshift), seeded from the clock: the streams' jitter.
struct Jitter(u64);

impl Jitter {
    fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(1, |d| d.as_nanos() as u64);
        Self(seed | 1)
    }

    fn next(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// The tokens of `text`: a word piece of up to four characters with the
/// space before it, or a run of spaces.
fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while chars.peek().is_some() {
        let mut token = String::new();
        while let Some(&c) = chars.peek().filter(|c| c.is_whitespace()) {
            token.push(c);
            chars.next();
        }
        let mut word = 0;
        while let Some(&c) = chars.peek().filter(|c| !c.is_whitespace()) {
            if word == 4 {
                break;
            }
            token.push(c);
            chars.next();
            word += 1;
        }
        out.push(token);
    }
    out
}

impl Chat {
    fn next_id(&mut self) -> String {
        self.ids += 1;
        format!("m{}", self.ids - 1)
    }

    fn values(&self) -> &[NativeValue; 3] {
        self.shown.as_ref().expect("set up")
    }

    fn set_following(&mut self, following: bool) {
        if self.following != following {
            self.following = following;
            let _ = self.values()[0].spring(if following { 0.0 } else { 1.0 }, motion::SETTLE);
        }
    }

    fn set_menu(&mut self, menu: bool) {
        if self.menu != menu {
            self.menu = menu;
            let _ = self.values()[1].spring(if menu { 1.0 } else { 0.0 }, motion::SETTLE);
        }
    }

    fn set_banner(&mut self, banner: Option<Banner>) {
        if self.banner.is_some() != banner.is_some() {
            let _ =
                self.values()[2].spring(if banner.is_some() { 1.0 } else { 0.0 }, motion::SETTLE);
        }
        self.banner = banner;
    }

    /// Scrolls to the end on the user's own action (send, jump, load).
    fn scroll_to_end(&mut self, animated: bool) {
        self.set_following(true);
        if let Some(list) = &self.list {
            list.scroll_to_end(animated);
        }
    }

    fn update_messages(&mut self, f: impl FnOnce(&mut Vec<Msg>)) {
        f(Rc::make_mut(&mut self.messages));
    }

    /// The streamed tokens so far reach the live reply.
    fn flush(&mut self, cx: &mut Context<'_, Self>) {
        let Some(run) = self.run.as_mut() else {
            return;
        };
        run.frame = None;
        let reason = !run.pending_reason.is_empty();
        let text = !run.pending_text.is_empty();
        if !reason && !text {
            return;
        }
        run.reason
            .push_str(&std::mem::take(&mut run.pending_reason));
        run.md.push_str(&std::mem::take(&mut run.pending_text));
        let (r, md) = (run.reason.clone(), run.md.clone());
        cx.update_global::<Live, _>(|live, _| {
            if reason {
                live.reason = r;
            }
            if text {
                live.blocks = parse(&md, true);
            }
        });
    }

    /// A token arrives: it shows with the next frame.
    fn token(&mut self, token: String, into: Stream, cx: &mut Context<'_, Self>) {
        let Some(run) = self.run.as_mut() else {
            return;
        };
        match into {
            Stream::Text => run.pending_text.push_str(&token),
            Stream::Reason => run.pending_reason.push_str(&token),
        }
        if run.frame.is_none() {
            run.frame = Some(cx.request_animation_frame(|this: &mut Self, _, cx| this.flush(cx)));
        }
    }

    /// Freezes the reply into a message and ends the run.
    fn freeze(&mut self, status: Status, error: Option<&'static str>, cx: &mut Context<'_, Self>) {
        let Some(mut run) = self.run.take() else {
            return;
        };
        run.md.push_str(&run.pending_text);
        run.reason.push_str(&run.pending_reason);
        let seconds = run
            .think_seconds
            .unwrap_or_else(|| (run.think_start.elapsed().as_secs_f64().round() as u64).max(1));
        let tools = cx.update_global::<Live, _>(|live, _| {
            live.id = None;
            std::mem::take(&mut live.tools)
        });
        let tools = tools
            .into_iter()
            .map(|t| {
                if t.state == ToolState::Done {
                    t
                } else {
                    Tool {
                        state: ToolState::Cancelled,
                        ..t
                    }
                }
            })
            .collect();
        let blocks = parse(&run.md, false);
        let reply = Assistant {
            id: run.id.clone(),
            model: run.model,
            script: run.script,
            reasoning: (!run.reason.is_empty()).then(|| Reasoning {
                text: run.reason.clone(),
                seconds,
                open: false,
            }),
            tools,
            md: run.md.clone(),
            blocks,
            status,
            error,
            rating: None,
        };
        cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
        self.update_messages(|messages| {
            if let Some(m) = messages.iter_mut().find(|m| m.id() == reply.id) {
                *m = Msg::Assistant(reply);
            }
        });
        cx.notify();
        // The list follows the final layout natively; once the layout
        // animation has settled, a following list ends exactly at the
        // bottom.
        self.settle = Some(
            cx.set_timeout(Duration::from_millis(360), |this: &mut Self, _| {
                if this.following
                    && let Some(list) = &this.list
                {
                    list.scroll_to_end(true);
                }
            }),
        );
    }

    /// Runs a scenario's reply; `at` replaces that message (retry,
    /// regenerate).
    fn reply(&mut self, s: Scenario, at: Option<String>, cx: &mut Context<'_, Self>) {
        let id = self.next_id();
        let placeholder = Assistant {
            id: id.clone(),
            model: s.model,
            script: s.id,
            reasoning: None,
            tools: Vec::new(),
            md: String::new(),
            blocks: Vec::new(),
            status: Status::Done,
            error: None,
            rating: None,
        };
        cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
        let (live_id, model) = (id.clone(), s.model);
        cx.update_global::<Live, _>(|live, _| {
            *live = Live {
                id: Some(live_id),
                model,
                ..Live::default()
            };
        });
        self.update_messages(|messages| match at {
            Some(at) => {
                if let Some(m) = messages.iter_mut().find(|m| m.id() == at) {
                    *m = Msg::Assistant(placeholder);
                }
            }
            None => messages.push(Msg::Assistant(placeholder)),
        });
        cx.notify();
        let steps = s.steps;
        let task = cx.spawn(move |this, cx| async move {
            let wait = |ms: u64| cx.timer(Duration::from_millis(ms));
            wait(450).await;
            for step in steps {
                match step {
                    Step::Think(text) => {
                        let started = this
                            .update(|chat: &mut Self, cx| {
                                let now = Instant::now();
                                if let Some(run) = chat.run.as_mut() {
                                    run.think_start = now;
                                    run.ticker = Some(cx.set_interval(
                                        Duration::from_secs(1),
                                        |_: &mut Self, cx| {
                                            cx.update_global::<Live, _>(|live, _| {
                                                live.now = Some(Instant::now());
                                            });
                                        },
                                    ));
                                }
                                cx.update_global::<Live, _>(|live, _| {
                                    live.thinking = Some((now, None));
                                    live.now = Some(now);
                                });
                            })
                            .await;
                        if started.is_none() {
                            return;
                        }
                        if !stream(&this, &cx, &text, 75.0, Stream::Reason).await {
                            return;
                        }
                        this.update(|chat: &mut Self, cx| {
                            if let Some(run) = chat.run.as_mut() {
                                run.ticker = None;
                                let seconds =
                                    (run.think_start.elapsed().as_secs_f64().round() as u64).max(1);
                                run.think_seconds = Some(seconds);
                                let start = run.think_start;
                                cx.update_global::<Live, _>(|live, _| {
                                    live.thinking = Some((start, Some(seconds)));
                                });
                            }
                        })
                        .await;
                        wait(350).await;
                    }
                    Step::Tool(tool, ms) => {
                        let id = tool.id;
                        let set = move |state: ToolState| {
                            move |_: &mut Self, cx: &mut Context<'_, Self>| {
                                if state == ToolState::Done {
                                    cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
                                }
                                cx.update_global::<Live, _>(|live, _| {
                                    if let Some(t) = live.tools.iter_mut().find(|t| t.id == id) {
                                        t.state = state;
                                    }
                                });
                            }
                        };
                        this.update(move |_: &mut Self, cx| {
                            cx.update_global::<Live, _>(|live, _| {
                                live.tools.push(Tool {
                                    state: ToolState::Pending,
                                    ..tool
                                });
                            });
                        })
                        .await;
                        wait(450).await;
                        this.update(set(ToolState::Running)).await;
                        wait(ms).await;
                        this.update(set(ToolState::Done)).await;
                        wait(400).await;
                    }
                    Step::Say(text) => {
                        if !stream(&this, &cx, &text, 40.0, Stream::Text).await {
                            return;
                        }
                    }
                    Step::Fail(error) => {
                        wait(900).await;
                        this.update(move |chat: &mut Self, cx| {
                            chat.freeze(Status::Error, Some(error), cx)
                        })
                        .await;
                        return;
                    }
                    Step::Offline(_) => {}
                }
            }
            this.update(|chat: &mut Self, cx| chat.freeze(Status::Done, None, cx))
                .await;
        });
        self.run = Some(Run {
            id,
            model: s.model,
            script: s.id,
            md: String::new(),
            reason: String::new(),
            think_start: Instant::now(),
            think_seconds: None,
            pending_text: String::new(),
            pending_reason: String::new(),
            frame: None,
            ticker: None,
            _task: Some(task),
        });
    }

    fn stop(&mut self, cx: &mut Context<'_, Self>) {
        // Its task ends with it.
        self.freeze(Status::Stopped, None, cx);
    }

    /// Sends the user's message for a scenario, then its reply.
    fn submit(
        &mut self,
        s: Scenario,
        text: String,
        image: Option<&'static str>,
        file: Option<FileRef>,
        cx: &mut Context<'_, Self>,
    ) {
        if self.run.is_some() {
            return;
        }
        self.set_menu(false);
        self.set_following(true);
        let offline = s.steps.iter().find_map(|step| match step {
            Step::Offline(ms) => Some(*ms),
            _ => None,
        });
        let user_id = self.next_id();
        let user = Msg::User {
            id: user_id.clone(),
            text,
            image,
            file,
            waiting: offline.is_some(),
        };
        cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
        let date = self.messages.is_empty().then(|| Msg::Date {
            id: self.next_id(),
            label: "Today",
        });
        self.update_messages(|messages| {
            messages.extend(date);
            messages.push(user);
        });
        self.draft.clear();
        self.attachment = false;
        self.model = s.model;
        self.scroll_to_end(true);
        cx.notify();
        let rest = Scenario {
            steps: s
                .steps
                .iter()
                .filter(|step| !matches!(step, Step::Offline(_)))
                .cloned()
                .collect(),
            ..s
        };
        match offline {
            Some(ms) => {
                self.banner_kind = Banner::Offline;
                self.set_banner(Some(Banner::Offline));
                self.offline = Some(cx.spawn(move |this, cx| async move {
                    cx.timer(Duration::from_millis(ms)).await;
                    this.update(move |chat: &mut Self, cx| {
                        chat.banner_kind = Banner::Online;
                        chat.update_messages(|messages| {
                            if let Some(Msg::User { waiting, .. }) =
                                messages.iter_mut().find(|m| m.id() == user_id)
                            {
                                *waiting = false;
                            }
                        });
                        cx.notify();
                    })
                    .await;
                    cx.timer(Duration::from_millis(900)).await;
                    this.update(move |chat: &mut Self, cx| {
                        chat.set_banner(None);
                        chat.reply(rest, None, cx);
                    })
                    .await;
                }));
            }
            None => self.reply(rest, None, cx),
        }
    }

    fn send(&mut self, cx: &mut Context<'_, Self>) {
        let text = self.draft.trim().to_owned();
        if text.is_empty() && !self.attachment {
            return;
        }
        if self.attachment {
            let vision = scenarios()
                .into_iter()
                .find(|s| s.id == "vision")
                .expect("the vision scenario");
            let model = if self.model == Model::Swift {
                Model::Vision
            } else {
                self.model
            };
            let prompt = if text.is_empty() {
                vision.prompt.to_owned()
            } else {
                text
            };
            self.submit(
                Scenario { model, ..vision },
                prompt,
                Some(PHOTO),
                Some(NOTES),
                cx,
            );
        } else {
            let s = freeform(&text, self.model);
            self.submit(s, text, None, None, cx);
        }
    }

    fn play(&mut self, id: &str, cx: &mut Context<'_, Self>) {
        if id == "history" {
            self.set_menu(false);
            let mut older = history(|| self.next_id());
            self.update_messages(|messages| {
                older.append(messages);
                *messages = older;
            });
            self.scroll_to_end(false);
            cx.notify();
            return;
        }
        let Some(s) = scenarios().into_iter().find(|s| s.id == id) else {
            return;
        };
        let (prompt, image, file) = (s.prompt.to_owned(), s.image, s.file.clone());
        self.submit(s, prompt, image, file, cx);
    }

    fn prompt_before(&self, id: &str) -> String {
        let at = self.messages.iter().position(|m| m.id() == id).unwrap_or(0);
        self.messages[..at]
            .iter()
            .rev()
            .find_map(|m| match m {
                Msg::User { text, .. } => Some(text.clone()),
                _ => None,
            })
            .unwrap_or_default()
    }

    fn assistant(&self, id: &str) -> Option<&Assistant> {
        self.messages.iter().find_map(|m| match m {
            Msg::Assistant(a) if a.id == id => Some(a),
            _ => None,
        })
    }

    fn update_assistant(&mut self, id: &str, f: impl FnOnce(&mut Assistant)) {
        self.update_messages(|messages| {
            if let Some(Msg::Assistant(a)) = messages.iter_mut().find(|m| m.id() == id) {
                f(a);
            }
        });
    }

    fn act(&mut self, action: &Action, cx: &mut Context<'_, Self>) {
        match action {
            Action::Copy(key, text) => {
                cx.set_clipboard(text.clone());
                self.copied = Some(key.clone());
                let key = key.clone();
                self.copy_timer = Some(cx.set_timeout(
                    Duration::from_millis(1600),
                    move |this: &mut Self, cx| {
                        if this.copied.as_deref() == Some(key.as_str()) {
                            this.copied = None;
                            cx.notify();
                        }
                    },
                ));
            }
            Action::Regenerate(id) => {
                if self.run.is_some() {
                    return;
                }
                let Some(m) = self.assistant(id).cloned() else {
                    return;
                };
                self.set_following(true);
                let s = if m.script == "retry" {
                    retried()
                } else {
                    scenarios()
                        .into_iter()
                        .find(|s| s.id == m.script)
                        .unwrap_or_else(|| freeform(&self.prompt_before(id), m.model))
                };
                let steps = s
                    .steps
                    .iter()
                    .filter(|step| !matches!(step, Step::Offline(_) | Step::Fail(_)))
                    .cloned()
                    .collect();
                self.reply(
                    Scenario {
                        model: m.model,
                        steps,
                        ..s
                    },
                    Some(id.clone()),
                    cx,
                );
            }
            Action::Retry(id) => {
                if self.run.is_some() {
                    return;
                }
                self.set_following(true);
                self.reply(retried(), Some(id.clone()), cx);
            }
            Action::Rate(id, rating) => {
                let rating = *rating;
                self.update_assistant(id, |a| {
                    a.rating = if a.rating == Some(rating) {
                        None
                    } else {
                        Some(rating)
                    };
                });
            }
            Action::Reasoning(id) => {
                cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
                self.update_assistant(id, |a| {
                    if let Some(r) = a.reasoning.as_mut() {
                        r.open = !r.open;
                    }
                });
            }
        }
        cx.notify();
    }
}

impl Component for Chat {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.shown = Some([
            cx.native_value(0.0),
            cx.native_value(0.0),
            cx.native_value(0.0),
        ]);
        self.list = Some(cx.element_ref());
    }

    fn cleanup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.run = None;
        cx.update_global::<Live, _>(|live, _| *live = Live::default());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let insets = cx.insets();
        let keyboard = cx.keyboard_height();
        let [jump, menu_value, banner_value] = self.values();
        let streaming = self.run.is_some();
        let actions = cx.listener(|this: &mut Self, action: &Action, cx| this.act(action, cx));
        let play =
            |id: &'static str| cx.listener(move |this: &mut Self, _: &(), cx| this.play(id, cx));
        let chips: Vec<(&'static str, &'static str)> = scenarios()
            .iter()
            .map(|s| (s.id, s.chip))
            .chain([("history", "Load a long conversation")])
            .collect();

        let content: AnyElement = if self.messages.is_empty() {
            self.empty(cx).into_element()
        } else {
            self.list(actions.clone(), cx).into_element()
        };

        // Jump to the latest message: shown while scrolled up.
        let jump_button = view()
            .pointer_events(if self.following {
                PointerEvents::None
            } else {
                PointerEvents::Auto
            })
            .absolute()
            .right_4()
            .bottom_3()
            .bind_opacity(jump)
            .bind_scale(&jump.interpolate(&[0.0, 1.0], &[0.7, 1.0]))
            .child(
                RoundButton::new(
                    IconName::ArrowDown,
                    "Jump to the latest message",
                    cx.listener(|this: &mut Self, _: &(), _| this.scroll_to_end(true)),
                )
                .secondary(),
            )
            .when(streaming, |v| {
                v.child(
                    view()
                        .absolute()
                        .top(px(2.0))
                        .right(px(2.0))
                        .size(px(10.0))
                        .rounded(px(5.0))
                        .bg(colors::ACCENT)
                        .border_2()
                        .border_color(colors::BACKGROUND),
                )
            });

        // The connection banner.
        let offline = self.banner_kind == Banner::Offline;
        let banner_tone = if offline { Tone::Warn } else { Tone::Up };
        let banner = view()
            .pointer_events(PointerEvents::None)
            .absolute()
            .top_2()
            .left_4()
            .right_4()
            .bind_opacity(banner_value)
            .bind_translate_y(&banner_value.interpolate(&[0.0, 1.0], &[-64.0, 0.0]))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .min_h(px(48.0))
                    .px(px(14.0))
                    .rounded(px(radius::INSET))
                    .border_curve(BorderCurve::Continuous)
                    .finish(Finish::Menu)
                    .border_1()
                    .border_color(banner_tone.color().opacity(0.35))
                    .child(
                        view()
                            .size(px(28.0))
                            .rounded(px(14.0))
                            .items_center()
                            .justify_center()
                            .bg(banner_tone.tint())
                            .child(glyph(
                                if offline {
                                    GlyphName::Offline
                                } else {
                                    GlyphName::Check
                                },
                                15.0,
                                banner_tone.color(),
                            )),
                    )
                    .child(
                        view()
                            .flex_1()
                            .gap(px(1.0))
                            .child(
                                text(if offline {
                                    "You're offline"
                                } else {
                                    "Back online"
                                })
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                            )
                            .child(
                                text(if offline {
                                    "Your message sends when the connection returns."
                                } else {
                                    "Sending your message."
                                })
                                .text_size(px(13.0))
                                .text_color(colors::MUTED),
                            ),
                    ),
            );

        // The "+" menu: attachments and the scenarios, to replay them.
        let pointer = if self.menu {
            PointerEvents::Auto
        } else {
            PointerEvents::None
        };
        let backdrop = view()
            .pointer_events(pointer)
            .on_press(cx.listener(|this: &mut Self, _, cx| {
                this.set_menu(false);
                cx.notify();
            }))
            .absolute()
            .inset_0()
            .bg(rgb_a(0x000000, 0.45))
            .bind_opacity(menu_value);
        let rows = std::iter::once(("attach", GlyphName::Clip, "Attach a photo and a file"))
            .chain(
                chips
                    .iter()
                    .map(|&(id, label)| (id, GlyphName::Regenerate, label)),
            )
            .enumerate()
            .map(|(i, (id, name, label))| {
                view()
                    .on_press(cx.listener(move |this: &mut Self, _, cx| {
                        this.set_menu(false);
                        if id == "attach" {
                            this.attachment = true;
                        } else {
                            this.play(id, cx);
                        }
                        cx.notify();
                    }))
                    .underlay(cx.color(colors::TEXT.opacity(0.06)), 0.85)
                    .role(AccessibilityRole::Button)
                    .label(label)
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .h(px(48.0))
                    .px_4()
                    .child(glyph(
                        name,
                        16.0,
                        if i == 0 {
                            colors::ACCENT
                        } else {
                            colors::MUTED
                        },
                    ))
                    .child(
                        text(label)
                            .flex_1()
                            .text_size(px(15.0))
                            .text_color(colors::TEXT),
                    )
            });
        let menu = view()
            .pointer_events(pointer)
            .absolute()
            .left_4()
            .right_4()
            .bottom_2()
            .bind_opacity(menu_value)
            .bind_translate_y(&menu_value.interpolate(&[0.0, 1.0], &[24.0, 0.0]))
            .bind_scale(&menu_value.interpolate(&[0.0, 1.0], &[0.97, 1.0]))
            .child(
                view()
                    .rounded(px(radius::CARD))
                    .border_curve(BorderCurve::Continuous)
                    .finish(Finish::Menu)
                    .border_1()
                    .py(px(6.0))
                    .max_h(px(460.0))
                    .overflow_hidden()
                    .child(
                        text("ADD AND REPLAY")
                            .text_size(px(12.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::FAINT)
                            .px_4()
                            .pt_2()
                            .pb_1(),
                    )
                    .child(scroll().flex_grow(0.0).children(rows)),
            );

        // Suggestions while the conversation is empty.
        let suggestions = self.messages.is_empty().then(|| {
            scroll()
                .horizontal()
                .shows_indicator(false)
                .keyboard_taps(KeyboardTaps::Handled)
                .flex_grow(0.0)
                .child(
                    view()
                        .flex_row()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .children(chips.iter().map(|&(id, label)| {
                            press_with(
                                PressProps {
                                    hit_slop: Some(4.0),
                                    label: Some(label.into()).into(),
                                    ..PressProps::default()
                                },
                                play(id),
                            )
                            .child(
                                view()
                                    .h(px(36.0))
                                    .px(px(14.0))
                                    .rounded(px(18.0))
                                    .justify_center()
                                    .finish(Finish::Pill)
                                    .border_1()
                                    .child(
                                        text(label)
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(colors::TEXT),
                                    ),
                            )
                        })),
                )
        });

        // The composer.
        let can_send = !self.draft.trim().is_empty() || self.attachment;
        let composer = view()
            .px_4()
            .pt_2()
            .pb(px(8.0 + if keyboard > 0.0 { 0.0 } else { insets.bottom }))
            .gap_2()
            .when(self.attachment, |v| {
                v.child(
                    appear(0).child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(photo(PHOTO, 48.0, 48.0))
                            .child(view().flex_1().child(file_chip(
                                &NOTES,
                                Some(cx.listener(|this: &mut Self, _: &(), cx| {
                                    this.attachment = false;
                                    cx.notify();
                                })),
                            ))),
                    ),
                )
            })
            .child(
                view()
                    .rounded(px(24.0))
                    .border_curve(BorderCurve::Continuous)
                    // Glass like the menu above it, without the blur:
                    // nothing scrolls under it.
                    .finish_fill(Finish::Menu)
                    .finish_shadow(Finish::Menu)
                    .when_some(Finish::Menu.border_color(), |v, c| v.border_color(c))
                    .border_1()
                    .p(px(6.0))
                    .gap_1()
                    .child(
                        input()
                            .value(self.draft.clone())
                            .multiline(true)
                            .placeholder(format!("Message {}", self.model.name()))
                            .placeholder_color(cx.color(colors::FAINT))
                            .selection_color(cx.color(colors::ACCENT))
                            .on_focus(cx.listener(|this: &mut Self, _, cx| {
                                this.set_menu(false);
                                cx.notify();
                            }))
                            .on_content_size_change(cx.listener(
                                |this: &mut Self, size: &Size, cx| {
                                    let h = size.height.clamp(20.0, 100.0);
                                    if this.input_height != h {
                                        this.input_height = h;
                                        cx.notify();
                                    }
                                },
                            ))
                            .on_change_text(cx.listener(|this: &mut Self, text: &str, cx| {
                                this.draft = text.to_owned();
                                cx.notify();
                            }))
                            .h(px(self.input_height + 20.0))
                            .text_size(px(15.0))
                            .line_height(px(20.0))
                            .text_color(colors::TEXT)
                            .px(px(10.0))
                            .py(px(10.0)),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                view()
                                    .rotate(if self.menu { 45.0 } else { 0.0 })
                                    .transition_transform()
                                    .duration(200.0)
                                    .ease_out()
                                    .child(
                                        RoundButton::new(
                                            IconName::Plus,
                                            "Add",
                                            cx.listener(|this: &mut Self, _: &(), cx| {
                                                this.set_menu(!this.menu);
                                                cx.notify();
                                            }),
                                        )
                                        .secondary(),
                                    ),
                            )
                            .child(
                                view().flex_1().child(
                                    Segmented::element(SegmentedProps {
                                        options: MODELS
                                            .iter()
                                            .map(|m| Segment::new(m.name()))
                                            .collect::<Vec<_>>()
                                            .into(),
                                        selected: (MODELS
                                            .iter()
                                            .position(|m| *m == self.model)
                                            .unwrap_or(0)
                                            as u32)
                                            .into(),
                                        size: SegmentedSize::Small,
                                    })
                                    .on_event(cx.listener(
                                        |this: &mut Self, e: &SegmentedEvent, cx| {
                                            let SegmentedEvent::Change(i) = e;
                                            this.model = MODELS[*i as usize];
                                            cx.notify();
                                        },
                                    )),
                                ),
                            )
                            .child(if streaming {
                                press_with(
                                    PressProps {
                                        scale: 0.9.into(),
                                        label: Some("Stop generating".into()).into(),
                                        ..PressProps::default()
                                    },
                                    cx.listener(|this: &mut Self, _: &(), cx| this.stop(cx)),
                                )
                                .child(
                                    view()
                                        .size(px(44.0))
                                        .rounded(px(22.0))
                                        .items_center()
                                        .justify_center()
                                        .bg(colors::LIGHT)
                                        .finish(Finish::Light)
                                        .child(glyph(GlyphName::Stop, 16.0, colors::ON_LIGHT)),
                                )
                                .into_element()
                            } else {
                                view()
                                    .opacity(if can_send { 1.0 } else { 0.35 })
                                    .pointer_events(if can_send {
                                        PointerEvents::Auto
                                    } else {
                                        PointerEvents::None
                                    })
                                    .child(RoundButton::new(
                                        IconName::ArrowUp,
                                        "Send",
                                        cx.listener(|this: &mut Self, _: &(), cx| this.send(cx)),
                                    ))
                                    .into_element()
                            }),
                    ),
            );

        view().flex_1().bg(colors::BACKGROUND).child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps::default())
                .flex_1()
                .child(
                    view()
                        .flex_1()
                        .child(content)
                        .child(jump_button)
                        .child(banner)
                        .child(backdrop)
                        .child(menu),
                )
                .children(suggestions)
                .child(composer),
        )
    }
}

impl Chat {
    /// The empty conversation: a greeting and four features.
    fn empty(&self, cx: &RenderCx<'_, Self>) -> View {
        let all = scenarios();
        view()
            .flex_1()
            .justify_center()
            .px_4()
            .py_6()
            .gap_6()
            .child(
                appear(0).child(
                    view()
                        .items_center()
                        .gap_3()
                        .child(IconTile::new(IconName::Spark).tint(colors::ACCENT).size(56.0))
                        .child(
                            text("How can I help?")
                                .text_size(px(22.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                        )
                        .child(
                            text("A simulated assistant: every reply is scripted and streamed locally.")
                                .text_size(px(15.0))
                                .line_height(px(21.0))
                                .text_color(colors::MUTED)
                                .text_center(),
                        ),
                ),
            )
            .child(view().flex_row().flex_wrap().gap_3().children(
                FEATURED.iter().enumerate().map(|(i, &(id, name, note))| {
                    let chip = all.iter().find(|s| s.id == id).map_or("", |s| s.chip);
                    view().w(relative(0.47)).flex_grow(1.0).child(
                        appear(i as u32 + 1).child(
                            press_with(
                                PressProps {
                                    label: Some(chip.into()).into(),
                                    ..PressProps::default()
                                },
                                cx.listener(move |this: &mut Self, _: &(), cx| this.play(id, cx)),
                            )
                            .child(
                                view()
                                    .min_h(px(104.0))
                                    .p_3()
                                    .gap_2()
                                    .rounded(px(radius::INSET))
                                    .border_curve(BorderCurve::Continuous)
                                    .bg(colors::SURFACE)
                                    .finish(Finish::Card)
                                    .border_1()
                                    .border_color(colors::BORDER)
                                    .child(icon(name, 18.0, colors::ACCENT))
                                    .child(
                                        text(chip)
                                            .text_size(px(14.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT),
                                    )
                                    .child(
                                        text(note)
                                            .text_size(px(12.0))
                                            .text_color(colors::MUTED),
                                    ),
                            ),
                        ),
                    )
                }),
            ))
    }

    /// The conversation, virtualized.
    fn list(&self, actions: Listener<Action>, cx: &RenderCx<'_, Self>) -> VirtualList {
        let messages = self.messages.clone();
        let live = self.run.as_ref().map(|r| r.id.clone());
        let copied = self.copied.clone();
        let count = messages.len();
        let ids: Vec<String> = messages.iter().map(|m| m.id().to_owned()).collect();
        let on_copy = actions.clone().map(|e: &MarkdownEvent| {
            let MarkdownEvent::Copy { key, text } = e;
            Action::Copy(key.to_string(), text.to_string())
        });
        let list = self.list.as_ref().expect("set up");
        virtual_list(count, move |at| match &messages[at.index()] {
            Msg::Date { label, .. } => date_separator(label).into_element(),
            Msg::User {
                text: body,
                image,
                file,
                waiting,
                ..
            } => user_row(body, *image, file.as_ref(), *waiting).into_element(),
            Msg::Assistant(m) if live.as_deref() == Some(m.id.as_str()) => view()
                .child(
                    LiveReply::element(LiveReplyProps {
                        copied: copied.clone().map(Into::into),
                    })
                    .on_event(on_copy.clone()),
                )
                .into_element(),
            Msg::Assistant(m) => {
                assistant_row(m, copied.as_deref(), &actions, &on_copy).into_element()
            }
        })
        .keys(move || ids.clone())
        .estimated_item_size(140.0)
        .initial_scroll_index(count.saturating_sub(1))
        .flex_1()
        .content_padding_top(8.0)
        .content_padding_bottom(16.0)
        .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
        .follow_end(40.0)
        .event_throttle(100.0)
        .element_ref(list)
        .on_scroll_begin_drag(cx.listener(|this: &mut Self, _: &ScrollOffset, _| {
            this.dragging = true;
        }))
        .on_scroll(cx.listener(|this: &mut Self, e: &ScrollEvent, cx| {
            if this.dragging {
                let following = e.y + e.height >= e.content_height - 40.0;
                if following != this.following {
                    this.set_following(following);
                    cx.notify();
                }
            }
        }))
        .on_momentum_scroll_end(cx.listener(|this: &mut Self, _: &ScrollOffset, _| {
            this.dragging = false;
        }))
        .on_scroll_end(cx.listener(|this: &mut Self, _: &ScrollOffset, _| {
            this.dragging = false;
        }))
    }
}

/// A message of the user: its attachments and its text in a bubble.
fn user_row(
    body: &str,
    image: Option<&'static str>,
    file: Option<&FileRef>,
    waiting: bool,
) -> View {
    view()
        .items_end()
        .px_4()
        .py_2()
        .gap_2()
        .children(image.map(|src| photo(src, 220.0, 165.0)))
        .children(file.map(|f| file_chip(f, None)))
        .when(!body.is_empty(), |v| {
            v.child(
                view()
                    .max_w(relative(0.82))
                    .px(px(14.0))
                    .py(px(10.0))
                    .rounded(px(20.0))
                    .rounded_br(px(6.0))
                    .border_curve(BorderCurve::Continuous)
                    .bg(colors::RAISED)
                    .finish(Finish::Raised)
                    .border_1()
                    .border_color(colors::BORDER)
                    .child(
                        text(body.to_owned())
                            .selectable(true)
                            .text_size(px(15.0))
                            .line_height(px(21.0))
                            .text_color(colors::TEXT),
                    ),
            )
        })
        .when(waiting, |v| {
            v.child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(spinner(12.0, colors::WARN))
                    .child(
                        text("Waiting for a connection")
                            .text_size(px(12.0))
                            .text_color(colors::MUTED),
                    ),
            )
        })
}

/// The header of a reply: the avatar and the model.
fn reply_header(model: Model, stopped: bool) -> View {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(26.0))
        .child(avatar())
        .child(
            text(model.name())
                .flex_1()
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::MUTED),
        )
        .when(stopped, |v| v.child(Badge::new("Stopped").tone(Tone::Warn)))
}

/// The reasoning's toggle: a glyph (a spinner while thinking), its label
/// and a caret that turns.
fn reasoning_toggle(label: String, thinking: bool, open: bool) -> View {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .h(px(32.0))
        .self_start()
        .child(if thinking {
            spinner(14.0, colors::MUTED).into_element()
        } else {
            glyph(GlyphName::Brain, 16.0, colors::MUTED).into_element()
        })
        .child(
            text(label)
                .text_size(px(14.0))
                .text_color(colors::MUTED)
                .tabular_nums(),
        )
        .child(
            view()
                .rotate(if open { 0.0 } else { -90.0 })
                .transition_transform()
                .duration(220.0)
                .ease_out()
                .child(glyph(GlyphName::CaretDown, 14.0, colors::FAINT)),
        )
}

/// The reasoning's text, beside a rule.
fn reasoning_text(reasoning: &str) -> View {
    view()
        .flex_row()
        .gap_3()
        .pl(px(7.0))
        .child(view().w(px(2.0)).rounded(px(1.0)).bg(colors::BORDER))
        .child(
            text(reasoning.to_owned())
                .selectable(true)
                .flex_1()
                .text_size(px(14.0))
                .line_height(px(20.0))
                .text_color(colors::MUTED),
        )
}

/// A finished reply.
fn assistant_row(
    m: &Assistant,
    copied: Option<&str>,
    actions: &Listener<Action>,
    on_copy: &Listener<MarkdownEvent>,
) -> View {
    let act = |action: Action| actions.clone().map(move |_: &()| action.clone());
    let id = m.id.clone();
    let reasoning = m.reasoning.as_ref().map(|r| {
        view()
            .gap_2()
            .child(
                press_with(
                    PressProps {
                        scale: 0.98.into(),
                        label: Some("Reasoning".into()).into(),
                        selected: r.open.into(),
                        ..PressProps::default()
                    },
                    act(Action::Reasoning(id.clone())),
                )
                .child(reasoning_toggle(
                    format!("Thought for {} s", r.seconds),
                    false,
                    r.open,
                )),
            )
            .when(r.open, |v| v.child(reasoning_text(&r.text)))
    });
    let copied_reply = copied == Some(m.id.as_str());
    view()
        .px_4()
        .pt_2()
        .pb_1()
        .gap_3()
        .child(reply_header(m.model, m.status == Status::Stopped))
        .children(reasoning)
        .children(m.tools.iter().map(|t| tool_card(t).key(t.id)))
        .child(markdown(&m.blocks, &m.id, false, copied, on_copy.clone()))
        .when(m.status == Status::Error, |v| {
            v.child(error_row(
                m.error.unwrap_or("Something went wrong."),
                act(Action::Retry(id.clone())),
            ))
        })
        .when(m.status == Status::Stopped, |v| {
            v.child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(glyph(GlyphName::Stop, 12.0, colors::FAINT))
                    .child(
                        text("You stopped this reply")
                            .text_size(px(13.0))
                            .text_color(colors::FAINT),
                    ),
            )
        })
        .when(m.status != Status::Error, |v| {
            v.child(
                view()
                    .flex_row()
                    .items_center()
                    .ml(px(-8.0))
                    .mt(px(-6.0))
                    .child(action_button(
                        if copied_reply {
                            GlyphName::Check
                        } else {
                            GlyphName::Copy
                        },
                        "Copy",
                        copied_reply,
                        Tone::Up,
                        act(Action::Copy(id.clone(), m.md.clone())),
                    ))
                    .child(action_button(
                        GlyphName::Regenerate,
                        "Regenerate",
                        false,
                        Tone::Accent,
                        act(Action::Regenerate(id.clone())),
                    ))
                    .child(action_button(
                        GlyphName::ThumbUp,
                        "Good reply",
                        m.rating == Some(Rating::Up),
                        Tone::Accent,
                        act(Action::Rate(id.clone(), Rating::Up)),
                    ))
                    .child(action_button(
                        GlyphName::ThumbDown,
                        "Bad reply",
                        m.rating == Some(Rating::Down),
                        Tone::Danger,
                        act(Action::Rate(id.clone(), Rating::Down)),
                    )),
            )
        })
}

/// The reply being written: it reads [`Live`], so a frame of tokens
/// renders it again and nothing else.
#[derive(Debug, Default)]
pub(crate) struct LiveReply;

/// The props of [`LiveReply`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct LiveReplyProps {
    /// The copied message or code block.
    pub(crate) copied: Option<SharedString>,
}

impl Component for LiveReply {
    type Props = LiveReplyProps;
    type Event = MarkdownEvent;

    fn render(&self, props: &LiveReplyProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let live = cx.global::<Live>();
        let id = live.id.clone().unwrap_or_default();
        let thinking_now = live.thinking.is_some_and(|(_, s)| s.is_none());
        let label = match live.thinking {
            Some((start, None)) => {
                let now = live.now.unwrap_or(start);
                format!(
                    "Thinking… {} s",
                    now.saturating_duration_since(start).as_secs()
                )
            }
            Some((_, Some(seconds))) => format!("Thought for {seconds} s"),
            None => "Thought for 1 s".into(),
        };
        let waiting = live.blocks.is_empty() && live.tools.is_empty() && !thinking_now;
        let on_copy = cx.listener(|_: &mut Self, e: &MarkdownEvent, cx| cx.emit(e.clone()));
        view()
            .px_4()
            .pt_2()
            .pb_1()
            .gap_3()
            .child(reply_header(live.model, false))
            .when(!live.reason.is_empty(), |v| {
                v.child(
                    view()
                        .gap_2()
                        .child(reasoning_toggle(label, thinking_now, true))
                        .child(reasoning_text(&live.reason)),
                )
            })
            .children(live.tools.iter().map(|t| tool_card(t).key(t.id)))
            .when(waiting, |v| {
                v.child(view().flex_row().h(px(22.0)).items_center().child(caret()))
            })
            .child(markdown(
                &live.blocks,
                &id,
                true,
                props.copied.as_deref(),
                on_copy,
            ))
    }
}
