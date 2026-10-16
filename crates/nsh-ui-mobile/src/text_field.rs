//! `TextField`: a labeled text input with a clear button.

use nsh_core::ui::prelude::*;

/// A labeled text input ([`TextFieldProps`]).
#[derive(Debug, Default)]
pub struct TextField {
    /// The text: the app's when it controls it, else our own.
    text: String,
    focused: bool,
    /// The input, to focus it again after clearing.
    input: Option<ElementRef>,
}

/// A labeled text input with a clear button. With `value` the app controls
/// the text (it updates `value` from `onChange`); without, the field keeps
/// its own, starting at `defaultValue`.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct TextFieldProps {
    /// Above the input.
    pub label: Option<SharedString>,
    /// Shown while it is empty.
    pub placeholder: Option<SharedString>,
    /// The text, when the app controls it.
    pub value: Option<SharedString>,
    /// The first text, when the app does not control it.
    #[prop(default)]
    pub default_value: SharedString,
    /// Hides what is typed (passwords).
    #[prop(default)]
    pub secure: bool,
    /// Shows a clear button while there is text (default yes).
    #[prop(default = true)]
    pub clearable: bool,
    /// Not editable, dimmed.
    #[prop(default)]
    pub disabled: bool,
}

/// The events of a [`TextField`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum TextFieldEvent {
    /// The text changed (the new text).
    Change(String),
    /// The return key was pressed (the text).
    Submit(String),
}

impl TextField {
    fn shown<'a>(&'a self, props: &'a TextFieldProps) -> &'a str {
        match &props.value {
            Some(value) => value.as_str(),
            None => &self.text,
        }
    }

    fn change(&mut self, text: String, cx: &mut Context<'_, Self>) {
        if cx.props().value.is_none() {
            self.text = text.clone();
            cx.notify();
        }
        cx.emit(TextFieldEvent::Change(text));
    }
}

impl Component for TextField {
    type Props = TextFieldProps;
    type Event = TextFieldEvent;

    fn setup(&mut self, props: &TextFieldProps, cx: &mut Context<'_, Self>) {
        self.text = props.default_value.as_str().to_owned();
        self.input = Some(cx.element_ref());
    }

    fn render(&self, props: &TextFieldProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let shown = self.shown(props);
        let field = self.input.as_ref().expect("set up");
        view()
            .gap_1()
            .when(props.disabled, |v| v.opacity(0.5))
            .when_some(props.label.clone(), |v, label| {
                v.child(text(label).text_sm().text_color(theme.muted_foreground))
            })
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .min_h(px(44.0))
                    .px_3()
                    .gap_2()
                    .rounded(px(theme.radius))
                    .bg(theme.background)
                    .border_1()
                    .border_color(if self.focused {
                        theme.ring
                    } else {
                        theme.input
                    })
                    .child(
                        input()
                            .element_ref(field)
                            .flex_1()
                            .py_2()
                            .text_base()
                            .text_color(theme.foreground)
                            .value(shown)
                            .placeholder(props.placeholder.as_ref().map_or("", |p| p.as_str()))
                            .placeholder_color(cx.color(theme.muted_foreground))
                            .selection_color(cx.color(theme.primary))
                            .secure(props.secure)
                            .editable(!props.disabled)
                            .on_change_text(cx.listener(|this: &mut Self, text: &str, cx| {
                                this.change(text.to_owned(), cx);
                            }))
                            .on_submit_editing(cx.listener(|_: &mut Self, text: &str, cx| {
                                cx.emit(TextFieldEvent::Submit(text.to_owned()));
                            }))
                            .on_focus(cx.listener(|this: &mut Self, _, cx| {
                                this.focused = true;
                                cx.notify();
                            }))
                            .on_blur(cx.listener(|this: &mut Self, _, cx| {
                                this.focused = false;
                                cx.notify();
                            })),
                    )
                    .when(
                        props.clearable && !shown.is_empty() && !props.disabled,
                        |v| {
                            v.child(
                                view()
                                    .size_6()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .bg(theme.muted)
                                    .active(|s| s.bg(theme.secondary_active))
                                    .role(AccessibilityRole::Button)
                                    .label("Clear")
                                    .hit_slop(EdgeInsets::all(8.0))
                                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                                        this.change(String::new(), cx);
                                        if let Some(input) = &this.input {
                                            input.focus();
                                        }
                                    }))
                                    .child(text("×").text_sm().text_color(theme.muted_foreground)),
                            )
                        },
                    ),
            )
    }
}
