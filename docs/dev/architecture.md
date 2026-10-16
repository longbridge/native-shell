# Architecture

## Data flow and threads

```
JS (engine thread, QuickJS)  the app's components on the native reactive graph; compiled JSX calls the native renderer (nsh:rt)
nsh-js                        the renderer decodes each change from its JS values (tags, style objects, event names) -> Op, queued natively
nsh-core (engine thread)      apply(Op) maintains the shadow tree -> commit(): render components, rebuild text, Taffy layout, diff frames -> MountBatch
Platform (main thread)       one main-queue hop per commit, applying MountItems to native views in order
Input                        native gestures -> EngineControl::dispatch_event -> engine thread -> JS handler
```

- The platform main thread never waits for JS.
- **Reactivity is native**: signals, memos, effects, owners and context are a graph in Rust (`crates/nsh-js/src/reactive.rs`,
  scheduled by Solid 1.9's algorithm) whose nodes are QuickJS objects of native classes, so the QuickJS GC sees the
  cycles through them. Apps reach it through `nsh` (`signal`, `computed`, `effect`, ...).
- **The runtime is native** (`native-pipeline.md`): a bundle is one ES module of the app's code importing nsh's native
  modules (`nsh`, `nsh:rt`, `nsh/*`, `js-modules.md`). The renderer pushes ops straight into the host's queue; one JS
  turn's ops become one `Core::commit`. Events reach handlers held on the nodes, list cells are bound natively, and
  animations settle natively.
- **Text is laid out once**: `TextEngine` (defined in core) runs `prepare` on the engine thread (Apple: CTFramesetter) → `measure` at various widths →
  `layout` at the final size after layout (Apple: CTFrame). `MountItem::Text` hands the immutable result to the main thread, which only draws it
  (`CTFrameDraw` in `TextView`'s `drawRect`); the main thread never lays out text. Android's counterpart is `StaticLayout` (like RN's PreparedLayout),
  Linux's Pango (glyph runs as plain data, drawn as `gsk::TextNode`s with the main thread's fonts).
  Implementations must not touch main-thread-only UI objects.
- Measure cache: one content-addressed LRU per engine (`MeasureCache`, two generations of hash maps, ≤1024 entries), keyed by a hash of the
  layout-relevant content (`AttributedText::layout_hash`, colors excluded) + the width constraint; nodes also cache `Prepared` and the final layout.
  A color-only change does not trigger layout, only a new final layout.
- One `Surface` (Apple, Linux) / `NshView` (Android) is one engine (one QuickJS runtime), with no global singletons, so nsh can be embedded in
  existing apps with several instances. `AppView` (Apple, Linux: a GTK widget) wraps a surface for embedding and owns its dev session; a standalone app is
  just "one AppView filling the window".
- **Android**: the engine thread typesets with `StaticLayout` through JNI; each commit is one `byte[]` plus an object
  table handed to `Mounter.java` on the UI thread, and animation items stay in Rust (the flow, the units and the JNI
  rules: `crates/nsh-platform-android/src/encode.rs`).
- **Linux** (GTK 4, `linux.md`): one `NshView` widget per node, placed at the core's frames in `size_allocate` and
  drawn with GSK nodes in `snapshot`; each commit wakes the GTK main loop once (`MainContext::invoke`); native
  animations run on the frame clock.
- **Web APIs** (TS apps only; Rust code uses crates of its choice): `fetch` with `Headers` / `Request` / `Response`, the
  Streams API (readable streams with byte streams and BYOB readers, writable and transform streams, `pipeTo` /
  `pipeThrough`, `TextEncoderStream` / `TextDecoderStream`), `Blob` / `File` / `FormData`, `URL` / `URLSearchParams`,
  `TextEncoder` / `TextDecoder`, `Event` / `CustomEvent` / `EventTarget`, `AbortController` / `AbortSignal`: native
  classes in `crates/nsh-js/src/web/` (module docs there), made on first use of any of those globals (zero startup cost
  otherwise), as in browsers so libraries such as axios work (`fetch.rs` lists where they differ). `Intl`
  (`NumberFormat`, `DateTimeFormat`) and the `toLocaleString`s of numbers, BigInts and dates, which QuickJS lacks, are
  there too (`web/intl/`): formatted in Rust from hand-written tables of the common languages rather than CLDR (a few
  kilobytes instead of ICU4X's megabytes; trade-offs and limits in its module doc), in `Environment::locale` by
  default (the platform's region format: `NSLocale.currentLocale`, `Locale.getDefault()`).
- **Networking**: requests go to the platform stack through `nsh_engine::Network`
  (NSURLSession, HttpURLConnection; ureq on Linux and for `nsh trace`), streaming both ways: the stack reports the response head,
  then body chunks as they arrive (`FetchEvent`), and JS can pause reading (back pressure); a streamed request body is
  sent in chunks the stack asks for one at a time (`FetchEvent::BodyWanted`, chunked transfer encoding).

## Crates

Dependency direction: `nsh-core` ← `nsh-js` ← `nsh-engine` (also ← `nsh-package`) ← `nsh-platform-*` ← `nsh-ios-host` / `nsh-ios-app` / `nsh-linux-host` / `nsh-linux-app`; `nsh-cli` depends on `nsh-bundler`, `nsh-js` and `nsh-engine`; `nsh-ui-mobile` (the mobile components, `components.md`) depends on `nsh-core` only. `nsh-core` must not depend on any platform or JS crate.

The CoreText, StaticLayout and Pango text engines are documented in `crates/nsh-platform-ios/src/text.rs`,
`crates/nsh-platform-android/src/text.rs` and `crates/nsh-platform-linux/src/text.rs`, the measure cache in `crates/nsh-core/src/text.rs` (`MeasureCache`).
