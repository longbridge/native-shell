# Animation

- **Animation**: `createNativeValue()` → `timing` / `spring` / `decay` (return a Promise), `interpolate` (numbers, colors or
  `"deg"` / `"rad"` angles; `extrapolate` / `extrapolateLeft` / `extrapolateRight`: `clamp` (default) / `extend` / `identity`;
  `easing`), `add/sub/mul/div/mod`, `diffClamp(min, max)`. `Easing` (React Native's: `linear`, `ease`, `quad`, `cubic`, `poly`,
  `sin`, `circle`, `exp`, `elastic`, `back`, `bounce`, `bezier`, `in` / `out` / `inOut`) is data decoded in Rust
  (`nsh_animation::Easing::Curve`), never a JS function: the curves, their parameters and defaults are one table
  (`nsh_core::js::animation`), which decodes them, declares their types (`animation::declarations`) and gives `nsh/animated` its curves (`animation::tables`). `Animated.timing` / `spring` / `decay` / `sequence` / `parallel` / `stagger` /
  `delay` / `loop` build a `Composite` tree that runs entirely natively (`AnimationOp::StartComposite`): each step starts at the exact end
  time of the previous one (no frame gaps), `stop()` / `reset()` (back to the values at start) / `start(cb)` report `finished` once
  for the whole tree; leaf animations get internal ids (`LEAF_IDS`) that never reach JS. Composites are released when JS
  lets go of them (their finalizers queue `ReleaseComposite`). Rust components build the same trees with
  `nsh_core::ui::Animation` (composite keys odd, JS's even).
  Bindable: `opacity`, `backgroundColor`, and the translate / scale / rotate parts of `transform`. Driven on the main thread by `CADisplayLink`,
  paused while nothing animates. `scrollValue` writes the scroll offset into a native value in the same frame; `pan` lets drags drive native values
  directly, and JS receives the velocity when the drag ends. Native values referenced by a view's style (or `pan`, `scrollValue`, ...) are kept by its node
  (`nsh-js` `dom.rs`) and derived nodes hold their inputs, so bindings do not break when JS objects are collected.
- **Layout animation**: `LayoutAnimation.configureNext(config)` (or the `easeInEaseOut` / `linear` / `spring` presets) animates
  the next commit's frames, insertions and removals natively. The presets are `LayoutAnimation::EASE_IN_EASE_OUT` /
  `LINEAR` / `SPRING` in Rust (what Rust components pass to `animate_layout`), handed to `nsh/animated` as data; the config's
  decoding and types come from the same module.
