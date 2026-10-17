//! A stock's screen (as the TSX gallery's `demos/stocks/Detail.tsx`):
//! quote and chart on the first screen, then the book, the tape, depth,
//! the flow and the ranges, all live. Buy / Sell at the bottom open the
//! order ticket in a sheet.

use std::{cell::RefCell, f32::consts::TAU, rc::Rc};

use nsh_core::{
    Rect,
    canvas::{CanvasTextAlign, CanvasTextBaseline, LineCap},
    nsh_animation::{Curve, EaseMode},
    ui::prelude::*,
};
use nsh_ui_mobile::{Sheet, SheetEvent, SheetProps};

use super::{
    Feed, FocusTick, Running,
    book::{Depth, DepthProps, OrderBook, OrderBookProps},
    chart::{Chart, ChartProps},
    market::{HORIZONS, Market, Quote, Range, compact, price, signed, time, tone_of},
    parts::{Block, live_dot},
    tape::{Flow, FlowProps, TapeProps, tape},
    ticket::{Side, Ticket, TicketEvent, TicketProps},
};
use crate::ui::{Button, ButtonVariant, Finish, Finished, IconName, Stat, colors, icon, radius};

/// A stock's screen: it focuses the stock while it is shown.
#[derive(Default)]
pub(crate) struct Detail {
    market: Option<Rc<RefCell<Market>>>,
    side: Side,
    ticket: bool,
    /// The confirmation's motion: 0 hidden, 1 shown.
    banner: Option<NativeValue>,
    banner_run: Option<Animation>,
    message: (SharedString, SharedString, Side),
}

/// The props of [`Detail`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct DetailProps {
    /// The stock's index in the market.
    pub(crate) stock: u32,
}

impl Detail {
    /// The confirmation drops in from the top and leaves by itself.
    fn show_banner(&mut self, title: SharedString, detail: SharedString, side: Side) {
        self.message = (title, detail, side);
        let t = self.banner.as_ref().expect("set up");
        let run = Animation::sequence([
            Animation::spring(
                t,
                1.0,
                Spring {
                    stiffness: 260.0,
                    damping: 22.0,
                    ..Spring::default()
                },
            ),
            Animation::delay(2200.0),
            Animation::timing(
                t,
                0.0,
                Timing::new(260.0, Easing::Curve(Curve::Cubic, EaseMode::In)),
            ),
        ]);
        let _ = run.start();
        self.banner_run = Some(run);
    }
}

impl Component for Detail {
    type Props = DetailProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &DetailProps, cx: &mut Context<'_, Self>) {
        self.market = cx.update_global::<Feed, _>(|feed, _| feed.market.clone());
        if let Some(market) = &self.market {
            market.borrow_mut().focus(Some(props.stock as usize));
        }
        self.banner = Some(cx.native_value(0.0));
    }

    fn cleanup(&mut self, _: &DetailProps, _: &mut Context<'_, Self>) {
        if let Some(market) = &self.market {
            market.borrow_mut().focus(None);
        }
    }

    fn render(&self, props: &DetailProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let _ = cx.global::<FocusTick>().0;
        let running = cx.global::<Running>().0;
        let bottom = cx.insets().bottom;
        let Some(market) = &self.market else {
            return view().flex_1().bg(colors::BACKGROUND);
        };
        let market = market.borrow();
        let index = props.stock as usize;
        let stock = &market.stocks[index];
        let q = stock.quote;
        let decimals = stock.decimals();
        let (year_lo, year_hi) = stock
            .days
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), d| {
                (lo.min(d.l), hi.max(d.h))
            });
        let stats = market.stats;
        let trades = &market.trades;
        let (buys, sells, blocks) = trades.iter().fold((0.0, 0.0, 0), |(b, s, k), t| {
            let block = u32::from(t.size >= super::market::BLOCK);
            if t.side > 0.0 {
                (b + t.size, s, k + block)
            } else {
                (b, s + t.size, k + block)
            }
        });
        let trade = |side: Side| {
            cx.listener(move |this: &mut Self, _: &(), cx| {
                this.side = side;
                this.ticket = true;
                cx.notify();
            })
        };
        let t = self.banner.as_ref().expect("set up");
        let (title, detail, side) = self.message.clone();
        let content = scroll()
            .flex_1()
            .child(QuoteSection::element(QuoteProps {
                name: stock.name.into(),
                sector: stock.sector.label().into(),
                quote: q,
                clock: market.clock,
                running,
                decimals,
                year_lo,
                year_hi,
                tick: stock.tick,
            }))
            .child(Block::new().child(Chart::element(ChartProps {
                code: stock.code.into(),
                quote: q,
                minutes: stock.minutes.clone(),
                days: stock.days.to_vec(),
                running,
                decimals,
            })))
            .child(OrderBook::element(OrderBookProps {
                bids: market.book.bids.clone(),
                asks: market.book.asks.clone(),
                prev_close: q.prev_close,
                decimals,
            }))
            .child(tape(
                buys,
                sells,
                blocks,
                TapeProps {
                    shown: market.tape.clone(),
                    decimals,
                },
            ))
            .child(Depth::element(DepthProps {
                bids: market.depth.bids.clone(),
                asks: market.depth.asks.clone(),
                tick: stock.tick,
                decimals,
            }))
            .child(Flow::element(FlowProps {
                buy: market.flow.buy.to_vec(),
                sell: market.flow.sell.to_vec(),
            }))
            .child(Ranges::element(RangesProps {
                rows: market.ranges.0.clone(),
                last: market.ranges.1.last,
                prev_close: market.ranges.1.prev_close,
                decimals,
            }))
            .child(
                text(format!(
                    "simulated · {:.2} ms Rust/tick · {} quotes/s",
                    stats.ms, stats.updates
                ))
                .px_4()
                .pt_2()
                .pb(px(96.0 + bottom))
                .text_size(px(12.0))
                .text_color(colors::FAINT)
                .tabular_nums(),
            );
        // The trade bar, over the content.
        let bar = view()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .px_4()
            .pt(px(10.0))
            .pb(px(10.0 + bottom))
            // Glass: the page shows through, blurred, as it scrolls under.
            .finish(Finish::Glass)
            .border_t_1()
            .when_some(Finish::Glass.border_color(), |v, c| v.border_t_color(c))
            .child(
                view()
                    .w(px(92.0))
                    .gap(px(1.0))
                    .child(
                        text(price(q.last, decimals))
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT)
                            .tabular_nums(),
                    )
                    .child(
                        text(format!("{}%", signed(q.percent(), 2)))
                            .text_size(px(12.0))
                            .tabular_nums()
                            .text_color(tone_of(q.percent())),
                    ),
            )
            .child(
                Button::new("Sell", trade(Side::Sell))
                    .variant(ButtonVariant::Secondary)
                    .flex_1(),
            )
            .child(Button::new("Buy", trade(Side::Buy)).flex_1());
        let banner = view()
            .pointer_events(PointerEvents::None)
            .absolute()
            .top(px(10.0))
            .left_4()
            .right_4()
            .flex_row()
            .items_center()
            .gap_3()
            .p(px(14.0))
            .rounded(px(radius::INSET + 3.0))
            .border_curve(BorderCurve::Continuous)
            .bg(colors::LIGHT)
            .bind_opacity(&t.interpolate(&[0.0, 0.3, 1.0], &[0.0, 1.0, 1.0]))
            .bind_translate_y(&t.interpolate(&[0.0, 1.0], &[-90.0, 0.0]))
            .bind_scale(&t.interpolate(&[0.0, 1.0], &[0.92, 1.0]))
            .child(
                view()
                    .size(px(34.0))
                    .rounded(px(17.0))
                    .items_center()
                    .justify_center()
                    .bg(if side == Side::Buy {
                        colors::UP
                    } else {
                        colors::DOWN
                    })
                    .child(icon(IconName::Check, 18.0, colors::ON_LIGHT)),
            )
            .child(
                view()
                    .flex_1()
                    .gap(px(2.0))
                    .child(
                        text(title)
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::ON_LIGHT),
                    )
                    .child(
                        text(detail)
                            .number_of_lines(1)
                            .text_size(px(13.0))
                            .text_color(AdaptiveColor::new(rgb(0xa4a8ae), rgb(0x4a4e55))),
                    ),
            );
        let sheet = Sheet::element(SheetProps {
            open: self.ticket,
            ..SheetProps::default()
        })
        .on_event(cx.listener(|this: &mut Self, _: &SheetEvent, cx| {
            this.ticket = false;
            cx.notify();
        }))
        .child(
            Ticket::element(TicketProps {
                open: self.ticket,
                code: stock.code.into(),
                tick: stock.tick,
                last: q.last,
                side: self.side,
            })
            .on_event(cx.listener(|this: &mut Self, e: &TicketEvent, cx| {
                match e {
                    TicketEvent::Side(side) => this.side = *side,
                    TicketEvent::Placed {
                        title,
                        detail,
                        side,
                    } => {
                        this.ticket = false;
                        this.show_banner(title.clone(), detail.clone(), *side);
                    }
                }
                cx.notify();
            })),
        );
        view()
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(content)
            .child(bar)
            .child(banner)
            .child(sheet)
    }
}

/// The quote: a big figure, a direction arrow that turns over natively,
/// the day's range and figures.
#[derive(Debug, Default)]
struct QuoteSection {
    /// The day is down: the arrow points down.
    down: bool,
    turn: Option<NativeValue>,
}

#[derive(Debug, Clone, PartialEq, Props)]
struct QuoteProps {
    name: SharedString,
    sector: SharedString,
    quote: Quote,
    clock: f64,
    running: bool,
    decimals: u32,
    year_lo: f64,
    year_hi: f64,
    tick: f64,
}

impl Component for QuoteSection {
    type Props = QuoteProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &QuoteProps, cx: &mut Context<'_, Self>) {
        self.down = props.quote.last < props.quote.prev_close;
        self.turn = Some(cx.native_value(if self.down { 1.0 } else { 0.0 }));
    }

    fn props_changed(&mut self, _: &QuoteProps, new: &QuoteProps, _: &mut Context<'_, Self>) {
        let down = new.quote.last < new.quote.prev_close;
        if down != self.down {
            self.down = down;
            let _ = self.turn.as_ref().expect("set up").spring(
                if down { 1.0 } else { 0.0 },
                Spring {
                    stiffness: 300.0,
                    damping: 18.0,
                    ..Spring::default()
                },
            );
        }
    }

    fn render(&self, props: &QuoteProps, _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let q = props.quote;
        let change = q.last - q.prev_close;
        let tone = tone_of(change);
        let p = |v: f64| price(v, props.decimals);
        let turn = self.turn.as_ref().expect("set up");
        let amplitude = (q.high - q.low) / q.prev_close * 100.0;
        let stats = [
            ("Open", p(q.open)),
            ("Prev close", p(q.prev_close)),
            (
                "Avg price",
                p(q.turnover / if q.volume == 0.0 { 1.0 } else { q.volume }),
            ),
            ("Amplitude", format!("{amplitude:.2}%")),
            ("Volume", compact(q.volume)),
            ("Turnover", compact(q.turnover)),
            ("120d high", p(props.year_hi)),
            ("120d low", p(props.year_lo)),
        ];
        view()
            .px_4()
            .pt_3()
            .pb(px(14.0))
            .gap(px(10.0))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        text(props.name.clone())
                            .number_of_lines(1)
                            .flex_1()
                            .text_size(px(15.0))
                            .text_color(colors::MUTED),
                    )
                    .child(
                        text(props.sector.clone())
                            .text_size(px(13.0))
                            .text_color(colors::FAINT),
                    )
                    .child(live_dot(props.running, 6.0))
                    .child(
                        text(time(props.clock, true))
                            .text_size(px(12.0))
                            .text_color(colors::MUTED)
                            .tabular_nums(),
                    ),
            )
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap(px(6.0))
                    .child(
                        text(p(q.last))
                            .text_size(px(36.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .letter_spacing(-1.0)
                            .text_color(tone)
                            .tabular_nums(),
                    )
                    .child(
                        view()
                            .bind_rotate(&turn.interpolate(&[0.0, 1.0], &[0.0, 180.0]))
                            .child(icon(IconName::ArrowUp, 20.0, tone)),
                    )
                    .child(
                        view()
                            .flex_1()
                            .items_end()
                            .gap(px(1.0))
                            .child(
                                text(format!("{}%", signed(q.percent(), 2)))
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .tabular_nums()
                                    .text_color(tone),
                            )
                            .child(
                                text(format!("{} today", signed(change, props.decimals as usize)))
                                    .text_size(px(12.0))
                                    .tabular_nums()
                                    .text_color(tone),
                            ),
                    ),
            )
            .child(DayRange::element(DayRangeProps {
                quote: q,
                tick: props.tick,
                decimals: props.decimals,
            }))
            .child(
                view().flex_row().flex_wrap().children(
                    stats
                        .into_iter()
                        .map(|(label, value)| Stat::new(label, value).width(0.25)),
                ),
            )
    }
}

/// Where the last price sits in the day, in one line: the low–high span
/// filled, the previous close marked, the last price a knob that springs
/// natively.
#[derive(Debug, Default)]
struct DayRange {
    x: Option<NativeValue>,
    width: f32,
    placed: bool,
}

#[derive(Debug, Clone, PartialEq, Props)]
struct DayRangeProps {
    quote: Quote,
    tick: f64,
    decimals: u32,
}

impl DayRangeProps {
    /// Where `v` sits in the span, 0..1.
    fn at(&self, v: f64) -> f32 {
        let q = self.quote;
        let lo = q.low.min(q.prev_close);
        let hi = q.high.max(q.prev_close);
        let hi = if hi > lo { hi } else { lo + self.tick };
        ((v - lo) / (hi - lo)) as f32
    }
}

impl DayRange {
    fn place(&mut self, props: &DayRangeProps) {
        if self.width <= 0.0 {
            return;
        }
        let to = props.at(props.quote.last) * self.width;
        let x = self.x.as_ref().expect("set up");
        if self.placed {
            let _ = x.spring(
                to,
                Spring {
                    stiffness: 220.0,
                    damping: 24.0,
                    ..Spring::default()
                },
            );
        } else {
            x.set(to);
        }
        self.placed = true;
    }
}

impl Component for DayRange {
    type Props = DayRangeProps;
    type Event = NoEvent;

    fn setup(&mut self, _: &DayRangeProps, cx: &mut Context<'_, Self>) {
        self.x = Some(cx.native_value(0.0));
    }

    fn props_changed(&mut self, _: &DayRangeProps, new: &DayRangeProps, _: &mut Context<'_, Self>) {
        self.place(new);
    }

    fn render(&self, props: &DayRangeProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let q = props.quote;
        let p = |v: f64| price(v, props.decimals);
        let label = |s: String| {
            text(s)
                .text_size(px(12.0))
                .text_color(colors::MUTED)
                .tabular_nums()
        };
        view()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .child(label(format!("L {}", p(q.low))))
            .child(
                view()
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.width != frame.width {
                            this.width = frame.width;
                            this.place(cx.props());
                        }
                    }))
                    .label(format!("Day range {} to {}", p(q.low), p(q.high)))
                    .flex_1()
                    .h(px(16.0))
                    .justify_center()
                    .child(view().h(px(4.0)).rounded(px(2.0)).bg(colors::RAISED))
                    .child(
                        view()
                            .absolute()
                            .left(relative(props.at(q.low)))
                            .w(relative(props.at(q.high) - props.at(q.low)))
                            .h(px(4.0))
                            .rounded(px(2.0))
                            .bg(colors::BORDER),
                    )
                    .child(
                        view()
                            .absolute()
                            .left(relative(props.at(q.prev_close)))
                            .w(px(2.0))
                            .h(px(12.0))
                            .ml(px(-1.0))
                            .rounded(px(1.0))
                            .bg(colors::MUTED),
                    )
                    .child(
                        view()
                            .absolute()
                            .left(px(-7.0))
                            .size(px(14.0))
                            .rounded(px(7.0))
                            .border(3.0)
                            .border_color(colors::BACKGROUND)
                            .bg(tone_of(q.last - q.prev_close))
                            .bind_translate_x(self.x.as_ref().expect("set up")),
                    ),
            )
            .child(label(format!("H {}", p(q.high))))
    }
}

/// Ranges over 5 to 120 days as dumbbells (low → high), the last price a
/// line across them and the previous close dashed; once a second.
#[derive(Debug, Default)]
struct Ranges {
    width: f32,
}

#[derive(Debug, Clone, PartialEq, Props)]
struct RangesProps {
    rows: Vec<Range>,
    last: f64,
    prev_close: f64,
    decimals: u32,
}

impl Ranges {
    fn draw(&self, props: &RangesProps, cx: &RenderCx<'_, Self>) -> Painter {
        let r = &props.rows;
        let width = self.width;
        if r.is_empty() || width <= 0.0 {
            return Painter::new();
        }
        let lo = r.iter().fold(f64::INFINITY, |m, x| m.min(x.lo));
        let hi = r.iter().fold(f64::NEG_INFINITY, |m, x| m.max(x.hi));
        let (left, right) = (40.0, 6.0);
        let span = if hi - lo == 0.0 { 1.0 } else { hi - lo };
        let x_of = |v: f64| left + ((v - lo) / span) as f32 * (width - left - right);
        let c = |color: AdaptiveColor| cx.color(color);
        let p_of = |v: f64| price(v, props.decimals);
        let mut p = Painter::new()
            .font("11px system-ui")
            .text_baseline(CanvasTextBaseline::Middle);
        for (i, row) in r.iter().enumerate() {
            let y = 10.0 + i as f32 * 34.0;
            p = p
                .fill_color(c(colors::MUTED))
                .text_align(CanvasTextAlign::Left)
                .fill_text(&format!("{}d", row.n), 0.0, y)
                .stroke_color(c(colors::BORDER))
                .line_width(3.0)
                .line_cap(LineCap::Round)
                .begin_path()
                .move_to(x_of(row.lo), y)
                .line_to(x_of(row.hi), y)
                .stroke();
            for (v, color) in [(row.lo, colors::DOWN), (row.hi, colors::UP)] {
                p = p
                    .fill_color(c(color))
                    .begin_path()
                    .arc(x_of(v), y, 4.5, 0.0, TAU)
                    .fill();
            }
            p = p
                .fill_color(c(colors::FAINT))
                .text_align(CanvasTextAlign::Left)
                .fill_text(&p_of(row.lo), x_of(row.lo) - 4.0, y + 13.0)
                .text_align(CanvasTextAlign::Right)
                .fill_text(&p_of(row.hi), x_of(row.hi) + 4.0, y + 13.0);
        }
        let bottom = HORIZONS.len() as f32 * 34.0;
        p.stroke_color(c(colors::FAINT))
            .line_width(1.0)
            .line_dash(&[3.0, 4.0])
            .begin_path()
            .move_to(x_of(props.prev_close), 0.0)
            .line_to(x_of(props.prev_close), bottom)
            .stroke()
            .line_dash(&[])
            .stroke_color(c(colors::ACCENT))
            .line_width(2.0)
            .begin_path()
            .move_to(x_of(props.last), 0.0)
            .line_to(x_of(props.last), bottom)
            .stroke()
            .fill_color(c(colors::ACCENT))
            .text_align(CanvasTextAlign::Center)
            .fill_text(
                &format!("last {}", p_of(props.last)),
                (left + 30.0).max((width - 40.0).min(x_of(props.last))),
                bottom + 8.0,
            )
    }
}

impl Component for Ranges {
    type Props = RangesProps;
    type Event = NoEvent;

    fn render(&self, props: &RangesProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        Block::new()
            .title("Ranges")
            .note("low → high · each second")
            .child(
                canvas(self.draw(props, cx))
                    .h(px(HORIZONS.len() as f32 * 34.0 + 16.0))
                    .label("Price ranges")
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.width != frame.width {
                            this.width = frame.width;
                            cx.notify();
                        }
                    })),
            )
    }
}
