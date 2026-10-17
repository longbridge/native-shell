//! Form: many inputs in a scroll view (the focused one stays above the
//! keyboard natively, return moves to the next one, dragging the form can
//! hide the keyboard), selection and keys, a sheet of inputs and a message
//! bar riding on the keyboard (as the TSX gallery's `demos/Form.tsx`).

use nsh_core::{
    Size,
    ui::{
        AnyElement, AutoCapitalize, AutoFill, KeyboardDismissMode, ReturnKeyType, Text, View,
        prelude::*,
    },
};
use nsh_ui_mobile::{
    KeyboardAvoidingView, KeyboardAvoidingViewProps, SafeAreaView, SafeAreaViewProps,
};

use crate::{
    nav,
    ui::legacy::{button, colors, page, section},
};

/// The green of success texts and the message bar's button.
fn green() -> Color {
    rgb(0x1a7f37)
}

/// An input in the form's look.
fn field_input() -> Input {
    input()
        .text_size(px(16.0))
        .text_color(colors::text())
        .bg(colors::surface())
        .border_1()
        .border_color(colors::border())
        .rounded(px(10.0))
        .px(px(12.0))
        .py(px(10.0))
}

/// A small muted caption.
fn caption(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_size(px(13.0))
        .text_color(colors::muted())
}

/// A labelled input.
fn field(label: &str, input: Input) -> AnyElement {
    view()
        .gap_1()
        .child(caption(label.to_owned()))
        .child(input)
        .into_element()
}

/// How dragging the form treats the keyboard, with its label.
const MODES: [(KeyboardDismissMode, &str); 3] = [
    (KeyboardDismissMode::None, "none"),
    (KeyboardDismissMode::OnDrag, "on-drag"),
    (KeyboardDismissMode::Interactive, "interactive"),
];

/// How many inputs return moves through.
const CHAIN: usize = 15;

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Form {
    name: String,
    email: String,
    shout: String,
    notes: String,
    mode: KeyboardDismissMode,
    submitted: String,
    sheet: bool,
    /// The inputs in return-key order.
    refs: Vec<ElementRef>,
}

impl Form {
    /// Input `i` of the return-key order: return focuses the next one.
    fn chained(&self, i: usize, input: Input, cx: &RenderCx<'_, Self>) -> Input {
        input
            .element_ref(&self.refs[i])
            .return_key_type(ReturnKeyType::Next)
            .on_submit_editing(cx.listener(move |this: &mut Self, _: &str, _| {
                if let Some(next) = this.refs.get(i + 1) {
                    next.focus();
                }
            }))
    }

    /// The keyboard's height and the dismiss modes.
    fn toolbar(&self, cx: &RenderCx<'_, Self>) -> View {
        let chips = MODES.map(|(mode, label)| {
            let on = self.mode == mode;
            text(label)
                .text_size(px(13.0))
                .px(px(8.0))
                .py(px(4.0))
                .rounded(px(6.0))
                .overflow_hidden()
                .text_color(if on { rgb(0xffffff) } else { colors::text() })
                .bg(if on { colors::accent() } else { rgb(0xe1e4e8) })
                .on_press(cx.listener(move |this: &mut Self, _, cx| {
                    this.mode = mode;
                    cx.notify();
                }))
        });
        view()
            .flex_row()
            .items_center()
            .gap_2()
            .child(
                text(format!("Keyboard {} pt", cx.keyboard_height().round()))
                    .flex_1()
                    .text_color(colors::muted()),
            )
            .children(chips)
    }

    fn account(&self, cx: &RenderCx<'_, Self>) -> Vec<AnyElement> {
        vec![
            field(
                "Name",
                self.chained(0, field_input(), cx)
                    .value(self.name.clone())
                    .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                        this.name = v.to_owned();
                        cx.notify();
                    }))
                    .auto_capitalize(AutoCapitalize::Words)
                    .auto_fill(AutoFill::Name)
                    .placeholder("Ada Lovelace"),
            ),
            field(
                "E-mail",
                self.chained(1, field_input(), cx)
                    .value(self.email.clone())
                    .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                        this.email = v.to_owned();
                        cx.notify();
                    }))
                    .keyboard_type(KeyboardType::Email)
                    .auto_capitalize(AutoCapitalize::None)
                    .auto_correct(false)
                    .auto_fill(AutoFill::Email)
                    .placeholder("ada@example.com"),
            ),
            field(
                "Password",
                self.chained(2, field_input(), cx)
                    .secure(true)
                    .auto_fill(AutoFill::NewPassword)
                    .placeholder("At least 8 characters"),
            ),
            field(
                "Phone",
                self.chained(3, field_input(), cx)
                    .keyboard_type(KeyboardType::Phone)
                    .auto_fill(AutoFill::Tel)
                    .placeholder("+1 555 0100"),
            ),
            field(
                "Website",
                self.chained(4, field_input(), cx)
                    .keyboard_type(KeyboardType::Url)
                    .auto_capitalize(AutoCapitalize::None)
                    .auto_correct(false)
                    .placeholder("https://"),
            ),
        ]
    }

    fn numbers(&self, cx: &RenderCx<'_, Self>) -> Vec<AnyElement> {
        vec![
            field(
                "Age (number pad, 3 digits)",
                self.chained(5, field_input(), cx)
                    .keyboard_type(KeyboardType::Number)
                    .max_length(3),
            ),
            field(
                "Amount (decimal pad)",
                self.chained(6, field_input(), cx)
                    .keyboard_type(KeyboardType::Decimal)
                    .placeholder("0.00"),
            ),
            field(
                "Code (6 digits)",
                self.chained(7, field_input(), cx)
                    .keyboard_type(KeyboardType::Number)
                    .max_length(6)
                    .letter_spacing(8.0)
                    .text_size(px(22.0))
                    .text_center(),
            ),
        ]
    }

    fn address(&self, cx: &RenderCx<'_, Self>) -> Vec<AnyElement> {
        vec![
            field(
                "Street",
                self.chained(8, field_input(), cx)
                    .placeholder("12 Analytical Way"),
            ),
            field(
                "City",
                self.chained(9, field_input(), cx).placeholder("London"),
            ),
            field(
                "Postal code",
                self.chained(10, field_input(), cx)
                    .auto_capitalize(AutoCapitalize::Characters)
                    .placeholder("SW1A 1AA"),
            ),
            field(
                "Country",
                self.chained(11, field_input(), cx)
                    .placeholder("United Kingdom"),
            ),
        ]
    }

    fn texts(&self, cx: &RenderCx<'_, Self>) -> Vec<AnyElement> {
        let notes = self.notes.encode_utf16().count();
        vec![
            field(
                "Controlled: uppercased as you type",
                self.chained(12, field_input(), cx)
                    .value(self.shout.clone())
                    .on_change_text(cx.listener(|this: &mut Self, t: &str, cx| {
                        this.shout = t.to_uppercase();
                        cx.notify();
                    })),
            ),
            field(
                "Read only",
                field_input()
                    .return_key_type(ReturnKeyType::Done)
                    .value("Not editable")
                    .editable(false)
                    .text_color(colors::muted())
                    .bg(rgb(0xf6f8fa)),
            ),
            field(
                "Custom colors",
                self.chained(13, field_input(), cx)
                    .placeholder("Placeholder in purple")
                    .placeholder_color(rgb(0x8250df))
                    .selection_color(rgb(0xbf3989)),
            ),
            view()
                .gap_1()
                .child(caption("Notes (multiline, grows, 280 max)"))
                .child(
                    field_input()
                        .element_ref(&self.refs[14])
                        .min_h(px(100.0))
                        .multiline(true)
                        .max_length(280)
                        .value(self.notes.clone())
                        .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                            this.notes = v.to_owned();
                            cx.notify();
                        }))
                        .placeholder("Return adds a line"),
                )
                .child(
                    text(format!("{notes}/280"))
                        .text_size(px(12.0))
                        .text_color(colors::muted())
                        .text_right(),
                )
                .into_element(),
            AutoGrow::element(()).into_element(),
            field(
                "caretHidden",
                field_input()
                    .return_key_type(ReturnKeyType::Done)
                    .caret_hidden(true)
                    .default_value("No caret here"),
            ),
        ]
    }

    fn actions(&self, cx: &RenderCx<'_, Self>) -> View {
        view()
            .gap(px(10.0))
            .child(button("Submit", colors::accent()).on_press(cx.listener(
                |this: &mut Self, _, cx| {
                    let name = if this.name.is_empty() {
                        "Nobody"
                    } else {
                        &this.name
                    };
                    let email = if this.email.is_empty() {
                        "no e-mail"
                    } else {
                        &this.email
                    };
                    this.submitted = format!("{name} <{email}>");
                    cx.notify();
                },
            )))
            .child(text(self.submitted.clone()).text_color(green()))
            .child(
                button("Hide the keyboard", colors::gray())
                    .on_press(cx.listener(|_: &mut Self, _, cx| cx.dismiss_keyboard())),
            )
            .child(
                button("Inputs in a sheet", colors::purple()).on_press(cx.listener(
                    |this: &mut Self, _, cx| {
                        this.sheet = true;
                        cx.notify();
                    },
                )),
            )
            .child(
                button("A message bar at the bottom", green()).on_press(cx.listener(
                    |_: &mut Self, _, cx| {
                        nav::push(cx, "message-bar", |_| {
                            screen().title("Message bar").child(MessageBar::element(()))
                        });
                    },
                )),
            )
    }

    /// The sheet of inputs, while it shows.
    fn sheet(&self, cx: &RenderCx<'_, Self>) -> Option<Modal> {
        if !self.sheet {
            return None;
        }
        let close = || {
            cx.listener(|this: &mut Self, _, cx| {
                this.sheet = false;
                cx.notify();
            })
        };
        let fields = [
            "First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh",
        ]
        .map(|label| {
            field(
                label,
                field_input()
                    .return_key_type(ReturnKeyType::Done)
                    .placeholder(format!("{label} field")),
            )
        });
        Some(
            modal()
                .presentation(ModalPresentation::PageSheet)
                .on_request_close(close())
                .child(
                    SafeAreaView::element(SafeAreaViewProps::default())
                        .flex_1()
                        .bg(colors::background())
                        .child(
                            scroll()
                                .flex_1()
                                .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
                                .child(
                                    view()
                                        .p_4()
                                        .gap_3()
                                        .child(
                                            text("A sheet")
                                                .text_size(px(22.0))
                                                .font_weight(FontWeight::BOLD),
                                        )
                                        .child(button("Close", colors::accent()).on_press(close()))
                                        .children(fields),
                                ),
                        ),
                ),
        )
    }
}

impl Component for Form {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.mode = KeyboardDismissMode::OnDrag;
        self.refs = (0..CHAIN).map(|_| cx.element_ref()).collect();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .child(
                page([
                    self.toolbar(cx).into_element(),
                    section("Account", self.account(cx)).into_element(),
                    section("Numbers", self.numbers(cx)).into_element(),
                    section(
                        "Address (far down: the form scrolls to keep the input visible)",
                        self.address(cx),
                    )
                    .into_element(),
                    section(
                        "Selection and keys",
                        [SelectionDemo::element(()).into_element()],
                    )
                    .into_element(),
                    section("Text", self.texts(cx)).into_element(),
                    self.actions(cx).into_element(),
                ])
                .keyboard_dismiss_mode(self.mode),
            )
            .children(self.sheet(cx))
    }
}

/// A multiline input that grows with its text (`on_content_size_change`),
/// up to 5 lines.
#[derive(Debug, Default)]
struct AutoGrow {
    height: f32,
    size: String,
}

impl Component for AutoGrow {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .gap_1()
            .child(caption(format!(
                "Grows with its text (onContentSizeChange): {}",
                self.size
            )))
            .child(
                field_input()
                    .multiline(true)
                    .placeholder("Type several lines")
                    .on_content_size_change(cx.listener(|this: &mut Self, e: &Size, cx| {
                        this.size = format!("{}×{}", e.width.round(), e.height.round());
                        this.height = e.height.min(110.0);
                        cx.notify();
                    }))
                    .h(px(self.height.max(20.0) + 20.0)),
            )
    }
}

/// `start..end` of `text` in UTF-16 units (as inputs count).
fn utf16_slice(text: &str, start: u32, end: u32) -> String {
    let units: Vec<u16> = text.encode_utf16().collect();
    let end = (end as usize).min(units.len());
    let start = (start as usize).min(end);
    String::from_utf16_lossy(&units[start..end])
}

/// The selection and the keys of an input, and selecting from code: the
/// controlled `selection` ("Select all") and the ref's `set_selection`.
#[derive(Debug, Default)]
struct SelectionDemo {
    text: String,
    range: Selection,
    /// The selection the render sets (TS: the `selection` prop's signal).
    selection: Option<Selection>,
    /// The last keys pressed.
    keys: Vec<String>,
    input: Option<ElementRef>,
}

impl SelectionDemo {
    fn input(&self) -> &ElementRef {
        self.input.as_ref().expect("set up")
    }
}

impl Component for SelectionDemo {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.text = "Select part of this text".to_owned();
        self.input = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let Selection { start, end } = self.range;
        let keys = if self.keys.is_empty() {
            "–".to_owned()
        } else {
            self.keys.join(" ")
        };
        view()
            .gap_2()
            .child(
                field_input()
                    .element_ref(self.input())
                    .value(self.text.clone())
                    .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                        this.text = v.to_owned();
                        cx.notify();
                    }))
                    .on_selection_change(cx.listener(|this: &mut Self, s: &Selection, cx| {
                        this.range = *s;
                        // The input has it: "Select all" sets it again.
                        this.selection = None;
                        cx.notify();
                    }))
                    .on_key_press(cx.listener(|this: &mut Self, key: &str, cx| {
                        this.keys.push(key.to_owned());
                        let extra = this.keys.len().saturating_sub(8);
                        this.keys.drain(..extra);
                        cx.notify();
                    }))
                    .auto_fill(AutoFill::Off)
                    .when_some(self.selection, |input, s| input.selection(s.start, s.end)),
            )
            .child(caption(format!(
                "Selection {start}–{end}: \"{}\"",
                utf16_slice(&self.text, start, end)
            )))
            .child(caption(format!("Keys: {keys}")))
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(
                        view()
                            .flex_1()
                            .child(button("Select all", colors::accent()).on_press(cx.listener(
                                |this: &mut Self, _, cx| {
                                    let end = this.text.encode_utf16().count() as u32;
                                    this.selection = Some(Selection { start: 0, end });
                                    cx.notify();
                                },
                            ))),
                    )
                    .child(view().flex_1().child(
                        button("Caret to start", colors::gray()).on_press(
                            cx.listener(|this: &mut Self, _, _| this.input().set_selection(0, 0)),
                        ),
                    )),
            )
            .child(field(
                "One-time code (autofill from messages)",
                field_input()
                    .return_key_type(ReturnKeyType::Done)
                    .keyboard_type(KeyboardType::Number)
                    .auto_fill(AutoFill::OneTimeCode)
                    .max_length(6),
            ))
    }
}

/// An input pinned to the bottom (outside any scroll view): it rides on the
/// keyboard.
#[derive(Debug, Default)]
struct MessageBar {
    messages: Vec<String>,
    draft: String,
}

impl MessageBar {
    fn send(&mut self, cx: &mut Context<'_, Self>) {
        if self.draft.trim().is_empty() {
            return;
        }
        self.messages.push(std::mem::take(&mut self.draft));
        cx.notify();
    }
}

impl Component for MessageBar {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let insets = cx.insets();
        // Above the home indicator, unless the keyboard covers it.
        let below = if cx.keyboard_height() > 0.0 {
            0.0
        } else {
            insets.bottom
        };
        let bubbles = self.messages.iter().map(|m| {
            text(m.clone())
                .self_end()
                .text_color(rgb(0xffffff))
                .bg(colors::accent())
                .rounded(px(14.0))
                .overflow_hidden()
                .px(px(12.0))
                .py(px(8.0))
        });
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps::default())
                .flex_1()
                .bg(colors::background())
                .child(
                    scroll()
                        .flex_1()
                        .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
                        .child(
                            view()
                                .p_4()
                                .gap_2()
                                .child(text(
                                    "The bar below sits outside the scroll view; KeyboardAvoidingView lifts it with the keyboard.",
                                ).text_color(colors::muted()))
                                .children(bubbles),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .gap_2()
                        .p_2()
                        .pb(px(8.0 + below))
                        .bg(colors::surface())
                        .child(
                            field_input()
                                .value(self.draft.clone())
                                .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                    this.draft = v.to_owned();
                                    cx.notify();
                                }))
                                .on_submit_editing(
                                    cx.listener(|this: &mut Self, _: &str, cx| this.send(cx)),
                                )
                                .blur_on_submit(false)
                                .placeholder("Message")
                                .return_key_type(ReturnKeyType::Send)
                                .flex_1()
                                .rounded(px(18.0))
                                .py(px(8.0)),
                        )
                        .child(
                            text("Send")
                                .self_center()
                                .text_color(colors::accent())
                                .text_size(px(16.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .on_press(cx.listener(|this: &mut Self, _, cx| this.send(cx))),
                        ),
                ),
        )
    }
}
