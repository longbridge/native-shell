//! Canvas: 2D drawing: paths, gradients, text, images, native animation
//! (as the TSX gallery's `demos/CanvasDemo.tsx`).

use std::{
    cell::Cell,
    f32::consts::{FRAC_PI_2, TAU},
    time::{SystemTime, UNIX_EPOCH},
};

use nsh_core::{
    Rect, TouchEvent, TouchPhase,
    canvas::{CanvasGradient, CanvasTextAlign, CanvasTextBaseline, Composite, LineCap, LineJoin},
    nsh_anim::{Curve, EaseMode},
    ui::{AnyElement, prelude::*},
};

use crate::ui::{button, colors, page, section};

/// The gallery's logo, as the TSX gallery imports it (`../logo@2x.png`).
const LOGO: &str = "asset:/logo@2x.png";

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
    const NAME: &'static str = "gallery.canvas.progress_ring";
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
            // TS strokes a conic gradient (#22d3ee to #6366f1): `Painter`
            // has no gradient strokes nor conic gradients.
            .stroke_color(rgb(0x6366f1))
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

/// A chart: gradient area, a line, labels.
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
    fn draw(&self, width: f32, height: f32) -> Painter {
        let values = &self.data;
        let (left, right, top, bottom) = (32.0, 12.0, 12.0, 24.0);
        let (w, h) = (width - left - right, height - top - bottom);
        let max = 100.0;
        let x = |i: usize| left + i as f32 / (values.len() - 1) as f32 * w;
        let y = |v: f32| top + h - v / max * h;
        let blue = rgb(0x3b82f6);
        let mut p = Painter::new()
            .font("11px system-ui")
            .fill_color(colors::muted())
            .stroke_color(rgb(0xe5e7eb))
            .line_width(1.0)
            // TS places each label at its measured width (`measureText`),
            // which `Painter` lacks; right-aligned text lands the same.
            .text_align(CanvasTextAlign::Right);
        for v in [0.0, 25.0, 50.0, 75.0, 100.0] {
            p = p
                .begin_path()
                .move_to(left, y(v))
                .line_to(width - right, y(v))
                .stroke()
                .fill_text(&format!("{v}"), left - 6.0, y(v) + 4.0);
        }
        p = p
            .text_align(CanvasTextAlign::Start)
            .begin_path()
            .move_to(x(0), y(values[0]));
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
    const NAME: &'static str = "gallery.canvas.chart";
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let height = 180.0;
        let drawing = if self.width > 0.0 {
            self.draw(self.width, height)
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
    const NAME: &'static str = "gallery.canvas.clock";
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

/// SVG path data: icons.
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
        let p = if name == "check" {
            // TS strokes the `Path2D` of the data; `Painter` only fills SVG
            // data, so the check is drawn with the same path's segments.
            p.stroke_color(color)
                .line_width(2.5)
                .line_cap(LineCap::Round)
                .line_join(LineJoin::Round)
                .begin_path()
                .move_to(20.0, 6.0)
                .line_to(9.0, 17.0)
                .line_to(4.0, 12.0)
                .stroke()
        } else {
            p.fill_color(color).fill_svg(d)
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
            .draw_image(LOGO, 12.0, 12.0, 96.0, 96.0)
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
            // Dashes and a gradient: TS fills a radial gradient, which
            // `Painter` lacks; a linear one has the same colors.
            .fill_style(
                CanvasGradient::linear(width - 50.0, 20.0, width - 50.0, 100.0)
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
    const NAME: &'static str = "gallery.canvas.compositing";
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

/// Touch drawing: each move draws again.
#[derive(Debug, Default)]
struct Sketch {
    strokes: Vec<Vec<(f32, f32)>>,
}

impl Component for Sketch {
    const NAME: &'static str = "gallery.canvas.sketch";
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
        // TS takes the touches as the responder and refuses to give them
        // up (`onResponderTerminationRequest`), so the page does not
        // scroll while drawing; a Rust component cannot claim them: a
        // vertical drag may scroll the page instead.
        let sketch = canvas(p)
            .h(px(180.0))
            .bg(rgb(0xf8fafc))
            .rounded(px(10.0))
            .on_touch(cx.listener(|this: &mut Self, touch: &TouchEvent, cx| {
                let Some(point) = touch.points.iter().find(|p| p.changed) else {
                    return;
                };
                let at = (point.location_x, point.location_y);
                match touch.phase {
                    TouchPhase::Start => this.strokes.push(vec![at]),
                    TouchPhase::Move => match this.strokes.last_mut() {
                        Some(stroke) => stroke.push(at),
                        None => this.strokes.push(vec![at]),
                    },
                    TouchPhase::End | TouchPhase::Cancel => return,
                }
                cx.notify();
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
    const NAME: &'static str = "gallery.canvas";
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
