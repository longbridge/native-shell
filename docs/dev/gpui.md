# The GPUI backend (macOS, Windows, Linux)

`crates/nsh-platform-gpui` renders nsh apps with GPUI (GPUI Fast: crates.io `gpui-fast` 0.1.x, aliased as `gpui` in
the workspace), on macOS, Windows and Linux alike. `crates/nsh-gpui-host` is its development host (`nsh run --target
gpui`), `crates/nsh-gpui-app` the release runner of `nsh package --target gpui`. Its main purpose is embedding: an
`AppView` is a GPUI view, so GPUI and GPUI Kit apps (`gpui_component`) show nsh modules anywhere in their element tree.
The crate started as a port of the Windows backend (`windows.md`): one retained view per node, nsh draws every view
itself, and the input, scrolling, transition, keymap and editing logic is the Windows backend's, platform-free. Only
drawing, text, the main-thread queue, events and the window are GPUI's. User-facing page: `website/docs/gpui.md`.

The backend is in development: the sections below say what is ported, and "Not implemented, or different" lists what
is not yet.

## Building and checking

On Linux GPUI needs the X11, Wayland, xkbcommon, fontconfig and FreeType development packages and a Vulkan driver
(GPUI Fast's CI list, also in `.github/workflows/gpui.yml`: `libfontconfig-dev libfreetype-dev libwayland-dev
libx11-xcb-dev libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libzstd-dev libvulkan1`, plus `libgtk-3-dev
libwebkit2gtk-4.1-dev` for web views); macOS needs Xcode (Metal), Windows the MSVC toolchain and LLVM as for the
Windows backend. Off the three desktops the crates are empty.

```sh
cargo clippy -p nsh-platform-gpui -p nsh-gpui-host -p nsh-gpui-app --all-targets --features nsh-platform-gpui/dev
cargo test -p nsh-platform-gpui -p nsh-gpui-app   # needs no display: tests open no window
cargo build -p nsh-gpui-host                      # target/debug/nsh-gpui-host
```

The GPUI binaries are built for the machine itself, so the CLI finds them in `target/<profile>` and builds them first
in a checkout; `cargo xtask dist` puts them in `dist/nsh/runtime/gpui/` on each of the three OSes (`tooling.md`,
`embedding.md`). CI: `.github/workflows/gpui.yml` (Ubuntu 24.04, macOS 15 and Windows: clippy with `-D warnings` and
tests of the GPUI crates, the CLI and `nsh-dev`).

Features: `js` (default; JS apps; without it the crate only runs Rust apps and links no QuickJS) and `dev` (the dev
server client, log forwarding, the error overlay: `dev.rs`). The host turns on `dev`, the runner only `js`.

To look at a change on Linux without touching the desktop, run the host on a rootful Xwayland display of its own and
take screenshots with ImageMagick (GPUI draws at the display's scale):

```sh
Xwayland :170 -noreset -geometry 1400x1000 & sleep 1.5
nsh build examples/gallery -o build
DISPLAY=:170 WAYLAND_DISPLAY= target/debug/nsh-gpui-host --bundle build/bundle.nshc --size 700x500 &
sleep 5; import -window root -display :170 a.png
```

## Architecture

- **One element per surface.** A surface (`surface.rs`) is drawn by one GPUI element, `render::SurfaceElement`, which
  fills the space its parent gives it, reports that size to the engine, takes the pointer and keys over it
  (`events.rs`) and paints the whole tree in its `paint` (`render::draw_view`): shadows and background (`decor.rs`),
  the content clipped to the bounds (text, image, canvas), the children (clipped when the view clips), the hosted
  items (`view::Hosted`: text editors, headers, dialogs) and the scroll indicator, then the border. Each node is an
  `NshView` (`view.rs`), a retained view the core placed at its frame, as on Windows.
- **`DrawCx`** (`render.rs`) is the drawing state GPUI does not keep: GPUI paints primitives (quads, paths, glyphs,
  images) in window coordinates (logical pixels, which are nsh's points) with no world transform and no group
  opacity, so `DrawCx` keeps the current transform (`geom::Mat4`; translations and scales are exact, rotations and
  skews draw at their bounds, paths and glyph runs aside) and multiplies the opacity into every color
  (`DrawCx::color`), as GPUI's own element opacity does. Clips are rectangles in window coordinates (GPUI's content
  masks), intersected as they nest (`push_clip_rect` / `push_opacity` / `pop`); `paint(|window, app| …)` paints inside
  the current clip.
- **One hop per commit.** The engine runs on its own thread (`nsh_engine::Engine`); its sink queues each commit and
  sends one closure to GPUI's main thread (`rt::invoke`), which applies every queued batch in order (`mount.rs`).
  GPUI's executors are bound to the main thread, so `rt::init` spawns one foreground task per process that drains a
  channel of `Send` closures; worker threads (image decoders, network callbacks, the dev client) use it too and find
  their surface by key. `rt::after_ms` runs timers. Changes only mark a surface invalid: every main-thread entry point
  (a closure run there, a timer, an input event, a frame) ends with `rt::flush`, which notifies the GPUI views of the
  invalid surfaces. Views change outside GPUI's tracked state, so GPUI Fast's retained rendering needs that `notify`.
- **Text is shaped on the engine thread** (`text.rs`): `GpuiTextEngine` has a `gpui::WindowTextSystem` of its own
  over the app's `gpui::TextSystem` (thread-safe; the fonts are the app's, so the `FontId`s it hands out are the ones
  the main thread paints with). GPUI shapes one line of one font size at a time, so each style run of each paragraph
  is shaped on its own into one flat glyph list per paragraph, and nsh breaks the lines (after spaces, around CJK
  ideographs, after hyphens and slashes, inside a word too long for its line). The final layout is plain data (lines
  and positioned glyphs); the main thread paints the glyphs (`paint_glyph`, `paint_emoji`) and reads colors and
  decorations from the spans by UTF-16 offset, as the Linux backend does.
- **Frames**: native animations, JS frame requests, transitions, scroll glides and navigation transitions tick on the
  window's frames (`Surface::tick`, called while drawing; `window.request_animation_frame()` while something
  animates), and nothing is drawn otherwise.
- **Events** (`events.rs`): GPUI's mouse, scroll-wheel, pinch and key listeners last one frame, so they are installed
  in the element's paint: presses on the surface's hitbox, moves and releases while a press that began there lasts
  (GPUI captures nothing), wheel and touchpad scrolls the hitbox should handle, keys while the surface's focus handle
  is in the focus path. Positions become root coordinates and go to `input.rs` (hit testing, presses, the responder
  system, gestures) as on Windows. Keys become Windows virtual key codes (`keys.rs`), which the shared input,
  scrolling, keymap and editing code reads.
- **Embedding**: `AppView` (`app.rs`) is one surface as a GPUI view: `cx.new(|cx| AppView::new(options, window,
  cx))`, then `load(App::Js(bundle))` or `load(App::Rust(root))`; it renders a `SurfaceElement` and is `Focusable`.
  `open_app_window` opens one in a window of a running GPUI app; `run_app` runs the standalone app (one `AppView`
  filling a window, the process quits when the last window closes). `register_component` registers native
  components; `set_ui_sets` replaces the default `nsh/ui/mobile` set for apps with Rust components of their own. Several
  views are several engines; nothing is global but the `rt` queue, which is per process.

## How each area maps to GPUI

| Area | Module | GPUI |
|---|---|---|
| Text | `text.rs` | GPUI's text system on the engine thread, lines broken by nsh, glyphs painted on the main thread (above). TODO: selection |
| Decoration | `decor.rs` | Quads (`paint_quad`: background, rounded corners, borders), drop and inset shadows (`paint_drop_shadows`, `paint_inset_shadows`), linear gradients as a quad's `Background`, paths for shapes quads cannot draw. TODO |
| Images | `image.rs` | Loaded and decoded (the `image` crate) on worker threads into `RenderImage`s (straight BGRA, a frame per animation frame), cached per location, drawn with `paint_image` (its corner radii). GPUI has no color matrix, blur, mipmaps or sprite opacity: a picture keeps a few variants made on the workers (halved when drawn small, blurred, filtered, tinted, opacity in 16 steps) and draws the closest ready one meanwhile. Textures leave the atlases when their last owner drops them |
| Canvas | `canvas.rs` | TODO |
| Scroll views | `scroll.rs` | The Windows backend's, unchanged: offsets of their own, deceleration, snapping, paging, sticky cells, the keyboard, the indicator dragged as a scroll bar |
| Input | `input.rs`, `events.rs` | The Windows backend's hit testing, presses, responder and gestures, fed by GPUI's mouse, wheel and pinch events |
| Text inputs | `text_input.rs` | TODO (the shared editor model, `text_input/model.rs`; IME through GPUI's `EntityInputHandler`) |
| Navigation, modals | `navigation.rs` | TODO |
| Dialogs, toasts | `system.rs` | TODO |
| Video, web views, native components | `native.rs` | TODO (GPUI's native surface composition, `Hosted::placed`) |
| Accessibility | `access.rs` | TODO |
| Environment | `system.rs` | TODO (`window.appearance()`, `observe_window_appearance`, `scale_factor`) |
| Window chrome (`nsh/desktop`) | `chrome.rs` | TODO (`set_window_title`, `TitlebarOptions`, `start_window_move`, `cx.set_menus`) |
| Clipboard | `system.rs` | TODO (`cx.write_to_clipboard`, `cx.read_from_clipboard`) |
| Networking | `net.rs` | ureq, as on Windows and Linux |
| Dev | `dev.rs` | TODO (the dev server client, the status pill and error overlay) |

## Checking a change: `--script`

TODO: `nsh-gpui-host --script "<steps>"` (`script.rs`), ported from the Windows backend's (`windows.md`).

## Not implemented, or different

- **Transforms**: GPUI has no transform for quads and glyphs, so rotated and skewed views draw at their bounds (paths
  and glyph runs aside); perspective is not drawn.
- **Opacity** is multiplied into each color rather than drawn as a group: overlapping translucent children show
  through each other.
- **Clips** are rectangles (GPUI's content masks): rounded `overflow: hidden` clips are not rounded yet.
- **Image opacity** (a view's, a fade-in) steps in sixteenths: each step is a variant of the image's pixels.
- TODO: canvas, text selection, text inputs, navigation and modals, dialogs, video, web views, native
  components, accessibility, the environment, window chrome and menus, the clipboard, the dev client and `--script`
  are being ported (each area's section above says how).
- **Packaged apps** are laid out as the native backend's of the machine they are made on (a directory and `.tar.gz`
  on Linux, an ad hoc signed `.app` on macOS, a directory and `.zip` on Windows; `embedding.md`), with
  `nsh-gpui-app` as the executable.
