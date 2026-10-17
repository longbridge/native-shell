---
title: macOS
description: Run nsh apps on macOS in AppKit windows, package them as .app bundles, embed them in a Mac app with AppView, and what the macOS backend does not support.
order: 3.24
---

# macOS

nsh apps also run as native Mac apps. The macOS backend renders them with AppKit views and Core Animation and lays out
text with CoreText; the core, the reactive graph, the renderer, Taffy layout and the `nsh/ui/mobile` components are the
same as on iOS and Android, so one bundle runs everywhere. On macOS, `Platform.OS` is `"macos"` and the
`desktop={{…}}` props apply.

This is not [Mac Catalyst](./getting-started.md#mac-catalyst-default-on-macos), which stays the default `nsh run`
target on macOS: Catalyst runs the iOS backend (UIKit) in an iPhone-sized window and behaves like a phone. The macOS
backend behaves like a desktop app: mouse, trackpad and keyboard input, resizable windows, centered sheets and a menu
bar.

## Requirements

- **macOS 14** or later, where the app runs.
- Xcode's command line tools (`xcode-select --install`), to build nsh's macOS runtime from a checkout and to package
  apps (`codesign`, `sips`). The distribution made with `cargo xtask dist` on a Mac ships the runtime in
  `runtime/macos/`.

## Running an app

```sh
nsh run my-app --target macos
```

The app opens in an AppKit window (1200×800 by default), with hot reload, logs in the terminal and the error overlay as
on the other targets; Cmd+R reloads the latest build. In a checkout, `nsh run` builds the development host
(`nsh-macos-host`) first. Stopping `nsh run` closes the window, and closing the window ends `nsh run`. The host also
runs a built bundle on its own:

```sh
nsh build my-app --release -o build
dist/nsh/runtime/macos/nsh-macos-host --bundle build/bundle.qbc --size 1200x800   # in a checkout: target/debug/nsh-macos-host
```

`--size` is the window's initial size; the window can be resized.

## Packaging

```sh
nsh package my-app --target macos [--run]
```

Writes an app bundle:

```text
out/<Name>.app/Contents/
  Info.plist
  MacOS/<Name>               the executable (nsh's release runner)
  Resources/nsh/bundle.qbc   your app, compiled
  Resources/nsh/assets/…     the assets it imports
  Resources/nsh/app.json     the window title and the app's appearance
  Resources/AppIcon.icns     the icon, when nsh.json names one (converted with sips)
```

The `.app` is signed ad hoc, so it runs on the Mac that built it. To distribute it, sign it with a Developer ID and
notarize it. `--run` opens the app after packaging it.

## Embedding in a Mac app

The `nsh-platform-macos` crate (from the nsh repository) gives a Rust Mac app `AppView`: one nsh engine as an `NSView`.
Each `AppView` is independent; several can run at once.

```rust
use std::rc::Rc;
use nsh_platform_macos::{
    objc2::MainThreadMarker, objc2_app_kit::NSWindow, AppOptions, AppView, AssetBase, Bundle,
};

fn show(window: &NSWindow) -> Rc<AppView> {
    let mtm = MainThreadMarker::new().expect("main thread");
    let view = AppView::new(mtm, AppOptions {
        asset_base: Some(AssetBase::Directory("app/assets".into())),
        ..AppOptions::default()
    });
    // register native components here, before loading
    let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
    view.load(Bundle::source(code).into());
    window.setContentView(Some(&view.view()));
    // Keep the view while the window shows it; `stop` it when the window closes.
    view
}
```

`AppView::load` replaces whatever ran; `stop` ends the app. `asset_base` tells it where `asset:/` URIs resolve
(`AssetBase::Directory` or `AssetBase::Http`). Bytecode bundles (`bundle.qbc`) load with
`unsafe { Bundle::bytecode(bytes) }`: QuickJS bytecode is trusted input, so only load it from your own app's files.
For an app that is only an nsh app, `run_app(source, options)` opens one `AppView` in a window with a standard menu
(Hide, Quit, Edit, Minimize, Close) and runs until the window closes.

Native components are registered with `AppView::register_component`; see
[Native components](./native-components.md#registering-on-macos-rust).

## What differs on macOS

The environment is a desktop's: the safe area and the keyboard height are 0, `Platform.constants.interfaceIdiom` is
`"desktop"`, `Platform.Version` is the macOS version, and dark mode follows the system's appearance live. See
[Device and environment](./device-and-environment.md).

Input is a desktop's: the mouse, the trackpad (scrolling with momentum, pinch and rotation) and the keyboard.
`selectable` text is selected by dragging, double- and triple-clicking, and copied with Cmd+C. Escape, Option+Left,
Cmd+[ and the mouse's back button go back in a stack or ask a modal to close; page and form sheets are cards centered
in the window over a dimmed backdrop. `Alert` shows an `NSAlert` sheet on the window.

Not supported, or different:

- **Zooming scroll views** (`zoomTo`, the zoom props).
- **`mixBlendMode`**, and `brightness()` above 1.
- **Inputs**: content types and autofill, keyboard types, autocapitalization, letter spacing; input accessory views
  are never shown (there is no software keyboard) and `keyboardShouldPersistTaps` is ignored.
- **`WebView`'s `scrollEnabled`** is ignored (macOS's web view has no scroll view to turn off).
- **Accessibility**: custom `accessibilityActions`.
- **The status bar** and **haptics** (no desktop equivalent).

## See also

- [CLI](./cli.md): `nsh run --target macos`, `nsh package --target macos`
- [Packaging](./packaging.md)
- [Platform-specific code](./platform-specific.md)
