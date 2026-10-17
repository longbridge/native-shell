//! Decoration: shadows, background and border.
//!
//! Drawn with GPUI's primitives: React Native's shadow as a drop shadow of
//! the view's rounded box (`paint_drop_shadows`), the background and the
//! border as one quad each (GPUI's quads have per-corner radii and
//! per-side widths but one border color: the top's, unless all sides
//! differ, then each side is filled on its own).

use gpui::{BoxShadow, point, px};
use nsh_core::Color;

use crate::{
    geom::Mat4,
    render::{DrawCx, corners, edges},
    view::ViewState,
};

/// True when `state` is turned away from the viewer and hides its back.
pub(crate) fn backface_away(state: &ViewState, world: &Mat4) -> bool {
    state.style.backface_hidden && faces_away(world)
}

/// True if the plane z = 0 seen through `m` shows its back: CSS's test, the
/// z-z entry of the inverse matrix is negative. That entry is the minor of
/// rows and columns x, y and w (the plane's homography) over the whole
/// determinant, so a mirror (`scaleX(-1)`) is not a back face, a half turn
/// about y is.
fn faces_away(m: &Mat4) -> bool {
    let a = &m.m;
    let h = [
        [a[0][0], a[0][1], a[0][3]],
        [a[1][0], a[1][1], a[1][3]],
        [a[3][0], a[3][1], a[3][3]],
    ];
    let minor = det3(&h);
    let det = det4(a);
    det.abs() > 1e-12 && minor / det < 0.0
}

fn det3(m: &[[f32; 3]; 3]) -> f32 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn det4(m: &[[f32; 4]; 4]) -> f32 {
    (0..4)
        .map(|c| {
            let minor: [[f32; 3]; 3] = std::array::from_fn(|r| {
                let row = &m[r + 1];
                let mut k = 0;
                std::array::from_fn(|_| {
                    if k == c {
                        k += 1;
                    }
                    k += 1;
                    row[k - 1]
                })
            });
            let sign = if c % 2 == 0 { 1.0 } else { -1.0 };
            sign * m[0][c] * det3(&minor)
        })
        .sum()
}

/// Draws the view's shadow and background (view coordinates set on `cx`).
pub(crate) fn draw_background(cx: &mut DrawCx<'_>, state: &ViewState) {
    let (w, h) = (state.frame.width, state.frame.height);
    let radii = state.radii();
    let shadow = state.shadow();
    if shadow.opacity > 0.0 && shadow.color.0 & 0xff != 0 && !cx.culled() {
        let k = cx.transform_scale();
        let mut color = cx.color(shadow.color);
        color.a *= shadow.opacity;
        let b = cx.rect(0.0, 0.0, w, h);
        let shadows = [BoxShadow {
            color,
            offset: point(px(shadow.offset_x * k), px(shadow.offset_y * k)),
            // React Native's radius is a Gaussian's two sigmas, as CSS's
            // blur radius.
            blur_radius: px(shadow.radius * k),
            spread_radius: px(0.0),
            inset: false,
        }];
        let c = corners(radii, k);
        // Shadows reach out of the view: drawn in the parent's clip.
        cx.paint(|window, _| window.paint_drop_shadows(b, c, &shadows));
    }
    let background = state.background();
    if background.0 & 0xff != 0 {
        cx.fill_rect(crate::geom::rect(0.0, 0.0, w, h), radii, background);
    }
}

/// Draws the view's border, over its content and children.
pub(crate) fn draw_border(cx: &mut DrawCx<'_>, state: &ViewState) {
    let borders = state.borders();
    if borders.widths.iter().all(|w| *w <= 0.0) || cx.culled() {
        return;
    }
    let (w, h) = (state.frame.width, state.frame.height);
    let k = cx.transform_scale();
    let b = cx.rect(0.0, 0.0, w, h);
    let radii = corners(state.radii(), k);
    let style = match borders.style {
        nsh_core::BorderStyle::Dashed | nsh_core::BorderStyle::Dotted => gpui::BorderStyle::Dashed,
        _ => gpui::BorderStyle::Solid,
    };
    let uniform = borders.colors.iter().all(|c| *c == borders.colors[0]);
    if uniform {
        let quad = gpui::quad(
            b,
            radii,
            gpui::transparent_black(),
            edges(borders.widths, k),
            cx.color(borders.colors[0]),
            style,
        );
        cx.paint_quad(quad);
        return;
    }
    // A color per side: one quad per side with only its width.
    for side in 0..4 {
        let width = borders.widths[side];
        if width <= 0.0 || borders.colors[side] == Color::TRANSPARENT {
            continue;
        }
        let mut widths = [0.0; 4];
        widths[side] = width;
        let quad = gpui::quad(
            b,
            radii,
            gpui::transparent_black(),
            edges(widths, k),
            cx.color(borders.colors[side]),
            style,
        );
        cx.paint_quad(quad);
    }
}
