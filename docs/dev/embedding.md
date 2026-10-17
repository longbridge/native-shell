# Packaging and embedding

- `nsh package <entry> --target ios|ios-simulator|android|linux|macos|windows|gpui` builds an installable app from the prebuilt runtime: metadata from
  `nsh.json` next to the entry (`name`, `id`, `version`, `build`, `icon`; `appearance` also sets `UIUserInterfaceStyle`
  and a dark window theme, so the launch screen is in it) or flags. iOS: `.app`, `.ipa` with `--sign <identity>
  --profile <file.mobileprovision>` (entitlements come from the profile; icons are compiled by `actool` into `Assets.car`).
  Android: an APK assembled without Gradle (`aapt2 link`, native libraries stored uncompressed and 16 KB aligned, `apksigner`;
  the debug key unless `--keystore`). Linux: a directory `<name>/` with the release runner (`crates/nsh-linux-app`) as
  the executable, `nsh/bundle.nshc`, `nsh/assets/` and `nsh/app.json` (the window title and the GTK application id,
  which the runner reads next to itself), `<id>.desktop` (`StartupWMClass` is the id, which GTK makes the window's
  app id) and `<id>.png`, plus the same as `<name>.tar.gz`; GTK 4 and Pango must be installed where it runs. macOS: `<name>.app` with the
  release runner (`crates/nsh-macos-app`) as `Contents/MacOS/<executable>`, `Contents/Resources/nsh/` holding
  `bundle.nshc`, `assets/` and `app.json` (the window title and appearance), the icon as `AppIcon.icns` (`sips`), signed
  ad hoc (`codesign -s -`); `--run` opens it.
  Windows: a directory `<name>\` with the release runner (`crates/nsh-windows-app`, a windowed program: no console)
  as `<name>.exe` (characters Windows forbids in file names dropped), the icon embedded as its icon group 1 (which the
  window class loads, and Explorer and the taskbar show) with Windows' resource update API (`crates/nsh-cli/src/windows.rs`:
  16 to 64 pixels as DIBs plus a 256-pixel PNG; packaging off Windows keeps the runner's icon, with a warning), and
  `nsh\bundle.nshc`, `nsh\assets\` and `nsh\app.json` (the window title, the app id, which groups the app's windows in
  the taskbar, the version and the appearance, which the window opens in); plus the same as `<name>.zip`. No tools
  are needed; `--run` launches the executable.
  GPUI: packaged as the native apps of the machine it runs on are (Linux's directory and `.tar.gz`, macOS' `.app`,
  Windows' directory and `.zip`), with the GPUI release runner (`crates/nsh-gpui-app`) as the executable; it finds
  `nsh/` next to itself, or `Contents/Resources/nsh` inside an app bundle (`nsh_gpui_app::resources`).
- Embedding: iOS apps link `NativeShell.xcframework` and `import NativeShell` (Swift: `NshAppView`, a `UIView`;
  `NshTrust`, `NshPackage`, `NshLimits`, `NshError`); Android apps add `nsh.aar` and use `com.longbridge.nsh.NshView`.
  `*Dev` / `-dev` variants can also load from a dev server. See `examples/embed-*`. Rust desktop apps use the platform
  crate itself: `nsh_platform_linux::AppView` (a GTK widget), `nsh_platform_macos::AppView` (an `NSView`) or
  `nsh_platform_windows::AppView` (a child window of the app's `HWND`, run by the app's message loop; `windows.md`).
  GPUI and GPUI Kit apps use `nsh_platform_gpui::AppView`, a GPUI view (`gpui.md`): `let view = cx.new(|cx|
  AppView::new(options, window, cx));` inside a window, `view.read(cx).load(App::Js(bundle))`, then put the entity in
  any element tree (in a GPUI Kit window, as a child of its `Root`); it fills the space its parent gives it and takes
  the focus it is given (`Focusable`). The crate depends on `gpui-fast` 0.1 from crates.io (re-exported as
  `nsh_platform_gpui::gpui`); GPUI Kit's `gpui-fast` feature makes `gpui_component` use that same crate, so both share
  one GPUI. `open_app_window` opens a window showing an app in a running GPUI app.
- **Native components** (`nsh/native`; `component.rs`, `ComponentBox.java`): the embedding app registers its own native
  views by name — Swift `NshAppView.register(component:) { context in … }` (`NshComponent`: `view`, `setProp`,
  `command`), Java `NshView.registerComponent(name, factory)` (`NshComponent`), Rust `AppView::register_component`
  (`NativeComponent`) — one registry per view, no globals. JS declares one with `nativeComponent<P, E>(name, { props,
  events })`; the element is a `<native>` leaf: `component` names it, each own prop goes as `$name` (any JSON value,
  `nsh_core::JsonValue`, sent when it changes; `undefined` removes it), view props (style, classes, accessibility, `ref`)
  stay the box's. The core sends `Create` (`ViewKind::Native`, a box decorated and placed like any view), then
  `MountItem::Component { name }` and `ComponentProps` (all props, then the changed ones). The component fills the box,
  emits events (`ComponentContext::emit` → `onNativeEvent` `{ name, data }` → the `on<Name>` handler) and may report
  its own size (`set_size`; the leaf measures to it where the style leaves a dimension open, like an image's natural
  size). `ref.command(name, args)` → `Command::Native`. Unknown names render an empty box (warned once). Props and events
  cross the C ABI / JNI as JSON text (Swift `JSONSerialization`, Java `org.json` as maps and lists). Registered
  components are the embedding app's own code: capabilities do not apply to them.
- **The iOS SDK** (`crates/xtask/src/ios.rs`): per platform slice (device, simulator, Catalyst) a static
  `NativeShell.framework` whose binary is `nsh-platform-ios` with feature `embed` (built by `cargo rustc --crate-type
  staticlib`, so the hosts never build or export it) merged by `libtool` with the Swift module, compiled with library
  evolution (`.swiftinterface`: apps on other Swift compilers can import it). The Swift module imports the C ABI's
  module `NativeShellC` with `internal import`, so it is neither in the interface nor shipped; its module map's `link`
  lines become autolink entries of the Swift objects (apps link nothing but the framework). Catalyst frameworks use the
  versioned macOS layout and Swift needs the SDK's `iOSSupport` paths. The C functions return errors as strings
  (`nsh_string_free`) that Swift throws as `NshError`; the error handler's context is released by a callback.
