//! Typed styles: What Tailwind classes do, as typed style methods: variants, grid, shadows, gradients, motion (as the TSX gallery's `demos/Tailwind.tsx`).
//!
//! The same cards as the Tailwind demo, written with `Styled`'s methods
//! instead of class strings; each card says how its classes map. Colors are
//! Tailwind's palette (`palette::SKY.shade(500)`); `dark:` colors are
//! [`AdaptiveColor`]s (a light and a dark value, switched natively without a
//! render), `dark:` styles of anything else `.dark(|s| ..)`.

use nsh_core::ui::{
    BlendMode,
    palette::{
        AMBER, BLUE, EMERALD, FUCHSIA, GREEN, Hue, INDIGO, ORANGE, PINK, RED, ROSE, SKY, VIOLET,
        WHITE, YELLOW, ZINC,
    },
    prelude::*,
};

use crate::assets::LOGO;

/// The photo of the filters' card.
const PHOTO: &str = "https://picsum.photos/seed/nsh-tailwind/600/400";

/// The TSX gallery's `brand` color (its `nsh.json` Tailwind theme).
const BRAND_50: Color = rgb(0xf5f3ff);
const BRAND_500: Color = rgb(0x7c3aed);
const BRAND_600: Color = rgb(0x6d28d9);
const BRAND_700: Color = rgb(0x5b21b6);

/// The TSX gallery's `rounded-card` radius.
const CARD_RADIUS: f32 = 14.0;

/// `light`, and `dark` in dark appearance (Tailwind's `dark:`).
fn adaptive(light: Color, dark: Color) -> AdaptiveColor {
    AdaptiveColor::new(light, dark)
}

/// `text-zinc-500 dark:text-zinc-400`.
fn secondary() -> AdaptiveColor {
    adaptive(ZINC.shade(500), ZINC.shade(400))
}

/// A titled card: the Tailwind demo's `Section`, with a line on how its
/// classes map to typed methods.
fn card(title: &'static str, how: &'static str, body: impl IntoElement) -> AnyElement {
    view()
        .gap_2()
        .child(
            text(title)
                .uppercase()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                // `tracking-wide`: 0.025em.
                .letter_spacing(0.3)
                .text_color(secondary()),
        )
        .child(
            view()
                .gap_3()
                .rounded(px(CARD_RADIUS))
                .bg(adaptive(WHITE, ZINC.shade(800)))
                .p_4()
                .child(body)
                .child(text(how).text_xs().text_color(secondary())),
        )
        .into_element()
}

/// A caption under a sample.
fn label(content: impl Into<SharedString>) -> Text {
    text(content.into())
        .text_center()
        .text_size(px(11.0))
        .text_color(secondary())
}

/// A sample with its caption under it.
fn captioned(sample: impl IntoElement, caption: &'static str) -> AnyElement {
    view()
        .gap_1()
        .child(sample)
        .child(label(caption))
        .into_element()
}

/// The demo's page: the Tailwind demo's cards in typed styles.
#[derive(Default)]
pub(crate) struct TypedStyles {
    /// The first button's presses.
    count: u32,
    /// The third button is disabled.
    busy: bool,
}

impl Component for TypedStyles {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let cards = [
            card(
                "Buttons: active, hover, transition, disabled",
                ".active(|s| s.bg(..)), .hover(..), .transition_colors().duration(150.0), \
                 .active(|s| s.scale(0.95)); .disabled(b) with .disabled_style(|s| \
                 s.opacity(0.4)); .text_decoration_dotted().underline_offset(4.0).",
                self.buttons(cx),
            ),
            card(
                "Cards: shadow, ring, group, gradient",
                ".shadow_lg().ring(1.0).ring_color(..), .group() and .group_active(|s| ..) \
                 inside; .bg_linear_to_br().gradient_from(..).gradient_via(..).gradient_to(..), \
                 .shadow_xl().shadow_color(brand-500 at 40%).",
                cards(),
            ),
            card(
                "Flex, grid, breakpoints",
                ".flex_1(), .flex_grow(2.0), .w_10(); .grid().grid_cols(3), .col_span(2), \
                 .row_span(2); sm: reads the window (.when(cx.window().width >= 640.0, |v| \
                 v.flex_row())).",
                layout(cx.window().width),
            ),
            card(
                "Palette",
                ".bg(palette::SKY.shade(500)): Tailwind's palette as typed colors.",
                colors(),
            ),
            card(
                "Shadows, rings, outlines",
                ".shadow_xs() to .shadow_2xl(), .inset_shadow_sm(), .ring(2.0).ring_offset(2.0), \
                 .outline(2.0).outline_offset(2.0).outline_dashed(), .shadow_color(..).",
                shadows(),
            ),
            card(
                "Gradients",
                ".bg_linear_to_r(), .bg_linear(45.0), .bg_radial(), .bg_conic(0.0) with \
                 .gradient_from(..), .gradient_via(..), .gradient_to(..).",
                gradients(),
            ),
            card(
                "Typography",
                ".text_3xl(), .font_weight(..), .italic(), .font_family(\"serif\"), \
                 .tabular_nums(), .underline().text_decoration_wavy(), .uppercase(), \
                 .number_of_lines(..), .text_shadow_md().",
                typography(),
            ),
            card(
                "Filters, backdrop blur, blend modes",
                ".grayscale(1.0), .sepia(1.0), .blur(8.0), .brightness(0.5); \
                 .backdrop_blur(12.0); .mix_blend_mode(BlendMode::Multiply).",
                effects(),
            ),
            card(
                "Animations and transforms",
                ".animate_spin(), .animate_ping(), .animate_pulse(), .animate_bounce() (native \
                 keyframes); .rotate(12.0).skew_x(6.0).scale(0.9).",
                motion(),
            ),
            card(
                "first:, even:, last:",
                "Plain Rust from the row's index (.when(i % 2 == 1, ..)); .active(|s| s.bg(..)).",
                lists(),
            ),
            card(
                "Inputs",
                ".focus(|s| s.border_color(..).ring(2.0).ring_color(..)), .placeholder_color(..), \
                 .selection_color(..) (caret).",
                inputs(cx),
            ),
        ];
        view().flex_1().child(
            scroll()
                .flex_1()
                .bg(adaptive(ZINC.shade(100), ZINC.shade(900)))
                .child(view().gap_5().p_4().children(cards)),
        )
    }
}

impl TypedStyles {
    /// Press states, hover, transitions, disabled.
    fn buttons(&self, cx: &mut RenderCx<'_, Self>) -> AnyElement {
        let busy = self.busy;
        view()
            .gap_3()
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .child(
                        view()
                            .rounded_lg()
                            .bg(BRAND_600)
                            .px_4()
                            .py_2()
                            .transition_colors()
                            .duration(150.0)
                            .active(|s| s.bg(BRAND_700))
                            .hover(|s| s.bg(BRAND_500))
                            .on_press(cx.listener(|this: &mut Self, _, cx| {
                                this.count += 1;
                                cx.notify();
                            }))
                            .child(
                                text(format!("Pressed {}×", self.count))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(WHITE),
                            ),
                    )
                    .child(
                        view()
                            .rounded_lg()
                            .border_1()
                            .border_color(adaptive(ZINC.shade(300), ZINC.shade(600)))
                            .px_4()
                            .py_2()
                            .transition()
                            .active(|s| {
                                s.scale(0.95).bg(adaptive(ZINC.shade(100), ZINC.shade(700)))
                            })
                            .on_press(cx.listener(|this: &mut Self, _, cx| {
                                this.count = 0;
                                cx.notify();
                            }))
                            .child(
                                text("Reset")
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(adaptive(ZINC.shade(900), WHITE)),
                            ),
                    )
                    .child(
                        view()
                            .rounded_full()
                            .bg(EMERALD.shade(500))
                            .px_4()
                            .py_2()
                            .disabled(busy)
                            .disabled_style(|s| s.opacity(0.4))
                            .active(|s| s.bg(EMERALD.shade(600)))
                            .on_press(cx.listener(|this: &mut Self, _, cx| {
                                this.busy = !this.busy;
                                cx.notify();
                            }))
                            .child(
                                text(if busy { "Disabled" } else { "Enabled" })
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(WHITE),
                            ),
                    ),
            )
            .child(
                view()
                    .self_start()
                    .rounded_md()
                    .px_2()
                    .py_1()
                    .active(|s| s.bg(adaptive(ZINC.shade(100), ZINC.shade(700))))
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        this.busy = false;
                        cx.notify();
                    }))
                    .child(
                        text("Enable again")
                            .text_sm()
                            .text_color(BRAND_600)
                            .underline()
                            .text_decoration_dotted()
                            .underline_offset(4.0),
                    ),
            )
            .into_element()
    }
}

/// A card whose line follows its press (a group), and a gradient card.
fn cards() -> AnyElement {
    let group = view()
        .group()
        .flex_row()
        .items_center()
        .gap_3()
        .rounded_2xl()
        .bg(adaptive(WHITE, ZINC.shade(700)))
        .p_3()
        .shadow_lg()
        .ring(1.0)
        .ring_color(rgba(0x0000_000d))
        .transition_transform()
        .duration(200.0)
        .active(|s| s.scale(0.98))
        .child(image(LOGO).size_12().rounded_xl())
        .child(
            view()
                .flex_1()
                .gap_0p5()
                .child(
                    text("Group press")
                        .text_base()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(adaptive(ZINC.shade(900), WHITE)),
                )
                .child(
                    text("Press the card: this line changes color")
                        .text_sm()
                        .text_color(adaptive(ZINC.shade(500), ZINC.shade(300)))
                        .group_active(|s| s.text_color(BRAND_600)),
                ),
        )
        .child(
            text("›")
                .text_2xl()
                .text_color(ZINC.shade(300))
                .transition_transform()
                .group_active(|s| s.translate_x(4.0)),
        );
    let gradient = view()
        .overflow_hidden()
        .rounded_2xl()
        .bg_linear_to_br()
        .gradient_from(BRAND_500)
        .gradient_via(FUCHSIA.shade(500))
        .gradient_to(ORANGE.shade(400))
        .p_5()
        .shadow_xl()
        .shadow_color(BRAND_500.with_alpha(0.4))
        .child(
            text("Gradient card")
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(WHITE),
        )
        .child(
            text(".bg_linear_to_br() from brand-500 via fuchsia-500 to orange-400")
                .mt_1()
                .text_sm()
                .text_color(WHITE.with_alpha(0.8)),
        );
    view().gap_3().child(group).child(gradient).into_element()
}

/// Flex rows, a grid and a breakpoint. `width` is the window's.
fn layout(width: f32) -> AnyElement {
    let cell = |shade: u16| view().rounded_md().bg(VIOLET.shade(shade));
    view()
        .gap_3()
        .child(
            view()
                .flex_row()
                .gap_2()
                .child(view().h_10().flex_1().rounded_md().bg(SKY.shade(200)))
                .child(
                    view()
                        .h_10()
                        .flex_1()
                        .flex_grow(2.0)
                        .rounded_md()
                        .bg(SKY.shade(400)),
                )
                .child(view().h_10().w_10().rounded_md().bg(SKY.shade(600))),
        )
        .child(
            view()
                .grid()
                .grid_cols(3)
                .gap_2()
                .child(
                    cell(200)
                        .col_span(2)
                        .h_12()
                        .items_center()
                        .justify_center()
                        .child(text("col-span-2").text_xs().text_color(VIOLET.shade(900))),
                )
                .child(cell(300).h_12())
                .child(cell(400).h_12())
                .child(cell(500).row_span(2))
                .child(cell(600).h_12())
                .child(cell(700).h_12())
                .child(cell(800).h_12()),
        )
        .child(
            view()
                .flex_col()
                .gap_2()
                .when(width >= 640.0, |v| v.flex_row())
                .child(view().h_8().flex_1().rounded_md().bg(AMBER.shade(300)))
                .child(view().h_8().flex_1().rounded_md().bg(AMBER.shade(500))),
        )
        .child(label(
            "The last row is a column on phones and a row from `sm:` (640pt) up.",
        ))
        .into_element()
}

/// Five shades of eight colors.
fn colors() -> AnyElement {
    let hues: [Hue; 8] = [RED, ORANGE, AMBER, EMERALD, SKY, INDIGO, PINK, ZINC];
    view()
        .gap_1()
        .children(hues.map(|hue| {
            view()
                .flex_row()
                .gap_1()
                .child(
                    text(hue.name())
                        .w_16()
                        .text_xs()
                        .text_color(ZINC.shade(500)),
                )
                .children(
                    [100, 300, 500, 700, 900]
                        .map(|shade| view().h_6().flex_1().rounded_sm().bg(hue.shade(shade))),
                )
        }))
        .into_element()
}

/// The shadow presets, an inset shadow, a ring, an outline, a colored
/// shadow.
fn shadows() -> AnyElement {
    type Shadow = fn(View) -> View;
    let sizes: [(&str, Shadow); 6] = [
        ("xs", |v| v.shadow_xs()),
        ("sm", |v| v.shadow_sm()),
        ("md", |v| v.shadow_md()),
        ("lg", |v| v.shadow_lg()),
        ("xl", |v| v.shadow_xl()),
        ("2xl", |v| v.shadow_2xl()),
    ];
    let tile = || view().size_14().rounded_xl();
    view()
        .gap_4()
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .justify_between()
                .gap_y_5()
                .px_1()
                .py_2()
                .children(sizes.map(|(name, shadow)| {
                    captioned(shadow(tile().bg(adaptive(WHITE, ZINC.shade(600)))), name)
                })),
        )
        .child(
            view()
                .flex_row()
                .justify_between()
                .px_1()
                .child(captioned(
                    tile().bg(ZINC.shade(100)).inset_shadow_sm(),
                    "inset",
                ))
                .child(captioned(
                    tile()
                        .bg(WHITE)
                        .ring(2.0)
                        .ring_color(SKY.shade(500))
                        .ring_offset(2.0),
                    "ring",
                ))
                .child(captioned(
                    tile()
                        .rounded_full()
                        .bg(WHITE)
                        .outline(2.0)
                        .outline_offset(2.0)
                        .outline_dashed()
                        .outline_color(PINK.shade(500)),
                    "outline",
                ))
                .child(captioned(
                    tile()
                        .bg(EMERALD.shade(500))
                        .shadow_lg()
                        .shadow_color(EMERALD.shade(500).with_alpha(0.5)),
                    "colored",
                )),
        )
        .into_element()
}

/// Linear, angled, radial and conic gradients.
fn gradients() -> AnyElement {
    let tile = || view().size_16().rounded_xl();
    view()
        .flex_row()
        .justify_between()
        .child(
            tile()
                .bg_linear_to_r()
                .gradient_from(SKY.shade(400))
                .gradient_to(BLUE.shade(600)),
        )
        .child(
            tile()
                .bg_linear(45.0)
                .gradient_from(ROSE.shade(400))
                .gradient_via(FUCHSIA.shade(500))
                .gradient_to(INDIGO.shade(500)),
        )
        .child(
            tile()
                .bg_radial()
                .gradient_from(YELLOW.shade(200))
                .gradient_to(ORANGE.shade(500)),
        )
        .child(
            tile()
                .bg_conic(0.0)
                .gradient_from(RED.shade(500))
                .gradient_via(GREEN.shade(500))
                .gradient_to(RED.shade(500)),
        )
        .into_element()
}

/// Sizes, weights, families, decorations, truncation, a text shadow.
fn typography() -> AnyElement {
    let body = adaptive(ZINC.shade(700), ZINC.shade(200));
    view()
        .gap_2()
        .child(
            text("Display")
                .text_3xl()
                .font_weight(FontWeight::BLACK)
                // `tracking-tight`: -0.025em.
                .letter_spacing(-0.75)
                .text_color(adaptive(ZINC.shade(900), WHITE)),
        )
        .child(
            text("Light, large and relaxed")
                .text_lg()
                .font_weight(FontWeight::LIGHT)
                .text_color(body),
        )
        .child(
            text("Serif italic")
                .font_family("serif")
                .text_lg()
                .italic()
                .text_color(body),
        )
        .child(
            text("font-mono 0123456789")
                .font_family("monospace")
                .text_sm()
                .tabular_nums()
                .text_color(body),
        )
        .child(
            view()
                .flex_row()
                .flex_wrap()
                .gap_x_4()
                .gap_y_1()
                .text_color(adaptive(ZINC.shade(800), ZINC.shade(100)))
                .child(
                    text("wavy")
                        .underline()
                        .text_decoration_wavy()
                        .text_decoration_color(RED.shade(500)),
                )
                .child(
                    text("double")
                        .underline()
                        .text_decoration_double()
                        .text_decoration_color(SKY.shade(500)),
                )
                .child(
                    text("dashed")
                        .underline()
                        .text_decoration_dashed()
                        .text_decoration_thickness(2.0),
                )
                .child(
                    text("struck")
                        .line_through()
                        .text_decoration_color(ZINC.shade(400)),
                )
                .child(
                    text("caps")
                        .uppercase()
                        // `tracking-widest`: 0.1em.
                        .letter_spacing(1.2)
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(BRAND_600),
                ),
        )
        .child(
            text("number_of_lines(1): a single line that is far too long to fit in the card cut with an ellipsis")
                .number_of_lines(1)
                .text_sm()
                .text_color(ZINC.shade(500)),
        )
        .child(
            text(
                "number_of_lines(2): Text is laid out once on the engine thread and drawn on the main thread; typed styles reach it through the same style path as everything else, so the third line of this paragraph never shows.",
            )
            .number_of_lines(2)
            .text_sm()
            // `leading-relaxed`: 1.625 × 14pt.
            .line_height(px(22.75))
            .text_color(ZINC.shade(500)),
        )
        .child(
            text("text-shadow-md")
                .text_shadow_md()
                .text_xl()
                .font_weight(FontWeight::BOLD)
                .text_color(WHITE)
                .bg(SKY.shade(500))
                .rounded_md()
                .px_2()
                .self_start(),
        )
        .into_element()
}

/// Filters on a photo, backdrop blur and blending.
fn effects() -> AnyElement {
    let photo = || image(PHOTO).size_14().rounded_lg();
    view()
        .gap_3()
        .child(
            view()
                .flex_row()
                .justify_between()
                .child(captioned(photo(), "none"))
                .child(captioned(photo().grayscale(1.0), "grayscale"))
                .child(captioned(photo().sepia(1.0), "sepia"))
                .child(captioned(photo().blur(8.0), "blur-sm"))
                .child(captioned(photo().brightness(0.5), "brightness-50")),
        )
        .child(
            view()
                .h_28()
                .overflow_hidden()
                .rounded_xl()
                .child(image(PHOTO).absolute().inset_0().size_full())
                .child(
                    view()
                        .absolute()
                        .left_3()
                        .right_3()
                        .bottom_3()
                        .rounded_lg()
                        .bg(WHITE.with_alpha(0.4))
                        .px_3()
                        .py_2()
                        .backdrop_blur(12.0)
                        .child(
                            text("backdrop-blur-md bg-white/40")
                                .text_sm()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(ZINC.shade(900)),
                        ),
                )
                .child(
                    view()
                        .absolute()
                        .right_3()
                        .top_3()
                        .size_10()
                        .rounded_full()
                        .bg(YELLOW.shade(300))
                        .mix_blend_mode(BlendMode::Multiply),
                ),
        )
        .into_element()
}

/// Spinning, pinging, pulsing and bouncing, and a fixed transform.
fn motion() -> AnyElement {
    let sample = |shape: AnyElement, caption: &'static str| {
        view()
            .gap_2()
            .items_center()
            .child(shape)
            .child(label(caption))
    };
    view()
        .flex_row()
        .items_center()
        .justify_between()
        .px_2()
        .py_3()
        .child(sample(
            view()
                .size_8()
                .rounded_md()
                .bg(BRAND_500)
                .animate_spin()
                .into_element(),
            "spin",
        ))
        .child(sample(
            view()
                .size_8()
                .items_center()
                .justify_center()
                .child(
                    view()
                        .absolute()
                        .size_4()
                        .rounded_full()
                        .bg(SKY.shade(400))
                        .animate_ping(),
                )
                .child(view().size_4().rounded_full().bg(SKY.shade(500)))
                .into_element(),
            "ping",
        ))
        .child(sample(
            view()
                .h_8()
                .w_14()
                .rounded_md()
                .bg(ZINC.shade(300))
                .animate_pulse()
                .into_element(),
            "pulse",
        ))
        .child(sample(
            view()
                .size_8()
                .rounded_full()
                .bg(ORANGE.shade(400))
                .animate_bounce()
                .into_element(),
            "bounce",
        ))
        .child(sample(
            view()
                .size_8()
                .rotate(12.0)
                .skew_x(6.0)
                .scale(0.9)
                .rounded_md()
                .bg(EMERALD.shade(500))
                .into_element(),
            "transform",
        ))
        .into_element()
}

/// Rows styled by their place.
fn lists() -> AnyElement {
    let items = ["Inbox", "Drafts", "Sent", "Archive", "Trash"];
    let count = items.len();
    let border = adaptive(ZINC.shade(200), ZINC.shade(700));
    view()
        .overflow_hidden()
        .rounded_xl()
        .border_1()
        .border_color(border)
        .children(items.iter().enumerate().map(|(i, item)| {
            let (first, last, even) = (i == 0, i + 1 == count, i % 2 == 1);
            view()
                .flex_row()
                .items_center()
                .justify_between()
                .when(!first, |v| v.border_t_1().border_color(border))
                .px_3()
                .py_2p5()
                .when(even, |v| v.bg(adaptive(ZINC.shade(50), ZINC.shade(900))))
                .when(last, |v| v.bg(adaptive(ROSE.shade(50), ROSE.shade(950))))
                .active(|s| s.bg(BRAND_50))
                .child(text(*item).text_color(adaptive(ZINC.shade(800), ZINC.shade(100))))
                .child(text("›").text_xs().text_color(ZINC.shade(400)))
        }))
        .into_element()
}

/// An input with focus, placeholder and caret colors.
fn inputs(cx: &mut RenderCx<'_, TypedStyles>) -> AnyElement {
    view()
        .gap_2()
        .child(
            input()
                .placeholder("Focus me")
                .placeholder_color(cx.color(ZINC.shade(400)))
                .selection_color(BRAND_600)
                .rounded_lg()
                .border_1()
                .border_color(adaptive(ZINC.shade(300), ZINC.shade(600)))
                .px_3()
                .py_2()
                .text_base()
                .text_color(adaptive(ZINC.shade(900), WHITE))
                .focus(|s| {
                    s.border_color(BRAND_500)
                        .ring(2.0)
                        .ring_color(BRAND_500.with_alpha(0.3))
                }),
        )
        .child(label(
            "focus: border and ring, placeholder: color, caret color",
        ))
        .into_element()
}
