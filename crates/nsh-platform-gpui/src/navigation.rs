//! Stacks, screens and modals. (Being ported: see
//! `nsh-platform-windows/src/navigation.rs`.)

use std::rc::Rc;

use nsh_core::{ModalConfig, ScreenConfig};

use crate::{input::KeyEvent, surface::Surface, view::NshView};

/// Header heights reported to the core: (compact, large title).
pub(crate) const HEADER_HEIGHTS: (f32, f32) = (47.0, 94.0);

/// Navigation state of a surface.
#[derive(Default)]
pub(crate) struct Nav;

impl Nav {
    /// Forgets the app's views.
    pub(crate) fn reset(&mut self) {}
}

/// A modal was inserted: present it.
pub(crate) fn present(surface: &Rc<Surface>, view: &NshView) {
    view.insert_before(&surface.root, None);
}
/// A modal was removed.
pub(crate) fn dismiss(_surface: &Rc<Surface>, view: &NshView) {
    view.unparent();
}
/// The batch is applied.
pub(crate) fn end_batch(_surface: &Rc<Surface>) {}
/// The view keyboard focus stays in (the newest modal, or the root).
pub(crate) fn focus_scope(surface: &Surface) -> NshView {
    surface.root.clone()
}
/// Goes back (a stack pops, a modal closes). True when something did.
pub(crate) fn go_back(_surface: &Rc<Surface>) -> bool {
    false
}
/// A header item was inserted.
pub(crate) fn header_item(_surface: &Rc<Surface>, _parent: &NshView, _child: &NshView) {}
/// A header item was removed.
pub(crate) fn header_item_removed(_surface: &Rc<Surface>, _view: &NshView) {}
/// Keys that go back. True when taken.
pub(crate) fn key_pressed(_surface: &Rc<Surface>, _event: &KeyEvent) -> bool {
    false
}
/// `ModalConfig`.
pub(crate) fn modal_config(_surface: &Rc<Surface>, _view: &NshView, _config: &ModalConfig) {}
/// A screen or modal is being removed.
pub(crate) fn removing(_surface: &Rc<Surface>, _view: &NshView) {}
/// `ScreenConfig`.
pub(crate) fn screen_config(_surface: &Rc<Surface>, _view: &NshView, _config: &ScreenConfig) {}
/// A stack's screens changed.
pub(crate) fn stack_changed(_surface: &Rc<Surface>, _stack: &NshView, _added: Option<&NshView>) {}
/// A stack was resized.
pub(crate) fn stack_resized(_surface: &Rc<Surface>, _stack: &NshView) {}
/// The surface was resized.
pub(crate) fn surface_resized(_surface: &Surface, _width: f32, _height: f32) {}
