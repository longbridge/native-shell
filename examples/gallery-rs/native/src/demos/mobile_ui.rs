//! Mobile UI: `nsh/ui/mobile`'s Rust components laying out the demo's
//! content (as the TSX gallery's `demos/MobileUi.tsx`): a `ListView`
//! recycles rows the demo renders, under a header of text fields, a sheet,
//! a select list, groups, steppers and buttons; a `Group` wraps each child
//! in a row. The demo writes the content; the components only place it.

use std::rc::Rc;

use nsh_core::ui::{AnyElement, prelude::*};
use nsh_ui_mobile::{
    Button, ButtonEvent, ButtonProps, ButtonVariant, Group, GroupProps, ListView, ListViewProps,
    ListViewVariant, SelectList, SelectListEvent, SelectListProps, Sheet, SheetEvent, SheetProps,
    Stepper, StepperEvent, StepperProps, TextField, TextFieldEvent, TextFieldProps,
};

/// The muted gray of the header's notes.
fn note_color() -> Color {
    rgb(0x57606a)
}

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

/// A text of the header.
fn note(content: impl Into<SharedString>) -> AnyElement {
    text(content.into())
        .text_size(px(13.0))
        .text_color(note_color())
        .into_element()
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

/// The list's header: the components.
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
    /// Switch between plain and inset rows.
    ToggleInset,
}

impl Component for Header {
    type Props = HeaderProps;
    type Event = HeaderEvent;

    fn setup(&mut self, _: &HeaderProps, cx: &mut Context<'_, Self>) {
        self.cities = Rc::new((1..=500).map(|i| format!("City {i}").into()).collect());
        self.own = Some(cx.component_ref());
    }

    fn render(&self, props: &HeaderProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let chosen = match self.city {
            Some(i) => self.cities[i as usize].to_string(),
            None => "none".to_owned(),
        };
        let between = || view().flex_row().items_center().justify_between();
        view()
            .p_4()
            .gap_4()
            .child(
                TextField::element(TextFieldProps {
                    label: Some("Name (controlled; × clears and focuses)".into()),
                    placeholder: Some("Your name".into()),
                    value: Some(self.name.clone().into()),
                    ..TextFieldProps::default()
                })
                .on_event(cx.listener(|this: &mut Self, e: &TextFieldEvent, cx| {
                    if let TextFieldEvent::Change(v) = e {
                        this.name = v.to_string();
                        cx.notify();
                    }
                })),
            )
            .child(TextField::element(TextFieldProps {
                label: Some("Password".into()),
                placeholder: Some("At least 8 characters".into()),
                secure: true,
                ..TextFieldProps::default()
            }))
            .child(note(format!(
                "Hello, {}",
                if self.name.is_empty() {
                    "nobody"
                } else {
                    &self.name
                }
            )))
            .child(
                mobile_button("Open the sheet (drag it down)", ButtonVariant::Primary).on_event(
                    cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                        this.sheet = true;
                        cx.notify();
                    }),
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
                    text("It follows the finger natively and springs back; drag it down, tap outside or press back to close.")
                        .text_color(note_color()),
                )
                .child(mobile_button("Done", ButtonVariant::Primary).on_event(cx.listener(
                    |this: &mut Self, _: &ButtonEvent, cx| {
                        this.sheet = false;
                        cx.notify();
                    },
                ))),
            )
            .child(note(format!(
                "SelectList: 500 rows of Rust data, chosen: {chosen}"
            )))
            .child(
                view().h(px(240.0)).rounded(px(12.0)).overflow_hidden().child(
                    SelectList::element(SelectListProps {
                        options: self.cities.to_vec(),
                        value: self.city,
                    })
                    .on_event(cx.listener(|this: &mut Self, e: &SelectListEvent, cx| {
                        let SelectListEvent::Select(i) = *e;
                        this.city = Some(i);
                        cx.notify();
                    })),
                ),
            )
            .child(
                Group::element(GroupProps::default())
                    .fill("header", text("Group: each child is a row"))
                    .fill(
                        "footer",
                        text("The rows are the demo's elements; the group adds the box and the dividers."),
                    )
                    .child(text("Notifications"))
                    .child(text("Privacy"))
                    .child(
                        view()
                            .flex_row()
                            .justify_between()
                            .child(text("Taps"))
                            .child(text(self.taps.to_string())),
                    ),
            )
            .child(
                Group::element(GroupProps::default())
                    .fill(
                        "header",
                        text("Stepper: state and timers in Rust (hold a button)"),
                    )
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
                            .child(text(format!("Controlled: {}", self.quantity)))
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
            )
            .child(
                mobile_button(
                    if self.asked.is_empty() {
                        "Ask the stepper, then reset it".to_owned()
                    } else {
                        self.asked.clone()
                    },
                    ButtonVariant::Secondary,
                )
                .on_event(cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
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
                                    this.asked = format!("It was {value}; reset to 5");
                                    cx.notify();
                                })
                                .await;
                        }
                    }));
                })),
            )
            .child(
                mobile_button("Tap", ButtonVariant::Secondary).on_event(cx.listener(
                    |this: &mut Self, _: &ButtonEvent, cx| {
                        this.taps += 1;
                        cx.notify();
                    },
                )),
            )
            .child(
                mobile_button(
                    if props.inset { "Plain rows" } else { "Inset rows" },
                    ButtonVariant::Plain,
                )
                .on_event(cx.listener(|_: &mut Self, _: &ButtonEvent, cx| {
                    cx.emit(HeaderEvent::ToggleInset);
                })),
            )
            .child(note(
                "ListView below: 300 rows the demo renders, recycled natively.",
            ))
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
        let rows = virtual_list(stocks.len(), move |i, _| {
            let stock = &stocks[i];
            let up = stock.change >= 0.0;
            view()
                .flex_row()
                .items_center()
                .child(
                    text(stock.symbol.clone())
                        .flex_1()
                        .font_weight(FontWeight::SEMIBOLD),
                )
                .child(text(stock.price.clone()).w(px(80.0)).text_right())
                .child(
                    text(format!("{}{:.2}%", if up { "+" } else { "" }, stock.change))
                        .w(px(64.0))
                        .text_right()
                        .text_color(if up { rgb(0x1a7f37) } else { rgb(0xcf222e) }),
                )
        })
        .keys(self.stocks.iter().map(|s| s.id))
        .header(
            Header::element(HeaderProps { inset: self.inset }).on_event(cx.listener(
                |this: &mut Self, _: &HeaderEvent, cx| {
                    this.inset = !this.inset;
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
