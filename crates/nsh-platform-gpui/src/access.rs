//! Accessibility. (Being ported: see `nsh-platform-windows/src/access.rs`.)

use std::{rc::Rc, sync::Arc};

use crate::{surface::Surface, view::NshView};

/// What accessibility last exposed of a view.
#[derive(Default)]
pub(crate) struct Applied;

/// A view was made.
pub(crate) fn created(_view: &NshView) {}
/// A view's events changed.
pub(crate) fn events_changed(_view: &NshView) {}
/// `AccessibilityFocus`.
pub(crate) fn focus(_surface: &Rc<Surface>, _view: &NshView) {}
/// A view's label changed.
pub(crate) fn label(_view: &NshView, _label: &Option<Arc<str>>) {}
/// A view's props changed.
pub(crate) fn props_changed(_view: &NshView) {}
/// A view's text changed.
pub(crate) fn text_changed(_view: &NshView) {}
/// A batch is about to be applied.
pub(crate) fn prepare(_batch: &nsh_core::MountBatch) {}
/// A batch was applied.
pub(crate) fn batch_applied(_surface: &Surface) {}
/// Announces `text` to assistive technologies (a toast, `announce`).
pub(crate) fn announce(_surface: &Rc<Surface>, text: &str) {
    log::debug!("announce: {text}");
}
