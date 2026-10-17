//! Physics: a card dragged on the UI thread and released into a spring, a
//! decay or the nearest corner, with the drag's velocity; a canvas draws a
//! tether to it from the same native values, without a render per frame
//! (as the TSX gallery's `demos/Physics.tsx`).

use std::f32::consts::TAU;

use nsh_core::{Rect, ui::prelude::*};

use crate::ui::{
    Badge, Finish, Finished, IconName, IconTile, MONO, Segment, Segmented, SegmentedEvent,
    SegmentedProps, SegmentedSize, Stat, Tone, appear, colors, icon, radius, segmented, tints,
};

/// The card's width and height.
const CARD: f32 = 128.0;

/// How the card moves once released.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Mode {
    /// Springs home.
    #[default]
    Spring,
    /// Coasts to a stop, then springs home.
    Coast,
    /// Springs to the corner the fling points at.
    Snap,
}

const MODES: [Mode; 3] = [Mode::Spring, Mode::Coast, Mode::Snap];

/// The release's spring settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Feel {
    Soft,
    #[default]
    Bouncy,
    Firm,
}

const FEELS: [Feel; 3] = [Feel::Soft, Feel::Bouncy, Feel::Firm];

impl Feel {
    /// Its stiffness and damping.
    fn spring(self) -> (f32, f32) {
        match self {
            Feel::Soft => (90.0, 12.0),
            Feel::Bouncy => (180.0, 11.0),
            Feel::Firm => (420.0, 32.0),
        }
    }
}

/// A spring of stiffness `k` and damping `c`.
const fn spring(k: f32, c: f32) -> Spring {
    Spring {
        stiffness: k,
        damping: c,
        mass: 1.0,
        velocity: None,
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Physics {
    /// The drag's horizontal translation (the drag moves it natively).
    x: Option<NativeValue>,
    /// The drag's vertical translation.
    y: Option<NativeValue>,
    /// The pulse: 0 at rest, 1 at its peak.
    pressed: Option<NativeValue>,
    /// 1 while the card is held.
    lift: Option<NativeValue>,
    mode: Mode,
    feel: Feel,
    /// The stage's size, from its layout.
    stage: (f32, f32),
    /// How often the card was let go.
    releases: u32,
    /// The last release's speed, in points per second.
    speed: f32,
    /// The corner the last snap went to.
    corner: &'static str,
    /// The pulse running.
    pulse: Option<Task<()>>,
}

/// One native value of the demo.
fn value(v: &Option<NativeValue>) -> &NativeValue {
    v.as_ref().expect("set up")
}

impl Physics {
    /// Springs `v` to `to` with the feel chosen, starting at `velocity`.
    fn settle(&self, v: &NativeValue, to: f32, velocity: f32) {
        let (k, c) = self.feel.spring();
        let _ = v.spring(
            to,
            Spring {
                velocity: Some(velocity),
                ..spring(k, c)
            },
        );
    }

    /// The snap corners' distance from the middle.
    fn corners(&self) -> (f32, f32) {
        (
            (self.stage.0 / 2.0 - CARD / 2.0 - 14.0).max(0.0),
            (self.stage.1 / 2.0 - CARD / 2.0 - 14.0).max(0.0),
        )
    }

    fn release(&mut self, e: &PanEvent, cx: &mut Context<'_, Self>) {
        let _ = value(&self.lift).spring(0.0, spring(300.0, 24.0));
        self.releases += 1;
        self.speed = e.velocity_x.hypot(e.velocity_y).round();
        let (x, y) = (value(&self.x).clone(), value(&self.y).clone());
        match self.mode {
            Mode::Spring => {
                self.settle(&x, 0.0, e.velocity_x);
                self.settle(&y, 0.0, e.velocity_y);
                self.corner = "—";
            }
            Mode::Coast => {
                // Coast with the release velocity, then spring home unless
                // grabbed again.
                for (v, velocity) in [(x, e.velocity_x), (y, e.velocity_y)] {
                    let end = v.decay(Some(velocity), 0.992);
                    cx.spawn(move |this, _| async move {
                        if end.await {
                            this.update(move |this: &mut Self, _| this.settle(&v, 0.0, 0.0))
                                .await;
                        }
                    })
                    .detach();
                }
                self.corner = "—";
            }
            Mode::Snap => {
                // Where the fling would carry it, then the nearest corner.
                let px = e.value_x.unwrap_or(0.0) + e.velocity_x * 0.2;
                let py = e.value_y.unwrap_or(0.0) + e.velocity_y * 0.2;
                let (cx_, cy_) = self.corners();
                self.settle(&x, if px < 0.0 { -cx_ } else { cx_ }, e.velocity_x);
                self.settle(&y, if py < 0.0 { -cy_ } else { cy_ }, e.velocity_y);
                self.corner = match (py < 0.0, px < 0.0) {
                    (true, true) => "top left",
                    (true, false) => "top right",
                    (false, true) => "bottom left",
                    (false, false) => "bottom right",
                };
            }
        }
        cx.notify();
    }

    /// The stage's backdrop: a dot grid, the axes, the snap corners, and
    /// the tether to the card, which follows the native values.
    fn backdrop(&self, cx: &RenderCx<'_, Self>) -> Painter {
        // The canvas fills the stage inside its 1 pt rim.
        let (width, height) = (self.stage.0 - 2.0, self.stage.1 - 2.0);
        let (mx, my) = (width / 2.0, height / 2.0);
        let mut p = Painter::new().fill_color(cx.color(colors::LINE));
        let mut gx = 10.0;
        while gx < width {
            let mut gy = 10.0;
            while gy < height {
                p = p.fill_rect(gx, gy, 1.5, 1.5);
                gy += 16.0;
            }
            gx += 16.0;
        }
        p = p
            .line_dash(&[3.0, 5.0])
            .stroke_color(cx.color(colors::BORDER))
            .line_width(1.0)
            .begin_path()
            .move_to(mx, 0.0)
            .line_to(mx, height)
            .move_to(0.0, my)
            .line_to(width, my)
            .stroke();
        if self.mode == Mode::Snap {
            let (cx_, cy_) = self.corners();
            p = p.stroke_color(cx.color(colors::FAINT));
            for sx in [-1.0, 1.0] {
                for sy in [-1.0, 1.0] {
                    p = p
                        .begin_path()
                        .round_rect(
                            mx + sx * cx_ - CARD / 2.0,
                            my + sy * cy_ - CARD / 2.0,
                            CARD,
                            CARD,
                            26.0,
                        )
                        .stroke();
                }
            }
        }
        let (x, y) = (value(&self.x), value(&self.y));
        p.line_dash(&[])
            // The tether, from native values: it follows drags and springs
            // without a render.
            .stroke_color(cx.color(colors::ACCENT))
            .global_alpha(0.45)
            .line_width(1.5)
            .begin_path()
            .move_to(mx, my)
            .line_to(&x.add(mx), &y.add(my))
            .stroke()
            .global_alpha(1.0)
            .fill_color(cx.color(colors::ACCENT))
            .begin_path()
            .arc(mx, my, 4.0, 0.0, TAU)
            .fill()
    }
}

impl Component for Physics {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.x = Some(cx.native_value(0.0));
        self.y = Some(cx.native_value(0.0));
        self.pressed = Some(cx.native_value(0.0));
        self.lift = Some(cx.native_value(0.0));
        self.corner = "—";
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (x, y, pressed, lift) = (
            value(&self.x),
            value(&self.y),
            value(&self.pressed),
            value(&self.lift),
        );
        let (k, c) = self.feel.spring();
        let (spring_label, spring_value) = if self.mode == Mode::Snap {
            ("Corner", self.corner.to_string())
        } else {
            ("Spring", format!("{k} / {c}"))
        };
        let speed = Stat::new("Speed", format!("{} pt/s", self.speed)).width(0.34);
        view()
            .flex_1()
            .p_4()
            .gap_3()
            .bg(colors::BACKGROUND)
            .child(
                appear(0).child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(IconTile::new(IconName::Physics).tint(colors::ACCENT))
                        .child(
                            view()
                                .flex_1()
                                .gap(px(2.0))
                                .child(
                                    text("Drag and release")
                                        .text_size(px(17.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::TEXT),
                                )
                                .child(
                                    text("Tap to pulse, long press to reset")
                                        .number_of_lines(1)
                                        .text_size(px(13.0))
                                        .text_color(colors::MUTED),
                                ),
                        )
                        .child(Badge::new("UI thread").tone(Tone::Accent)),
                ),
            )
            .child(
                view()
                    .flex_1()
                    .overflow_hidden()
                    .rounded(px(radius::CARD))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(colors::BORDER)
                    .bg(colors::INSET)
                    .finish(Finish::Inset)
                    .items_center()
                    .justify_center()
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.stage != (frame.width, frame.height) {
                            this.stage = (frame.width, frame.height);
                            cx.notify();
                        }
                    }))
                    .child(
                        canvas(self.backdrop(cx))
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .inset_0(),
                    )
                    .child(
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .size(px(CARD))
                            .rounded(px(30.0))
                            .bg(tints::ACCENT)
                            .bind_opacity(lift)
                            .bind_translate_x(x)
                            .bind_translate_y(y)
                            .bind_scale(&lift.interpolate(&[0.0, 1.0], &[1.0, 1.32])),
                    )
                    .child(
                        view()
                            .size(px(CARD))
                            .rounded(px(30.0))
                            .border_curve(BorderCurve::Continuous)
                            .items_center()
                            .justify_center()
                            .gap_2()
                            .role(AccessibilityRole::Button)
                            .label("Draggable card")
                            .pan_values(Some(x), Some(y))
                            .bind_background(&pressed.interpolate_colors(
                                &[0.0, 1.0],
                                &[cx.color(colors::ACCENT_FILL), cx.color(colors::INFO)],
                            ))
                            // The accent's highlight and glow; the color
                            // itself animates.
                            .finish_shadow(Finish::Accent)
                            .bind_translate_x(x)
                            .bind_translate_y(y)
                            .bind_scale(&pressed.interpolate(&[0.0, 1.0], &[1.0, 1.12]))
                            .bind_scale(&lift.interpolate(&[0.0, 1.0], &[1.0, 1.06]))
                            .bind_rotate(&x.interpolate(&[-200.0, 200.0], &[-14.0, 14.0]))
                            .on_pan_start(cx.listener(|this: &mut Self, _: &PanEvent, _| {
                                let _ = value(&this.lift).spring(1.0, spring(400.0, 26.0));
                            }))
                            .on_pan_end(
                                cx.listener(|this: &mut Self, e: &PanEvent, cx| {
                                    this.release(e, cx)
                                }),
                            )
                            .on_press(cx.listener(|this: &mut Self, _, cx| {
                                let pressed = value(&this.pressed).clone();
                                this.pulse = Some(cx.spawn(|_, _| async move {
                                    let _ = pressed.timing(1.0, 120.0, Easing::EaseOut).await;
                                    let _ = pressed.timing(0.0, 320.0, Easing::EaseInOut).await;
                                }));
                            }))
                            .on_long_press(cx.listener(|this: &mut Self, _, cx| {
                                this.releases = 0;
                                this.speed = 0.0;
                                this.corner = "—";
                                this.settle(value(&this.x), 0.0, 0.0);
                                this.settle(value(&this.y), 0.0, 0.0);
                                cx.notify();
                            }))
                            .child(icon(IconName::Gestures, 28.0, colors::ON_ACCENT))
                            .child(
                                text("Drag me")
                                    .text_color(colors::ON_ACCENT)
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::BOLD),
                            ),
                    ),
            )
            .child(
                view()
                    .bg(colors::SURFACE)
                    .finish(Finish::Card)
                    .rounded(px(radius::CARD))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(colors::BORDER)
                    .p_3()
                    .gap_3()
                    .child(
                        segmented(
                            ["Spring", "Coast", "Snap"].map(Segment::new),
                            MODES.iter().position(|m| *m == self.mode).unwrap_or(0) as u32,
                        )
                        .on_event(cx.listener(
                            |this: &mut Self, SegmentedEvent::Change(i): &SegmentedEvent, cx| {
                                this.mode = MODES[*i as usize];
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        Segmented::element(SegmentedProps {
                            options: ["Soft", "Bouncy", "Firm"].map(Segment::new).to_vec().into(),
                            selected: (FEELS.iter().position(|f| *f == self.feel).unwrap_or(0)
                                as u32)
                                .into(),
                            size: SegmentedSize::Small,
                        })
                        .on_event(cx.listener(
                            |this: &mut Self, SegmentedEvent::Change(i): &SegmentedEvent, cx| {
                                this.feel = FEELS[*i as usize];
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        view()
                            .flex_row()
                            .px_1()
                            .child(Stat::new("Releases", self.releases.to_string()).width(0.3))
                            .child(if self.speed > 1500.0 {
                                speed.tone(Tone::Warn)
                            } else {
                                speed
                            })
                            .child(Stat::new(spring_label, spring_value).width(0.36)),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_baseline()
                            .gap_2()
                            .px_1()
                            .child(
                                text(match self.mode {
                                    Mode::Spring => "Springs home with the release velocity",
                                    Mode::Coast => "Coasts to a stop, then springs home",
                                    Mode::Snap => "Springs to the corner the fling points at",
                                })
                                .flex_1()
                                .text_size(px(13.0))
                                .text_color(colors::MUTED),
                            )
                            .child(
                                text(if self.mode == Mode::Coast {
                                    "decay"
                                } else {
                                    "spring"
                                })
                                .text_size(px(12.0))
                                .font_family(MONO)
                                .text_color(colors::FAINT),
                            ),
                    ),
            )
    }
}
