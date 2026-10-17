---
title: GPUI
description: Run nsh apps drawn by GPUI on macOS, Windows and Linux, package them, embed them in a GPUI or GPUI Kit app with AppView, and what the GPUI backend does differently.
order: 15.5
group: platforms
---

# GPUI

The GPUI backend draws nsh apps with [GPUI](https://gpui-kit.com) (GPUI Fast), the GPU-accelerated UI framework that
GPUI Kit is built on, on macOS, Windows and Linux alike. Its main use is embedding: an `AppView` is a GPUI view, so a
GPUI or GPUI Kit app shows nsh modules anywhere in its own element tree. The core, the reactive graph, the renderer,
Taffy layout and the `nsh/ui/mobile` components are the same as on every other platform, so one bundle runs
everywhere. `Platform.OS` is the OS it runs on (`"macos"`, `"windows"` or `"linux"`) and the `desktop={{…}}` props
apply.

> [!WARNING]
> The GPUI backend is in development. Parity with the native [macOS](./macos.md) (AppKit), [Windows](./windows.md)
> (Win32) and [Linux](./linux.md) (GTK) backends is being worked on; until then, some elements and modules are
> missing or draw differently. For shipping desktop apps today, use the native backends.

## How it works

- **One element per surface.** A surface is one GPUI element that fills the space its parent gives it and paints the
  whole view tree with GPUI's primitives: quads, shadows, paths, glyphs and images. nsh keeps one retained view per
  node, as on Windows, and redraws only when a commit changed something.
- **Text is shaped on the engine thread** with GPUI's text system (the app's own fonts), broken into lines by nsh;
  the main thread only paints the glyphs.
- **One hop per commit.** The engine runs on its own thread; each commit is one closure sent to GPUI's main thread,
  which notifies the view.
- **Input** comes from GPUI's mouse, scroll, key and input method events; hit testing, presses, the responder system,
  scrolling and text editing are the same logic as the Windows backend's.

## Requirements

- macOS, Windows or Linux, with a GPU GPUI can draw with (Metal on macOS, DirectX on Windows, Vulkan on Linux).
- On Linux: a Vulkan driver, and Wayland or X11 with `libxkbcommon`. Building the runtime from a checkout also needs
  their development packages; on Debian and Ubuntu:

  ```sh
  sudo apt-get install build-essential pkg-config libasound2-dev libfontconfig-dev libfreetype-dev libssl-dev \
    libvulkan1 libwayland-dev libx11-xcb-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libzstd-dev
  ```

## Running an app

```sh
nsh run my-app --target gpui
```

Opens the app in a GPUI window (1200×800 by default), with hot reload, logs in the terminal and the error overlay as
on the other targets. The development host (`nsh-gpui-host`) runs in place on every OS, with no app bundle around it.
In a checkout, `nsh run` builds it first; the distribution ships it in `runtime/gpui/`. The host also runs a built
bundle on its own:

```sh
nsh build my-app --release -o build
dist/nsh/runtime/gpui/nsh-gpui-host --bundle build/bundle.nshc --size 390x844   # in a checkout: target/debug/nsh-gpui-host
```

`--size` is the window's size in points (`<width>x<height>`).

## Packaging

```sh
nsh package my-app --target gpui [--run]
```

The package is laid out as the native backend's of the machine it is made on, with GPUI's release runner
(`nsh-gpui-app`) as the executable:

| Packaged on | Output |
|---|---|
| macOS | an ad hoc signed `<Name>.app`, as [`--target macos`](./macos.md#packaging) makes |
| Windows | a directory `<Name>\` with `<Name>.exe` and `nsh\`, and `<Name>.zip`, as [`--target windows`](./windows.md#packaging) makes |
| Linux | a directory `<Name>/` with the executable, `nsh/` and a `.desktop` file, and `<Name>.tar.gz`, as [`--target linux`](./linux.md#packaging) makes |

As with the native backends, the runner is built for the machine it is built on: package a Mac app on a Mac, a
Windows app on Windows and a Linux app on Linux. `--run` launches the app after packaging it.

## Embedding in a GPUI app

The `nsh-platform-gpui` crate (from the nsh repository) gives a Rust GPUI app `AppView`: one nsh engine as a GPUI
view (`Entity<AppView>`), which fills the space its parent element gives it. Each `AppView` is independent; several
can run at once.

```rust
use gpui::{AppContext, Application, WindowOptions};
use nsh_platform_gpui::{App, AppOptions, AppView, Bundle};

gpui_platform::application().run(|cx| {
    cx.open_window(WindowOptions::default(), |window, cx| {
        cx.new(|cx| {
            let view = AppView::new(AppOptions::default(), window, cx);
            let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
            view.load(App::Js(Bundle::source(code)));
            view
        })
    })
    .unwrap();
});
```

Put the entity in your element tree like any other view (a GPUI Kit `Root`, a panel, a dock). `AppView::load`
replaces whatever ran; `stop` (or dropping the view) ends the app. `AppOptions::asset_base` tells it where `asset:/`
URIs resolve (`AssetBase::Directory` or `AssetBase::Http`). Bytecode bundles (`bundle.nshc`) load with
`unsafe { Bundle::bytecode(bytes) }` (or `Bundle::read_trusted(path)`, unsafe too): QuickJS bytecode is trusted input,
so only load it from your own app's files.

- `open_app_window(source, options, cx)` opens a window with one `AppView` in a running GPUI app.
- `run_app(source, options)` starts GPUI and runs one app in a window until it closes (what the release runner does).
- `AppView::register_component(name, factory)` registers a native component of yours before loading
  ([Native components](./native-components.md)).
- `set_ui_sets(sets)` replaces the default `nsh/ui/mobile` set, for apps with Rust components of their own.

## Differences

What is known to differ from the native desktop backends:

- **Transforms**: GPUI has no transform for quads and glyphs. Translations and scales draw exactly; a view that is
  rotated or skewed draws at its bounds.
- **Opacity** is multiplied into the colors of what a view draws rather than drawn as a group, so overlapping
  children of a translucent view show through each other.
- **Web views and native views** are platform views composited through GPUI's native surfaces.
- **The rest** of what the native backends support is being ported; see each backend's own list for comparison:
  [macOS](./macos.md#what-differs-on-macos), [Windows](./windows.md#what-differs-on-windows),
  [Linux](./linux.md#what-differs-on-linux).

## See also

- [CLI](./cli.md): `nsh run --target gpui`, `nsh package --target gpui`
- [Packaging](./packaging.md)
- [Platform overview](./platforms.md)
