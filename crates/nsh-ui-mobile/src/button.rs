//! `Button`: a press target with a label.

use nsh_core::ui::prelude::*;

/// A press target with a label ([`ButtonProps`]).
#[derive(Debug, Default)]
pub struct Button;

/// A press target with a label, the same on iOS and Android.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ButtonProps {
    /// The label.
    pub title: SharedString,
    /// `primary` (default): filled; `secondary`: tinted; `danger`: a
    /// destructive action; `plain`: text only.
    #[prop(default)]
    pub variant: ButtonVariant,
    /// No presses, no feedback, dimmed.
    #[prop(default)]
    pub disabled: bool,
    /// For screen readers, when `title` is not enough.
    pub accessibility_label: Option<SharedString>,
}

/// How a [`Button`] looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum ButtonVariant {
    /// Filled with the primary color.
    #[default]
    Primary,
    /// Filled with the secondary color.
    Secondary,
    /// Filled with the danger color.
    Danger,
    /// Text only; a background while pressed.
    Plain,
}

/// The events of a [`Button`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum ButtonEvent {
    /// The button was pressed.
    Press,
}

impl Component for Button {
    type Props = ButtonProps;
    type Event = ButtonEvent;

    fn render(&self, props: &ButtonProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (bg, pressed, fg) = match props.variant {
            ButtonVariant::Primary => (
                theme.primary,
                theme.primary_active,
                theme.primary_foreground,
            ),
            ButtonVariant::Secondary => (
                theme.secondary,
                theme.secondary_active,
                theme.secondary_foreground,
            ),
            ButtonVariant::Danger => (theme.danger, theme.danger_active, theme.danger_foreground),
            ButtonVariant::Plain => (AdaptiveColor::TRANSPARENT, theme.accent, theme.foreground),
        };
        view()
            .flex_row()
            .items_center()
            .justify_center()
            .px_4()
            .py_3()
            .rounded(px(theme.radius))
            .bg(bg)
            .active(|s| s.bg(pressed))
            .when(props.disabled, |v| v.opacity(0.5))
            .disabled(props.disabled)
            .role(AccessibilityRole::Button)
            .label(
                props
                    .accessibility_label
                    .clone()
                    .unwrap_or_else(|| props.title.clone()),
            )
            .on_press(cx.listener(|_, _, cx| cx.emit(ButtonEvent::Press)))
            .child(
                text(props.title.clone())
                    .text_size(px(theme.font_size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(fg),
            )
    }
}
