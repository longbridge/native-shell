//! Lists: three columns with a header of the viewable rows, a footer and
//! jumps, contacts under sticky letters, an inverted chat that loads older
//! messages, horizontal lists, pull to refresh on a scroll view and an
//! empty list with separators, each under a tab (as the TSX gallery's
//! `demos/Lists.tsx`).

use std::{rc::Rc, time::Duration};

use nsh_core::ui::{LayoutAnimation, prelude::*};
use nsh_ui_mobile::{KeyboardAvoidingView, KeyboardAvoidingViewProps};

use crate::ui::{
    Appear, AppearProps, Badge, Button, ButtonVariant, Finish, Finished, IconName, IconTile, MONO,
    Meter, PressProps, PressRole, RoundButton, Section, Segment, SegmentedEvent, Tone, colors,
    icon, press_with, radius, segmented,
};

/// The tones rows cycle through (the last, neutral, only in the grid).
const TONES: [Tone; 6] = [
    Tone::Accent,
    Tone::Info,
    Tone::Warn,
    Tone::Up,
    Tone::Danger,
    Tone::Neutral,
];

/// A tone's name (TS: the `Tone` string).
fn tone_name(tone: Tone) -> &'static str {
    match tone {
        Tone::Neutral => "neutral",
        Tone::Accent => "accent",
        Tone::Info => "info",
        Tone::Warn => "warn",
        Tone::Danger => "danger",
        Tone::Up => "up",
        Tone::Down => "down",
    }
}

/// A small pill action for list headers (36 pt, 44 pt with its hit slop).
fn action(
    label: &'static str,
    icon_name: Option<IconName>,
    on_press: Listener<()>,
) -> impl IntoElement {
    press_with(
        PressProps {
            label: Some(label.into()),
            hit_slop: Some(4.0),
            ..PressProps::default()
        },
        on_press,
    )
    .child(
        view()
            .h(px(36.0))
            .px(px(14.0))
            .rounded(px(18.0))
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .bg(colors::RAISED)
            .border_1()
            .border_color(colors::BORDER)
            .finish(Finish::Raised)
            .children(icon_name.map(|name| icon(name, 14.0, colors::TEXT)))
            .child(
                text(label)
                    .text_size(px(13.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT),
            ),
    )
}

/// The grid's items: 90, of four heights.
const ITEMS: usize = 90;

/// Three columns, a header and a footer, padding and gaps, viewability.
#[derive(Debug, Default)]
struct Grid {
    /// The first and last viewable rows.
    visible: Option<(usize, usize)>,
    list: Option<ElementRef>,
}

impl Grid {
    fn header(&self, cx: &RenderCx<'_, Self>) -> impl IntoElement + use<> {
        let (value, note) = match self.visible {
            Some((first, last)) => (
                format!("{first}–{last}"),
                format!("  {} visible", last - first + 1),
            ),
            None => ("—".to_owned(), String::new()),
        };
        let progress = self
            .visible
            .map_or(0.0, |(_, last)| (last + 1) as f32 / ITEMS as f32);
        view()
            .bg(colors::SURFACE)
            .rounded(px(radius::CARD))
            .border_curve(BorderCurve::Continuous)
            .border_1()
            .border_color(colors::BORDER)
            .p_4()
            .gap_3()
            .mb_1()
            .finish(Finish::Card)
            .child(
                view()
                    .flex_row()
                    .items_end()
                    .gap_2()
                    .child(
                        view()
                            .flex_1()
                            .gap(px(2.0))
                            .child(
                                text("Viewable items (at least half shown)")
                                    .text_size(px(13.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(
                                text(value)
                                    .text_size(px(30.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT)
                                    .tabular_nums()
                                    .letter_spacing(-0.5)
                                    .child(
                                        text(note)
                                            .text_size(px(15.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(colors::ACCENT),
                                    ),
                            ),
                    )
                    .child(
                        text("90 items, 3 columns")
                            .text_size(px(13.0))
                            .text_color(colors::FAINT)
                            .pb(px(6.0)),
                    ),
            )
            .child(Meter::new(progress))
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(action(
                        "Jump to 60",
                        Some(IconName::ArrowDown),
                        cx.listener(|this: &mut Self, _, _| {
                            if let Some(list) = &this.list {
                                list.scroll_to_index(60, 0.5, 0.0, true);
                            }
                        }),
                    ))
                    .child(action(
                        "End",
                        None,
                        cx.listener(|this: &mut Self, _, _| {
                            if let Some(list) = &this.list {
                                list.scroll_to_end(true);
                            }
                        }),
                    )),
            )
    }
}

impl Component for Grid {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.list = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let list = self.list.clone().expect("set up");
        view().flex_1().child(
            virtual_list(ITEMS, |id, _| {
                let height = 84 + (id % 4) * 22;
                view()
                    .h(px(height as f32))
                    .rounded(px(radius::INSET))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(if id == 60 {
                        Tone::Accent.color()
                    } else {
                        colors::BORDER
                    })
                    .bg(colors::RAISED)
                    .finish(Finish::Raised)
                    .p_3()
                    .justify_between()
                    .child(
                        view()
                            .w(px(18.0))
                            .h(px(4.0))
                            .rounded(px(2.0))
                            .bg(TONES[id % TONES.len()].color()),
                    )
                    .child(
                        view()
                            .child(
                                text(id.to_string())
                                    .text_size(px(22.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT)
                                    .tabular_nums(),
                            )
                            .child(
                                text(format!("{height} pt"))
                                    .text_size(px(12.0))
                                    .text_color(colors::FAINT)
                                    .tabular_nums(),
                            ),
                    )
            })
            .columns(3)
            .estimated_item_size(110.0)
            .content_padding(16.0)
            .row_gap(8.0)
            .column_gap(8.0)
            .viewability(Viewability {
                item_visible_percent: Some(50.0),
                ..Viewability::default()
            })
            .on_viewable_items_changed(cx.listener(|this: &mut Self, rows: &ViewableItems, cx| {
                this.visible = rows.viewable;
                cx.notify();
            }))
            .header(self.header(cx))
            .footer(
                text("End of the list")
                    .text_center()
                    .text_color(colors::FAINT)
                    .text_size(px(13.0))
                    .p_4(),
            )
            .flex_1()
            .element_ref(&list),
        )
    }
}

/// The people of the contacts: first and last name, what they made.
const PEOPLE: [(&str, &str, &str); 15] = [
    ("Ada", "Lovelace", "Analytical Engine notes"),
    ("Alan", "Kay", "Smalltalk"),
    ("Barbara", "Liskov", "CLU, substitution"),
    ("Brian", "Kernighan", "AWK, The C book"),
    ("Claude", "Shannon", "Information theory"),
    ("Dennis", "Ritchie", "C, Unix"),
    ("Donald", "Knuth", "TeX, TAOCP"),
    ("Edsger", "Dijkstra", "Shortest paths"),
    ("Frances", "Allen", "Optimizing compilers"),
    ("Grace", "Hopper", "COBOL, the first compiler"),
    ("Guido", "van Rossum", "Python"),
    ("Ken", "Thompson", "Unix, UTF-8"),
    ("Linus", "Torvalds", "Linux, Git"),
    ("Margaret", "Hamilton", "Apollo flight software"),
    ("Niklaus", "Wirth", "Pascal, Oberon"),
];

/// A row of the contacts: a letter's header or a person.
#[derive(Debug, Clone)]
enum ContactRow {
    Header {
        letter: char,
        count: usize,
    },
    Person {
        first: &'static str,
        last: &'static str,
        note: &'static str,
        tone: Tone,
        /// The last of its letter: no hairline under it.
        end: bool,
    },
}

/// One row of the contacts.
fn contact_row(row: &ContactRow, on_press: Listener<()>) -> AnyElement {
    match *row {
        ContactRow::Header { letter, count } => view()
            .flex_row()
            .items_center()
            .h(px(34.0))
            .px_4()
            .gap(px(10.0))
            // Opaque: rows scroll under it while it sticks.
            .bg(colors::BACKGROUND)
            .border_b_1()
            .border_b_color(colors::BORDER)
            .child(
                text(letter.to_string())
                    .text_size(px(13.0))
                    .font_weight(FontWeight::BOLD)
                    .text_color(colors::ACCENT),
            )
            .child(view().flex_1().h(px(1.0)).bg(colors::LINE))
            .child(
                text(count.to_string())
                    .text_size(px(13.0))
                    .text_color(colors::FAINT)
                    .tabular_nums(),
            )
            .into_element(),
        ContactRow::Person {
            first,
            last,
            note,
            tone,
            end,
        } => {
            let initials: String = first.chars().take(1).chain(last.chars().take(1)).collect();
            view()
                .on_press(on_press)
                .active(|s| s.bg(colors::RAISED))
                .flex_row()
                .items_center()
                .gap_3()
                .pl_4()
                .h(px(60.0))
                .child(
                    view()
                        .size(px(38.0))
                        .rounded(px(19.0))
                        .bg(tone.tint())
                        .items_center()
                        .justify_center()
                        .child(
                            text(initials)
                                .text_size(px(14.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(tone.color()),
                        ),
                )
                .child(
                    view()
                        .flex_1()
                        .self_stretch()
                        .justify_center()
                        .gap(px(2.0))
                        .pr_4()
                        .when(!end, |v| v.border_b_1().border_b_color(colors::LINE))
                        .child(
                            text(format!("{first} "))
                                .text_size(px(15.0))
                                .text_color(colors::TEXT)
                                .child(text(last).font_weight(FontWeight::SEMIBOLD)),
                        )
                        .child(
                            text(note)
                                .number_of_lines(1)
                                .text_size(px(13.0))
                                .text_color(colors::MUTED),
                        ),
                )
                .into_element()
        }
    }
}

/// Section headers stick to the top until the next one pushes them out.
#[derive(Debug, Default)]
struct Contacts {
    rows: Rc<Vec<ContactRow>>,
}

impl Component for Contacts {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        let initial = |name: &str| name.chars().next().unwrap_or(' ');
        let mut letters: Vec<char> = PEOPLE.iter().map(|p| initial(p.0)).collect();
        letters.dedup();
        let mut rows = Vec::new();
        for letter in letters {
            let group: Vec<_> = PEOPLE.iter().filter(|p| initial(p.0) == letter).collect();
            rows.push(ContactRow::Header {
                letter,
                count: group.len(),
            });
            for (i, &&(first, last, note)) in group.iter().enumerate() {
                rows.push(ContactRow::Person {
                    first,
                    last,
                    note,
                    tone: TONES[(rows.len() + i) % (TONES.len() - 1)],
                    end: i + 1 == group.len(),
                });
            }
        }
        self.rows = Rc::new(rows);
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let rows = self.rows.clone();
        // The headers' rows stick.
        let headers: Vec<usize> = (0..rows.len())
            .filter(|&i| matches!(rows[i], ContactRow::Header { .. }))
            .collect();
        // A tap lights a person's row up and does nothing else.
        let noop = cx.listener(|_: &mut Self, _: &(), _| {});
        view().flex_1().child(
            virtual_list(rows.len(), move |i, _| contact_row(&rows[i], noop.clone()))
                .sticky_header_indices(headers)
                .estimated_item_size(60.0)
                .content_padding_bottom(24.0)
                .flex_1(),
        )
    }
}

/// What the chat's messages say, in turn.
const LINES: [&str; 7] = [
    "Did the build go out?",
    "Yes, 2.4 is live on both stores",
    "Crash rate?",
    "Flat. 0.02 %",
    "Nice. The list scrolls so much smoother now",
    "Recycling plus native layout, no JS per frame",
    "Ship it",
];

/// A message of the chat.
#[derive(Debug, Clone)]
struct Message {
    id: u32,
    text: String,
    mine: bool,
    time: String,
}

/// An inverted list: starts at the newest message, keeps its place as
/// older ones load.
#[derive(Debug, Default)]
struct Chat {
    /// Newest first.
    messages: Rc<Vec<Message>>,
    next: u32,
    draft: String,
    /// Pages of older messages loaded.
    older: u32,
}

impl Chat {
    fn message(&mut self, text: String, mine: bool) -> Message {
        let id = self.next;
        self.next += 1;
        Message {
            id,
            text,
            mine,
            time: format!("{:02}:{:02}", 9 + (id / 60) % 12, id % 60),
        }
    }

    fn send(&mut self, cx: &mut Context<'_, Self>) {
        if self.draft.trim().is_empty() {
            return;
        }
        cx.animate_layout(LayoutAnimation::SPRING);
        let text = std::mem::take(&mut self.draft);
        let message = self.message(text, true);
        Rc::make_mut(&mut self.messages).insert(0, message);
        cx.notify();
    }
}

impl Component for Chat {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        let messages = (0..30)
            .map(|i| self.message(LINES[i % LINES.len()].to_owned(), i % 3 == 0))
            .collect();
        self.messages = Rc::new(messages);
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let messages = self.messages.clone();
        let bottom = if cx.keyboard_height() > 0.0 {
            0.0
        } else {
            cx.insets().bottom
        };
        let load_older = cx.listener(|this: &mut Self, _, cx| {
            let count = this.messages.len();
            this.older += 1;
            let older: Vec<_> = (0..20)
                .map(|i| this.message(format!("Older message {}", count + i + 1), i % 4 == 0))
                .collect();
            Rc::make_mut(&mut this.messages).extend(older);
            cx.notify();
        });
        let list = virtual_list(messages.len(), move |i, _| {
            let m = &messages[i];
            let mine = m.mine;
            view()
                .map(|v| {
                    if mine {
                        v.self_end().items_end()
                    } else {
                        v.self_start().items_start()
                    }
                })
                .max_w(relative(0.78))
                .gap(px(3.0))
                .child(
                    view()
                        .px(px(14.0))
                        .py(px(9.0))
                        .rounded(px(18.0))
                        .border_curve(BorderCurve::Continuous)
                        .rounded_br(px(if mine { 6.0 } else { 18.0 }))
                        .rounded_bl(px(if mine { 18.0 } else { 6.0 }))
                        .map(|v| {
                            if mine {
                                v.bg(colors::ACCENT_FILL).finish(Finish::Accent)
                            } else {
                                v.bg(colors::RAISED)
                                    .finish(Finish::Raised)
                                    .border_1()
                                    .border_color(colors::BORDER)
                            }
                        })
                        .child(
                            text(m.text.clone())
                                .text_color(if mine {
                                    colors::ON_ACCENT
                                } else {
                                    colors::TEXT
                                })
                                .text_size(px(15.0)),
                        ),
                )
                .child(
                    text(m.time.clone())
                        .text_size(px(11.0))
                        .text_color(colors::FAINT)
                        .tabular_nums()
                        .px_1(),
                )
        })
        .keys(self.messages.iter().map(|m| m.id))
        .inverted()
        .estimated_item_size(52.0)
        .content_padding_x(16.0)
        .content_padding_y(12.0)
        .row_gap(6.0)
        .flex_1()
        .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
        .on_end_reached(load_older)
        .end_reached_threshold(0.5);
        let room = view()
            .flex_row()
            .items_center()
            .gap_2()
            .px_4()
            .pb_2()
            .child(view().size(px(8.0)).rounded(px(4.0)).bg(colors::UP))
            .child(
                text("Release room")
                    .flex_1()
                    .text_size(px(13.0))
                    .text_color(colors::MUTED),
            )
            .child(
                text(format!(
                    "{} messages, {} older pages",
                    self.messages.len(),
                    self.older
                ))
                .text_size(px(13.0))
                .text_color(colors::FAINT)
                .tabular_nums(),
            );
        let bar = view()
            .flex_row()
            .items_center()
            .gap_2()
            .px_4()
            .pt_2()
            .pb(px(8.0 + bottom))
            .bg(colors::SURFACE)
            .border_t_1()
            .border_t_color(colors::BORDER)
            .child(
                input()
                    .value(self.draft.clone())
                    .on_change_text(cx.listener(|this: &mut Self, text: &str, cx| {
                        this.draft = text.to_owned();
                        cx.notify();
                    }))
                    .on_submit_editing(cx.listener(|this: &mut Self, _: &str, cx| this.send(cx)))
                    .blur_on_submit(false)
                    .placeholder("Message")
                    .placeholder_color(cx.color(colors::FAINT))
                    .selection_color(cx.color(colors::ACCENT))
                    .return_key_type(ReturnKeyType::Send)
                    .flex_1()
                    .h(px(44.0))
                    .text_size(px(15.0))
                    .text_color(colors::TEXT)
                    .bg(colors::INSET)
                    .finish(Finish::Inset)
                    .border_1()
                    .border_color(colors::BORDER)
                    .rounded(px(22.0))
                    .px_4(),
            )
            .child(RoundButton::new(
                IconName::ArrowUp,
                "Send",
                cx.listener(|this: &mut Self, _, cx| this.send(cx)),
            ));
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps::default())
                .flex_1()
                .child(room)
                .child(list)
                .child(bar),
        )
    }
}

/// The ranges of the horizontal scroll view.
const RANGES: [&str; 7] = [
    "Today",
    "Yesterday",
    "This week",
    "This month",
    "This year",
    "All time",
    "Custom",
];

/// An item of the empty list.
#[derive(Debug, Clone)]
struct Item {
    n: usize,
    tone: Tone,
}

/// Horizontal scrolling, pull to refresh on a plain scroll view, and an
/// empty list.
#[derive(Debug)]
struct More {
    refreshing: bool,
    refreshed: u32,
    range: &'static str,
    items: Rc<Vec<Item>>,
    /// The refresh in progress (dropping it cancels it).
    refresh: Option<Task<()>>,
}

impl Default for More {
    fn default() -> Self {
        Self {
            refreshing: false,
            refreshed: 0,
            range: "This week",
            items: Rc::default(),
            refresh: None,
        }
    }
}

impl More {
    fn refresh_section(&self) -> Section {
        Section::new("Pull to refresh").child(
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .child(IconTile::new(IconName::ArrowDown).tint(if self.refreshing {
                    colors::ACCENT
                } else {
                    colors::TEXT
                }))
                .child(
                    view()
                        .flex_1()
                        .gap(px(2.0))
                        .child(
                            text(if self.refreshing {
                                "Refreshing…"
                            } else {
                                "Pull the page down"
                            })
                            .text_size(px(15.0))
                            .text_color(colors::TEXT),
                        )
                        .child(
                            text("The system spinner on a plain scroll view")
                                .text_size(px(13.0))
                                .text_color(colors::MUTED),
                        ),
                )
                .child(
                    view()
                        .items_end()
                        .child(
                            text(self.refreshed.to_string())
                                .text_size(px(28.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT)
                                .tabular_nums(),
                        )
                        .child(
                            text("refreshes")
                                .text_size(px(12.0))
                                .text_color(colors::FAINT),
                        ),
                ),
        )
    }

    fn ranges(&self, cx: &RenderCx<'_, Self>) -> Section {
        let chips = RANGES.map(|label| {
            let selected = self.range == label;
            press_with(
                PressProps {
                    hit_slop: Some(4.0),
                    role: PressRole::Tab,
                    selected,
                    label: Some(label.into()),
                    ..PressProps::default()
                },
                cx.listener(move |this: &mut Self, _, cx| {
                    this.range = label;
                    cx.notify();
                }),
            )
            .child(
                view()
                    .h(px(36.0))
                    .px(px(14.0))
                    .rounded(px(18.0))
                    .justify_center()
                    .map(|v| {
                        if selected {
                            v.bg(colors::LIGHT).finish(Finish::Light)
                        } else {
                            v.bg(colors::RAISED)
                                .finish(Finish::Raised)
                                .border_1()
                                .border_color(colors::BORDER)
                        }
                    })
                    .child(
                        text(label)
                            .text_size(px(13.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(if selected {
                                colors::ON_LIGHT
                            } else {
                                colors::MUTED
                            }),
                    ),
            )
        });
        Section::new("Horizontal scroll view")
            .note(self.range)
            .child(
                view().mx(px(-14.0)).child(
                    scroll()
                        .horizontal()
                        .shows_indicator(false)
                        .child(view().flex_row().gap_2().px(px(14.0)).children(chips)),
                ),
            )
    }

    fn cards(&self) -> Section {
        let cards = virtual_list(1000, |i, _| {
            let tone = TONES[i % (TONES.len() - 1)];
            let load = ((i * 37) % 100) as f32 / 100.0;
            view()
                .w(px(120.0))
                .h(px(124.0))
                .rounded(px(radius::INSET))
                .border_curve(BorderCurve::Continuous)
                .bg(colors::RAISED)
                .finish(Finish::Raised)
                .border_1()
                .border_color(colors::BORDER)
                .p_3()
                .justify_between()
                .child(
                    text(format!("Card {}", i + 1))
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                        .tabular_nums(),
                )
                .child(
                    text(((load * 100.0).round() as u32).to_string())
                        .text_size(px(26.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT)
                        .tabular_nums()
                        .child(
                            text(" %")
                                .text_size(px(15.0))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(tone.color()),
                        ),
                )
                .child(Meter::new(load).tone(tone))
        })
        .horizontal()
        .estimated_item_size(130.0)
        .shows_indicator(false)
        .content_padding_x(14.0)
        .row_gap(10.0)
        .h(px(124.0));
        Section::new("Horizontal list")
            .note("1,000 recycled cards")
            .child(view().mx(px(-14.0)).child(cards))
    }

    fn empty_list(&self, cx: &RenderCx<'_, Self>) -> Section {
        // Changes to the list animate.
        let add = cx.listener(|this: &mut Self, _, cx| {
            cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
            let items = Rc::make_mut(&mut this.items);
            items.push(Item {
                n: items.len() + 1,
                tone: TONES[items.len() % (TONES.len() - 1)],
            });
            cx.notify();
        });
        let clear = cx.listener(|this: &mut Self, _, cx| {
            cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
            this.items = Rc::default();
            cx.notify();
        });
        let items = self.items.clone();
        let list = virtual_list(items.len(), move |i, _| {
            let item = &items[i];
            view()
                .h(px(48.0))
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .px(px(14.0))
                .child(view().size(px(8.0)).rounded(px(2.5)).bg(item.tone.color()))
                .child(
                    text(format!("Item {}", item.n))
                        .flex_1()
                        .text_size(px(15.0))
                        .text_color(colors::TEXT),
                )
                .child(Badge::new(tone_name(item.tone)).tone(item.tone))
        })
        .estimated_item_size(48.0)
        .empty(
            view()
                .items_center()
                .justify_center()
                .gap(px(10.0))
                .h(px(194.0))
                .child(IconTile::new(IconName::Lists).tint(colors::FAINT))
                .child(
                    text("Nothing here yet: tap Add")
                        .text_size(px(13.0))
                        .text_color(colors::MUTED),
                ),
        )
        .separator(|| view().h(px(1.0)).ml(px(14.0)).bg(colors::LINE))
        .h(px(196.0))
        .rounded(px(radius::CONTROL))
        .bg(colors::SURFACE)
        .finish(Finish::Card)
        .border_1()
        .border_color(colors::LINE);
        let mono = |name: &'static str| text(name).font_family(MONO).text_color(colors::TEXT);
        Section::new("An empty list")
            .note(format!("{} items", self.items.len()))
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(
                        view()
                            .flex_1()
                            .child(Button::new("Add", add).icon(IconName::Plus)),
                    )
                    .child(
                        view().flex_1().child(
                            Button::new("Clear", clear)
                                .variant(ButtonVariant::Secondary)
                                .disabled(self.items.is_empty()),
                        ),
                    ),
            )
            .child(list)
            .child(
                text("")
                    .text_size(px(13.0))
                    .text_color(colors::MUTED)
                    .child(mono("ListEmptyComponent"))
                    .child(" while empty, separators between rows; changes animate with ")
                    .child(mono("LayoutAnimation"))
                    .child("."),
            )
    }
}

impl Component for More {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().flex_1().child(
            scroll()
                .flex_1()
                .refreshing(self.refreshing)
                .on_refresh(cx.listener(|this: &mut Self, _, cx| {
                    this.refreshing = true;
                    cx.notify();
                    this.refresh = Some(cx.set_timeout(Duration::from_secs(1), |this, cx| {
                        this.refreshed += 1;
                        this.refreshing = false;
                        cx.notify();
                    }));
                }))
                .child(
                    view()
                        .p_4()
                        .gap_4()
                        .child(self.refresh_section())
                        .child(self.ranges(cx))
                        .child(self.cards())
                        .child(self.empty_list(cx)),
                ),
        )
    }
}

/// The demo's tabs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Grid,
    Sections,
    Chat,
    More,
}

const TABS: [(Tab, &str); 4] = [
    (Tab::Grid, "Grid"),
    (Tab::Sections, "Sections"),
    (Tab::Chat, "Chat"),
    (Tab::More, "More"),
];

/// The demo's page: the tabs and the selected one's list.
#[derive(Debug, Default)]
pub(crate) struct Lists {
    tab: Tab,
}

impl Component for Lists {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let selected = TABS.iter().position(|t| t.0 == self.tab).unwrap_or(0);
        let (key, content) = match self.tab {
            Tab::Grid => ("grid", Grid::element(()).into_element()),
            Tab::Sections => ("sections", Contacts::element(()).into_element()),
            Tab::Chat => ("chat", Chat::element(()).into_element()),
            Tab::More => ("more", More::element(()).into_element()),
        };
        view()
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(view().px_4().pt_2().pb_2().child(
                segmented(TABS.map(|t| Segment::new(t.1)), selected as u32).on_event(cx.listener(
                    |this: &mut Self, e: &SegmentedEvent, cx| {
                        let SegmentedEvent::Change(i) = *e;
                        this.tab = TABS[i as usize].0;
                        cx.notify();
                    },
                )),
            ))
            .child(
                Appear::element(AppearProps {
                    flex: true,
                    ..AppearProps::default()
                })
                .key(key)
                .child(content),
            )
    }
}
