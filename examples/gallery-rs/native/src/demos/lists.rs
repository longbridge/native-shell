//! Lists: columns with a header, a footer and the visible rows, sticky
//! section headers, an inverted chat, horizontal lists and an empty list
//! with separators, each under a tab (as the TSX gallery's
//! `demos/Lists.tsx`).

use std::{rc::Rc, time::Duration};

use nsh_core::ui::{Text, prelude::*};
use nsh_ui_mobile::{KeyboardAvoidingView, KeyboardAvoidingViewProps};

use crate::ui::colors;

/// The tiles' colors.
const TINTS: [u32; 6] = [0xfde2e4, 0xe2ece9, 0xdfe7fd, 0xfff1e6, 0xe8e8e4, 0xcddafd];

/// The rows' separator color.
fn separator() -> Color {
    rgb(0xeaeef2)
}

/// A text that does `on_press` when tapped.
fn link(label: &'static str, on_press: Listener<()>) -> Text {
    text(label).text_color(colors::accent()).on_press(on_press)
}

/// Three columns, a header and a footer, padding and gaps, viewability.
#[derive(Debug, Default)]
struct Grid {
    /// The visible rows.
    visible: String,
    list: Option<ElementRef>,
}

impl Component for Grid {
    const NAME: &'static str = "gallery.lists.grid";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.list = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let list = self.list.clone().expect("set up");
        let header = view()
            .flex_row()
            .items_center()
            .gap_3()
            .pb_1()
            .child(
                text(format!("Visible {}", self.visible))
                    .flex_1()
                    .text_color(colors::muted()),
            )
            .child(link(
                "Item 60",
                cx.listener(|this: &mut Self, _, _| {
                    if let Some(list) = &this.list {
                        list.scroll_to_index(60, 0.5, 0.0, true);
                    }
                }),
            ))
            .child(link(
                "End",
                cx.listener(|this: &mut Self, _, _| {
                    if let Some(list) = &this.list {
                        list.scroll_to_end(true);
                    }
                }),
            ));
        view().flex_1().child(
            virtual_list(90, |id, _| {
                view()
                    .h(px(70.0 + (id % 4) as f32 * 20.0))
                    .rounded(px(10.0))
                    .bg(rgb(TINTS[id % TINTS.len()]))
                    .items_center()
                    .justify_center()
                    .child(
                        text(id.to_string())
                            .text_size(px(17.0))
                            .font_weight(FontWeight::SEMIBOLD),
                    )
            })
            .columns(3)
            .estimated_item_size(90.0)
            .content_padding(12.0)
            .row_gap(8.0)
            .column_gap(8.0)
            .viewability(Viewability {
                item_visible_percent: Some(50.0),
                ..Viewability::default()
            })
            .on_viewable_items_changed(cx.listener(|this: &mut Self, rows: &ViewableItems, cx| {
                this.visible = match rows.viewable {
                    Some((first, last)) => format!("{first}–{last}"),
                    None => String::new(),
                };
                cx.notify();
            }))
            .header(header)
            .footer(
                text("— the end —")
                    .text_center()
                    .text_color(colors::muted())
                    .p_3(),
            )
            .flex_1()
            .element_ref(&list),
        )
    }
}

/// A row of the contacts: a letter's header or a name.
#[derive(Debug, Clone)]
enum ContactRow {
    Header(char),
    Name { name: &'static str, last: bool },
}

/// Contacts in sections by their first letter.
#[derive(Debug, Default)]
struct Contacts {
    rows: Rc<Vec<ContactRow>>,
}

impl Component for Contacts {
    const NAME: &'static str = "gallery.lists.contacts";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        const NAMES: [&str; 15] = [
            "Ada", "Alan", "Barbara", "Brian", "Claude", "Dennis", "Donald", "Edsger", "Frances",
            "Grace", "Guido", "Ken", "Linus", "Margaret", "Niklaus",
        ];
        let initial = |name: &str| name.chars().next().unwrap_or(' ');
        let mut rows = Vec::new();
        let mut letters: Vec<char> = NAMES.iter().map(|n| initial(n)).collect();
        letters.dedup();
        for letter in letters {
            rows.push(ContactRow::Header(letter));
            let group: Vec<_> = NAMES.iter().filter(|n| initial(n) == letter).collect();
            for (i, &&name) in group.iter().enumerate() {
                rows.push(ContactRow::Name {
                    name,
                    last: i + 1 == group.len(),
                });
            }
        }
        self.rows = Rc::new(rows);
    }

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let rows = self.rows.clone();
        // The headers' rows stick.
        let headers: Vec<usize> = (0..rows.len())
            .filter(|&i| matches!(rows[i], ContactRow::Header(_)))
            .collect();
        view().flex_1().child(
            virtual_list(rows.len(), move |i, _| match &rows[i] {
                ContactRow::Header(letter) => view().bg(separator()).px_4().py(px(6.0)).child(
                    text(letter.to_string())
                        .font_weight(FontWeight::BOLD)
                        .text_color(colors::muted()),
                ),
                ContactRow::Name { name, last } => view()
                    .px_4()
                    .py_3()
                    .when(!last, |v| v.border_b_1().border_color(separator()))
                    .child(text(*name).text_size(px(17.0))),
            })
            .sticky_header_indices(headers)
            .estimated_item_size(44.0)
            .flex_1()
            .bg(colors::surface()),
        )
    }
}

/// A message of the chat.
#[derive(Debug, Clone)]
struct Message {
    id: u32,
    text: String,
    mine: bool,
}

/// An inverted list: starts at the newest message, keeps its place as
/// older ones load.
#[derive(Debug, Default)]
struct Chat {
    /// Newest first.
    messages: Rc<Vec<Message>>,
    next: u32,
    draft: String,
}

impl Chat {
    fn message(&mut self, text: String, mine: bool) -> Message {
        let id = self.next;
        self.next += 1;
        Message { id, text, mine }
    }

    fn send(&mut self, cx: &mut Context<'_, Self>) {
        if self.draft.trim().is_empty() {
            return;
        }
        let text = std::mem::take(&mut self.draft);
        let message = self.message(text, true);
        Rc::make_mut(&mut self.messages).insert(0, message);
        cx.notify();
    }
}

impl Component for Chat {
    const NAME: &'static str = "gallery.lists.chat";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        let messages = (0..30)
            .map(|i| self.message(format!("Message {}", 30 - i), i % 3 == 0))
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
            let older: Vec<_> = (0..20)
                .map(|i| this.message(format!("Older {}", count + i + 1), i % 4 == 0))
                .collect();
            Rc::make_mut(&mut this.messages).extend(older);
            cx.notify();
        });
        let list = virtual_list(messages.len(), move |i, _| {
            let m = &messages[i];
            view()
                .map(|v| if m.mine { v.self_end() } else { v.self_start() })
                .max_w(relative(0.75))
                .px_3()
                .py_2()
                .rounded(px(14.0))
                .bg(if m.mine {
                    colors::accent()
                } else {
                    colors::surface()
                })
                .child(
                    text(m.text.clone())
                        .text_color(if m.mine {
                            rgb(0xffffff)
                        } else {
                            colors::text()
                        })
                        .text_size(px(16.0)),
                )
        })
        .keys(self.messages.iter().map(|m| m.id))
        .inverted()
        .estimated_item_size(44.0)
        .content_padding(12.0)
        .row_gap(6.0)
        .flex_1()
        .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
        .on_end_reached(load_older)
        .end_reached_threshold(0.5);
        let bar = view()
            .flex_row()
            .gap_2()
            .p_2()
            .pb(px(8.0 + bottom))
            .bg(colors::surface())
            .child(
                input()
                    .value(self.draft.clone())
                    .placeholder("Message")
                    .return_key_type(ReturnKeyType::Send)
                    .blur_on_submit(false)
                    .flex_1()
                    .text_size(px(16.0))
                    .border_1()
                    .border_color(colors::border())
                    .rounded(px(18.0))
                    .px_3()
                    .py_2()
                    .on_change_text(cx.listener(|this: &mut Self, text: &str, cx| {
                        this.draft = text.to_owned();
                        cx.notify();
                    }))
                    .on_submit_editing(cx.listener(|this: &mut Self, _: &str, cx| this.send(cx))),
            )
            .child(
                text("Send")
                    .self_center()
                    .text_color(colors::accent())
                    .text_size(px(16.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .on_press(cx.listener(|this: &mut Self, _, cx| this.send(cx))),
            );
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps::default())
                .flex_1()
                .child(list)
                .child(bar),
        )
    }
}

/// Horizontal scrolling, pull to refresh on a plain scroll view, and an
/// empty list.
#[derive(Debug, Default)]
struct More {
    refreshing: bool,
    refreshed: u32,
    items: Rc<Vec<usize>>,
    /// The refresh in progress (dropping it cancels it).
    refresh: Option<Task<()>>,
}

impl Component for More {
    const NAME: &'static str = "gallery.lists.more";
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let heading = |label: &'static str| text(label).mx_3().font_weight(FontWeight::SEMIBOLD);
        let chips = [
            "Today",
            "Yesterday",
            "This week",
            "This month",
            "This year",
            "All time",
            "Custom",
        ]
        .map(|label| {
            text(label)
                .px(px(14.0))
                .py_2()
                .rounded(px(16.0))
                .overflow_hidden()
                .bg(colors::surface())
        });
        let cards = virtual_list(1000, |i, _| {
            view()
                .size(px(120.0))
                .rounded(px(12.0))
                .bg(rgb(TINTS[i % TINTS.len()]))
                .items_center()
                .justify_center()
                .child(
                    text(i.to_string())
                        .text_size(px(20.0))
                        .font_weight(FontWeight::BOLD),
                )
        })
        .horizontal()
        .estimated_item_size(130.0)
        .content_padding_x(12.0)
        .row_gap(10.0)
        .h(px(120.0));
        let items = self.items.clone();
        let list = virtual_list(items.len(), move |i, _| {
            text(format!("Item {}", items[i])).p_3()
        })
        .estimated_item_size(44.0)
        .empty(
            text("Nothing here yet: tap Add")
                .p_6()
                .text_center()
                .text_color(colors::muted()),
        )
        .separator(|| view().h(px(1.0)).bg(separator()))
        .h(px(180.0))
        .mx_3()
        .rounded(px(10.0))
        .bg(colors::surface());
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
                        .py_3()
                        .gap_4()
                        .child(
                            text(format!(
                                "Pull down to refresh (refreshed {} times)",
                                self.refreshed
                            ))
                            .mx_3()
                            .text_color(colors::muted()),
                        )
                        .child(heading("A horizontal scroll view"))
                        .child(
                            scroll()
                                .horizontal()
                                .shows_indicator(false)
                                .child(view().flex_row().gap_2().px_3().children(chips)),
                        )
                        .child(heading("A horizontal list of 1000 recycled cards"))
                        .child(cards)
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .mx_3()
                                .child(
                                    text(format!("An empty list ({} items)", self.items.len()))
                                        .flex_1()
                                        .font_weight(FontWeight::SEMIBOLD),
                                )
                                .child(
                                    link(
                                        "Add",
                                        cx.listener(|this: &mut Self, _, cx| {
                                            let items = Rc::make_mut(&mut this.items);
                                            items.push(items.len() + 1);
                                            cx.notify();
                                        }),
                                    )
                                    .mr_4(),
                                )
                                .child(link(
                                    "Clear",
                                    cx.listener(|this: &mut Self, _, cx| {
                                        this.items = Rc::default();
                                        cx.notify();
                                    }),
                                )),
                        )
                        .child(list),
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

/// The demo's page: the tabs and the selected one's list.
#[derive(Debug, Default)]
pub(crate) struct Lists {
    tab: Tab,
}

impl Component for Lists {
    const NAME: &'static str = "gallery.lists";
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let tabs = [
            (Tab::Grid, "Grid"),
            (Tab::Sections, "Sections"),
            (Tab::Chat, "Chat"),
            (Tab::More, "More"),
        ]
        .map(|(tab, label)| {
            text(label)
                .flex_1()
                .text_center()
                .py(px(6.0))
                .rounded(px(8.0))
                .overflow_hidden()
                .when(self.tab == tab, |t| t.bg(colors::surface()))
                .font_weight(FontWeight::SEMIBOLD)
                .on_press(cx.listener(move |this: &mut Self, _, cx| {
                    this.tab = tab;
                    cx.notify();
                }))
        });
        let content = match self.tab {
            Tab::Grid => Grid::element(()).key("grid").into_element(),
            Tab::Sections => Contacts::element(()).key("sections").into_element(),
            Tab::Chat => Chat::element(()).key("chat").into_element(),
            Tab::More => More::element(()).key("more").into_element(),
        };
        view()
            .flex_1()
            .bg(colors::background())
            .child(
                view()
                    .flex_row()
                    .m_3()
                    .p(px(3.0))
                    .rounded(px(10.0))
                    .bg(rgb(0xe1e4e8))
                    .children(tabs),
            )
            .child(content)
    }
}
