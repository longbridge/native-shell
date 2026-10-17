//! Form: an account form in a scroll view (the focused input stays above
//! the keyboard natively, return moves to the next one, dragging the form
//! can hide the keyboard; leaving a field or submitting checks it), fields
//! whose border glows while focused and that shake on an error, selection
//! and keys, a sheet of inputs and a message bar riding on the keyboard
//! (as the TSX gallery's `demos/Form.tsx`).

use nsh_core::{
    Size,
    ui::{
        AutoCapitalize, AutoFill, KeyboardDismissMode, LayoutAnimation, ReturnKeyType, View,
        prelude::*,
    },
};
use nsh_ui_mobile::{
    KeyboardAvoidingView, KeyboardAvoidingViewProps, SafeAreaView, SafeAreaViewProps,
};

use crate::{
    nav,
    ui::{
        Badge, Button, ButtonVariant, Card, Finish, Finished, IconName, IconTile, Inset, MONO,
        Meter, PressProps, RoundButton, Section, Segment, SegmentedEvent, Tone, appear, caption,
        colors, header, icon, press_with, radius, segmented, tints,
    },
};

/// What a [`field`] animates: the focus ring and the shake of an error.
#[derive(Debug)]
struct FieldFx {
    /// 0 blurred, 1 focused: the ring's opacity.
    focus: NativeValue,
    /// The field's horizontal offset while it shakes.
    shake: NativeValue,
    /// It shows an error now.
    shown: bool,
}

impl FieldFx {
    fn new<C: Component>(cx: &mut Context<'_, C>) -> Self {
        Self {
            focus: cx.native_value(0.0),
            shake: cx.native_value(0.0),
            shown: false,
        }
    }

    /// The field shows an error (`error`) or not: a change slides the
    /// message in or out (layout animation), one coming shakes the field.
    fn error<C: Component>(&mut self, error: bool, cx: &mut Context<'_, C>) {
        if error == self.shown {
            return;
        }
        self.shown = error;
        cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
        if error {
            Animation::sequence(
                [9.0, -8.0, 6.0, -4.0, 2.0, 0.0].map(|x| {
                    Animation::timing(&self.shake, x, Timing::new(45.0, Easing::EaseInOut))
                }),
            )
            .start();
        }
    }
}

/// A labelled input (`api` names the prop it shows, `hint` is a plain
/// note): the border glows accent while focused (a ring fading in
/// natively), an error turns it red, slides a message in and shakes the
/// field ([`FieldFx::error`]).
struct Field<'a> {
    fx: &'a FieldFx,
    label: SharedString,
    api: Option<&'static str>,
    hint: Option<SharedString>,
    error: Option<&'static str>,
    prefix: Option<&'static str>,
    multiline: bool,
    editable: bool,
}

/// A [`Field`] labelled `label`, animated by `fx`.
fn field(fx: &FieldFx, label: impl Into<SharedString>) -> Field<'_> {
    Field {
        fx,
        label: label.into(),
        api: None,
        hint: None,
        error: None,
        prefix: None,
        multiline: false,
        editable: true,
    }
}

impl Field<'_> {
    fn api(mut self, api: &'static str) -> Self {
        self.api = Some(api);
        self
    }

    fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    fn error(mut self, error: Option<&'static str>) -> Self {
        self.error = error;
        self
    }

    fn prefix(mut self, prefix: &'static str) -> Self {
        self.prefix = Some(prefix);
        self
    }

    fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }

    fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }

    /// The field, its input made by `input` from the styled one (values,
    /// listeners, refs, styles of its own); `on_blur` runs when it loses
    /// focus, after the ring fades.
    fn render<C: Component>(
        self,
        cx: &RenderCx<'_, C>,
        on_blur: Option<fn(&mut C, &mut Context<'_, C>)>,
        input: impl FnOnce(Input) -> Input,
    ) -> View {
        let (focus_in, focus_out) = (self.fx.focus.clone(), self.fx.focus.clone());
        let base = nsh_core::ui::input()
            .placeholder_color(cx.color(colors::FAINT))
            .selection_color(cx.color(colors::ACCENT))
            .return_key_type(ReturnKeyType::Done)
            .multiline(self.multiline)
            .editable(self.editable)
            .on_focus(cx.listener(move |_: &mut C, _: &(), _| {
                focus_in.timing(1.0, 160.0, Easing::EaseInOut);
            }))
            .on_blur(cx.listener(move |this: &mut C, _: &(), cx| {
                focus_out.timing(0.0, 220.0, Easing::EaseInOut);
                if let Some(on_blur) = on_blur {
                    on_blur(this, cx);
                }
            }))
            .flex_1()
            .px(px(14.0))
            .text_size(px(15.0))
            .text_color(if self.editable {
                colors::TEXT
            } else {
                colors::MUTED
            })
            .map(|i| {
                if self.multiline {
                    i.min_h(px(46.0)).pt(px(13.0)).pb(px(13.0))
                } else {
                    i.h(px(46.0))
                }
            });
        let error = self.error;
        view()
            .gap(px(6.0))
            .bind_translate_x(&self.fx.shake)
            .child(
                view()
                    .flex_row()
                    .items_baseline()
                    .gap_2()
                    .child(
                        text(self.label)
                            .flex_1()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::MUTED),
                    )
                    .children(self.api.map(|api| {
                        text(api)
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .font_family(MONO)
                    }))
                    .children(self.hint.map(|hint| {
                        text(hint)
                            .text_size(px(13.0))
                            .text_color(colors::FAINT)
                            .tabular_nums()
                    })),
            )
            .child(
                view()
                    .flex_row()
                    .map(|v| {
                        if self.multiline {
                            v.items_start()
                        } else {
                            v.items_center()
                        }
                    })
                    .rounded(px(radius::CONTROL))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(if error.is_some() {
                        colors::DANGER
                    } else if self.editable {
                        colors::BORDER
                    } else {
                        colors::LINE
                    })
                    .bg(colors::INSET)
                    // Recessed while editable; a read-only field lies flat.
                    .when(self.editable, |v| v.finish(Finish::Inset))
                    .children(self.prefix.map(|prefix| {
                        text(prefix)
                            .pl(px(14.0))
                            .text_size(px(15.0))
                            .text_color(colors::MUTED)
                    }))
                    .child(input(base))
                    .child(
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .top(px(-1.0))
                            .left(px(-1.0))
                            .right(px(-1.0))
                            .bottom(px(-1.0))
                            .rounded(px(radius::CONTROL))
                            .border_curve(BorderCurve::Continuous)
                            .border(1.5)
                            .border_color(if error.is_some() {
                                colors::DANGER
                            } else {
                                colors::ACCENT
                            })
                            .bind_opacity(&self.fx.focus),
                    ),
            )
            .children(error.map(|error| {
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(7.0))
                    .child(view().size(px(6.0)).rounded(px(3.0)).bg(colors::DANGER))
                    .child(
                        text(error)
                            .flex_1()
                            .text_size(px(13.0))
                            .text_color(colors::DANGER),
                    )
            }))
    }
}

/// Two items side by side, equal widths.
fn pair(a: impl IntoElement, b: impl IntoElement) -> View {
    view()
        .flex_row()
        .gap_2()
        .child(view().flex_1().child(a))
        .child(view().flex_1().child(b))
}

/// A row that opens something: an icon, a title, the API it shows.
fn link_row(
    icon_name: IconName,
    title: &'static str,
    note: &'static str,
    first: bool,
    on_press: Listener<()>,
) -> impl IntoElement {
    press_with(
        PressProps {
            scale: 0.98.into(),
            label: Some(title.into()).into(),
            ..PressProps::default()
        },
        on_press,
    )
    .child(
        view()
            .min_h(px(60.0))
            .flex_row()
            .items_center()
            .gap_3()
            .px(px(10.0))
            .py_2()
            .when(!first, |v| v.border_t_1().border_t_color(colors::LINE))
            .child(IconTile::new(icon_name).size(34.0))
            .child(
                view()
                    .flex_1()
                    .gap(px(2.0))
                    .child(
                        text(title)
                            .text_size(px(15.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        text(note)
                            .number_of_lines(1)
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .font_family(MONO),
                    ),
            )
            .child(icon(IconName::Chevron, 14.0, colors::FAINT)),
    )
}

/// How dragging the form treats the keyboard: its segment and its name.
const MODES: [(KeyboardDismissMode, &str, &str); 3] = [
    (KeyboardDismissMode::None, "None", "none"),
    (KeyboardDismissMode::OnDrag, "On drag", "on-drag"),
    (
        KeyboardDismissMode::Interactive,
        "Interactive",
        "interactive",
    ),
];

/// How many inputs return moves through.
const CHAIN: usize = 15;

/// The form's fields, by their [`FieldFx`].
mod f {
    pub(super) const NAME: usize = 0;
    pub(super) const EMAIL: usize = 1;
    pub(super) const PASSWORD: usize = 2;
    pub(super) const PHONE: usize = 3;
    pub(super) const WEBSITE: usize = 4;
    pub(super) const AGE: usize = 5;
    pub(super) const AMOUNT: usize = 6;
    pub(super) const OFFSET: usize = 7;
    pub(super) const CODE: usize = 8;
    pub(super) const STREET: usize = 9;
    pub(super) const CITY: usize = 10;
    pub(super) const POSTAL: usize = 11;
    pub(super) const COUNTRY: usize = 12;
    pub(super) const SHOUT: usize = 13;
    pub(super) const READ_ONLY: usize = 14;
    pub(super) const COLORS: usize = 15;
    pub(super) const NOTES: usize = 16;
    pub(super) const CARET: usize = 17;
    /// The sheet's seven fields follow.
    pub(super) const SHEET: usize = 18;
    pub(super) const COUNT: usize = SHEET + 7;
}

/// The sheet's fields.
const SHEET_FIELDS: [&str; 7] = [
    "First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh",
];

/// `/^[^@\s]+@[^@\s]+\.[^@\s]+$/`.
fn is_email(s: &str) -> bool {
    let Some((local, domain)) = s.split_once('@') else {
        return false;
    };
    let plain = |part: &str| {
        !part.is_empty() && !part.contains(['@']) && !part.contains(char::is_whitespace)
    };
    plain(local)
        && plain(domain)
        && domain
            .char_indices()
            .any(|(i, c)| c == '.' && i > 0 && i + 1 < domain.len())
}

/// 0–4: length, mixed case, a digit, a symbol.
fn strength(p: &str) -> usize {
    if p.is_empty() {
        return 0;
    }
    [
        p.encode_utf16().count() >= 8,
        p.chars().any(|c| c.is_ascii_lowercase()) && p.chars().any(|c| c.is_ascii_uppercase()),
        p.chars().any(|c| c.is_ascii_digit()),
        p.chars()
            .any(|c| !(c.is_ascii_alphanumeric() || c == '_' || c.is_whitespace())),
    ]
    .into_iter()
    .filter(|&b| b)
    .count()
}

const STRENGTH_LABEL: [&str; 5] = ["Empty", "Weak", "Fair", "Good", "Strong"];
const STRENGTH_TONE: [Tone; 5] = [
    Tone::Neutral,
    Tone::Danger,
    Tone::Warn,
    Tone::Info,
    Tone::Up,
];

/// The fields that were left (or submitted): only those show errors.
#[derive(Debug, Default)]
struct Touched {
    name: bool,
    email: bool,
    password: bool,
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Form {
    name: String,
    email: String,
    password: String,
    code: String,
    shout: String,
    notes: String,
    mode: KeyboardDismissMode,
    /// What submitting found: up (sent) or danger, and what it says.
    status: Option<(Tone, String)>,
    sheet: bool,
    touched: Touched,
    /// The inputs in return-key order.
    refs: Vec<ElementRef>,
    fx: Vec<FieldFx>,
}

impl Form {
    fn name_error(&self) -> Option<&'static str> {
        (self.touched.name && self.name.trim().is_empty()).then_some("Enter your name")
    }

    fn email_error(&self) -> Option<&'static str> {
        (self.touched.email && !is_email(&self.email)).then_some(if self.email.is_empty() {
            "Enter your e-mail"
        } else {
            "That address needs an @ and a domain"
        })
    }

    fn password_error(&self) -> Option<&'static str> {
        (self.touched.password && self.password.encode_utf16().count() < 8)
            .then_some("Use at least 8 characters")
    }

    /// The fields' errors may have come or gone: they animate.
    fn check(&mut self, cx: &mut Context<'_, Self>) {
        let errors = [
            (f::NAME, self.name_error().is_some()),
            (f::EMAIL, self.email_error().is_some()),
            (f::PASSWORD, self.password_error().is_some()),
        ];
        for (i, error) in errors {
            self.fx[i].error(error, cx);
        }
        cx.notify();
    }

    /// Input `i` of the return-key order: return focuses the next one.
    fn chained(&self, i: usize, input: Input, cx: &RenderCx<'_, Self>) -> Input {
        let input = input.element_ref(&self.refs[i]);
        if i + 1 == CHAIN {
            return input;
        }
        input
            .return_key_type(ReturnKeyType::Next)
            .on_submit_editing(cx.listener(move |this: &mut Self, _: &str, _| {
                if let Some(next) = this.refs.get(i + 1) {
                    next.focus();
                }
            }))
    }

    fn keyboard(&self, cx: &RenderCx<'_, Self>) -> Card {
        let height = cx.keyboard_height();
        let selected = MODES.iter().position(|m| m.0 == self.mode).unwrap_or(0);
        Card::new()
            .icon(IconName::Form)
            .title("Keyboard")
            .subtitle("keyboardDismissMode while dragging")
            .right(
                Badge::new(format!("{} pt", height.round())).tone(if height > 0.0 {
                    Tone::Accent
                } else {
                    Tone::Neutral
                }),
            )
            .child(
                segmented(MODES.map(|m| Segment::new(m.1)), selected as u32).on_event(cx.listener(
                    |this: &mut Self, e: &SegmentedEvent, cx| {
                        let SegmentedEvent::Change(i) = *e;
                        this.mode = MODES[i as usize].0;
                        cx.notify();
                    },
                )),
            )
            .child(caption(
                "Drag the form with the keyboard up",
                Some(MODES[selected].2),
            ))
    }

    fn account(&self, cx: &RenderCx<'_, Self>) -> Section {
        let score = strength(&self.password);
        Section::new("Account")
            .note("autoComplete")
            .child(
                field(&self.fx[f::NAME], "Name")
                    .error(self.name_error())
                    .render(
                        cx,
                        Some(|this: &mut Self, cx| {
                            this.touched.name = true;
                            this.check(cx);
                        }),
                        |i| {
                            self.chained(0, i, cx)
                                .value(self.name.clone())
                                .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                    this.name = v.to_owned();
                                    this.check(cx);
                                }))
                                .auto_capitalize(AutoCapitalize::Words)
                                .auto_fill(AutoFill::Name)
                                .placeholder("Ada Lovelace")
                        },
                    ),
            )
            .child(
                field(&self.fx[f::EMAIL], "E-mail")
                    .api("email-address")
                    .error(self.email_error())
                    .render(
                        cx,
                        Some(|this: &mut Self, cx| {
                            this.touched.email = true;
                            this.check(cx);
                        }),
                        |i| {
                            self.chained(1, i, cx)
                                .value(self.email.clone())
                                .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                    this.email = v.to_owned();
                                    this.check(cx);
                                }))
                                .keyboard_type(KeyboardType::Email)
                                .auto_capitalize(AutoCapitalize::None)
                                .auto_correct(false)
                                .auto_fill(AutoFill::Email)
                                .placeholder("ada@example.com")
                        },
                    ),
            )
            .child(
                view()
                    .gap_2()
                    .child(
                        field(&self.fx[f::PASSWORD], "Password")
                            .api("secureTextEntry")
                            .error(self.password_error())
                            .render(
                                cx,
                                Some(|this: &mut Self, cx| {
                                    this.touched.password = true;
                                    this.check(cx);
                                }),
                                |i| {
                                    self.chained(2, i, cx)
                                        .value(self.password.clone())
                                        .on_change_text(cx.listener(
                                            |this: &mut Self, v: &str, cx| {
                                                if strength(v) != strength(&this.password) {
                                                    cx.animate_layout(
                                                        LayoutAnimation::EASE_IN_EASE_OUT,
                                                    );
                                                }
                                                this.password = v.to_owned();
                                                this.check(cx);
                                            },
                                        ))
                                        .secure(true)
                                        .auto_fill(AutoFill::NewPassword)
                                        .placeholder("At least 8 characters")
                                },
                            ),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .child(
                                view().flex_1().child(
                                    Meter::new(score as f32 / 4.0).tone(STRENGTH_TONE[score]),
                                ),
                            )
                            .child(
                                text(STRENGTH_LABEL[score])
                                    .w(px(52.0))
                                    .text_right()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(if score > 0 {
                                        STRENGTH_TONE[score].color()
                                    } else {
                                        colors::FAINT
                                    }),
                            ),
                    ),
            )
            .child(
                field(&self.fx[f::PHONE], "Phone")
                    .api("phone-pad")
                    .render(cx, None, |i| {
                        self.chained(3, i, cx)
                            .keyboard_type(KeyboardType::Phone)
                            .auto_fill(AutoFill::Tel)
                            .placeholder("+1 555 0100")
                    }),
            )
            .child(
                field(&self.fx[f::WEBSITE], "Website")
                    .api("url")
                    .render(cx, None, |i| {
                        self.chained(4, i, cx)
                            .keyboard_type(KeyboardType::Url)
                            .auto_capitalize(AutoCapitalize::None)
                            .auto_correct(false)
                            .placeholder("https://")
                    }),
            )
    }

    fn numbers(&self, cx: &RenderCx<'_, Self>) -> Section {
        Section::new("Numbers")
            .note("keyboardType")
            .child(pair(
                field(&self.fx[f::AGE], "Age")
                    .hint("3 digits")
                    .render(cx, None, |i| {
                        self.chained(5, i, cx)
                            .keyboard_type(KeyboardType::Number)
                            .max_length(3)
                            .placeholder("36")
                    }),
                field(&self.fx[f::AMOUNT], "Amount")
                    .api("decimal-pad")
                    .prefix("$")
                    .render(cx, None, |i| {
                        self.chained(6, i, cx)
                            .keyboard_type(KeyboardType::Decimal)
                            .placeholder("0.00")
                    }),
            ))
            .child(
                field(&self.fx[f::OFFSET], "Offset")
                    .api("numbers-and-punctuation")
                    .hint("dark keyboard (iOS)")
                    .render(cx, None, |i| {
                        i.keyboard_type(KeyboardType::NumbersAndPunctuation)
                            .keyboard_appearance(KeyboardAppearance::Dark)
                            .placeholder("-12.5")
                    }),
            )
            .child(
                field(&self.fx[f::CODE], "Verification code")
                    .hint(format!("{}/6", self.code.encode_utf16().count()))
                    .render(cx, None, |i| {
                        self.chained(7, i, cx)
                            .value(self.code.clone())
                            .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                this.code = v.to_owned();
                                cx.notify();
                            }))
                            .keyboard_type(KeyboardType::Number)
                            .max_length(6)
                            .placeholder("······")
                            .letter_spacing(10.0)
                            .text_size(px(22.0))
                            .font_weight(FontWeight::MEDIUM)
                            .tabular_nums()
                            .text_center()
                    }),
            )
    }

    fn address(&self, cx: &RenderCx<'_, Self>) -> Section {
        Section::new("Shipping address")
            .note("scrolls into view")
            .child(field(&self.fx[f::STREET], "Street").render(cx, None, |i| {
                self.chained(8, i, cx)
                    .auto_fill(AutoFill::StreetAddress)
                    .placeholder("12 Analytical Way")
            }))
            .child(pair(
                field(&self.fx[f::CITY], "City")
                    .render(cx, None, |i| self.chained(9, i, cx).placeholder("London")),
                field(&self.fx[f::POSTAL], "Postal code").render(cx, None, |i| {
                    self.chained(10, i, cx)
                        .auto_capitalize(AutoCapitalize::Characters)
                        .auto_fill(AutoFill::PostalCode)
                        .placeholder("SW1A 1AA")
                }),
            ))
            .child(
                field(&self.fx[f::COUNTRY], "Country").render(cx, None, |i| {
                    self.chained(11, i, cx).placeholder("United Kingdom")
                }),
            )
    }

    fn texts(&self, cx: &RenderCx<'_, Self>) -> Section {
        let notes = self.notes.encode_utf16().count();
        Section::new("Text")
            .note("Input props")
            .child(
                field(&self.fx[f::SHOUT], "Controlled")
                    .hint("Uppercased as you type")
                    .render(cx, None, |i| {
                        self.chained(12, i, cx)
                            .value(self.shout.clone())
                            .on_change_text(cx.listener(|this: &mut Self, t: &str, cx| {
                                this.shout = t.to_uppercase();
                                cx.notify();
                            }))
                            .placeholder("TYPE IN LOWER CASE")
                    }),
            )
            .child(
                field(&self.fx[f::READ_ONLY], "Read only")
                    .api("editable={false}")
                    .read_only()
                    .render(cx, None, |i| i.value("Not editable")),
            )
            .child(
                field(&self.fx[f::COLORS], "Custom colors")
                    .api("selectionColor")
                    .render(cx, None, |i| {
                        self.chained(13, i, cx)
                            .placeholder("A blue placeholder, an amber selection")
                            .placeholder_color(cx.color(colors::INFO))
                            .selection_color(cx.color(colors::WARN))
                    }),
            )
            .child(
                view()
                    .gap_2()
                    .child(
                        field(&self.fx[f::NOTES], "Notes")
                            .hint(format!("{notes}/280"))
                            .multiline()
                            .render(cx, None, |i| {
                                self.chained(14, i, cx)
                                    .max_length(280)
                                    .value(self.notes.clone())
                                    .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                        this.notes = v.to_owned();
                                        cx.notify();
                                    }))
                                    .placeholder("Return adds a line")
                                    .min_h(px(100.0))
                            }),
                    )
                    .child(Meter::new(notes as f32 / 280.0).tone(if notes > 250 {
                        Tone::Warn
                    } else {
                        Tone::Accent
                    })),
            )
            .child(AutoGrow::element(()))
            .child(
                field(&self.fx[f::CARET], "No caret")
                    .api("caretHidden")
                    .render(cx, None, |i| {
                        i.caret_hidden(true).default_value("No caret here")
                    }),
            )
    }

    fn submit_card(&self, cx: &RenderCx<'_, Self>) -> Card {
        let submit = cx.listener(|this: &mut Self, _, cx| {
            this.touched = Touched {
                name: true,
                email: true,
                password: true,
            };
            this.check(cx);
            let issues = [this.name_error(), this.email_error(), this.password_error()]
                .iter()
                .flatten()
                .count();
            cx.animate_layout(LayoutAnimation::EASE_IN_EASE_OUT);
            this.status = Some(if issues > 0 {
                (
                    Tone::Danger,
                    format!(
                        "{issues} {} attention",
                        if issues == 1 {
                            "field needs"
                        } else {
                            "fields need"
                        }
                    ),
                )
            } else {
                (
                    Tone::Up,
                    format!("Account created for {} <{}>", this.name, this.email),
                )
            });
            cx.notify();
        });
        let card = Card::new()
            .icon(IconName::Check)
            .title("Create the account")
            .subtitle("Submitting checks every field");
        let card = match &self.status {
            Some((tone, _)) => {
                card.right(Badge::new(if *tone == Tone::Up { "Sent" } else { "Check" }).tone(*tone))
            }
            None => card,
        };
        card.children(self.status.as_ref().map(|(tone, message)| {
            let up = *tone == Tone::Up;
            let fg = if up { colors::UP } else { colors::DANGER };
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .p_3()
                .rounded(px(radius::CONTROL))
                .bg(if up { tints::UP } else { tints::DANGER })
                .child(icon(
                    if up { IconName::Check } else { IconName::Bell },
                    16.0,
                    fg,
                ))
                .child(
                    text(message.clone())
                        .flex_1()
                        .text_size(px(14.0))
                        .line_height(px(19.0))
                        .text_color(fg),
                )
        }))
        .child(pair(
            Button::new("Submit", submit),
            Button::new(
                "Hide keyboard",
                cx.listener(|_: &mut Self, _, cx| cx.dismiss_keyboard()),
            )
            .variant(ButtonVariant::Secondary),
        ))
    }

    fn more(&self, cx: &RenderCx<'_, Self>) -> Card {
        Card::new()
            .title("More keyboards")
            .subtitle("Sheets and bars that ride on the keyboard")
            .flush()
            .child(
                Inset::new()
                    .padding(0.0)
                    .gap(0.0)
                    .child(link_row(
                        IconName::Components,
                        "Inputs in a sheet",
                        "presentationStyle=\"pageSheet\"",
                        true,
                        cx.listener(|this: &mut Self, _, cx| {
                            this.sheet = true;
                            cx.notify();
                        }),
                    ))
                    .child(link_row(
                        IconName::Text,
                        "A message bar at the bottom",
                        "KeyboardAvoidingView",
                        false,
                        cx.listener(|_: &mut Self, _, _| {
                            nav::push("message-bar", || {
                                header(screen().title("Message bar")).child(MessageBar::element(()))
                            });
                        }),
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
        let fields = SHEET_FIELDS.iter().enumerate().map(|(n, label)| {
            field(&self.fx[f::SHEET + n], *label)
                .render(cx, None, |i| i.placeholder(format!("{label} field")))
        });
        Some(
            modal()
                .presentation(ModalPresentation::PageSheet)
                .on_request_close(close())
                .child(
                    SafeAreaView::element(SafeAreaViewProps::default())
                        .flex_1()
                        .bg(colors::BACKGROUND)
                        .child(
                            scroll()
                                .flex_1()
                                .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
                                .child(
                                    view()
                                        .p_4()
                                        .pb(px(40.0))
                                        .gap_4()
                                        .child(
                                            view()
                                                .flex_row()
                                                .items_center()
                                                .gap_3()
                                                .pt_2()
                                                .pl_1()
                                                .child(
                                                    view()
                                                        .flex_1()
                                                        .gap(px(2.0))
                                                        .child(
                                                            text("A sheet")
                                                                .text_size(px(24.0))
                                                                .font_weight(FontWeight::BOLD)
                                                                .letter_spacing(-0.4)
                                                                .text_color(colors::TEXT),
                                                        )
                                                        .child(
                                                            text("Its inputs stay above the keyboard too")
                                                                .text_size(px(13.0))
                                                                .text_color(colors::MUTED),
                                                        ),
                                                )
                                                .child(
                                                    Button::new("Done", close())
                                                        .variant(ButtonVariant::Secondary),
                                                ),
                                        )
                                        .child(Inset::new().children(fields)),
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
        self.fx = (0..f::COUNT).map(|_| FieldFx::new(cx)).collect();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let cards = [
            self.keyboard(cx).into_element(),
            self.account(cx).into_element(),
            self.numbers(cx).into_element(),
            self.address(cx).into_element(),
            Section::new("Selection and keys")
                .note("onSelectionChange")
                .child(SelectionDemo::element(()))
                .into_element(),
            self.texts(cx).into_element(),
            self.submit_card(cx).into_element(),
            self.more(cx).into_element(),
        ];
        view()
            .flex_1()
            .child(
                scroll()
                    .flex_1()
                    .bg(colors::BACKGROUND)
                    .keyboard_dismiss_mode(self.mode)
                    .child(
                        view().p_4().pb(px(40.0)).gap_4().children(
                            cards
                                .into_iter()
                                .enumerate()
                                .map(|(i, card)| appear(i as u32).child(card)),
                        ),
                    ),
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
    fx: Option<FieldFx>,
}

impl Component for AutoGrow {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.size = "–".to_owned();
        self.fx = Some(FieldFx::new(cx));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        field(self.fx.as_ref().expect("set up"), "Grows with its text")
            .hint(self.size.clone())
            .multiline()
            .render(cx, None, |i| {
                i.placeholder("Type several lines")
                    .on_content_size_change(cx.listener(|this: &mut Self, e: &Size, cx| {
                        this.size = format!("{}×{}", e.width.round(), e.height.round());
                        this.height = e.height.min(110.0);
                        cx.notify();
                    }))
                    .h(px(self.height.max(20.0) + 26.0))
            })
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
    /// The text's field and the one-time code's.
    fx: Vec<FieldFx>,
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
        self.fx = vec![FieldFx::new(cx), FieldFx::new(cx)];
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let Selection { start, end } = self.range;
        let keys = view()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(6.0))
            .min_h(px(28.0))
            .map(|v| {
                if self.keys.is_empty() {
                    v.child(
                        text("Keys you press show here")
                            .text_size(px(13.0))
                            .text_color(colors::FAINT),
                    )
                } else {
                    v.children(self.keys.iter().map(|key| {
                        view()
                            .min_w(px(28.0))
                            .h(px(28.0))
                            .px_2()
                            .rounded(px(7.0))
                            .items_center()
                            .justify_center()
                            .bg(colors::RAISED)
                            .border_1()
                            .border_color(colors::BORDER)
                            .finish(Finish::Raised)
                            .child(
                                text(if key == " " {
                                    "␣".to_owned()
                                } else {
                                    key.clone()
                                })
                                .text_size(px(13.0))
                                .text_color(colors::TEXT),
                            )
                    }))
                }
            });
        view()
            .gap_3()
            .child(
                field(&self.fx[0], "Text")
                    .hint(format!("{start}–{end}"))
                    .render(cx, None, |i| {
                        i.element_ref(self.input())
                            .value(self.text.clone())
                            .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                                this.text = v.to_owned();
                                cx.notify();
                            }))
                            .on_selection_change(cx.listener(
                                |this: &mut Self, s: &Selection, cx| {
                                    this.range = *s;
                                    // The input has it: "Select all" sets it again.
                                    this.selection = None;
                                    cx.notify();
                                },
                            ))
                            .on_key_press(cx.listener(|this: &mut Self, key: &str, cx| {
                                this.keys.push(key.to_owned());
                                let extra = this.keys.len().saturating_sub(8);
                                this.keys.drain(..extra);
                                cx.notify();
                            }))
                            .auto_fill(AutoFill::Off)
                            .when_some(self.selection, |input, s| input.selection(s.start, s.end))
                    }),
            )
            .child(caption(
                "Selected",
                Some(format!("\"{}\"", utf16_slice(&self.text, start, end))),
            ))
            .child(keys)
            .child(pair(
                Button::new(
                    "Select all",
                    cx.listener(|this: &mut Self, _, cx| {
                        let end = this.text.encode_utf16().count() as u32;
                        this.selection = Some(Selection { start: 0, end });
                        cx.notify();
                    }),
                ),
                Button::new(
                    "Caret to start",
                    cx.listener(|this: &mut Self, _, _| this.input().set_selection(0, 0)),
                )
                .variant(ButtonVariant::Secondary),
            ))
            .child(
                field(&self.fx[1], "One-time code")
                    .api("one-time-code")
                    .render(cx, None, |i| {
                        i.keyboard_type(KeyboardType::Number)
                            .auto_fill(AutoFill::OneTimeCode)
                            .max_length(6)
                            .placeholder("123456")
                    }),
            )
    }
}

/// A message of the message bar's page: mine, and what it says.
type Message = (bool, String);

/// An input pinned to the bottom (outside any scroll view): it rides on the
/// keyboard.
#[derive(Debug, Default)]
struct MessageBar {
    messages: Vec<Message>,
    draft: String,
}

impl MessageBar {
    fn send(&mut self, cx: &mut Context<'_, Self>) {
        if self.draft.trim().is_empty() {
            return;
        }
        self.messages.push((true, std::mem::take(&mut self.draft)));
        cx.notify();
    }
}

impl Component for MessageBar {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.messages = vec![
            (
                false,
                "The bar below sits outside the scroll view.".to_owned(),
            ),
            (
                false,
                "KeyboardAvoidingView lifts it with the keyboard. Say something.".to_owned(),
            ),
        ];
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let insets = cx.insets();
        // Above the home indicator, unless the keyboard covers it.
        let below = if cx.keyboard_height() > 0.0 {
            0.0
        } else {
            insets.bottom
        };
        let bubbles = self.messages.iter().enumerate().map(|(i, (mine, m))| {
            let mine = *mine;
            appear(0).key(i.to_string()).child(
                view()
                    .map(|v| if mine { v.self_end() } else { v.self_start() })
                    .max_w(relative(0.8))
                    .px(px(14.0))
                    .py(px(10.0))
                    .rounded(px(18.0))
                    .border_curve(BorderCurve::Continuous)
                    .rounded_br(px(if mine { 6.0 } else { 18.0 }))
                    .rounded_bl(px(if mine { 18.0 } else { 6.0 }))
                    .map(|v| {
                        if mine {
                            v.bg(colors::ACCENT_FILL).finish(Finish::Accent)
                        } else {
                            v.bg(colors::RAISED).finish(Finish::Raised)
                        }
                    })
                    .child(
                        text(m.clone())
                            .text_size(px(15.0))
                            .line_height(px(20.0))
                            .text_color(if mine {
                                colors::ON_ACCENT
                            } else {
                                colors::TEXT
                            }),
                    ),
            )
        });
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps::default())
                .flex_1()
                .bg(colors::BACKGROUND)
                .child(
                    scroll()
                        .flex_1()
                        .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
                        .child(
                            view()
                                .p_4()
                                .gap_2()
                                .child(
                                    text("Today")
                                        .self_center()
                                        .text_size(px(12.0))
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(colors::FAINT)
                                        .pb_2(),
                                )
                                .children(bubbles),
                        ),
                )
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_4()
                        .pt_2()
                        .pb(px(8.0 + below))
                        .bg(colors::SURFACE)
                        .border_t_1()
                        .border_t_color(colors::BORDER)
                        .child(
                            input()
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
                                .placeholder_color(cx.color(colors::FAINT))
                                .selection_color(cx.color(colors::ACCENT))
                                .return_key_type(ReturnKeyType::Send)
                                .flex_1()
                                .h(px(44.0))
                                .px_4()
                                .text_size(px(15.0))
                                .text_color(colors::TEXT)
                                .bg(colors::INSET)
                                .border_1()
                                .border_color(colors::BORDER)
                                .rounded(px(22.0))
                                .finish(Finish::Inset),
                        )
                        .child(RoundButton::new(
                            IconName::ArrowUp,
                            "Send",
                            cx.listener(|this: &mut Self, _, cx| this.send(cx)),
                        )),
                ),
        )
    }
}
