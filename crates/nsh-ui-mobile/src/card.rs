//! `Card`: a surface with an optional header and footer around its content.

use nsh_core::ui::prelude::*;

/// A rounded surface ([`CardProps`]).
#[derive(Debug, Default)]
pub struct Card;

/// A rounded surface with an optional header and footer around its content.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct CardProps {
    /// `elevated` (default): a border and a soft shadow; `outlined`: a
    /// border only.
    #[prop(default)]
    pub variant: CardVariant,
    /// Above the content: a title (its texts are semibold), an avatar row.
    pub header: Option<Slot>,
    /// Below the content, separated by a divider: actions, at the end.
    pub footer: Option<Slot>,
    /// The content.
    pub children: Children,
}

/// How a [`Card`] stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum CardVariant {
    /// A border and a soft shadow.
    #[default]
    Elevated,
    /// A border only.
    Outlined,
}

impl Component for Card {
    type Props = CardProps;
    type Event = NoEvent;

    fn render(&self, props: &CardProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let has_header = props.header.is_some();
        view()
            .rounded(px(theme.radius_lg))
            .bg(theme.group_box)
            .text_color(theme.group_box_foreground)
            .border_1()
            .border_color(theme.border)
            .when(props.variant == CardVariant::Elevated, |v| v.shadow_sm())
            .slot(props.header, |header| {
                header
                    .px_4()
                    .pt_4()
                    .text_size(px(theme.font_size + 1.0))
                    .font_weight(FontWeight::SEMIBOLD)
            })
            .child(
                view()
                    .gap_2()
                    .map(|v| {
                        if has_header {
                            v.px_4().pt_2().pb_4()
                        } else {
                            v.p_4()
                        }
                    })
                    .child(props.children),
            )
            .slot(props.footer, |footer| {
                footer
                    .flex_row()
                    .justify_end()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .border_t_1()
                    .border_color(theme.border)
            })
    }
}
