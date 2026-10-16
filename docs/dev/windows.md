# The Windows backend (Win32, Direct2D)

`crates/nsh-platform-windows` renders nsh apps in Win32 windows: the views are drawn with Direct2D, text is laid out
with DirectWrite. `crates/nsh-windows-host` is its development host (`nsh run --target windows`, the default on
Windows), `crates/nsh-windows-app` the release runner of `nsh package --target windows`. It follows the Linux
backend's design module by module (`docs/dev/linux.md`): one retained view per node, the core lays everything out.
User-facing page: `website/docs/windows.md`.

## Building and checking

Windows 10 1809 or later (Windows 11 recommended), the MSVC toolchain (Visual Studio Build Tools with the C++
workload, for QuickJS) and the Windows SDK. Off Windows the crates are empty (like the other platform crates off their
targets), so check them on Windows:

```sh
cargo clippy -p nsh-platform-windows -p nsh-windows-host -p nsh-windows-app --all-targets
cargo test -p nsh-platform-windows          # needs no window: tests do not open one
cargo build -p nsh-windows-host             # target\debug\nsh-windows-host.exe
```

The Windows binaries are built for the machine itself, so the CLI finds them in `target\<profile>` and builds them
first in a checkout; `cargo xtask dist` on Windows puts them in `dist\nsh\runtime\windows\` (`docs/dev/tooling.md`,
`embedding.md`). CI: `.github/workflows/windows.yml`.

Features: `js` (default; JS apps; without it the crate only runs Rust apps and links no QuickJS) and `dev` (the dev
server client, log forwarding, the error overlay: `dev.rs`).

## Architecture

- **One retained view per node, one window per surface.** Each node is an `NshView` (`view.rs`): an `Rc` node with
  its `ViewState` (frame, style, animated values, content), its children in drawing order and a weak parent. There are
  no child windows: a surface's window draws its whole tree with Direct2D into a flip-model swap chain
  (`render.rs`), so nothing nsh draws is ever covered by another `HWND` (the "airspace" problem of mixing child windows
  with DirectX content). Changing a view (`changed` / `moved`) invalidates the window; `WM_PAINT` draws the tree once,
  however many views changed.
- **Drawing a view** (`render::draw_view`): shadows and background (`decor.rs`), the content clipped to the bounds
  (text, image, canvas), the children (clipped to the rounded bounds when the view clips), the items the view hosts
  (`ViewState::natives`: text editors, headers, dialogs, overlays — the `Hosted` trait), the scroll indicator, then the
  border, all inside an opacity layer when translucent. Transforms are CSS-like 4x4 matrices (`geom.rs`); affine ones
  are Direct2D's 3x2 world transform, projective ones (perspective) draw the subtree into an offscreen bitmap mapped
  with `DrawBitmap`'s perspective transform. Units are points: the device context's DPI is the window's.
- **One hop per commit.** The engine runs on its own thread (`nsh_engine::Engine`); its sink queues each commit and
  posts one `WM_NSH_WAKE` to the window, and the main thread applies every queued batch in order (`surface.rs`,
  `mount.rs`). Worker threads (image decoding, media callbacks) run code on the main thread with `window::invoke`
  (a message-only dispatcher window) and find their surface by key; no globals are shared between surfaces.
- **Text is typeset on the engine thread** with DirectWrite's shared, free-threaded factory; the final
  `IDWriteTextLayout` is handed to the main thread, which only draws it and asks it where characters are (`text.rs`).
- **Frames**: a ticker thread waits for each composition (`DwmFlush`) and posts `WM_NSH_TICK` while something
  animates: native animation values (`animation.rs`), JS frame requests, transitions and keyframes (`transition.rs`),
  scroll glides, navigation transitions, video frames (`Surface::on_frame`). The ticker parks when nothing animates.
- **Devices**: one Direct3D 11 device (hardware, else WARP) and its Direct2D device and context per thread
  (`gfx.rs`); a lost device (driver update, GPU reset) is made again and `gfx::generation` tells caches of device
  resources (bitmaps, command lists) to make theirs again.
- **Embedding**: `AppView` (`app.rs`) is one surface as a window: a child of the embedding app's window, or top-level;
  `run_app` opens one and runs the message loop. `set_ui_sets` replaces the default `nsh/ui/mobile` set for apps with
  Rust components of their own.

## Checking a change: `--script`

`nsh-windows-host --script "<steps>"` (or `AppOptions::script`) runs steps once the window shows, separated by `;` or
new lines (`script.rs`), so a change can be checked without touching the screen. Steps run on a thread of their own and
each one on the main thread; input is synthesized at the input module's entry points, so it works while the window is
in the background or covered.

| Step | Does |
|---|---|
| `wait <ms>` | Waits. |
| `shot <file.png>` | Draws the window's content offscreen at scale 1 into a PNG, whatever covers the window. |
| `tap <x> <y>` | Presses and releases the mouse at a point (points, root coordinates). |
| `drag <x0> <y0> <x1> <y1> [ms]` | Presses, moves in steps over `ms` (default 300), releases; `touch` the same with a finger. |
| `wheel <x> <y> <dx> <dy>` | A wheel scroll (points, positive toward the end); `pad` a precision touchpad's. |
| `hover <x> <y>` | Moves the pointer. |
| `type <text>` | Types into the focused field. |
| `key <name>` | Presses a key (`Return`, `BackSpace`, `Tab`, `Escape`, `Left`, `Page_Down`, `F5`, a letter; `ctrl+` / `shift+` / `alt+` prefixes); `selectall` is Ctrl+A. |
| `dark` / `light` | Switches the system's appearance (an app that forces one keeps it). |
| `resize <w> <h>` | Resizes the window's client area (points). |
| `quit` | Closes the window. |

```sh
nsh build examples/gallery -o build
target\debug\nsh-windows-host --bundle build\bundle.qbc --size 1200x800 --script "wait 800; shot a.png; tap 100 200; wait 300; shot b.png; quit"
```

Without `--bundle` the host connects to the dev server on this machine (`--dev-server ws://…/ws` names another);
`--trace` logs per-commit timings.


