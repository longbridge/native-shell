//! The software keyboard: `Keyboard` and `KeyboardAvoidingView`.

use nsh_core::ui::{
    LayoutAnimation, LayoutCurve, LayoutProperty, LayoutTransition, Style, prelude::*,
};

/// The software keyboard.
#[derive(Debug)]
pub struct Keyboard;

/// The software keyboard.
#[module]
impl Keyboard {
    /// Unfocuses the focused input, hiding the keyboard.
    pub fn dismiss(cx: &mut ModuleCx) {
        cx.dismiss_keyboard();
    }
}

/// A view keeping its content above the keyboard ([`KeyboardAvoidingViewProps`]).
#[derive(Debug, Default)]
pub struct KeyboardAvoidingView;

/// A view that keeps its content above the software keyboard, animated
/// with the keyboard. For content outside scroll views, such as a message
/// bar at the bottom of a screen (inputs inside scroll views are kept
/// visible natively). It assumes it reaches the bottom of the surface, as
/// a screen's content does. Where the window shrinks for the keyboard
/// instead (an Android host that resizes), nothing moves.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct KeyboardAvoidingViewProps {
    /// Avoid the keyboard (default true).
    #[prop(default = true)]
    pub enabled: bool,
    /// `padding` (default) pads the bottom by the keyboard's height;
    /// `height` shrinks the view instead (a bottom margin, so its
    /// background ends at the keyboard); `position` moves an inner content
    /// view (`contentContainerStyle`) up.
    #[prop(default)]
    pub behavior: AvoidBehavior,
    /// Added to the keyboard's height while it shows (a tab bar below the
    /// view, say).
    #[prop(default)]
    pub keyboard_vertical_offset: f32,
    /// The inner view's style with `behavior="position"`.
    #[prop(default)]
    pub content_container_style: Style,
    /// The content.
    pub children: Children,
}

/// How a [`KeyboardAvoidingView`] makes room.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PropEnum)]
pub enum AvoidBehavior {
    /// Pads its bottom.
    #[default]
    Padding,
    /// Shrinks.
    Height,
    /// Moves its content up.
    Position,
}

impl Component for KeyboardAvoidingView {
    type Props = KeyboardAvoidingViewProps;
    type Event = NoEvent;

    fn environment_changed(
        &mut self,
        props: &KeyboardAvoidingViewProps,
        old: &Environment,
        new: &Environment,
        cx: &mut Context<'_, Self>,
    ) {
        // Moves with the keyboard: the frame change animates natively.
        if props.enabled
            && old.keyboard_height != new.keyboard_height
            && new.keyboard_duration > 0.0
        {
            cx.animate_layout(LayoutAnimation {
                create: None,
                update: Some(LayoutTransition {
                    curve: LayoutCurve::EaseInEaseOut,
                    property: LayoutProperty::Opacity,
                    duration_ms: new.keyboard_duration * 1000.0,
                    delay_ms: 0.0,
                }),
                delete: None,
            });
        }
    }

    fn render(
        &self,
        props: &KeyboardAvoidingViewProps,
        cx: &mut RenderCx<'_, Self>,
    ) -> impl IntoElement {
        let height = cx.keyboard_height();
        let lift = if !props.enabled || height == 0.0 {
            0.0
        } else {
            (height + props.keyboard_vertical_offset).max(0.0)
        };
        let below = EdgeInsets {
            bottom: lift,
            ..EdgeInsets::default()
        };
        match props.behavior {
            AvoidBehavior::Padding => view().inset_padding(below).child(props.children),
            AvoidBehavior::Height => view().inset_margin(below).child(props.children),
            AvoidBehavior::Position => view().child(
                view()
                    .inline_style(&props.content_container_style)
                    .relative()
                    .bottom(px(lift))
                    .child(props.children),
            ),
        }
    }
}
