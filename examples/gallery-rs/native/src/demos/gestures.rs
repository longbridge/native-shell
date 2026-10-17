//! Gestures: a card dragged, pinched and rotated natively, a ball dragged
//! as the touch responder, raw touches, press events, hit slop, pointer
//! events and accessibility labels (as the TSX gallery's
//! `demos/Gestures.tsx`).

use nsh_core::{
    Ripple, TouchPhase, TouchPoint,
    ui::{Text, TouchEvent, prelude::*},
};

use crate::ui::legacy::{button, colors, page, section};

/// A muted note under a demo.
fn note(content: impl Into<SharedString>) -> Text {
    text(content)
        .text_size(px(13.0))
        .text_color(colors::muted())
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

/// Drag, pinch and rotate one card at the same time, all on the UI thread.
#[derive(Debug, Default)]
struct Card {
    values: Option<CardValues>,
}

/// The card's native values: where it is, its scale and its angle.
#[derive(Debug)]
struct CardValues {
    x: NativeValue,
    y: NativeValue,
    scale: NativeValue,
    angle: NativeValue,
}

impl Card {
    fn values(&self) -> &CardValues {
        self.values.as_ref().expect("set up")
    }
}

impl Component for Card {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.values = Some(CardValues {
            x: cx.native_value(0.0),
            y: cx.native_value(0.0),
            scale: cx.native_value(1.0),
            angle: cx.native_value(0.0),
        });
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let CardValues { x, y, scale, angle } = self.values();
        view()
            .gap_3()
            .items_center()
            .child(
                view()
                    .h(px(220.0))
                    .self_stretch()
                    .items_center()
                    .justify_center()
                    .overflow_hidden()
                    .child(
                        view()
                            .w(px(200.0))
                            .h(px(150.0))
                            .rounded(px(14.0))
                            .overflow_hidden()
                            .pan_values(Some(x), Some(y))
                            .pinch_value(scale)
                            .rotation_value(angle)
                            .bind_translate_x(x)
                            .bind_translate_y(y)
                            .bind_scale(scale)
                            .bind_rotate(angle)
                            .child(image("https://picsum.photos/seed/nsh-card/400/300").flex_1()),
                    ),
            )
            .child(note(
                "Drag with one finger; pinch and rotate with two (Option-drag in the simulator).",
            ))
            .child(button("Reset", colors::gray()).on_press(cx.listener(
                |this: &mut Self, _, _| {
                    let CardValues { x, y, scale, angle } = this.values();
                    x.spring(0.0, Spring::default());
                    y.spring(0.0, Spring::default());
                    scale.spring(1.0, Spring::default());
                    angle.spring(0.0, Spring::default());
                },
            )))
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
    /// What happened last.
    state: String,
}

impl Default for Ball {
    fn default() -> Self {
        Self {
            pos: (0.0, 0.0),
            start: (0.0, 0.0),
            state: "idle".into(),
        }
    }
}

impl Component for Ball {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .gap_2()
            .child(
                view()
                    .h(px(180.0))
                    .rounded(px(10.0))
                    .bg(rgb(0xf6f8fa))
                    .overflow_hidden()
                    .child(
                        view()
                            .absolute()
                            .left(px(130.0))
                            .top(px(60.0))
                            .size(px(60.0))
                            .rounded(px(30.0))
                            .bg(colors::accent())
                            .translate_x(self.pos.0)
                            .translate_y(self.pos.1)
                            .claim_touches(TouchClaim::on_start())
                            .on_responder_grant(cx.listener(|this: &mut Self, _, cx| {
                                this.start = this.pos;
                                this.state = "dragging".into();
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
                                    this.state =
                                        format!("released at {:.2}, {:.2} pt/ms", g.vx, g.vy);
                                    cx.notify();
                                },
                            ))
                            .on_responder_terminate(cx.listener(|this: &mut Self, _, cx| {
                                this.state = "taken over".into();
                                cx.notify();
                            })),
                    ),
            )
            .child(note(self.state.clone()))
            .child(button("Reset", colors::gray()).on_press(cx.listener(
                |this: &mut Self, _, cx| {
                    this.pos = (0.0, 0.0);
                    cx.notify();
                },
            )))
    }
}

/// Raw touch events, with every finger.
#[derive(Debug, Default)]
struct TouchLog {
    /// The latest first.
    log: Vec<String>,
}

impl Component for TouchLog {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let log = if self.log.is_empty() {
            "No touches yet".to_owned()
        } else {
            self.log.join("\n")
        };
        view()
            .gap_2()
            .child(
                view()
                    .h(px(100.0))
                    .rounded(px(10.0))
                    .bg(rgb(0xddf4ff))
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
                        let entry = format!(
                            "{name} at {},{} (page {},{}), {} down",
                            p.location_x.round(),
                            p.location_y.round(),
                            p.page_x.round(),
                            p.page_y.round(),
                            down.len()
                        );
                        this.log.insert(0, entry);
                        this.log.truncate(4);
                        cx.notify();
                    }))
                    .child(text("Touch here (try two fingers)").text_color(colors::accent())),
            )
            .child(note(log))
    }
}

/// The demo's page; it keeps the log of the press events and the counts of
/// the hit slop and pointer events demos.
#[derive(Debug, Default)]
pub(crate) struct Gestures {
    /// The latest first.
    log: Vec<String>,
    /// Taps on the small dot.
    slop: u32,
    /// Taps through the overlay.
    through: u32,
}

impl Gestures {
    /// A listener adding `event` to the log.
    fn add(cx: &RenderCx<'_, Self>, event: &'static str) -> Listener<()> {
        cx.listener(move |this: &mut Self, _, cx| {
            this.log.insert(0, event.to_owned());
            this.log.truncate(6);
            cx.notify();
        })
    }
}

impl Component for Gestures {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let log = if self.log.is_empty() {
            "No events yet".to_owned()
        } else {
            self.log.join(" · ")
        };
        let press = [
            view()
                .p(px(18.0))
                .rounded(px(10.0))
                .bg(rgb(0xddf4ff))
                .items_center()
                .active_opacity(0.5)
                .on_press_in(Self::add(cx, "pressIn"))
                .on_press_out(Self::add(cx, "pressOut"))
                .on_press(Self::add(cx, "press"))
                .on_long_press(Self::add(cx, "longPress"))
                .child(text("Tap, hold, or slide off").font_weight(FontWeight::SEMIBOLD))
                .into_element(),
            text(log).text_color(colors::muted()).into_element(),
            view()
                .flex_row()
                .gap(px(10.0))
                .child(
                    view()
                        .flex_1()
                        .p(px(14.0))
                        .rounded(px(10.0))
                        .bg(rgb(0xf6f8fa))
                        .items_center()
                        .on_press(Self::add(cx, "ripple"))
                        .on_long_press(Self::add(cx, "fast long press"))
                        .long_press_delay(250.0)
                        .ripple(Ripple {
                            color: Some(rgba(0x0969_da4d)),
                            ..Ripple::default()
                        })
                        .child(text("Ripple, long press 250 ms")),
                )
                .child(
                    view()
                        .size(px(56.0))
                        .items_center()
                        .justify_center()
                        .on_press(Self::add(cx, "borderless"))
                        .ripple(Ripple {
                            borderless: true,
                            radius: Some(40.0),
                            ..Ripple::default()
                        })
                        .child(text("★").text_size(px(22.0))),
                )
                .into_element(),
            text("Text with activeOpacity")
                .text_color(colors::accent())
                .active_opacity(0.3)
                .on_press(Self::add(cx, "text"))
                .into_element(),
        ];
        let hit_slop = view()
            .flex_row()
            .items_center()
            .gap_4()
            .child(
                view()
                    .size(px(12.0))
                    .rounded(px(6.0))
                    .bg(rgb(0xcf222e))
                    .m_5()
                    .hit_slop(EdgeInsets::all(30.0))
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        this.slop += 1;
                        cx.notify();
                    })),
            )
            .child(text(format!("The dot takes taps 30 pt around it: {}", self.slop)).flex_1());
        let pointer_events = [
            view()
                .h(px(70.0))
                .child(
                    view()
                        .flex_1()
                        .rounded(px(10.0))
                        .bg(rgb(0x1a7f37))
                        .items_center()
                        .justify_center()
                        .on_press(cx.listener(|this: &mut Self, _, cx| {
                            this.through += 1;
                            cx.notify();
                        }))
                        .child(
                            text(format!("Button under an overlay: {}", self.through))
                                .text_color(rgb(0xffffff)),
                        ),
                )
                .child(
                    view()
                        .absolute()
                        .inset(px(0.0))
                        .rounded(px(10.0))
                        .border(3.0)
                        .border_color(rgb(0xbf8700))
                        .border_dashed()
                        .pointer_events(PointerEvents::None),
                )
                .into_element(),
            note("The dashed overlay has pointerEvents=\"none\": taps reach the button.")
                .into_element(),
        ];
        let accessibility =
            [("Play", "Starts the song"), ("Like", "Adds to favorites")].map(|(label, hint)| {
                view()
                    .accessible(true)
                    .role(AccessibilityRole::Button)
                    .label(label)
                    .hint(hint)
                    .on_press(Self::add(cx, label))
                    .p_3()
                    .rounded(px(10.0))
                    .bg(rgb(0xeaeef2))
                    .child(text(format!("{label} (labelled for VoiceOver / TalkBack)")))
                    .into_element()
            });
        view().flex_1().child(page([
            section(
                "Pan, pinch and rotation",
                [Card::element(()).into_element()],
            )
            .into_element(),
            section(
                "PanResponder (drag the ball; the page does not scroll)",
                [Ball::element(()).into_element()],
            )
            .into_element(),
            section("Touch events", [TouchLog::element(()).into_element()]).into_element(),
            section("Press events", press).into_element(),
            section("Hit slop", [hit_slop.into_element()]).into_element(),
            section("Pointer events", pointer_events).into_element(),
            section("Accessibility", accessibility).into_element(),
        ]))
    }
}
