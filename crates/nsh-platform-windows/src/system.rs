//! The environment (color scheme, scale, locale, text scale, reduce
//! motion, screen reader), dialogs, toasts and the clipboard.

use std::rc::Rc;

use nsh_core::{Color, Command, NodeId, platform::Appearance};
use nsh_engine::ColorScheme;
use windows::{
    Win32::{
        System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_DWORD, RegGetValueW},
        UI::Shell::SetCurrentProcessExplicitAppUserModelID,
    },
    core::{HSTRING, w},
};

use crate::surface::Surface;

thread_local! {
    /// The system appearance a script switched to (`dark` / `light`).
    static SIMULATED: std::cell::Cell<Option<bool>> = const { std::cell::Cell::new(None) };
}

/// Makes the system look dark (or light) from now on (`--script`).
pub(crate) fn simulate_system_scheme(dark: bool) {
    SIMULATED.with(|s| s.set(Some(dark)));
}

/// True when the system's apps use the dark theme (Settings >
/// Personalization > Colors).
pub(crate) fn system_dark() -> bool {
    if let Some(dark) = SIMULATED.with(std::cell::Cell::get) {
        return dark;
    }
    let mut value: u32 = 1;
    let mut size = std::mem::size_of::<u32>() as u32;
    // SAFETY: a DWORD out value of the size given.
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            w!("AppsUseLightTheme"),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut _),
            Some(&mut size),
        )
    };
    result.is_ok() && value == 0
}

/// True when the surface draws in its dark appearance.
pub(crate) fn dark(surface: &Surface) -> bool {
    match surface.appearance.get() {
        Appearance::Dark => true,
        Appearance::Light => false,
        Appearance::System => system_dark(),
    }
}

/// The color behind the app's views.
pub(crate) fn window_background(surface: &Surface) -> Color {
    if dark(surface) {
        Color(0x0000_00ff)
    } else {
        Color(0xffff_ffff)
    }
}

/// Reads the environment and keeps it current.
pub(crate) fn watch_environment(surface: &Rc<Surface>) {
    settings_changed(surface);
}

/// A system setting changed (`WM_SETTINGCHANGE`, `WM_THEMECHANGED`).
pub(crate) fn settings_changed(surface: &Rc<Surface>) {
    let mut env = surface.environment.get();
    env.color_scheme = if dark(surface) {
        ColorScheme::Dark
    } else {
        ColorScheme::Light
    };
    surface.set_environment(env);
    environment_applied(surface);
    surface.invalidate();
}

/// The environment changed: the window follows it.
pub(crate) fn environment_applied(surface: &Surface) {
    let hwnd = surface.hwnd();
    if !hwnd.is_invalid() {
        crate::window::set_dark_title_bar(hwnd, dark(surface));
    }
}

/// `Appearance`: the app's appearance changed.
pub(crate) fn set_appearance(surface: &Rc<Surface>, appearance: Appearance) {
    surface.appearance.set(appearance);
    settings_changed(surface);
}

/// The taskbar groups the process's windows under `id`.
pub(crate) fn set_app_id(id: &str) {
    // SAFETY: a valid string.
    let _ = unsafe { SetCurrentProcessExplicitAppUserModelID(&HSTRING::from(id)) };
}

/// System commands (alerts, toasts, the clipboard, ...).
pub(crate) fn command(surface: &Rc<Surface>, id: NodeId, command: &Command) {
    let _ = (surface, id, command);
}

/// A batch was applied.
pub(crate) fn end_batch(surface: &Surface) {
    let _ = surface;
}

/// The surface was resized.
pub(crate) fn surface_resized(surface: &Surface, width: f32, height: f32) {
    let _ = (surface, width, height);
}

/// The app stopped: dialogs and toasts go.
pub(crate) fn clear(surface: &Surface) {
    let _ = surface;
}

/// Keys for a dialog shown over the app (Enter, Escape, Tab); true when
/// handled.
pub(crate) fn key(surface: &Rc<Surface>, event: &crate::input::KeyEvent) -> bool {
    let _ = (surface, event);
    false
}

/// Puts `text` on the clipboard (Unicode text).
pub(crate) fn set_clipboard_text(text: &str) -> bool {
    use windows::Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
    };
    let units: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    // SAFETY: the clipboard is opened and closed here; the global memory
    // holds `units` and is owned by the clipboard once `SetClipboardData`
    // succeeds (freed here otherwise).
    unsafe {
        if OpenClipboard(None).is_err() {
            return false;
        }
        let mut ok = false;
        if EmptyClipboard().is_ok()
            && let Ok(memory) = GlobalAlloc(GMEM_MOVEABLE, units.len() * 2)
        {
            let ptr = GlobalLock(memory) as *mut u16;
            if !ptr.is_null() {
                std::ptr::copy_nonoverlapping(units.as_ptr(), ptr, units.len());
                let _ = GlobalUnlock(memory);
                ok = SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(memory.0))).is_ok();
            }
            if !ok {
                let _ = GlobalFree(Some(HGLOBAL(memory.0)));
            }
        }
        let _ = CloseClipboard();
        ok
    }
}

/// The clipboard's text, if it has some.
pub(crate) fn clipboard_text() -> Option<String> {
    use windows::Win32::{
        Foundation::HGLOBAL,
        System::{
            DataExchange::{CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard},
            Memory::{GlobalLock, GlobalSize, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
    };
    // SAFETY: the clipboard is opened and closed here; the data is read
    // while locked and copied out.
    unsafe {
        IsClipboardFormatAvailable(u32::from(CF_UNICODETEXT.0)).ok()?;
        OpenClipboard(None).ok()?;
        let text = GetClipboardData(u32::from(CF_UNICODETEXT.0)).ok().and_then(|h| {
            let memory = HGLOBAL(h.0);
            let ptr = GlobalLock(memory) as *const u16;
            if ptr.is_null() {
                return None;
            }
            let max = GlobalSize(memory) / 2;
            let units = std::slice::from_raw_parts(ptr, max);
            let len = units.iter().position(|u| *u == 0).unwrap_or(max);
            let text = String::from_utf16_lossy(&units[..len]);
            let _ = GlobalUnlock(memory);
            Some(text)
        });
        let _ = CloseClipboard();
        text
    }
}

/// True when the clipboard has text.
pub(crate) fn clipboard_has_text() -> bool {
    use windows::Win32::System::{DataExchange::IsClipboardFormatAvailable, Ole::CF_UNICODETEXT};
    // SAFETY: a plain query.
    unsafe { IsClipboardFormatAvailable(u32::from(CF_UNICODETEXT.0)) }.is_ok()
}
