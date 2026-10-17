//! Gestures: a photo dragged, pinched and rotated natively, a ball dragged
//! as the touch responder, raw touches with every finger drawn, press
//! events lighting up as they arrive, hit slop, pointer events and
//! accessibility labels (as the TSX gallery's `demos/Gestures.tsx`).

use nsh_core::{
    Rect, Ripple, TouchPhase, TouchPoint, UNDERLAY_OPACITY,
    nsh_animation::{Curve, EaseMode},
    ui::{Animation, BoxShadow, TouchEvent, prelude::*},
};

use crate::ui::{
    Badge, Button, ButtonVariant, Card, Finish, Finished, IconName, IconTile, MONO, PressProps,
    Row, Stat, Tone, appear, caption, colors, icon, inset_shadow, press_with, radius, rgb_a, tints,
};

/// A dot matrix behind a stage, drawn for its size.
#[derive(Debug, Default)]
struct Dots {
    size: (f32, f32),
}

impl Component for Dots {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (width, height) = self.size;
        let mut p = Painter::new().fill_color(cx.color(colors::LINE));
        let mut x = 8.0;
        while x < width {
            let mut y = 8.0;
            while y < height {
                p = p.fill_rect(x, y, 1.5, 1.5);
                y += 14.0;
            }
            x += 14.0;
        }
        view()
            .flex_1()
            .pointer_events(PointerEvents::None)
            .child(canvas(p).flex_1())
            .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                if this.size != (frame.width, frame.height) {
                    this.size = (frame.width, frame.height);
                    cx.notify();
                }
            }))
    }
}

/// [`Dots`] filling its stage.
fn dots() -> impl IntoElement {
    Dots::element(()).absolute().inset_0()
}

/// A stage: the darker surface a demo plays on.
fn stage() -> View {
    view()
        .rounded(px(radius::CONTROL))
        .border_curve(BorderCurve::Continuous)
        .overflow_hidden()
        .bg(colors::BACKGROUND)
        .border_1()
        .border_color(colors::LINE)
        .finish(Finish::Track)
}

/// The highlight and shadow of a small colored knob.
fn knob<E: Finished>(element: E) -> E {
    let (light, _) = Finish::Knob.shadows();
    element.adaptive_shadow(
        light.iter().copied(),
        [
            inset_shadow(1.0, 0.0, rgb_a(0xffffff, 0.35)),
            BoxShadow {
                offset_x: 0.0,
                offset_y: 3.0,
                blur: 10.0,
                spread: 0.0,
                color: rgb_a(0x000000, 0.45),
                inset: false,
            },
        ],
    )
}

/// A plain label with the API it uses, in code type.
fn api(label: &'static str, api: &'static str) -> View {
    view()
        .flex_row()
        .items_baseline()
        .gap_2()
        .child(
            text(label)
                .flex_1()
                .text_size(px(13.0))
                .text_color(colors::MUTED),
        )
        .child(
            text(api)
                .text_size(px(12.0))
                .font_family(MONO)
                .text_color(colors::FAINT),
        )
}

/// The fingers still down during `touch` (an end or cancel lists the lifted
/// ones too), and the one it is about.
fn fingers(touch: &TouchEvent) -> (Vec<&TouchPoint>, Option<&TouchPoint>) {
    let lifted = matches!(touch.phase, TouchPhase::End | TouchPhase::Cancel);
    let down = touch
        .points
        .iter()
        .filter(|p| !lifted || !p.changed)
        .collect();
    (down, touch.points.iter().find(|p| p.changed))
}

/// Drag, pinch and rotate one photo at the same time, all on the UI thread.
#[derive(Debug, Default)]
struct Transformable {
    values: Option<[NativeValue; 4]>,
    /// The last drag's distance, scale and angle.
    drag: f32,
    scale: f32,
    angle: f32,
}

impl Component for Transformable {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.values = Some([0.0, 0.0, 1.0, 0.0].map(|v| cx.native_value(v)));
        self.scale = 1.0;
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let [x, y, scale, angle] = self.values.as_ref().expect("set up");
        Card::new()
            .icon(IconName::Gestures)
            .title("Pan, pinch and rotation")
            .subtitle("Three recognizers on one view, driven natively")
            .child(
                stage()
                    .h(px(260.0))
                    .items_center()
                    .justify_center()
                    .child(dots())
                    .child(
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .w(px(200.0))
                            .h(px(150.0))
                            .rounded(px(18.0))
                            .border_1()
                            .border_color(colors::FAINT)
                            .border_dashed(),
                    )
                    .child(
                        view()
                            .w(px(200.0))
                            .h(px(150.0))
                            .rounded(px(18.0))
                            .border_curve(BorderCurve::Continuous)
                            .overflow_hidden()
                            .border_1()
                            .border_color(colors::BORDER)
                            .bg(colors::RAISED)
                            .pan_values(Some(x), Some(y))
                            .pinch_value(scale)
                            .rotation_value(angle)
                            .on_pan_end(cx.listener(|this: &mut Self, e: &PanEvent, cx| {
                                this.drag = e.translation_x.hypot(e.translation_y).round();
                                cx.notify();
                            }))
                            .on_pinch_end(cx.listener(|this: &mut Self, e: &GestureEvent, cx| {
                                this.scale = e.value;
                                cx.notify();
                            }))
                            .on_rotation_end(cx.listener(
                                |this: &mut Self, e: &GestureEvent, cx| {
                                    this.angle = e.value;
                                    cx.notify();
                                },
                            ))
                            .bind_translate_x(x)
                            .bind_translate_y(y)
                            .bind_scale(scale)
                            .bind_rotate(angle)
                            .child(image("https://picsum.photos/seed/nsh-card/400/300").flex_1()),
                    ),
            )
            .child(
                view()
                    .flex_row()
                    .child(Stat::new("Drag", format!("{} pt", self.drag)).width(0.33))
                    .child(
                        Stat::new("Scale", format!("{:.2}×", self.scale))
                            .tone(Tone::Accent)
                            .width(0.33),
                    )
                    .child(
                        Stat::new("Angle", format!("{}°", self.angle.round()))
                            .tone(Tone::Info)
                            .width(0.34),
                    ),
            )
            .child(
                text("Pinch and rotate with two fingers (Option-drag in the simulator).")
                    .text_size(px(13.0))
                    .line_height(px(18.0))
                    .text_color(colors::MUTED),
            )
            .child(
                Button::new(
                    "Reset",
                    cx.listener(|this: &mut Self, _, cx| {
                        let [x, y, scale, angle] = this.values.as_ref().expect("set up");
                        let _ = x.spring(0.0, Spring::default());
                        let _ = y.spring(0.0, Spring::default());
                        let _ = scale.spring(1.0, Spring::default());
                        let _ = angle.spring(0.0, Spring::default());
                        (this.drag, this.scale, this.angle) = (0.0, 1.0, 0.0);
                        cx.notify();
                    }),
                )
                .variant(ButtonVariant::Secondary),
            )
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

/// A ball dragged as the touch responder (positions go through Rust, a
/// render per move). It claims the touches, so the page does not scroll
/// while it is dragged.
#[derive(Debug)]
struct Ball {
    /// Where the ball is, from where it started.
    pos: (f32, f32),
    /// Where it was when the drag began.
    start: (f32, f32),
    /// What happened last, and its tone.
    state: (String, Tone),
    /// 1 while held.
    grab: Option<NativeValue>,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            pos: (0.0, 0.0),
            start: (0.0, 0.0),
            state: ("Idle".into(), Tone::Neutral),
            grab: None,
        }
    }
}

impl Ball {
    fn grab(&self) -> &NativeValue {
        self.grab.as_ref().expect("set up")
    }
}

impl Component for Ball {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.grab = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let grab = self.grab();
        Card::new()
            .icon(IconName::Scroll)
            .title("PanResponder")
            .subtitle("Positions go through Rust; the page does not scroll")
            .right(Badge::new(self.state.0.clone()).tone(self.state.1))
            .child(
                stage().h(px(200.0)).child(dots()).child(
                    view()
                        .absolute()
                        .left(relative(0.5))
                        .top(px(70.0))
                        .ml(px(-30.0))
                        .size(px(60.0))
                        .rounded(px(30.0))
                        .items_center()
                        .justify_center()
                        .bind_background(&grab.interpolate_colors(
                            &[0.0, 1.0],
                            &[cx.color(colors::INFO), cx.color(colors::ACCENT_FILL)],
                        ))
                        .map(knob)
                        .translate_x(self.pos.0)
                        .translate_y(self.pos.1)
                        .bind_scale(&grab.interpolate(&[0.0, 1.0], &[1.0, 1.15]))
                        .claim_touches(TouchClaim::on_start())
                        .on_responder_grant(cx.listener(|this: &mut Self, _, cx| {
                            this.start = this.pos;
                            let _ = this.grab().spring(1.0, spring(420.0, 24.0));
                            this.state = ("Dragging".into(), Tone::Accent);
                            cx.notify();
                        }))
                        .on_responder_move(cx.listener(
                            |this: &mut Self, e: &ResponderEvent, cx| {
                                let g = e.gesture;
                                this.pos = (this.start.0 + g.dx, this.start.1 + g.dy);
                                cx.notify();
                            },
                        ))
                        .on_responder_release(cx.listener(
                            |this: &mut Self, e: &ResponderEvent, cx| {
                                let g = e.gesture;
                                let _ = this.grab().spring(0.0, spring(300.0, 20.0));
                                this.state =
                                    (format!("{:.2}, {:.2} pt/ms", g.vx, g.vy), Tone::Info);
                                cx.notify();
                            },
                        ))
                        .on_responder_terminate(cx.listener(|this: &mut Self, _, cx| {
                            let _ = this.grab().spring(0.0, Spring::default());
                            this.state = ("Taken over".into(), Tone::Warn);
                            cx.notify();
                        }))
                        .child(
                            view()
                                .size(px(18.0))
                                .rounded(px(9.0))
                                .bg(rgb_a(0x13160c, 0.35)),
                        ),
                ),
            )
            .child(caption(
                "Position",
                Some(format!("{}, {}", self.pos.0.round(), self.pos.1.round())),
            ))
            .child(
                Button::new(
                    "Center",
                    cx.listener(|this: &mut Self, _, cx| {
                        this.pos = (0.0, 0.0);
                        cx.notify();
                    }),
                )
                .variant(ButtonVariant::Secondary),
            )
    }
}

/// Raw touch events, every finger drawn where it is.
#[derive(Debug, Default)]
struct TouchPad {
    /// The latest first: the event's name and what it says.
    log: Vec<(&'static str, String)>,
    /// The fingers down, where they are in the pad.
    fingers: Vec<(f32, f32)>,
}

impl Component for TouchPad {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let down = self.fingers.len();
        let log: Vec<AnyElement> = self
            .log
            .iter()
            .enumerate()
            .map(|(i, (name, entry))| {
                view()
                    .flex_row()
                    .items_baseline()
                    .gap_2()
                    .child(
                        text(*name)
                            .w(px(52.0))
                            .text_size(px(12.0))
                            .font_family(MONO)
                            .text_color(if i == 0 {
                                colors::ACCENT
                            } else {
                                colors::FAINT
                            }),
                    )
                    .child(
                        text(entry.clone())
                            .number_of_lines(1)
                            .flex_1()
                            .text_size(px(13.0))
                            .text_color(if i == 0 { colors::TEXT } else { colors::FAINT })
                            .tabular_nums(),
                    )
                    .into_element()
            })
            .collect();
        Card::new()
            .icon(IconName::System)
            .title("Touch events")
            .subtitle("Start, move, end and cancel, for every finger")
            .right(Badge::new(format!("{down} down")).tone(if down > 0 {
                Tone::Accent
            } else {
                Tone::Neutral
            }))
            .child(
                stage()
                    .h(px(150.0))
                    .items_center()
                    .justify_center()
                    .on_touch(cx.listener(|this: &mut Self, touch: &TouchEvent, cx| {
                        let (down, about) = fingers(touch);
                        let Some(p) = about else { return };
                        let name = match touch.phase {
                            TouchPhase::Start => "start",
                            TouchPhase::Move => "move",
                            TouchPhase::End => "end",
                            TouchPhase::Cancel => "cancel",
                        };
                        this.fingers = if touch.phase == TouchPhase::Cancel {
                            Vec::new()
                        } else {
                            down.iter().map(|f| (f.location_x, f.location_y)).collect()
                        };
                        let entry = format!(
                            "{}, {} · page {}, {} · {} down",
                            p.location_x.round(),
                            p.location_y.round(),
                            p.page_x.round(),
                            p.page_y.round(),
                            down.len()
                        );
                        this.log.insert(0, (name, entry));
                        this.log.truncate(4);
                        cx.notify();
                    }))
                    .child(dots())
                    .when(down == 0, |v| {
                        v.child(
                            text("Touch here, try two fingers")
                                .text_size(px(14.0))
                                .text_color(colors::MUTED),
                        )
                    })
                    .children(self.fingers.iter().map(|&(x, y)| {
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .left(px(x - 26.0))
                            .top(px(y - 26.0))
                            .size(px(52.0))
                            .rounded(px(26.0))
                            .border_2()
                            .border_color(colors::ACCENT)
                            .bg(tints::ACCENT)
                    })),
            )
            .child(view().gap_1().min_h(px(4.0 * 18.0)).map(|v| {
                if log.is_empty() {
                    v.child(
                        text("No touches yet")
                            .text_size(px(13.0))
                            .text_color(colors::FAINT),
                    )
                } else {
                    v.children(log)
                }
            }))
    }
}

/// The press events, in the order of their chips.
const PRESS_EVENTS: [&str; 4] = ["pressIn", "press", "longPress", "pressOut"];

/// Each press event lights up its chip as it arrives.
#[derive(Debug, Default)]
struct PressEvents {
    flashes: Vec<NativeValue>,
    count: u32,
    /// The latest first.
    log: Vec<&'static str>,
}

impl PressEvents {
    /// A listener noting `event`.
    fn fire(cx: &RenderCx<'_, Self>, event: &'static str) -> Listener<()> {
        cx.listener(move |this: &mut Self, _, cx| {
            this.count += 1;
            this.log.insert(0, event);
            this.log.truncate(6);
            if let Some(i) = PRESS_EVENTS.iter().position(|e| *e == event) {
                let flash = &this.flashes[i];
                flash.set(1.0);
                let _ = flash.timing(0.0, 700.0, Easing::Curve(Curve::Quad, EaseMode::Out));
            }
            cx.notify();
        })
    }
}

impl Component for PressEvents {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.flashes = PRESS_EVENTS.iter().map(|_| cx.native_value(0.0)).collect();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (raised, fill) = (cx.color(colors::RAISED), cx.color(colors::ACCENT_FILL));
        let log = if self.log.is_empty() {
            "No events yet".to_owned()
        } else {
            self.log.join(" · ")
        };
        Card::new()
            .icon(IconName::Components)
            .title("Press events")
            .subtitle("In, press, long press and out; native feedback")
            .right(Badge::new(self.count.to_string()))
            .child(
                view()
                    .on_press_in(Self::fire(cx, "pressIn"))
                    .on_press_out(Self::fire(cx, "pressOut"))
                    .on_press(Self::fire(cx, "press"))
                    .on_long_press(Self::fire(cx, "longPress"))
                    .active_opacity(0.55)
                    .role(AccessibilityRole::Button)
                    .h(px(64.0))
                    .rounded(px(radius::CONTROL))
                    .border_curve(BorderCurve::Continuous)
                    .bg(colors::LIGHT)
                    .finish(Finish::Light)
                    .items_center()
                    .justify_center()
                    .child(
                        text("Tap, hold, or slide off")
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::ON_LIGHT),
                    ),
            )
            .child(view().flex_row().gap(px(6.0)).children(
                PRESS_EVENTS.iter().zip(&self.flashes).map(|(name, flash)| {
                    view()
                        .flex_1()
                        .h(px(30.0))
                        .rounded(px(15.0))
                        .items_center()
                        .justify_center()
                        .border_1()
                        .border_color(colors::BORDER)
                        .bind_background(&flash.interpolate_colors(&[0.0, 1.0], &[raised, fill]))
                        .finish_shadow(Finish::Raised)
                        .child(
                            text(*name)
                                .text_size(px(11.0))
                                .font_family(MONO)
                                .text_color(colors::TEXT),
                        )
                }),
            ))
            .child(
                text(log)
                    .number_of_lines(1)
                    .text_size(px(12.0))
                    .font_family(MONO)
                    .text_color(colors::FAINT),
            )
            .child(
                view()
                    .flex_row()
                    .gap(px(10.0))
                    .child(
                        view()
                            .on_press(Self::fire(cx, "ripple"))
                            .on_long_press(Self::fire(cx, "fast long press"))
                            .long_press_delay(250.0)
                            .underlay(cx.color(colors::BORDER), UNDERLAY_OPACITY)
                            .ripple(Ripple {
                                color: Some(rgb_a(0xc8f26d, 0.25)),
                                ..Ripple::default()
                            })
                            .flex_1()
                            .h(px(56.0))
                            .rounded(px(radius::CONTROL))
                            .border_curve(BorderCurve::Continuous)
                            .bg(colors::RAISED)
                            .finish(Finish::Raised)
                            .border_1()
                            .border_color(colors::BORDER)
                            .items_center()
                            .justify_center()
                            .gap(px(2.0))
                            .child(text("Ripple").text_size(px(14.0)).text_color(colors::TEXT))
                            .child(
                                text("Long press after 250 ms")
                                    .text_size(px(12.0))
                                    .text_color(colors::FAINT),
                            ),
                    )
                    .child(
                        view()
                            .on_press(Self::fire(cx, "borderless"))
                            .ripple(Ripple {
                                borderless: true,
                                radius: Some(40.0),
                                ..Ripple::default()
                            })
                            .active_opacity(0.4)
                            .role(AccessibilityRole::Button)
                            .label("Star")
                            .size(px(56.0))
                            .items_center()
                            .justify_center()
                            .child(icon(IconName::Star, 24.0, colors::WARN)),
                    ),
            )
            .child(
                text("Pressable text, dimmed while pressed")
                    .on_press(Self::fire(cx, "text"))
                    .active_opacity(0.3)
                    .self_start()
                    .text_size(px(15.0))
                    .text_color(colors::ACCENT)
                    .py_3(),
            )
    }
}

/// A tiny dot with a ring showing the 30 pt around it that also takes taps.
#[derive(Debug, Default)]
struct HitSlop {
    taps: u32,
    ring: Option<NativeValue>,
    breathe: Option<Animation>,
}

impl Component for HitSlop {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        let ring = cx.native_value(0.0);
        let sin = Easing::Curve(Curve::Sin, EaseMode::InOut);
        let breathe = Animation::sequence([
            Animation::timing(&ring, 1.0, Timing::new(1400.0, sin)),
            Animation::timing(&ring, 0.0, Timing::new(1400.0, sin)),
        ])
        .looped(None);
        let _ = breathe.start();
        self.ring = Some(ring);
        self.breathe = Some(breathe);
    }

    fn cleanup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        if let Some(breathe) = &self.breathe {
            breathe.stop();
        }
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let ring = self.ring.as_ref().expect("set up");
        Card::new()
            .icon(IconName::Search)
            .title("Hit slop")
            .subtitle("A 12 pt dot that takes taps 30 pt around it")
            .right(
                Badge::new(format!("{} taps", self.taps)).tone(if self.taps > 0 {
                    Tone::Accent
                } else {
                    Tone::Neutral
                }),
            )
            .child(
                stage()
                    .h(px(140.0))
                    .items_center()
                    .justify_center()
                    .child(dots())
                    .child(
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .size(px(72.0))
                            .rounded(px(36.0))
                            .border_1()
                            .border_dashed()
                            .border_color(colors::DANGER)
                            .bg(tints::DANGER)
                            .bind_opacity(&ring.interpolate(&[0.0, 1.0], &[0.45, 1.0]))
                            .bind_scale(&ring.interpolate(&[0.0, 1.0], &[0.96, 1.04])),
                    )
                    .child(
                        press_with(
                            PressProps {
                                hit_slop: Some(30.0),
                                scale: 0.8,
                                label: Some("Tiny dot".into()),
                                ..PressProps::default()
                            },
                            cx.listener(|this: &mut Self, _, cx| {
                                this.taps += 1;
                                cx.notify();
                            }),
                        )
                        .child(
                            view()
                                .size(px(12.0))
                                .rounded(px(6.0))
                                .bg(colors::DANGER)
                                .map(knob),
                        ),
                    ),
            )
            .child(api("The dashed ring is the touch area", "hitSlop={30}"))
    }
}

/// The items labelled for screen readers: label, hint, icon.
const LABELLED: [(&str, &str, IconName); 3] = [
    ("Play", "Starts the song", IconName::Play),
    ("Like", "Adds to favorites", IconName::Star),
    ("Notify", "Turns on alerts", IconName::Bell),
];

/// The demo's page; it keeps the state of the pointer events and the
/// accessibility cards.
#[derive(Debug, Default)]
pub(crate) struct Gestures {
    /// Taps through the overlay.
    through: u32,
    /// The overlay's width, from its layout.
    overlay: f32,
    /// The item pressed last.
    last: Option<&'static str>,
}

impl Gestures {
    /// A hatched band across the middle, framed with dashes.
    fn hatch(&self, cx: &RenderCx<'_, Self>) -> Painter {
        let (width, height) = (self.overlay, 88.0);
        let mut p = Painter::new()
            .save()
            .begin_path()
            .rect(width * 0.18, 0.0, width * 0.64, height)
            .clip()
            .stroke_color(cx.color(colors::WARN.opacity(0.28)))
            .line_width(1.0)
            .begin_path();
        let mut i = -height;
        while i < width {
            p = p.move_to(i, height).line_to(i + height, 0.0);
            i += 8.0;
        }
        p.stroke()
            .restore()
            .line_dash(&[4.0, 4.0])
            .stroke_color(cx.color(colors::WARN))
            .begin_path()
            .round_rect(
                width * 0.18 + 0.5,
                0.5,
                width * 0.64 - 1.0,
                height - 1.0,
                10.0,
            )
            .stroke()
    }

    /// An overlay that lets taps through to the button under it.
    fn pointer_events(&self, cx: &mut RenderCx<'_, Self>) -> impl IntoElement + use<> {
        Card::new()
            .icon(IconName::Nav)
            .title("Pointer events")
            .subtitle("An overlay that lets taps through")
            .right(Badge::new(self.through.to_string()).tone(Tone::Up))
            .child(
                view()
                    .h(px(88.0))
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.overlay != frame.width {
                            this.overlay = frame.width;
                            cx.notify();
                        }
                    }))
                    .child(
                        press_with(
                            PressProps {
                                scale: 0.98,
                                ..PressProps::default()
                            },
                            cx.listener(|this: &mut Self, _, cx| {
                                this.through += 1;
                                cx.notify();
                            }),
                        )
                        .flex_1()
                        .child(
                            view()
                                .flex_1()
                                .rounded(px(radius::CONTROL))
                                .border_curve(BorderCurve::Continuous)
                                .bg(tints::UP)
                                .border_1()
                                .border_color(colors::UP)
                                .items_center()
                                .justify_center()
                                .child(
                                    text("Button under an overlay")
                                        .text_size(px(15.0))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(colors::UP),
                                ),
                        ),
                    )
                    .child(
                        canvas(self.hatch(cx))
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .inset_0(),
                    ),
            )
            .child(api(
                "Taps on the hatched band reach the button",
                "pointerEvents=\"none\"",
            ))
    }

    /// Elements labelled for VoiceOver / TalkBack.
    fn accessibility(&self, cx: &mut RenderCx<'_, Self>) -> impl IntoElement + use<> {
        let last = self.last;
        Card::new()
            .icon(IconName::Text)
            .title("Accessibility")
            .subtitle("Roles, labels and hints for screen readers")
            .when_some(last, |card, last| {
                card.right(Badge::new(last).tone(Tone::Info))
            })
            .children(
                LABELLED
                    .iter()
                    .enumerate()
                    .map(|(i, &(label, hint, name))| {
                        view()
                            .accessible(true)
                            .role(AccessibilityRole::Button)
                            .label(label)
                            .hint(hint)
                            .on_press(cx.listener(move |this: &mut Self, _, cx| {
                                this.last = Some(label);
                                cx.notify();
                            }))
                            .child(
                                Row::new(i == 0)
                                    .child(IconTile::new(name).size(34.0).tint(
                                        if last == Some(label) {
                                            colors::ACCENT
                                        } else {
                                            colors::TEXT
                                        },
                                    ))
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(1.0))
                                            .child(
                                                text(label)
                                                    .text_size(px(15.0))
                                                    .text_color(colors::TEXT),
                                            )
                                            .child(
                                                text(hint)
                                                    .text_size(px(12.0))
                                                    .text_color(colors::MUTED),
                                            ),
                                    )
                                    .child(
                                        text("button")
                                            .text_size(px(11.0))
                                            .font_family(MONO)
                                            .text_color(colors::FAINT),
                                    ),
                            )
                    }),
            )
    }
}

impl Component for Gestures {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let cards: [AnyElement; 7] = [
            Transformable::element(()).into_element(),
            Ball::element(()).into_element(),
            TouchPad::element(()).into_element(),
            PressEvents::element(()).into_element(),
            HitSlop::element(()).into_element(),
            self.pointer_events(cx).into_element(),
            self.accessibility(cx).into_element(),
        ];
        view().flex_1().child(
            scroll().flex_1().bg(colors::BACKGROUND).child(
                view().p_4().pb(px(40.0)).gap_4().children(
                    cards
                        .into_iter()
                        .enumerate()
                        .map(|(i, card)| appear(i as u32).child(card)),
                ),
            ),
        )
    }
}
