# Rust gallery

The app is one native component: `App.tsx` mounts `Gallery`, and everything it shows is written in Rust, in the
app's own crate `native/` (`nsh.json` `"rust": "native"`). It shows what Rust components can do, next to the TSX
gallery (`examples/gallery`). The crate's set is the `#[native_module("gallery/native")]` function in
`native/src/lib.rs`, which registers `Gallery` and the crate's `assets/`; `App.tsx` imports it by that specifier.

```sh
cargo build -p nsh-cli
./target/debug/nsh run examples/gallery-rs --target ios   # or android, catalyst
```

`nsh` builds the app its own command line and hosts with the crate linked in (`docs/dev/tooling.md`, "Apps with Rust
components of their own"); editing the Rust code builds them again and restarts the app.
