//! Stacks, screens with headers, and modals.

use std::rc::Rc;

use nsh_core::{ModalConfig, ScreenConfig};

use crate::{surface::Surface, view::NshView};

/// Header heights (normal, large title) the core lays screens out with.
pub(crate) const HEADER_HEIGHTS: (f32, f32) = (47.0, 94.0);

/// Navigation state of a surface.
#[derive(Default)]
pub(crate) struct Nav {}

impl Nav {
    /// Forgets everything (the app stopped).
    pub(crate) fn reset(&mut self) {
        *self = Nav::default();
    }
}

/// A stack's children changed (`added`: the child just inserted).
pub(crate) fn stack_changed(_surface: &Rc<Surface>, _stack: &NshView, _added: Option<&NshView>) {}

/// A stack got a new frame.
pub(crate) fn stack_resized(_surface: &Rc<Surface>, _stack: &NshView) {}

/// A screen's configuration.
pub(crate) fn screen_config(_surface: &Rc<Surface>, _view: &NshView, _config: &ScreenConfig) {}

/// A modal's configuration.
pub(crate) fn modal_config(_surface: &Rc<Surface>, _view: &NshView, _config: &ModalConfig) {}

/// A modal is presented.
pub(crate) fn present(surface: &Rc<Surface>, view: &NshView) {
    view.insert_before(&surface.root, None);
}

/// A modal is dismissed.
pub(crate) fn dismiss(_surface: &Rc<Surface>, view: &NshView) {
    view.unparent();
}

/// A header item (button, title view) was inserted into a screen.
pub(crate) fn header_item(_surface: &Rc<Surface>, _parent: &NshView, _child: &NshView) {}

/// A header item was removed.
pub(crate) fn header_item_removed(_surface: &Rc<Surface>, _view: &NshView) {}

/// The surface got a new size.
pub(crate) fn surface_resized(_surface: &Surface, _width: f32, _height: f32) {}

/// A batch was applied.
pub(crate) fn end_batch(_surface: &Rc<Surface>) {}
