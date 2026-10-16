# Performance

Performance is the project's first priority; the rules are in `AGENTS.md`. Changes to hot paths come with before /
after `nsh trace` timings and item counts. Current baselines (Apple Silicon):

- counter, headless (debug): first frame ~0.5–0.65ms (58 ops / 49 mount items), one tap ~0.15ms (12 ops / 8 items).
- counter, Catalyst (release): first frame ~1.05–1.15ms (of which ~0.7ms is cold CoreText layout of 10 texts), one tap ~0.12ms.
- Hot reload: ~120ms from saving a file to the device running the new version (file event latency and ~60ms of development
  bundling included); applying a hot update takes 0.2–0.5ms on the engine thread.
- gallery Photos list (`examples/gallery/Bench.tsx`, debug, headless): first frame 653 ops / 224 nodes, ~2.7ms (list bind
  rounds included).
- Photos scrolling (release): a frame that rebinds one row is 4 ops / 4 mount items; frames that reveal no new row call no JS and produce
  no batch. `list_bench` runs at ~9µs per frame back to back (JS ~4µs; ~30µs / JS ~20µs before reactivity went native) and
  ~50µs with caches flushed between frames, but ~60–150µs after the thread sleeps 10ms. The difference is the CPU clocking down while the thread sleeps (Apple CLPC), not the code path; neither QoS (the engine
  thread is USER_INTERACTIVE) nor the Mach time-constraint policy changes it. Judge optimizations by the back-to-back and flushed-cache numbers:
  single-frame `nsh trace` timings are dominated by the clock state.
- On Catalyst (release), rebinding one row while scrolling slowly takes ~0.4ms (clocked down) and rebinding a whole screen of ~32 rows on a
  fast fling 0.9–1.6ms.
- Host startup (`Host::new`, release): ~75–155µs (clock state). The web APIs add nothing until first used, then ~90µs to
  make their classes (`bench_first_web_global`; as JS they cost ~0.8ms as source, ~0.33ms as bytecode).
- Android emulator (arm64, release): rebinding one gallery row while scrolling ~0.3–0.4ms (JS ~60µs); text costs per new
  text: `PrecomputedText` ~120µs, measure ~50µs, final layout reused. First gallery frame 20–50ms (noisy on the emulator).

- Rust components vs the same component in TSX (`component_bench`, `examples/bench`, release, minimum of the runs;
  `TsButton.tsx` is the TSX Button `nsh/ui/mobile` had before components moved into Rust, restyled to draw exactly
  like the Rust one; measured on the former JS runtime):

  | | Rust Button: JS / core | TSX Button: JS / core |
  |---|---|---|
  | first frame, 200 Buttons (ops) | 1.32ms / 0.77ms (1015) | 2.32ms / 0.70ms (2415) |
  | one update of all 200 titles (ops) | 0.42ms / 0.42ms (200) | 0.77ms / 0.36ms (600) |
  | scrolling a 1000-row list, per rebinding frame | 8.3µs / 5.9µs | 10.2µs / 5.4µs |

  JS does half the work or less, a net win of ~30% on the first frame and ~25% on updates. The core's extra time is
  not the component code (its renders and diffs are ~15% of a commit) but the allocator: Taffy's compute takes the
  same samples in both, while its per-layout vectors spend 2.5× longer in `malloc` / `free` / `madvise` after the
  render trees the components build and drop each update (libmalloc coalesces the freed blocks and returns pages).
  Pooling the style buffers bought 3%; a different global allocator in the binaries is the lever left. The
  components' JS face must stay one effect per instance reading the props directly: the first version (Solid's
  `splitProps` / `mergeProps` / `spread` in the former JS runtime) cost 4.0ms of JS for the same first frame, more
  than the TSX Button.
- Canvas recording (release `nsh trace`, minimum of the runs): in the former JS runtime, 2500 iterations of
  `beginPath`, `moveTo`, `lineTo`, `lineWidth =`, `strokeStyle =`, `stroke`, `fillRect` (17 500 calls) recorded into a
  JS array in ~1.4ms, the same with methods generated from the specs as with a hand-written class (an A/B of the two
  mechanisms in one QuickJS run: 4.16 vs 4.22ms, medians within 1%, for 80 000 calls).
  Decoding that drawing in the core (`nsh_core::js::canvas::decode`, from the QuickJS array) takes ~0.52–0.57ms
  (`cargo test --release -p nsh-js --lib bench_canvas_decode -- --ignored --nocapture`), as the hand-written decoder
  it replaced did (0.57ms).
  The native context records into a Rust list (`nsh-js` `canvas.rs`): the same 17 500 calls take ~1.0–1.1ms
  (~1.7ms in the former JS runtime, measured A/B), and 12 of them plus decoding one commit 21–22ms (61ms).

- Where JS time went in the former JS runtime (release `nsh trace`, which logs `event js` per dispatched event;
  2026-10-16, before the runtime went native): with Solid in JS, the empty app's eval was ~250µs (all of it the
  runtime and Solid starting), counter's ~380µs (first frame ~500µs), and a counter tap spent a median 52µs in JS
  (handler, Solid's updates, the microtask flush and the op decode) against 17µs of core commit (11 ops, 10 items).
  The native reactive graph (`nsh-js` `reactive.rs`, still under the JS runtime), measured A/B on the same loaded
  machine: empty eval ~210µs vs ~300µs, counter eval ~330µs
  vs ~460µs, counter tap median ~47µs vs ~69µs in JS, gallery list rebind (`list_bench`) JS ~4µs vs ~21µs per frame.
- The native runtime (`native-pipeline.md`) against the former JS runtime, release, the same counter app (a counter,
  `<If>`, a 3-row `<Each>`) on both: bundle 886 B JS / 1.7 KB bytecode vs 14 KB / 19.6 KB, eval ~95µs vs ~390µs, tap
  median ~5µs vs ~14µs in JS, commit ~15µs vs ~22µs. Scrolling a 1000-row `List` (avatar, two texts, a bound color)
  6000 pt: ~77µs vs ~150µs of JS per rebinding frame, commit ~205µs vs ~276µs.

The dev profile builds `rquickjs-sys` and `taffy` with `opt-level = 3`, so debug timings are meaningful too; release uses fat LTO.
