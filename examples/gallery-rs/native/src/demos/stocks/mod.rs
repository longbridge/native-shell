//! Stocks: a live watchlist and a stock (as the TSX gallery's
//! `demos/Stocks.tsx` and `demos/stocks/`): quotes that change every 50 ms
//! under a one-line market strip, sorted filters that re-sort once a second
//! with a layout animation; a stock opens its candles, depth, the tape and
//! a ticket. Leaving the demo stops the market.
//!
//! The watchlist owns the market and steps it; each row, the strip and the
//! filters are components of their own, given what they show as props, so a
//! tick renders again only what it changed. A stock's screen is pushed into
//! the gallery's stack, outside the watchlist: it reads the market through
//! [`Feed`] when [`FocusTick`] says it moved, and the market buttons pause
//! and resume it through the watchlist's handle.

mod book;
mod chart;
mod detail;
mod market;
mod parts;
mod tape;
mod ticket;

use std::{
    cell::RefCell,
    f32::consts::TAU,
    rc::Rc,
    time::{Duration, Instant},
};

use nsh_core::{
    LayoutAnimation, LayoutCurve, LayoutProperty, LayoutTransition,
    canvas::{CanvasGradient, Composite, LineCap, LineJoin},
    nsh_animation::{Curve, EaseMode},
    ui::{AnyElement, prelude::*},
};

use self::{
    detail::{Detail, DetailProps},
    market::{
        Breadth, INDEX_BASE, ISSUES, Market, Quote, SECTORS, TICK_MS, grouped, price, signed, time,
        tone_of,
    },
    parts::{MarketButton, SpringBar, SpringBarProps, spring_bar},
};
use crate::{
    nav,
    ui::{
        IconName, Segment, Segmented, SegmentedEvent, SegmentedProps, SegmentedSize, colors,
        header, icon, motion,
    },
};

/// The market the watchlist runs, and the watchlist itself, for the
/// screens it pushes.
#[derive(Default)]
pub(crate) struct Feed {
    market: Option<Rc<RefCell<Market>>>,
    owner: Option<WeakComponent<Stocks>>,
}

/// Whether the market runs.
#[derive(Default)]
pub(crate) struct Running(pub(crate) bool);

/// Counts the ticks while a stock is focused: its screen renders again
/// with each.
#[derive(Default)]
pub(crate) struct FocusTick(u64);

/// Expanding and collapsing, reordering rows: one native layout animation.
const RELAYOUT: LayoutAnimation = LayoutAnimation {
    create: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 380.0)),
    update: Some(LayoutTransition {
        curve: LayoutCurve::Spring { damping: 0.84 },
        property: LayoutProperty::Opacity,
        duration_ms: 380.0,
        delay_ms: 0.0,
    }),
    delete: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 380.0)),
};

/// The watchlist's filters.
const FILTERS: [&str; 4] = ["All", "Gainers", "Losers", "Active"];

/// The watchlist: it owns the market.
pub(crate) struct Stocks {
    market: Rc<RefCell<Market>>,
    timer: Option<Task<()>>,
    filter: usize,
    /// The stocks shown, in order.
    rows: Vec<usize>,
    /// Gainers and losers, once a second.
    counts: (u32, u32),
}

impl Default for Stocks {
    fn default() -> Self {
        Self {
            market: Rc::new(RefCell::new(Market::new())),
            timer: None,
            filter: 0,
            rows: Vec::new(),
            counts: (0, 0),
        }
    }
}

impl Stocks {
    /// Starts or stops the market (the market buttons).
    pub(crate) fn toggle(&mut self, cx: &mut Context<'_, Self>) {
        if self.timer.is_some() {
            self.timer = None;
            cx.update_global::<Running, _>(|r, _| r.0 = false);
        } else {
            self.start(cx);
        }
    }

    fn start(&mut self, cx: &mut Context<'_, Self>) {
        self.timer = Some(
            cx.set_interval(Duration::from_millis(TICK_MS), |this: &mut Self, cx| {
                this.tick(cx)
            }),
        );
        cx.update_global::<Running, _>(|r, _| r.0 = true);
    }

    fn tick(&mut self, cx: &mut Context<'_, Self>) {
        let (second, focused) = {
            let mut market = self.market.borrow_mut();
            market.step();
            (market.on_second(), market.focused.is_some())
        };
        if second {
            self.count();
            self.order(cx);
        }
        if focused {
            cx.update_global::<FocusTick, _>(|t, _| t.0 += 1);
        }
        cx.notify();
    }

    fn count(&mut self) {
        let market = self.market.borrow();
        let changes = market.stocks.iter().map(|s| s.quote.percent());
        self.counts = changes.fold((0, 0), |(up, down), c| {
            (up + u32::from(c > 0.0), down + u32::from(c < 0.0))
        });
    }

    /// Orders the rows by the filter; a change animates.
    fn order(&mut self, cx: &mut Context<'_, Self>) {
        let market = self.market.borrow();
        let change = |i: &usize| market.stocks[*i].quote.percent();
        let all = 0..market.stocks.len();
        let mut next: Vec<usize> = match self.filter {
            0 => all.collect(),
            1 => all.filter(|i| change(i) > 0.0).collect(),
            2 => all.filter(|i| change(i) < 0.0).collect(),
            _ => all.collect(),
        };
        match self.filter {
            1 => next.sort_by(|a, b| change(b).total_cmp(&change(a))),
            2 => next.sort_by(|a, b| change(a).total_cmp(&change(b))),
            3 => {
                let turnover = |i: &usize| market.stocks[*i].quote.turnover;
                next.sort_by(|a, b| turnover(b).total_cmp(&turnover(a)));
                next.truncate(10);
            }
            _ => {}
        }
        drop(market);
        if next != self.rows {
            cx.animate_layout(RELAYOUT);
            self.rows = next;
        }
    }
}

impl Component for Stocks {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        let (market, owner) = (self.market.clone(), cx.weak());
        cx.update_global::<Feed, _>(|feed, _| {
            feed.market = Some(market);
            feed.owner = Some(owner);
        });
        self.count();
        self.rows = (0..self.market.borrow().stocks.len()).collect();
        self.start(cx);
    }

    fn cleanup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.timer = None;
        cx.update_global::<Feed, _>(|feed, _| *feed = Feed::default());
        cx.update_global::<Running, _>(|r, _| r.0 = false);
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let market = self.market.borrow();
        let options = FILTERS.iter().enumerate().map(|(i, label)| {
            let segment = Segment::new(*label);
            match i {
                0 => segment.count(market.stocks.len() as u32),
                1 => segment.count(self.counts.0),
                2 => segment.count(self.counts.1),
                _ => segment,
            }
        });
        let open = cx.listener(|this: &mut Self, index: &usize, cx| {
            let index = *index;
            let code = this.market.borrow().stocks[index].code;
            nav::push(cx, format!("stock:{code}"), move |cx| {
                header(screen().title(code), cx)
                    .header_right(MarketButton::element(()))
                    .child(Detail::element(DetailProps {
                        stock: index as u32,
                    }))
            });
        });
        let rows: Vec<AnyElement> = self
            .rows
            .iter()
            .enumerate()
            .map(|(at, &i)| {
                let stock = &market.stocks[i];
                StockRow::element(StockRowProps {
                    code: stock.code.into(),
                    name: stock.name.into(),
                    quote: stock.quote,
                    spark: stock.spark.clone(),
                    decimals: stock.decimals(),
                    first: at == 0,
                })
                .key(stock.code)
                .on_event(open.clone().map(move |_: &StockRowEvent| i))
                .into_element()
            })
            .collect();
        let empty = self.rows.is_empty();
        let stats = market.stats;
        view()
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(MarketStrip::element(MarketStripProps {
                composite: market.composite,
                clock: market.clock,
                breadth: market.breadth,
                sectors: market.sectors.to_vec(),
            }))
            .child(
                view().px_4().pb(px(6.0)).child(
                    Segmented::element(SegmentedProps {
                        options: options.collect(),
                        selected: self.filter as u32,
                        size: SegmentedSize::Small,
                    })
                    .on_event(cx.listener(|this: &mut Self, e: &SegmentedEvent, cx| {
                        let SegmentedEvent::Change(i) = e;
                        this.filter = *i as usize;
                        this.order(cx);
                        cx.notify();
                    })),
                ),
            )
            .child(
                scroll()
                    .flex_1()
                    .child(view().children(rows).when(empty, |v| {
                        v.child(
                            text("Nothing here right now")
                                .py(px(32.0))
                                .text_center()
                                .text_size(px(14.0))
                                .text_color(colors::MUTED),
                        )
                    }))
                    .child(
                        text(format!(
                            "Simulated, fictional companies · tick {TICK_MS} ms · {:.2} ms Rust/tick · {} quotes/s",
                            stats.ms, stats.updates
                        ))
                        .px_4()
                        .pt_4()
                        .pb(px(40.0))
                        .text_size(px(12.0))
                        .line_height(px(17.0))
                        .text_color(colors::FAINT)
                        .tabular_nums(),
                    ),
            )
    }
}

/// The market in one line, on the 16 pt gutters: the composite and its
/// change, the clock, pause. A tap opens breadth and sectors (a layout
/// animation; the chevron turns natively).
#[derive(Debug, Default)]
struct MarketStrip {
    opened: bool,
    turn: Option<NativeValue>,
}

#[derive(Debug, Clone, PartialEq, Props)]
struct MarketStripProps {
    composite: f64,
    clock: f64,
    breadth: Breadth,
    /// Each sector's mean change, as [`SECTORS`].
    sectors: Vec<f64>,
}

impl Component for MarketStrip {
    type Props = MarketStripProps;
    type Event = NoEvent;

    fn setup(&mut self, _: &MarketStripProps, cx: &mut Context<'_, Self>) {
        self.turn = Some(cx.native_value(0.0));
    }

    fn render(&self, props: &MarketStripProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let turn = self.turn.as_ref().expect("set up");
        let change = (props.composite - INDEX_BASE) / INDEX_BASE * 100.0;
        let toggle = cx.listener(|this: &mut Self, _, cx| {
            cx.animate_layout(RELAYOUT);
            this.opened = !this.opened;
            let _ = this
                .turn
                .as_ref()
                .expect("set up")
                .spring(if this.opened { 1.0 } else { 0.0 }, motion::SETTLE);
            cx.notify();
        });
        view()
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .px_4()
                    .h(px(56.0))
                    .child(
                        view()
                            .on_press(toggle)
                            .role(AccessibilityRole::Button)
                            .label(if self.opened {
                                "Hide breadth"
                            } else {
                                "Show breadth"
                            })
                            .flex_1()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .h(px(56.0))
                            .child(
                                text("Composite")
                                    .number_of_lines(1)
                                    .text_size(px(13.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(
                                text(grouped(props.composite, 2))
                                    .number_of_lines(1)
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::TEXT)
                                    .tabular_nums(),
                            )
                            .child(
                                text(format!("{}%", signed(change, 2)))
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(tone_of(change))
                                    .tabular_nums(),
                            )
                            .child(
                                view()
                                    .bind_rotate(&turn.interpolate(&[0.0, 1.0], &[90.0, -90.0]))
                                    .child(icon(IconName::Chevron, 12.0, colors::FAINT)),
                            ),
                    )
                    .child(
                        text(time(props.clock, true))
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .tabular_nums(),
                    )
                    .child(MarketButton::element(())),
            )
            .when(self.opened, |v| v.child(breadth(props)))
    }
}

/// Advancers vs decliners across the composite and the sectors' moves:
/// what the strip opens to.
fn breadth(props: &MarketStripProps) -> View {
    let b = props.breadth;
    let scale = props.sectors.iter().fold(0.5_f64, |m, c| m.max(c.abs())) * 1.1;
    let figure = |s: String, color: AdaptiveColor| {
        text(s).text_size(px(12.0)).tabular_nums().text_color(color)
    };
    view()
        .px_4()
        .pb(px(14.0))
        .gap(px(10.0))
        .child(
            view()
                .gap(px(6.0))
                .child(
                    view()
                        .flex_row()
                        .h(px(6.0))
                        .rounded(px(3.0))
                        .overflow_hidden()
                        .gap(px(2.0))
                        .child(view().flex_1().flex_grow(b.up as f32).bg(colors::UP))
                        .child(view().flex_1().flex_grow(b.flat as f32).bg(colors::FAINT))
                        .child(view().flex_1().flex_grow(b.down as f32).bg(colors::DOWN)),
                )
                .child(
                    view()
                        .flex_row()
                        .child(figure(format!("{} up", b.up), colors::UP).flex_1())
                        .child(figure(
                            format!("{} flat · {ISSUES} issues", b.flat),
                            colors::FAINT,
                        ))
                        .child(
                            figure(format!("{} down", b.down), colors::DOWN)
                                .flex_1()
                                .text_right(),
                        ),
                ),
        )
        .child(
            view().gap(px(2.0)).children(
                SECTORS
                    .iter()
                    .zip(&props.sectors)
                    .map(|((_, label), &change)| sector_bar(label, change, scale)),
            ),
        )
}

/// A sector's mean change as a bar diverging from the middle.
fn sector_bar(label: &'static str, change: f64, scale: f64) -> View {
    let share = (change.abs() / scale).min(1.0);
    view()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .h(px(22.0))
        .child(
            text(label)
                .number_of_lines(1)
                .w(px(84.0))
                .text_size(px(13.0))
                .text_color(colors::MUTED),
        )
        .child(
            view()
                .flex_1()
                .flex_row()
                .items_center()
                .child(view().flex_1().child(SpringBar::element(SpringBarProps {
                    from_right: true,
                    radius: Some(2.0),
                    ..spring_bar(if change < 0.0 { share } else { 0.0 }, colors::DOWN)
                })))
                .child(view().w(px(1.0)).h(px(14.0)).bg(colors::BORDER))
                .child(view().flex_1().child(SpringBar::element(SpringBarProps {
                    radius: Some(2.0),
                    ..spring_bar(if change > 0.0 { share } else { 0.0 }, colors::UP)
                }))),
        )
        .child(
            text(format!("{}%", signed(change, 2)))
                .w(px(56.0))
                .text_right()
                .text_size(px(12.0))
                .tabular_nums()
                .text_color(tone_of(change)),
        )
}

/// The newest point's ring at its widest: the canvas reaches this far past
/// the plot on every side, so the ring is never clipped.
const RING: f32 = 10.0;
/// The sparkline's plot.
const SPARK_W: f32 = 104.0;
const SPARK_H: f32 = 38.0;
/// The shortest time between two pulses of a row: each one completes.
const PULSE_GAP: Duration = Duration::from_millis(600);

/// A watchlist row: code and name, today's sparkline, the price and its
/// change, plain text on a flat row.
#[derive(Debug, Default)]
struct StockRow {
    /// The sparkline's y range, easing instead of jumping.
    scale: parts::Calm,
    range: (f64, f64),
    /// The newest point's pulse: 0 → 1 on a change.
    ring: Option<NativeValue>,
    seen: i64,
    fired: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq, Props)]
struct StockRowProps {
    code: SharedString,
    name: SharedString,
    quote: Quote,
    /// Today's sparkline.
    spark: Vec<f64>,
    decimals: u32,
    first: bool,
}

#[derive(Debug, Clone, PartialEq, Event)]
enum StockRowEvent {
    /// The row was pressed.
    Press,
}

impl StockRow {
    fn fit(&mut self, props: &StockRowProps) {
        let q = props.quote;
        let (lo, hi) = props
            .spark
            .iter()
            .fold((q.prev_close, q.prev_close), |(lo, hi), &v| {
                (lo.min(v), hi.max(v))
            });
        self.range = self.scale.fit(lo, hi);
    }

    /// Today in 104 × 38 pt: a smooth line through a moving average of the
    /// minutes' closes ending at the live price, a soft fill under it that
    /// fades in from the left and down from the line, the previous close
    /// dotted and a dot at the newest point that rings natively on each
    /// change.
    fn sparkline(&self, props: &StockRowProps, cx: &RenderCx<'_, Self>) -> Painter {
        let ring = self.ring.as_ref().expect("set up");
        let ring_radius = ring.interpolate(&[0.0, 1.0], &[3.0, RING]);
        let halo_alpha = ring.interpolate(&[0.0, 1.0], &[0.35, 0.0]);
        let stroke_alpha = ring.interpolate(&[0.0, 1.0], &[0.9, 0.0]);
        let ys = &props.spark;
        let n = ys.len();
        let q = props.quote;
        let (width, height) = (SPARK_W + RING * 2.0, SPARK_H + RING * 2.0);
        let (lo, hi) = self.range;
        let (left, right, bottom) = (RING, width - RING, height - RING);
        let span = if hi - lo == 0.0 { 1.0 } else { hi - lo };
        let y_of = |v: f64| RING + 2.0 + ((hi - v) / span) as f32 * (bottom - RING - 4.0);
        let x_of = |k: usize| left + k as f32 / (n.max(2) - 1) as f32 * (right - left);
        let tone = tone_of(q.last - q.prev_close);
        let tone_color = cx.color(tone);
        // A smooth curve: quadratic segments through the midpoints.
        let curve = |p: Painter| {
            let mut p = p.move_to(x_of(0), y_of(ys[0]));
            for k in 1..n.saturating_sub(1) {
                p = p.quadratic_curve_to(
                    (x_of(k), y_of(ys[k])),
                    (
                        (x_of(k) + x_of(k + 1)) / 2.0,
                        (y_of(ys[k]) + y_of(ys[k + 1])) / 2.0,
                    ),
                );
            }
            p.line_to(x_of(n - 1), y_of(ys[n - 1]))
        };
        if n == 0 {
            return Painter::new();
        }

        // The fill: down from the line's top, strictly under the curve...
        let top = ys.iter().fold(bottom, |t, &v| t.min(y_of(v)));
        let fill = CanvasGradient::linear(0.0, top, 0.0, bottom)
            .add_color_stop(0.0, cx.color(tone.opacity(0.22)))
            .add_color_stop(1.0, cx.color(tone.opacity(0.0)));
        let p = curve(Painter::new().begin_path())
            .line_to(x_of(n - 1), bottom)
            .line_to(x_of(0), bottom)
            .close_path()
            .fill_style(fill)
            .fill();
        // ...and faded in over the first fifth (erased, not overdrawn).
        let fade_to = left + (right - left) * 0.2;
        let fade = CanvasGradient::linear(left, 0.0, fade_to, 0.0)
            .add_color_stop(0.0, rgb(0x000000))
            .add_color_stop(1.0, Color(0));
        let p = p
            .composite(Composite::DestinationOut)
            .fill_style(fade)
            .fill_rect(0.0, 0.0, fade_to, height)
            .composite(Composite::SourceOver);

        // The previous close, dotted.
        let p = p
            .stroke_color(cx.color(colors::MUTED.opacity(0.4)))
            .line_width(1.0)
            .line_dash(&[1.0, 3.0])
            .begin_path()
            .move_to(left, y_of(q.prev_close))
            .line_to(right, y_of(q.prev_close))
            .stroke()
            .line_dash(&[]);

        let p = curve(p.begin_path())
            .stroke_color(tone_color)
            .line_width(1.75)
            .line_join(LineJoin::Round)
            .line_cap(LineCap::Round)
            .stroke();

        // The newest point and its pulse: a soft halo and a ring that grow
        // and fade natively (the canvas has room for them on every side).
        let (x, y) = (x_of(n - 1), y_of(ys[n - 1]));
        p.fill_color(tone_color)
            .stroke_color(tone_color)
            .global_alpha(&halo_alpha)
            .begin_path()
            .arc(x, y, &ring_radius, 0.0, TAU)
            .fill()
            .global_alpha(&stroke_alpha)
            .line_width(1.5)
            .begin_path()
            .arc(x, y, &ring_radius, 0.0, TAU)
            .stroke()
            .global_alpha(1.0)
            .begin_path()
            .arc(x, y, 3.0, 0.0, TAU)
            .fill()
    }
}

impl Component for StockRow {
    type Props = StockRowProps;
    type Event = StockRowEvent;

    fn setup(&mut self, props: &StockRowProps, cx: &mut Context<'_, Self>) {
        self.scale = parts::Calm::new(0.1);
        self.ring = Some(cx.native_value(1.0));
        self.seen = props.quote.changed_at;
        self.fit(props);
    }

    fn props_changed(
        &mut self,
        old: &StockRowProps,
        new: &StockRowProps,
        _: &mut Context<'_, Self>,
    ) {
        if old.spark != new.spark || old.quote != new.quote {
            self.fit(new);
        }
        if new.quote.changed_at != self.seen {
            self.seen = new.quote.changed_at;
            // A pulse at most every 600 ms, so each one plays out.
            if self.fired.is_none_or(|at| at.elapsed() >= PULSE_GAP) {
                self.fired = Some(Instant::now());
                let ring = self.ring.as_ref().expect("set up");
                ring.set(0.0);
                let _ = ring.timing(1.0, 700.0, Easing::Curve(Curve::Cubic, EaseMode::Out));
            }
        }
    }

    fn render(&self, props: &StockRowProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let q = props.quote;
        let change = q.percent();
        view()
            .h(px(64.0))
            .flex_row()
            .items_center()
            .gap_3()
            .px_4()
            .on_press(cx.listener(|_: &mut Self, _, cx| cx.emit(StockRowEvent::Press)))
            .underlay(cx.color(colors::SURFACE), 0.85)
            .role(AccessibilityRole::Button)
            .label(props.name.clone())
            .when(!props.first, |v| {
                v.child(
                    view()
                        .absolute()
                        .top_0()
                        .left(px(16.0))
                        .right_0()
                        .h(px(1.0))
                        .bg(colors::LINE),
                )
            })
            .child(
                view()
                    .flex_1()
                    .gap(px(3.0))
                    .child(
                        text(props.code.clone())
                            .number_of_lines(1)
                            .text_size(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        text(props.name.clone())
                            .number_of_lines(1)
                            .text_size(px(13.0))
                            .text_color(colors::MUTED),
                    ),
            )
            .child(
                canvas(self.sparkline(props, cx))
                    .pointer_events(PointerEvents::None)
                    .w(px(SPARK_W + RING * 2.0))
                    .h(px(SPARK_H + RING * 2.0))
                    .m(px(-RING)),
            )
            .child(
                view()
                    .w(px(84.0))
                    .items_end()
                    .gap(px(3.0))
                    .child(
                        text(price(q.last, props.decimals))
                            .number_of_lines(1)
                            .text_size(px(16.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT)
                            .tabular_nums(),
                    )
                    .child(
                        text(format!("{}%", signed(change, 2)))
                            .number_of_lines(1)
                            .text_size(px(13.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(tone_of(change))
                            .tabular_nums(),
                    ),
            )
    }
}
