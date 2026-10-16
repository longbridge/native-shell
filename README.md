# native-shell (nsh)

**AI-generated UI, native in your app.** native-shell is designed for UI that a model writes and a native app loads
after it ships: an iOS or Android app embeds an `NshAppView` / `NshView`, and nsh runs TSX modules in it — signed
packages the app verifies, each in a sandboxed QuickJS VM with host-set limits and capabilities — rendered with the
system's own views (UIKit, Android views). The same mechanism makes native apps extensible in general (plugins,
features delivered after release); standalone apps are a second use. Mobile clients come first; desktop clients follow
through a GPUI backend in development. The host stays a native app; nsh is the layer inside it.

Inside, only a module's code is JavaScript: signals, the renderer and every module it imports are Rust, layout is
Taffy, and the toolchain is a single `nsh` binary (no Node.js, and no Rust toolchain for module authors).

The components of the mobile UI set (`import { Button, Card } from "nsh/ui/mobile"`) are written in Rust and run in
the core, laid out with the rest of the app: one implementation, the same look on iOS and Android, and JS only
describes the UI and handles the events.

## Running the examples

| Example | What it shows |
|---|---|
| `examples/gallery` | A demo index covering what nsh supports: lists (recycling, columns, sticky sections, an inverted chat, horizontal lists), navigation (header options, modals, the back button), forms and the keyboard, controls, gestures, native animations, images, styles, text, web view and video, environment / timers / fetch |
| `examples/counter` | The smallest app |
| `examples/bench` | Rust components against the same component in TSX (see its README) |
| `examples/embed-ios`, `examples/embed-android` | nsh inside existing native apps (see their READMEs), running `examples/embedded` with a native component the host app registers |

Every `nsh run` starts a dev server, launches the app and updates it on every save (hot module replacement: only the
edited components render again, the rest of the app keeps its state). Logs and errors appear in the terminal.

### With the distribution (no Rust needed)

`dist/nsh` (made by `cargo xtask dist`) contains the CLI and prebuilt runtimes:

```sh
dist/nsh/bin/nsh run examples/gallery                      # Mac Catalyst window (390×844), macOS only
dist/nsh/bin/nsh run examples/gallery --target ios         # iOS simulator (boots one if none is running)
dist/nsh/bin/nsh run examples/gallery --target android     # Android emulator or device
```

Requirements: Xcode for the Mac and iOS targets; for Android, the Android SDK (`ANDROID_HOME`, or
`~/Library/Android/sdk`) with a running emulator or a device connected over `adb`. Pick a simulator with
`--device "iPhone 16 Pro"`.

### From a checkout

`cargo run` runs the gallery in the Mac Catalyst window, with hot reload; it rebuilds and restarts it when the Rust
code changes. Other apps and targets:

```sh
cargo run -- examples/counter
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
nsh build examples/counter --release                        # bundle.js / bundle.qbc for embedding
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

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
