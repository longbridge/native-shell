//! The gallery's controls with motion of their own: [`Toggle`] and
//! [`Segmented`].

use nsh_core::{
    Rect,
    ui::{JsObject, PropEnum, prelude::*},
};

use super::theme::{Finish, Finished, colors, inset_shadow, motion, rgb_a};

/// An on / off switch: the knob springs across, the track changes color
/// natively.
#[derive(Debug, Default)]
pub(crate) struct Toggle {
    /// 0 off, 1 on.
    on: Option<NativeValue>,
}

/// The props of [`Toggle`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct ToggleProps {
    /// On.
    #[prop(default)]
    pub(crate) value: bool,
    /// Its accessibility label.
    pub(crate) label: Option<SharedString>,
}

/// What a [`Toggle`] reports.
#[derive(Debug, Clone, PartialEq, Event)]
pub(crate) enum ToggleEvent {
    /// It was flipped (the new value; the owner sets `value`).
    Change(bool),
}

impl Toggle {
    fn on(&self) -> &NativeValue {
        self.on.as_ref().expect("set up")
    }
}

impl Component for Toggle {
    type Props = ToggleProps;
    type Event = ToggleEvent;

    fn setup(&mut self, props: &ToggleProps, cx: &mut Context<'_, Self>) {
        self.on = Some(cx.native_value(if props.value { 1.0 } else { 0.0 }));
    }

    fn props_changed(&mut self, old: &ToggleProps, new: &ToggleProps, _: &mut Context<'_, Self>) {
        if old.value != new.value {
            self.on()
                .spring(if new.value { 1.0 } else { 0.0 }, motion::SETTLE);
        }
    }

    fn render(&self, props: &ToggleProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let on = self.on();
        let (track, fill) = (cx.color(colors::TRACK), cx.color(colors::ACCENT_FILL));
        let (knob_light, _) = Finish::Knob.shadows();
        view()
            .w(px(50.0))
            .h(px(30.0))
            .rounded(px(15.0))
            .p(px(3.0))
            .border_1()
            .border_color(colors::BORDER)
            .finish_shadow(Finish::Track)
            .bind_background(&on.interpolate_colors(&[0.0, 1.0], &[track, fill]))
            .role(AccessibilityRole::Switch)
            .when_some(props.label.clone(), |v, label| v.label(label))
            .accessibility_state(AccessibilityState {
                checked: Some(if props.value {
                    nsh_core::Checked::True
                } else {
                    nsh_core::Checked::False
                }),
                ..AccessibilityState::default()
            })
            .hit_slop(EdgeInsets::all(8.0))
            .on_press(cx.listener(|this: &mut Self, _, cx| {
                let next = !cx.props().value;
                this.on()
                    .spring(if next { 1.0 } else { 0.0 }, motion::SETTLE);
                cx.emit(ToggleEvent::Change(next));
            }))
            .child(
                view()
                    .size(px(22.0))
                    .rounded(px(11.0))
                    .bg(if props.value {
                        colors::ON_ACCENT
                    } else {
                        colors::KNOB
                    })
                    // A drop of glass: a specular top over the knob's color.
                    .bg_linear_to_b()
                    .gradient_from(rgb_a(0xffffff, 0.3))
                    .gradient_to(rgb_a(0xffffff, 0.0))
                    .gradient_to_position(0.6)
                    .adaptive_shadow(
                        knob_light.iter().copied(),
                        [
                            inset_shadow(1.0, 0.0, rgb_a(0xffffff, 0.2)),
                            BoxShadow {
                                offset_x: 0.0,
                                offset_y: 1.0,
                                blur: 3.0,
                                spread: 0.0,
                                color: rgb_a(0x000000, 0.4),
                                inset: false,
                            },
                        ],
                    )
                    .bind_translate_x(&on.interpolate(&[0.0, 1.0], &[0.0, 20.0])),
            )
    }
}

/// A [`Toggle`] at `value` with accessibility label `label`; its owner
/// handles [`ToggleEvent::Change`] with `.on_event(..)`.
pub(crate) fn toggle(value: bool, label: impl Into<SharedString>) -> ComponentElement<Toggle> {
    Toggle::element(ToggleProps {
        value,
        label: Some(label.into()),
    })
}

/// A segment of [`Segmented`].
#[derive(Debug, Clone, Default, PartialEq, JsObject)]
pub(crate) struct Segment {
    /// Its label.
    pub(crate) label: SharedString,
    /// A quiet number after the label.
    pub(crate) count: Option<u32>,
}

impl Segment {
    /// A segment labelled `label`.
    pub(crate) fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            count: None,
        }
    }

    /// With `count` after the label.
    pub(crate) fn count(mut self, count: u32) -> Self {
        self.count = Some(count);
        self
    }
}

/// How tall a [`Segmented`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub(crate) enum SegmentedSize {
    /// 40 pt.
    #[default]
    Regular,
    /// 32 pt.
    Small,
}

/// Equal segments with an indicator that springs to the selected one.
#[derive(Debug, Default)]
pub(crate) struct Segmented {
    /// Its width as laid out (0 before the first layout).
    width: f32,
    /// The indicator's offset.
    x: Option<NativeValue>,
    /// The indicator was placed: later moves spring, the first jumps.
    placed: bool,
}

/// The props of [`Segmented`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct SegmentedProps {
    /// The segments.
    pub(crate) options: Vec<Segment>,
    /// The selected one's index.
    #[prop(default)]
    pub(crate) selected: u32,
    /// Its height.
    #[prop(default)]
    pub(crate) size: SegmentedSize,
}

/// What a [`Segmented`] reports.
#[derive(Debug, Clone, PartialEq, Event)]
pub(crate) enum SegmentedEvent {
    /// A segment was pressed (its index; the owner sets `selected`).
    Change(u32),
}

impl Segmented {
    /// The width of a segment.
    fn segment(&self, props: &SegmentedProps) -> f32 {
        (self.width - 6.0) / props.options.len().max(1) as f32
    }

    /// Moves the indicator under the selected segment.
    fn place(&mut self, props: &SegmentedProps) {
        if self.width <= 0.0 {
            return;
        }
        let index = (props.selected as usize).min(props.options.len().saturating_sub(1));
        let to = index as f32 * self.segment(props);
        let x = self.x.as_ref().expect("set up");
        if self.placed {
            x.spring(to, motion::SETTLE);
        } else {
            x.set(to);
        }
        self.placed = true;
    }
}

impl Component for Segmented {
    type Props = SegmentedProps;
    type Event = SegmentedEvent;

    fn setup(&mut self, _: &SegmentedProps, cx: &mut Context<'_, Self>) {
        self.x = Some(cx.native_value(0.0));
    }

    fn props_changed(
        &mut self,
        old: &SegmentedProps,
        new: &SegmentedProps,
        _: &mut Context<'_, Self>,
    ) {
        if old.selected != new.selected || old.options.len() != new.options.len() {
            self.place(new);
        }
    }

    fn render(&self, props: &SegmentedProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let small = props.size == SegmentedSize::Small;
        let height = if small { 32.0 } else { 40.0 };
        let (track_light, track_dark) = Finish::Track.shadows();
        // The track's glass: the sunk shadow plus a lit lower rim.
        let rim = |color| inset_shadow(-1.0, 0.0, color);
        let segments = props.options.iter().enumerate().map(|(i, option)| {
            let selected = i as u32 == props.selected;
            view()
                .flex_1()
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(5.0))
                .role(AccessibilityRole::Tab)
                .accessibility_state(AccessibilityState {
                    selected,
                    ..AccessibilityState::default()
                })
                .on_press(cx.listener(move |_: &mut Self, _, cx| {
                    cx.emit(SegmentedEvent::Change(i as u32));
                }))
                .child(
                    text(option.label.clone())
                        .number_of_lines(1)
                        .text_size(px(if small { 13.0 } else { 14.0 }))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(if selected {
                            colors::TEXT
                        } else {
                            colors::MUTED
                        }),
                )
                .children(option.count.map(|count| {
                    text(count.to_string())
                        .text_size(px(12.0))
                        .text_color(colors::FAINT)
                        .tabular_nums()
                }))
        });
        view()
            .h(px(height))
            .flex_row()
            .p(px(3.0))
            .rounded(px(height / 2.0))
            .bg(colors::INSET)
            .adaptive_shadow(
                track_light
                    .iter()
                    .copied()
                    .chain([rim(rgb_a(0xffffff, 0.85))]),
                track_dark
                    .iter()
                    .copied()
                    .chain([rim(rgb_a(0xffffff, 0.04))]),
            )
            .border_1()
            .border_color(colors::BORDER)
            .role(AccessibilityRole::Tablist)
            .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                if this.width != frame.width {
                    this.width = frame.width;
                    this.place(cx.props());
                    cx.notify();
                }
            }))
            .child(
                view()
                    .absolute()
                    .top(px(3.0))
                    .left(px(3.0))
                    .bottom(px(3.0))
                    .w(px(self.segment(props).max(0.0)))
                    .rounded(px((height - 6.0) / 2.0))
                    .finish(Finish::Pill)
                    .border_1()
                    .pointer_events(PointerEvents::None)
                    .bind_translate_x(self.x.as_ref().expect("set up")),
            )
            .children(segments)
    }
}

/// A [`Segmented`] of `options` with `selected` selected; its owner
/// handles [`SegmentedEvent::Change`] with `.on_event(..)`.
pub(crate) fn segmented(
    options: impl IntoIterator<Item = Segment>,
    selected: u32,
) -> ComponentElement<Segmented> {
    Segmented::element(SegmentedProps {
        options: options.into_iter().collect(),
        selected,
        size: SegmentedSize::Regular,
    })
}
