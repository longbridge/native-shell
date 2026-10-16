//! Backgrounds, borders, rounded corners, clipping and shadows, drawn by
//! Core Animation on the view's layer and sublayers ([`apply`]).

use nsh_core::{Color, ViewStyle};
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::CGColor;

use crate::view::{NshView, ViewState};

/// The layers decoration added to a view.
#[derive(Default)]
pub(crate) struct Decor {}

/// Applies a view's decoration (background, corners, border, shadow,
/// clipping) to its layers.
pub(crate) fn apply(view: &NshView) {
    let Some(layer) = view.ns().layer() else {
        return;
    };
    let s = view.state();
    layer.setBackgroundColor(cg_color(s.background()).as_deref());
    layer.setCornerRadius(s.radii()[0] as f64);
    layer.setMasksToBounds(s.clips());
}

/// A view's style changed (`old` is the previous one).
pub(crate) fn style_changed(_view: &NshView, _old: &ViewStyle) {}

/// Whether the view shows its back (`backfaceVisibility: hidden`).
pub(crate) fn backface_away(_state: &ViewState) -> bool {
    false
}

/// `color` as a Core Graphics color (`None` when transparent).
pub(crate) fn cg_color(color: Color) -> Option<CFRetained<CGColor>> {
    if color.is_transparent() {
        return None;
    }
    let (r, g, b, a) = color.to_f64();
    Some(CGColor::new_srgb(r, g, b, a))
}
