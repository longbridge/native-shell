//! Scrolling: paging, snapping on a recycled list, zooming, scroll events,
//! prepending and taps with the keyboard up (as the TSX gallery's
//! `demos/Scrolling.tsx`).

use std::rc::Rc;

use nsh_core::{
    Checked, Rect,
    ui::{AnyElement, View, prelude::*},
};

use crate::ui::legacy::{button, colors, page, section};

/// The pages', cards' and tiles' colors.
const PALETTE: [u32; 7] = [
    0x0969da, 0x8250df, 0xbf3989, 0xcf222e, 0xbc4c00, 0x1a7f37, 0x0a3069,
];

/// An on / off switch drawn with views (the TSX gallery's `Toggle` of
/// `ui.tsx`); the knob and the color move with a native value (typed
/// styles have no transitions).
#[derive(Debug, Default)]
struct Switch {
    /// 0 off, 1 on.
    on: Option<NativeValue>,
}

/// An on / off switch drawn with views.
#[derive(Debug, Clone, PartialEq, Props)]
struct SwitchProps {
    /// On.
    #[prop(default)]
    value: bool,
}

/// The events of a [`Switch`].
#[derive(Debug, Clone, PartialEq, Event)]
enum SwitchEvent {
    /// Pressed: the value it asks for.
    Change(bool),
}

impl Component for Switch {
    type Props = SwitchProps;
    type Event = SwitchEvent;

    fn setup(&mut self, props: &SwitchProps, cx: &mut Context<'_, Self>) {
        self.on = Some(cx.native_value(if props.value { 1.0 } else { 0.0 }));
    }

    fn props_changed(&mut self, old: &SwitchProps, new: &SwitchProps, _: &mut Context<'_, Self>) {
        if let Some(on) = &self.on
            && old.value != new.value
        {
            // Tailwind's `transition duration-200`.
            let _ = on.timing(
                if new.value { 1.0 } else { 0.0 },
                200.0,
                Easing::Bezier(0.4, 0.0, 0.2, 1.0),
            );
        }
    }

    fn render(&self, props: &SwitchProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let on = self.on.as_ref().expect("set up");
        let value = props.value;
        view()
            .h(px(31.0))
            .w(px(51.0))
            .rounded_full()
            .p(px(2.0))
            .bind_background(&on.interpolate_colors(&[0.0, 1.0], &[rgb(0xd4d4d8), rgb(0x34c759)]))
            .role(AccessibilityRole::Switch)
            .accessibility_state(AccessibilityState {
                checked: Some(if value { Checked::True } else { Checked::False }),
                ..AccessibilityState::default()
            })
            .hit_slop(EdgeInsets::all(8.0))
            .on_press(cx.listener(move |_: &mut Self, _, cx| cx.emit(SwitchEvent::Change(!value))))
            .child(
                view()
                    .size(px(27.0))
                    .rounded_full()
                    .bg(rgb(0xffffff))
                    .shadow_md()
                    .bind_translate_x(&on.interpolate(&[0.0, 1.0], &[0.0, 20.0])),
            )
    }
}

/// A labeled switch; `on_change` gets the new value.
fn toggle(label: &'static str, value: bool, on_change: Listener<bool>) -> View {
    view()
        .flex_row()
        .items_center()
        .child(text(label).flex_1())
        .child(
            Switch::element(SwitchProps { value }).on_event(on_change.map(|e: &SwitchEvent| {
                let SwitchEvent::Change(value) = e;
                *value
            })),
        )
}

/// Buttons for one of `options`; `on_change` gets the one pressed.
fn choice<T: Copy + PartialEq + 'static>(
    options: &[(T, &'static str)],
    value: T,
    on_change: Listener<T>,
) -> View {
    view()
        .flex_row()
        .gap(px(6.0))
        .flex_wrap()
        .children(options.iter().map(|&(option, label)| {
            let selected = option == value;
            text(label)
                .text_size(px(13.0))
                .px(px(10.0))
                .py(px(5.0))
                .rounded(px(6.0))
                .overflow_hidden()
                .text_color(if selected {
                    rgb(0xffffff)
                } else {
                    colors::text()
                })
                .bg(if selected {
                    colors::accent()
                } else {
                    rgb(0xe1e4e8)
                })
                .on_press(on_change.clone().map(move |_: &()| option))
        }))
}

/// Full-width pages: one per fling, with a page indicator.
#[derive(Debug, Default)]
struct Pager {
    /// The pages' width: the scroll view's.
    width: f32,
    page: usize,
}

impl Component for Pager {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        // The page's padding and the section's, until the layout tells.
        self.width = cx.environment().window.width - 60.0;
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let settle = || {
            cx.listener(|this: &mut Self, e: &ScrollOffset, cx| {
                if this.width > 0.0 {
                    this.page = (e.x / this.width).round().max(0.0) as usize;
                    cx.notify();
                }
            })
        };
        let pages = PALETTE[..5].iter().enumerate().map(|(i, &color)| {
            view()
                .w(px(self.width))
                .h(px(140.0))
                .bg(rgb(color))
                .items_center()
                .justify_center()
                .child(
                    text(format!("Page {}", i + 1))
                        .text_color(rgb(0xffffff))
                        .text_size(px(28.0))
                        .font_weight(FontWeight::BOLD),
                )
        });
        let dots = (0..5).map(|i| {
            view().size(px(8.0)).rounded(px(4.0)).bg(if self.page == i {
                colors::accent()
            } else {
                colors::border()
            })
        });
        view()
            .gap_2()
            .child(
                scroll()
                    .horizontal()
                    .paging(true)
                    .shows_indicator(false)
                    .h(px(140.0))
                    .rounded(px(12.0))
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        this.width = frame.width;
                        cx.notify();
                    }))
                    .on_momentum_scroll_end(settle())
                    .on_scroll_end(settle())
                    .child(view().flex_row().children(pages)),
            )
            .child(
                view()
                    .flex_row()
                    .justify_center()
                    .gap(px(6.0))
                    .children(dots),
            )
    }
}

/// Cards that snap to the center; a fling can pass several unless limited
/// to one.
#[derive(Debug, Default)]
struct Cards {
    align: Option<SnapAlignment>,
    one_step: bool,
}

impl Component for Cards {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        const CARD: f32 = 220.0;
        const GAP: f32 = 12.0;
        let align = self.align.unwrap_or(SnapAlignment::Center);
        let cards = virtual_list(20, |i, _| {
            view()
                .w(px(CARD))
                .h(px(120.0))
                .rounded(px(14.0))
                .bg(rgb(PALETTE[i % PALETTE.len()]))
                .items_center()
                .justify_center()
                .child(
                    text(format!("Card {}", i + 1))
                        .text_color(rgb(0xffffff))
                        .text_size(px(22.0))
                        .font_weight(FontWeight::BOLD),
                )
        })
        .horizontal()
        .estimated_item_size(CARD + GAP)
        .snap_interval(CARD + GAP)
        .snap_alignment(align)
        // `decelerationRate="fast"`.
        .deceleration(0.99)
        .disable_interval_momentum(self.one_step)
        .shows_indicator(false)
        .h(px(120.0))
        .row_gap(GAP);
        view()
            .gap(px(10.0))
            .child(cards)
            .child(choice(
                &[
                    (SnapAlignment::Start, "start"),
                    (SnapAlignment::Center, "center"),
                    (SnapAlignment::End, "end"),
                ],
                align,
                cx.listener(|this: &mut Self, align: &SnapAlignment, cx| {
                    this.align = Some(*align);
                    cx.notify();
                }),
            ))
            .child(toggle(
                "One card per fling",
                self.one_step,
                cx.listener(|this: &mut Self, on: &bool, cx| {
                    this.one_step = *on;
                    cx.notify();
                }),
            ))
    }
}

/// `onScroll` (throttled), drag and momentum events, `scrollEnabled`,
/// bounces, content insets.
#[derive(Debug)]
struct Events {
    offset: Option<ScrollEvent>,
    /// The latest drag and momentum events, newest first.
    log: Vec<String>,
    enabled: bool,
    bounces: bool,
    inset: bool,
    /// Milliseconds between `onScroll` events.
    throttle: u32,
}

impl Default for Events {
    fn default() -> Self {
        Self {
            offset: None,
            log: Vec::new(),
            enabled: true,
            bounces: true,
            inset: false,
            throttle: 100,
        }
    }
}

impl Component for Events {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let note = |name: &'static str| {
            cx.listener(move |this: &mut Self, e: &ScrollOffset, cx| {
                this.log.insert(0, format!("{name} @ {}", e.y.round()));
                this.log.truncate(5);
                cx.notify();
            })
        };
        let lines = (1..=30).map(|i| text(format!("Line {i}")).text_color(colors::muted()));
        let offset = match &self.offset {
            Some(e) => format!(
                "y {} of {} (viewport {})",
                e.y.round(),
                e.content_height.round(),
                e.height.round()
            ),
            None => "Scroll the box".to_owned(),
        };
        let log = if self.log.is_empty() {
            "No drags yet".to_owned()
        } else {
            self.log.join("\n")
        };
        view()
            .gap(px(10.0))
            .child(
                scroll()
                    .h(px(180.0))
                    .rounded(px(10.0))
                    .bg(rgb(0xf6f8fa))
                    .scroll_enabled(self.enabled)
                    .bounces(self.bounces)
                    .when(self.inset, |s| {
                        s.content_inset(EdgeInsets {
                            top: 40.0,
                            bottom: 40.0,
                            ..EdgeInsets::default()
                        })
                    })
                    .event_throttle(self.throttle as f32)
                    .on_scroll(cx.listener(|this: &mut Self, e: &ScrollEvent, cx| {
                        this.offset = Some(*e);
                        cx.notify();
                    }))
                    .on_scroll_begin_drag(note("begin drag"))
                    .on_scroll_end_drag(note("end drag"))
                    .on_momentum_scroll_begin(note("momentum begin"))
                    .on_momentum_scroll_end(note("momentum end"))
                    .child(view().p_3().gap(px(6.0)).children(lines)),
            )
            .child(text(offset).tabular_nums())
            .child(text(log).text_color(colors::muted()).text_size(px(13.0)))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(text("onScroll every").flex_1())
                    .child(choice(
                        &[(0, "0"), (100, "100"), (500, "500")],
                        self.throttle,
                        cx.listener(|this: &mut Self, ms: &u32, cx| {
                            this.throttle = *ms;
                            cx.notify();
                        }),
                    ))
                    .child(text("ms")),
            )
            .child(toggle(
                "Scroll enabled",
                self.enabled,
                cx.listener(|this: &mut Self, on: &bool, cx| {
                    this.enabled = *on;
                    cx.notify();
                }),
            ))
            .child(toggle(
                "Bounces",
                self.bounces,
                cx.listener(|this: &mut Self, on: &bool, cx| {
                    this.bounces = *on;
                    cx.notify();
                }),
            ))
            .child(toggle(
                "Content inset 40 (top and bottom)",
                self.inset,
                cx.listener(|this: &mut Self, on: &bool, cx| {
                    this.inset = *on;
                    cx.notify();
                }),
            ))
    }
}

/// Pinch to zoom (natively); tap a tile to zoom in on it.
#[derive(Debug)]
struct Zoom {
    scroll: Option<ElementRef>,
    /// The zoom, as the scroll view reports it.
    scale: f32,
    bounces: bool,
}

impl Default for Zoom {
    fn default() -> Self {
        Self {
            scroll: None,
            scale: 1.0,
            bounces: true,
        }
    }
}

impl Zoom {
    /// Zooms to `scale`, around `center` (content points) or the middle.
    fn zoom_to(&self, scale: f32, center: Option<(f32, f32)>) {
        if let Some(scroll) = &self.scroll {
            scroll.zoom_to(scale, center, true);
        }
    }
}

impl Component for Zoom {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.scroll = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let scroll_ref = self.scroll.clone().expect("set up");
        let tiles = (0..24).map(|i| {
            view()
                .w(px(74.0))
                .h(px(50.0))
                .rounded(px(8.0))
                .border_curve(BorderCurve::Continuous)
                .bg(rgb(PALETTE[i % PALETTE.len()]))
                .items_center()
                .justify_center()
                .on_press(cx.listener(move |this: &mut Self, _, _| {
                    // The tile's middle, in the content's (unzoomed) points.
                    let x = 6.0 + (i % 4) as f32 * 80.0 + 37.0;
                    let y = 6.0 + (i / 4) as f32 * 56.0 + 25.0;
                    let scale = if this.scale < 2.5 { 3.0 } else { 1.0 };
                    this.zoom_to(scale, Some((x, y)));
                }))
                .child(
                    text(format!("Tile {}", i + 1))
                        .text_color(rgb(0xffffff))
                        .text_size(px(11.0)),
                )
        });
        let zoom_button = |label: &'static str, scale: f32| {
            button(label, colors::accent())
                .on_press(cx.listener(move |this: &mut Self, _, _| this.zoom_to(scale, None)))
        };
        view()
            .gap(px(10.0))
            .child(
                scroll()
                    .h(px(240.0))
                    .rounded(px(10.0))
                    .bg(rgb(0x0d1117))
                    .zoom_range(0.5, 5.0)
                    .bounces_zoom(self.bounces)
                    .center_content(true)
                    .event_throttle(50.0)
                    .on_scroll(cx.listener(|this: &mut Self, e: &ScrollEvent, cx| {
                        if this.scale != e.zoom_scale {
                            this.scale = e.zoom_scale;
                            cx.notify();
                        }
                    }))
                    .element_ref(&scroll_ref)
                    .child(
                        view()
                            .flex_row()
                            .flex_wrap()
                            .p(px(6.0))
                            .gap(px(6.0))
                            .children(tiles),
                    ),
            )
            .child(text(format!("Zoom {:.2}×", self.scale)).tabular_nums())
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(zoom_button("1×", 1.0))
                    .child(zoom_button("2×", 2.0))
                    .child(zoom_button("0.5×", 0.5)),
            )
            .child(toggle(
                "Bounces past the limits",
                self.bounces,
                cx.listener(|this: &mut Self, on: &bool, cx| {
                    this.bounces = *on;
                    cx.notify();
                }),
            ))
    }
}

/// A post of the feed.
#[derive(Debug, Clone)]
struct Post {
    id: u32,
    label: String,
}

/// New items arrive at the top; the rows on screen stay where they are.
#[derive(Debug, Default)]
struct Feed {
    posts: Rc<Vec<Post>>,
    next: u32,
    fresh: u32,
}

impl Feed {
    fn post(&mut self, label: String) -> Post {
        let id = self.next;
        self.next += 1;
        Post { id, label }
    }
}

impl Component for Feed {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        let posts = (1..=30).map(|i| self.post(format!("Post {i}"))).collect();
        self.posts = Rc::new(posts);
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let posts = self.posts.clone();
        view()
            .gap(px(10.0))
            .child(
                virtual_list(posts.len(), move |i, _| {
                    let post = &posts[i];
                    view()
                        .px_3()
                        .py_3()
                        .border_b_1()
                        .border_color(rgb(0xeaeef2))
                        .child(text(post.label.clone()).text_color(
                            if post.label.starts_with("New") {
                                colors::accent()
                            } else {
                                colors::text()
                            },
                        ))
                })
                .keys(self.posts.iter().map(|p| p.id))
                .estimated_item_size(44.0)
                .h(px(240.0))
                .rounded(px(10.0))
                .bg(rgb(0xf6f8fa)),
            )
            .child(
                button("Add 3 posts on top", colors::accent()).on_press(cx.listener(
                    |this: &mut Self, _, cx| {
                        let fresh: Vec<_> = (0..3)
                            .map(|_| {
                                this.fresh += 1;
                                let label = format!("New {}", this.fresh);
                                this.post(label)
                            })
                            .collect();
                        Rc::make_mut(&mut this.posts).splice(0..0, fresh);
                        cx.notify();
                    },
                )),
            )
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Scrolling {
    taps: KeyboardTaps,
    presses: u32,
}

impl Component for Scrolling {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let explained = match self.taps {
            KeyboardTaps::Never => {
                "A tap outside the input only hides the keyboard; the button does not get it."
            }
            KeyboardTaps::Handled => {
                "The button gets the tap and the keyboard stays; a tap on empty space hides it."
            }
            KeyboardTaps::Always => "Taps never hide the keyboard.",
        };
        let taps: Vec<AnyElement> = vec![
            choice(
                &[
                    (KeyboardTaps::Never, "never"),
                    (KeyboardTaps::Handled, "handled"),
                    (KeyboardTaps::Always, "always"),
                ],
                self.taps,
                cx.listener(|this: &mut Self, taps: &KeyboardTaps, cx| {
                    this.taps = *taps;
                    cx.notify();
                }),
            )
            .into_element(),
            text(explained)
                .text_color(colors::muted())
                .text_size(px(13.0))
                .into_element(),
            input()
                .placeholder("Type here, then tap the button")
                .text_size(px(16.0))
                .border_1()
                .border_color(colors::border())
                .rounded(px(10.0))
                .p(px(10.0))
                .into_element(),
            button(format!("Pressed {} times", self.presses), colors::accent())
                .on_press(cx.listener(|this: &mut Self, _, cx| {
                    this.presses += 1;
                    cx.notify();
                }))
                .into_element(),
            view().h(px(300.0)).into_element(),
        ];
        let one = |content: AnyElement| [content];
        view().flex_1().child(
            page([
                section("Paging", one(Pager::element(()).into_element())).into_element(),
                section(
                    "Snapping (snapToInterval)",
                    one(Cards::element(()).into_element()),
                )
                .into_element(),
                section(
                    "Zoom (pinch, tap a tile)",
                    one(Zoom::element(()).into_element()),
                )
                .into_element(),
                section(
                    "Events and behavior",
                    one(Events::element(()).into_element()),
                )
                .into_element(),
                section(
                    "Prepending keeps the position",
                    one(Feed::element(()).into_element()),
                )
                .into_element(),
                section(
                    "Taps with the keyboard up (keyboardShouldPersistTaps)",
                    taps,
                )
                .into_element(),
            ])
            .keyboard_taps(self.taps),
        )
    }
}
