//! The tape and the trade flow (as the TSX gallery's
//! `demos/stocks/Tape.tsx`).

use std::f32::consts::TAU;

use nsh_core::{
    Rect,
    canvas::{CanvasTextAlign, CanvasTextBaseline},
    nsh_animation::{Curve, EaseMode},
    ui::{AnyElement, prelude::*},
};

use super::{
    market::{BLOCK, BUCKETS, TAPE, TAPE_HELD, Trade, compact, price, time},
    parts::{Block, Col, GRID, SpringBar, SpringBarProps, head, spring_bar},
};
use crate::ui::{colors, legend, tints};

/// Rows shown.
const SHOWN: usize = 8;
const ROW: f32 = 28.0;

/// The tape: the newest trades on top, blocks highlighted. It is one
/// canvas redrawn four times a second (its section's totals change every
/// tick, outside it); the new rows slide in by a native
/// offset (the canvas redraws itself as the value moves, without a
/// render).
#[derive(Debug, Default)]
pub(crate) struct Tape {
    offset: Option<NativeValue>,
    /// The newest trade shown so far (-1: none yet).
    top: i64,
    width: f32,
}

/// The props of [`Tape`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct TapeProps {
    /// The newest trades as of the last quarter second.
    pub(crate) shown: Vec<Trade>,
    pub(crate) decimals: u32,
}

impl Tape {
    fn draw(&self, props: &TapeProps, cx: &RenderCx<'_, Self>) -> Painter {
        let (width, height) = (self.width, SHOWN as f32 * ROW);
        let c = |color: AdaptiveColor| cx.color(color);
        let mut p = Painter::new()
            .save()
            .begin_path()
            .rect(0.0, 0.0, width, height)
            .clip()
            .translate(0.0, self.offset.as_ref().expect("set up"))
            .text_baseline(CanvasTextBaseline::Middle);
        for (i, t) in props.shown.iter().enumerate() {
            let top = i as f32 * ROW;
            let y = top + ROW / 2.0;
            let block = t.size >= BLOCK;
            let color = if t.side > 0.0 {
                colors::UP
            } else {
                colors::DOWN
            };
            if block {
                p = p
                    .fill_color(c(tints::ACCENT))
                    .begin_path()
                    .round_rect(0.0, top + 2.0, width, ROW - 4.0, 8.0)
                    .fill();
            } else if i > 0 {
                p = p
                    .stroke_color(c(GRID))
                    .line_width(1.0)
                    .begin_path()
                    .move_to(0.0, top)
                    .line_to(width, top)
                    .stroke();
            }
            p = p
                .font("12px system-ui")
                .text_align(CanvasTextAlign::Left)
                .fill_color(c(colors::FAINT))
                .fill_text(&time(t.t, true), 8.0, y)
                .fill_color(c(color))
                .begin_path()
                .arc(84.0, y, 3.0, 0.0, TAU)
                .fill()
                .font("14px system-ui")
                .fill_text(&price(t.price, props.decimals), 96.0, y)
                .text_align(CanvasTextAlign::Right)
                .font(if block {
                    "600 14px system-ui"
                } else {
                    "14px system-ui"
                })
                .fill_color(c(if block { colors::ACCENT } else { colors::TEXT }))
                .fill_text(&compact(t.size), width - 8.0, y);
            if block {
                p = p
                    .font("600 10px system-ui")
                    .fill_text("BLOCK", width - 70.0, y);
            }
        }
        p.restore()
    }
}

impl Component for Tape {
    type Props = TapeProps;
    type Event = NoEvent;

    fn setup(&mut self, _: &TapeProps, cx: &mut Context<'_, Self>) {
        self.offset = Some(cx.native_value(0.0));
        self.top = -1;
    }

    fn props_changed(&mut self, old: &TapeProps, new: &TapeProps, _: &mut Context<'_, Self>) {
        if old.shown == new.shown {
            return;
        }
        // What came in since the last frame, and the slide that shows it.
        let Some(first) = new.shown.first() else {
            return;
        };
        let fresh = if self.top < 0 {
            0
        } else {
            first.id as i64 - self.top
        };
        self.top = first.id as i64;
        // Ids are consecutive: past the trades kept, nothing old is left.
        let k = if fresh >= TAPE as i64 {
            TAPE_HELD as i64
        } else {
            fresh.min(4)
        };
        if k > 0 {
            let offset = self.offset.as_ref().expect("set up");
            offset.set(-(k as f32) * ROW);
            let _ = offset.timing(0.0, 220.0, Easing::Curve(Curve::Cubic, EaseMode::Out));
        }
    }

    fn render(&self, props: &TapeProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().child(
            canvas(self.draw(props, cx))
                .h(px(SHOWN as f32 * ROW))
                .label("Recent trades")
                .on_layout(cx.listener(|this: &mut Self, frame: &Rect, cx| {
                    if this.width != frame.width {
                        this.width = frame.width;
                        cx.notify();
                    }
                })),
        )
    }
}

/// The tape's section: its totals (every tick), the columns and the
/// [`Tape`] (four times a second).
pub(crate) fn tape(buys: f64, sells: f64, blocks: u32, tape: TapeProps) -> Block {
    Block::new()
        .title("Time & sales")
        .note(format!("blocks ≥ {}", compact(BLOCK)))
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .gap_x(px(16.0))
                .gap_y(px(6.0))
                .child(legend(colors::UP, "At ask", Some(compact(buys))))
                .child(legend(colors::DOWN, "At bid", Some(compact(sells))))
                .child(legend(colors::ACCENT, "Blocks", Some(blocks.to_string()))),
        )
        .child(
            view()
                .flex_row()
                .gap(px(10.0))
                .px_2()
                .child(head("TIME", Col::Fixed(80.0), false))
                .child(head("PRICE", Col::Flex, false))
                .child(head("SIZE", Col::Auto, true)),
        )
        .child(Tape::element(tape))
}

/// Bought vs sold volume by order size: who is moving the price; once a
/// second.
#[derive(Debug, Default)]
pub(crate) struct Flow;

/// The props of [`Flow`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct FlowProps {
    /// Volume bought, by bucket.
    pub(crate) buy: Vec<f64>,
    /// Volume sold, by bucket.
    pub(crate) sell: Vec<f64>,
}

/// One bucket: sold to the left of the middle, bought to the right.
fn flow_row(label: &'static str, note: &'static str, buy: f64, sell: f64, max: f64) -> AnyElement {
    let figure = |s: String, color: AdaptiveColor| {
        text(s).text_size(px(12.0)).tabular_nums().text_color(color)
    };
    view()
        .gap_1()
        .child(
            view()
                .flex_row()
                .items_baseline()
                .gap_2()
                .child(
                    text(label)
                        .flex_1()
                        .text_size(px(13.0))
                        .text_color(colors::TEXT)
                        .child(
                            text(format!(" {note}"))
                                .text_size(px(12.0))
                                .text_color(colors::FAINT)
                                .tabular_nums(),
                        ),
                )
                .child(figure(compact(sell), colors::DOWN))
                .child(figure("/".into(), colors::FAINT))
                .child(figure(compact(buy), colors::UP)),
        )
        .child(
            view()
                .flex_row()
                .items_center()
                .child(view().flex_1().child(SpringBar::element(SpringBarProps {
                    from_right: true,
                    height: 10.0,
                    radius: Some(3.0),
                    ..spring_bar(sell / max, colors::DOWN)
                })))
                .child(
                    view()
                        .w(px(2.0))
                        .h(px(18.0))
                        .mx(px(2.0))
                        .rounded(px(1.0))
                        .bg(colors::BORDER),
                )
                .child(view().flex_1().child(SpringBar::element(SpringBarProps {
                    height: 10.0,
                    radius: Some(3.0),
                    ..spring_bar(buy / max, colors::UP)
                }))),
        )
        .into_element()
}

impl Component for Flow {
    type Props = FlowProps;
    type Event = NoEvent;

    fn render(&self, props: &FlowProps, _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let at = |v: &Vec<f64>, i: usize| v.get(i).copied().unwrap_or(0.0);
        let max = props
            .buy
            .iter()
            .chain(&props.sell)
            .fold(1.0_f64, |m, &v| m.max(v));
        let net = props.buy.iter().sum::<f64>() - props.sell.iter().sum::<f64>();
        let big = at(&props.buy, 2) + at(&props.buy, 3) - at(&props.sell, 2) - at(&props.sell, 3);
        let sign = |v: f64| if v >= 0.0 { "+" } else { "" };
        Block::new()
            .title("Trade flow")
            .note("sold ← · → bought")
            .child(
                text(format!("{}{}", sign(net), compact(net)))
                    .text_size(px(22.0))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors::TEXT)
                    .tabular_nums()
                    .child(
                        text(format!(" net · {}{} large orders", sign(big), compact(big)))
                            .text_size(px(13.0))
                            .font_weight(FontWeight::NORMAL)
                            .tabular_nums()
                            .text_color(if big >= 0.0 { colors::UP } else { colors::DOWN }),
                    ),
            )
            .child(
                view()
                    .gap(px(10.0))
                    .children(BUCKETS.iter().enumerate().map(|(i, b)| {
                        flow_row(b.label, b.note, at(&props.buy, i), at(&props.sell, i), max)
                    })),
            )
    }
}
