//! Stacks, screens, headers and modals.

use std::rc::Rc;

use nsh_core::{ModalConfig, ScreenConfig};

use crate::{surface::Surface, view::NshView};

/// The heights of a header without and with a large title.
pub(crate) const HEADER_HEIGHTS: (f32, f32) = (47.0, 94.0);

/// The navigation state of a surface.
#[derive(Default)]
pub(crate) struct Nav {}

impl Nav {
    /// Forgets everything (the app stopped).
    pub(crate) fn reset(&mut self) {}
}

/// A stack's screens changed.
pub(crate) fn stack_changed(surface: &Rc<Surface>, stack: &NshView, added: Option<&NshView>) {
    let _ = (surface, stack, added);
}

/// A stack was resized.
pub(crate) fn stack_resized(surface: &Rc<Surface>, stack: &NshView) {
    let _ = (surface, stack);
}

/// `ScreenConfig`.
pub(crate) fn screen_config(surface: &Rc<Surface>, view: &NshView, config: &ScreenConfig) {
    let _ = (surface, view, config);
}

/// `ModalConfig`.
pub(crate) fn modal_config(surface: &Rc<Surface>, view: &NshView, config: &ModalConfig) {
    let _ = (surface, view, config);
}

/// A modal is presented.
pub(crate) fn present(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// A modal is dismissed.
pub(crate) fn dismiss(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// A header item was inserted.
pub(crate) fn header_item(surface: &Rc<Surface>, parent: &NshView, child: &NshView) {
    let _ = (surface, parent, child);
}

/// A header item was removed.
pub(crate) fn header_item_removed(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// The surface was resized.
pub(crate) fn surface_resized(surface: &Surface, width: f32, height: f32) {
    let _ = (surface, width, height);
}

/// A batch was applied.
pub(crate) fn end_batch(surface: &Rc<Surface>) {
    let _ = surface;
}
