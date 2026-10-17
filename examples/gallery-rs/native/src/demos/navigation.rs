//! Navigation: screens with header options, a deep stack, a screen that
//! asks before it is left, and modals (as the TSX gallery's
//! `demos/Navigation.tsx`).

use nsh_core::{
    Dialog, DialogButton, DialogButtonStyle, ModalPresentation,
    ui::{AnyElement, RenderCx, Screen, prelude::*},
};
use nsh_ui_mobile::{SafeAreaView, SafeAreaViewProps};

use crate::{
    Gallery, nav,
    ui::{button, colors, page, section},
};

/// The safe-area insets where it renders.
#[derive(Debug, Default)]
struct Insets;

impl Component for Insets {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let i = cx.insets();
        view().child(
            text(format!(
                "insets: top {}, bottom {}, left {}, right {}",
                i.top.round(),
                i.bottom.round(),
                i.left.round(),
                i.right.round()
            ))
            .text_size(px(13.0))
            .text_color(colors::muted()),
        )
    }
}

fn insets() -> AnyElement {
    Insets::element(()).into_element()
}

fn filler(count: usize) -> Vec<AnyElement> {
    (1..=count)
        .map(|i| {
            view()
                .p(px(14.0))
                .bg(colors::surface())
                .rounded(px(10.0))
                .child(text(format!("Row {i}")))
                .into_element()
        })
        .collect()
}

fn note(content: &str) -> AnyElement {
    text(content.to_owned())
        .text_color(colors::muted())
        .into_element()
}

/// A page of a pushed screen: its insets, then `content`.
fn screen_page(content: Vec<AnyElement>) -> AnyElement {
    page(std::iter::once(insets()).chain(content)).into_element()
}

/// The likes of the custom header (its header item and content both show
/// them).
#[derive(Default)]
struct Likes(u32);

/// The draft of the unsaved-changes screen (its `prevent_back` depends on
/// it).
#[derive(Default)]
struct Draft(String);

fn large_title(_: &mut RenderCx<'_, Gallery>) -> Screen {
    screen()
        .title("Large title")
        .header_large_title(true)
        .child(screen_page(
            std::iter::once(note(
                "iOS: the title shrinks into the bar as the content scrolls.",
            ))
            .chain(filler(20))
            .collect(),
        ))
}

fn transparent(_: &mut RenderCx<'_, Gallery>) -> Screen {
    screen()
        .title("Transparent")
        .header_transparent(true)
        .header_tint_color(rgb(0xffffff))
        .header_title_color(rgb(0xffffff))
        .child(
            view()
                .flex_1()
                .bg(colors::purple())
                .p_6()
                .justify_center()
                .gap_3()
                .child(
                    text("Content under the header")
                        .text_color(rgb(0xffffff))
                        .text_size(px(22.0))
                        .font_weight(FontWeight::BOLD),
                )
                .child(insets()),
        )
}

fn custom_header(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let likes = cx.global::<Likes>().0;
    let white = rgb(0xffffff);
    screen()
        .title("Custom")
        .header_tint_color(white)
        .header_background_color(rgb(0x1a7f37))
        .header_title_color(white)
        .header_back_visible(false)
        .header_left(
            text("Close")
                .text_color(white)
                .text_size(px(17.0))
                .on_press(cx.listener(|_: &mut Gallery, _, cx| nav::pop(cx))),
        )
        .header_title(
            view()
                .flex_row()
                .items_center()
                .gap(px(6.0))
                .child(view().size(px(12.0)).rounded(px(6.0)).bg(white))
                .child(
                    text("Custom title")
                        .text_color(white)
                        .text_size(px(17.0))
                        .font_weight(FontWeight::BOLD),
                ),
        )
        .header_right(
            text(format!("♥ {likes}"))
                .text_color(white)
                .text_size(px(17.0))
                .on_press(cx.listener(|_: &mut Gallery, _, cx| {
                    cx.update_global::<Likes, _>(|likes, _| likes.0 += 1);
                })),
        )
        .child(screen_page(vec![
            text(format!(
                "Header items are elements: the ♥ button updates the header ({likes})."
            ))
            .into_element(),
        ]))
}

fn locked(_: &mut RenderCx<'_, Gallery>) -> Screen {
    screen()
        .title("No swipe back")
        .gesture_enabled(false)
        .child(screen_page(vec![note(
            "The back gesture is off on this screen (no swipe on iOS, no predictive animation on Android); the back buttons still work.",
        )]))
}

/// The draft's input (the screen's `prevent_back` follows it).
#[derive(Debug, Default)]
struct DraftInput;

impl Component for DraftInput {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let draft = cx.global::<Draft>().0.clone();
        let editing = !draft.is_empty();
        view()
            .gap_3()
            .child(
                input()
                    .value(draft)
                    .placeholder("A draft")
                    .text_size(px(16.0))
                    .border_1()
                    .border_color(colors::border())
                    .rounded(px(8.0))
                    .p(px(10.0))
                    .on_change_text(cx.listener(|_: &mut Self, text: &str, cx| {
                        let text = text.to_owned();
                        cx.update_global::<Draft, _>(|draft, _| draft.0 = text);
                    })),
            )
            .child(note(if editing {
                "prevent_back is on"
            } else {
                "Nothing to lose: back pops"
            }))
    }
}

fn unsaved(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    let editing = !cx.global::<Draft>().0.is_empty();
    screen()
        .title("Unsaved changes")
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
                        cx.update_global::<Draft, _>(|draft, _| draft.0.clear());
                        nav::pop(cx);
                    })
                    .await;
                }
            })
            .detach();
        }))
        .child(screen_page(vec![
            note(
                "Type something, then go back (header button, swipe, Android back): the screen asks first.",
            ),
            DraftInput::element(()).into_element(),
        ]))
}

fn faded(_: &mut RenderCx<'_, Gallery>) -> Screen {
    screen()
        .title("Fade")
        .animation(ScreenAnimation::Fade)
        .child(screen_page(vec![note(
            "This screen faded in, and fades out.",
        )]))
}

fn instant(_: &mut RenderCx<'_, Gallery>) -> Screen {
    screen()
        .title("No animation")
        .animation(ScreenAnimation::None)
        .child(screen_page(vec![note("Shown without a transition.")]))
}

fn headerless(cx: &mut RenderCx<'_, Gallery>) -> Screen {
    screen().header_shown(false).child(
        SafeAreaView::element(SafeAreaViewProps::default())
            .flex_1()
            .bg(rgb(0xfff8c5))
            .p_4()
            .gap_3()
            .child(
                text("No header")
                    .text_size(px(24.0))
                    .font_weight(FontWeight::BOLD),
            )
            .child(insets())
            .child(
                button("Back", colors::accent())
                    .on_press(cx.listener(|_: &mut Gallery, _, cx| nav::pop(cx))),
            ),
    )
}

/// Level `depth` of a deep stack.
fn deep(depth: usize) -> impl Fn(&mut RenderCx<'_, Gallery>) -> Screen {
    move |cx| {
        let deeper = cx.listener(move |_: &mut Gallery, _, cx| {
            nav::push(cx, format!("deep-{}", depth + 1), deep(depth + 1));
        });
        screen()
            .title(format!("Level {depth}"))
            .child(screen_page(vec![
                button("Deeper", colors::accent())
                    .on_press(deeper)
                    .into_element(),
                button("Back to the index", colors::gray())
                    .on_press(cx.listener(|_: &mut Gallery, _, cx| nav::pop_to_top(cx)))
                    .into_element(),
            ]))
    }
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

/// Whether the gallery takes the Android back button (before the stack
/// pops), and how often it did: the gallery listens (`Gallery::setup`),
/// this demo switches it.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct BackIntercept {
    pub(crate) on: bool,
    pub(crate) count: u32,
}

/// The demo's page: buttons pushing the screens, its modals, and the
/// Android back button.
#[derive(Debug, Default)]
pub(crate) struct Navigation {
    sheet: Option<Presented>,
}

impl Component for Navigation {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let intercept = *cx.global::<BackIntercept>();
        type Make = fn(&mut RenderCx<'_, Gallery>) -> Screen;
        let screens: [(&str, &str, Make); 8] = [
            ("large-title", "Large title", large_title),
            ("transparent", "Transparent header", transparent),
            ("custom", "Custom header items and colors", custom_header),
            ("headerless", "No header", headerless),
            ("faded", "Fade in", faded),
            ("instant", "No animation", instant),
            ("locked", "No swipe back", locked),
            ("unsaved", "Unsaved changes (prevent_back)", unsaved),
        ];
        let mut pushes: Vec<AnyElement> = vec![insets()];
        pushes.extend(screens.into_iter().map(|(key, label, make)| {
            button(label, colors::accent())
                .on_press(cx.listener(move |_: &mut Self, _, cx| nav::push(cx, key, make)))
                .into_element()
        }));
        pushes.push(
            button("A deep stack", colors::accent())
                .on_press(cx.listener(|_: &mut Self, _, cx| {
                    let depth = nav::depth(cx);
                    nav::push(cx, format!("deep-{depth}"), deep(1));
                }))
                .into_element(),
        );
        let sheets = [
            (Presented::FullScreen, "Full screen (slide)"),
            (Presented::PageSheet, "Page sheet (iOS)"),
            (Presented::FormSheet, "Form sheet (iOS)"),
            (Presented::Fade, "Fade"),
            (Presented::Dialog, "Transparent dialog"),
            (
                Presented::Dismissible,
                "Dismissible sheet (swipe / back closes)",
            ),
        ]
        .map(|(sheet, label)| {
            button(label, colors::purple())
                .on_press(cx.listener(move |this: &mut Self, _, cx| {
                    this.sheet = Some(sheet);
                    cx.notify();
                }))
                .into_element()
        });
        let close = || {
            cx.listener(|this: &mut Self, _, cx| {
                this.sheet = None;
                cx.notify();
            })
        };
        let modal_content = |title: &str, detail: &str, close| {
            SafeAreaView::element(SafeAreaViewProps::default())
                .flex_1()
                .bg(colors::surface())
                .p_5()
                .gap_3()
                .child(
                    text(title.to_owned())
                        .text_size(px(24.0))
                        .font_weight(FontWeight::BOLD),
                )
                .child(insets())
                .child(note(detail))
                .child(button("Close", colors::accent()).on_press(close))
        };
        let modal_view = match self.sheet {
            Some(
                sheet @ (Presented::FullScreen | Presented::PageSheet | Presented::FormSheet | Presented::Fade),
            ) => {
                let (title, presentation) = match sheet {
                    Presented::PageSheet => ("pageSheet", ModalPresentation::PageSheet),
                    Presented::FormSheet => ("formSheet", ModalPresentation::FormSheet),
                    Presented::Fade => ("fade", ModalPresentation::FullScreen),
                    _ => ("fullScreen", ModalPresentation::FullScreen),
                };
                Some(
                    modal()
                        .presentation(presentation)
                        .when(sheet == Presented::Fade, |m| m.animation(ModalAnimation::Fade))
                        .on_request_close(close())
                        .child(modal_content(
                            title,
                            "Close with the button, the back button (Android) or a swipe down (iOS sheets).",
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
                    .child(modal_content(
                        "Dismissible",
                        "Swipe it down (iOS) or press back (Android): it closes natively, then tells the app.",
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
                            .bg(rgba(0x0000_0066))
                            .items_center()
                            .justify_center()
                            .p_8()
                            .child(
                                view()
                                    .self_stretch()
                                    .bg(colors::surface())
                                    .rounded(px(14.0))
                                    .p_5()
                                    .gap_3()
                                    .child(
                                        text("A dialog")
                                            .text_size(px(20.0))
                                            .font_weight(FontWeight::BOLD),
                                    )
                                    .child(note("A transparent modal shows what is behind it."))
                                    .child(button("OK", colors::accent()).on_press(close())),
                            ),
                    ),
            ),
            None => None,
        };
        view()
            .flex_1()
            .child(page([
                section("Screens", pushes).into_element(),
                section("Modals", sheets).into_element(),
                section(
                    "Back button (Android)",
                    [view()
                        .flex_row()
                        .items_center()
                        .gap_3()
                        .child(
                            text(if intercept.on {
                                format!(
                                    "On the index, back is intercepted ({} times so far)",
                                    intercept.count
                                )
                            } else {
                                "Back pops screens, then leaves the app".to_owned()
                            })
                            .flex_1()
                            .text_color(colors::text()),
                        )
                        .child(
                            button(
                                if intercept.on { "Release" } else { "Intercept" },
                                colors::gray(),
                            )
                            .on_press(cx.listener(
                                |_: &mut Self, _, cx| {
                                    cx.update_global::<BackIntercept, _>(|b, _| b.on = !b.on);
                                },
                            )),
                        )
                        .into_element()],
                )
                .into_element(),
            ]))
            .children(modal_view)
    }
}
