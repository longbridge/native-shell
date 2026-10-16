//! `nsh/desktop`'s window functions (macOS, Windows, Linux; nothing
//! happens elsewhere). Menus, which call the app's functions, are
//! `nsh-js`'s.

use crate::{
    Command, ROOT_ID,
    desktop::{DesktopCommand, TitleBarStyle},
    ui::{ModuleCx, module, task::Action},
};

fn desktop(cx: &mut ModuleCx, command: DesktopCommand) {
    cx.act(Action::Command(
        ROOT_ID,
        Command::Desktop(Box::new(command)),
    ));
}

/// The app's window.
pub(crate) struct Window;

/// The app's window (desktops; its functions do nothing elsewhere).
#[module]
impl Window {
    /// Sets the window's title (the title bar, the taskbar / Dock, the
    /// window switcher).
    pub(crate) fn set_title(title: String, cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::SetTitle(title));
    }

    /// Shows the system's title bar (`"default"`) or hides it
    /// (`"hidden"`): the app then fills the window up to its top edge and
    /// draws a title bar of its own, made draggable with
    /// `desktop={{ windowDrag: true }}`; the system's window controls stay,
    /// in the area `titleBar()` reports.
    pub(crate) fn set_title_bar(style: TitleBarStyle, cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::SetTitleBar(style));
    }

    /// Minimizes the window.
    pub(crate) fn minimize(cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::Minimize);
    }

    /// Maximizes the window, or restores it when it is maximized.
    pub(crate) fn toggle_maximize(cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::ToggleMaximize);
    }

    /// Enters or leaves full screen.
    pub(crate) fn toggle_full_screen(cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::ToggleFullScreen);
    }

    /// Closes the window; the app ends with it.
    pub(crate) fn close(cx: &mut ModuleCx) {
        desktop(cx, DesktopCommand::Close);
    }
}
