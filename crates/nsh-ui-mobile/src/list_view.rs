//! `ListView`: a scrolling list of the app's rows, recycled natively.

use nsh_core::ui::prelude::*;

/// A scrolling list of rows ([`ListViewProps`]).
#[derive(Debug, Default)]
pub struct ListView;

/// A scrolling list of the app's rows, padded and divided: only the rows
/// near the screen exist, cells are reused. The app passes the data and
/// renders a row (`each` and children, like `List`); the look is the
/// list's.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct ListViewProps {
    /// The rows.
    pub items: Items,
    /// `plain` (default): full-width rows; `inset`: rows in a rounded box
    /// with a margin, like grouped settings.
    #[prop(default)]
    pub variant: ListViewVariant,
    /// Hairlines between rows.
    #[prop(default = true)]
    pub dividers: bool,
}

/// How a [`ListView`] shows its rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum ListViewVariant {
    /// Full-width rows on the background.
    #[default]
    Plain,
    /// Rows in a rounded box with a margin around it.
    Inset,
}

impl Component for ListView {
    type Props = ListViewProps;
    type Event = NoEvent;

    fn render(&self, props: &ListViewProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let inset = props.variant == ListViewVariant::Inset;
        // The last rows scroll clear of the home indicator (renders again
        // when the safe area changes, as `safeArea()` in Solid).
        let below = cx.safe_area().bottom;
        // Inset: rows on the background color over a muted page, as
        // grouped settings look.
        let (border, dividers, surface, radius) = (
            theme.border,
            props.dividers,
            theme.background,
            px(theme.radius_lg),
        );
        view()
            .flex_1()
            .bg(if inset { theme.muted } else { theme.background })
            .text_color(theme.foreground)
            .list(
                props.items,
                |list| {
                    list.flex_1()
                        .estimated_item_size(48.0)
                        .when(inset, |l| l.content_padding(16.0))
                        .content_padding_bottom(below + if inset { 16.0 } else { 0.0 })
                },
                move |row, at| {
                    view()
                        .px_4()
                        .py_3()
                        .min_h(px(48.0))
                        .justify_center()
                        .when(dividers && !at.is_first(), |v| {
                            v.border_t_1().border_color(border)
                        })
                        .when(inset, |v| {
                            v.bg(surface)
                                .when(at.is_first(), |v| v.rounded_t(radius))
                                .when(at.is_last(), |v| v.rounded_b(radius))
                        })
                        .child(row)
                },
            )
    }
}
