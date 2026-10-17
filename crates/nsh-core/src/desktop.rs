//! What only desktops have: the window's title and title bar, the
//! application menu, context menus and keyboard shortcuts (TS
//! `nsh/desktop`). The platforms get them as [`DesktopCommand`]s on the
//! root and report a chosen menu item as [`crate::EventKind::Menu`];
//! mobile platforms ignore them.

use std::sync::Arc;

use crate::ui::PropEnum;

/// How the window's title bar looks (`Window.setTitleBar`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum TitleBarStyle {
    /// The system's title bar, above the app.
    #[default]
    Default,
    /// No system title bar: the app fills the window up to its top edge
    /// and draws its own title bar, which views with `windowDrag` make
    /// draggable. The system's window controls stay (macOS's traffic
    /// lights, Windows' caption buttons, GTK's window buttons) in the area
    /// the environment's `title_bar` reports.
    Hidden,
}

/// A key of a [`Shortcut`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    /// A character key, lowercase (`a`, `1`, `,`, `=`).
    Char(char),
    /// Enter / Return.
    Enter,
    /// Escape.
    Escape,
    /// Tab.
    Tab,
    /// Backspace.
    Backspace,
    /// Delete (forward).
    Delete,
    /// Insert.
    Insert,
    /// The space bar.
    Space,
    /// Arrow up.
    Up,
    /// Arrow down.
    Down,
    /// Arrow left.
    Left,
    /// Arrow right.
    Right,
    /// Home.
    Home,
    /// End.
    End,
    /// Page up.
    PageUp,
    /// Page down.
    PageDown,
    /// A function key, F1 to F24.
    F(u8),
}

impl Key {
    /// The key named `name` (`"n"`, `"enter"`, `"f5"`, `"plus"`), ignoring
    /// case. A character is the US layout's unshifted one.
    pub fn parse(name: &str) -> Option<Key> {
        let mut chars = name.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return (!c.is_whitespace() && !c.is_control())
                .then(|| Key::Char(c.to_ascii_lowercase()));
        }
        let lower = name.to_ascii_lowercase();
        Some(match lower.as_str() {
            "enter" | "return" => Key::Enter,
            "escape" | "esc" => Key::Escape,
            "tab" => Key::Tab,
            "backspace" => Key::Backspace,
            "delete" | "del" => Key::Delete,
            "insert" => Key::Insert,
            "space" => Key::Space,
            "up" | "arrowup" => Key::Up,
            "down" | "arrowdown" => Key::Down,
            "left" | "arrowleft" => Key::Left,
            "right" | "arrowright" => Key::Right,
            "home" => Key::Home,
            "end" => Key::End,
            "pageup" => Key::PageUp,
            "pagedown" => Key::PageDown,
            "plus" => Key::Char('+'),
            "minus" => Key::Char('-'),
            _ => {
                let n: u8 = lower.strip_prefix('f')?.parse().ok()?;
                if !(1..=24).contains(&n) {
                    return None;
                }
                Key::F(n)
            }
        })
    }

    /// Its name in a shortcut's label (`N`, `Enter`, `F5`).
    pub fn label(self) -> String {
        match self {
            Key::Char(c) => c.to_uppercase().collect(),
            Key::Enter => "Enter".into(),
            Key::Escape => "Esc".into(),
            Key::Tab => "Tab".into(),
            Key::Backspace => "Backspace".into(),
            Key::Delete => "Del".into(),
            Key::Insert => "Ins".into(),
            Key::Space => "Space".into(),
            Key::Up => "Up".into(),
            Key::Down => "Down".into(),
            Key::Left => "Left".into(),
            Key::Right => "Right".into(),
            Key::Home => "Home".into(),
            Key::End => "End".into(),
            Key::PageUp => "PgUp".into(),
            Key::PageDown => "PgDn".into(),
            Key::F(n) => format!("F{n}"),
        }
    }
}

/// A keyboard shortcut of a menu item (`"mod+shift+n"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Shortcut {
    /// The key.
    pub key: Key,
    /// Control.
    pub ctrl: bool,
    /// Alt / Option.
    pub alt: bool,
    /// Shift.
    pub shift: bool,
    /// Command (macOS) / the Windows key / Super.
    pub meta: bool,
}

impl Shortcut {
    /// A shortcut of `key` alone.
    pub const fn key(key: Key) -> Shortcut {
        Shortcut {
            key,
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    /// Parses `"mod+shift+n"`: modifiers and one key joined by `+`, written
    /// in lowercase (other cases are read too). `mod` is Command when
    /// `mod_is_meta` (macOS), else Control; `cmd` / `super` / `win` is
    /// Command (the Windows key, Super), then `ctrl`, `alt` / `option` and
    /// `shift`. The key is a name ([`Key::parse`]) or the US layout's
    /// unshifted character; `plus` is `+`. `None` when it is not one.
    pub fn parse(text: &str, mod_is_meta: bool) -> Option<Shortcut> {
        // `+` itself is a key: "mod+plus" (or "mod++").
        let (mods, key) = match text.strip_suffix("++") {
            Some(mods) => (mods, "+"),
            None => match text.rsplit_once('+') {
                Some((mods, key)) => (mods, key),
                None => ("", text),
            },
        };
        let mut shortcut = Shortcut::key(Key::parse(key.trim())?);
        for m in mods.split('+').map(str::trim).filter(|m| !m.is_empty()) {
            match m.to_ascii_lowercase().as_str() {
                "mod" | "cmdorctrl" | "commandorcontrol" => {
                    if mod_is_meta {
                        shortcut.meta = true;
                    } else {
                        shortcut.ctrl = true;
                    }
                }
                "cmd" | "command" | "meta" | "super" | "win" => shortcut.meta = true,
                "ctrl" | "control" => shortcut.ctrl = true,
                "alt" | "option" | "opt" => shortcut.alt = true,
                "shift" => shortcut.shift = true,
                _ => return None,
            }
        }
        Some(shortcut)
    }

    /// Its label on Windows and Linux (`Ctrl+Shift+N`).
    pub fn label(&self) -> String {
        let mut out = String::new();
        for (on, name) in [
            (self.ctrl, "Ctrl+"),
            (self.alt, "Alt+"),
            (self.shift, "Shift+"),
            (self.meta, "Win+"),
        ] {
            if on {
                out.push_str(name);
            }
        }
        out.push_str(&self.key.label());
        out
    }
}

/// A standard menu item: the platform carries it out (and labels it when
/// the item has no label of its own).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PropEnum)]
pub enum MenuRole {
    /// The about panel (macOS).
    About,
    /// Hides the app (macOS).
    Hide,
    /// Hides the other apps (macOS).
    HideOthers,
    /// Shows all apps (macOS).
    ShowAll,
    /// Quits the app.
    Quit,
    /// Undo in the focused text field.
    Undo,
    /// Redo in the focused text field.
    Redo,
    /// Cut the focused text field's selection.
    Cut,
    /// Copy the selection.
    Copy,
    /// Paste into the focused text field.
    Paste,
    /// Select all of the focused text field.
    SelectAll,
    /// Minimizes the window.
    Minimize,
    /// Maximizes (zooms) or restores the window.
    Zoom,
    /// Enters or leaves full screen.
    ToggleFullScreen,
    /// Closes the window.
    Close,
}

impl MenuRole {
    /// Every role.
    pub const ALL: &[MenuRole] = &[
        MenuRole::About,
        MenuRole::Hide,
        MenuRole::HideOthers,
        MenuRole::ShowAll,
        MenuRole::Quit,
        MenuRole::Undo,
        MenuRole::Redo,
        MenuRole::Cut,
        MenuRole::Copy,
        MenuRole::Paste,
        MenuRole::SelectAll,
        MenuRole::Minimize,
        MenuRole::Zoom,
        MenuRole::ToggleFullScreen,
        MenuRole::Close,
    ];

    /// The label of an item with this role and no label of its own.
    pub fn label(self) -> &'static str {
        match self {
            MenuRole::About => "About",
            MenuRole::Hide => "Hide",
            MenuRole::HideOthers => "Hide Others",
            MenuRole::ShowAll => "Show All",
            MenuRole::Quit => "Quit",
            MenuRole::Undo => "Undo",
            MenuRole::Redo => "Redo",
            MenuRole::Cut => "Cut",
            MenuRole::Copy => "Copy",
            MenuRole::Paste => "Paste",
            MenuRole::SelectAll => "Select All",
            MenuRole::Minimize => "Minimize",
            MenuRole::Zoom => "Zoom",
            MenuRole::ToggleFullScreen => "Toggle Full Screen",
            MenuRole::Close => "Close",
        }
    }

    /// The shortcut of an item with this role and no shortcut of its own,
    /// on a Mac (`mac`) or elsewhere.
    pub fn shortcut(self, mac: bool) -> Option<Shortcut> {
        let primary = |key: char| Shortcut {
            ctrl: !mac,
            meta: mac,
            ..Shortcut::key(Key::Char(key))
        };
        Some(match self {
            MenuRole::About | MenuRole::ShowAll => return None,
            MenuRole::Hide if mac => primary('h'),
            MenuRole::HideOthers if mac => Shortcut {
                alt: true,
                ..primary('h')
            },
            MenuRole::Hide | MenuRole::HideOthers => return None,
            MenuRole::Quit if mac => primary('q'),
            MenuRole::Quit => Shortcut {
                alt: true,
                ..Shortcut::key(Key::F(4))
            },
            MenuRole::Undo => primary('z'),
            MenuRole::Redo if mac => Shortcut {
                shift: true,
                ..primary('z')
            },
            MenuRole::Redo => primary('y'),
            MenuRole::Cut => primary('x'),
            MenuRole::Copy => primary('c'),
            MenuRole::Paste => primary('v'),
            MenuRole::SelectAll => primary('a'),
            MenuRole::Minimize if mac => primary('m'),
            MenuRole::Minimize | MenuRole::Zoom => return None,
            MenuRole::ToggleFullScreen if mac => Shortcut {
                ctrl: true,
                ..primary('f')
            },
            MenuRole::ToggleFullScreen => Shortcut::key(Key::F(11)),
            MenuRole::Close => primary('w'),
        })
    }

    /// The role edits the focused text field (its shortcut is the field's
    /// own: a menu does not take it from the field).
    pub fn edits_text(self) -> bool {
        matches!(
            self,
            MenuRole::Undo
                | MenuRole::Redo
                | MenuRole::Cut
                | MenuRole::Copy
                | MenuRole::Paste
                | MenuRole::SelectAll
        )
    }
}

/// An item of a menu that does something.
#[derive(Debug, Clone, PartialEq)]
pub struct MenuAction {
    /// Reported as [`crate::EventKind::Menu`] when chosen (0: never
    /// reported, a role-only item).
    pub id: u32,
    /// What it shows.
    pub label: String,
    /// Its keyboard shortcut.
    pub shortcut: Option<Shortcut>,
    /// It can be chosen.
    pub enabled: bool,
    /// A check mark: `Some` makes it a checkable item.
    pub checked: Option<bool>,
    /// What the platform does with it (the app is not told).
    pub role: Option<MenuRole>,
}

/// An item of a menu.
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItem {
    /// A line between groups of items.
    Separator,
    /// An item that does something.
    Action(MenuAction),
    /// An item opening a menu of its own (the top-level menus of a menu
    /// bar).
    Submenu {
        /// What it shows.
        label: String,
        /// It can be opened.
        enabled: bool,
        /// Its items.
        items: Vec<MenuItem>,
    },
}

impl MenuItem {
    /// Calls `f` with every action of `items`, submenus' too.
    pub fn each_action<'a>(items: &'a [MenuItem], f: &mut impl FnMut(&'a MenuAction)) {
        for item in items {
            match item {
                MenuItem::Action(action) => f(action),
                MenuItem::Submenu { items, .. } => MenuItem::each_action(items, f),
                MenuItem::Separator => {}
            }
        }
    }

    /// The action of `items` (or their submenus) that `shortcut` chooses:
    /// enabled, with an id or a role that is not a text field's own.
    pub fn find_shortcut<'a>(items: &'a [MenuItem], shortcut: &Shortcut) -> Option<&'a MenuAction> {
        for item in items {
            match item {
                MenuItem::Action(action)
                    if action.enabled
                        && action.shortcut.as_ref() == Some(shortcut)
                        && !action.role.is_some_and(MenuRole::edits_text) =>
                {
                    return Some(action);
                }
                MenuItem::Submenu {
                    enabled: true,
                    items,
                    ..
                } => {
                    if let Some(found) = MenuItem::find_shortcut(items, shortcut) {
                        return Some(found);
                    }
                }
                _ => {}
            }
        }
        None
    }
}

/// A desktop command (on the root).
#[derive(Debug, Clone, PartialEq)]
pub enum DesktopCommand {
    /// The window's title.
    SetTitle(String),
    /// The title bar's style.
    SetTitleBar {
        /// Shown or hidden.
        style: TitleBarStyle,
        /// The height of the app's own title bar, in points: the window
        /// controls are centered in it (Windows' caption buttons fill it);
        /// `None` for the platform's height.
        height: Option<f32>,
    },
    /// Minimizes the window.
    Minimize,
    /// Maximizes the window, or restores it when it is maximized.
    ToggleMaximize,
    /// Enters or leaves full screen.
    ToggleFullScreen,
    /// Closes the window (the app ends with its last window).
    Close,
    /// The application menu (macOS's menu bar, the window's menu bar on
    /// Windows and Linux; not shown there while the title bar is hidden,
    /// its shortcuts still work); `None` restores the platform's default.
    SetMenu(Option<Arc<[MenuItem]>>),
    /// Shows a context menu at `at` (root coordinates, points) or at the
    /// pointer.
    PopupMenu {
        /// Its items.
        items: Arc<[MenuItem]>,
        /// Where its top left corner goes.
        at: Option<(f32, f32)>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_parse() {
        let s = Shortcut::parse("mod+shift+n", false).unwrap();
        assert_eq!(
            s,
            Shortcut {
                ctrl: true,
                shift: true,
                ..Shortcut::key(Key::Char('n'))
            }
        );
        // Labels stay the platform's style.
        assert_eq!(s.label(), "Ctrl+Shift+N");
        // Other cases are read too.
        assert_eq!(Shortcut::parse("Mod+Shift+N", false), Some(s));
        let mac = Shortcut::parse("cmdorctrl+,", true).unwrap();
        assert!(mac.meta && !mac.ctrl);
        assert_eq!(mac.key, Key::Char(','));
        assert_eq!(
            Shortcut::parse("ctrl+plus", false).unwrap().key,
            Key::Char('+')
        );
        assert_eq!(
            Shortcut::parse("ctrl++", false).unwrap().key,
            Key::Char('+')
        );
        assert_eq!(Shortcut::parse("alt+f4", false).unwrap().key, Key::F(4));
        assert_eq!(
            Shortcut::parse("f11", false).unwrap(),
            Shortcut::key(Key::F(11))
        );
        let win = Shortcut::parse("win+option+pagedown", true).unwrap();
        assert!(win.meta && win.alt && win.key == Key::PageDown);
        assert_eq!(Shortcut::parse("hyper+k", false), None);
        assert_eq!(Shortcut::parse("ctrl+f25", false), None);
    }

    #[test]
    fn shortcuts_find_items() {
        let copy = Shortcut::parse("ctrl+c", false).unwrap();
        let new = Shortcut::parse("ctrl+n", false).unwrap();
        let action = |id, shortcut, role| {
            MenuItem::Action(MenuAction {
                id,
                label: String::new(),
                shortcut: Some(shortcut),
                enabled: true,
                checked: None,
                role,
            })
        };
        let menu = vec![MenuItem::Submenu {
            label: "File".into(),
            enabled: true,
            items: vec![
                action(1, new, None),
                MenuItem::Separator,
                action(0, copy, Some(MenuRole::Copy)),
            ],
        }];
        assert_eq!(MenuItem::find_shortcut(&menu, &new).map(|a| a.id), Some(1));
        // Text fields keep their own shortcuts.
        assert!(MenuItem::find_shortcut(&menu, &copy).is_none());
    }
}
