//! Canvas: 2D drawing: paths, gradients, text, images, native animation
//! (as the TSX gallery's `demos/CanvasDemo.tsx`).

use std::{
    cell::Cell,
    f32::consts::{FRAC_PI_2, TAU},
    time::{SystemTime, UNIX_EPOCH},
};

use nsh_core::{
    Rect,
    canvas::{
        CanvasGradient, CanvasTextAlign, CanvasTextBaseline, Composite, FillRule, LineCap,
        LineJoin, svg_path,
    },
    nsh_animation::{Curve, EaseMode},
    ui::{AnyElement, prelude::*},
};

use crate::ui::{button, colors, page, section};

/// A number in `0..1` (TS: `Math.random()`): xorshift, seeded by the clock.
fn random() -> f32 {
    thread_local! {
        static STATE: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0x9e37_79b9_7f4a_7c15, |d| d.as_nanos() as u64)
                | 1,
        );
    }
    STATE.with(|state| {
        let mut x = state.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        state.set(x);
        (x >> 40) as f32 / (1u64 << 24) as f32
    })
}

/// A ring whose sweep is a native value: animating it redraws natively.
#[derive(Debug, Default)]
struct ProgressRing {
    progress: Option<NativeValue>,
    /// Where the ring goes (its label).
    target: f32,
}

impl ProgressRing {
    fn go(&mut self, to: f32, cx: &mut Context<'_, Self>) {
        self.target = to;
        if let Some(progress) = &self.progress {
            // The ring follows natively; nothing waits for the end.
            let _ = progress.timing(to, 900.0, Easing::Curve(Curve::Cubic, EaseMode::InOut));
        }
        cx.notify();
    }
}

impl Component for ProgressRing {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.progress = Some(cx.native_value(0.3));
        self.target = 0.3;
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (x, y, r) = (60.0, 60.0, 50.0);
        let mut ring = Painter::new()
            .line_width(12.0)
            .line_cap(LineCap::Round)
            .stroke_color(rgb(0xe5e7eb))
            .begin_path()
            .arc(x, y, r, 0.0, TAU)
            .stroke()
            .stroke_style(
                CanvasGradient::conic(-FRAC_PI_2, x, y)
                    .add_color_stop(0.0, rgb(0x22d3ee))
                    .add_color_stop(1.0, rgb(0x6366f1)),
            )
            .begin_path();
        if let Some(progress) = &self.progress {
            // The end angle reads the native value on every frame.
            ring = ring.arc(
                x,
                y,
                r,
                -FRAC_PI_2,
                &progress.node().mul(TAU).sub(FRAC_PI_2),
            );
        }
        let ring = ring
            .stroke()
            .fill_color(colors::text())
            .font("600 20px system-ui")
            .text_align(CanvasTextAlign::Center)
            .text_baseline(CanvasTextBaseline::Middle)
            .fill_text(&format!("{}%", (self.target * 100.0).round()), x, y);
        view().child(section(
            "Native animation (progress ring)",
            [view()
                .flex_row()
                .items_center()
                .gap_4()
                .child(canvas(ring).size(px(120.0)))
                .child(
                    view()
                        .gap_2()
                        .child(button("Random", colors::accent()).on_press(cx.listener(
                            |this: &mut Self, _, cx| {
                                this.go((random() * 100.0).round() / 100.0, cx)
                            },
                        )))
                        .child(
                            button("Full", rgb(0x1a7f37))
                                .on_press(cx.listener(|this: &mut Self, _, cx| this.go(1.0, cx))),
                        ),
                )
                .into_element()],
        ))
    }
}

/// A chart: gradient area, a line, labels measured with `measure_text`.
#[derive(Debug)]
struct Chart {
    data: Vec<f32>,
    /// The canvas's width, from its layout.
    width: f32,
}

impl Default for Chart {
    fn default() -> Self {
        Self {
            data: vec![12.0, 30.0, 22.0, 48.0, 36.0, 60.0, 52.0, 75.0, 64.0, 88.0],
            width: 0.0,
        }
    }
}

impl Chart {
    fn draw(&self, cx: &RenderCx<'_, Self>, width: f32, height: f32) -> Painter {
        let values = &self.data;
        let (left, right, top, bottom) = (32.0, 12.0, 12.0, 24.0);
        let (w, h) = (width - left - right, height - top - bottom);
        let max = 100.0;
        let x = |i: usize| left + i as f32 / (values.len() - 1) as f32 * w;
        let y = |v: f32| top + h - v / max * h;
        let blue = rgb(0x3b82f6);
        let font = "11px system-ui";
        let mut p = Painter::new()
            .font(font)
            .fill_color(colors::muted())
            .stroke_color(rgb(0xe5e7eb))
            .line_width(1.0);
        for v in [0.0, 25.0, 50.0, 75.0, 100.0] {
            let label = format!("{v}");
            p = p
                .begin_path()
                .move_to(left, y(v))
                .line_to(width - right, y(v))
                .stroke()
                .fill_text(
                    &label,
                    left - 6.0 - cx.measure_text(font, &label),
                    y(v) + 4.0,
                );
        }
        p = p.begin_path().move_to(x(0), y(values[0]));
        for (i, v) in values.iter().enumerate().skip(1) {
            p = p.line_to(x(i), y(*v));
        }
        p = p
            .line_to(x(values.len() - 1), top + h)
            .line_to(x(0), top + h)
            .close_path()
            .fill_style(
                CanvasGradient::linear(0.0, top, 0.0, top + h)
                    .add_color_stop(0.0, rgba(0x3b82_f659))
                    .add_color_stop(1.0, rgba(0x3b82_f600)),
            )
            .fill()
            .begin_path();
        for (i, v) in values.iter().enumerate() {
            p = if i == 0 {
                p.move_to(x(i), y(*v))
            } else {
                p.line_to(x(i), y(*v))
            };
        }
        p = p
            .stroke_color(blue)
            .line_width(2.5)
            .line_join(LineJoin::Round)
            .stroke()
            .fill_color(rgb(0xffffff))
            .stroke_color(blue)
            .line_width(2.0);
        for (i, v) in values.iter().enumerate() {
            p = p
                .begin_path()
                .arc(x(i), y(*v), 3.5, 0.0, TAU)
                .fill()
                .stroke();
        }
        p
    }
}

impl Component for Chart {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let height = 180.0;
        let drawing = if self.width > 0.0 {
            self.draw(cx, self.width, height)
        } else {
            Painter::new()
        };
        view().child(section(
            "Chart",
            [
                canvas(drawing)
                    .h(px(height))
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.width != frame.width {
                            this.width = frame.width;
                            cx.notify();
                        }
                    }))
                    .into_element(),
                button("New data", colors::accent())
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        for v in &mut this.data {
                            *v = (10.0 + random() * 85.0).round();
                        }
                        cx.notify();
                    }))
                    .into_element(),
            ],
        ))
    }
}

/// A clock: transforms, save / restore, a second hand spinning natively.
#[derive(Debug, Default)]
struct Clock {
    seconds: Option<NativeValue>,
    spin: Option<Task<()>>,
}

impl Component for Clock {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        let seconds = cx.native_value(0.0);
        let spun = seconds.clone();
        // One turn a minute, natively (no work per frame).
        self.spin = Some(cx.spawn(|_, _| async move {
            loop {
                spun.set(0.0);
                if !spun.timing(60.0, 60_000.0, Easing::Linear).await {
                    break;
                }
            }
        }));
        self.seconds = Some(seconds);
    }

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let size = 160.0;
        let r = size / 2.0;
        let ink = rgb(0x0f172a);
        let red = rgb(0xef4444);
        let mut p = Painter::new()
            .translate(r, r)
            .fill_color(rgb(0xf8fafc))
            .stroke_color(rgb(0x334155))
            .line_width(3.0)
            .begin_path()
            .arc(0.0, 0.0, r - 4.0, 0.0, TAU)
            .fill()
            .stroke();
        for i in 0..60 {
            let hour = i % 5 == 0;
            p = p
                .save()
                .rotate(i as f32 / 60.0 * TAU)
                .line_width(if hour { 3.0 } else { 1.0 })
                .begin_path()
                .move_to(r - 10.0, 0.0)
                .line_to(r - if hour { 20.0 } else { 14.0 }, 0.0)
                .stroke()
                .restore();
        }
        let hand = |p: Painter, angle: f32, length: f32, width: f32| {
            p.save()
                .rotate(angle - FRAC_PI_2)
                .stroke_color(ink)
                .line_width(width)
                .line_cap(LineCap::Round)
                .begin_path()
                .move_to(-8.0, 0.0)
                .line_to(length, 0.0)
                .stroke()
                .restore()
        };
        p = hand(p, 10.0 / 12.0 * TAU, r * 0.5, 5.0);
        p = hand(p, 10.0 / 60.0 * TAU, r * 0.72, 3.5);
        p = p.save();
        if let Some(seconds) = &self.seconds {
            // A turn per minute, as a native rotation.
            p = p.rotate(&seconds.node().mul(TAU / 60.0));
        }
        let p = p
            .rotate(-FRAC_PI_2)
            .stroke_color(red)
            .line_width(1.5)
            .begin_path()
            .move_to(-14.0, 0.0)
            .line_to(r * 0.8, 0.0)
            .stroke()
            .restore()
            .fill_color(red)
            .begin_path()
            .arc(0.0, 0.0, 4.0, 0.0, TAU)
            .fill();
        view().child(section(
            "Transforms (a clock)",
            [canvas(p).size(px(size)).self_center().into_element()],
        ))
    }
}

/// SVG path data (TS: `Path2D`): icons.
fn icons() -> AnyElement {
    let icons = [
        (
            "heart",
            "M12 21s-7-4.35-9.5-8.5C.5 8.5 3 4 7 4c2 0 3.5 1 5 3 1.5-2 3-3 5-3 4 0 6.5 4.5 4.5 8.5C19 16.65 12 21 12 21z",
            rgb(0xef4444),
        ),
        (
            "star",
            "M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 21 12 17.27 5.82 21 7 14.14l-5-4.87 6.91-1.01z",
            rgb(0xf59e0b),
        ),
        ("bolt", "M13 2L3 14h7l-1 8 10-12h-7z", rgb(0x8b5cf6)),
        ("check", "M20 6L9 17l-5-5", rgb(0x10b981)),
    ];
    let icons = icons.map(|(name, d, color)| {
        let p = Painter::new().scale(40.0 / 24.0, 40.0 / 24.0);
        let path = svg_path(d);
        let p = if name == "check" {
            p.stroke_color(color)
                .line_width(2.5)
                .line_cap(LineCap::Round)
                .line_join(LineJoin::Round)
                .stroke_path(&path)
        } else {
            p.fill_color(color).fill_path(&path, FillRule::NonZero)
        };
        view()
            .items_center()
            .gap_1()
            .child(canvas(p).size(px(40.0)))
            .child(text(name).text_size(px(12.0)).text_color(colors::muted()))
            .into_element()
    });
    section(
        "Path2D (SVG path data)",
        [view().flex_row().gap_4().children(icons).into_element()],
    )
    .into_element()
}

/// Images, compositing, shadows, dashes.
#[derive(Debug, Default)]
struct Compositing {
    /// The canvas's width, from its layout.
    width: f32,
}

impl Compositing {
    fn draw(&self) -> Painter {
        let width = self.width;
        Painter::new()
            // An image clipped to a circle.
            .save()
            .begin_path()
            .arc(60.0, 60.0, 48.0, 0.0, TAU)
            .clip()
            .draw_image(crate::assets::LOGO.uri, 12.0, 12.0, 96.0, 96.0)
            .restore()
            // A shadowed card with a cut-out hole (destination-out).
            .save()
            .shadow(rgba(0x0000_0040), 12.0, (0.0, 4.0))
            .fill_color(rgb(0x6366f1))
            .begin_path()
            .round_rect(130.0, 16.0, 110.0, 90.0, 14.0)
            .fill()
            .restore()
            .composite(Composite::DestinationOut)
            .begin_path()
            .arc(185.0, 61.0, 22.0, 0.0, TAU)
            .fill()
            .composite(Composite::SourceOver)
            // Dashes and a radial gradient.
            .fill_style(
                CanvasGradient::radial(width - 50.0, 50.0, 4.0, width - 50.0, 50.0, 44.0)
                    .add_color_stop(0.0, rgb(0xfde68a))
                    .add_color_stop(1.0, rgb(0xf97316)),
            )
            .begin_path()
            .arc(width - 50.0, 60.0, 40.0, 0.0, TAU)
            .fill()
            .line_dash(&[6.0, 5.0])
            .stroke_color(rgb(0x9a3412))
            .line_width(2.0)
            .stroke()
    }
}

impl Component for Compositing {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let drawing = if self.width > 0.0 {
            self.draw()
        } else {
            Painter::new()
        };
        view().child(section(
            "Images, compositing, shadows",
            [canvas(drawing)
                .h(px(130.0))
                .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                    if this.width != frame.width {
                        this.width = frame.width;
                        cx.notify();
                    }
                }))
                .into_element()],
        ))
    }
}

/// Touch drawing: each move draws again. The canvas claims the touches
/// and keeps them: the page around it does not scroll while drawing.
#[derive(Debug, Default)]
struct Sketch {
    strokes: Vec<Vec<(f32, f32)>>,
}

impl Component for Sketch {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let mut p = Painter::new()
            .stroke_color(rgb(0x0f172a))
            .line_width(3.0)
            .line_cap(LineCap::Round)
            .line_join(LineJoin::Round);
        for stroke in &self.strokes {
            p = p.begin_path();
            for (i, &(x, y)) in stroke.iter().enumerate() {
                p = if i == 0 {
                    p.move_to(x, y)
                } else {
                    p.line_to(x, y)
                };
            }
            p = p.stroke();
        }
        let sketch = canvas(p)
            .h(px(180.0))
            .bg(rgb(0xf8fafc))
            .rounded(px(10.0))
            .claim_touches(TouchClaim::on_start())
            .keep_touches(true)
            .on_responder_grant(cx.listener(|this: &mut Self, e: &ResponderEvent, cx| {
                this.strokes.push(vec![(e.location_x, e.location_y)]);
                cx.notify();
            }))
            .on_responder_move(cx.listener(|this: &mut Self, e: &ResponderEvent, cx| {
                if let Some(stroke) = this.strokes.last_mut() {
                    stroke.push((e.location_x, e.location_y));
                    cx.notify();
                }
            }));
        view().child(section(
            "Sketch (touch → draw)",
            [
                sketch.into_element(),
                button("Clear", colors::gray())
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        this.strokes.clear();
                        cx.notify();
                    }))
                    .into_element(),
            ],
        ))
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct CanvasDemo;

impl Component for CanvasDemo {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), _cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().flex_1().child(page([
            ProgressRing::element(()).into_element(),
            Chart::element(()).into_element(),
            Clock::element(()).into_element(),
            icons(),
            Compositing::element(()).into_element(),
            Sketch::element(()).into_element(),
        ]))
    }
}
