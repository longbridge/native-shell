//! System: Status bar, alerts, action sheets, environment, timers (as the TSX gallery's `demos/System.tsx`).

use std::time::{Duration, Instant};

use nsh_core::ui::{AnyElement, Dialog, DialogButton, DialogButtonStyle, prelude::*};
use nsh_ui_mobile::{BarStyle, StatusBar, StatusBarProps};

use crate::ui::{button, colors, page, section};

/// The status bar's styles, in the order "Next style" goes through them.
const BAR_STYLES: [(BarStyle, &str); 3] = [
    (BarStyle::Default, "default"),
    (BarStyle::LightContent, "light-content"),
    (BarStyle::DarkContent, "dark-content"),
];

/// A label and its value on one line.
fn row(label: &str, value: impl Into<SharedString>) -> AnyElement {
    view()
        .flex_row()
        .child(text(label.to_owned()).flex_1().text_color(colors::muted()))
        .child(text(value.into()))
        .into_element()
}

fn note(content: &str) -> AnyElement {
    text(content.to_owned())
        .text_color(colors::muted())
        .into_element()
}

/// A dialog button.
fn choice(label: &str, style: DialogButtonStyle) -> DialogButton {
    DialogButton {
        text: label.into(),
        style,
    }
}

/// The demo's page: the environment, the status bar, system dialogs and a
/// timer.
#[derive(Default)]
pub(crate) struct System {
    /// Seconds since the page opened, counted by an interval.
    ticks: u32,
    /// The interval counting `ticks` (dropped with the page: cleared).
    timer: Option<Task<()>>,
    /// What the last dialog answered.
    answer: String,
    /// The index of the status bar's style in [`BAR_STYLES`].
    bar_style: usize,
    /// The status bar is hidden.
    bar_hidden: bool,
    /// Counts the display frames while it runs ("Count frames").
    frames: Option<Task<()>>,
    /// Frames since `since`.
    frame_count: u32,
    /// When the frames counted started.
    since: Option<Instant>,
    /// Frames per second, over the last second.
    fps: u32,
}

impl System {
    /// Shows `dialog`, then shows what `answer` makes of the index chosen
    /// (-1: dismissed).
    fn ask(
        cx: &mut Context<'_, Self>,
        dialog: Dialog,
        answer: impl FnOnce(i32) -> String + 'static,
    ) {
        let chosen = cx.dialog(dialog);
        cx.spawn(|this, _| async move {
            let index = chosen.await;
            this.update(move |this, cx| {
                this.answer = answer(index);
                cx.notify();
            })
            .await;
        })
        .detach();
    }
}

impl System {
    /// Starts or stops counting frames: a task awaits each display frame
    /// (TS: `requestAnimationFrame` asking again from its callback).
    fn toggle_frames(&mut self, cx: &mut Context<'_, Self>) {
        if self.frames.take().is_some() {
            cx.notify();
            return;
        }
        self.frame_count = 0;
        self.since = Some(Instant::now());
        self.frames = Some(cx.spawn(|this, cx| async move {
            loop {
                let at = cx.next_frame().await;
                if this
                    .update(move |this, cx| this.frame(at, cx))
                    .await
                    .is_none()
                {
                    break;
                }
            }
        }));
        cx.notify();
    }

    /// A frame began at `at`: once a second, the rate.
    fn frame(&mut self, at: Instant, cx: &mut Context<'_, Self>) {
        self.frame_count += 1;
        let since = *self.since.get_or_insert(at);
        let elapsed = at.saturating_duration_since(since).as_secs_f64();
        if elapsed >= 1.0 {
            self.fps = (f64::from(self.frame_count) / elapsed).round() as u32;
            self.frame_count = 0;
            self.since = Some(at);
            cx.notify();
        }
    }
}

/// `labels[index]`, or `"dismissed"`.
fn chosen(labels: &[&str], index: i32) -> String {
    usize::try_from(index)
        .ok()
        .and_then(|i| labels.get(i))
        .map_or("dismissed", |l| l)
        .to_owned()
}

impl Component for System {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.timer = Some(cx.set_interval(Duration::from_secs(1), |this, cx| {
            this.ticks += 1;
            cx.notify();
        }));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let window = cx.window();
        let safe = cx.safe_area();
        let environment = section(
            "Environment (reactive)",
            [
                row(
                    "Window",
                    format!("{} × {}", window.width.round(), window.height.round()),
                ),
                row(
                    "Safe area",
                    format!(
                        "{} / {} / {} / {}",
                        safe.top.round(),
                        safe.right.round(),
                        safe.bottom.round(),
                        safe.left.round()
                    ),
                ),
                row("Keyboard", format!("{} pt", cx.keyboard_height().round())),
                row("Color scheme", if cx.dark() { "dark" } else { "light" }),
                row("Font scale", format!("{:.2}", cx.font_scale())),
            ],
        );

        let (style, style_name) = BAR_STYLES[self.bar_style];
        let status_bar =
            section(
                "Status bar",
                [
                    StatusBar::element(StatusBarProps {
                        bar_style: Some(style),
                        hidden: Some(self.bar_hidden),
                        animated: Some(true),
                        android: None,
                    })
                    .into_element(),
                    row("Style", style_name),
                    view()
                        .flex_row()
                        .gap_2()
                        .child(view().flex_1().child(
                            button("Next style", colors::accent()).on_press(cx.listener(
                                |this: &mut Self, _, cx| {
                                    this.bar_style = (this.bar_style + 1) % BAR_STYLES.len();
                                    cx.notify();
                                },
                            )),
                        ))
                        .child(
                            view().flex_1().child(
                                button(
                                    if self.bar_hidden { "Show" } else { "Hide" },
                                    colors::gray(),
                                )
                                .on_press(cx.listener(
                                    |this: &mut Self, _, cx| {
                                        this.bar_hidden = !this.bar_hidden;
                                        cx.notify();
                                    },
                                )),
                            ),
                        )
                        .into_element(),
                ],
            );

        let alerts = section(
            "Alerts",
            [
                button("Alert with one button", colors::accent())
                    .on_press(cx.listener(|_: &mut Self, _, cx| {
                        let dialog = Dialog {
                            id: 0,
                            title: Some("Saved".into()),
                            message: Some("Your changes were saved.".into()),
                            buttons: vec![choice("OK", DialogButtonStyle::Default)],
                            cancelable: false,
                            sheet: false,
                        };
                        Self::ask(cx, dialog, |_| "OK".into());
                    }))
                    .into_element(),
                button("Confirm (cancel / destructive)", rgb(0xcf222e))
                    .on_press(cx.listener(|_: &mut Self, _, cx| {
                        let dialog = Dialog {
                            id: 0,
                            title: Some("Delete the photo?".into()),
                            message: Some("This cannot be undone.".into()),
                            buttons: vec![
                                choice("Cancel", DialogButtonStyle::Cancel),
                                choice("Delete", DialogButtonStyle::Destructive),
                            ],
                            cancelable: true,
                            sheet: false,
                        };
                        Self::ask(cx, dialog, |i| chosen(&["Cancel", "Delete"], i));
                    }))
                    .into_element(),
                button("Three buttons", colors::purple())
                    .on_press(cx.listener(|_: &mut Self, _, cx| {
                        let dialog = Dialog {
                            id: 0,
                            title: Some("Update available".into()),
                            message: Some("Install it now?".into()),
                            buttons: vec![
                                choice("Later", DialogButtonStyle::Default),
                                choice("Never", DialogButtonStyle::Cancel),
                                choice("Install", DialogButtonStyle::Default),
                            ],
                            cancelable: false,
                            sheet: false,
                        };
                        Self::ask(cx, dialog, |i| chosen(&["Later", "Never", "Install"], i));
                    }))
                    .into_element(),
                button("Action sheet", rgb(0x1a7f37))
                    .on_press(cx.listener(|_: &mut Self, _, cx| {
                        let options = ["Cancel", "Copy link", "Save", "Delete"];
                        let dialog = Dialog {
                            id: 0,
                            title: Some("Share the photo".into()),
                            message: None,
                            buttons: options
                                .iter()
                                .enumerate()
                                .map(|(i, option)| {
                                    choice(
                                        option,
                                        match i {
                                            0 => DialogButtonStyle::Cancel,
                                            3 => DialogButtonStyle::Destructive,
                                            _ => DialogButtonStyle::Default,
                                        },
                                    )
                                })
                                .collect(),
                            cancelable: true,
                            sheet: true,
                        };
                        // Dismissing the sheet is its cancel action (as
                        // `ActionSheet.show` answers).
                        Self::ask(cx, dialog, move |i| chosen(&options, i.max(0)));
                    }))
                    .into_element(),
                button("Toast (Android)", colors::gray())
                    .on_press(cx.listener(|_: &mut Self, _, cx| {
                        cx.toast("Hello from a toast", ToastDuration::Long);
                    }))
                    .into_element(),
                row(
                    "Answer",
                    if self.answer.is_empty() {
                        "–".to_owned()
                    } else {
                        self.answer.clone()
                    },
                ),
            ],
        );

        let counting = self.frames.is_some();
        let timers = section(
            "Frames and timers",
            [
                row(
                    "requestAnimationFrame",
                    if counting {
                        format!("{} fps", self.fps)
                    } else {
                        "stopped".to_owned()
                    },
                ),
                row("setInterval (1 s)", self.ticks.to_string()),
                button(
                    if counting {
                        "Stop counting frames"
                    } else {
                        "Count frames"
                    },
                    colors::accent(),
                )
                .on_press(cx.listener(|this: &mut Self, _, cx| this.toggle_frames(cx)))
                .into_element(),
            ],
        );

        let network = section(
            "Network and logs",
            [
                note(
                    "fetch() is a JS API: a Rust component fetches with an HTTP crate in a task (cx.spawn runs on tokio).",
                ),
                button("log::info! (shown by nsh run)", colors::gray())
                    .on_press(cx.listener(|_: &mut Self, _, _| {
                        log::info!(target: "gallery", "hello from the gallery");
                    }))
                    .into_element(),
            ],
        );

        view().flex_1().child(page([
            environment.into_element(),
            status_bar.into_element(),
            alerts.into_element(),
            timers.into_element(),
            network.into_element(),
        ]))
    }
}
