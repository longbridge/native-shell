//! Mobile UI: `nsh/ui/mobile`'s Rust components laying out the demo's
//! content (as the TSX gallery's `demos/MobileUi.tsx`): a `ListView`
//! recycles rows the demo renders, under a header of cards with a text
//! field, a sheet, a select list, groups, steppers and their methods; a
//! `Group` wraps each child in a row. The demo writes the content; the
//! components only place it. Their colors come from the app's theme, so
//! the text the demo puts inside them keeps the color they give it.

use std::rc::Rc;

use nsh_core::ui::prelude::*;
use nsh_ui_mobile::{
    Button, ButtonEvent, ButtonProps, ButtonVariant, Group, GroupProps, ListView, ListViewProps,
    ListViewVariant, SelectList, SelectListEvent, SelectListProps, Sheet, SheetEvent, SheetProps,
    Stepper, StepperEvent, StepperProps, TextField, TextFieldEvent, TextFieldProps,
};

use crate::ui::{
    Badge, Card, IconName, MONO, Row, ToggleEvent, Tone, appear, caption, colors, toggle,
};

/// A row of the list.
#[derive(Debug, Clone)]
struct Stock {
    id: usize,
    symbol: String,
    price: String,
    change: f64,
}

fn stocks() -> Vec<Stock> {
    (0..300)
        .map(|i| Stock {
            id: i,
            symbol: format!("SYM{i:03}"),
            price: format!(
                "{:.2}",
                50.0 + ((i * 37) % 400) as f64 + (i % 7) as f64 * 0.13
            ),
            change: (((i * 13) % 21) as f64 - 10.0) / 4.0,
        })
        .collect()
}

/// A mobile UI button.
fn mobile_button(
    title: impl Into<SharedString>,
    variant: ButtonVariant,
) -> ComponentElement<Button> {
    Button::element(ButtonProps {
        title: title.into(),
        variant,
        ..ButtonProps::default()
    })
}

/// A row of two things at its ends.
fn between() -> View {
    view().flex_row().items_center().justify_between()
}

/// The list's header: the components, on cards.
#[derive(Debug)]
struct Header {
    cities: Rc<Vec<SharedString>>,
    taps: u32,
    /// The controlled stepper's value.
    quantity: f64,
    /// What asking the other stepper answered.
    asked: String,
    name: String,
    sheet: bool,
    city: Option<u32>,
    /// The stepper with a value of its own, asked for it (TS: its ref).
    own: Option<ComponentRef<Stepper>>,
    ask: Option<Task<()>>,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            cities: Rc::default(),
            taps: 0,
            quantity: 2.0,
            asked: String::new(),
            name: "Ada".to_owned(),
            sheet: false,
            city: None,
            own: None,
            ask: None,
        }
    }
}

/// The header's props.
#[derive(Debug, Clone, PartialEq, Props)]
struct HeaderProps {
    /// The page's rows are inset (the page keeps it: TS keeps it at
    /// module level).
    #[prop(default)]
    inset: bool,
}

/// What the header asks of the page.
#[derive(Debug, Clone, PartialEq, Event)]
enum HeaderEvent {
    /// Show plain or inset rows.
    Inset(bool),
}

impl Component for Header {
    type Props = HeaderProps;
    type Event = HeaderEvent;

    fn setup(&mut self, _: &HeaderProps, cx: &mut Context<'_, Self>) {
        self.cities = Rc::new((1..=500).map(|i| format!("City {i}").into()).collect());
        self.own = Some(cx.component_ref());
    }

    fn render(&self, props: &HeaderProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (chosen, chosen_tone) = match self.city {
            Some(i) => (self.cities[i as usize].clone(), Tone::Accent),
            None => ("none".into(), Tone::Neutral),
        };
        view()
            .p_4()
            .gap_4()
            .bg(colors::BACKGROUND)
            .child(
                appear(0).child(
                    view()
                        .gap(px(6.0))
                        .px_1()
                        .child(
                            view()
                                .flex_row()
                                .gap_2()
                                .child(Badge::new("Rust").tone(Tone::Accent))
                                .child(Badge::new("nsh-ui-mobile")),
                        )
                        .child(
                            text(
                                "Components written in Rust, running in the core: the app hands \
                                 them JSX and data, they draw the boxes and keep their own state.",
                            )
                            .text_size(px(15.0))
                            .line_height(px(21.0))
                            .text_color(colors::MUTED),
                        ),
                ),
            )
            .child(
                appear(1).child(
                    Card::new()
                        .icon(IconName::Form)
                        .title("TextField")
                        .subtitle("A label, a clear button, controlled or not")
                        .child(
                            TextField::element(TextFieldProps {
                                label: Some("Name".into()),
                                placeholder: Some("Your name".into()),
                                value: Some(self.name.clone().into()),
                                ..TextFieldProps::default()
                            })
                            .on_event(cx.listener(
                                |this: &mut Self, e: &TextFieldEvent, cx| {
                                    if let TextFieldEvent::Change(v) = e {
                                        this.name = v.to_string();
                                        cx.notify();
                                    }
                                },
                            )),
                        )
                        .child(TextField::element(TextFieldProps {
                            label: Some("Password".into()),
                            placeholder: Some("At least 8 characters".into()),
                            secure: true,
                            ..TextFieldProps::default()
                        }))
                        .child(caption(
                            format!(
                                "Hello, {}",
                                if self.name.is_empty() {
                                    "nobody"
                                } else {
                                    &self.name
                                }
                            ),
                            Some("× clears and focuses"),
                        )),
                ),
            )
            .child(
                appear(2).child(
                    Card::new()
                        .icon(IconName::Components)
                        .title("Sheet")
                        .subtitle("Follows the finger natively, springs back")
                        .child(
                            mobile_button("Open the sheet", ButtonVariant::Primary).on_event(
                                cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                                    this.sheet = true;
                                    cx.notify();
                                }),
                            ),
                        )
                        .child(caption("Drag, tap outside or press back", Some("onClose"))),
                ),
            )
            .child(
                Sheet::element(SheetProps {
                    open: self.sheet,
                    ..SheetProps::default()
                })
                .on_event(cx.listener(|this: &mut Self, _: &SheetEvent, cx| {
                    this.sheet = false;
                    cx.notify();
                }))
                .child(
                    text("A sheet in Rust")
                        .text_size(px(17.0))
                        .font_weight(FontWeight::SEMIBOLD),
                )
                .child(
                    text(
                        "It follows the finger natively and springs back; drag it down, tap \
                         outside or press back to close.",
                    )
                    .opacity(0.7),
                )
                .child(
                    mobile_button("Done", ButtonVariant::Primary).on_event(cx.listener(
                        |this: &mut Self, _: &ButtonEvent, cx| {
                            this.sheet = false;
                            cx.notify();
                        },
                    )),
                ),
            )
            .child(
                appear(3).child(
                    Card::new()
                        .icon(IconName::Lists)
                        .title("SelectList")
                        .subtitle("500 rows of Rust data")
                        .right(Badge::new(chosen).tone(chosen_tone))
                        .flush()
                        .child(
                            view()
                                .h(px(240.0))
                                .rounded(px(15.0))
                                .border_curve(BorderCurve::Continuous)
                                .overflow_hidden()
                                .child(
                                    SelectList::element(SelectListProps {
                                        options: self.cities.to_vec(),
                                        value: self.city,
                                    })
                                    .on_event(cx.listener(
                                        |this: &mut Self, e: &SelectListEvent, cx| {
                                            let SelectListEvent::Select(i) = *e;
                                            this.city = Some(i);
                                            cx.notify();
                                        },
                                    )),
                                ),
                        ),
                ),
            )
            .child(
                appear(4).child(
                    Card::new()
                        .icon(IconName::Mobile)
                        .title("Group")
                        .subtitle("Each JSX child becomes a row")
                        .flush()
                        .child(
                            Group::element(GroupProps::default())
                                .fill("header", text("Settings"))
                                .fill(
                                    "footer",
                                    text(
                                        "The rows are JSX; the group adds the box and the \
                                         dividers.",
                                    ),
                                )
                                .child(text("Notifications"))
                                .child(text("Privacy"))
                                .child(
                                    view()
                                        .flex_row()
                                        .justify_between()
                                        .child(text("Taps"))
                                        .child(text(self.taps.to_string()).tabular_nums()),
                                ),
                        ),
                ),
            )
            .child(
                appear(5).child(
                    Card::new()
                        .icon(IconName::Plus)
                        .title("Stepper")
                        .subtitle("State and timers in Rust: hold a button")
                        .flush()
                        .child(
                            Group::element(GroupProps::default())
                                .child(
                                    between().child(text("Own value")).child(
                                        Stepper::element(StepperProps {
                                            default_value: 5.0,
                                            min: 0.0,
                                            max: 50.0,
                                            ..StepperProps::default()
                                        })
                                        .component_ref(self.own.as_ref().expect("set up")),
                                    ),
                                )
                                .child(
                                    between()
                                        .child(
                                            text(format!("Controlled: {}", self.quantity))
                                                .tabular_nums(),
                                        )
                                        .child(
                                            Stepper::element(StepperProps {
                                                value: Some(self.quantity),
                                                min: 0.0,
                                                max: 10.0,
                                                step: 0.5,
                                                ..StepperProps::default()
                                            })
                                            .on_event(cx.listener(
                                                |this: &mut Self, e: &StepperEvent, cx| {
                                                    let StepperEvent::Change(v) = *e;
                                                    this.quantity = v;
                                                    cx.notify();
                                                },
                                            )),
                                        ),
                                ),
                        ),
                ),
            )
            .child(
                appear(6).child(
                    Card::new()
                        .icon(IconName::Spark)
                        .title("Methods")
                        .subtitle("A Rust component's ref, called from Rust")
                        .child(
                            mobile_button(
                                if self.asked.is_empty() {
                                    "Ask the stepper, then reset it".to_owned()
                                } else {
                                    self.asked.clone()
                                },
                                ButtonVariant::Secondary,
                            )
                            .on_event(cx.listener(
                                |this: &mut Self, _: &ButtonEvent, cx| {
                                    // Methods of a Rust component, called from Rust.
                                    let own = this.own.clone().expect("set up");
                                    let value = own.update(|stepper, cx| {
                                        let value = stepper.value();
                                        stepper.set_value(5.0, cx);
                                        value
                                    });
                                    this.ask = Some(cx.spawn(|this, _| async move {
                                        if let Some(value) = value.await {
                                            let _ = this
                                                .update(move |this, cx| {
                                                    this.asked =
                                                        format!("It was {value}; reset to 5");
                                                    cx.notify();
                                                })
                                                .await;
                                        }
                                    }));
                                },
                            )),
                        )
                        .child(
                            mobile_button(format!("Tap ({})", self.taps), ButtonVariant::Secondary)
                                .on_event(cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                                    this.taps += 1;
                                    cx.notify();
                                })),
                        )
                        .child(
                            view().child(
                                Row::new(true)
                                    .child(
                                        view()
                                            .flex_1()
                                            .gap(px(2.0))
                                            .child(
                                                text("Inset rows")
                                                    .text_size(px(15.0))
                                                    .text_color(colors::TEXT),
                                            )
                                            .child(
                                                text(format!(
                                                    "variant=\"{}\"",
                                                    if props.inset { "inset" } else { "plain" }
                                                ))
                                                .text_size(px(12.0))
                                                .text_color(colors::FAINT)
                                                .font_family(MONO),
                                            ),
                                    )
                                    .child(toggle(props.inset, "Inset rows").on_event(
                                        cx.listener(|_: &mut Self, e: &ToggleEvent, cx| {
                                            let ToggleEvent::Change(on) = *e;
                                            cx.emit(HeaderEvent::Inset(on));
                                        }),
                                    )),
                            ),
                        ),
                ),
            )
            .child(
                text("Below: a ListView of 300 rows the app renders, recycled natively")
                    .text_size(px(13.0))
                    .text_color(colors::MUTED)
                    .px_1(),
            )
    }
}

/// The demo's page: the stocks in a `ListView`, under the header.
#[derive(Debug, Default)]
pub(crate) struct MobileUi {
    stocks: Rc<Vec<Stock>>,
    inset: bool,
}

impl Component for MobileUi {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.stocks = Rc::new(stocks());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let stocks = self.stocks.clone();
        // White on the fills in light appearance, the inset's black in dark.
        let on_fill = AdaptiveColor::new(rgb(0xffffff), colors::INSET.dark);
        let rows = virtual_list(stocks.len(), move |i, _| {
            let stock = &stocks[i];
            let up = stock.change >= 0.0;
            view()
                .flex_row()
                .items_center()
                .gap_2()
                .child(
                    view()
                        .flex_1()
                        .gap(px(1.0))
                        .child(
                            text(stock.symbol.clone())
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD),
                        )
                        .child(
                            text(format!("No. {}", stock.id + 1))
                                .text_size(px(12.0))
                                .opacity(0.55)
                                .tabular_nums(),
                        ),
                )
                .child(
                    text(stock.price.clone())
                        .w(px(80.0))
                        .text_right()
                        .text_size(px(15.0))
                        .tabular_nums(),
                )
                .child(
                    view()
                        .w(px(72.0))
                        .h(px(28.0))
                        .rounded(px(8.0))
                        .items_center()
                        .justify_center()
                        .bg(if up { colors::UP } else { colors::DOWN })
                        .child(
                            text(format!("{}{:.2}%", if up { "+" } else { "" }, stock.change))
                                .text_size(px(13.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(on_fill)
                                .tabular_nums(),
                        ),
                )
        })
        .keys(self.stocks.iter().map(|s| s.id))
        .header(
            Header::element(HeaderProps { inset: self.inset }).on_event(cx.listener(
                |this: &mut Self, e: &HeaderEvent, cx| {
                    let HeaderEvent::Inset(on) = *e;
                    this.inset = on;
                    cx.notify();
                },
            )),
        );
        // A component's root is a view (its host node).
        view().flex_1().child(
            ListView::element(ListViewProps {
                variant: if self.inset {
                    ListViewVariant::Inset
                } else {
                    ListViewVariant::Plain
                },
                ..ListViewProps::default()
            })
            .items(rows),
        )
    }
}
