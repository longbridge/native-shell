//! The order book and the depth chart (as the TSX gallery's
//! `demos/stocks/Book.tsx`).

use nsh_core::{
    LayoutAnimation, LayoutCurve, LayoutProperty, LayoutTransition, Rect,
    canvas::{CanvasTextAlign, CanvasTextBaseline},
    ui::{AnyElement, prelude::*},
};

use super::{
    market::{LEVELS, Level, compact, price, tone_of},
    parts::{Block, Calm, Col, GRID, SpringBar, SpringBarProps, hatch, head, spring_bar},
};
use crate::ui::{PressProps, colors, legend, press_with, radius};

const ROW: f32 = 26.0;
/// Ticks either side of the mid price the depth chart spans.
const SPAN: f64 = LEVELS as f64 + 2.0;

/// Sizes summed outward from the best price.
fn cumulative(levels: &[Level]) -> Vec<f64> {
    let mut sum = 0.0;
    levels
        .iter()
        .map(|l| {
            sum += l.size;
            sum
        })
        .collect()
}

/// Asks above bids, five levels or ten (the change animates the layout),
/// the bid / ask balance springing natively on top.
#[derive(Debug, Default)]
pub(crate) struct OrderBook {
    depth: usize,
    /// The bars' scale: the deeper side's total, eased (it changes every
    /// tick).
    scale: Calm,
    max: f64,
}

/// The props of [`OrderBook`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct OrderBookProps {
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
    pub(crate) prev_close: f64,
    pub(crate) decimals: u32,
}

impl OrderBook {
    /// The totals at the depth shown, bids and asks.
    fn totals(&self, props: &OrderBookProps) -> (f64, f64) {
        let n = self.depth - 1;
        let at = |sums: Vec<f64>| sums.get(n).copied().unwrap_or(0.0);
        (at(cumulative(&props.bids)), at(cumulative(&props.asks)))
    }

    fn fit(&mut self, props: &OrderBookProps) {
        let (bids, asks) = self.totals(props);
        self.max = self.scale.fit(0.0, bids.max(asks).max(1.0)).1;
    }
}

/// One level: its label, price, size and running total, over a bar of
/// the running total from the right edge (a depth ladder). Rows are kept
/// by their place on their side (`slot`), as the TSX demo's `Each` over
/// the places: more levels add rows at the inside of each side.
#[allow(clippy::too_many_arguments)]
fn level_row(
    level: Option<&Level>,
    total: f64,
    bid: bool,
    n: usize,
    slot: usize,
    max: f64,
    prev_close: f64,
    decimals: u32,
) -> AnyElement {
    let tint = if bid { colors::UP } else { colors::DOWN }.opacity(0.1);
    let share = (total / max).min(1.0) as f32;
    view()
        .h(px(ROW))
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .child(
            view()
                .absolute()
                .right_0()
                .top(px(1.0))
                .bottom(px(1.0))
                .w(relative(share))
                .bg(tint),
        )
        .child(
            text(format!("{}{n}", if bid { "B" } else { "A" }))
                .w(px(28.0))
                .text_size(px(12.0))
                .text_color(colors::FAINT)
                .tabular_nums(),
        )
        .child(
            text(level.map_or("–".into(), |l| price(l.price, decimals)))
                .flex_1()
                .text_size(px(14.0))
                .text_color(tone_of(level.map_or(0.0, |l| l.price) - prev_close))
                .tabular_nums(),
        )
        .child(
            text(level.map_or(String::new(), |l| compact(l.size)))
                .w(px(64.0))
                .text_right()
                .text_size(px(14.0))
                .text_color(colors::TEXT)
                .tabular_nums(),
        )
        .child(
            text(compact(total))
                .w(px(64.0))
                .pr(px(6.0))
                .text_right()
                .text_size(px(12.0))
                .text_color(colors::FAINT)
                .tabular_nums(),
        )
        .key(format!("{}{slot}", if bid { "b" } else { "a" }))
        .into_element()
}

impl Component for OrderBook {
    type Props = OrderBookProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &OrderBookProps, _: &mut Context<'_, Self>) {
        self.depth = 5;
        self.scale = Calm::new(0.05);
        self.fit(props);
    }

    fn props_changed(
        &mut self,
        _: &OrderBookProps,
        new: &OrderBookProps,
        _: &mut Context<'_, Self>,
    ) {
        self.fit(new);
    }

    fn render(&self, props: &OrderBookProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (bids, asks) = (cumulative(&props.bids), cumulative(&props.asks));
        let (bid_total, ask_total) = self.totals(props);
        let share = if bid_total + ask_total > 0.0 {
            bid_total / (bid_total + ask_total)
        } else {
            0.5
        };
        let spread = match (props.asks.first(), props.bids.first()) {
            (Some(a), Some(b)) => a.price - b.price,
            _ => 0.0,
        };
        let toggle = cx.listener(|this: &mut Self, _: &(), cx| {
            cx.animate_layout(LayoutAnimation {
                create: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 360.0)),
                update: Some(LayoutTransition {
                    curve: LayoutCurve::Spring { damping: 0.85 },
                    property: LayoutProperty::Opacity,
                    duration_ms: 360.0,
                    delay_ms: 0.0,
                }),
                delete: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 360.0)),
            });
            this.depth = if this.depth == 5 { LEVELS } else { 5 };
            this.fit(cx.props());
            cx.notify();
        });
        let five = self.depth == 5;
        let figure = |s: String, color: AdaptiveColor| {
            text(s).text_size(px(12.0)).text_color(color).tabular_nums()
        };
        let row = |side: &[Level], sums: &[f64], k: usize, bid: bool, n: usize, slot: usize| {
            level_row(
                side.get(k),
                sums.get(k).copied().unwrap_or(0.0),
                bid,
                n,
                slot,
                self.max,
                props.prev_close,
                props.decimals,
            )
        };
        let depth = self.depth;
        Block::new()
            .title("Order book")
            .right(
                press_with(
                    PressProps {
                        hit_slop: Some(8.0),
                        label: Some(
                            if five {
                                "Show ten levels"
                            } else {
                                "Show five levels"
                            }
                            .into(),
                        ),
                        ..PressProps::default()
                    },
                    toggle,
                )
                .child(
                    view()
                        .h(px(30.0))
                        .px_3()
                        .rounded(px(radius::CONTROL))
                        .justify_center()
                        .bg(colors::RAISED)
                        .child(
                            text(if five { "5 → 10" } else { "10 → 5" })
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT)
                                .tabular_nums(),
                        ),
                ),
            )
            .child(
                view()
                    .gap(px(5.0))
                    .child(
                        view()
                            .h(px(6.0))
                            .rounded(px(3.0))
                            .overflow_hidden()
                            .bg(colors::DOWN)
                            .child(SpringBar::element(SpringBarProps {
                                height: 6.0,
                                radius: Some(0.0),
                                ..spring_bar(share, colors::UP)
                            })),
                    )
                    .child(
                        view()
                            .flex_row()
                            .child(
                                figure(format!("BID {:.0}%", share * 100.0), colors::UP).flex_1(),
                            )
                            .child(figure(
                                format!("spread {}", price(spread, props.decimals)),
                                colors::FAINT,
                            ))
                            .child(
                                figure(format!("{:.0}% ASK", (1.0 - share) * 100.0), colors::DOWN)
                                    .flex_1()
                                    .text_right(),
                            ),
                    ),
            )
            .child(
                view()
                    .child(
                        view()
                            .flex_row()
                            .gap(px(10.0))
                            .pb_1()
                            .child(head("LVL", Col::Fixed(28.0), false))
                            .child(head("PRICE", Col::Flex, false))
                            .child(head("SIZE", Col::Fixed(64.0), true))
                            .child(head("TOTAL", Col::Fixed(64.0), true)),
                    )
                    .children((0..depth).map(|i| {
                        let k = depth - 1 - i;
                        row(&props.asks, &asks, k, false, depth - i, i)
                    }))
                    .child(view().h(px(1.0)).my(px(3.0)).bg(colors::BORDER))
                    .children((0..depth).map(|i| row(&props.bids, &bids, i, true, i + 1, i))),
            )
    }
}

/// Cumulative depth: bids stepping down to the left, asks to the right,
/// the spread hatched and the mid price dashed. It is calm on purpose: it
/// redraws four times a second, the price axis is a fixed span around the
/// mid, and the size axis widens at once but narrows over about a second.
#[derive(Debug, Default)]
pub(crate) struct Depth {
    top: Calm,
    ymax: f64,
    width: f32,
}

/// The props of [`Depth`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct DepthProps {
    /// The book as of the last quarter second.
    pub(crate) bids: Vec<Level>,
    pub(crate) asks: Vec<Level>,
    pub(crate) tick: f64,
    pub(crate) decimals: u32,
}

impl Depth {
    fn fit(&mut self, props: &DepthProps) {
        let last = |levels: &[Level]| levels.iter().map(|l| l.size).sum::<f64>();
        self.ymax = self
            .top
            .fit(0.0, last(&props.bids).max(last(&props.asks)) * 1.1)
            .1;
    }

    fn draw(&self, props: &DepthProps, cx: &RenderCx<'_, Self>) -> Painter {
        let (width, height) = (self.width, 120.0);
        let (Some(best_bid), Some(best_ask)) = (props.bids.first(), props.asks.first()) else {
            return Painter::new();
        };
        if width <= 0.0 {
            return Painter::new();
        }
        let c = |color: AdaptiveColor| cx.color(color);
        let (bids, asks) = (cumulative(&props.bids), cumulative(&props.asks));
        let mid = (best_bid.price + best_ask.price) / 2.0;
        let (lo, hi) = (mid - SPAN * props.tick, mid + SPAN * props.tick);
        let (pad_top, pad_bottom) = (4.0, 18.0);
        let h = height - pad_top - pad_bottom;
        let ymax = self.ymax;
        let x_of = |p: f64| ((p - lo) / (hi - lo)) as f32 * width;
        let y_of = |v: f64| pad_top + h - (v / ymax) as f32 * h;
        let mut p = Painter::new().stroke_color(c(GRID)).line_width(1.0);
        for k in 1..=3 {
            let y = pad_top + h * k as f32 / 4.0;
            p = p.begin_path().move_to(0.0, y).line_to(width, y).stroke();
        }
        let (bx, ax) = (x_of(best_bid.price), x_of(best_ask.price));
        p = hatch(p, bx, pad_top, ax - bx, h, c(colors::BORDER), 4.0);
        // A side: a step line out from the best price, filled below.
        let side =
            |p: Painter, levels: &[Level], sums: &[f64], color: AdaptiveColor, left: bool| {
                let mut y = y_of(sums[0]);
                let mut p = p
                    .begin_path()
                    .move_to(x_of(levels[0].price), pad_top + h)
                    .line_to(x_of(levels[0].price), y);
                for i in 1..levels.len() {
                    p = p.line_to(x_of(levels[i].price), y);
                    y = y_of(sums[i]);
                    p = p.line_to(x_of(levels[i].price), y);
                }
                let edge = if left { 0.0 } else { width };
                p.line_to(edge, y)
                    .line_to(edge, pad_top + h)
                    .close_path()
                    .fill_color(c(color.opacity(0.16)))
                    .fill()
                    .stroke_color(c(color))
                    .line_width(1.5)
                    .stroke()
            };
        p = side(p, &props.bids, &bids, colors::UP, true);
        p = side(p, &props.asks, &asks, colors::DOWN, false);
        let label_y = pad_top + h + 5.0;
        p.stroke_color(c(colors::MUTED))
            .line_dash(&[2.0, 3.0])
            .begin_path()
            .move_to(x_of(mid), pad_top)
            .line_to(x_of(mid), pad_top + h)
            .stroke()
            .line_dash(&[])
            .font("11px system-ui")
            .fill_color(c(colors::FAINT))
            .text_baseline(CanvasTextBaseline::Top)
            .text_align(CanvasTextAlign::Left)
            .fill_text(&price(lo, props.decimals), 0.0, label_y)
            .text_align(CanvasTextAlign::Right)
            .fill_text(&price(hi, props.decimals), width, label_y)
            .text_align(CanvasTextAlign::Center)
            .fill_color(c(colors::TEXT))
            .fill_text(
                &format!("mid {}", price(mid, props.decimals)),
                width / 2.0,
                label_y,
            )
    }
}

impl Component for Depth {
    type Props = DepthProps;
    type Event = NoEvent;

    fn setup(&mut self, props: &DepthProps, _: &mut Context<'_, Self>) {
        self.top = Calm::new(0.25);
        self.fit(props);
    }

    fn props_changed(&mut self, _: &DepthProps, new: &DepthProps, _: &mut Context<'_, Self>) {
        self.fit(new);
    }

    fn render(&self, props: &DepthProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let total = |levels: &[Level]| levels.iter().map(|l| l.size).sum::<f64>();
        let (bids, asks) = (total(&props.bids), total(&props.asks));
        let imbalance = if bids + asks > 0.0 {
            (bids - asks) / (bids + asks)
        } else {
            0.0
        };
        Block::new()
            .title("Depth")
            .note(format!("±{SPAN} ticks · cumulative"))
            .child(
                canvas(self.draw(props, cx))
                    .h(px(120.0))
                    .label("Depth chart")
                    .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                        if this.width != frame.width {
                            this.width = frame.width;
                            cx.notify();
                        }
                    })),
            )
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_x(px(16.0))
                    .gap_y(px(6.0))
                    .child(legend(colors::UP, "Bids", Some(compact(bids))))
                    .child(legend(colors::DOWN, "Asks", Some(compact(asks))))
                    .child(legend(
                        colors::BORDER,
                        "Imbalance",
                        Some(format!(
                            "{}{:.0}%",
                            if imbalance >= 0.0 { "+" } else { "" },
                            imbalance * 100.0
                        )),
                    )),
            )
    }
}
