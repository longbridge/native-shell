//! Styles: corners (also continuous), borders, shadows, clipping, 3D
//! transforms (as the TSX gallery's `demos/Styles.tsx`).

use nsh_core::ui::{TransformKind, prelude::*};

fn card() -> View {
    view()
        .w(px(150.0))
        .h(px(110.0))
        .items_center()
        .justify_center()
        .bg(rgb(0xffffff))
}

fn label(content: impl Into<SharedString>) -> Text {
    text(content)
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(0x1f2328))
}

/// A label under a card's bottom edge.
fn label_below(content: &'static str) -> View {
    view().absolute().bottom(px(-18.0)).child(label(content))
}

/// Two faces back to back: each hides while turned away.
#[derive(Debug, Default)]
struct FlipCard {
    angle: Option<NativeValue>,
    flipped: bool,
}

impl Component for FlipCard {
    const NAME: &'static str = "gallery.styles.flip_card";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.angle = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let flip =
            view()
                .w(px(150.0))
                .h(px(110.0))
                .on_press(cx.listener(|this: &mut Self, _, _| {
                    this.flipped = !this.flipped;
                    if let Some(angle) = &this.angle {
                        let to = if this.flipped { 180.0 } else { 0.0 };
                        let _ = angle.timing(to, 500.0, Easing::EaseInOut);
                    }
                }));
        let Some(angle) = &self.angle else {
            return flip;
        };
        // The bound rotation follows the static perspective.
        let face = |color: u32| {
            card()
                .absolute()
                .left(px(0.0))
                .top(px(0.0))
                .rounded(px(14.0))
                .backface_hidden()
                .bg(rgb(color))
                .perspective(600.0)
        };
        flip.child(
            face(0xddf4ff)
                .bind_transform(TransformKind::RotateY, angle)
                .child(label("Front (tap)")),
        )
        .child(
            face(0xfff8c5)
                .bind_transform(
                    TransformKind::RotateY,
                    &angle.interpolate(&[0.0, 180.0], &[180.0, 360.0]),
                )
                .child(label("Back: backfaceVisibility")),
        )
    }
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Styles {
    /// The yellow card shows what overflows it.
    unclipped: bool,
    /// The square of the zIndex card on top.
    front: usize,
}

impl Styles {
    fn clipping(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        card()
            .bg(rgb(0xfff8c5))
            .rounded(px(24.0))
            .border_2()
            .border_color(rgb(0x9a6700))
            .when(!self.unclipped, |v| v.overflow_hidden())
            .shadow_opacity(0.3)
            .shadow_radius(8.0)
            .shadow_offset(0.0, 4.0)
            .on_press(cx.listener(|this: &mut Self, _, cx| {
                this.unclipped = !this.unclipped;
                cx.notify();
            }))
            .child(
                view()
                    .absolute()
                    .left(px(-30.0))
                    .top(px(-30.0))
                    .size(px(90.0))
                    .rounded(px(45.0))
                    .bg(rgb(0xbf3989)),
            )
            .child(label(if self.unclipped {
                "Visible (tap)"
            } else {
                "Clipped (tap)"
            }))
            .into_element()
    }

    fn stacking(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let squares = [0xfd8c73, 0x54aeff, 0x4ac26b]
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                view()
                    .absolute()
                    .left(px(20.0 + i as f32 * 30.0))
                    .top(px(10.0 + i as f32 * 22.0))
                    .w(px(70.0))
                    .h(px(56.0))
                    .rounded(px(10.0))
                    .bg(rgb(color))
                    .z_index(if self.front == i { 10 } else { 0 })
                    .shadow_opacity(0.2)
                    .shadow_radius(4.0)
                    .shadow_offset(0.0, 2.0)
            });
        card()
            .bg(Color::TRANSPARENT)
            .on_press(cx.listener(|this: &mut Self, _, cx| {
                this.front = (this.front + 1) % 3;
                cx.notify();
            }))
            .children(squares)
            .child(label_below("zIndex (tap)"))
            .into_element()
    }

    fn tilted() -> [AnyElement; 3] {
        let card = |color: u32, content: &'static str| card().bg(rgb(color)).child(label(content));
        [
            card(0xddf4ff, "rotateY + perspective")
                .perspective(400.0)
                .rotate_y(35.0)
                .into_element(),
            card(0xfbefff, "rotateX")
                .perspective(400.0)
                .rotate_x(40.0)
                .into_element(),
            card(0xfff1e5, "skewX (iOS)").skew_x(-12.0).into_element(),
        ]
    }

    /// Rows, `marginStart` and text under a right-to-left layout.
    fn right_to_left(cx: &RenderCx<'_, Self>) -> AnyElement {
        let boxes = [(0xfd8c73, "1"), (0x54aeff, "2"), (0x4ac26b, "3")]
            .into_iter()
            .enumerate()
            .map(|(i, (color, n))| {
                view()
                    .size(px(44.0))
                    .rounded(px(8.0))
                    .rounded_ss(px(22.0))
                    .items_center()
                    .justify_center()
                    .bg(rgb(color))
                    .when(i == 0, |v| v.ms(px(16.0)))
                    .child(
                        text(n)
                            .text_color(rgb(0xffffff))
                            .font_weight(FontWeight::BOLD),
                    )
            });
        let tile = |n: usize, color: u32| {
            view()
                .size(px(40.0))
                .rounded(px(6.0))
                .bg(rgb(color))
                .items_center()
                .justify_center()
                .child(text(n.to_string()))
        };
        let muted = rgb(0x6e7781);
        view()
            .w_full()
            .gap(px(10.0))
            .p(px(14.0))
            .bg(rgb(0xffffff))
            .rounded(px(12.0))
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        text(if cx.rtl() {
                            "Right to left"
                        } else {
                            "Left to right"
                        })
                        .flex_1()
                        .font_weight(FontWeight::SEMIBOLD),
                    )
                    .child(
                        text("Flip")
                            .text_color(rgb(0xffffff))
                            .bg(rgb(0x0969da))
                            .px_3()
                            .py(px(6.0))
                            .rounded(px(8.0))
                            .overflow_hidden()
                            .on_press(cx.listener(|_: &mut Self, _, cx| {
                                let rtl = cx.environment().rtl;
                                cx.force_rtl(!rtl);
                            })),
                    ),
            )
            .child(view().flex_row().gap_2().children(boxes))
            .child(text(
                "Text aligns to the start side; the first box has marginStart 16 and a round top-start corner.",
            ))
            .child(
                text("A horizontal scroll view and a horizontal list start at the start side:")
                    .text_size(px(13.0))
                    .text_color(muted),
            )
            .child(
                scroll()
                    .horizontal()
                    .shows_indicator(false)
                    .h(px(40.0))
                    .child(
                        view()
                            .flex_row()
                            .gap(px(6.0))
                            .px_1()
                            .children((1..=12).map(|n| tile(n, 0xddf4ff))),
                    ),
            )
            .child(
                virtual_list(30, move |i, _| tile(i + 1, 0xfff8c5).me(px(6.0)))
                    .horizontal()
                    .estimated_item_size(46.0)
                    .shows_indicator(false)
                    .h(px(40.0)),
            )
            .child(
                text("textAlign left (the start side right to left)")
                    .text_left()
                    .text_color(muted),
            )
            .child(
                text("مرحبا بالعالم — writingDirection rtl")
                    .writing_direction(WritingDirection::Rtl)
                    .text_color(muted),
            )
            .into_element()
    }
}

impl Component for Styles {
    const NAME: &'static str = "gallery.styles";
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let purple = rgb(0x8250df);
        let blue = rgb(0x0969da);
        let green = rgb(0x1a7f37);
        let figures = view()
            .flex_row()
            .flex_wrap()
            .gap_6()
            .p_6()
            .justify_center()
            .child(
                card()
                    .rounded(px(16.0))
                    .shadow_opacity(0.18)
                    .shadow_radius(12.0)
                    .shadow_offset(0.0, 6.0)
                    .child(label("Shadow")),
            )
            .child(
                card()
                    .rounded_t(px(28.0))
                    .border_2()
                    .border_color(purple)
                    .child(label("Top corners")),
            )
            .child(
                card()
                    .bg(rgb(0xddf4ff))
                    .rounded_tl(px(44.0))
                    .rounded_tr(px(6.0))
                    .rounded_br(px(28.0))
                    .border(3.0)
                    .border_color(blue)
                    .shadow_color(blue)
                    .shadow_opacity(0.35)
                    .shadow_radius(10.0)
                    .shadow_offset(4.0, 8.0)
                    .child(label("Mixed radii")),
            )
            .child(self.clipping(cx))
            .child(
                image("https://picsum.photos/seed/nsh/300/220")
                    .w(px(150.0))
                    .h(px(110.0))
                    .rounded_tl(px(36.0))
                    .rounded_br(px(36.0))
                    .bg(rgb(0xe1e4e8))
                    .shadow_opacity(0.3)
                    .shadow_radius(10.0)
                    .shadow_offset(0.0, 6.0),
            )
            // A transparent view's shadow follows its content: the text.
            .child(
                card()
                    .bg(Color::TRANSPARENT)
                    .shadow_opacity(0.5)
                    .shadow_radius(3.0)
                    .shadow_offset(2.0, 2.0)
                    .child(
                        text("Text shadow")
                            .text_size(px(22.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x1f2328)),
                    ),
            )
            .child(
                view().w(px(150.0)).items_center().child(
                    text("Text corners")
                        .text_size(px(15.0))
                        .text_color(rgb(0xffffff))
                        .bg(green)
                        .rounded_tl(px(14.0))
                        .rounded_br(px(14.0))
                        .px_3()
                        .py(px(6.0)),
                ),
            )
            .child(
                card()
                    .border_t(6.0)
                    .border_t_color(rgb(0xcf222e))
                    .border_r(2.0)
                    .border_r_color(green)
                    .border_b(4.0)
                    .border_b_color(blue)
                    .border_l(1.0)
                    .border_l_color(purple)
                    .child(label("Per-side borders")),
            )
            .child(
                card()
                    .gap_2()
                    .child(
                        view()
                            .w(px(120.0))
                            .h(px(30.0))
                            .rounded(px(8.0))
                            .border_2()
                            .border_color(blue)
                            .border_dashed(),
                    )
                    .child(
                        view()
                            .w(px(120.0))
                            .h(px(30.0))
                            .rounded(px(8.0))
                            .border(3.0)
                            .border_color(rgb(0xbf3989))
                            .border_dotted(),
                    )
                    .child(label("Dashed / dotted")),
            )
            .child(
                card()
                    .w(px(324.0))
                    .gap_2()
                    .child(
                        view()
                            .flex_row()
                            .gap_3()
                            .child(view().size(px(64.0)).rounded(px(20.0)).bg(purple))
                            .child(
                                view()
                                    .size(px(64.0))
                                    .rounded(px(20.0))
                                    .border_curve(BorderCurve::Continuous)
                                    .bg(purple),
                            )
                            .child(
                                view()
                                    .size(px(64.0))
                                    .rounded(px(20.0))
                                    .border_curve(BorderCurve::Continuous)
                                    .border(3.0)
                                    .border_color(blue)
                                    .rounded_tl(px(4.0)),
                            )
                            .child(
                                view()
                                    .w(px(64.0))
                                    .h(px(30.0))
                                    .rounded(px(15.0))
                                    .border_curve(BorderCurve::Continuous)
                                    .bg(green),
                            ),
                    )
                    .child(label("Circular / continuous corners")),
            )
            .child(self.stacking(cx))
            .children(Self::tilted())
            .child(
                card().bg(rgb(0x1f2328)).opacity(0.45).child(
                    text("opacity 0.45")
                        .text_color(rgb(0xffffff))
                        .font_weight(FontWeight::SEMIBOLD),
                ),
            )
            .child(
                card()
                    .rounded(px(16.0))
                    .elevation(10.0)
                    .child(label("elevation 10 (Android)")),
            )
            .child(
                card()
                    .bg(Color::TRANSPARENT)
                    .child(
                        view()
                            .size(px(60.0))
                            .border_1()
                            .border_color(rgb(0xd0d7de))
                            .border_dashed()
                            .absolute()
                            .left(px(45.0))
                            .top(px(15.0)),
                    )
                    .child(
                        view()
                            .size(px(60.0))
                            .absolute()
                            .left(px(45.0))
                            .top(px(15.0))
                            .bg(rgba(0x8250_df99))
                            .origin_top_left()
                            .rotate(30.0),
                    )
                    .child(label_below("transformOrigin top left")),
            )
            .child(FlipCard::element(()))
            .child(
                card()
                    .items_start()
                    .px_4()
                    .gap(px(2.0))
                    .child(
                        text("Proportional / tabular")
                            .text_size(px(13.0))
                            .text_color(rgb(0x6e7781)),
                    )
                    .children(
                        ["1,111.11", "8,888.88"]
                            .map(|n| text(n).text_size(px(18.0)))
                            .into_iter()
                            .chain(
                                ["1,111.11", "8,888.88"]
                                    .map(|n| text(n).text_size(px(18.0)).tabular_nums()),
                            ),
                    ),
            )
            .child(
                card()
                    .child(text("Small Caps").text_size(px(18.0)).small_caps())
                    .child(
                        text("Oldstyle 1234567890")
                            .text_size(px(18.0))
                            .oldstyle_nums(),
                    ),
            )
            .child(Self::right_to_left(cx));
        view().flex_1().child(scroll().flex_1().child(figures))
    }
}
