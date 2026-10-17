//! Styles: corners (also continuous), borders, shadows and glows,
//! clipping, stacking and opacity, 3D transforms, numerals and right to
//! left layout, each on a card of specimens (as the TSX gallery's
//! `demos/Styles.tsx`).

use nsh_core::{
    nsh_animation::{Curve, EaseMode},
    ui::{Animation, TransformKind, prelude::*},
};

use crate::ui::{
    Badge, Button, ButtonVariant, Card, Finish, Finished, IconName, MONO, PressProps, Segment,
    SegmentedEvent, ToggleEvent, appear, caption, colors, inset_shadow, press_with, radius, rgb_a,
    segmented, tints, toggle,
};

/// The stage every specimen stands on: the card's light, without its
/// drop shadow.
fn stage() -> View {
    view()
        .h(px(104.0))
        .items_center()
        .justify_center()
        .rounded(px(radius::CONTROL))
        .border_curve(BorderCurve::Continuous)
        .bg(colors::SURFACE)
        .border_1()
        .border_color(colors::LINE)
        .finish_fill(Finish::Card)
        .adaptive_shadow(
            [inset_shadow(1.0, 0.0, rgb(0xffffff))],
            [inset_shadow(1.0, 0.0, rgb_a(0xffffff, 0.05))],
        )
}

/// A specimen box: a raised surface the styles are tried on.
fn specimen_box() -> View {
    view()
        .w(px(112.0))
        .h(px(76.0))
        .items_center()
        .justify_center()
        .bg(colors::RAISED)
}

fn label(content: impl Into<SharedString>, color: AdaptiveColor) -> Text {
    text(content.into())
        .text_size(px(13.0))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
}

/// One cell of a two-column grid: the specimen on a stage, a label, the
/// style it shows (`code`, monospaced) and a plain note.
fn tile(
    label: impl Into<SharedString>,
    code: Option<String>,
    note: Option<String>,
    on_press: Option<Listener<()>>,
    specimen: impl IntoElement,
) -> View {
    view()
        .basis(relative(0.4))
        .flex_grow(1.0)
        .gap_2()
        .child(
            stage()
                .when_some(on_press, |v, listener| v.on_press(listener))
                .child(specimen),
        )
        .child(
            view()
                .gap(px(3.0))
                .px(px(2.0))
                .child(
                    text(label.into())
                        .text_size(px(13.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(colors::TEXT),
                )
                .children(code.map(|code| {
                    text(code)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(colors::MUTED)
                        .font_family(MONO)
                }))
                .children(note.map(|note| {
                    text(note)
                        .number_of_lines(1)
                        .text_size(px(12.0))
                        .text_color(colors::FAINT)
                        .tabular_nums()
                })),
        )
}

fn some(s: &str) -> Option<String> {
    Some(s.to_owned())
}

/// Tiles two to a row.
fn grid() -> View {
    view()
        .flex_row()
        .flex_wrap()
        .gap_x(px(12.0))
        .gap_y(px(16.0))
}

/// A caption whose label and note are spans of their own (TS `<Caption
/// label={<Text …/>} note={<Text …/>} />`).
fn caption_spans(label: Text, note: &'static str) -> View {
    view()
        .flex_row()
        .items_baseline()
        .gap_2()
        .child(
            text("")
                .flex_1()
                .text_size(px(14.0))
                .text_color(colors::MUTED)
                .child(label),
        )
        .child(
            text("")
                .text_size(px(13.0))
                .text_color(colors::FAINT)
                .tabular_nums()
                .child(text(note).font_family(MONO)),
        )
}

/// A face of the flip card: its fill, its gradient's two stops and its
/// rim, a badge, its angle and two lines.
fn face(
    [fill, from, to, rim]: [AdaptiveColor; 4],
    badge: Badge,
    degrees: &'static str,
    title: &'static str,
    detail: &'static str,
) -> View {
    view()
        .absolute()
        .inset(px(0.0))
        .rounded(px(radius::CARD))
        .border_curve(BorderCurve::Continuous)
        .border_1()
        .p_4()
        .justify_between()
        .backface_hidden()
        .bg(fill)
        .bg_linear_to_b()
        .gradient_from(from)
        .gradient_to(to)
        .adaptive_shadow(
            [
                inset_shadow(1.0, 0.0, rgb(0xffffff)),
                BoxShadow {
                    offset_x: 0.0,
                    offset_y: 8.0,
                    blur: 20.0,
                    spread: 0.0,
                    color: rgb_a(0x14140a, 0.10),
                    inset: false,
                },
            ],
            [
                inset_shadow(1.0, 0.0, rgb_a(0xffffff, 0.07)),
                BoxShadow {
                    offset_x: 0.0,
                    offset_y: 10.0,
                    blur: 24.0,
                    spread: 0.0,
                    color: rgb_a(0x000000, 0.4),
                    inset: false,
                },
            ],
        )
        .border_color(rim)
        .perspective(800.0)
        .child(
            view().flex_row().justify_between().child(badge).child(
                text(degrees)
                    .text_size(px(13.0))
                    .text_color(colors::MUTED)
                    .tabular_nums(),
            ),
        )
        .child(
            view()
                .gap(px(2.0))
                .child(
                    text(title)
                        .text_size(px(20.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT),
                )
                .child(text(detail).text_size(px(13.0)).text_color(colors::MUTED)),
        )
}

/// Two faces back to back: each hides while turned away.
#[derive(Debug, Default)]
struct FlipCard {
    angle: Option<NativeValue>,
    flipped: bool,
}

impl Component for FlipCard {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.angle = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let angle = self.angle.as_ref().expect("set up");
        let flip = cx.listener(|this: &mut Self, _: &(), _| {
            this.flipped = !this.flipped;
            let to = if this.flipped { 180.0 } else { 0.0 };
            this.angle.as_ref().expect("set up").spring(
                to,
                Spring {
                    stiffness: 140.0,
                    damping: 16.0,
                    mass: 1.0,
                    velocity: None,
                },
            );
        });
        let tone = |light: u32, dark: u32| AdaptiveColor::new(rgb(light), rgb(dark));
        view().flex_1().child(
            press_with(
                PressProps {
                    scale: 0.98.into(),
                    label: Some("Flip the card".into()).into(),
                    ..PressProps::default()
                },
                flip,
            )
            .child(
                view()
                    .h(px(132.0))
                    // The bound rotations follow the static perspective.
                    .child(
                        face(
                            [
                                tone(0xeef2fc, 0x1a2134),
                                tone(0xf5f8fe, 0x212a42),
                                tone(0xe6ecfa, 0x171d2d),
                                tone(0xcdd8f3, 0x2b3a5c),
                            ],
                            Badge::new("front").tone(crate::ui::Tone::Info),
                            "0°",
                            "Tap to flip",
                            "A spring turns it on the UI thread",
                        )
                        .bind_transform(TransformKind::RotateY, angle),
                    )
                    .child(
                        face(
                            [
                                tone(0xf1f7e2, 0x202814),
                                tone(0xf7fbec, 0x283219),
                                tone(0xe9f2d2, 0x1b2211),
                                tone(0xd3e3a8, 0x3b4a22),
                            ],
                            Badge::new("back").tone(crate::ui::Tone::Accent),
                            "180°",
                            "backfaceVisibility",
                            "hidden: each face hides while turned away",
                        )
                        .bind_transform(
                            TransformKind::RotateY,
                            &angle.interpolate(&[0.0, 180.0], &[180.0, 360.0]),
                        ),
                    ),
            ),
        )
    }
}

/// The opacity levels of the layers card.
const LEVELS: [(&str, f32); 3] = [("0.2", 0.2), ("0.45", 0.45), ("1", 1.0)];

/// The demo's page.
#[derive(Default)]
pub(crate) struct Styles {
    /// The warning box clips what overflows it.
    clip: bool,
    /// The square of the stacking tile on top.
    front: usize,
    /// The opacity level's index.
    level: u32,
    /// The opacity box's opacity, animated natively.
    opacity: Option<NativeValue>,
    /// The perspective tile's rotation, swaying natively.
    sway: Option<NativeValue>,
    /// The sway (kept: dropping the last handle releases it).
    _sway_loop: Option<Animation>,
}

impl Styles {
    /// Circular and continuous corners side by side, at the same radius.
    fn corners() -> Card {
        let shape = |name: &'static str, shape: View| {
            view()
                .items_center()
                .gap(px(10.0))
                .child(view().h(px(64.0)).justify_center().child(shape))
                .child(text(name).text_size(px(12.0)).text_color(colors::MUTED))
        };
        Card::new()
            .icon(IconName::Styles)
            .title("Corners")
            .subtitle("borderRadius · borderCurve · per corner")
            .child(
                view()
                    .flex_row()
                    .justify_between()
                    .child(shape(
                        "Circular",
                        view().size(px(60.0)).rounded(px(20.0)).bg(colors::INFO),
                    ))
                    .child(shape(
                        "Continuous",
                        view()
                            .size(px(60.0))
                            .rounded(px(20.0))
                            .border_curve(BorderCurve::Continuous)
                            .bg(colors::INFO),
                    ))
                    .child(shape(
                        "One corner",
                        view()
                            .size(px(60.0))
                            .rounded(px(20.0))
                            .border_curve(BorderCurve::Continuous)
                            .border(2.5)
                            .border_color(colors::ACCENT)
                            .rounded_tl(px(4.0)),
                    ))
                    .child(shape(
                        "Pill",
                        view()
                            .w(px(60.0))
                            .h(px(28.0))
                            .rounded(px(14.0))
                            .border_curve(BorderCurve::Continuous)
                            .bg(colors::UP),
                    )),
            )
            .child(caption(
                "Continuous corners ease into the edges",
                Some("Radius 20 pt"),
            ))
            .child(
                grid()
                    .child(tile(
                        "Top corners",
                        some("borderTop*Radius"),
                        some("28 pt"),
                        None,
                        specimen_box()
                            .rounded_t(px(28.0))
                            .border(1.5)
                            .border_color(colors::ACCENT)
                            .child(label("28 pt", colors::ACCENT)),
                    ))
                    .child(tile(
                        "Mixed radii",
                        None,
                        some("44 · 6 · 28 · 0 pt, clockwise"),
                        None,
                        specimen_box()
                            .bg(tints::INFO)
                            .rounded_tl(px(44.0))
                            .rounded_tr(px(6.0))
                            .rounded_br(px(28.0))
                            .border_2()
                            .border_color(colors::INFO)
                            .child(label("Mixed", colors::INFO)),
                    )),
            )
    }

    fn borders() -> Card {
        Card::new()
            .icon(IconName::Components)
            .title("Borders")
            .subtitle("per side · dashed · dotted")
            .child(
                grid()
                    .child(tile(
                        "Per-side borders",
                        None,
                        some("6 · 2 · 4 · 1 pt, clockwise"),
                        None,
                        specimen_box()
                            .border_t(6.0)
                            .border_t_color(colors::DANGER)
                            .border_r(2.0)
                            .border_r_color(colors::UP)
                            .border_b(4.0)
                            .border_b_color(colors::INFO)
                            .border_l(1.0)
                            .border_l_color(colors::WARN),
                    ))
                    .child(tile(
                        "Dashed / dotted",
                        some("borderStyle"),
                        None,
                        None,
                        view()
                            .gap_2()
                            .child(
                                view()
                                    .w(px(112.0))
                                    .h(px(28.0))
                                    .rounded(px(8.0))
                                    .border(1.5)
                                    .border_color(colors::INFO)
                                    .border_dashed(),
                            )
                            .child(
                                view()
                                    .w(px(112.0))
                                    .h(px(28.0))
                                    .rounded(px(8.0))
                                    .border(3.0)
                                    .border_color(colors::WARN)
                                    .border_dotted(),
                            ),
                    )),
            )
    }

    /// Shadows that read on a dark page: colored glows rather than black.
    fn shadows(dark: bool) -> Card {
        Card::new()
            .icon(IconName::Spark)
            .title("Shadows and glow")
            .subtitle("shadowColor · elevation · text shadow")
            .child(
                grid()
                    .child(tile(
                        "Glow",
                        some("shadowColor"),
                        some("A colored shadow"),
                        None,
                        specimen_box()
                            .rounded(px(16.0))
                            .border_curve(BorderCurve::Continuous)
                            .bg(colors::ACCENT_FILL)
                            .shadow_color(colors::ACCENT_FILL)
                            .shadow_opacity(0.55)
                            .shadow_radius(18.0)
                            .shadow_offset(0.0, 6.0)
                            .child(label("Glow", colors::ON_ACCENT)),
                    ))
                    .child(tile(
                        "Offset shadow",
                        some("shadowOffset"),
                        some("4, 8 pt · radius 10"),
                        None,
                        specimen_box()
                            .bg(AdaptiveColor::new(rgb(0xe7edfb), rgb(0x1d2740)))
                            .rounded_tl(px(30.0))
                            .rounded_br(px(30.0))
                            .border_1()
                            .border_color(colors::INFO)
                            .shadow_color(colors::INFO)
                            .shadow_opacity(0.45)
                            .shadow_radius(10.0)
                            .shadow_offset(4.0, 8.0)
                            .child(label("Offset", colors::INFO)),
                    ))
                    .child(tile(
                        "Image corners",
                        None,
                        some("Two radii and a shadow"),
                        None,
                        image("https://picsum.photos/seed/nsh/300/220")
                            .w(px(112.0))
                            .h(px(76.0))
                            .rounded_tl(px(28.0))
                            .rounded_br(px(28.0))
                            .bg(colors::RAISED)
                            .shadow_color(rgb(0x000000))
                            .shadow_opacity(if dark { 0.6 } else { 0.22 })
                            .shadow_radius(12.0)
                            .shadow_offset(0.0, 8.0),
                    ))
                    // A transparent view's shadow follows its content: the
                    // text.
                    .child(tile(
                        "Text shadow",
                        None,
                        some("Shadow of the content"),
                        None,
                        specimen_box()
                            .bg(Color::TRANSPARENT)
                            .shadow_color(colors::DANGER)
                            .shadow_opacity(0.9)
                            .shadow_radius(8.0)
                            .shadow_offset(0.0, 0.0)
                            .child(
                                text("Shadow")
                                    .text_size(px(24.0))
                                    .font_weight(FontWeight::EXTRA_BOLD)
                                    .text_color(colors::TEXT)
                                    .letter_spacing(-0.5),
                            ),
                    ))
                    .child(tile(
                        "Text corners",
                        None,
                        some("Radii on a Text"),
                        None,
                        text("Text corners")
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::UP)
                            .bg(tints::UP)
                            .rounded_tl(px(14.0))
                            .rounded_br(px(14.0))
                            .px(px(14.0))
                            .py_2(),
                    ))
                    .child(tile(
                        "Elevation",
                        some("elevation: 10"),
                        some("Android"),
                        None,
                        specimen_box()
                            .elevation(10.0)
                            .rounded(px(16.0))
                            .bg(colors::RAISED)
                            .shadow_opacity(0.5)
                            .shadow_radius(10.0)
                            .shadow_offset(0.0, 6.0)
                            .child(label("Elevation", colors::TEXT)),
                    )),
            )
    }

    /// Clipping, stacking order and opacity, each one tap away.
    fn layers(&self, dark: bool, cx: &RenderCx<'_, Self>) -> Card {
        let clip = self.clip;
        let switch = cx.listener(|this: &mut Self, _: &(), cx| {
            this.clip = !this.clip;
            cx.notify();
        });
        let next = cx.listener(|this: &mut Self, _: &(), cx| {
            this.front = (this.front + 1) % 3;
            cx.notify();
        });
        let squares = [colors::DANGER, colors::INFO, colors::UP]
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                let front = self.front == i;
                view()
                    .absolute()
                    .left(px(4.0 + i as f32 * 28.0))
                    .top(px(2.0 + i as f32 * 14.0))
                    .w(px(64.0))
                    .h(px(52.0))
                    .rounded(px(12.0))
                    .border_curve(BorderCurve::Continuous)
                    .bg(color)
                    .border(if front { 2.0 } else { 0.0 })
                    .border_color(colors::TEXT)
                    .z_index(if front { 10 } else { 0 })
                    .shadow_color(rgb(0x000000))
                    .shadow_opacity(if dark { 0.5 } else { 0.18 })
                    .shadow_radius(6.0)
                    .shadow_offset(0.0, 3.0)
            });
        let opacity = self.opacity.as_ref().expect("set up");
        Card::new()
            .icon(IconName::Lists)
            .title("Clipping and layers")
            .subtitle("overflow · zIndex · opacity")
            .right(toggle(clip, "Clip").on_event(cx.listener(
                |this: &mut Self, event: &ToggleEvent, cx| {
                    let ToggleEvent::Change(on) = *event;
                    this.clip = on;
                    cx.notify();
                },
            )))
            .child(
                grid()
                    .child(tile(
                        if clip { "Clipped" } else { "Visible" },
                        Some(format!(
                            "overflow: {}",
                            if clip { "hidden" } else { "visible" }
                        )),
                        some("Tap to switch"),
                        Some(switch),
                        specimen_box()
                            .bg(tints::WARN)
                            .rounded(px(22.0))
                            .border(1.5)
                            .border_color(colors::WARN)
                            .when(clip, |v| v.overflow_hidden())
                            .child(
                                view()
                                    .absolute()
                                    .left(px(-26.0))
                                    .top(px(-26.0))
                                    .size(px(72.0))
                                    .rounded(px(36.0))
                                    .bg(colors::DANGER),
                            )
                            .child(label("Tap", colors::WARN)),
                    ))
                    .child(tile(
                        "Stacking order",
                        some("zIndex"),
                        Some(format!("Front: {} of 3 · tap", self.front + 1)),
                        Some(next),
                        view().w(px(130.0)).h(px(84.0)).children(squares),
                    )),
            )
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(
                        view()
                            .size(px(64.0))
                            .rounded(px(14.0))
                            .border_curve(BorderCurve::Continuous)
                            .bg(colors::INFO)
                            .bind_opacity(opacity)
                            .items_center()
                            .justify_center()
                            .child(
                                text(LEVELS[self.level as usize].0)
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(AdaptiveColor::new(rgb(0xffffff), rgb(0x0c1020)))
                                    .tabular_nums(),
                            ),
                    )
                    .child(
                        view()
                            .flex_1()
                            .gap_2()
                            .child(caption("Opacity", Some("Animated natively")))
                            .child(
                                segmented(LEVELS.map(|(l, _)| Segment::new(l)), self.level)
                                    .on_event(cx.listener(
                                        |this: &mut Self, event: &SegmentedEvent, cx| {
                                            let SegmentedEvent::Change(level) = *event;
                                            this.level = level;
                                            this.opacity.as_ref().expect("set up").timing(
                                                LEVELS[level as usize].1,
                                                260.0,
                                                Easing::Curve(Curve::Cubic, EaseMode::Out),
                                            );
                                            cx.notify();
                                        },
                                    )),
                            ),
                    ),
            )
    }

    fn transforms(&self) -> Card {
        let sway = self.sway.as_ref().expect("set up");
        Card::new()
            .icon(IconName::Physics)
            .title("3D transforms")
            .subtitle("perspective · rotateX/Y · skew · origin")
            .child(FlipCard::element(()))
            .child(
                grid()
                    .child(tile(
                        "Perspective",
                        some("rotateY"),
                        some("Swaying, driven natively"),
                        None,
                        specimen_box()
                            .rounded(px(14.0))
                            .bg(tints::INFO)
                            .border_1()
                            .border_color(colors::INFO)
                            .perspective(400.0)
                            .bind_transform(TransformKind::RotateY, sway)
                            .child(label("Y axis", colors::INFO)),
                    ))
                    .child(tile(
                        "Tilt",
                        some("rotateX: 40deg"),
                        None,
                        None,
                        specimen_box()
                            .rounded(px(14.0))
                            .bg(tints::ACCENT)
                            .border_1()
                            .border_color(colors::ACCENT)
                            .perspective(400.0)
                            .rotate_x(40.0)
                            .child(label("X axis", colors::ACCENT)),
                    ))
                    .child(tile(
                        "Skew",
                        some("skewX: -12deg"),
                        some("iOS"),
                        None,
                        specimen_box()
                            .rounded(px(10.0))
                            .bg(tints::WARN)
                            .border_1()
                            .border_color(colors::WARN)
                            .skew_x(-12.0)
                            .child(label("Skew", colors::WARN)),
                    ))
                    .child(tile(
                        "Origin",
                        some("transformOrigin"),
                        some("Top left · rotated 30°"),
                        None,
                        view()
                            .w(px(112.0))
                            .h(px(84.0))
                            .child(
                                view()
                                    .absolute()
                                    .left(px(30.0))
                                    .top(px(14.0))
                                    .size(px(54.0))
                                    .rounded(px(6.0))
                                    .border_1()
                                    .border_color(colors::FAINT)
                                    .border_dashed(),
                            )
                            .child(
                                view()
                                    .absolute()
                                    .left(px(30.0))
                                    .top(px(14.0))
                                    .size(px(54.0))
                                    .rounded(px(6.0))
                                    .bg(colors::DANGER.opacity(0.55))
                                    .origin_top_left()
                                    .rotate(30.0),
                            )
                            .child(
                                view()
                                    .absolute()
                                    .left(px(26.0))
                                    .top(px(10.0))
                                    .size(px(8.0))
                                    .rounded(px(4.0))
                                    .bg(colors::TEXT),
                            ),
                    )),
            )
    }

    fn numerals() -> Card {
        let figure = |n: &'static str| {
            text(n)
                .text_size(px(22.0))
                .text_color(colors::TEXT)
                .font_weight(FontWeight::MEDIUM)
        };
        let rule = || view().h(px(1.0)).bg(colors::LINE);
        Card::new()
            .icon(IconName::Text)
            .title("Numerals")
            .subtitle("fontVariant")
            .child(
                view()
                    .flex_row()
                    .gap_3()
                    .child(
                        view()
                            .flex_1()
                            .gap_1()
                            .child(
                                text("Proportional")
                                    .text_size(px(12.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(figure("1,111.11"))
                            .child(figure("8,888.88")),
                    )
                    .child(view().w(px(1.0)).bg(colors::LINE))
                    .child(
                        view()
                            .flex_1()
                            .gap_1()
                            .child(
                                text("tabular-nums")
                                    .text_size(px(12.0))
                                    .text_color(colors::ACCENT)
                                    .font_family(MONO),
                            )
                            .child(figure("1,111.11").tabular_nums())
                            .child(figure("8,888.88").tabular_nums()),
                    ),
            )
            .child(rule())
            .child(caption_spans(
                text("Small Caps")
                    .text_size(px(18.0))
                    .text_color(colors::TEXT)
                    .small_caps(),
                "small-caps",
            ))
            .child(caption_spans(
                text("Oldstyle 1234567890")
                    .text_size(px(18.0))
                    .text_color(colors::TEXT)
                    .oldstyle_nums(),
                "oldstyle-nums",
            ))
    }

    /// Rows, `marginStart` and text under a right-to-left layout.
    fn right_to_left(cx: &RenderCx<'_, Self>) -> Card {
        let boxes = [colors::DANGER, colors::INFO, colors::UP]
            .into_iter()
            .enumerate()
            .map(|(i, color)| {
                view()
                    .size(px(44.0))
                    .rounded(px(10.0))
                    .rounded_ss(px(22.0))
                    .items_center()
                    .justify_center()
                    .bg(color)
                    .when(i == 0, |v| v.ms(px(16.0)))
                    .child(
                        text((i + 1).to_string())
                            .text_color(AdaptiveColor::new(rgb(0xffffff), rgb(0x0c0d0f)))
                            .font_weight(FontWeight::BOLD)
                            .text_size(px(15.0)),
                    )
            });
        let chip = |n: usize, fill: AdaptiveColor, fg: AdaptiveColor| {
            view()
                .size(px(40.0))
                .rounded(px(10.0))
                .border_curve(BorderCurve::Continuous)
                .items_center()
                .justify_center()
                .bg(fill)
                .child(
                    text(n.to_string())
                        .text_size(px(13.0))
                        .text_color(fg)
                        .tabular_nums(),
                )
        };
        let mono_note = |label: &'static str, note: &'static str| caption_spans(text(label), note);
        let flip = cx.listener(|_: &mut Self, _: &(), cx| {
            let rtl = cx.environment().rtl;
            cx.force_rtl(!rtl);
        });
        Card::new()
            .icon(IconName::Nav)
            .title("Right to left")
            .subtitle(if cx.rtl() {
                "isRTL: true"
            } else {
                "isRTL: false"
            })
            .right(Button::new("Flip", flip).variant(ButtonVariant::Secondary))
            .child(view().flex_row().gap_2().children(boxes))
            .child(
                text(
                    "Text aligns to the start side; the first box has marginStart 16 and a round top-start corner.",
                )
                .text_size(px(15.0))
                .line_height(px(21.0))
                .text_color(colors::TEXT),
            )
            .child(mono_note("Horizontal scroll view", "Scroll"))
            .child(
                scroll()
                    .horizontal()
                    .shows_indicator(false)
                    .h(px(40.0))
                    .child(
                        view()
                            .flex_row()
                            .gap(px(6.0))
                            .children((1..=12).map(|n| chip(n, tints::INFO, colors::INFO))),
                    ),
            )
            .child(mono_note("Horizontal list", "List"))
            .child(
                virtual_list(30, move |at| {
                    let i = at.index();
                    chip(i + 1, tints::ACCENT, colors::ACCENT).me(px(6.0))
                })
                .horizontal()
                .estimated_item_size(46.0)
                .shows_indicator(false)
                .h(px(40.0)),
            )
            .child(view().h(px(1.0)).bg(colors::LINE))
            .child(
                text("textAlign left (the start side right to left)")
                    .text_size(px(13.0))
                    .text_left()
                    .text_color(colors::MUTED),
            )
            .child(
                text("مرحبا بالعالم — writingDirection rtl")
                    .text_size(px(13.0))
                    .writing_direction(WritingDirection::Rtl)
                    .text_color(colors::MUTED),
            )
    }
}

impl Component for Styles {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.clip = true;
        self.level = 1;
        self.opacity = Some(cx.native_value(0.45));
        // A slow native sway, so the perspective reads as depth.
        let sway = cx.native_value(-30.0);
        let sin = Easing::Curve(Curve::Sin, EaseMode::InOut);
        let sway_loop = Animation::sequence([
            Animation::timing(&sway, 30.0, Timing::new(2200.0, sin)),
            Animation::timing(&sway, -30.0, Timing::new(2200.0, sin)),
        ])
        .looped(None);
        sway_loop.start();
        self.sway = Some(sway);
        self._sway_loop = Some(sway_loop);
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let dark = cx.dark();
        let sections = [
            Self::corners(),
            Self::borders(),
            Self::shadows(dark),
            self.layers(dark, cx),
            self.transforms(),
            Self::numerals(),
            Self::right_to_left(cx),
        ];
        view().flex_1().child(
            scroll().flex_1().bg(colors::BACKGROUND).child(
                view().p_4().pb(px(40.0)).gap_4().children(
                    sections
                        .into_iter()
                        .enumerate()
                        .map(|(i, card)| appear(i as u32).child(card)),
                ),
            ),
        )
    }
}
