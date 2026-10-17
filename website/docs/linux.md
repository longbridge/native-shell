---
title: Linux
description: Run nsh apps on Linux in GTK 4 windows, package them, embed them in a GTK app with AppView, and what the Linux backend does not support.
order: 15
group: platforms
---

# Linux

nsh apps also run on Linux. The Linux backend renders them with GTK 4 widgets and lays out text with Pango; the core,
the reactive graph, the renderer, Taffy layout and the `nsh/ui/mobile` components are the same as on iOS and Android,
so one bundle runs everywhere. On Linux, `Platform.OS` is `"linux"` and the `desktop={{…}}` props apply.

## Requirements

- **GTK 4.14** or later. To build nsh's Linux runtime (from a checkout, or `cargo xtask
  dist` on Linux) you need their development packages and clang: `libgtk-4-dev clang` on Debian and Ubuntu (24.04 or
  later), `gtk4 clang` on Arch, `gtk4-devel clang` on Fedora.
- **GStreamer** plugins for the video formats your app plays (GTK plays video through GStreamer).

## Running an app

On Linux, `nsh run` opens the app in a GTK window (390×844 by default), with hot reload, logs in the terminal and the
error overlay as on the other targets:

```sh
nsh run my-app                 # --target linux is the default on Linux
```

In a checkout, `nsh run` builds the development host (`nsh-linux-host`) first; the distribution made on Linux ships
it in `runtime/linux/`. Ctrl-C closes the window, and closing the window ends `nsh run`. The host also runs a built
bundle on its own:

```sh
nsh build my-app --release -o build
dist/nsh/runtime/linux/nsh-linux-host --bundle build/bundle.nshc --size 1024x768   # in a checkout: target/debug/nsh-linux-host
```

`--size` is the window's default size; tiling window managers may give the window another one.

## Packaging

```sh
nsh package my-app --target linux [--run]
```

Writes a directory and the same as an archive:

```text
out/<Name>/
  <Name>                 the executable (nsh's release runner)
  nsh/bundle.nshc        your app, compiled
  nsh/assets/…           the assets it imports
  nsh/app.json           the window title and the application id
  <id>.desktop           a desktop entry (StartupWMClass is the id)
  <id>.png               the icon, when nsh.json names one
out/<Name>.tar.gz
```

The executable finds `nsh/` next to itself (a symbolic link to it, in `~/.local/bin` say, works too). GTK 4 and Pango
must be installed where the app runs. To show the app in a desktop's launcher, put the executable (or a link) on the
`PATH`, the `.desktop` file in `~/.local/share/applications` and the icon in an icon theme directory. `--run` launches
the app after packaging it.

## Embedding in a GTK app

The `nsh-platform-linux` crate (from the nsh repository) gives a Rust GTK app `AppView`: one nsh engine as a GTK widget.
Each `AppView` is independent; several can run at once.

```rust
use std::rc::Rc;
use nsh_platform_linux::{gtk4 as gtk, AppOptions, AppView, AssetBase, Bundle};
use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder().application_id("com.example.host").build();
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder().application(app).default_width(390).default_height(844).build();
        let view = AppView::new(AppOptions {
            asset_base: Some(AssetBase::Directory("app/assets".into())),
            ..AppOptions::default()
        });
        // register native components here, before loading
        let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
        view.load(Bundle::source(code).into());
        window.set_child(Some(&view.widget()));
        // The window keeps the view; closing it stops the app.
        let keep = std::cell::RefCell::new(Some(Rc::clone(&view)));
        window.connect_destroy(move |_| if let Some(view) = keep.take() { view.stop() });
        window.present();
    });
    app.run()
}
```

`AppView::load` replaces whatever ran; `stop` ends the app. `asset_base` tells it where `asset:/` URIs resolve
(`AssetBase::Directory` or `AssetBase::Http`). Bytecode bundles (`bundle.nshc`) load with `unsafe { Bundle::bytecode(bytes) }`:
QuickJS bytecode is trusted input, so only load it from your own app's files.

Native components are registered with `AppView::register_component`; see [Native components](./native-components.md#registering-on-linux-rust).

## What differs on Linux

The environment is a desktop's: the safe area and the keyboard height are 0, `Platform.constants.interfaceIdiom` is
`"desktop"`, `Platform.Version` is the kernel's major and minor version, and dark mode follows the desktop's setting
(through the XDG settings portal) live. See [Device and environment](./device-and-environment.md).

### Keyboard

| Keys | Do |
|---|---|
| Return | In an `Input`, what `submitBehavior` says: submit (`onSubmitEditing`), submit and leave the field, or a new line (multiline inputs) |
| Shift+Return, Alt+Return | A new line in a multiline input whose return submits (Ctrl+Return submits) |
| Tab, Shift+Tab | The next / previous field in screen order, within the newest modal |
| Escape | Leaves the focused field; else clears a text selection, then goes back or asks the newest modal to close |
| Alt+Left, Back key, mouse back button | Go back (also from a field) |
| Ctrl+arrows, Home / End, with Shift; Ctrl+Backspace / Delete | Move, select and delete by word and line in a field, as in every GTK app |
| Ctrl+Z, Ctrl+Shift+Z, Ctrl+X / C / V / A | Undo, redo, cut, copy, paste, select all (a secure field refuses cut and copy); Ctrl+A / Ctrl+C select and copy `selectable` text |
| Arrows; Page Up / Down, Space, Shift+Space; Home / End | Scroll the scroll view around the focused view or under the mouse by a line, a page, to the start / end (not while a field has the focus) |
| Ctrl+W, F11, Ctrl+Q | Close the window, full screen, quit (`nsh package` and `nsh run` apps) |
| Ctrl+R | Reloads the latest build (`nsh run`) |

The app's own shortcuts ([`nsh/keymap`](../modules/nsh-keymap.md)) are a window-wide shortcut controller; Ctrl+Q and
F11 cannot be bound.

Shortcuts work with any keyboard layout (on a Cyrillic or Greek one, Ctrl+C is the key that types C on a US layout).
A return that commits an input method's composition (Chinese, Japanese, ...) never submits. `onKeyPress` reports
what React Native reports: the typed text, `"Enter"`, `"Backspace"`, and `"Escape"`; pasted text is not reported.

Not supported:

- **`WebView`**: it would need WebKitGTK; a web view shows a neutral placeholder.
- **Video `rate`**: GTK plays at normal speed only. `onEnd` only comes for videos that do not loop.
- **Zooming scroll views** (`zoomTo`, the zoom props).
- **`mixBlendMode` and `backdropFilter`** (the background color still draws); `borderCurve: "continuous"` draws
  circular corners.
- **Input accessory views** and the **status bar** (no desktop equivalent).
- **Accessibility**: `accessible` grouping, custom `accessibilityActions`; live regions are announced instead.

## See also

- [CLI](./cli.md): `nsh run --target linux`, `nsh package --target linux`
- [Packaging](./packaging.md)
- [Platform-specific code](./platform-specific.md)
