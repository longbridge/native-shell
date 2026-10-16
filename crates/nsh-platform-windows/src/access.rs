//! Accessibility: UI Automation providers for the views.

use std::{rc::Rc, sync::Arc};

use nsh_core::MountBatch;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};

use crate::{surface::Surface, view::NshView};

/// What accessibility last exposed for a view.
#[derive(Debug, Default, Clone)]
pub(crate) struct Applied {}

/// A batch is about to be applied.
pub(crate) fn prepare(batch: &MountBatch) {
    let _ = batch;
}

/// A batch was applied.
pub(crate) fn batch_applied(surface: &Rc<Surface>) {
    let _ = surface;
}

/// A view was made.
pub(crate) fn created(view: &NshView) {
    let _ = view;
}

/// The view's props changed.
pub(crate) fn props_changed(view: &NshView) {
    let _ = view;
}

/// The view's events changed.
pub(crate) fn events_changed(view: &NshView) {
    let _ = view;
}

/// The view's text changed.
pub(crate) fn text_changed(view: &NshView) {
    let _ = view;
}

/// `AccessibilityLabel`.
pub(crate) fn label(view: &NshView, label: &Option<Arc<str>>) {
    let _ = (view, label);
}

/// `AccessibilityFocus`.
pub(crate) fn focus(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// `WM_GETOBJECT`.
pub(crate) fn get_object(
    surface: &Rc<Surface>,
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
) -> Option<LRESULT> {
    let _ = (surface, hwnd, wparam, lparam);
    None
}
