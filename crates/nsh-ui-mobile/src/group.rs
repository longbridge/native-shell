//! `Group`: rows in a rounded box with dividers (settings-style).

use nsh_core::ui::prelude::*;

/// Rows in a rounded box ([`GroupProps`]).
#[derive(Debug, Default)]
pub struct Group;

/// Rows in a rounded box, divided by hairlines: each JSX child is a row.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct GroupProps {
    /// Above the box: a short caption.
    pub header: Option<Slot>,
    /// Below the box: an explanation.
    pub footer: Option<Slot>,
    /// The rows, one per child.
    pub children: Children,
}

impl Component for Group {
    type Props = GroupProps;
    type Event = NoEvent;

    fn render(&self, props: &GroupProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let (border, muted) = (theme.border, theme.muted_foreground);
        view()
            .gap_2()
            .slot(props.header, |header| {
                header.px_4().text_sm().text_color(muted)
            })
            .child(
                view()
                    .rounded(px(theme.radius_lg))
                    .bg(theme.group_box)
                    .text_color(theme.group_box_foreground)
                    .overflow_hidden()
                    .children_each(props.children, move |row, at| {
                        view()
                            .px_4()
                            .py_3()
                            .min_h(px(44.0))
                            .justify_center()
                            .when(!at.is_first(), |v| v.border_t_1().border_color(border))
                            .child(row)
                    }),
            )
            .slot(props.footer, |footer| {
                footer.px_4().text_xs().text_color(muted)
            })
    }
}
