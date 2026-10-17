//! `--script` steps. (Being ported: see `nsh-platform-windows/src/script.rs`.)

use gpui::{App, WindowHandle};

use crate::app::AppView;

/// Runs `script`'s steps in `window` once it shows.
pub(crate) fn run(_window: WindowHandle<AppView>, script: &str, _cx: &mut App) {
    log::warn!("script steps are not supported yet: {script}");
}
