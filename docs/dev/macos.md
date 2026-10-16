# The macOS backend (AppKit)

`crates/nsh-platform-macos` renders nsh apps with AppKit views, Core Animation and CoreText. `crates/nsh-macos-host` is
its development host (`nsh run --target macos`), `crates/nsh-macos-app` the release runner of `nsh package --target
macos`. It follows the Linux backend's desktop design (`linux.md`: nsh picks views itself, scroll views keep offsets of
their own, nsh draws screens, headers and modals, sheets are centered cards, `desktop={{…}}` props apply) with the iOS
backend's Apple drawing (CoreText, Core Graphics, Core Animation layers, ImageIO, `NSURLSession`, AVFoundation, WebKit).
Mac Catalyst (`nsh-platform-ios` on `aarch64-apple-ios-macabi`) stays the default `nsh run` target on macOS: it is
UIKit, phone-style. In a checkout, `cargo run` runs the gallery here (`--target catalyst` for Catalyst). User-facing page: `website/docs/macos.md`.

## Building and checking

The crates need only Xcode's command line tools (the macOS SDK) and, for QuickJS, clang. Off macOS they are empty (like
the other platform crates off their targets). They are built for the machine itself, so the CLI finds them in
`target/<profile>` and builds them first in a checkout; `cargo xtask dist` puts them in `dist/nsh/runtime/macos/`.

```sh
cargo clippy -p nsh-platform-macos -p nsh-macos-host -p nsh-macos-app --all-targets --features nsh-platform-macos/dev
cargo test -p nsh-platform-macos --features dev   # needs no window server: tests do not start AppKit
cargo build -p nsh-macos-host                     # ./target/debug/nsh-macos-host
```

Features: `js` (default; JS apps; without it the crate only runs Rust apps and links no QuickJS) and `dev` (the dev
server client, log forwarding, the error overlay: `dev.rs`). The runtime needs macOS 14 (`NSView`'s display links).

## Architecture

- **One view per node.** Each node is an `NshView` (`view.rs`, a flipped, layer-backed `NSView` subclass holding the
  node's `ViewState`). The core lays everything out: a view sits at its frame (`place()`: the origin, a sticky shift,
  mirrored lists), and its transform, opacity and visibility are its layer's. AppKit resets a layer-backed view's
  anchor, position and transform when it moves the view, so `place()` sets them again every time, with the anchor at
  the top left (transforms are conjugated around the transform origin).
- **Children live in a container** when the view needs one: scroll views always (the container's bounds origin is
  the scroll offset, as `NSClipView` scrolls, so a scroll is one call), and views that clip their children and cast a
  shadow (clipping a layer clips its shadow). Hosted AppKit views (text fields, headers, videos, web views) are
  `ViewState::natives`, above the children, except a sheet's scrim (hosted by the root, below its card); hit testing
  follows the AppKit order of hosted views and children, so a click on the card reaches the card.
- **Web views and videos are views among the others**, so window composition needs no surfaces of its own (unlike
  Windows' child windows): a `WKWebView` is a layer-backed subview of its node's view, and the layer WebKit's process
  draws into is composited by Core Animation with the rest. What is above it in the view tree draws over it (later
  siblings, modals and sheets with their scrim, toasts; the dev status line, error overlay and FPS HUD are above the
  root), and its ancestors' clipping (corner radii, masks), opacity, transforms, hiding and scroll offsets apply to
  it. AppKit's input and in-process rendering do not follow the layers: the root's `hitTest:` decides with nsh's
  picking (Input, below), a page under a transformed ancestor is drawn transformed but gets mouse positions as if it
  were not, and `renderInContext:` leaves pages and videos out (leaving screens and modals take the live views
  along; `--script`'s `shot` shows them blank). `NSAlert` sheets are windows of their own, above everything.
- **Drawing**: Core Animation draws backgrounds, borders, corners, shadows, gradients and images (`decor.rs`,
  `visual.rs`, `image.rs`, the iOS backend's design). Text and canvas views draw in `drawRect:` (their layer's
  contents, below sublayers: they keep their background on the layer itself); other views use `updateLayer` and get no
  backing store. Implicit animations are off for every change nsh makes (`surface::quiet`).
- **Drawing views clip to their bounds** (`decor.rs`, as UIKit's `drawRect:` does), unless a shadow or outline is drawn
  outside them. Since macOS 14 views do not clip by default, and AppKit then backs what a `drawRect:` view could draw
  outside its bounds up to what its ancestors show, and draws it again whenever that changes
  (`NS_viewVisibleRectDidChange`): every text of a scroll view on every scroll step (~150 redraws per wheel event in
  the gallery's Lists demo). `NSView.clipsToBounds` is the layer's `masksToBounds`, so decoration owns it.
- **Out-of-sight canvases are not redrawn by animations** (`Mounter::draw_when_shown`): a canvas whose animated values
  change (on every frame while they run) is drawn again only if some of it can be seen (`visibleRect`), else when it
  shows (after scrolls, batches and frames). Lists keep rows out of sight; the Stocks demo's pulsing sparklines
  redrew all 22 rows at 120 Hz.
- **AppKit draws `drawRect:` content twice**: it records the drawing into a display list, then replays it into the
  layer's backing store (half-float on wide-color screens, on several threads). Drawing into a bitmap of our own in
  `updateLayer` instead was measured slower (one thread), and `contentsFormat` / `drawsAsynchronously` changed nothing:
  the lever is drawing less. Rendering canvases into bitmaps on a background queue was measured too (Stocks:
  main-thread frame 2.7 → 0.3 ms) and left out: the process' CPU went from ~53% to ~140% (Core Graphics on the CPU
  instead of AppKit's display-list replay), and the canvas showed a frame late.
- **One hop per commit.** The engine runs on its own thread; its sink queues each commit and wakes the main queue once
  (`DispatchQueue::main`), and the main thread applies every queued batch in order (`surface.rs`, `mount.rs`).
  Dispatched closures must be `Send`, so they find their surface by key in a thread-local map; no globals are shared
  between surfaces.
- **Text is typeset on the engine thread** by CoreText (`text.rs`, the iOS engine with `NSFont`); the main thread
  draws the finished `CTLine`s.
- **Frames**: native animations, JS frame requests, scroll glides, transitions and toasts run on the root view's display
  link (`NSView::displayLinkWithTarget_selector`) while something animates. Its `preferredFrameRateRange` asks for the
  screen's `maximumFramesPerSecond` (120 Hz on ProMotion; set again when the link resumes, as the window may have
  moved to another screen); measured, it delivered 120 Hz before that too (refresh gaps p50 8.33 ms). The link does
  not fire while the window is occluded or minimized or the screen is locked; while frames are wanted and it has not
  ticked for 100 ms a 60 Hz
  main-queue timer ticks instead, so animations still end (a transition detaches its views when it does).
- **Embedding**: `AppView` (`app.rs`) is one surface as a view; `run_app` opens one in a window with a standard menu
  (About, Hide, Quit; Edit's undo / redo / cut / copy / paste / select all, which fields answer; Minimize, Zoom, Enter
  Full Screen, Close), titled with `AppOptions::title`: `name_app` sets the main bundle's `CFBundleName` to it, as the
  menu bar shows that (the development host has no bundle; `nsh run` passes it the app's name with `--name`). The app
  ends when its window closes, a minimized one too (AppKit's `applicationShouldTerminateAfterLastWindowClosed:` skips
  windows that are not on screen, so `windowWillClose:` terminates). `set_ui_sets`
  replaces the default `nsh/ui/mobile` set for apps with Rust components of their own.

## How each area maps to AppKit

| Area | Module | AppKit |
|---|---|---|
| Text | `text.rs` | CoreText on the engine thread, `CTLine`s drawn in `drawRect:`; decorations CoreText lacks drawn by hand (iOS); `selectable` text's range selection, desktop style: drag, double-click word, triple-click paragraph, Shift extends, Cmd+A / Cmd+C (the root's `selectAll:` / `copy:`), Escape clears |
| Decoration | `decor.rs`, `visual.rs` | The view's layer for one radius (circular or continuous), shape layers for differing radii and borders whose sides differ or are dashed / dotted; box and inset shadows, gradients, outlines, filters (iOS's `visual.rs`); backdrop blur as an `NSVisualEffectView` subview, with the background moved to a fill layer above it; `apply` caches what it set, so per-frame calls are cheap |
| Images | `image.rs`, `blur.rs` | ImageIO decoding off the main thread, downsampled to the view; an LRU cache (128 MB); `asset:/`, http(s) (`NSURLSession`), `file:`, `data:`; tint as a colored layer masked by the image; animated GIF / APNG / WebP as a keyframe animation of `contents` |
| Canvas | `canvas.rs` | The recorded drawing replayed with Core Graphics (iOS); drawings that clear or composite go into a transparency layer, so they only touch the canvas's own pixels |
| Scroll views | `scroll.rs` | Offsets of their own, shown by the container's bounds origin; trackpad scrolls follow the system's phases and momentum (rubber banding while the fingers are down, a spring back after), notched wheels move 10 points per line and stay on their view for 200 ms; mouse drags scroll as on Catalyst; snapping and paging glide from where the fingers lift; sticky cells; the indicator and refresh spinner in an overlay view that lets the mouse through; the keyboard scrolls as on Linux; a field that gets the focus is scrolled into view |
| Input | `input.rs` | The root's `hitTest:` takes every mouse event inside it, and nsh picks the view itself (transforms inverted, `overflow`, `pointerEvents`, hit slop, hidden screens, views animating out); a hosted view gets the event from AppKit only where nsh's picking lands on it, even below `pointerEvents: none` overlays (`native_at`): AppKit's own picking would give a web view the clicks on what nsh draws over it (a menu outside its parent's frame, transformed views) and on web views that are hidden, animating out or under `pointerEvents: none`. Touches for the responder system, `panActivation` gates, presses, long presses, pressed and hovered states, cursors (`NSCursor`), native pans, trackpad pinch and rotation (Control + wheel pinches), the mouse's back button goes back |
| Text inputs | `text_input.rs` | `NSTextField` / `NSSecureTextField`, multiline `NSTextView` in an `NSScrollView`, hosted at the content box in a view that fills the input (`InputPad`: a click on the padding, or beside a field's line, goes to the editor at the nearest point), borderless (the view draws the decoration); edit counts drop stale JS values; offsets are `NSString`'s UTF-16; a click outside a field that edits leaves the focus in it (the root refuses first responder during a mouse press), `blur()` and Escape hand it over; return does what `submitBehavior` says (see "Keyboard") |
| Navigation | `navigation.rs` | Screens are child views, only the top one (and the one below during a transition) visible; headers are AppKit views (an `NSButton` back button with `chevron.left`, `NSTextField` titles, large titles, header items) of a fixed height reported to the core; a screen the app removes leaves as an image (`renderInContext`), since the core empties it in the next batches, and so does a modal; the rendering has no web page or video in it (other processes draw them), so their live views are lent to the image where they were, clipped to a rectangle (`native::lend`), and given back or let go when it is gone; Escape (unless a field or a web view has the focus: then it hands the keyboard to the root), Option+Left, Cmd+[ and the mouse's back button pop natively (a stack inside the newest modal first, else that modal is asked to close); a focused page sees these keys first, and they reach the root only when it does not use them (WebKit sends unused keys up the responder chain) |
| Modals | `navigation.rs` | Children of the root above the app's views, sliding up or fading in; page and form sheets are centered cards (720 / 540 points wide at most) over a scrim that dims the window, and clicking the scrim asks them to close |
| Transitions | `transition.rs` | `LayoutAnimation`, style transitions and keyframes on frame callbacks (Linux's) |
| Dialogs, toasts | `system.rs` | `NSAlert` as a sheet on the window. NSAlert puts two buttons in a row, its first rightmost, and stacks more, its first on top but the Return button moved above all: up to two alert buttons are added in reverse with the cancel button last (they read in order, cancel leftmost) and the last regular one is the default; more, and action sheets' options, are added in order with the cancel button last and no default (iOS's order). Cancel is Escape; destructive buttons have `hasDestructiveAction`. Toasts are a view the root hosts at its bottom |
| Video | `native.rs` | AVFoundation (loaded at run time, as on iOS): an `AVPlayerLayer` in a view that lets the mouse through, or AVKit's `AVPlayerView` with `controls` |
| Web views | `native.rs` | `WKWebView` (iOS's bridge: `ReactNativeWebView.postMessage`, injected JS, the `webview` capability's navigation policy, every prop, event and command iOS has); macOS's web view has no scroll view to turn off, so with `scrollEnabled={false}` the root's `hitTest:` keeps wheel and trackpad scrolls from it (`passes_wheel`) and they scroll the app's scroll views around it (the page's own keys and scripts still scroll it; swipes between pages, `allowsBackForwardNavigationGestures`, are scrolls too) |
| Native components | `native.rs` | `AppView::register_component(name, factory)`; a `NativeComponent`'s `NSView` fills the node's view; `ComponentContext::emit` / `set_size` from any thread |
| Accessibility | `access.rs` | `NSAccessibility` setters on the views: roles and subroles, labels, values, states, `testID` as the identifier; `accessible={true}` hides the children; live regions announced |
| Environment | `system.rs` | Color scheme: the app's appearance when it forces one, else the root's effective appearance (the root's appearance is set to the app's, so hosted controls follow); locale and RTL from `NSLocale`, reduce motion and VoiceOver from `NSWorkspace` (live), the backing scale factor, the OS version; `Platform.OS` `"macos"`, idiom `desktop`, font scale 1, safe area and keyboard height 0 |
| Clipboard | `system.rs` | `NSPasteboard` |
| Networking | `net.rs` | `NSURLSession` (the iOS backend's) |
| Dev | `dev.rs` | The dev server client on its own thread; a status line and the error overlay above the surface (opaque; code frames keep their lines and scroll sideways; a click that selects no text dismisses it, in an inactive window too); Cmd+R reloads the latest build |
| Performance monitor | `fps.rs`, `gpu.rs` | `--fps` / `AppOptions::fps` / `AppView::set_fps_monitor`: a panel above the surface that lets the mouse through, redrawn twice a second from the surface's `nsh_engine::perf::PerfMonitor` (see below); GPU time from the IO registry (`accumulatedGPUTime` of the process' accelerator clients) |

## Keyboard

A key goes to the window's first responder: a field's editor (the window's shared field editor, an `NSTextView`), a
multiline input's `NshTextView`, a hosted view, or the surface's root. Menus' key equivalents (`app.rs`: Cmd+Q / H /
Opt+Cmd+H, the Edit menu's Z / Shift+Z / X / C / V / A, Cmd+M, Ctrl+Cmd+F, Cmd+W) come first, matched by AppKit on
any layout; the dev host's Cmd+R is a local key monitor (`dev.rs`).

- **In a field** (`text_input.rs`), AppKit's key bindings edit: Option / Cmd + arrows, their Shift variants,
  Option+Delete, Cmd+Delete, Ctrl+A / E / K, undo and the Edit menu's commands (a secure field refuses Copy and
  Cut). Changes pass `Ctx::should_change` (one `onKeyPress` per insertion or deletion, `maxLength` clipping, line
  breaks of a single line become spaces: its cell is not in single-line mode, which would replace them in a second
  edit). The delegates' `doCommandBySelector:` (`Ctx::command`) takes return (`insertNewline:`, as
  `InputConfig::submit()` says; Shift+Return breaks the line of a multiline input that submits, and a return with
  marked text only commits the composition), Option+Return and Ctrl+Return (a `\n` in a multiline input, nothing in
  a single line; `insertLineBreak:` alone would insert U+2028), Option+Tab (a tab character in a multiline input
  only), Tab / Shift+Tab / Ctrl+Tab (`tab`: the next field on screen within the newest modal, skipping the clip view
  of a text view, which hands the focus back to it) and Escape (leaves the field). Keys a field uses never reach the
  root, so Option+Left moves the caret and Cmd+[ does nothing there.
- **At the root** (`view.rs` `keyDown:` → `input::key`): the selected text's Cmd+A / Cmd+C / Escape (`text.rs`;
  also the root's `selectAll:` / `copy:` for the menu), going back (`navigation::key_pressed`: Escape, Option+Left,
  Cmd+[), Tab / Shift+Tab into the fields, and scrolling the scroll view around the focused view or under the mouse:
  arrows by a line, Page Up / Down, Option+Up / Down and Space / Shift+Space by a page, Home / End and Cmd+Up / Down
  to the edges. The mouse's back button goes back from `input::event`.
- Shortcuts nsh matches itself (Cmd+A / C / [ / R) read `input::shortcut_key`: the character without modifiers, or on
  a layout that types none in ASCII the US layout's key, as menus do.

`onKeyPress` reports what React Native's does: the inserted text (`"Enter"` for a line break), `"Backspace"` for a
deletion (in an empty input too), and `"Escape"`; Tab and navigation keys are not reported.

## Measuring: the performance monitor

`nsh-macos-host --fps` (`nsh run --target macos --fps`) shows `fps.rs`'s HUD (rows: `website/docs/macos.md`). What
it reads is `nsh_engine::perf` (`docs/dev/performance.md`):

- The surface measures (`Surface::measure`, `drain`, `frame`) only while a monitor is installed; then the display
  link runs on every refresh, and each refresh closes a frame: the main-thread time since the previous one (batches
  with Core Animation's commit, which is where `drawRect:` content is drawn; ticks; wheel, drag and key handling).
  Refreshes in which nothing changed are not frames. The fallback timer closes frames at 60 Hz while the link is
  stopped (the screen locked).
- Batches carry when the sink got them (only while measuring), for `HOP`.
- The engine records its commits (`EngineControl::set_perf_monitor`).

For checking a change: run a release host with `--fps --trace` and `--script` (`wait`, `wheel`, `tap`); the host logs
the readings once a second (`nsh::perf fps …`, milliseconds as mean/p50/p95/max: `tick` the delivered refresh gaps,
`frame-gap` the gaps between shown frames, `main` the main-thread time per frame). The script's `wheel` sends an
event every 16 ms (a real trackpad on ProMotion: every 8 ms). For where the time goes, `sample
<pid> 5` the host (Instruments' Time Profiler: `xctrace record --template 'Time Profiler' --attach <pid>`).


## Checking a change: `--script`

`nsh-macos-host --script "<steps>"` (or `AppOptions::script`) runs steps once the window shows, separated by `;` or new
lines (`script.rs`, whose module doc lists them all): `wait <ms>`, `shot <file.png>`, `tap [mods] <x> <y> [count]`
(`tap shift 10 20`: a shift-click), `down` / `up` (a held press, for pressed states), `rtap`,
`drag <x0> <y0> <x1> <y1> [ms]`, `wheel <x> <y> <dx> <dy> [ms]` (a trackpad gesture), `notch` (a mouse wheel),
`hover`, `type <text>`, `mark <text>` (an input method's marked text), `key <name>` (with modifiers: `cmd+a`,
`shift+Tab`), `dark` / `light`, `resize <w> <h>`,
`quit`. Mouse events are posted to the application's queue, so they take the path real ones take (and become
`NSApp.currentEvent`).

```sh
nsh build examples/gallery -o /tmp/gallery
./target/debug/nsh-macos-host --bundle /tmp/gallery/bundle.nshc --size 1200x800 \
  --script "wait 1000; shot /tmp/a.png; tap 83 350; wait 500; shot /tmp/b.png; quit"
```

`shot` renders the window's content in process (`cacheDisplayInRect:`), so it needs no Screen Recording permission and
works while the screen is locked. That rendering is not Core Animation's own: it ignores `maskedCorners`, cuts masks at
their layer's bounds, ignores `shadowPath` and Core Image filters, flattens 3D transforms (perspective, `rotateX`,
`rotateY`), and does not show backdrop blur, sheets, `AVPlayerLayer` or `WKWebView` content. `decor.rs` avoids the first three (shape layers for differing corners, masked holders for box
shadows), so screenshots match the window closely, except that spread rings of box shadows are missing from them.

## Not implemented, or different

- **Zooming scroll views**: `zoomTo` and the zoom props are ignored (as on Linux).
- **`mixBlendMode`**, `brightness()` above 1. **`backdropFilter: blur()`** is an `NSVisualEffectView` (within-window
  blending, the popover material) below the view's background: AppKit picks the blur radius and adds the material's
  light tint; in-process screenshots cannot show it.
- **Inputs**: content types / autofill, keyboard types, autocapitalization, letter spacing; input accessory views are
  never shown (there is no software keyboard); `keyboardShouldPersistTaps` is ignored.
- **Accessibility**: `accessibilityActions`.
- **No desktop equivalent**: the status bar, haptics; the safe area and keyboard height are 0.
- **Packaged apps** are signed ad hoc; to distribute one, sign it with a Developer ID and notarize it.
