//! Components: the mobile UI's buttons and cards, native press feedback, a
//! sheet opened by a pressable with press styles, layout animation, an
//! input's accessory bar, the keyboard avoiding view, modal options and
//! the environment (as the TSX gallery's `demos/Components.tsx`).

use std::hash::{BuildHasher, RandomState};

use nsh_core::{
    JsonValue, UNDERLAY_OPACITY,
    ui::{
        AnyElement, Idiom, Input, KeyboardDismissMode, LayoutAnimation, Orientation, Os, Style,
        prelude::*,
    },
};
use nsh_ui_mobile::{
    AvoidBehavior, Button, ButtonEvent, ButtonProps, ButtonVariant, Card, CardProps, CardVariant,
    KeyboardAvoidingView, KeyboardAvoidingViewProps, SafeAreaView, SafeAreaViewProps,
};

use crate::{
    nav,
    ui::{
        self, Badge, ButtonVariant as UiVariant, Finish, Finished, IconName, IconTile, Inset, MONO,
        PressProps, Row, Segment, Segmented, SegmentedEvent, SegmentedProps, SegmentedSize,
        ToggleEvent, Tone, appear, caption, colors, header, icon, inset_shadow, press_with, radius,
        rgb_a, segmented, tints, toggle,
    },
};

/// One device pixel, in points (TS: `1 / PixelRatio.get()`).
fn hairline<C: Component>(cx: &RenderCx<'_, C>) -> f32 {
    1.0 / cx.pixel_ratio()
}

/// A dark input, 48 pt tall, lit while `focused` (its owner keeps that
/// from `on_focus` / `on_blur`).
fn field<C: Component>(focused: bool, cx: &RenderCx<'_, C>) -> Input {
    input()
        .placeholder_color(cx.color(colors::FAINT))
        .selection_color(cx.color(colors::ACCENT))
        .h(px(48.0))
        .px(px(14.0))
        .text_size(px(15.0))
        .text_color(colors::TEXT)
        .bg(colors::INSET)
        .border_1()
        .border_color(if focused {
            colors::ACCENT
        } else {
            colors::BORDER
        })
        .rounded(px(radius::CONTROL))
        .finish(Finish::Inset)
}

/// A cell of a row of equal widths.
fn cell(content: impl IntoElement) -> View {
    view().flex_1().child(content)
}

/// Muted body text, 14 pt.
fn note(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_size(px(14.0))
        .line_height(px(20.0))
        .text_color(colors::MUTED)
}

/// A modal's or a sheet's big title over a badge.
fn title_block(badge: impl Into<SharedString>, title: &str) -> View {
    view()
        .gap_2()
        .child(
            view()
                .flex_row()
                .child(Badge::new(badge).tone(Tone::Accent)),
        )
        .child(
            text(title.to_owned())
                .text_size(px(28.0))
                .font_weight(FontWeight::BOLD)
                .letter_spacing(-0.5)
                .text_color(colors::TEXT),
        )
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
        let mobile_button = |title: &str, variant: ButtonVariant, disabled: bool| {
            Button::element(ButtonProps {
                title: title.to_owned().into(),
                variant: variant.into(),
                disabled: disabled.into(),
                ..ButtonProps::default()
            })
        };
        let presses = self.presses;
        ui::Card::new()
            .icon(IconName::Mobile)
            .title("nsh/ui/mobile")
            .subtitle("Rust components, the app's theme")
            .right(
                Badge::new(format!(
                    "{presses} {}",
                    if presses == 1 { "tap" } else { "taps" }
                ))
                .tone(if presses > 0 {
                    Tone::Accent
                } else {
                    Tone::Neutral
                }),
            )
            .child(mobile_button("Primary", ButtonVariant::Primary, false).on_event(pressed()))
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(cell(
                        mobile_button("Secondary", ButtonVariant::Secondary, false)
                            .on_event(pressed()),
                    ))
                    .child(cell(
                        mobile_button("Danger", ButtonVariant::Danger, false).on_event(pressed()),
                    )),
            )
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(cell(
                        mobile_button("Plain", ButtonVariant::Plain, false).on_event(pressed()),
                    ))
                    .child(cell(
                        mobile_button("Disabled", ButtonVariant::Secondary, true)
                            .on_event(pressed()),
                    )),
            )
            .child(
                Card::element(CardProps::default())
                    .fill("header", text("Card with slots"))
                    .fill(
                        "footer",
                        view()
                            .flex_row()
                            .gap_2()
                            .child(cell(
                                mobile_button("Reset", ButtonVariant::Secondary, false).on_event(
                                    cx.listener(|this: &mut Self, _: &ButtonEvent, cx| {
                                        this.presses = 0;
                                        cx.notify();
                                    }),
                                ),
                            ))
                            .child(cell(
                                mobile_button("Add one", ButtonVariant::Primary, false)
                                    .on_event(pressed()),
                            )),
                    )
                    .child(text(
                        "The header, this body and the footer are JSX in a Rust component's \
                         slots and children.",
                    )),
            )
            .child(
                Card::element(CardProps {
                    variant: CardVariant::Outlined.into(),
                    ..CardProps::default()
                })
                .child(text("An outlined card, children only.")),
            )
            .child(caption(
                "Colors come from the app's theme",
                Some("theme.json"),
            ))
    }
}

/// A tile to press: its title, the prop it shows, something on the right.
fn tile_content(title: impl Into<SharedString>, prop: &str, right: Option<AnyElement>) -> View {
    view()
        .min_h(px(60.0))
        .flex_row()
        .items_center()
        .gap_3()
        .px_3()
        .py_2()
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(title.into())
                        .text_size(px(15.0))
                        .text_color(colors::TEXT),
                )
                .child(
                    text(prop.to_owned())
                        .text_size(px(12.0))
                        .text_color(colors::FAINT)
                        .font_family(MONO),
                ),
        )
        .children(right)
}

/// A pressable tile: the raised highlight without its gradient, so an
/// underlay color shows.
fn tile() -> View {
    view()
        .rounded(px(radius::CONTROL))
        .border_curve(BorderCurve::Continuous)
        .border_1()
        .border_color(colors::BORDER)
        .bg(colors::SURFACE)
        .finish_shadow(Finish::Raised)
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
        self.last = "None yet";
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let last = |what: &'static str| {
            cx.listener(move |this: &mut Self, _, cx| {
                this.last = what;
                cx.notify();
            })
        };
        let disabled = self.disabled;
        ui::Card::new()
            .icon(IconName::Gestures)
            .title("Press feedback")
            .subtitle("Drawn natively while the finger is down")
            .child(
                tile()
                    .active_opacity(0.4)
                    .on_press(last("Fading tile"))
                    .child(tile_content(
                        "Fades while pressed",
                        "activeOpacity={0.4}",
                        None,
                    )),
            )
            .child(
                tile()
                    .underlay(cx.color(colors::RAISED), UNDERLAY_OPACITY)
                    .on_press(last("Underlay tile"))
                    .child(tile_content(
                        "Lights up while pressed",
                        "underlayColor",
                        None,
                    )),
            )
            .child(
                tile()
                    .disabled(disabled)
                    .underlay(cx.color(tints::DANGER), UNDERLAY_OPACITY)
                    .disabled_style(|s| s.opacity(0.4))
                    .on_press(last("Enabled tile"))
                    .child(tile_content(
                        if disabled {
                            "Takes no presses"
                        } else {
                            "Enabled"
                        },
                        "disabled + disabled:opacity-40",
                        Some(
                            Badge::new(if disabled { "Off" } else { "On" })
                                .tone(if disabled { Tone::Neutral } else { Tone::Up })
                                .into_element(),
                        ),
                    )),
            )
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(cell(caption("Last pressed", Some(self.last))))
                    .child(
                        toggle(disabled, "Disable the third tile").on_event(cx.listener(
                            |this: &mut Self, e: &ToggleEvent, cx| {
                                let ToggleEvent::Change(on) = *e;
                                this.disabled = on;
                                cx.notify();
                            },
                        )),
                    ),
            )
    }
}

/// A pressable with press styles (TS: Tailwind's `active:`) that opens a
/// sheet.
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
                        .bg(colors::BACKGROUND)
                        .child(
                            view()
                                .flex_1()
                                .p_4()
                                .gap_4()
                                .child(
                                    title_block("pageSheet", "A sheet").px_1().pt_2().child(
                                        text(
                                            "Swipe down (iOS) or press back (Android) to \
                                                 close.",
                                        )
                                        .text_size(px(15.0))
                                        .line_height(px(21.0))
                                        .text_color(colors::MUTED),
                                    ),
                                )
                                .child(view().flex_1())
                                .child(ui::Button::new("Close", open(false))),
                        ),
                )
        });
        let (accent_light, accent_dark) = Finish::Accent.shadows();
        ui::Card::new()
            .icon(IconName::Components)
            .title("Sheet")
            .subtitle("A Tailwind active: variant opens it")
            .child(
                view()
                    .role(AccessibilityRole::Button)
                    .on_press(open(true))
                    .adaptive_shadow(accent_light.iter().copied(), accent_dark.iter().copied())
                    .h_12()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .rounded(px(12.0))
                    .bg(rgb(0xc8f26d))
                    .transition()
                    .active(|s| s.scale(0.97).bg(rgb(0xa9d14f)))
                    .child(
                        text("Open a sheet")
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::ON_ACCENT),
                    ),
            )
            .child(caption(
                "Scales and darkens while pressed",
                Some("active:scale-[0.97]"),
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

/// The layout animation presets, with their labels.
const PRESETS: [(LayoutAnimation, &str); 3] = [
    (LayoutAnimation::SPRING, "Spring"),
    (LayoutAnimation::EASE_IN_EASE_OUT, "Ease"),
    (LayoutAnimation::LINEAR, "Linear"),
];

/// The tiles' tones, by number.
const TILE_TONES: [Tone; 5] = [Tone::Accent, Tone::Info, Tone::Warn, Tone::Up, Tone::Danger];

/// Views added, moved and removed with a layout animation.
#[derive(Debug, Default)]
struct LayoutAnimations {
    items: Vec<u32>,
    next: u32,
    preset: u32,
}

impl LayoutAnimations {
    /// Changes the items, animating the layout change.
    fn animate(&mut self, cx: &mut Context<'_, Self>, change: impl FnOnce(&mut Self)) {
        cx.animate_layout(PRESETS[self.preset as usize].0);
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
        let white = 0xffffff;
        let boxes = self.items.iter().map(|&n| {
            let tone = TILE_TONES[n as usize % TILE_TONES.len()];
            view()
                .key(n.to_string())
                .size(px(56.0))
                .rounded(px(14.0))
                .border_curve(BorderCurve::Continuous)
                .bg(tone.tint())
                .adaptive_shadow(
                    [inset_shadow(1.0, 0.0, rgb(white))],
                    [inset_shadow(1.0, 0.0, rgb_a(white, 0.06))],
                )
                .items_center()
                .justify_center()
                .child(
                    text(n.to_string())
                        .text_size(px(17.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .tabular_nums()
                        .text_color(tone.color()),
                )
        });
        ui::Card::new()
            .icon(IconName::Animation)
            .title("Layout animation")
            .subtitle("configureNext, then change the tree")
            .right(Badge::new(self.items.len().to_string()))
            .child(
                Segmented::element(SegmentedProps {
                    options: PRESETS
                        .iter()
                        .map(|(_, label)| Segment::new(*label))
                        .collect(),
                    selected: self.preset,
                    size: SegmentedSize::Small,
                })
                .on_event(cx.listener(
                    |this: &mut Self, e: &SegmentedEvent, cx| {
                        let SegmentedEvent::Change(i) = *e;
                        this.preset = i;
                        cx.notify();
                    },
                )),
            )
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .min_h(px(56.0))
                    .children(boxes),
            )
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(cell(
                        ui::Button::new(
                            "Add",
                            cx.listener(|this: &mut Self, _, cx| {
                                this.animate(cx, |this| {
                                    this.items.insert(0, this.next);
                                    this.next += 1;
                                });
                            }),
                        )
                        .icon(IconName::Plus),
                    ))
                    .child(cell(
                        ui::Button::new(
                            "Shuffle",
                            cx.listener(|this: &mut Self, _, cx| {
                                this.animate(cx, |this| shuffle(&mut this.items));
                            }),
                        )
                        .variant(UiVariant::Secondary),
                    ))
                    .child(cell(
                        ui::Button::new(
                            "Remove",
                            cx.listener(|this: &mut Self, _, cx| {
                                this.animate(cx, |this| {
                                    if !this.items.is_empty() {
                                        this.items.remove(0);
                                    }
                                });
                            }),
                        )
                        .variant(UiVariant::Secondary)
                        .disabled(self.items.is_empty()),
                    )),
            )
    }
}

/// An input with a bar over the keyboard (iOS's input accessory).
#[derive(Debug, Default)]
struct Accessory {
    text: String,
    focused: bool,
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
        let hairline = hairline(cx);
        ui::Card::new()
            .icon(IconName::Text)
            .title("Input accessory")
            .subtitle("A bar over the iOS keyboard")
            .right(Badge::new("nsh/ios"))
            .child(
                field(self.focused, cx)
                    .element_ref(input_ref)
                    .value(self.text.clone())
                    .placeholder("Focus me")
                    .input_accessory("counter")
                    .on_focus(cx.listener(|this: &mut Self, _, cx| {
                        this.focused = true;
                        cx.notify();
                    }))
                    .on_blur(cx.listener(|this: &mut Self, _, cx| {
                        this.focused = false;
                        cx.notify();
                    }))
                    .on_change_text(cx.listener(|this: &mut Self, v: &str, cx| {
                        this.text = v.to_owned();
                        cx.notify();
                    })),
            )
            .child(caption(
                if cx.os() == Os::Android {
                    "Android has no input accessories"
                } else {
                    "The bar counts what you type"
                },
                Some("inputAccessoryViewID=\"counter\""),
            ))
            .child(
                input_accessory_view("counter").bg(colors::SURFACE).child(
                    view()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .border_t(hairline)
                        .border_t_color(colors::BORDER)
                        .child(
                            text(format!("{} characters", self.text.encode_utf16().count()))
                                .flex_1()
                                .text_size(px(13.0))
                                .text_color(colors::MUTED)
                                .tabular_nums(),
                        )
                        .child(
                            ui::Button::new(
                                "Clear",
                                cx.listener(|this: &mut Self, _, cx| {
                                    this.text.clear();
                                    cx.notify();
                                }),
                            )
                            .variant(UiVariant::Secondary),
                        )
                        .child(ui::Button::new(
                            "Done",
                            cx.listener(|this: &mut Self, _, _| {
                                if let Some(input) = &this.input {
                                    input.blur();
                                }
                            }),
                        )),
                ),
            )
    }
}

/// The behaviors of the keyboard avoiding view: label, what it does.
const BEHAVIORS: [(AvoidBehavior, &str, &str); 3] = [
    (
        AvoidBehavior::Padding,
        "padding",
        "The bottom pads: the tinted background stays under the keyboard.",
    ),
    (
        AvoidBehavior::Height,
        "height",
        "The view shrinks: its background ends at the keyboard.",
    ),
    (
        AvoidBehavior::Position,
        "position",
        "The inner content view moves up as a whole.",
    ),
];

/// The content of the keyboard avoiding view's screen.
#[derive(Debug, Default)]
struct AvoidingPage {
    behavior: u32,
    focused: bool,
}

impl Component for AvoidingPage {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (behavior, name, what) = BEHAVIORS[self.behavior as usize];
        // The inner view of `position` fills the avoiding view.
        let fill = Style::from_json(JsonValue::from_iter([("flex", 1)])).unwrap_or_default();
        view().flex_1().child(
            KeyboardAvoidingView::element(KeyboardAvoidingViewProps {
                behavior: behavior.into(),
                content_container_style: fill.into(),
                ..KeyboardAvoidingViewProps::default()
            })
            .flex_1()
            .bg(tints::INFO)
            .child(
                view()
                    .flex_1()
                    .p_4()
                    .gap_4()
                    .child(
                        appear(0).child(
                            ui::Card::new()
                                .icon(IconName::Form)
                                .title(format!("behavior=\"{name}\""))
                                .subtitle("Focus the field at the bottom")
                                .child(
                                    segmented(
                                        BEHAVIORS.map(|(_, label, _)| Segment::new(label)),
                                        self.behavior,
                                    )
                                    .on_event(cx.listener(
                                        |this: &mut Self, e: &SegmentedEvent, cx| {
                                            let SegmentedEvent::Change(i) = *e;
                                            this.behavior = i;
                                            cx.notify();
                                        },
                                    )),
                                )
                                .child(note(what)),
                        ),
                    )
                    .child(view().flex_1())
                    .child(
                        field(self.focused, cx)
                            .placeholder("Type here")
                            .on_focus(cx.listener(|this: &mut Self, _, cx| {
                                this.focused = true;
                                cx.notify();
                            }))
                            .on_blur(cx.listener(|this: &mut Self, _, cx| {
                                this.focused = false;
                                cx.notify();
                            })),
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
        let translucent = self.translucent;
        let modal_view = self.visible.then(|| {
            modal()
                .animation(ModalAnimation::Slide)
                .status_bar_translucent(translucent)
                .supported_orientations([Orientation::Portrait, Orientation::Landscape])
                .on_request_close(show(false))
                .child(
                    view()
                        .flex_1()
                        .bg(AdaptiveColor::new(rgb(0xeef5dc), rgb(0x16200d)))
                        .child(
                            SafeAreaView::element(SafeAreaViewProps::default())
                                .flex_1()
                                .p_4()
                                .gap_3()
                                .child(
                                    view().flex_row().pt_2().child(
                                        Badge::new(if translucent {
                                            "translucent"
                                        } else {
                                            "opaque bar"
                                        })
                                        .tone(Tone::Accent),
                                    ),
                                )
                                .child(
                                    text("A modal")
                                        .text_size(px(28.0))
                                        .font_weight(FontWeight::BOLD)
                                        .letter_spacing(-0.5)
                                        .text_color(colors::TEXT),
                                )
                                .child(
                                    text(format!(
                                        "The olive background starts {} the status bar on \
                                         Android.",
                                        if translucent { "under" } else { "below" }
                                    ))
                                    .text_size(px(15.0))
                                    .line_height(px(21.0))
                                    .text_color(colors::MUTED),
                                )
                                .child(view().flex_1())
                                .child(ui::Button::new("Close", show(false))),
                        ),
                )
        });
        ui::Card::new()
            .icon(IconName::System)
            .title("Modal options")
            .subtitle("statusBarTranslucent, orientations")
            .right(
                toggle(translucent, "statusBarTranslucent").on_event(cx.listener(
                    |this: &mut Self, e: &ToggleEvent, cx| {
                        let ToggleEvent::Change(on) = *e;
                        this.translucent = on;
                        cx.notify();
                    },
                )),
            )
            .child(note(
                "Android: translucent content reaches under the status bar. iOS: this modal \
                 allows portrait and landscape.",
            ))
            .child(caption(
                "statusBarTranslucent",
                Some(translucent.to_string()),
            ))
            .child(ui::Button::new("Open the modal", show(true)).variant(UiVariant::Secondary))
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
        let hairline = hairline(cx);
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
        .into_iter()
        .enumerate()
        .map(|(i, (name, value))| {
            Row::new(i == 0)
                .child(
                    text(name)
                        .number_of_lines(1)
                        .flex_1()
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                        .font_family(MONO),
                )
                .child(
                    text(value)
                        .number_of_lines(1)
                        .text_size(px(14.0))
                        .text_color(colors::TEXT)
                        .tabular_nums(),
                )
        });
        ui::Card::new()
            .icon(IconName::Spark)
            .title("Platform")
            .subtitle("Platform, PixelRatio, AccessibilityInfo")
            .right(Badge::new(os.name()).tone(Tone::Accent))
            .child(view().children(rows))
            .child(view().h(px(hairline)).bg(colors::ACCENT))
            .child(caption(
                "The line above is one device pixel tall",
                Some(format!("{hairline:.3} pt")),
            ))
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(cell(ui::Button::new(
                        "Announce",
                        cx.listener(|this: &mut Self, _, cx| {
                            this.announced += 1;
                            cx.announce_for_accessibility(format!(
                                "Announcement {}",
                                this.announced
                            ));
                        }),
                    )))
                    .child(cell(
                        ui::Button::new(
                            "Focus the note",
                            cx.listener(|this: &mut Self, _, _| {
                                if let Some(note) = &this.note {
                                    note.set_accessibility_focus();
                                }
                            }),
                        )
                        .variant(UiVariant::Secondary),
                    )),
            )
            .child(
                text(
                    "With VoiceOver or TalkBack on, the buttons above speak and move the reader \
                     here.",
                )
                .text_size(px(13.0))
                .line_height(px(18.0))
                .text_color(colors::FAINT)
                .element_ref(self.note.as_ref().expect("set up")),
            )
    }
}

/// A row that pushes a screen.
fn link_row(
    icon_name: IconName,
    title: &'static str,
    prop: &'static str,
    on_press: Listener<()>,
) -> impl IntoElement {
    press_with(
        PressProps {
            scale: 0.98,
            label: Some(title.into()),
            ..PressProps::default()
        },
        on_press,
    )
    .child(
        view()
            .min_h(px(60.0))
            .flex_row()
            .items_center()
            .gap_3()
            .px(px(10.0))
            .py_2()
            .child(IconTile::new(icon_name).size(34.0))
            .child(
                view()
                    .flex_1()
                    .gap(px(2.0))
                    .child(
                        text(title)
                            .text_size(px(15.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        text(prop)
                            .number_of_lines(1)
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .font_family(MONO),
                    ),
            )
            .child(icon(IconName::Chevron, 14.0, colors::FAINT)),
    )
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Components;

impl Component for Components {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let avoiding = ui::Card::new()
            .title("Keyboard avoidance")
            .subtitle("Content outside scroll views")
            .flush()
            .child(Inset::new().padding(0.0).gap(0.0).child(link_row(
                IconName::Form,
                "KeyboardAvoidingView",
                "padding / height / position",
                cx.listener(|_: &mut Self, _, cx| {
                    nav::push(cx, "keyboard-avoiding", |cx| {
                        header(screen().title("KeyboardAvoidingView"), cx)
                            .child(AvoidingPage::element(()))
                    });
                }),
            )));
        let cards = [
            MobileComponents::element(()).into_element(),
            PressFeedback::element(()).into_element(),
            PressableSheet::element(()).into_element(),
            LayoutAnimations::element(()).into_element(),
            Accessory::element(()).into_element(),
            avoiding.into_element(),
            ModalOptions::element(()).into_element(),
            PlatformInfo::element(()).into_element(),
        ];
        view().flex_1().child(
            scroll()
                .flex_1()
                .bg(colors::BACKGROUND)
                .keyboard_dismiss_mode(KeyboardDismissMode::Interactive)
                .child(
                    view().p_4().pb(px(40.0)).gap_4().children(
                        cards
                            .into_iter()
                            .enumerate()
                            .map(|(i, card)| appear(i as u32).child(card)),
                    ),
                ),
        )
    }
}
