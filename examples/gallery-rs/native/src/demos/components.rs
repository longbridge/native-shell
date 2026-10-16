//! Components: the mobile UI's buttons and cards, native press feedback, a
//! sheet, layout animation, an input's accessory bar, the keyboard avoiding
//! view, modal options and the environment (as the TSX gallery's
//! `demos/Components.tsx`).

use std::hash::{BuildHasher, RandomState};

use nsh_core::{
    JsonValue,
    ui::{
        AnyElement, Idiom, KeyboardDismissMode, LayoutAnimation, Orientation, Os, Style, Text,
        View, prelude::*,
    },
};
use nsh_ui_mobile::{
    AvoidBehavior, Button, ButtonEvent, ButtonProps, ButtonVariant, Card, CardProps, CardVariant,
    KeyboardAvoidingView, KeyboardAvoidingViewProps, SafeAreaView, SafeAreaViewProps,
};

use crate::{
    nav,
    ui::{button, colors, page, section},
};

/// A row of controls.
fn row() -> View {
    view().flex_row().items_center().gap_3()
}

/// One device pixel, in points (TS: `1 / PixelRatio.get()`).
fn hairline<C: Component>(cx: &RenderCx<'_, C>) -> f32 {
    1.0 / cx.pixel_ratio()
}

/// A bordered card taking presses.
fn card(hairline: f32) -> View {
    view()
        .bg(rgb(0xffffff))
        .rounded(px(10.0))
        .p_3()
        .border(hairline)
        .border_color(colors::border())
}

/// A label.
fn label(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_size(px(15.0))
        .text_color(colors::text())
}

/// A muted note.
fn muted(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_size(px(13.0))
        .text_color(colors::muted())
}

/// The components of `nsh/ui/mobile` (Rust, `nsh-ui-mobile`) in a Rust
/// render.
#[derive(Debug, Default)]
struct MobileComponents {
    presses: u32,
}

impl Component for MobileComponents {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let pressed = || {
            cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                this.presses += 1;
                cx.notify();
            })
        };
        let mobile_button = |title: String, variant, disabled| {
            Button::element(ButtonProps {
                title: title.into(),
                variant,
                disabled,
                ..ButtonProps::default()
            })
        };
        section(
            "nsh/ui/mobile",
            [
                mobile_button(
                    format!("Primary ({})", self.presses),
                    ButtonVariant::Primary,
                    false,
                )
                .on_event(pressed())
                .into_element(),
                mobile_button("Secondary".into(), ButtonVariant::Secondary, false)
                    .on_event(pressed())
                    .into_element(),
                mobile_button("Danger".into(), ButtonVariant::Danger, false)
                    .on_event(pressed())
                    .into_element(),
                mobile_button("Plain".into(), ButtonVariant::Plain, false)
                    .on_event(pressed())
                    .into_element(),
                mobile_button("Disabled".into(), ButtonVariant::Primary, true)
                    .on_event(pressed())
                    .into_element(),
                Card::element(CardProps::default())
                    .fill("header", text("Card with slots"))
                    .fill(
                        "footer",
                        mobile_button("Reset".into(), ButtonVariant::Plain, false).on_event(
                            cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                                this.presses = 0;
                                cx.notify();
                            }),
                        ),
                    )
                    .child(text(
                        "The header, this body and the footer are Rust elements in a Rust component's slots and children.",
                    ))
                    .into_element(),
                Card::element(CardProps {
                    variant: CardVariant::Outlined,
                    ..CardProps::default()
                })
                .child(text("An outlined card, children only."))
                .into_element(),
            ],
        )
    }
}

/// Press feedback is native: opacity, an underlay, styles; `disabled`
/// takes no presses.
#[derive(Debug, Default)]
struct PressFeedback {
    last: &'static str,
    disabled: bool,
}

impl Component for PressFeedback {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.last = "nothing yet";
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let last = |what: &'static str| {
            cx.listener(move |this: &mut Self, _, cx| {
                this.last = what;
                cx.notify();
            })
        };
        // The underlay's default children opacity (`underlayOpacity`).
        let underlay_opacity = 0.85;
        let card = || card(hairline(cx));
        section(
            "Press feedback",
            [
                card()
                    .active_opacity(0.4)
                    .on_press(last("activeOpacity"))
                    .child(label("activeOpacity: fades while pressed"))
                    .into_element(),
                card()
                    .underlay(rgb(0xdbeafe), underlay_opacity)
                    .on_press(last("underlayColor"))
                    .child(label("underlayColor: the background while pressed"))
                    .into_element(),
                card()
                    .disabled(self.disabled)
                    .underlay(rgb(0xfee2e2), underlay_opacity)
                    .when(self.disabled, |v| v.opacity(0.4))
                    .on_press(last("disabled view"))
                    .child(label(if self.disabled {
                        "disabled: takes no presses"
                    } else {
                        "enabled"
                    }))
                    .into_element(),
                row()
                    .child(
                        button(
                            if self.disabled { "Enable" } else { "Disable" },
                            colors::gray(),
                        )
                        .on_press(cx.listener(|this: &mut Self, _, cx| {
                            this.disabled = !this.disabled;
                            cx.notify();
                        })),
                    )
                    .child(muted(format!("Last pressed: {}", self.last)))
                    .into_element(),
            ],
        )
    }
}

/// A pressable that opens a sheet.
#[derive(Debug, Default)]
struct PressableSheet {
    sheet: bool,
}

impl Component for PressableSheet {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let open = |sheet: bool| {
            cx.listener(move |this: &mut Self, _, cx| {
                this.sheet = sheet;
                cx.notify();
            })
        };
        let modal_view = self.sheet.then(|| {
            modal()
                .presentation(ModalPresentation::PageSheet)
                .animation(ModalAnimation::Slide)
                .on_request_close(open(false))
                .child(
                    SafeAreaView::element(SafeAreaViewProps::default())
                        .flex_1()
                        .bg(colors::surface())
                        .p_5()
                        .gap_4()
                        .child(
                            text("A sheet")
                                .text_size(px(24.0))
                                .font_weight(FontWeight::BOLD),
                        )
                        .child(
                            text("Swipe down (iOS) or press back (Android) to close.")
                                .text_color(colors::muted()),
                        )
                        .child(button("Close", colors::accent()).on_press(open(false))),
                )
        });
        // TS adds `transition`: the press styles animate there. Typed styles
        // have no transitions, so they switch at once.
        view()
            .child(section(
                "Sheet",
                [view()
                    .rounded(px(10.0))
                    .bg(rgb(0x8250df))
                    .p(px(14.0))
                    .active(|s| s.scale(0.97).bg(rgb(0xbf3989)))
                    .role(AccessibilityRole::Button)
                    .on_press(open(true))
                    .child(
                        text("Open a sheet")
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::SEMIBOLD),
                    )
                    .into_element()],
            ))
            .children(modal_view)
    }
}

/// Shuffles `items` in place.
fn shuffle(items: &mut [u32]) {
    // Fresh random keys each time, from the standard library.
    let random = RandomState::new();
    for i in (1..items.len()).rev() {
        let j = (random.hash_one(i) % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// Views added, moved and removed with a layout animation.
#[derive(Debug, Default)]
struct LayoutAnimations {
    items: Vec<u32>,
    next: u32,
}

impl LayoutAnimations {
    /// Changes the items, animating the layout change.
    fn animate(&mut self, cx: &mut Context<'_, Self>, change: impl FnOnce(&mut Self)) {
        cx.animate_layout(LayoutAnimation::SPRING);
        change(self);
        cx.notify();
    }
}

impl Component for LayoutAnimations {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.items = vec![1, 2, 3];
        self.next = 4;
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let boxes = self.items.iter().map(|n| {
            view()
                .key(n.to_string())
                .size(px(56.0))
                .rounded(px(12.0))
                .bg(rgb(0xddf4ff))
                .items_center()
                .justify_center()
                .child(text(n.to_string()).font_weight(FontWeight::SEMIBOLD))
        });
        section(
            "Layout animation",
            [
                row()
                    .child(button("Add", colors::accent()).on_press(cx.listener(
                        |this: &mut Self, _, cx| {
                            this.animate(cx, |this| {
                                this.items.insert(0, this.next);
                                this.next += 1;
                            });
                        },
                    )))
                    .child(button("Shuffle", colors::purple()).on_press(cx.listener(
                        |this: &mut Self, _, cx| this.animate(cx, |this| shuffle(&mut this.items)),
                    )))
                    .child(button("Remove", rgb(0xcf222e)).on_press(cx.listener(
                        |this: &mut Self, _, cx| {
                            this.animate(cx, |this| {
                                if !this.items.is_empty() {
                                    this.items.remove(0);
                                }
                            });
                        },
                    )))
                    .into_element(),
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .children(boxes)
                    .into_element(),
            ],
        )
    }
}

/// An input with a bar over the keyboard (iOS's input accessory).
#[derive(Debug, Default)]
struct Accessory {
    text: String,
    input: Option<ElementRef>,
}

impl Component for Accessory {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.input = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let input_ref = self.input.as_ref().expect("set up");
        let mut content = vec![
            input()
                .element_ref(input_ref)
                .value(self.text.clone())
                .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                    this.text = v.to_owned();
                    cx.notify();
                }))
                .placeholder("Focus me (iOS shows a bar over the keyboard)")
                .input_accessory("counter")
                .border_1()
                .border_color(colors::border())
                .rounded(px(8.0))
                .px(px(10.0))
                .py(px(8.0))
                .text_size(px(16.0))
                .into_element(),
        ];
        if cx.os() == Os::Android {
            content.push(muted("Android has no input accessories.").into_element());
        }
        content.push(
            input_accessory_view("counter")
                .bg(rgb(0xf6f8fa))
                .child(
                    view()
                        .flex_row()
                        .items_center()
                        .px_3()
                        .py_2()
                        .border_t(hairline(cx))
                        .border_t_color(colors::border())
                        .child(
                            muted(format!("{} characters", self.text.encode_utf16().count()))
                                .flex_1(),
                        )
                        .child(button("Clear", colors::gray()).on_press(cx.listener(
                            |this: &mut Self, _, cx| {
                                this.text.clear();
                                cx.notify();
                            },
                        )))
                        .child(view().w(px(8.0)))
                        .child(button("Done", colors::accent()).on_press(cx.listener(
                            |this: &mut Self, _, _| {
                                if let Some(input) = &this.input {
                                    input.blur();
                                }
                            },
                        ))),
                )
                .into_element(),
        );
        section("InputAccessoryView", content)
    }
}

/// The behaviors of the keyboard avoiding view, with their labels.
const BEHAVIORS: [(AvoidBehavior, &str); 3] = [
    (AvoidBehavior::Padding, "padding"),
    (AvoidBehavior::Height, "height"),
    (AvoidBehavior::Position, "position"),
];

/// The content of the keyboard avoiding view's screen.
#[derive(Debug, Default)]
struct AvoidingPage {
    behavior: AvoidBehavior,
}

impl Component for AvoidingPage {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let current = self.behavior;
        let name = BEHAVIORS
            .iter()
            .find(|(b, _)| *b == current)
            .map_or("padding", |(_, n)| n);
        let choices = BEHAVIORS.map(|(behavior, label)| {
            button(
                label,
                if behavior == current {
                    colors::accent()
                } else {
                    colors::gray()
                },
            )
            .on_press(cx.listener(move |this: &mut Self, _, cx| {
                this.behavior = behavior;
                cx.notify();
            }))
        });
        // The inner view of `position` fills the avoiding view.
        let fill = Style::from_json(JsonValue::from_iter([("flex", 1)])).unwrap_or_default();
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps {
                behavior: current,
                content_container_style: fill,
                ..KeyboardAvoidingViewProps::default()
            })
            .flex_1()
            .bg(rgb(0xddf4ff))
            .child(
                view()
                    .flex_1()
                    .p_4()
                    .gap_3()
                    .child(label(format!("behavior=\"{name}\"")))
                    .child(row().children(choices))
                    .child(muted(
                        "padding: the bottom pads (the blue background stays under the keyboard); height: the view shrinks; position: the content moves up.",
                    ))
                    .child(view().flex_1())
                    .child(
                        input()
                            .placeholder("Type here")
                            .bg(rgb(0xffffff))
                            .rounded(px(8.0))
                            .px(px(10.0))
                            .py(px(10.0))
                            .text_size(px(16.0)),
                    ),
            ),
        )
    }
}

/// A modal's platform options.
#[derive(Debug, Default)]
struct ModalOptions {
    visible: bool,
    translucent: bool,
}

impl Component for ModalOptions {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let show = |visible: bool| {
            cx.listener(move |this: &mut Self, _, cx| {
                this.visible = visible;
                cx.notify();
            })
        };
        let modal_view = self.visible.then(|| {
            modal()
                .animation(ModalAnimation::Slide)
                .status_bar_translucent(self.translucent)
                .supported_orientations([Orientation::Portrait, Orientation::Landscape])
                .on_request_close(show(false))
                .child(
                    view().flex_1().bg(rgb(0xfff8c5)).child(
                        SafeAreaView::element(SafeAreaViewProps::default())
                            .flex_1()
                            .p_4()
                            .gap_3()
                            .child(
                                text("A modal")
                                    .text_size(px(22.0))
                                    .font_weight(FontWeight::BOLD),
                            )
                            .child(muted(format!(
                                "The yellow background starts {} the status bar on Android.",
                                if self.translucent { "under" } else { "below" }
                            )))
                            .child(button("Close", colors::accent()).on_press(show(false))),
                    ),
                )
        });
        view()
            .child(section(
                "Modal options",
                [
                    row()
                        .child(button("Open", colors::accent()).on_press(show(true)))
                        .child(
                            button(
                                format!("statusBarTranslucent: {}", self.translucent),
                                colors::gray(),
                            )
                            .on_press(cx.listener(
                                |this: &mut Self, _, cx| {
                                    this.translucent = !this.translucent;
                                    cx.notify();
                                },
                            )),
                        )
                        .into_element(),
                    muted(
                        "Android: translucent content reaches under the status bar. iOS: this modal allows portrait and landscape.",
                    )
                    .into_element(),
                ],
            ))
            .children(modal_view)
    }
}

/// The platform, the pixel ratio and the screen reader.
#[derive(Debug, Default)]
struct PlatformInfo {
    /// Announcements made.
    announced: u32,
    /// The note the reader's focus moves to.
    note: Option<ElementRef>,
}

impl Component for PlatformInfo {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.note = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let os = cx.os();
        let (major, minor) = cx.os_version();
        // A string on iOS ("17.5"), the API level on Android (as TS's).
        let version = if os == Os::Android {
            major.to_string()
        } else {
            format!("{major}.{minor}")
        };
        let idiom = cx.idiom();
        let branch = match os {
            Os::Ios => "ios branch",
            Os::Android => "android branch",
            _ => "",
        };
        let rows = [
            ("Platform.OS", os.name().to_owned()),
            ("Platform.Version", version),
            (
                "isPad / isTV / isMacCatalyst",
                format!(
                    "{} / {} / {}",
                    idiom == Idiom::Pad,
                    idiom == Idiom::Tv,
                    idiom == Idiom::Mac
                ),
            ),
            ("Platform.select", branch.to_owned()),
            ("PixelRatio.get()", cx.pixel_ratio().to_string()),
            (
                "roundToNearestPixel(10.3)",
                cx.round_to_nearest_pixel(10.3).to_string(),
            ),
            ("screenReaderEnabled()", cx.screen_reader().to_string()),
            ("reduceMotion()", cx.reduce_motion().to_string()),
        ]
        .map(|(name, value)| {
            row()
                .justify_between()
                .child(muted(name))
                .child(label(value))
                .into_element()
        });
        let mut content: Vec<AnyElement> = rows.into();
        content.extend([
            view().h(px(hairline(cx))).bg(colors::text()).into_element(),
            muted("The line above is one device pixel tall.").into_element(),
            row()
                .child(button("Announce", colors::accent()).on_press(cx.listener(
                    |this: &mut Self, _, cx| {
                        this.announced += 1;
                        cx.announce_for_accessibility(format!("Announcement {}", this.announced));
                    },
                )))
                .child(
                    button("Focus the note", colors::gray()).on_press(cx.listener(
                        |this: &mut Self, _, _| {
                            if let Some(note) = &this.note {
                                note.set_accessibility_focus();
                            }
                        },
                    )),
                )
                .into_element(),
            muted(
                "With VoiceOver or TalkBack on, the buttons above speak and move the reader here.",
            )
            .element_ref(self.note.as_ref().expect("set up"))
            .into_element(),
        ]);
        section("Platform, PixelRatio, AccessibilityInfo", content)
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Components;

impl Component for Components {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let avoiding = section(
            "KeyboardAvoidingView",
            [button("padding / height / position", colors::accent())
                .on_press(cx.listener(|_: &mut Self, _, cx| {
                    nav::push(cx, "keyboard-avoiding", |_| {
                        screen()
                            .title("KeyboardAvoidingView")
                            .child(AvoidingPage::element(()))
                    });
                }))
                .into_element()],
        );
        view().flex_1().child(
            page([
                MobileComponents::element(()).into_element(),
                PressFeedback::element(()).into_element(),
                PressableSheet::element(()).into_element(),
                LayoutAnimations::element(()).into_element(),
                Accessory::element(()).into_element(),
                avoiding.into_element(),
                ModalOptions::element(()).into_element(),
                PlatformInfo::element(()).into_element(),
            ])
            .keyboard_dismiss_mode(KeyboardDismissMode::Interactive),
        )
    }
}
