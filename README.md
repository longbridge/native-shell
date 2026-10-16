# native-shell (nsh)

A React Native–like UI framework: apps are written in TSX, run in QuickJS on a native runtime (signals, the renderer
and every module apps import are Rust), are laid out by Taffy and render with the system's native views (UIKit on
iOS / Mac Catalyst, Android views). The toolchain is a single `nsh` binary written in Rust: no Node.js, and no Rust
toolchain for app developers.

The components of the mobile UI set (`import { Button, Card } from "nsh/ui/mobile"`) are written in Rust and run in
the core, laid out with the rest of the app: one implementation, the same look on iOS and Android, and JS only
describes the UI and handles the events.

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

Contributors: see [CONTRIBUTING.md](CONTRIBUTING.md) (pull requests, commits, testing), [AGENTS.md](AGENTS.md)
(commands, rules, conventions) and [docs/dev](docs/dev/README.md) (architecture and internals).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this project by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
