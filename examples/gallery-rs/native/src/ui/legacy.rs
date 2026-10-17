//! The building blocks of the gallery's former, light-only design, which
//! the demos not yet restyled still use: they go once every demo is on
//! the shared design of [`super`].

use nsh_core::ui::{AnyElement, Scroll, View, prelude::*};

/// The gallery's colors.
pub(crate) mod colors {
    use nsh_core::ui::{Color, rgb};

    pub(crate) fn text() -> Color {
        rgb(0x1f2328)
    }
    pub(crate) fn muted() -> Color {
        rgb(0x6e7781)
    }
    pub(crate) fn border() -> Color {
        rgb(0xd0d7de)
    }
    pub(crate) fn surface() -> Color {
        rgb(0xffffff)
    }
    pub(crate) fn background() -> Color {
        rgb(0xf4f5f7)
    }
    pub(crate) fn accent() -> Color {
        rgb(0x0969da)
    }
    pub(crate) fn purple() -> Color {
        rgb(0x8250df)
    }
    pub(crate) fn gray() -> Color {
        rgb(0x57606a)
    }
}

/// A filled button with a label.
pub(crate) fn button(label: impl Into<SharedString>, color: Color) -> View {
    view()
        .bg(color)
        .rounded(px(10.0))
        .py(px(10.0))
        .px_4()
        .active(|s| s.opacity(0.6))
        .role(AccessibilityRole::Button)
        .child(
            text(label.into())
                .text_color(rgb(0xffffff))
                .font_weight(FontWeight::SEMIBOLD)
                .text_center(),
        )
}

/// A titled group of controls on a white card.
pub(crate) fn section(title: &str, content: impl IntoIterator<Item = AnyElement>) -> View {
    view()
        .gap_2()
        .child(
            text(title.to_uppercase())
                .text_size(px(13.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(colors::muted()),
        )
        .child(
            view()
                .bg(colors::surface())
                .rounded(px(12.0))
                .p(px(14.0))
                .gap_3()
                .children(content),
        )
}

/// A scrolling page of a demo.
pub(crate) fn page(content: impl IntoIterator<Item = AnyElement>) -> Scroll {
    scroll()
        .flex_1()
        .bg(colors::background())
        .child(view().p_4().gap_5().children(content))
}
