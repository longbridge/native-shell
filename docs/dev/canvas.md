# Canvas

- **Canvas** (`Canvas` component, `<canvas>` element): an HTML Canvas 2D subset, recorded and replayed natively
  (`nsh_core::canvas`: `Drawing { ops, slots }`). `draw={(ctx, { width, height }) => ...}` runs again when the signals it reads
  change (sizes come from `onLayout`). `CanvasRenderingContext2D`: state (`save` / `restore`, `globalAlpha`,
  `globalCompositeOperation`, line styles and dashes, shadows, `font` (CSS shorthand), `textAlign` / `textBaseline`),
  transforms, paths (`arc`, `arcTo`, `ellipse`, curves, `rect`, `roundRect`), `fill` / `stroke` / `clip` (also of a `Path2D`,
  which takes SVG path data; arcs become cubics), `fillRect` / `strokeRect` / `clearRect`, `fillText` / `strokeText` /
  `measureText` (synchronous, through the engine's text engine; only `width`), linear / radial / conic
  gradients, `drawImage` (all three forms, images by URI or asset). Any number argument and colors may be native values
  (`createNativeValue`, `interpolate`, ...): those become slots bound like style values (`AnimProp::CanvasArg`) and the
  platform redraws on its own while they animate, without JS. A drawing travels in its own op (`Op::Draw`), recorded into
  a Rust list of its own (`nsh-js` `canvas.rs`): creating native values while drawing pushes ops of its own. Each
  operation is one variant of `CanvasOp` / `PathOp` with `#[derive(JsOps)]` (`nsh_core::js::ops`): `#[op(code = CV_…)]` reads its fields in order
  (`OpArg`: numbers or native values, paints, `Path2D` lists, string unions; an invalid color or font drops the
  operation, as in HTML Canvas), and `method = …, params = …` / `property = …, initial = …` describe the methods and
  properties that only record (`moveTo`, `fillRect`, `lineWidth`, ...): `nsh-js` makes them natively from these specs
  (`CanvasOp::methods` / `properties`) and their declarations are generated (`canvas::declarations`); what has logic
  (overloads, `roundRect`'s radii, `save` / `restore`, gradients, images) is written in `canvas.rs`. The core's decoder
  (`nsh_core::js::canvas::decode`) reads the recorded list. Recording costs: `docs/dev/performance.md`. iOS replays
  into the `drawRect:` context, keeping its own transform and path in view space; Android `CanvasBox.java` interprets an `int[]` (`encode.rs`). Not
  supported: `getImageData` / `putImageData`, filters, `isPointInPath`, SVG images.

Rust components record drawings with `ui::Painter` (the same operations; `docs/dev/components.md`) and measure text
with `RenderCx::measure_text`, on the core's text engine.

The drawing format is documented in `crates/nsh-core/src/canvas.rs`.
