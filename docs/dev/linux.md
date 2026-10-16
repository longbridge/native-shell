# The Linux backend (GTK 4)

`crates/nsh-platform-linux` renders nsh apps with GTK 4 widgets and lays text out with Pango. `crates/nsh-linux-host`
is its development host (`nsh run --target linux`, the default on Linux), `crates/nsh-linux-app` the release runner of
`nsh package --target linux`. The GPUI desktop backend is a separate effort; this one follows the mobile backends'
design (one native view per node, the core lays everything out). User-facing page: `website/docs/linux.md`.

## Building and checking

The crates need the GTK 4 (≥ 4.14; Pango from 1.50 on, the app's fonts going through Fontconfig) development
packages (`libgtk-4-dev` on Debian / Ubuntu, `gtk4` on Arch, `gtk4-devel` on Fedora) and, for QuickJS, clang. Off Linux
the crates are empty (like the other platform crates off their targets), so check them on Linux:

```sh
cargo clippy -p nsh-platform-linux -p nsh-linux-host -p nsh-linux-app --all-targets
cargo test -p nsh-platform-linux          # needs no display: tests do not start GTK
cargo build -p nsh-linux-host             # ./target/debug/nsh-linux-host
```

The Linux binaries are built for the machine itself, so the CLI finds them in `target/<profile>` (not
`target/<triple>/<profile>`) and builds them first in a checkout; `cargo xtask dist` on Linux puts them in
`dist/nsh/runtime/linux/` (`docs/dev/tooling.md`, `embedding.md`). CI: `.github/workflows/linux.yml` (Ubuntu 24.04:
clippy and tests of the Linux crates, the CLI, `nsh-dev` and xtask).

Features: `js` (default; JS apps; without it the crate only runs Rust apps and links no QuickJS) and `dev` (the dev
server client, log forwarding, the error overlay: `dev.rs`).

## Architecture

- **One widget per node.** Each node is an `NshView` (`view.rs`, a `gtk::Widget` subclass). The core lays everything
  out, so a view never measures itself: its parent places it in `size_allocate` at the core's frame (origin, scroll
  offset, sticky shift and transform in one `gsk::Transform`, the offset at whole points as GTK places its own widgets);
  frames are kept as `f32` points and the integer
  allocation only bounds picking. `snapshot` draws, in order: shadows and background (`decor.rs`), the content (text,
  image, canvas), the children (clipped to the rounded bounds with `overflow: hidden`), the border, then hosted GTK
  widgets (`ViewState::natives`: text editors, headers, videos).
- **One hop per commit.** The engine runs on its own thread (`nsh_engine::Engine`); its sink queues each commit and wakes
  the main loop once (`MainContext::invoke`), and the main thread applies every queued batch in order (`surface.rs`,
  `mount.rs`). `invoke` closures must be `Send`, so they find their surface by key in a thread-local map; no globals
  are shared between surfaces.
- **Text is typeset on the engine thread** with a Pango font map of the engine's own (Pango objects are not
  thread-safe, and GTK's font map belongs to the main thread). The final layout is plain data, per glyph run the font's
  description, glyph ids and positions; the main thread loads the same fonts by description from GTK's font map and
  draws `gsk::TextNode`s where the engine put them. Hinted metrics and rounded positions are off on both sides, so
  what was measured is what is drawn (`text.rs`).
- **Animations run on the frame clock**: native animation values (`anim.rs`, the Android animator's design), JS frame
  requests (the root's tick callback, `surface.rs`), `LayoutAnimation`, style transitions and keyframes
  (`transition.rs`), scroll glides and navigation transitions; tick callbacks exist only while something animates.
- **Embedding**: `AppView` (`app.rs`) is one surface as a widget; `run_app` opens one in a window. `set_ui_sets`
  replaces the default `nsh/ui/mobile` set for apps with Rust components of their own.

## How each area maps to GTK

| Area | Module | GTK |
|---|---|---|
| Text | `text.rs` | Pango layouts on the engine thread, `gsk::TextNode` glyph runs; colors and decorations read per cluster from the spans (a layout is reused for texts that only differ in them); cap / x height from the font's OS/2 table (HarfBuzz); `selectable` text's range selection, desktop style (below) |
| Decoration | `decor.rs` | GSK nodes: shadows (React Native's take the box's shape with a fill, the content's without, as iOS), colors, gradients, inset shadows, borders (over the children, as UIKit), outlines, filters |
| Images | `image.rs` | Loaded and decoded on worker threads (GDK's PNG / JPEG / TIFF loaders, the `image` crate for GIF and WebP), textures in an LRU cache per resolved location; blur and tint at draw time |
| Canvas | `canvas.rs` | The recorded drawing replayed with Cairo into a recording surface (`Snapshot::append_cairo`), sharp at any scale; shadows blurred into a mask; conic gradients as mesh patterns |
| Scroll views | `scroll.rs` | Offsets of their own (`ScrollState`), reported as UIKit's `contentOffset`; wheel / touchpad / drag scrolling with UIKit-like deceleration, snapping, paging, sticky cells, rubber banding for touch drags; the keyboard (arrows by 40 pt, Page Up / Page Down and Space / Shift+Space by a page, Home / End) scrolls the view around the focused widget or under the pointer while no field has focus; a widget that gets the focus (Tab, a click, `focus()`) is scrolled into view, animated |
| Input | `input.rs` | Its own hit testing over the views (GTK's `pick` ignores `overflow: hidden`, `hidden` and `box-only`; hosted widgets are picked inside with GTK's; outside a view's bounds only in its `Overflow` area); touches for the responder system, with `panActivation` gates (the scroll views around wait for the pan to fail; `TouchEvent::activated`); native presses, long presses, hover and pressed states; pan / pinch / rotate native values (Ctrl + wheel pinches); wheel, touchpad, mouse drag and touch scrolling routed to scroll views: wheels go to views along their axis, and a scroll stays on its view until it ends (touchpad gesture; wheel notches within 200 ms) |
| Text inputs | `text_input.rs` | `gtk::Text` / `gtk::TextView` in a `gtk::ScrolledWindow`, hosted at the content box; the view draws the decoration, a per-input style provider clears GTK's (an animated `color` goes to a second one-rule provider, so frames do not reload the sheet); edit counts drop stale JS values; offsets converted between UTF-16 and characters |
| Navigation | `navigation.rs` | Screens are child views, only the top one (and the one below during a transition) child-visible; headers are plain GTK widgets, of a fixed height reported to the core (`set_screen_header`); departing screens leave as a `gtk::WidgetPaintable` image; Escape (unless a text field has the focus: Escape leaves the field), Alt+Left, the Back key and the mouse's back button pop natively (a stack inside the newest modal first, else that modal is asked to close) |
| Modals | `navigation.rs` | Children of the root above the app's views, sliding up or fading in; page and form sheets are centered cards (720 / 540 points wide at most) over a scrim that dims the window, and clicking the scrim asks them to close |
| Dialogs, toasts | `system.rs` | GTK widgets the root hosts over the app's views (not separate windows: they stay with an embedded surface and never block the main loop) |
| Video | `native.rs` | `gtk::MediaFile` (GStreamer) in a `gtk::Picture`, or `gtk::Video` with `controls` |
| Native components | `native.rs` | `AppView::register_component(name, factory)`; a `NativeComponent`'s widget fills the node's view; `ComponentContext::emit` / `set_size` from any thread |
| Accessibility | `access.rs` | `GtkAccessible` (AT-SPI): roles from `accessibilityRole` / text / press handlers / labelled images, labels, descriptions, values, states; `testID` as the widget name; live regions emulated with `gtk_accessible_announce` |
| Environment | `system.rs` | Color scheme: the app's appearance (`nsh.json` `"appearance"`, `setColorScheme`) when it forces one, else the XDG settings portal's (`org.freedesktop.appearance color-scheme`, live); hosted GTK widgets, dialogs and the root's background follow it, and a packaged app's window opens in it (`nsh/app.json`). The locale (`Intl`) from `LC_ALL` / `LC_TIME` / `LC_NUMERIC` / `LC_MESSAGES` / `LANG`, else GLib's language names. Font scale from `gtk-xft-dpi`, reduce motion from `gtk-enable-animations`, the surface's (fractional) scale, GTK's text direction, AT-SPI's `ScreenReaderEnabled`; `Platform.OS` `"linux"`, idiom `desktop`, OS version the kernel's major.minor; safe area and keyboard height 0 |
| Clipboard | `system.rs` | The display's `GdkClipboard`; reads are asynchronous (another app's text), `hasString` only looks at the formats on offer |
| Networking | `net.rs` | ureq (rustls, webpki roots), one thread per request, streaming both ways; only connecting and the response head have deadlines |
| Dev | `dev.rs` | The dev server client on its own thread; status line and error overlay above the surface |

## Checking a change: `--script`

`nsh-linux-host --script "<steps>"` (or `AppOptions::script`) runs steps once the window shows, separated by `;` or new
lines (`script.rs`), so a change can be checked without touching the screen:

| Step | Does |
|---|---|
| `wait <ms>` | Waits. |
| `shot <file.png>` | Renders the window's content to a PNG, offscreen at scale 1, whatever covers the window. |
| `tap <x> <y>` | Presses and releases at a point (points, root coordinates). |
| `drag <x0> <y0> <x1> <y1> [ms]` | Presses, moves in steps over `ms` (default 300), releases. |
| `wheel <x> <y> <dx> <dy>` | A wheel / touchpad scroll. |
| `hover <x> <y>` | Moves the pointer. |
| `type <text>` | Types into the focused field. |
| `key <name>` | Presses a key (GDK names: `Return`, `BackSpace`, ...). |
| `dark` / `light` | Switches the system's appearance (an app that forces one keeps it). |
| `resize <w> <h>` | Resizes the window. |
| `quit` | Closes the window. |

```sh
nsh dev examples/gallery &       # or: nsh build examples/gallery and --bundle <path>/bundle.qbc
./target/debug/nsh-linux-host --size 390x844 --script "wait 800; shot a.png; tap 100 200; wait 300; shot b.png; quit"
```

Without `--bundle` the host connects to the dev server on this machine (`--dev-server ws://…/ws` names another);
`--trace` logs per-commit timings. GTK Inspector (`GTK_DEBUG=interactive`) shows the widget tree, with each view's
`testID` as its name.

## Not implemented, or different

- **Web views** need WebKitGTK (`webkit6`, WebKitGTK 6.0 for GTK 4), which many machines lack and nsh does not depend
  on: a `WebView` shows a neutral placeholder and ignores its commands.
- **Video**: GTK streams have no playback rate, so `rate` is ignored (warned once); a looping video never ends, so
  `onEnd` only comes for videos that do not loop. Which formats play depends on the installed GStreamer plugins.
- **Images**: animated GIF / WebP show their first frame.
- **Zooming scroll views**: `zoomTo` and the zoom props are ignored.
- **`mixBlendMode` and `backdropFilter`** are not drawn: they need what lies behind the view, which a widget's snapshot
  does not have (the background color still draws). `borderCurve: "continuous"` draws circular corners.
- **Input accessory views** belong over a software keyboard: never shown.
- **`keyboardShouldPersistTaps`** is ignored: it decides what a tap does while the software keyboard is up, and there is
  none (the keyboard height is 0). iOS ignores it with a hardware keyboard too (Mac Catalyst always): taps outside the
  focused field act at once and leave the focus where it is (a click on another field moves it).
- **Accessibility**: `accessible={true}` grouping (children stay separate elements), `accessibilityActions` (GTK has no
  custom actions) and real live regions (emulated by announcements).
- **No desktop equivalent**: the status bar; `AccessibilityFocus` moves keyboard focus when the view can take it.
- **Packaged apps** are a directory and a `.tar.gz` (`embedding.md`); GTK 4 and Pango must be installed where they run.
