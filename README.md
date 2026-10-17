# native-shell (nsh)

**AI-generated UI, native in your app — on every platform.** native-shell is designed for UI that a model writes and
a native app loads after it ships: the app embeds an nsh view, and nsh runs TSX modules in it — signed packages the app
verifies, each in a sandboxed QuickJS VM with host-set limits and capabilities — drawn with the platform's own UI. The
same mechanism makes native apps extensible in general (plugins, features delivered after release); standalone apps
are a second use.

nsh runs on phones and desktops, each with a native backend of its own:

| Platform | Backend | Embed with | Requirements |
|---|---|---|---|
| iOS | UIKit, CoreText | `NshAppView` (Swift, `NativeShell.xcframework`) | iOS 15.0+ |
| Android | Android views, `StaticLayout` | `NshView` (Java / Kotlin, `nsh.aar`) | Android 9 (API 28)+ |
| macOS | AppKit, Core Animation, CoreText | `AppView` (Rust, `nsh-platform-macos`) | macOS 14+ |
| Windows | Win32 windows drawn with Direct2D, DirectWrite | `AppView` (Rust, `nsh-platform-windows`) | Windows 10 1809+ |
| Linux | GTK 4 widgets, Pango | `AppView` (Rust, `nsh-platform-linux`) | GTK 4.14+ |

Every platform runs apps with hot reload (`nsh run`) and packages them (`nsh package`). Mac Catalyst (an iPhone-sized
window on a Mac) is a development target, and a WebAssembly build runs the website's live demos. A GPUI desktop
backend is in development and not available yet. The host stays a native app; nsh is the layer inside it.

Inside, only a module's code is JavaScript: signals, the renderer and every module it imports are Rust, layout is
Taffy, and the toolchain is a single `nsh` binary (no Node.js, and no Rust toolchain for module authors).

The components of the mobile UI set (`import { Button, Card } from "nsh/ui/mobile"`) are written in Rust and run in
the core, laid out with the rest of the app: one implementation, the same look on every platform, and JS only
describes the UI and handles the events. Desktop layouts use the primitives with `desktop={{…}}` props and
`Platform.select`; there is no desktop UI set yet.

## Related project: GPUI Kit

[GPUI Kit](https://gpui-kit.com) is our sibling project: components and tools for building desktop apps with GPUI.
nsh's theme colors follow gpui-kit's fields, and the GPUI desktop backend of nsh builds on it. The documentation
site's top-left menu switches between the two projects.

## Running the examples

| Example | What it shows |
|---|---|
| `examples/gallery` | A demo index covering what nsh supports: lists (recycling, columns, sticky sections, an inverted chat, horizontal lists), navigation (header options, modals, the back button), forms and the keyboard, controls, gestures, native animations, images, styles, text, web view and video, environment / timers / fetch |
| `examples/counter` | The smallest app |
| `examples/hopper` | Hopper Lab: a dark, tabbed dashboard for test hops of a small lander, built only from primitives: interactive canvas charts (scrub, drag handles, resize zones), a slider, switches, menus, native animations and navigation. `FINDINGS.md` lists what it found missing in nsh |
| `examples/bench` | Rust components against the same component in TSX (see its README) |
| `examples/embed-ios`, `examples/embed-android` | nsh inside existing native apps (see their READMEs), running `examples/embedded` with a native component the host app registers |

Every `nsh run` starts a dev server, launches the app and updates it on every save (hot module replacement: only the
edited components render again, the rest of the app keeps its state). Logs and errors appear in the terminal.

### With the distribution (no Rust needed)

`dist/nsh` (made by `cargo xtask dist`) contains the CLI and prebuilt runtimes:

```sh
dist/nsh/bin/nsh run examples/gallery                      # Mac Catalyst window (390×844) on macOS, a GTK window on Linux, a Win32 window on Windows
dist/nsh/bin/nsh run examples/gallery --target ios         # iOS simulator (boots one if none is running)
dist/nsh/bin/nsh run examples/gallery --target android     # Android emulator or device
dist/nsh/bin/nsh run examples/gallery --target macos       # the AppKit app on macOS (Windows and Linux run their own by default)
```

Requirements: Xcode for the Mac and iOS targets; GTK 4 (4.14 or later) for Linux; Windows 10 1809 or later for
Windows (the WebView2 Runtime for web views); for Android, the Android SDK (`ANDROID_HOME`, or
`~/Library/Android/sdk`, `%LOCALAPPDATA%\Android\Sdk` on Windows) with a running emulator or a device connected over `adb`. Pick a simulator with
`--device "iPhone 16 Pro"`.

### From a checkout

`cargo run` runs the gallery on the machine's own backend (an AppKit window on macOS, a GTK window on Linux, a Win32 window on Windows), with hot reload; it rebuilds and restarts it when the Rust
code changes. Other apps and targets:

```sh
cargo run -- examples/counter
cargo run -- --target catalyst     # Mac Catalyst window
cargo run -- --target ios          # iOS simulator
cargo run -- --target android      # Android emulator or device
```

`./target/debug/nsh run <app>` works as well (after `cargo build -p nsh-cli`): in a checkout it builds the development
host it needs.

### Other ways to run an app

`nsh` below is `dist/nsh/bin/nsh` or `./target/debug/nsh`.

```sh
nsh dev examples/gallery                                    # dev server only; prints the URLs for hosts on this machine and the LAN
nsh trace examples/counter                                  # headless: prints the native view operations and timings
nsh package examples/gallery --target ios-simulator --run   # an installable app (.app / .ipa / .apk)
nsh build examples/counter --release                        # bundle.js / bundle.nshc for embedding
nsh publish --key release.secret --cert release.cert        # a signed package (.nshpkg) for apps to download and verify
```

Signing keys: `nsh keygen` (a root key kept offline and release keys), `nsh certify` (the root certifies a release
key for some app ids), `nsh verify`, `nsh revoke` (a signed revocation list). See [docs/dev/signed-packages.md](docs/dev/signed-packages.md).

## A new app

```sh
nsh init my-app
nsh run my-app
```

`nsh init` creates `App.tsx`, `nsh.json` (name, id, version, icon) and a `tsconfig.json`; type declarations for
editors are kept in `my-app/.nsh/types`, so TypeScript checking works without `node_modules`.

Documentation for app developers is the website in [website/](website/README.md) (guides, element, component and
API reference in English and Chinese, with live demos running on nsh's WebAssembly runtime).

Contributors: see [CONTRIBUTING.md](CONTRIBUTING.md) (pull requests, commits, testing), [AGENTS.md](AGENTS.md)
(commands, rules, conventions) and [docs/dev](docs/dev/README.md) (architecture and internals).

## License

Licensed under the [Apache License, Version 2.0](LICENSE-APACHE).

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as
defined in the Apache-2.0 license, shall be licensed under Apache-2.0, without any additional terms or conditions.
