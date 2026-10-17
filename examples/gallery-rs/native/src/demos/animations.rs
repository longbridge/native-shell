//! Animations: easing curves, composed animations (sequence, parallel,
//! stagger, loop) with stop and reset, springs, interpolation, `diff_clamp`
//! and layout animation (as the TSX gallery's `demos/Animations.tsx`).

use nsh_core::ui::{LayoutAnimation, prelude::*};

use crate::ui::legacy::{button, colors, page, section};

/// How far the dots and squares travel.
const TRACK: f32 = 220.0;

/// The colors of the composition's squares and of the layout's bars.
const PALETTE: [u32; 5] = [0xfd8c73, 0xf2cc60, 0x4ac26b, 0x54aeff, 0x8250df];

/// React Native's `Easing.ease` (CSS `ease`).
const EASE: Easing = Easing::Bezier(0.25, 0.1, 0.25, 1.0);

/// `Easing.inOut(Easing.cubic)`.
const CUBIC_IN_OUT: Easing = Easing::Curve(Curve::Cubic, EaseMode::InOut);

/// `Easing.out(Easing.back())`.
const BACK_OUT: Easing = Easing::Curve(Curve::Back(1.70158), EaseMode::Out);

/// `Easing.sin`.
const SIN: Easing = Easing::Curve(Curve::Sin, EaseMode::In);

/// A pill-shaped text button.
fn chip(label: impl Into<SharedString>, on_press: Listener<()>) -> Text {
    text(label)
        .px_3()
        .py(px(6.0))
        .rounded(px(14.0))
        .overflow_hidden()
        .bg(rgb(0xddf4ff))
        .text_color(colors::accent())
        .font_weight(FontWeight::SEMIBOLD)
        .on_press(on_press)
}

/// A row of a track: its label, then `track`.
fn track_row(label: &str, track: View) -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .gap_2()
        .child(
            text(label.to_owned())
                .w(px(84.0))
                .text_size(px(12.0))
                .text_color(colors::muted()),
        )
        .child(track)
        .into_element()
}

/// One dot per easing, all started together natively.
#[derive(Debug, Default)]
struct Curves {
    /// The dots' offsets, one per curve.
    dots: Vec<NativeValue>,
    /// At the end of the track (the next play goes back).
    there: bool,
}

impl Curves {
    fn curves() -> [(&'static str, Easing); 10] {
        [
            ("linear", Easing::Linear),
            ("ease", EASE),
            ("quad out", Easing::Curve(Curve::Quad, EaseMode::Out)),
            ("cubic inOut", CUBIC_IN_OUT),
            ("sin in", SIN),
            ("circle out", Easing::Curve(Curve::Circle, EaseMode::Out)),
            ("exp inOut", Easing::Curve(Curve::Exp, EaseMode::InOut)),
            ("back out", BACK_OUT),
            (
                "elastic out",
                Easing::Curve(Curve::Elastic(1.0), EaseMode::Out),
            ),
            ("bounce out", Easing::Curve(Curve::Bounce, EaseMode::Out)),
        ]
    }
}

impl Component for Curves {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.dots = Self::curves()
            .iter()
            .map(|_| cx.native_value(0.0))
            .collect();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let rows = Self::curves()
            .into_iter()
            .zip(&self.dots)
            .map(|((label, _), dot)| {
                track_row(
                    label,
                    view()
                        .w(px(TRACK + 14.0))
                        .h(px(14.0))
                        .rounded(px(7.0))
                        .bg(rgb(0xf6f8fa))
                        .child(
                            view()
                                .size(px(14.0))
                                .rounded(px(7.0))
                                .bg(colors::accent())
                                .bind_translate_x(dot),
                        ),
                )
            });
        view()
            .gap(px(6.0))
            .children(rows)
            .child(button("Play", colors::accent()).on_press(cx.listener(
                |this: &mut Self, _, _| {
                    this.there = !this.there;
                    let to = if this.there { TRACK } else { 0.0 };
                    let runs = this
                        .dots
                        .iter()
                        .zip(Self::curves())
                        .map(|(dot, (_, easing))| {
                            Animation::timing(dot, to, Timing::new(1000.0, easing))
                        });
                    // It runs to its end natively; nothing follows it up.
                    let _ = Animation::parallel(runs).start();
                },
            )))
    }
}

/// A composed animation of [`Composition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Run {
    Stagger,
    Sequence,
    Loop,
}

impl Run {
    fn name(self) -> &'static str {
        match self {
            Run::Stagger => "stagger",
            Run::Sequence => "sequence",
            Run::Loop => "loop",
        }
    }
}

/// `sequence`, `parallel`, `stagger` and `loop`, with stop and reset.
#[derive(Debug, Default)]
struct Composition {
    /// The squares' vertical offsets.
    ys: Vec<NativeValue>,
    /// The squares' turn (0..1, each square turning its own angle).
    spin: Option<NativeValue>,
    /// The squares' opacity.
    fade: Option<NativeValue>,
    /// What last happened ("idle", "stagger…", "loop: stopped").
    state: String,
    /// The compositions, by [`Run`].
    runs: Vec<Animation>,
}

impl Composition {
    fn spin(&self) -> &NativeValue {
        self.spin.as_ref().expect("set up")
    }

    fn fade(&self) -> &NativeValue {
        self.fade.as_ref().expect("set up")
    }

    /// Starts `run` (again: a start stops the run going on, which reports
    /// it stopped) and reports how it ends.
    fn run(&mut self, run: Run, cx: &mut Context<'_, Self>) {
        let end = self.runs[run as usize].start();
        cx.spawn(move |this, _| async move {
            let finished = end.await;
            this.update(move |this, cx| {
                let end = if finished { "finished" } else { "stopped" };
                this.state = format!("{}: {end}", run.name());
                cx.notify();
            })
            .await;
        })
        .detach();
        self.state = format!("{}…", run.name());
        cx.notify();
    }
}

impl Component for Composition {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.ys = PALETTE.iter().map(|_| cx.native_value(0.0)).collect();
        let (spin, fade) = (cx.native_value(0.0), cx.native_value(1.0));
        let stagger = Animation::stagger(
            80.0,
            self.ys.iter().map(|y| {
                Animation::sequence([
                    Animation::timing(
                        y,
                        -36.0,
                        Timing::new(220.0, Easing::Curve(Curve::Quad, EaseMode::Out)),
                    ),
                    Animation::spring(
                        y,
                        0.0,
                        Spring {
                            stiffness: 300.0,
                            damping: 12.0,
                            ..Spring::default()
                        },
                    ),
                ])
            }),
        );
        // `Animated.timing`'s curve when it names none.
        let ms = |duration_ms| Timing {
            duration_ms,
            ..Timing::default()
        };
        let sequence = Animation::sequence([
            Animation::timing(&spin, 1.0, Timing::new(500.0, CUBIC_IN_OUT)),
            Animation::delay(200.0),
            Animation::parallel([
                Animation::timing(&fade, 0.2, ms(300.0)),
                Animation::timing(&spin, 0.0, Timing::new(600.0, BACK_OUT)),
            ]),
            Animation::timing(&fade, 1.0, ms(300.0)),
        ]);
        let pulse = Animation::sequence([
            Animation::timing(&fade, 0.3, Timing::new(400.0, SIN)),
            Animation::timing(&fade, 1.0, Timing::new(400.0, SIN)),
        ])
        .looped(None);
        self.runs = vec![stagger, sequence, pulse];
        self.spin = Some(spin);
        self.fade = Some(fade);
        self.state = "idle".into();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let squares = PALETTE
            .iter()
            .zip(&self.ys)
            .enumerate()
            .map(|(i, (&color, y))| {
                let angle = (i + 1) as f32 * 36.0;
                view()
                    .size(px(34.0))
                    .rounded(px(8.0))
                    .bg(rgb(color))
                    .bind_opacity(self.fade())
                    .bind_translate_y(y)
                    .bind_rotate(&self.spin().interpolate(&[0.0, 1.0], &[0.0, angle]))
            });
        view()
            .gap_3()
            .child(
                view()
                    .flex_row()
                    .justify_center()
                    .items_end()
                    .gap(px(10.0))
                    .h(px(80.0))
                    .children(squares),
            )
            .child(
                text(self.state.clone())
                    .text_size(px(13.0))
                    .text_color(colors::muted())
                    .text_center(),
            )
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .child(chip(
                        "Stagger",
                        cx.listener(|this: &mut Self, _, cx| this.run(Run::Stagger, cx)),
                    ))
                    .child(chip(
                        "Sequence",
                        cx.listener(|this: &mut Self, _, cx| this.run(Run::Sequence, cx)),
                    ))
                    .child(chip(
                        "Loop",
                        cx.listener(|this: &mut Self, _, cx| this.run(Run::Loop, cx)),
                    ))
                    .child(chip(
                        "Stop",
                        cx.listener(|this: &mut Self, _, _| {
                            this.runs.iter().for_each(Animation::stop)
                        }),
                    ))
                    .child(chip(
                        "Reset",
                        cx.listener(|this: &mut Self, _, _| {
                            this.runs.iter().for_each(Animation::reset)
                        }),
                    )),
            )
    }
}

/// Springs with different settings, released together.
#[derive(Debug, Default)]
struct Springs {
    /// The squares' offsets, one per setting.
    xs: Vec<NativeValue>,
    /// At the end of the track.
    there: bool,
}

impl Springs {
    fn settings() -> [(&'static str, Spring); 3] {
        let spring = |stiffness, damping| Spring {
            stiffness,
            damping,
            ..Spring::default()
        };
        [
            ("gentle", spring(120.0, 14.0)),
            ("wobbly", spring(180.0, 8.0)),
            ("stiff", spring(400.0, 30.0)),
        ]
    }
}

impl Component for Springs {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.xs = Self::settings()
            .iter()
            .map(|_| cx.native_value(0.0))
            .collect();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let rows = Self::settings()
            .into_iter()
            .zip(&self.xs)
            .map(|((label, _), x)| {
                track_row(
                    label,
                    view().w(px(TRACK + 20.0)).h(px(20.0)).child(
                        view()
                            .size(px(20.0))
                            .rounded(px(4.0))
                            .bg(colors::purple())
                            .bind_translate_x(x),
                    ),
                )
            });
        view()
            .gap(px(6.0))
            .children(rows)
            .child(button("Release", colors::purple()).on_press(cx.listener(
                |this: &mut Self, _, _| {
                    this.there = !this.there;
                    let to = if this.there { TRACK } else { 0.0 };
                    let runs = this
                        .xs
                        .iter()
                        .zip(Self::settings())
                        .map(|(x, (_, spring))| Animation::spring(x, to, spring));
                    let _ = Animation::parallel(runs).start();
                },
            )))
    }
}

/// Colors and angles from one value; outside its range it extends on the
/// right only.
#[derive(Debug, Default)]
struct Interpolations {
    /// The value: 0, 1 or 2 at rest.
    t: Option<NativeValue>,
    /// Where it goes.
    at: u8,
}

impl Interpolations {
    fn t(&self) -> &NativeValue {
        self.t.as_ref().expect("set up")
    }
}

impl Component for Interpolations {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.t = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let t = self.t();
        let color = t.interpolate_colors(
            &[0.0, 1.0, 2.0],
            &[rgb(0x54aeff), rgb(0x8250df), rgb(0xcf222e)],
        );
        let angle = t.interpolate_with(
            &[0.0, 1.0],
            AnimationOutput::Numbers(vec![0.0, 180.0]),
            Interpolation {
                extrapolate_right: Extrapolate::Extend,
                ..Interpolation::default()
            },
        );
        let scale = t.interpolate(&[0.0, 1.0], &[0.6, 1.2]);
        let chips = (0..3u8).map(|n| {
            let label = format!("t = {n}{}", if self.at == n { " ●" } else { "" });
            chip(
                label,
                cx.listener(move |this: &mut Self, _, cx| {
                    this.at = n;
                    let _ = this.t().timing(f32::from(n), 700.0, CUBIC_IN_OUT);
                    cx.notify();
                }),
            )
        });
        view()
            .gap(px(10.0))
            .child(
                view()
                    .flex_row()
                    .justify_center()
                    .gap_6()
                    .h(px(90.0))
                    .items_center()
                    .child(
                        view()
                            .size(px(60.0))
                            .rounded(px(12.0))
                            .bind_background(&color)
                            .bind_rotate(&angle),
                    )
                    .child(
                        view()
                            .size(px(60.0))
                            .rounded(px(30.0))
                            .bg(rgb(0x4ac26b))
                            .bind_scale(&scale),
                    ),
            )
            .child(view().flex_row().gap_2().children(chips))
            .child(
                text("Color through three stops; rotation extends past 1 (to 360° at 2); scale clamps at 1.2.")
                    .text_size(px(12.0))
                    .text_color(colors::muted()),
            )
    }
}

/// A header that hides while the list scrolls up and comes back as it
/// scrolls down: `diff_clamp` of the scroll offset, natively.
#[derive(Debug, Default)]
struct HidingHeader {
    /// The scroll offset.
    scroll: Option<NativeValue>,
}

impl Component for HidingHeader {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.scroll = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let scroll_value = self.scroll.as_ref().expect("set up");
        let header = scroll_value.diff_clamp(0.0, 44.0);
        let rows = (1..=30).map(|n| {
            view()
                .p(px(10.0))
                .rounded(px(8.0))
                .bg(colors::surface())
                .child(text(format!("Row {n}")))
        });
        view()
            .h(px(260.0))
            .rounded(px(10.0))
            .overflow_hidden()
            .bg(rgb(0xf6f8fa))
            .child(
                scroll()
                    .flex_1()
                    .scroll_value(scroll_value)
                    .child(view().pt(px(52.0)).px_3().pb_3().gap_2().children(rows)),
            )
            .child(
                view()
                    .absolute()
                    .left(px(0.0))
                    .right(px(0.0))
                    .top(px(0.0))
                    .h(px(44.0))
                    .justify_center()
                    .px_3()
                    .bg(colors::accent())
                    .bind_translate_y(&header.mul(-1.0))
                    .bind_opacity(&header.interpolate_with(
                        &[0.0, 44.0],
                        AnimationOutput::Numbers(vec![1.0, 0.0]),
                        Interpolation {
                            easing: Easing::Curve(Curve::Quad, EaseMode::In),
                            ..Interpolation::default()
                        },
                    ))
                    .child(
                        text("Scroll up to hide me")
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::BOLD),
                    ),
            )
    }
}

/// Layout changes animated natively by the next commit.
#[derive(Debug, Default)]
struct Layout {
    /// The bars in a column (else in a row).
    open: bool,
}

impl Component for Layout {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let open = self.open;
        let bars = PALETTE.iter().map(|&color| {
            view()
                .h(px(if open { 24.0 } else { 44.0 }))
                .when(!open, |v| v.flex_1())
                .rounded(px(8.0))
                .bg(rgb(color))
        });
        view()
            .gap(px(10.0))
            .child(
                view()
                    .map(|v| if open { v.flex_col() } else { v.flex_row() })
                    .gap_2()
                    .children(bars),
            )
            .child(
                button(if open { "Row" } else { "Column" }, colors::gray()).on_press(cx.listener(
                    |this: &mut Self, _, cx| {
                        cx.animate_layout(LayoutAnimation::SPRING);
                        this.open = !this.open;
                        cx.notify();
                    },
                )),
            )
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Animations;

impl Component for Animations {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), _cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().flex_1().child(page([
            section("Easing curves", [Curves::element(()).into_element()]).into_element(),
            section(
                "Sequence, parallel, stagger, loop",
                [Composition::element(()).into_element()],
            )
            .into_element(),
            section("Springs", [Springs::element(()).into_element()]).into_element(),
            section(
                "Interpolation",
                [Interpolations::element(()).into_element()],
            )
            .into_element(),
            section(
                "diffClamp: a header that hides on scroll",
                [HidingHeader::element(()).into_element()],
            )
            .into_element(),
            section("Layout animation", [Layout::element(()).into_element()]).into_element(),
        ]))
    }
}
