# Verified constraints and pitfalls

Things that were found the hard way. Read the section of the area you change before changing it.

## QuickJS and the runtime

- **QuickJS microtasks are run by the host.** Call `Host::run_jobs()` after every call from Rust into JS. `setTimeout` / `console` are our own;
  `queueMicrotask`, `WeakRef` and `FinalizationRegistry` are built into QuickJS-NG.
- **The JS object graph must stay reachable from a GC root.** The renderer holds the root node, which holds its children's
  objects and each node its handlers (`dom.rs`, reported by the classes' GC marks). Without that the whole rendered app (nodes,
  handlers, signals) is an isolated cycle that idle GC collects, and taps stop working. `nsh trace` taps after 1.2s of idling by
  default; `crates/nsh-engine/tests/engine/counter.rs` is the regression test.
- **Nodes are released by GC, not by Remove.** A memo may keep removed nodes and insert them again, so `Remove` only detaches;
  `Release` (queued by the node's finalizer) frees the node and its native view.
- **`nsh_js::BUNDLE_NAME` (`bundle.js`)** is the module name of every bundle and is how stacks are matched for symbolication; `BYTECODE_VERSION`
  is tested against the linked QuickJS version: update both when upgrading rquickjs.
- **Encoded operations' codes** (`ANIMATION_*`, `CV_*`) come from `nsh_core::js` (`protocol.rs`, `canvas.rs`): never write a code's value
  by hand in `nsh-js`.
- **rquickjs converts `Option::None` to `undefined`, not `null`**: compare native results with `== null` in JS.
- **`on*` props are event handlers to the renderer** (held on the node, sent to the core as a listener flag): settings must not
  be named `on...` on host elements (`List` sends `onEndReachedThreshold` as `endReachedThreshold`).
- **There is no error boundary**: hot component proxies catch what a component throws, report it and render nothing until the
  fix (`nsh-js` `hot.rs`).
- **Hot proxies return accessors**, not nodes: runtime code that inspects what a component returned (`Stack` looks for a
  `<screen>`, `navigation.rs`) must accept an accessor.

## Core and layout

- **Text nodes outside `<Text>`** (e.g. a string child of a `<View>`) create no native view and take no part in layout. A `<Text>`
  nested in `<Text>` is a span without a native view; the whole block is merged into one `AttributedText`.
- **`TextSpan` offsets are UTF-16**, like NSAttributedString / Android Spannable.
- **Never re-insert a native child that is already in place.** `Insert` of a mounted view is a move: the platform takes it
  out of the window for a moment, which ends its touches and its focus (an Android input loses the keyboard). Child diffs
  (Rust components' `reconcile_children`) move only the children out of order, from the end, with `before` anchors.
- **Taffy 0.15 custom measure callbacks must delegate to `taffy::compute_leaf_layout`** and only compute the content size. Returning the
  content size directly skips resolving the leaf's own style (width / height / padding / min / max), so leaves such as
  `<Image style={{width: 48}}>` end up with a zero size.
- **List cells are independent Taffy roots** (not children of the list's Taffy node; their width follows the list's cross axis): scrolling only
  relays out dirty cells and only emits frames for relaid cell subtrees; when the main tree is clean, `compute_layout` and the frame walk of the
  main tree are skipped (Taffy runs a full rounding pass even over a clean root).
- **Taffy defaults are CSS semantics**; `Style::default()` switches to RN semantics: column, flexShrink 0, alignContent flex-start, position relative,
  and min sizes 0 (an unset or `auto` `minWidth` / `minHeight` is 0). Taffy's `auto` minimum is CSS's automatic minimum size (the
  min-content size); text has no cheap min-content and reports its natural width, so `flex: 1` text in a row could not shrink and
  overflowed instead of wrapping. Yoga has no automatic minimum.
- **`MountItem::Insert` positions by sibling (`before`), not by index**: platforms insert their own subviews into containers
  (`UIRefreshControl` at the bottom, scroll indicators), which would shift indices.
- **Platforms clamp scroll offsets to the content**: UIKit clamps `contentOffset` when `contentSize` shrinks, Android's
  ScrollView clamps `scrollBy` to the child's current size. The core grows the content before a `ScrollBy` and scrolls
  before shrinking it; Android sizes the content view at once in `setContentSize` (not at the next layout pass).
- **Scroll reports can predate the core's own scrolls**: each report carries how many `ScrollBy` / list `ScrollTo` the view
  has applied (`EngineControl::scroll(.., applied)`), and the core adds the ones still pending; otherwise a stale offset
  moves the window (and fires `onEndReached`) backwards.
- **Unbound list cells still show their last item**: the core moves them out of sight (never with a NaN frame: cells that
  were never placed are skipped; UIKit throws on NaN geometry).
- **Tailwind variant order decides between variants**, not their order in the string: `odd:bg-a last:bg-b` gives an odd last row
  `bg-a` (Tailwind sorts `odd` after `last`), exactly as in CSS.

## Bundler and toolchain

- **rquickjs must not enable the `full` feature.** It pulls in `dyn-load` (dlopen2), which does not compile for iOS / Catalyst. iOS targets
  have no prebuilt bindings and need the `bindgen` feature (and libclang).
- **Use `oxc_sourcemap` 9.x** (matching rolldown / oxc 0.153): source maps are composed with rolldown's types, so never mix in
  another version's.
- **Our plugin's own compile errors** are kept in `NshPlugin.error` and shown directly (clean code frame + relative path) instead of rolldown's
  wrapped report.
- **rolldown CommonJS chunks define reassigned exports as non-configurable getters**: a module run again fills a fresh
  `exports` object and the hot runtime forwards the importers' object to it.
- **`nsh run` installs the host by copying to a temporary file and renaming it**: overwriting in place an executable that another session is
  running makes the kernel SIGKILL the newly launched process for a code signature mismatch.
- **Development hosts never restore windows**: `nsh run` launches the Catalyst host with `-ApplePersistenceIgnoreState YES`;
  after a crash, macOS would otherwise show a modal "reopen windows?" alert over it.
- **IPAs are zipped with `ditto -c -k --norsrc --keepParent`**: `--sequesterRsrc` adds `__MACOSX/` resource forks.

## iOS, UIKit and Mac Catalyst

- **UIFont and CTFont are toll-free bridged**: the text engine gets system fonts with `UIFont::systemFontOfSize_weight` and uses them as
  CTFonts (verified by rendering). CoreText attributes must use CT keys (colors are CGColors), not UIKit's NSForegroundColor.
- **Catalyst's Mac idiom does not support `UIRefreshControl`** (attaching one throws `_UICatalystUnsupportedMacIdiomBehavior`): in the Mac
  idiom the pull is detected in the scroll delegate and shown with a spinner + contentInset; iOS uses the native control.
- **Mac idiom custom pull to refresh**: on release, the system's bounce already targets the old inset (back to 0), so changing only
  `contentInset` hides the spinner; once refreshing starts, the offset is animated to `-REFRESH_INSET` on the next main-queue turn. Synthetic
  tests need continuous scroll events with phases (began / changed / ended); plain wheel events never overscroll.
- **Set backgrounds on `layer.backgroundColor`, not `UIView.backgroundColor`**: views with `drawRect:` (`TextView`) paint the latter into their
  contents bitmap, where corners cannot round it.
- **Styles arrive before frames** (new views have a zero size): a shared radius goes straight to the layer (Core Animation clamps it itself);
  only paths, shadow paths, the content view and the image layer depend on the size, and they are rebuilt on `Frame` while `Decor::sized()`
  is true. Scaling radii by the CSS rule at a zero size once turned every rounded corner square.
- **Property changes on standalone `CALayer`s animate implicitly for 0.25s** (a UIView's own layer does not): wrap changes to shape / image
  layers in `CATransaction.setDisableActions(true)`.
- **Core Animation always draws `border` above sublayers**, while CSS borders are below children: non-clipping views use a stroke layer below
  the children; when clipping both look the same, so the layer's own border is used.
- **A UIScrollView's sublayers scroll with its content** (`bounds.origin` is the scroll offset), so scroll views cannot use shape layers / masks.
- **UIKit callbacks may fire synchronously while the Mounter applies a batch** (e.g. setting `contentOffset` triggers `scrollViewDidScroll`):
  when `EventTarget::with_mounter` cannot borrow, it defers to the next main-queue turn; never `borrow_mut` directly.
- **Views with a transform must not set `frame`**: always use `bounds` + `center`, keeping the bounds origin (a scroll view's bounds origin is
  its scroll offset).
- **Catalyst**: `UIDeviceFamily` must include 6 (Mac idiom), otherwise the UI is scaled to 77%. Under Catalyst the container extends below the
  title bar, so overlays are constrained to `safeAreaLayoutGuide`. UIDevice reports "iPad" under Catalyst. Use the UIScene lifecycle
  (`UIScreen.mainScreen` and `UIWindow initWithFrame` are deprecated). The window size is fixed to 390×844 through `sizeRestrictions`.
- Methods in `define_class!` that return `Retained<T>` need `#[unsafe(method_id(...))]`. Classes UIKit instantiates by name (AppDelegate,
  SceneDelegate) cannot have ivars; keep their state in statics or thread locals.
- **UIKit implements the `UITextInputTraits` setters of `UITextField` by message forwarding**: objc2's debug-build signature
  check cannot find them on the class and panics, so `input.rs` sends them with a raw `objc_msgSend`.
- **objc2 crates are used with `default-features = false`** and an explicit feature list in the root `Cargo.toml` (one feature per
  header): the defaults link CloudKit, CoreData, CoreImage, CoreLocation, Metal, ... into every app (each one costs dyld time at
  launch). When a method is missing after using a new API, enable the header feature named in its `#[cfg(feature = ...)]`; check
  `otool -L` on a release `nsh-ios-app` afterwards (only UIKit, Foundation, CoreFoundation, CoreGraphics, CoreText, QuartzCore, ImageIO).
- **`TextView` uses `UIViewContentMode::Redraw`**: list cells can get their text before their frame; without redrawing on
  resize the text stays drawn at zero size.
- **UINavigationController breaks transitions started during another one**: stack changes that arrive while
  `transitionCoordinator()` is set wait for `didShow` and are applied on the next main-queue turn (the coordinator outlives
  the callback); only screens the controller actually had count as popped by the user.
- **`UIAlertController` asserts when its presentation controller's delegate is set**; a popover action sheet (iPad, Mac) hides
  its cancel action and runs it on a tap outside, so sheets without one get a hidden cancel answering "dismissed".
- **Catalyst: a mouse drag does not scroll a UIScrollView** (Mac behavior); test scrolling with trackpad-style scroll wheel
  events (with phases), which also produce drag and momentum callbacks. Mouse drags do reach views as touches.
- **Catalyst: the Mac idiom moves a navigation bar's contents into the window's toolbar** (`UINavigationBar.behavioralStyle`
  resolves to `.mac`; macOS 26 does it even with the scene's `titlebar.toolbar` nil): the back button sits next to the traffic
  lights, the title is gone (or is the window's) and the header's colors do not apply. Stacks' bars (and the bar measuring
  the header height) set `preferredBehavioralStyle = .pad` (iOS 16+) to stay in the window, and standalone apps hide the
  window's title and toolbar (`UITitlebar`, which objc2 does not declare: `app.rs` sends its setters by name), so the title
  bar is a transparent strip over the in-window header, like iOS's status bar; apps embedding nsh on Catalyst should hide
  theirs the same way. The core's estimate of a header (the bar's height plus the safe area's top) can still miss what UIKit
  lays out, which showed the screen's root view in a band under the bar: screens report the top of their controller view's
  safe area (the bottom of whatever bar UIKit shows) as their header (`Core::set_screen_header`), and the stack's bar gets
  the screen's appearance as well as its navigation item.
- **Catalyst's Mac idiom presents page / form sheets as macOS sheets** at a default size wider than a 390pt window: they get
  the window's size as `preferredContentSize`. Standalone apps have a white root view (UIKit windows are black).
- **iOS clamps shared corner radii itself** (`rounded-full` is 9999): Core Animation does not reliably clamp a `cornerRadius`
  over half the short side, so `decor.rs` clamps it and clamps again when the size changes.

## Android

- **Android, JNI**: cache classes and method ids in `JNI_OnLoad`; wrap every call from the (permanently attached) engine thread in
  a local frame (`java::with_env`), or local references pile up.
- **Android, text**: `StaticLayout` ignores `maxLines` without an ellipsis (clip mode counts lines itself and the view clips);
  build layouts from `PrecomputedText` with the same break strategy / hyphenation / text direction as its params, or it
  measures again. A layout of precomputed text does not draw `UnderlineSpan` / `StrikethroughSpan` (even ones set after
  `create`): runs carry a `TextLayouts.Decoration` marker and `TextBox` draws the lines. `Layout.Alignment` is relative to each
  paragraph's direction: absolute left / right alignment sets an `AlignmentSpan` per paragraph from its resolved direction.
- **Android, JNI event maps intern their keys from a fixed list** (`Java_com_longbridge_nsh_Native_eventMap`): a new payload key must be
  added there or it is dropped silently.
- **Android, inputs**: `EditText` reconfiguration (input type, filters, single line) sets the same text again and fires
  `afterTextChanged`: only text that differs from the last known one counts as an edit, or JS values are rejected as stale.
  Soft keyboards edit through the input connection (`KeyConnection`: composing text, commits, deletes), hardware keys arrive
  as key events; both feed `onKeyPress`. A hardware Enter reports its down and its up — act on the down only and consume the up, or the up advances
  focus again in the next field; blur by focusing the (focusable, highlight-free) `NshView`, as `clearFocus()` hands focus to the
  first focusable view; set the typeface after the input type (password types reset it to monospace).
- **Android, keyboard avoidance**: the keyboard's height becomes bottom padding of the scroll around the focused input, and
  `ScrollBox.reveal` scrolls it into view; `requestRectangleOnScreen` cannot, as ScrollView ignores its padding when it decides
  whether a rectangle is visible. `NshView` is a scroll container (`setScrollContainer(true)`) so that windows without a soft
  input mode resize instead of panning: the system picks the mode at the first layout, before the app's content is mounted,
  and a mode set from code afterwards is overridden. Windows that ask for `adjustPan` are left to the system.
- **Android, drags that start on a child**: a pannable `Box` intercepts the move that exceeds the slop and starts its pan right
  there; that intercepted move never reaches its `onTouchEvent`, so starting the pan there is what makes a quick flick (few moves)
  pan and end at all.
- **Android, modal content placement**: `Box.place` lays a view out at its core frame directly, so the status bar offset of a
  non-translucent modal is a content view inside the dialog's container, not an offset applied to the root's layout; the dialog
  copies the activity's bar appearance (light / dark icons).
- **Android, nested scrolling**: scroll views enable nested scrolling, or an outer vertical ScrollView intercepts every vertical
  drag meant for an inner one.
- **Android, safe area**: `NshView` reports only the parts of the system bars / keyboard that overlap it (like UIKit's
  `safeAreaInsets`), so embedded views below a toolbar get no top inset. Overlays use the same overlap (`topInset()`).
- **Android, dev assets come over plain HTTP**, which apps block by default (API 28+): the dev host APK and `nsh-dev.aar` set
  `usesCleartextTraffic`; the release AAR does not.
- **Android, activities**: create the decor view (`getDecorView()`) before asking for the insets controller; apps draw edge to edge
  and get the system bars through `safeArea()`.
- **Android widgets**: `MediaPlayer.setPlaybackParams` starts playback (set the rate only while playing) and a paused player draws nothing until
  it seeks; `WebView` JavaScript bridge calls arrive on a binder thread (post to the UI thread).
- **Android, negative letter spacing**: Minikin adds half the spacing on both sides of each glyph and trims the halves at line edges,
  so a line is wider than its sum of advances; `TextLayouts.Prepared.desired` adds the largest negative spacing back, or text laid
  out at its natural width wraps (`tracking-tight`).

## Testing on simulators and emulators

- **iOS simulator: launch apps only after `simctl bootstatus -b`**; apps launched while the simulator boots are sent to the
  background when SpringBoard comes up.
- **Synthetic clicks on the Simulator / Catalyst windows go to whatever macOS window is on top at that point**: floating windows
  of other apps (e.g. a Slack huddle bar at window level 1001) silently swallow clicks over a band of the app. When a tap does
  nothing, list the windows at the point (`CGWindowListCopyWindowInfo`) before suspecting hit testing.
- **Not supported / not verifiable here**: `I18nManager.swapLeftAndRightInRTL(false)` (left / right always swap right to left);
  after switching direction at runtime UIKit keeps an existing back button's chevron; Android has no skew transforms (as in React Native); the iOS simulator cannot
  inject edge swipes (interactive back is untested there); Android's stylus handwriting mode (on by default on recent
  emulators) intercepts taps on inputs — turn it off with `adb shell settings put secure stylus_handwriting_enabled 0`.
