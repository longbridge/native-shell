//! The stock's chart (as the TSX gallery's `demos/stocks/Chart.tsx`): the
//! intraday time-share line with its average price, or candles with MA5 /
//! 10 / 20, both over volume. Drag sideways for the crosshair; its lines
//! move natively and the readout floats beside it.

use std::f32::consts::TAU;

use nsh_core::{
    PanActivation, Rect,
    canvas::{CanvasGradient, CanvasTextAlign, CanvasTextBaseline, LineJoin},
    ui::prelude::*,
};

use super::{
    market::{Candle, OPEN, Quote, SESSION, compact, price, signed, time, tone_of},
    parts::{Calm, GRID, dashed, hatch, pulse, run_pulse},
};
use crate::ui::{
    Segment, Segmented, SegmentedEvent, SegmentedProps, SegmentedSize, colors, legend, radius,
};

/// What the chart shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Period {
    #[default]
    Intraday,
    M1,
    M5,
    M15,
    D1,
}

const PERIODS: [(Period, &str, &str); 5] = [
    (Period::Intraday, "Intraday", "intraday"),
    (Period::M1, "1m", "1m"),
    (Period::M5, "5m", "5m"),
    (Period::M15, "15m", "15m"),
    (Period::D1, "1D", "1D"),
];

impl Period {
    /// Candle bodies, in points.
    fn body(self) -> f32 {
        match self {
            Period::Intraday => 1.0,
            Period::M1 | Period::D1 => 5.0,
            Period::M5 => 7.0,
            Period::M15 => 9.0,
        }
    }

    /// The step between candles.
    fn step(self) -> f32 {
        self.body() + 2.0
    }
}

/// The moving averages: their spans and colors.
const AVERAGES: [(usize, AdaptiveColor); 3] =
    [(5, colors::ACCENT), (10, colors::INFO), (20, colors::WARN)];

const PAD_RIGHT: f32 = 54.0;
const PAD_TOP: f32 = 8.0;
const PAD_BOTTOM: f32 = 18.0;
const HEIGHT: f32 = 216.0;
const VOL_H: f32 = 40.0;

/// The candles of a period: minutes merged into 5 / 15, or the days with
/// today last.
fn candles(props: &ChartProps, period: Period) -> Vec<Candle> {
    let span = match period {
        Period::Intraday | Period::M1 => return props.minutes.clone(),
        Period::D1 => {
            let q = props.quote;
            let today = Candle {
                t: 0.0,
                o: q.open,
                h: q.high,
                l: q.low,
                c: q.last,
                v: q.volume,
                a: q.turnover / q.volume,
            };
            return props.days.iter().copied().chain([today]).collect();
        }
        Period::M5 => 300.0,
        Period::M15 => 900.0,
    };
    let mut out: Vec<Candle> = Vec::new();
    for m in &props.minutes {
        let t = m.t - (m.t - OPEN) % span;
        match out.last_mut() {
            Some(last) if last.t == t => {
                last.h = last.h.max(m.h);
                last.l = last.l.min(m.l);
                last.c = m.c;
                last.v += m.v;
                last.a = m.a;
            }
            _ => out.push(Candle { t, ..*m }),
        }
    }
    out
}

/// The mean close of the `n` candles ending at `i` (none before there are
/// `n`).
fn average(data: &[Candle], i: usize, n: usize) -> Option<f64> {
    if i + 1 < n || i >= data.len() {
        return None;
    }
    Some(data[i + 1 - n..=i].iter().map(|c| c.c).sum::<f64>() / n as f64)
}

fn label(c: &Candle, period: Period) -> String {
    if period == Period::D1 {
        if c.t == 0.0 {
            "Today".into()
        } else {
            format!("{}d ago", c.t)
        }
    } else {
        time(c.t, false)
    }
}

/// The geometry of one drawing, shared with the crosshair.
#[derive(Debug, Clone, Copy, Default)]
struct Frame {
    first: usize,
    count: usize,
    lo: f64,
    hi: f64,
    main_h: f32,
    vol_top: f32,
    plot_w: f32,
}

impl Frame {
    fn y(&self, v: f64) -> f32 {
        PAD_TOP + ((self.hi - v) / (self.hi - self.lo)) as f32 * self.main_h
    }
}

/// The chart.
#[derive(Debug, Default)]
pub(crate) struct Chart {
    period: Period,
    cursor: Option<usize>,
    width: f32,
    /// The live point's ring, pulsing while the market runs.
    live: Option<(NativeValue, Animation)>,
    /// Candles scroll: their range eases in rather than jumping.
    scale: Calm,
    data: Vec<Candle>,
    frame: Option<Frame>,
    /// The crosshair: where it is, and whether it shows.
    cross: Option<[NativeValue; 3]>,
}

/// The props of [`Chart`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct ChartProps {
    pub(crate) code: SharedString,
    pub(crate) quote: Quote,
    /// Today's minutes.
    pub(crate) minutes: Vec<Candle>,
    /// The days before today.
    pub(crate) days: Vec<Candle>,
    pub(crate) running: bool,
    pub(crate) decimals: u32,
}

impl Chart {
    fn intraday(&self) -> bool {
        self.period == Period::Intraday
    }

    /// Minutes the intraday axis spans: the session so far (an hour at
    /// least) and a narrow margin ahead, never past the close.
    fn span(&self) -> f32 {
        SESSION.min((self.data.len().max(60) as f64 * 1.06).ceil()) as f32
    }

    fn plot_w(&self) -> f32 {
        self.width - PAD_RIGHT
    }

    /// The candles shown: today's minutes intraday, else the newest that
    /// fit.
    fn view(&self) -> (usize, usize) {
        if self.intraday() {
            return (0, self.span() as usize);
        }
        let count = ((self.plot_w() / self.period.step()).floor() as usize).max(1);
        (self.data.len().saturating_sub(count), count)
    }

    fn x(&self, i: usize) -> f32 {
        if self.intraday() {
            return i as f32 / self.span() * self.plot_w();
        }
        let step = self.period.step();
        (i as f32 - self.view().0 as f32) * step + step / 2.0
    }

    /// The data and its frame, after the props, the period or the width
    /// changed. Redrawn on every tick: about a second to settle after the
    /// data narrows.
    fn refit(&mut self, props: &ChartProps) {
        self.data = candles(props, self.period);
        if let Some(i) = self.cursor {
            self.cursor = Some(i.min(self.data.len().saturating_sub(1)));
        }
        self.frame = None;
        let (first, count) = self.view();
        let end = self.data.len().min(first + count);
        if end <= first || self.width <= 0.0 {
            return;
        }
        let prev_close = props.quote.prev_close;
        let (mut lo, mut hi) = if self.intraday() {
            (prev_close, prev_close)
        } else {
            (f64::INFINITY, f64::NEG_INFINITY)
        };
        for c in &self.data[first..end] {
            lo = lo.min(c.l);
            hi = hi.max(c.h);
        }
        let margin = match (hi - lo) * 0.08 {
            0.0 => hi * 0.01,
            m => m,
        };
        // Intraday the range is the day's, which only widens; candles
        // scroll, so their range eases in rather than jumping.
        let (lo, hi) = if self.intraday() {
            (lo - margin, hi + margin)
        } else {
            self.scale.fit(lo - margin, hi + margin)
        };
        let main_h = HEIGHT - PAD_TOP - PAD_BOTTOM - VOL_H - 12.0;
        self.frame = Some(Frame {
            first,
            count,
            lo,
            hi,
            main_h,
            vol_top: PAD_TOP + main_h + 12.0,
            plot_w: self.plot_w(),
        });
    }

    /// The crosshair at `x` along the chart, or gone.
    fn pick(&mut self, x: Option<f32>, cx: &mut Context<'_, Self>) {
        let [cross_x, cross_y, shown] = self.cross.as_ref().expect("set up");
        let Some(x) = x else {
            self.cursor = None;
            let _ = shown.timing(0.0, 180.0, Easing::EaseInOut);
            cx.notify();
            return;
        };
        if self.data.is_empty() {
            return;
        }
        let (first, _) = self.view();
        let raw = if self.intraday() {
            (x / self.plot_w() * self.span()).round() as i64
        } else {
            first as i64 + (x / self.period.step()).floor() as i64
        };
        let i = raw.clamp(first as i64, self.data.len() as i64 - 1).max(0) as usize;
        if self.cursor.is_none() {
            let _ = shown.timing(1.0, 140.0, Easing::EaseInOut);
        }
        self.cursor = Some(i);
        cross_x.set(self.x(i));
        if let Some(f) = self.frame {
            cross_y.set(f.y(self.data[i].c));
        }
        cx.notify();
    }

    fn draw(&self, props: &ChartProps, cx: &RenderCx<'_, Self>) -> Painter {
        let Some(f) = self.frame else {
            return Painter::new();
        };
        let c = |color: AdaptiveColor| cx.color(color);
        let p_of = |v: f64| price(v, props.decimals);
        let width = self.width;
        let prev_close = props.quote.prev_close;
        let intraday = self.intraday();

        // The grid and the price axis (percent from the close intraday).
        let mut p = Painter::new()
            .font("11px system-ui")
            .text_baseline(CanvasTextBaseline::Middle)
            .text_align(CanvasTextAlign::Left)
            .line_width(1.0);
        for k in 0..=4 {
            let v = f.hi - (f.hi - f.lo) * k as f64 / 4.0;
            let y = f.y(v);
            let label = if intraday {
                format!("{}%", signed((v - prev_close) / prev_close * 100.0, 2))
            } else {
                p_of(v)
            };
            p = p
                .stroke_color(c(GRID))
                .begin_path()
                .move_to(0.0, y)
                .line_to(f.plot_w, y)
                .stroke()
                .fill_color(c(colors::FAINT))
                .text_align(CanvasTextAlign::Right)
                .fill_text(&label, width, y);
        }
        p = p
            .stroke_color(c(GRID))
            .begin_path()
            .move_to(0.0, f.vol_top + VOL_H)
            .line_to(f.plot_w, f.vol_top + VOL_H)
            .stroke();

        p = if intraday {
            self.draw_intraday(p, props, &f, cx)
        } else {
            self.draw_candles(p, &f, cx)
        };

        // The last price on the axis.
        let d = &self.data;
        let last = d[d.len() - 1];
        let ly = f.y(last.c).clamp(PAD_TOP + 8.0, PAD_TOP + f.main_h - 8.0);
        let tone = tone_of(if intraday {
            last.c - prev_close
        } else {
            last.c - last.o
        });
        if !intraday {
            p = dashed(p, 0.0, f.plot_w, ly, c(tone.opacity(0.7)), &[3.0, 3.0]);
        }
        p.fill_color(c(tone))
            .begin_path()
            .round_rect(f.plot_w + 2.0, ly - 9.0, PAD_RIGHT - 2.0, 18.0, 5.0)
            .fill()
            .fill_color(c(colors::INSET))
            .text_align(CanvasTextAlign::Right)
            .text_baseline(CanvasTextBaseline::Middle)
            .font("600 11px system-ui")
            .fill_text(&p_of(last.c), width - 4.0, ly)
    }

    fn draw_intraday(
        &self,
        p: Painter,
        props: &ChartProps,
        f: &Frame,
        cx: &RenderCx<'_, Self>,
    ) -> Painter {
        let c = |color: AdaptiveColor| cx.color(color);
        let d = &self.data;
        let prev_close = props.quote.prev_close;
        let now = self.x(d.len() - 1);
        let last = d[d.len() - 1].c;
        let tone = tone_of(last - prev_close);
        let bottom = PAD_TOP + f.main_h;
        let mut p = hatch(
            p,
            now + 4.0,
            PAD_TOP,
            f.plot_w - now - 4.0,
            f.main_h,
            c(GRID),
            7.0,
        );
        p = dashed(
            p,
            0.0,
            f.plot_w,
            f.y(prev_close),
            c(colors::FAINT),
            &[3.0, 4.0],
        );
        // The price area and line.
        let fill = CanvasGradient::linear(0.0, PAD_TOP, 0.0, bottom)
            .add_color_stop(0.0, c(tone.opacity(0.26)))
            .add_color_stop(1.0, c(tone.opacity(0.0)));
        p = p.begin_path().move_to(self.x(0), bottom);
        for (i, candle) in d.iter().enumerate() {
            p = p.line_to(self.x(i), f.y(candle.c));
        }
        p = p
            .line_to(now, bottom)
            .close_path()
            .fill_style(fill)
            .fill()
            .line_join(LineJoin::Round);
        let line = |p: Painter, y: &dyn Fn(&Candle) -> f64| {
            let mut p = p.begin_path().move_to(self.x(0), f.y(y(&d[0])));
            for (i, candle) in d.iter().enumerate().skip(1) {
                p = p.line_to(self.x(i), f.y(y(candle)));
            }
            p
        };
        p = line(p, &|c| c.c)
            .stroke_color(c(tone))
            .line_width(1.5)
            .stroke();
        // The day's average price.
        p = line(p, &|c| c.a)
            .stroke_color(c(colors::WARN))
            .line_width(1.1)
            .stroke();
        // Volume.
        let vw = (f.plot_w / self.span() - 0.8).max(1.0);
        let vmax = d.iter().fold(0.0_f64, |m, c| m.max(c.v));
        let vmax = if vmax == 0.0 { 1.0 } else { vmax };
        for (i, candle) in d.iter().enumerate() {
            let h = (candle.v / vmax) as f32 * VOL_H;
            let color = if candle.c >= candle.o {
                colors::UP
            } else {
                colors::DOWN
            };
            p = p.fill_color(c(color.opacity(0.6))).fill_rect(
                self.x(i) - vw / 2.0,
                f.vol_top + VOL_H - h,
                vw,
                h,
            );
        }
        // The live point, pulsing natively.
        let (live, _) = self.live.as_ref().expect("set up");
        let ring_alpha = live.interpolate(&[0.0, 1.0], &[0.45, 0.0]);
        let ring_radius = live.interpolate(&[0.0, 1.0], &[3.0, 13.0]);
        let ly = f.y(last);
        p = p
            .fill_color(c(tone))
            .global_alpha(&ring_alpha)
            .begin_path()
            .arc(now, ly, &ring_radius, 0.0, TAU)
            .fill()
            .global_alpha(1.0)
            .begin_path()
            .arc(now, ly, 3.0, 0.0, TAU)
            .fill();
        // The open, the middle and the end of the span.
        p = p
            .fill_color(c(colors::FAINT))
            .text_baseline(CanvasTextBaseline::Top);
        for (k, align) in [
            (0.0, CanvasTextAlign::Left),
            (0.5, CanvasTextAlign::Center),
            (1.0, CanvasTextAlign::Right),
        ] {
            let at = OPEN + (k * self.span() as f64).round() * 60.0;
            p = p.text_align(align).fill_text(
                &time(at, false),
                k as f32 * f.plot_w,
                f.vol_top + VOL_H + 5.0,
            );
        }
        p
    }

    fn draw_candles(&self, p: Painter, f: &Frame, cx: &RenderCx<'_, Self>) -> Painter {
        let c = |color: AdaptiveColor| cx.color(color);
        let d = &self.data;
        let body = self.period.body();
        let end = d.len().min(f.first + f.count);
        let vmax = d[f.first..end].iter().fold(0.0_f64, |m, c| m.max(c.v));
        let vmax = if vmax == 0.0 { 1.0 } else { vmax };
        let mut p = p;
        for (i, candle) in d.iter().enumerate().take(end).skip(f.first) {
            let x = self.x(i);
            let up = candle.c >= candle.o;
            let color = if up { colors::UP } else { colors::DOWN };
            let top = f.y(candle.o.max(candle.c));
            let h = (f.y(candle.o.min(candle.c)) - top).max(1.0);
            p = p
                .stroke_color(c(color))
                .line_width(1.0)
                .begin_path()
                .move_to(x, f.y(candle.h))
                .line_to(x, f.y(candle.l))
                .stroke()
                .fill_color(c(if up { colors::INSET } else { color }))
                .fill_rect(x - body / 2.0, top, body, h);
            if up {
                p = p.stroke_rect(
                    x - body / 2.0 + 0.5,
                    top + 0.5,
                    body - 1.0,
                    (h - 1.0).max(0.0),
                );
            }
            let vh = (candle.v / vmax) as f32 * VOL_H;
            p = p.fill_color(c(color.opacity(0.5))).fill_rect(
                x - body / 2.0,
                f.vol_top + VOL_H - vh,
                body,
                vh,
            );
        }
        p = p.line_width(1.2).line_join(LineJoin::Round);
        for (n, color) in AVERAGES {
            p = p.stroke_color(c(color)).begin_path();
            let mut started = false;
            for i in f.first..end {
                let Some(v) = average(d, i, n) else { continue };
                p = if started {
                    p.line_to(self.x(i), f.y(v))
                } else {
                    p.move_to(self.x(i), f.y(v))
                };
                started = true;
            }
            p = p.stroke();
        }
        p = p
            .fill_color(c(colors::FAINT))
            .text_baseline(CanvasTextBaseline::Top);
        let n = end - f.first;
        for (k, align) in [
            (0, CanvasTextAlign::Left),
            (n / 2, CanvasTextAlign::Center),
            (n - 1, CanvasTextAlign::Right),
        ] {
            let i = f.first + k;
            p = p.text_align(align).fill_text(
                &label(&d[i], self.period),
                self.x(i).clamp(0.0, f.plot_w),
                f.vol_top + VOL_H + 5.0,
            );
        }
        p
    }

    /// The readout's left edge: beside the cursor, flipped near the right.
    fn readout_left(&self, i: usize) -> f32 {
        let x = self.x(i);
        if x > self.width / 2.0 {
            (x - 150.0).max(0.0)
        } else {
            (self.width - 140.0).min(x + 10.0)
        }
    }
}

impl Component for Chart {
    type Props = ChartProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &ChartProps, cx: &mut Context<'_, Self>) {
        let live = pulse(cx);
        run_pulse(&live, props.running);
        self.live = Some(live);
        self.scale = Calm::new(0.06);
        self.cross = Some([
            cx.native_value(0.0),
            cx.native_value(0.0),
            cx.native_value(0.0),
        ]);
        self.refit(props);
    }

    fn props_changed(&mut self, old: &ChartProps, new: &ChartProps, _: &mut Context<'_, Self>) {
        if old.running != new.running {
            run_pulse(self.live.as_ref().expect("set up"), new.running);
        }
        self.refit(new);
    }

    fn render(&self, props: &ChartProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let p_of = |v: f64| price(v, props.decimals);
        let prev_close = props.quote.prev_close;
        let d = &self.data;
        let focus = self
            .cursor
            .or(d.len().checked_sub(1))
            .and_then(|i| d.get(i))
            .copied()
            .unwrap_or_default();
        let legends = if self.intraday() {
            vec![
                legend(tone_of(focus.c - prev_close), "Price", Some(p_of(focus.c))),
                legend(colors::WARN, "Avg", Some(p_of(focus.a))),
                legend(colors::FAINT, "Vol", Some(compact(focus.v))),
            ]
        } else {
            let at = self.cursor.unwrap_or(d.len().saturating_sub(1));
            AVERAGES
                .iter()
                .map(|&(n, color)| {
                    let value = average(d, at, n).map_or("–".into(), p_of);
                    legend(color, format!("MA{n}"), Some(value))
                })
                .collect()
        };
        let [cross_x, cross_y, shown] = self.cross.as_ref().expect("set up");
        let change = (focus.c - prev_close) / prev_close * 100.0;
        let readout = self.cursor.map(|i| {
            view()
                .absolute()
                .top(px(PAD_TOP + 2.0))
                .left(px(self.readout_left(i)))
                .w(px(140.0))
                .p(px(10.0))
                .gap(px(3.0))
                .rounded(px(radius::CONTROL))
                .border_curve(BorderCurve::Continuous)
                .bg(colors::SURFACE.opacity(0.94))
                .border_1()
                .border_color(colors::BORDER)
                .child(
                    text(label(&focus, self.period))
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                        .tabular_nums(),
                )
                .child(
                    text(p_of(focus.c))
                        .text_size(px(17.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT)
                        .tabular_nums()
                        .child(
                            text(format!(" {}%", signed(change, 2)))
                                .text_size(px(12.0))
                                .font_weight(FontWeight::NORMAL)
                                .tabular_nums()
                                .text_color(tone_of(focus.c - prev_close)),
                        ),
                )
                .child(if self.intraday() {
                    text(format!("avg {} · vol {}", p_of(focus.a), compact(focus.v)))
                        .text_size(px(11.0))
                        .text_color(colors::MUTED)
                        .tabular_nums()
                } else {
                    text(format!(
                        "O {} H {}\nL {} V {}",
                        p_of(focus.o),
                        p_of(focus.h),
                        p_of(focus.l),
                        compact(focus.v)
                    ))
                    .text_size(px(11.0))
                    .line_height(px(15.0))
                    .text_color(colors::MUTED)
                    .tabular_nums()
                })
        });
        let period_name = PERIODS
            .iter()
            .find(|p| p.0 == self.period)
            .map_or("", |p| p.2);
        let selected = PERIODS.iter().position(|p| p.0 == self.period).unwrap_or(0);
        view()
            .gap(px(10.0))
            .child(
                Segmented::element(SegmentedProps {
                    options: PERIODS
                        .iter()
                        .map(|p| Segment::new(p.1))
                        .collect::<Vec<_>>()
                        .into(),
                    selected: (selected as u32).into(),
                    size: SegmentedSize::Small,
                })
                .on_event(cx.listener(
                    |this: &mut Self, e: &SegmentedEvent, cx| {
                        let SegmentedEvent::Change(i) = e;
                        this.period = PERIODS[*i as usize].0;
                        this.scale.reset();
                        this.refit(cx.props());
                        cx.notify();
                    },
                )),
            )
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_x(px(14.0))
                    .gap_y(px(6.0))
                    .children(legends),
            )
            .child(
                view()
                    .h(px(HEIGHT))
                    .child(
                        canvas(self.draw(props, cx))
                            .h(px(HEIGHT))
                            .label(format!("{} {period_name} chart", props.code))
                            .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                                if this.width != frame.width {
                                    this.width = frame.width;
                                    this.refit(cx.props());
                                    cx.notify();
                                }
                            }))
                            // The platform gives the chart the finger only once
                            // it has moved sideways, so vertical drags still
                            // scroll the page.
                            .claim_touches(TouchClaim::on_pan(PanActivation::horizontal(6.0, 8.0)))
                            .keep_touches(true)
                            .on_responder_grant(cx.listener(
                                |this: &mut Self, e: &ResponderEvent, cx| {
                                    this.pick(Some(e.location_x), cx);
                                },
                            ))
                            .on_responder_move(cx.listener(
                                |this: &mut Self, e: &ResponderEvent, cx| {
                                    this.pick(Some(e.location_x), cx);
                                },
                            ))
                            .on_responder_release(cx.listener(
                                |this: &mut Self, _: &ResponderEvent, cx| {
                                    this.pick(None, cx);
                                },
                            ))
                            .on_responder_terminate(cx.listener(
                                |this: &mut Self, _: &ResponderEvent, cx| {
                                    this.pick(None, cx);
                                },
                            )),
                    )
                    // The crosshair over the canvas, moved by native values.
                    .child(
                        view()
                            .pointer_events(PointerEvents::None)
                            .absolute()
                            .inset_0()
                            .bind_opacity(shown)
                            .child(
                                view()
                                    .absolute()
                                    .top(px(PAD_TOP))
                                    .bottom(px(PAD_BOTTOM))
                                    .left_0()
                                    .w(px(1.0))
                                    .bg(colors::MUTED)
                                    .bind_translate_x(cross_x),
                            )
                            .child(
                                view()
                                    .absolute()
                                    .left_0()
                                    .right(px(PAD_RIGHT))
                                    .top_0()
                                    .h(px(1.0))
                                    .bg(colors::MUTED.opacity(0.6))
                                    .bind_translate_y(cross_y),
                            )
                            .child(
                                view()
                                    .absolute()
                                    .top(px(-4.0))
                                    .left(px(-4.0))
                                    .size(px(9.0))
                                    .rounded(px(4.5))
                                    .bg(colors::TEXT)
                                    .border_2()
                                    .border_color(colors::INSET)
                                    .bind_translate_x(cross_x)
                                    .bind_translate_y(cross_y),
                            )
                            .children(readout),
                    ),
            )
    }
}
