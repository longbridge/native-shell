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
- **Drawing**: Core Animation draws backgrounds, borders, corners, shadows, gradients and images (`decor.rs`,
  `visual.rs`, `image.rs`, the iOS backend's design). Text and canvas views draw into a **content layer** of their own
  (`NshView::redraw`: a plain `CALayer` sublayer whose delegate draws in `drawLayer:inContext:`, above the decoration
  behind the view and below its children); no view uses `drawRect:`, every view uses `updateLayer` and gets no backing
  store. Implicit animations are off for every change nsh makes (`surface::quiet`; the content layer's delegate
  answers no action).
- **Why not `drawRect:`**: scrolling moves a container's bounds origin, and `setBoundsOrigin:` walks the subtree
  (`NSViewHierarchyInvalidateVisibleRect`) and marks every `drawRect:` view's layer for display
  (`NS_viewVisibleRectDidChange`), clipped or not: every text of a scroll view went through AppKit's display pass on
  every frame of a scroll (~165 `viewWillDraw` and backing-store checks per frame in the gallery's Lists demo, ~70% of
  the frame's main-thread time). Plain sublayers are not visible-rect managed. A content layer is drawn again when
  its content changes and when its size does (`needsDisplayOnBoundsChange`), not when the view moves (transitions
  redrew every moving text on every frame before). Canvases draw asynchronously (`drawsAsynchronously`: Core
  Animation replays the drawing on its threads; Stocks idle, main-thread time per frame ~3.3 → ~1.8 ms, process CPU
  ~60% → ~30%); text draws at once, which measured faster than asynchronously.
- **No masks on what scrolls, where avoidable** (`visual.rs`): the window server pays for a mask on every frame its
  layer moves. Outer box shadows under an opaque background (a fill layer, or an opaque gradient, both above them)
  are plain `shadowPath` casters, masked only under translucent ones; inset shadows are clipped by their holder's
  rounded bounds (`masksToBounds`), masked only for differing corners. Measured by screen capture, a scroll of the
  gallery's Lists (a shadow and an inset highlight on every card) was composited at ~55 of 120 Hz with the masks and
  ~107 without, although the app committed every frame in time (Instruments' Animation Hitches: every frame one
  refresh late).
- **Out-of-sight content is not drawn** (`Mounter::draw_when_shown`): text and canvases whose content changes (a
  canvas's animated values on every frame while they run) are drawn only if some of the view can be seen
  (`visibleRect`), else when it shows (after scrolls, batches and frames; a view out of the tree waits until it is
  inserted and shows). Lists keep rows out of sight, hidden screens stay mounted, and a content layer, unlike AppKit's
  lazy `drawRect:`, would otherwise draw them all at once (the Stocks demo's pulsing sparklines redrew all 22 rows at
  120 Hz).
- **Trackpad scrolls are shown in step with the display** (`scroll.rs`, `Resampled`): gesture and momentum events come
  at about the display's rate (120 Hz) but not in step with it, so applied as they come some frames moved twice and
  some not at all (up to a quarter of a fling's frames, depending on how the events fell; the frame rate still read
  120). Events record where they put the offset at their timestamp; each display frame shows that path one event
  interval (+2 ms) back, interpolated. It adds ~10 ms of latency, and the main thread does the move once per frame
  instead of once per event (each event committed a Core Animation transaction of its own). Mouse wheels (no phases)
  still move at once. Anything else that moves the view (a drag, an animated scroll, a spring back) first shows where
  the events got to (`settle`).
- **One hop per commit.** The engine runs on its own thread; its sink queues each commit and wakes the main queue once
  (`DispatchQueue::main`), and the main thread applies every queued batch in order (`surface.rs`, `mount.rs`).
  Dispatched closures must be `Send`, so they find their surface by key in a thread-local map; no globals are shared
  between surfaces.
- **Resizes are shown at once** (`Surface::resized`, the one place the main thread waits for the engine): when the
  window resizes the root, the main thread waits for the layout at the new size (the engine reports it after its
  commit: `MountSink::resized`), at most 16 ms, and applies it before AppKit draws the window. AppKit's animated
  resizes (zoom, the green button, a double click on the title bar) run the run loop in a private mode
  (`_NSMoveTimerRunLoopMode`) where the main queue does not drain: the window grew around the old content for the
  whole animation (~360 ms, 41 of 41 captured frames), its new area white. Now it is laid out on every step (the wait
  is ~0.1–1.5 ms of engine time; mounting what comes into view is the rest); a live resize takes the same path. A capture still shows a few frames one step behind (2–12 of ~60 per zoom and unzoom), as a plain AppKit window
  does (13): the window server shows the window's new frame before the commit that fills it. An engine that misses
  the deadline (JS busy) is not waited for again until it reports a resize done, so a busy app resizes as before.
  Where the app has not drawn yet, the surface's background shows: the app's top view's opaque background when it
  fills the root (`system.rs`), else white / neutral-950 by the scheme.
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
| Input | `input.rs` | The root's `hitTest:` takes every mouse event over an app view, and nsh picks the view itself (transforms inverted, `overflow`, `pointerEvents`, hit slop, hidden screens, views animating out); a hosted view under the pointer gets the event from AppKit, even below `pointerEvents: none` overlays (`native_at`). Touches for the responder system, `panActivation` gates, presses, long presses, pressed and hovered states, cursors (`NSCursor`), native pans, trackpad pinch and rotation (Control + wheel pinches), the mouse's back button goes back |
| Text inputs | `text_input.rs` | `NSTextField` / `NSSecureTextField`, multiline `NSTextView` in an `NSScrollView`, hosted at the content box in a view that fills the input (`InputPad`: a click on the padding, or beside a field's line, goes to the editor at the nearest point), borderless (the view draws the decoration); edit counts drop stale JS values; offsets are `NSString`'s UTF-16; a click outside a field that edits leaves the focus in it (the root refuses first responder during a mouse press), `blur()` and Escape hand it over; return does what `submitBehavior` says (see "Keyboard") |
| Navigation | `navigation.rs` | Screens are child views, only the top one (and the one below during a transition) visible; headers are AppKit views (an `NSButton` back button with `chevron.left`, `NSTextField` titles, large titles, header items) of a fixed height reported to the core; a screen the app removes leaves as an image (`renderInContext`), since the core empties it in the next batches; Escape (unless a field has the focus), Option+Left, Cmd+[ and the mouse's back button pop natively (a stack inside the newest modal first, else that modal is asked to close) |
| Modals | `navigation.rs` | Children of the root above the app's views, sliding up or fading in; page and form sheets are centered cards (720 / 540 points wide at most) over a scrim that dims the window, and clicking the scrim asks them to close |
| Transitions | `transition.rs` | `LayoutAnimation`, style transitions and keyframes on frame callbacks (Linux's) |
| Dialogs, toasts | `system.rs` | `NSAlert` as a sheet on the window. NSAlert puts two buttons in a row, its first rightmost, and stacks more, its first on top but the Return button moved above all: up to two alert buttons are added in reverse with the cancel button last (they read in order, cancel leftmost) and the last regular one is the default; more, and action sheets' options, are added in order with the cancel button last and no default (iOS's order). Cancel is Escape; destructive buttons have `hasDestructiveAction`. Toasts are a view the root hosts at its bottom |
| Video | `native.rs` | AVFoundation (loaded at run time, as on iOS): an `AVPlayerLayer` in a view that lets the mouse through, or AVKit's `AVPlayerView` with `controls` |
| Web views | `native.rs` | `WKWebView` (iOS's bridge: `ReactNativeWebView.postMessage`, injected JS, the `webview` capability's navigation policy); `scrollEnabled` is ignored (macOS's web view has no scroll view) |
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

For checking a change: run a release host with `--fps --trace` and `--script` (`wait`, `wheel`, `fling`, `tap`); the
host logs the readings once a second (`nsh::perf fps …`, milliseconds as mean/p50/p95/max: `tick` the delivered
refresh gaps, `frame-gap` the gaps between shown frames, `main` the main-thread time per frame). The script's `wheel`
sends an event every 16 ms; `fling` sends them as a ProMotion trackpad does, every 8.33 ms from a timer thread with
the system's momentum after, stamped when they happen. A frame rate of 120 does not mean a smooth scroll: check how
far each frame moves (fling a few times with starting times a millisecond apart, as event and display phases differ
from gesture to gesture). For where the time goes, `sample <pid> 5` the host (Instruments' Time Profiler: `xctrace
record --template 'Time Profiler' --attach <pid>`).


## Checking a change: `--script`

`nsh-macos-host --script "<steps>"` (or `AppOptions::script`) runs steps once the window shows, separated by `;` or new
lines (`script.rs`, whose module doc lists them all): `wait <ms>`, `shot <file.png>`, `tap [mods] <x> <y> [count]`
(`tap shift 10 20`: a shift-click), `down` / `up` (a held press, for pressed states), `rtap`,
`drag <x0> <y0> <x1> <y1> [ms]`, `wheel <x> <y> <dx> <dy> [ms]` (a trackpad gesture), `fling <x> <y> <dx> <dy> [ms]`
(a 120 Hz trackpad flick with momentum), `notch` (a mouse wheel), `hover`, `type <text>`, `mark <text>` (an input
method's marked text), `key <name>` (with modifiers: `cmd+a`, `shift+Tab`), `dark` / `light`, `resize <w> <h>`,
`zoom` (AppKit's animated zoom), `fullscreen` (enter or leave), `quit`. Mouse events are posted to the application's
queue, so they take the path real ones take (and become `NSApp.currentEvent`).

```sh
nsh build examples/gallery -o /tmp/gallery
./target/debug/nsh-macos-host --bundle /tmp/gallery/bundle.nshc --size 1200x800 \
  --script "wait 1000; shot /tmp/a.png; tap 83 350; wait 500; shot /tmp/b.png; quit"
```

`shot` renders the window's content in process (`cacheDisplayInRect:`), so it needs no Screen Recording permission and
works while the screen is locked. That rendering is not Core Animation's own: it ignores `maskedCorners`, cuts masks at
their layer's bounds, ignores `shadowPath` and Core Image filters, flattens 3D transforms (perspective, `rotateX`,
`rotateY`), and does not show backdrop blur, sheets, `AVPlayerLayer` or `WKWebView` content. `decor.rs` avoids the first three (shape layers for differing corners, masked holders for box
shadows), so screenshots match the window closely, except that spread rings of box shadows are missing from them, and
so are outer box shadows under an opaque background (unmasked `shadowPath` casters, `visual.rs`).

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
