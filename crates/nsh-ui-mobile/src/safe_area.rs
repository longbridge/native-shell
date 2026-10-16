//! `SafeAreaView`: content kept clear of system UI and headers.

use nsh_core::ui::prelude::*;

/// A view padded by the insets where it renders ([`SafeAreaViewProps`]).
#[derive(Debug, Default)]
pub struct SafeAreaView;

/// A view padded by the insets of system UI or headers where it renders
/// (the surface's safe area, a modal's, or a screen's, below its header),
/// on top of its own padding.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SafeAreaViewProps {
    /// The edges to pad (all by default).
    pub edges: Option<Vec<Edge>>,
    /// The content.
    pub children: Children,
}

/// An edge of a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum Edge {
    /// The top edge.
    Top,
    /// The right edge.
    Right,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
}

impl Component for SafeAreaView {
    type Props = SafeAreaViewProps;
    type Event = NoEvent;

    fn render(&self, props: &SafeAreaViewProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let insets = cx.insets();
        let on = |edge: Edge| props.edges.as_ref().is_none_or(|e| e.contains(&edge));
        let pick = |edge, v: f32| if on(edge) { v } else { 0.0 };
        view()
            .inset_padding(EdgeInsets {
                top: pick(Edge::Top, insets.top),
                right: pick(Edge::Right, insets.right),
                bottom: pick(Edge::Bottom, insets.bottom),
                left: pick(Edge::Left, insets.left),
            })
            .child(props.children)
    }
}
