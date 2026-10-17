//! The gallery's motion: [`Press`] sinks under the finger, [`Appear`]
//! brings content in. Both run natively: only press in / out reach Rust.

use nsh_core::ui::{PropEnum, prelude::*};

use super::theme::motion;

/// What a [`Press`] is to assistive technologies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub(crate) enum PressRole {
    /// A button.
    #[default]
    Button,
    /// A tab.
    Tab,
    /// A switch.
    Switch,
    /// A link.
    Link,
}

impl PressRole {
    fn role(self) -> AccessibilityRole {
        match self {
            PressRole::Button => AccessibilityRole::Button,
            PressRole::Tab => AccessibilityRole::Tab,
            PressRole::Switch => AccessibilityRole::Switch,
            PressRole::Link => AccessibilityRole::Link,
        }
    }
}

/// A pressable that sinks a little under the finger and springs back (the
/// spring runs natively; only press in / out reach Rust). Its content is
/// its children: `Press::element(..).on_event(..).child(..)`, or
/// [`press`].
#[derive(Debug, Default)]
pub(crate) struct Press {
    scale: Option<NativeValue>,
}

/// The props of [`Press`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct PressProps {
    /// How far it shrinks.
    #[prop(default = 0.97)]
    pub(crate) scale: f32,
    /// Takes no presses, at 40% opacity.
    #[prop(default)]
    pub(crate) disabled: bool,
    /// What it is to assistive technologies.
    #[prop(default)]
    pub(crate) role: PressRole,
    /// Its accessibility label.
    pub(crate) label: Option<SharedString>,
    /// Selected (a tab).
    #[prop(default)]
    pub(crate) selected: bool,
    /// Checked (a switch).
    pub(crate) checked: Option<bool>,
    /// How far around it a touch still hits it, in points.
    pub(crate) hit_slop: Option<f32>,
    /// It reports long presses ([`PressEvent::LongPress`]) instead of the
    /// presses they end in.
    #[prop(default)]
    pub(crate) long_press: bool,
    /// The content.
    pub(crate) children: Children,
}

/// What a [`Press`] reports.
#[derive(Debug, Clone, PartialEq, Event)]
pub(crate) enum PressEvent {
    /// It was pressed.
    Press,
    /// It was held (with `long_press`).
    LongPress,
}

impl Press {
    fn value(&self) -> &NativeValue {
        self.scale.as_ref().expect("set up")
    }
}

impl Component for Press {
    type Props = PressProps;
    type Event = PressEvent;

    fn setup(&mut self, _: &PressProps, cx: &mut Context<'_, Self>) {
        self.scale = Some(cx.native_value(1.0));
    }

    fn render(&self, props: &PressProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let checked = props.checked.map(|on| {
            if on {
                nsh_core::Checked::True
            } else {
                nsh_core::Checked::False
            }
        });
        view()
            .bind_scale(self.value())
            .opacity(if props.disabled { 0.4 } else { 1.0 })
            .disabled(props.disabled)
            .role(props.role.role())
            .when_some(props.label.clone(), |v, label| v.label(label))
            .accessibility_state(AccessibilityState {
                disabled: props.disabled,
                selected: props.selected,
                checked,
                ..AccessibilityState::default()
            })
            .when_some(props.hit_slop, |v, slop| v.hit_slop(EdgeInsets::all(slop)))
            .on_press_in(cx.listener(|this: &mut Self, _, cx| {
                this.value().spring(cx.props().scale, motion::PRESS);
            }))
            .on_press_out(cx.listener(|this: &mut Self, _, _| {
                this.value().spring(1.0, motion::PRESS);
            }))
            .on_press(cx.listener(|_: &mut Self, _, cx| cx.emit(PressEvent::Press)))
            .when(props.long_press, |v| {
                v.on_long_press(cx.listener(|_: &mut Self, _, cx| cx.emit(PressEvent::LongPress)))
            })
            .child(props.children)
    }
}

/// A [`Press`] with `props`, calling `on_press` when pressed (and, with
/// `long_press`, held: [`Press::element`] with `.on_event(..)` tells them
/// apart).
pub(crate) fn press_with(props: PressProps, on_press: Listener<()>) -> ComponentElement<Press> {
    Press::element(props).on_event(on_press.map(|_: &PressEvent| ()))
}

/// Fades and lifts its content in once mounted; `index` staggers siblings
/// (40 ms apart). `flex` lets it wrap a page that fills its parent.
#[derive(Debug, Default)]
pub(crate) struct Appear {
    shown: Option<NativeValue>,
}

/// The props of [`Appear`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct AppearProps {
    /// Its place among siblings coming in together: it waits 40 ms per
    /// place (8 at most).
    #[prop(default)]
    pub(crate) index: u32,
    /// It fills its parent.
    #[prop(default)]
    pub(crate) flex: bool,
    /// The content.
    pub(crate) children: Children,
}

impl Component for Appear {
    type Props = AppearProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &AppearProps, cx: &mut Context<'_, Self>) {
        let shown = cx.native_value(0.0);
        shown.timing_with(
            1.0,
            Timing {
                delay_ms: props.index.min(8) as f32 * 40.0,
                ..motion::ENTER
            },
        );
        self.shown = Some(shown);
    }

    fn render(&self, props: &AppearProps, _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let shown = self.shown.as_ref().expect("set up");
        view()
            .when(props.flex, |v| v.flex_1())
            .bind_opacity(shown)
            .bind_translate_y(&shown.interpolate(&[0.0, 1.0], &[14.0, 0.0]))
            .child(props.children)
    }
}

/// [`Appear`] at `index` among its siblings.
pub(crate) fn appear(index: u32) -> ComponentElement<Appear> {
    Appear::element(AppearProps {
        index,
        ..AppearProps::default()
    })
}
