//! The order ticket (as the TSX gallery's `demos/stocks/Ticket.tsx`), the
//! content of a sheet: side, a limit price stepper, quantity chips and the
//! estimated cost; reviewing expands the ticket (a layout animation) and
//! placing reports [`TicketEvent::Placed`]. Nothing is sent anywhere.

use nsh_core::{
    LayoutAnimation, LayoutCurve, LayoutProperty, LayoutTransition,
    ui::{AnyElement, PropEnum, prelude::*},
};

use super::market::{grouped, price};
use crate::ui::{
    Button, ButtonVariant, Finish, Finished, IconName, PressProps, Segment, Segmented,
    SegmentedEvent, SegmentedProps, colors, icon, press_with, radius, tints,
};

/// Buying or selling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub(crate) enum Side {
    #[default]
    Buy,
    Sell,
}

impl Side {
    fn verb(self) -> &'static str {
        match self {
            Side::Buy => "Buy",
            Side::Sell => "Sell",
        }
    }
}

/// Simulated buying power.
const POWER: f64 = 250_000.0;
const QUANTITIES: [f64; 4] = [100.0, 500.0, 1000.0, 5000.0];

/// Reviewing and editing: the ticket's layout springs.
const EXPAND: LayoutAnimation = LayoutAnimation {
    create: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 380.0)),
    update: Some(LayoutTransition {
        curve: LayoutCurve::Spring { damping: 0.86 },
        property: LayoutProperty::Opacity,
        duration_ms: 380.0,
        delay_ms: 0.0,
    }),
    delete: Some(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 380.0)),
};

/// The ticket: each opening starts from the last price, not reviewing.
#[derive(Debug, Default)]
pub(crate) struct Ticket {
    limit: f64,
    quantity: f64,
    review: bool,
}

/// The props of [`Ticket`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct TicketProps {
    /// The sheet is open.
    pub(crate) open: bool,
    pub(crate) code: SharedString,
    pub(crate) tick: f64,
    /// The last price, live.
    pub(crate) last: f64,
    pub(crate) side: Side,
}

/// What a [`Ticket`] reports.
#[derive(Debug, Clone, PartialEq, Event)]
pub(crate) enum TicketEvent {
    /// The side was switched.
    Side(Side),
    /// The order was placed.
    Placed {
        title: SharedString,
        detail: SharedString,
        side: Side,
    },
}

impl Ticket {
    fn expand(&mut self, open: bool, cx: &mut Context<'_, Self>) {
        cx.animate_layout(EXPAND);
        self.review = open;
        cx.notify();
    }
}

/// A square 44 pt stepper button.
fn step(plus: bool, label: &'static str, on_press: Listener<()>) -> impl IntoElement {
    press_with(
        PressProps {
            label: Some(label.into()).into(),
            ..PressProps::default()
        },
        on_press,
    )
    .child(
        view()
            .size(px(44.0))
            .rounded(px(radius::CONTROL))
            .items_center()
            .justify_center()
            .finish(Finish::Pill)
            .border_1()
            .map(|v| {
                if plus {
                    v.child(icon(IconName::Plus, 18.0, colors::TEXT))
                } else {
                    v.child(
                        view()
                            .w(px(14.0))
                            .h(px(2.0))
                            .rounded(px(1.0))
                            .bg(colors::TEXT),
                    )
                }
            }),
    )
}

/// A line of the review: a label and a value.
fn line(label: &'static str, value: String, strong: bool) -> View {
    view()
        .flex_row()
        .items_baseline()
        .min_h(px(24.0))
        .child(
            text(label)
                .flex_1()
                .text_size(px(14.0))
                .text_color(colors::MUTED),
        )
        .child(
            text(value)
                .text_size(px(if strong { 17.0 } else { 14.0 }))
                .font_weight(if strong {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .text_color(colors::TEXT)
                .tabular_nums(),
        )
}

/// A caption row: a muted label, a value on the right.
fn caption(label: String) -> View {
    view().flex_row().items_baseline().child(
        text(label)
            .flex_1()
            .text_size(px(14.0))
            .text_color(colors::MUTED),
    )
}

impl Component for Ticket {
    type Props = TicketProps;
    type Event = TicketEvent;

    fn setup(&mut self, props: &TicketProps, _: &mut Context<'_, Self>) {
        self.limit = props.last;
        self.quantity = 500.0;
        self.review = false;
    }

    fn props_changed(&mut self, old: &TicketProps, new: &TicketProps, _: &mut Context<'_, Self>) {
        if new.open && !old.open {
            self.limit = new.last;
            self.review = false;
        }
    }

    fn render(&self, props: &TicketProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let decimals = super::market::decimals(props.tick);
        let p = |v: f64| price(v, decimals);
        let cost = self.limit * self.quantity;
        let fee = (cost * 0.0003).max(1.0);
        let buy = props.side == Side::Buy;
        let tone = if buy { colors::UP } else { colors::DOWN };
        let verb = props.side.verb();
        let nudge = |steps: f64| {
            cx.listener(move |this: &mut Self, _: &(), cx| {
                let tick = cx.props().tick;
                this.limit = tick.max(((this.limit / tick).round() + steps) * tick);
                cx.notify();
            })
        };
        let chips: Vec<AnyElement> = QUANTITIES
            .iter()
            .map(|&q| {
                let selected = self.quantity == q;
                view()
                    .flex_1()
                    .child(
                        press_with(
                            PressProps {
                                label: Some(format!("{q} shares").into()).into(),
                                selected: selected.into(),
                                ..PressProps::default()
                            },
                            cx.listener(move |this: &mut Self, _: &(), cx| {
                                this.quantity = q;
                                cx.notify();
                            }),
                        )
                        .child(
                            view()
                                .h(px(44.0))
                                .rounded(px(radius::CONTROL))
                                .items_center()
                                .justify_center()
                                .bg(if selected {
                                    tints::ACCENT
                                } else {
                                    colors::RAISED
                                })
                                .border_1()
                                .border_color(if selected {
                                    colors::ACCENT
                                } else {
                                    colors::BORDER
                                })
                                .child(
                                    text(if q >= 1000.0 {
                                        format!("{}K", q / 1000.0)
                                    } else {
                                        format!("{q}")
                                    })
                                    .text_size(px(14.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .tabular_nums()
                                    .text_color(if selected {
                                        colors::ACCENT
                                    } else {
                                        colors::TEXT
                                    }),
                                ),
                        ),
                    )
                    .into_element()
            })
            .collect();
        let review = if self.review {
            view()
                .gap_3()
                .p(px(14.0))
                .rounded(px(radius::INSET))
                .border_curve(BorderCurve::Continuous)
                .border_1()
                .border_color(tone)
                .bg(if buy { tints::UP } else { tints::DOWN })
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .child(
                            text(format!(
                                "{verb} {} {}",
                                grouped(self.quantity, 0),
                                props.code
                            ))
                            .flex_1()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(tone),
                        )
                        .child(
                            text("LIMIT · DAY")
                                .text_size(px(12.0))
                                .text_color(colors::MUTED)
                                .tabular_nums(),
                        ),
                )
                .child(
                    view()
                        .gap(px(2.0))
                        .child(line("Limit", p(self.limit), false))
                        .child(line("Fees (est.)", grouped(fee, 2), false))
                        .child(line(
                            "Total",
                            grouped(if buy { cost + fee } else { cost - fee }, 2),
                            true,
                        )),
                )
                .child(
                    view()
                        .flex_row()
                        .gap(px(10.0))
                        .child(
                            Button::new(
                                "Edit",
                                cx.listener(|this: &mut Self, _: &(), cx| this.expand(false, cx)),
                            )
                            .variant(ButtonVariant::Secondary)
                            .flex_1(),
                        )
                        .child(
                            Button::new(
                                format!("Place {} order", verb.to_lowercase()),
                                cx.listener(|this: &mut Self, _: &(), cx| {
                                    let props = cx.props().clone();
                                    let decimals = super::market::decimals(props.tick);
                                    let verb = props.side.verb();
                                    cx.emit(TicketEvent::Placed {
                                        title: format!("{verb} order placed").into(),
                                        detail: format!(
                                            "{} {} at {} · simulated",
                                            grouped(this.quantity, 0),
                                            props.code,
                                            price(this.limit, decimals)
                                        )
                                        .into(),
                                        side: props.side,
                                    });
                                    this.expand(false, cx);
                                }),
                            )
                            .flex_1()
                            .flex_grow(2.0),
                        ),
                )
                .into_element()
        } else {
            Button::new(
                format!("Review {}", verb.to_lowercase()),
                cx.listener(|this: &mut Self, _: &(), cx| this.expand(true, cx)),
            )
            .into_element()
        };
        view()
            .gap_4()
            .pt_1()
            .child(
                view()
                    .flex_row()
                    .items_baseline()
                    .gap_2()
                    .child(
                        text(format!("{verb} {}", props.code))
                            .flex_1()
                            .text_size(px(20.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        text("simulated · nothing is sent")
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .tabular_nums(),
                    ),
            )
            .child(
                Segmented::element(SegmentedProps {
                    options: vec![Segment::new("Buy"), Segment::new("Sell")].into(),
                    selected: (u32::from(!buy)).into(),
                    ..SegmentedProps::default()
                })
                .on_event(cx.listener(|_: &mut Self, e: &SegmentedEvent, cx| {
                    let SegmentedEvent::Change(i) = e;
                    cx.emit(TicketEvent::Side(if *i == 0 {
                        Side::Buy
                    } else {
                        Side::Sell
                    }));
                })),
            )
            .child(
                view()
                    .gap_2()
                    .child(
                        caption("Limit price".into()).child(
                            text(format!("use last {}", p(props.last)))
                                .on_press(cx.listener(|this: &mut Self, _, cx| {
                                    this.limit = cx.props().last;
                                    cx.notify();
                                }))
                                .role(AccessibilityRole::Button)
                                .hit_slop(EdgeInsets::all(10.0))
                                .text_size(px(13.0))
                                .text_color(colors::ACCENT)
                                .tabular_nums(),
                        ),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(10.0))
                            .child(step(false, "Lower the price", nudge(-1.0)))
                            .child(
                                view()
                                    .flex_1()
                                    .h(px(44.0))
                                    .rounded(px(radius::CONTROL))
                                    .items_center()
                                    .justify_center()
                                    .bg(colors::INSET)
                                    .border_1()
                                    .border_color(colors::BORDER)
                                    .child(
                                        text(p(self.limit))
                                            .text_size(px(20.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT)
                                            .tabular_nums(),
                                    ),
                            )
                            .child(step(true, "Raise the price", nudge(1.0))),
                    ),
            )
            .child(
                view()
                    .gap_2()
                    .child(
                        caption("Quantity".into()).child(
                            text(format!("{} sh", grouped(self.quantity, 0)))
                                .text_size(px(13.0))
                                .text_color(colors::FAINT)
                                .tabular_nums(),
                        ),
                    )
                    .child(view().flex_row().gap_2().children(chips)),
            )
            .child(
                view()
                    .gap(px(6.0))
                    .pt(px(2.0))
                    .child(
                        caption(format!(
                            "Estimated {}",
                            if buy { "cost" } else { "proceeds" }
                        ))
                        .child(
                            text(grouped(cost, 2))
                                .text_size(px(22.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT)
                                .tabular_nums(),
                        ),
                    )
                    .child(
                        view()
                            .h(px(6.0))
                            .rounded(px(3.0))
                            .bg(colors::RAISED)
                            .overflow_hidden()
                            .child(
                                view()
                                    .w(relative((cost / POWER).min(1.0) as f32))
                                    .h(px(6.0))
                                    .rounded(px(3.0))
                                    .bg(if cost > POWER { colors::DANGER } else { tone }),
                            ),
                    )
                    .child(
                        text(format!(
                            "{:.1}% of {} buying power",
                            cost / POWER * 100.0,
                            grouped(POWER, 0)
                        ))
                        .text_size(px(12.0))
                        .text_color(colors::FAINT)
                        .tabular_nums(),
                    ),
            )
            .child(review)
    }
}
