//! `Stepper`: a number with − and + buttons; holding one keeps stepping.

use std::time::Duration;

use nsh_core::ui::prelude::*;

/// A number with − and + buttons ([`StepperProps`]).
#[derive(Debug, Default)]
pub struct Stepper {
    /// The value shown: the app's when it controls it, else our own.
    value: f64,
    /// Stepping while a button is held (dropping it stops).
    repeat: Option<Task<()>>,
}

/// A number with − and + buttons, within `min`–`max`; holding a button
/// keeps stepping. With `value` the app controls it (it updates `value`
/// from `onChange`); without, the stepper keeps its own, starting at
/// `defaultValue`.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct StepperProps {
    /// The value, when the app controls it.
    pub value: Option<f64>,
    /// The first value, when the app does not control it.
    #[prop(default)]
    pub default_value: f64,
    /// The smallest value.
    #[prop(default = 0.0)]
    pub min: f64,
    /// The largest value.
    #[prop(default = 100.0)]
    pub max: f64,
    /// How much a press changes it.
    #[prop(default = 1.0)]
    pub step: f64,
    /// No presses, dimmed.
    #[prop(default)]
    pub disabled: bool,
}

/// The events of a [`Stepper`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum StepperEvent {
    /// The value changed (the new one).
    Change(f64),
}

/// How long a press is held before it repeats, and how fast it repeats.
const HOLD: Duration = Duration::from_millis(400);
const REPEAT: Duration = Duration::from_millis(80);

/// The bounds and the step, as a render's listeners see them.
#[derive(Debug, Clone, Copy)]
struct Range {
    min: f64,
    max: f64,
    step: f64,
    controlled: bool,
}

impl Range {
    fn of(props: &StepperProps) -> Self {
        let (min, max) = (props.min.min(props.max), props.min.max(props.max));
        Self {
            min,
            max,
            step: props.step.abs(),
            controlled: props.value.is_some(),
        }
    }

    /// `value` on the step grid within the bounds.
    fn fit(&self, value: f64) -> f64 {
        let decimals = decimals(self.step);
        let scale = 10f64.powi(decimals as i32);
        ((value * scale).round() / scale).clamp(self.min, self.max)
    }
}

/// The decimals of `step` (0.25 has 2), to show and round to.
fn decimals(step: f64) -> usize {
    let text = format!("{step}");
    text.split_once('.').map_or(0, |(_, d)| d.len())
}

impl Stepper {
    /// One step of `direction` (−1 or 1) within the current props; stops
    /// repeating at a bound.
    fn step(&mut self, direction: f64, cx: &mut Context<'_, Self>) {
        let range = Range::of(cx.props());
        self.change(range.fit(self.value + direction * range.step), cx);
        if self.value == range.min || self.value == range.max {
            self.stop();
        }
    }

    /// Shows `next` (unless the app controls the value) and reports it.
    fn change(&mut self, next: f64, cx: &mut Context<'_, Self>) {
        if next == self.value {
            self.stop();
            return;
        }
        // The app's value follows from `onChange` when it controls it.
        if !Range::of(cx.props()).controlled {
            self.value = next;
            cx.notify();
        }
        cx.emit(StepperEvent::Change(next));
    }

    fn stop(&mut self) {
        self.repeat = None;
    }

    /// A press on − or +: one step now, more while it is held.
    fn press(&mut self, direction: f64, cx: &mut Context<'_, Self>) {
        self.stop();
        self.step(direction, cx);
        // One task: wait for the hold, then step until released.
        self.repeat = Some(cx.spawn(move |this, cx| async move {
            cx.timer(HOLD).await;
            loop {
                let stepping = this.update(move |this: &mut Self, cx| {
                    this.step(direction, cx);
                    this.repeat.is_some()
                });
                if stepping.await != Some(true) {
                    break;
                }
                cx.timer(REPEAT).await;
            }
        }));
    }
}

/// What JS calls on a stepper's ref.
#[methods]
impl Stepper {
    /// Sets the value (on the step grid, within the bounds) and reports it
    /// as a press would.
    pub fn set_value(&mut self, value: f64, cx: &mut Context<'_, Self>) {
        let next = Range::of(cx.props()).fit(value);
        self.change(next, cx);
    }

    /// The value shown.
    pub fn value(&self) -> f64 {
        self.value
    }
}

impl Component for Stepper {
    type Props = StepperProps;
    type Event = StepperEvent;
    const METHODS: &'static [Method<Self>] = Self::JS_METHODS;

    fn setup(&mut self, props: &StepperProps, _cx: &mut Context<'_, Self>) {
        self.value = Range::of(props).fit(props.value.unwrap_or(props.default_value));
    }

    fn props_changed(
        &mut self,
        old: &StepperProps,
        new: &StepperProps,
        _cx: &mut Context<'_, Self>,
    ) {
        let range = Range::of(new);
        if let Some(value) = new.value
            && Some(value) != old.value
        {
            self.value = range.fit(value);
        } else {
            // New bounds or step: the value stays within them.
            self.value = range.fit(self.value);
        }
        if new.disabled {
            self.stop();
        }
    }

    fn render(&self, props: &StepperProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let range = Range::of(props);
        let button = |label: &'static str, name: &'static str, direction: f64, at_bound: bool| {
            let off = props.disabled || at_bound;
            view()
                .size_10()
                .items_center()
                .justify_center()
                .rounded(px(theme.radius))
                .bg(theme.secondary)
                .active(|s| s.bg(theme.secondary_active))
                .when(off, |v| v.opacity(0.4))
                .disabled(off)
                .role(AccessibilityRole::Button)
                .label(name)
                .on(
                    EventKind::PressIn,
                    cx.listener(move |this: &mut Self, _, cx| this.press(direction, cx)),
                )
                .on(
                    EventKind::PressOut,
                    cx.listener(|this: &mut Self, _, _| this.stop()),
                )
                .child(
                    text(label)
                        .text_size(px(theme.font_size + 4.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme.secondary_foreground),
                )
        };
        view()
            .flex_row()
            .items_center()
            .gap_3()
            .child(button("−", "Decrease", -1.0, self.value <= range.min))
            .child(
                text(format!("{:.*}", decimals(range.step), self.value))
                    .min_w(px(48.0))
                    .text_center()
                    .text_size(px(theme.font_size + 1.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground),
            )
            .child(button("+", "Increase", 1.0, self.value >= range.max))
    }
}
