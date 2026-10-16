//! Accessibility: roles, labels, values and states of views for VoiceOver
//! (`NSAccessibility`).

use std::sync::Arc;

use nsh_core::MountBatch;

use crate::view::NshView;

/// What accessibility last gave AppKit for a view.
#[derive(Default)]
pub(crate) struct Applied {}

/// A batch is about to be applied.
pub(crate) fn prepare(_batch: &MountBatch) {}

/// A view was created.
pub(crate) fn created(_view: &NshView) {}

/// A view's props changed.
pub(crate) fn props_changed(_view: &NshView) {}

/// A view's events changed.
pub(crate) fn events_changed(_view: &NshView) {}

/// A text view's text changed.
pub(crate) fn text_changed(_view: &NshView) {}

/// A view's label (from its text descendants).
pub(crate) fn label(_view: &NshView, _label: &Option<Arc<str>>) {}

/// Moves accessibility (and keyboard) focus to `view`.
pub(crate) fn focus(_view: &NshView) {}
