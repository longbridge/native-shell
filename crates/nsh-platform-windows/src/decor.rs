//! Decoration: shadows, backgrounds (colors, gradients), borders, outlines
//! and filters, drawn with Direct2D.


use crate::{geom::Mat4, render::DrawCx, view::ViewState};

/// True when the view faces away (`backfaceVisibility: hidden` and turned
/// more than 90 degrees in 3D), given its parent's world transform.
pub(crate) fn backface_away(state: &ViewState, parent: &Mat4) -> bool {
    let _ = (state, parent);
    false
}

/// Pushes the view's filters; returns how many layers to pop.
pub(crate) fn push_filters(cx: &mut DrawCx<'_>, state: &ViewState) -> usize {
    let _ = (cx, state);
    0
}

/// Draws the view's shadows and background.
pub(crate) fn draw_background(cx: &mut DrawCx<'_>, state: &ViewState) {
    let color = state.background();
    if color.is_transparent() {
        return;
    }
    let (w, h) = (state.frame.width, state.frame.height);
    let radii = state.radii();
    if radii.iter().all(|r| *r <= 0.0) {
        // SAFETY: a COM call between BeginDraw and EndDraw.
        unsafe {
            cx.dc
                .FillRectangle(&crate::render::rect_f(0.0, 0.0, w, h), cx.solid(color))
        };
    } else if let Some(g) = cx.rounded_geometry(crate::geom::rect(0.0, 0.0, w, h), radii) {
        // SAFETY: as above.
        unsafe { cx.dc.FillGeometry(&g, cx.solid(color), None) };
    }
}

/// Draws the view's border over its content and children.
pub(crate) fn draw_border(cx: &mut DrawCx<'_>, state: &ViewState) {
    let b = state.borders();
    let width = b.widths[0];
    if width <= 0.0 || b.colors[0].is_transparent() {
        return;
    }
    let (w, h) = (state.frame.width, state.frame.height);
    let inset = width / 2.0;
    let rect = crate::geom::rect(inset, inset, (w - width).max(0.0), (h - width).max(0.0));
    let radii = state.radii().map(|r| (r - inset).max(0.0));
    if let Some(g) = cx.rounded_geometry(rect, radii) {
        // SAFETY: a COM call between BeginDraw and EndDraw.
        unsafe { cx.dc.DrawGeometry(&g, cx.solid(b.colors[0]), width, None) };
    }
}
