//! Navigation: screens with header options, a deep stack, a screen that
//! asks before it is left, modals of every presentation and the Android
//! back button (as the TSX gallery's `demos/Navigation.tsx`). Each pushed
//! screen shows where its safe area is.

use nsh_core::{
    Dialog, DialogButton, DialogButtonStyle, ModalPresentation,
    ui::{AnyElement, RenderCx, Screen, prelude::*},
};
use nsh_ui_mobile::{SafeAreaView, SafeAreaViewProps};

use crate::{
    Gallery, nav,
    ui::{
        Badge, Button, ButtonVariant, Card, Figure, Finish, Finished, IconName, IconTile, Inset,
        MONO, PressProps, Section, Stat, ToggleEvent, Tone, appear, colors, header, icon, motion,
        press_with, radius, tints, toggle,
    },
};

/// The safe-area insets where it renders, one figure per edge.
#[derive(Debug, Default)]
struct Insets;

impl Component for Insets {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let i = cx.insets();
        view().flex_row().children(
            [
                ("Top", i.top),
                ("Bottom", i.bottom),
                ("Left", i.left),
                ("Right", i.right),
            ]
            .map(|(edge, inset)| Stat::new(edge, format!("{} pt", inset.round())).width(0.25)),
        )
    }
}

fn insets() -> AnyElement {
    Insets::element(()).into_element()
}

/// Muted body text.
fn body(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_size(px(15.0))
        .line_height(px(21.0))
        .text_color(colors::MUTED)
}

/// A screen's big title over a badge naming its option.
fn title_block(badge: impl Into<SharedString>, title: impl Into<SharedString>, size: f32) -> View {
    view()
        .gap_2()
        .child(
            view()
                .flex_row()
                .child(Badge::new(badge).tone(Tone::Accent)),
        )
        .child(
            text(title.into())
                .text_size(px(size))
                .font_weight(FontWeight::BOLD)
                .letter_spacing(-0.5)
                .text_color(colors::TEXT),
        )
}

/// The body of a pushed screen: what it shows, then where its safe area
/// is, then `content`.
fn page(intro: &str, content: impl IntoIterator<Item = AnyElement>) -> AnyElement {
    scroll()
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .p_4()
                .pb(px(40.0))
                .gap_4()
                .child(appear(0).child(body(intro.to_owned()).px_1()))
                .child(
                    appear(1).child(
                        Section::new("Safe area")
                            .note("useSafeAreaInsets()")
                            .child(insets()),
                    ),
                )
                .child(appear(2).child(view().gap_4().children(content))),
        )
        .into_element()
}

/// The stars of the custom header (its header item and content both show
/// them).
#[derive(Default)]
struct Likes(u32);

/// The draft of the unsaved-changes screen (its `prevent_back` depends on
/// it).
#[derive(Default)]
struct Draft(String);

fn large_title(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let rows = (1..=20).map(|i| {
        view()
            .h(px(48.0))
            .flex_row()
            .items_center()
            .px(px(10.0))
            .when(i > 1, |v| v.border_t_1().border_t_color(colors::LINE))
            .child(
                text(format!("Row {i}"))
                    .flex_1()
                    .text_size(px(15.0))
                    .text_color(colors::TEXT),
            )
            .child(
                text(format!("{} pt", i * 48))
                    .text_size(px(13.0))
                    .text_color(colors::FAINT)
                    .tabular_nums(),
            )
    });
    header(screen().title("Large title"), cx)
        .header_large_title(true)
        .child(page(
            "iOS: the title shrinks into the bar as the content scrolls.",
            [Card::new()
                .flush()
                .child(Inset::new().padding(0.0).gap(0.0).children(rows))
                .into_element()],
        ))
}

fn transparent(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let text_color = cx.color(colors::TEXT);
    screen()
        .title("Transparent")
        .header_transparent(true)
        .header_tint_color(text_color)
        .header_title_color(text_color)
        .child(
            view()
                .flex_1()
                .bg(AdaptiveColor::new(rgb(0xeef3e2), rgb(0x11170c)))
                .overflow_hidden()
                .p_4()
                .pb(px(56.0))
                .justify_end()
                .gap_4()
                .child(
                    view()
                        .absolute()
                        .top(px(-90.0))
                        .right(px(-70.0))
                        .size(px(320.0))
                        .rounded(px(160.0))
                        .bg(tints::ACCENT),
                )
                .child(
                    view()
                        .absolute()
                        .top(px(150.0))
                        .left(px(-110.0))
                        .size(px(260.0))
                        .rounded(px(130.0))
                        .bg(tints::INFO),
                )
                .child(
                    appear(0).child(
                        title_block("headerTransparent", "Content under the header", 30.0)
                            .gap_3()
                            .child(body(
                                "The circles start at the top of the screen; the bar and its back \
                                 button float over them.",
                            )),
                    ),
                )
                .child(appear(1).child(Inset::new().child(insets()))),
        )
}

fn custom_header(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let likes = cx.global::<Likes>().0;
    let on_accent = cx.color(colors::ON_ACCENT);
    screen()
        .title("Custom")
        .header_tint_color(on_accent)
        .header_background_color(cx.color(colors::ACCENT_FILL))
        .header_title_color(on_accent)
        .header_back_visible(false)
        .header_left(
            text("Close")
                .text_color(on_accent)
                .text_size(px(17.0))
                .font_weight(FontWeight::MEDIUM)
                .hit_slop(EdgeInsets::all(10.0))
                .on_press(cx.listener(|_: &mut Gallery, _, cx| nav::pop(cx))),
        )
        .header_title(
            view()
                .flex_row()
                .items_center()
                .gap(px(7.0))
                .child(view().size(px(10.0)).rounded(px(5.0)).bg(on_accent))
                .child(
                    text("Custom title")
                        .text_color(on_accent)
                        .text_size(px(17.0))
                        .font_weight(FontWeight::BOLD),
                ),
        )
        .header_right(
            view()
                .flex_row()
                .items_center()
                .gap(px(5.0))
                .hit_slop(EdgeInsets::all(10.0))
                .on_press(cx.listener(|_: &mut Gallery, _, cx| {
                    cx.update_global::<Likes, _>(|likes, _| likes.0 += 1);
                }))
                .child(icon(IconName::Star, 18.0, colors::ON_ACCENT))
                .child(
                    text(likes.to_string())
                        .text_color(on_accent)
                        .text_size(px(17.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .tabular_nums(),
                ),
        )
        .child(page(
            "The header's items are views: the star on the right counts presses and re-renders \
             the header.",
            [Section::new("Header")
                .note("headerRight")
                .child(Figure::new(likes.to_string()).unit(if likes == 1 {
                    "star"
                } else {
                    "stars"
                }))
                .into_element()],
        ))
}

fn locked(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    header(screen().title("No swipe back"), cx)
        .gesture_enabled(false)
        .child(page(
            "The back gesture is off on this screen: no swipe on iOS, no predictive animation on \
             Android. The back buttons still work.",
            [Section::new("Leaving")
                .note("gestureEnabled={false}")
                .child(
                    view()
                        .flex_row()
                        .child(
                            Stat::new("Swipe / predictive", "Off")
                                .tone(Tone::Danger)
                                .width(0.5),
                        )
                        .child(Stat::new("Back buttons", "On").tone(Tone::Up).width(0.5)),
                )
                .into_element()],
        ))
}

/// The draft's card (the screen's `prevent_back` follows the draft).
#[derive(Debug, Default)]
struct DraftCard {
    focused: bool,
}

impl Component for DraftCard {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let draft = cx.global::<Draft>().0.clone();
        let length = draft.chars().count();
        let guarded = length > 0;
        Card::new()
            .icon(IconName::Form)
            .title("Draft")
            .subtitle("preventBack + onBackRequest")
            .right(
                Badge::new(if guarded { "Guarded" } else { "Empty" }).tone(if guarded {
                    Tone::Warn
                } else {
                    Tone::Neutral
                }),
            )
            .child(
                input()
                    .value(draft)
                    .placeholder("A draft")
                    .placeholder_color(cx.color(colors::FAINT))
                    .selection_color(cx.color(colors::ACCENT))
                    .h(px(48.0))
                    .px(px(14.0))
                    .text_size(px(15.0))
                    .text_color(colors::TEXT)
                    .bg(colors::INSET)
                    .border_1()
                    .border_color(if self.focused {
                        colors::ACCENT
                    } else {
                        colors::BORDER
                    })
                    .rounded(px(radius::CONTROL))
                    .finish(Finish::Inset)
                    .on_focus(cx.listener(|this: &mut Self, _, cx| {
                        this.focused = true;
                        cx.notify();
                    }))
                    .on_blur(cx.listener(|this: &mut Self, _, cx| {
                        this.focused = false;
                        cx.notify();
                    }))
                    .on_change_text(cx.listener(|_: &mut Self, text: &str, cx| {
                        let text = text.to_owned();
                        cx.update_global::<Draft, _>(|draft, _| draft.0 = text);
                    })),
            )
            .child(
                text(if guarded {
                    format!("{length} characters would be lost: back asks first")
                } else {
                    "Nothing to lose: back pops".to_owned()
                })
                .text_size(px(13.0))
                .text_color(colors::MUTED),
            )
    }
}

fn unsaved(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let editing = !cx.global::<Draft>().0.is_empty();
    header(screen().title("Unsaved changes"), cx)
        .prevent_back(editing)
        .on_back_request(cx.listener(|_: &mut Gallery, _, cx| {
            let answer = cx.dialog(Dialog {
                id: 0,
                title: Some("Discard your changes?".into()),
                message: None,
                buttons: vec![
                    DialogButton {
                        text: "Keep editing".into(),
                        style: DialogButtonStyle::Cancel,
                    },
                    DialogButton {
                        text: "Discard".into(),
                        style: DialogButtonStyle::Destructive,
                    },
                ],
                cancelable: true,
                sheet: false,
            });
            cx.spawn(|this, _| async move {
                if answer.await == 1 {
                    this.update(|_, cx| {
                        // TS's draft is the screen's own and goes with it.
                        cx.update_global::<Draft, _>(|draft, _| draft.0.clear());
                        nav::pop(cx);
                    })
                    .await;
                }
            })
            .detach();
        }))
        .child(page(
            "Type something, then go back (header button, swipe, Android back): the screen asks \
             first.",
            [DraftCard::element(()).into_element()],
        ))
}

fn faded(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    header(screen().title("Fade"), cx)
        .animation(ScreenAnimation::Fade)
        .child(page("This screen faded in, and fades out.", []))
}

fn instant(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    header(screen().title("No animation"), cx)
        .animation(ScreenAnimation::None)
        .child(page(
            "Shown without a transition, and removed without one.",
            [],
        ))
}

fn headerless(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    screen().header_shown(false).child(
        SafeAreaView::element(SafeAreaViewProps::default())
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(
                view()
                    .flex_1()
                    .p_4()
                    .gap_4()
                    .child(
                        appear(0).child(
                            title_block("headerShown={false}", "No header", 30.0)
                                .px_1()
                                .pt_3()
                                .child(body(
                                    "The screen owns the whole surface; a SafeAreaView keeps it \
                                     clear of the status bar and the home indicator.",
                                )),
                        ),
                    )
                    .child(
                        appear(1).child(
                            Section::new("Safe area")
                                .note("SafeAreaView")
                                .child(insets()),
                        ),
                    )
                    .child(view().flex_1())
                    .child(Button::new(
                        "Back",
                        cx.listener(|_: &mut Gallery, _, cx| nav::pop(cx)),
                    )),
            ),
    )
}

/// Level `depth` of a deep stack.
fn deep(depth: usize) -> impl Fn(&mut RenderCx<'_, Gallery>) -> Screen {
    move |cx| {
        let deeper = cx.listener(move |_: &mut Gallery, _, cx| {
            nav::push(cx, format!("deep-{}", depth + 1), deep(depth + 1));
        });
        let levels = (1..=depth).map(|level| {
            let current = level == depth;
            view()
                .size(px(32.0))
                .rounded(px(8.0))
                .items_center()
                .justify_center()
                .map(|v| {
                    if current {
                        v.bg(colors::ACCENT_FILL).finish(Finish::Accent)
                    } else {
                        v.border_1()
                            .border_color(colors::BORDER)
                            .bg(colors::RAISED)
                            .finish(Finish::Raised)
                    }
                })
                .child(
                    text(level.to_string())
                        .text_size(px(13.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .tabular_nums()
                        .text_color(if current {
                            colors::ON_ACCENT
                        } else {
                            colors::MUTED
                        }),
                )
        });
        header(screen().title(format!("Level {depth}")), cx).child(page(
            "Every push is a native screen; popToTop() unwinds them all at once.",
            [Card::new()
                .icon(IconName::Nav)
                .title(format!("Level {depth}"))
                .subtitle(format!("{depth} deep in the stack"))
                .child(view().flex_row().flex_wrap().gap(px(6.0)).children(levels))
                .child(
                    view()
                        .flex_row()
                        .gap_2()
                        .child(Button::new("Deeper", deeper).icon(IconName::Plus).flex_1())
                        .child(
                            Button::new(
                                "To the index",
                                cx.listener(|_: &mut Gallery, _, cx| nav::pop_to_top(cx)),
                            )
                            .variant(ButtonVariant::Secondary)
                            .flex_1(),
                        ),
                )
                .into_element()],
        ))
    }
}

/// A row that pushes a screen: an icon, a title, the API it shows.
fn link_row(
    icon_name: IconName,
    title: &'static str,
    note: &'static str,
    first: bool,
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
            .when(!first, |v| v.border_t_1().border_t_color(colors::LINE))
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
                        text(note)
                            .number_of_lines(1)
                            .text_size(px(12.0))
                            .text_color(colors::FAINT)
                            .font_family(MONO),
                    ),
            )
            .child(icon(IconName::Chevron, 14.0, colors::FAINT)),
    )
}

/// Which modal shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Presented {
    FullScreen,
    PageSheet,
    FormSheet,
    Fade,
    Dialog,
    Dismissible,
}

/// The modals: what they are, their note, their icon.
const MODALS: [(Presented, &str, &str, &str, IconName); 6] = [
    (
        Presented::FullScreen,
        "fullScreen",
        "Full screen",
        "Slides up",
        IconName::Mobile,
    ),
    (
        Presented::PageSheet,
        "pageSheet",
        "Page sheet",
        "iOS sheet",
        IconName::Components,
    ),
    (
        Presented::FormSheet,
        "formSheet",
        "Form sheet",
        "iOS, centered",
        IconName::Form,
    ),
    (Presented::Fade, "fade", "Fade", "Fades in", IconName::Spark),
    (
        Presented::Dialog,
        "dialog",
        "Dialog",
        "See-through",
        IconName::Bell,
    ),
    (
        Presented::Dismissible,
        "dismissible",
        "Dismissible",
        "Swipe to close",
        IconName::ArrowDown,
    ),
];

/// A modal's content: what it is, its safe area, a close button.
fn modal_body(title: &str, note: &str, text_body: &str, close: Listener<()>) -> impl IntoElement {
    SafeAreaView::element(SafeAreaViewProps::default())
        .flex_1()
        .bg(colors::BACKGROUND)
        .child(
            view()
                .flex_1()
                .p_4()
                .gap_4()
                .child(
                    title_block(note.to_owned(), title.to_owned(), 28.0)
                        .px_1()
                        .pt_2()
                        .child(body(text_body.to_owned())),
                )
                .child(
                    Section::new("Safe area")
                        .note("the modal's")
                        .child(insets()),
                )
                .child(view().flex_1())
                .child(Button::new("Close", close)),
        )
}

/// Whether the gallery takes the Android back button where it would leave
/// the app (its index), and how often it did: the gallery listens
/// (`Gallery::setup`), this demo switches it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct BackIntercept {
    pub(crate) on: bool,
    pub(crate) count: u32,
}

/// The demo's page: rows pushing the screens, its modals, and the Android
/// back button.
#[derive(Debug, Default)]
pub(crate) struct Navigation {
    sheet: Option<Presented>,
    /// The dialog's spring in, 0 to 1.
    pop: Option<NativeValue>,
}

impl Component for Navigation {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.pop = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let intercept = *cx.global::<BackIntercept>();
        let pop = self.pop.as_ref().expect("set up");
        type Make = fn(&mut RenderCx<'_, Gallery>) -> Screen;
        let screens: [(&str, IconName, &str, &str, Make); 8] = [
            (
                "large-title",
                IconName::Text,
                "Large title",
                "ios.headerLargeTitle",
                large_title,
            ),
            (
                "transparent",
                IconName::Styles,
                "Transparent header",
                "headerTransparent",
                transparent,
            ),
            (
                "custom",
                IconName::Star,
                "Custom header items",
                "headerLeft / Title / Right",
                custom_header,
            ),
            (
                "headerless",
                IconName::Mobile,
                "No header",
                "headerShown={false}",
                headerless,
            ),
            (
                "faded",
                IconName::Spark,
                "Fade in",
                "animation=\"fade\"",
                faded,
            ),
            (
                "instant",
                IconName::Play,
                "No animation",
                "animation=\"none\"",
                instant,
            ),
            (
                "locked",
                IconName::Gestures,
                "No swipe back",
                "gestureEnabled={false}",
                locked,
            ),
            (
                "unsaved",
                IconName::Form,
                "Unsaved changes",
                "preventBack + onBackRequest",
                unsaved,
            ),
        ];
        let mut links: Vec<AnyElement> = screens
            .into_iter()
            .enumerate()
            .map(|(i, (key, icon_name, title, note, make))| {
                link_row(
                    icon_name,
                    title,
                    note,
                    i == 0,
                    cx.listener(move |_: &mut Self, _, cx| nav::push(cx, key, make)),
                )
                .into_element()
            })
            .collect();
        links.push(
            link_row(
                IconName::Lists,
                "A deep stack",
                "push / popToTop",
                false,
                cx.listener(|_: &mut Self, _, cx| {
                    let depth = nav::depth(cx);
                    nav::push(cx, format!("deep-{depth}"), deep(1));
                }),
            )
            .into_element(),
        );
        let tiles = MODALS.map(|(sheet, _, title, note, icon_name)| {
            press_with(
                PressProps {
                    label: Some(title.into()),
                    ..PressProps::default()
                },
                cx.listener(move |this: &mut Self, _, cx| {
                    if sheet == Presented::Dialog {
                        // The dialog springs up from a little smaller each
                        // time it opens.
                        let pop = this.pop.as_ref().expect("set up");
                        pop.set(0.0);
                        pop.spring(1.0, motion::SETTLE);
                    }
                    this.sheet = Some(sheet);
                    cx.notify();
                }),
            )
            .w(relative(0.485))
            .child(
                view()
                    .min_h(px(100.0))
                    .p_3()
                    .gap_3()
                    .rounded(px(radius::INSET))
                    .border_curve(BorderCurve::Continuous)
                    .bg(colors::INSET)
                    .border_1()
                    .border_color(colors::LINE)
                    .finish(Finish::Inset)
                    .child(IconTile::new(icon_name).size(32.0))
                    .child(
                        view()
                            .gap(px(2.0))
                            .child(
                                text(title)
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::TEXT),
                            )
                            .child(text(note).text_size(px(13.0)).text_color(colors::FAINT)),
                    ),
            )
        });
        let close = || {
            cx.listener(|this: &mut Self, _, cx| {
                this.sheet = None;
                cx.notify();
            })
        };
        let modal_view = match self.sheet {
            Some(
                sheet @ (Presented::FullScreen
                | Presented::PageSheet
                | Presented::FormSheet
                | Presented::Fade),
            ) => {
                let presentation = match sheet {
                    Presented::PageSheet => ModalPresentation::PageSheet,
                    Presented::FormSheet => ModalPresentation::FormSheet,
                    _ => ModalPresentation::FullScreen,
                };
                let (_, key, title, ..) =
                    MODALS.into_iter().find(|m| m.0 == sheet).expect("a modal");
                Some(
                    modal()
                        .presentation(presentation)
                        .animation(if sheet == Presented::Fade {
                            ModalAnimation::Fade
                        } else {
                            ModalAnimation::Slide
                        })
                        .on_request_close(close())
                        .child(modal_body(
                            title,
                            key,
                            "Close with the button, the back button (Android) or a swipe down \
                             (iOS sheets).",
                            close(),
                        )),
                )
            }
            Some(Presented::Dismissible) => Some(
                modal()
                    .dismissible(true)
                    .presentation(ModalPresentation::PageSheet)
                    .on_request_close(close())
                    .on_dismiss(close())
                    .child(modal_body(
                        "Dismissible",
                        "dismissible",
                        "Swipe it down (iOS) or press back (Android): it closes natively, then \
                         tells the app.",
                        close(),
                    )),
            ),
            Some(Presented::Dialog) => Some(
                modal()
                    .transparent(true)
                    .animation(ModalAnimation::Fade)
                    .on_request_close(close())
                    .child(
                        view()
                            .flex_1()
                            .bg(rgba(0x0000_009e))
                            .items_center()
                            .justify_center()
                            .p_4()
                            .child(
                                view()
                                    .self_stretch()
                                    .bind_opacity(pop)
                                    .bind_scale(&pop.interpolate(&[0.0, 1.0], &[0.9, 1.0]))
                                    .child(
                                        Card::new()
                                            .icon(IconName::Bell)
                                            .title("A dialog")
                                            .subtitle("transparent + fade")
                                            .child(body(
                                                "A transparent modal shows what is behind it; \
                                                 the dialog springs in natively.",
                                            ))
                                            .child(
                                                view()
                                                    .flex_row()
                                                    .gap_2()
                                                    .child(
                                                        Button::new("Cancel", close())
                                                            .variant(ButtonVariant::Secondary)
                                                            .flex_1(),
                                                    )
                                                    .child(Button::new("OK", close()).flex_1()),
                                            ),
                                    ),
                            ),
                    ),
            ),
            None => None,
        };
        let presses = intercept.count;
        view().flex_1().child(
            scroll()
                .flex_1()
                .bg(colors::BACKGROUND)
                .child(
                    view()
                        .p_4()
                        .pb(px(40.0))
                        .gap_4()
                        .child(
                            appear(0).child(
                                Card::new()
                                    .icon(IconName::Nav)
                                    .title("Native stack")
                                    .subtitle("Headers, transitions, the safe area")
                                    .right(Badge::new("UIKit / View"))
                                    .child(insets()),
                            ),
                        )
                        .child(
                            appear(1).child(
                                Card::new()
                                    .title("Screens")
                                    .subtitle("Screen options, pushed with nav.push")
                                    .flush()
                                    .child(Inset::new().padding(0.0).gap(0.0).children(links)),
                            ),
                        )
                        .child(
                            appear(2).child(
                                Card::new()
                                    .title("Modals")
                                    .subtitle("Presentation styles and transitions")
                                    .flush()
                                    .child(
                                        view()
                                            .flex_row()
                                            .flex_wrap()
                                            .justify_between()
                                            .gap_y_2()
                                            .children(tiles),
                                    ),
                            ),
                        )
                        .child(
                            appear(3).child(
                                Card::new()
                                    .icon(IconName::System)
                                    .title("Back button")
                                    .subtitle("Android: BackHandler")
                                    .right(
                                        toggle(intercept.on, "Intercept the back button").on_event(
                                            cx.listener(|_: &mut Self, e: &ToggleEvent, cx| {
                                                let ToggleEvent::Change(on) = *e;
                                                cx.update_global::<BackIntercept, _>(|b, _| {
                                                    b.on = on
                                                });
                                            }),
                                        ),
                                    )
                                    .child(
                                        Figure::new(presses.to_string())
                                            .unit(if presses == 1 { "press" } else { "presses" })
                                            .note(if intercept.on { "caught" } else { "popped" })
                                            .tone(if intercept.on {
                                                Tone::Warn
                                            } else {
                                                Tone::Accent
                                            }),
                                    )
                                    .child(
                                        text(if intercept.on {
                                            "The handler returns true: back presses on the index \
                                         stay in the gallery."
                                        } else {
                                            "Off: the back button pops this screen."
                                        })
                                        .text_size(px(13.0))
                                        .line_height(px(18.0))
                                        .text_color(colors::MUTED),
                                    ),
                            ),
                        ),
                )
                .children(modal_view),
        )
    }
}
