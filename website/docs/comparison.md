---
title: Comparison with React Native
description: What nsh shares with React Native, where it differs (signals instead of re-rendering, a native runtime, Rust components, a single-binary toolchain), and how common React Native APIs map to nsh.
order: 36
---

# Comparison with React Native

nsh will feel familiar if you know React Native: apps are TSX, they render real native views, styles use React
Native's names and flexbox defaults, and many APIs (`Animated`, `PanResponder`, `Platform`, `Alert`, list props) are
modeled on React Native's. Under that surface, the two work differently, and the differences decide how you write
components. This page goes through both, then maps common React Native APIs to nsh.

React Native is a reference for which capabilities exist, not an API nsh tries to be compatible with: there is no
compatibility layer, and React Native code does not run unchanged.

## What is similar

- **TSX and components.** UI is written as functions returning JSX, composed from a few host elements (`<view>`,
  `<text>`, `<image>`, `<scroll>`, `<input>`, ...).
- **Native views.** Everything renders with UIKit and Android views, not a canvas or a web view, so text, scrolling,
  inputs and accessibility behave like the platform's.
- **Styles.** Style objects with React Native's property names (`backgroundColor`, `paddingHorizontal`,
  `borderRadius`, `shadowOpacity`, `transform: [...]`) and React Native's flexbox defaults: `flexDirection: "column"`,
  `flexShrink: 0`, `position: "relative"`.
- **Familiar APIs.** `Animated` (timing, spring, decay, sequence, parallel, interpolate), `LayoutAnimation`,
  `Easing`, `PanResponder`, `Platform`, `PixelRatio`, `I18nManager`, `AccessibilityInfo`, `Alert`, `Keyboard`,
  `StatusBar`, `SafeAreaView`, `KeyboardAvoidingView`, `BackHandler`, `ToastAndroid`'s `show`: same ideas, often the
  same option names.
- **Lists.** `List` takes `FlatList`'s vocabulary: `onEndReached`, `onEndReachedThreshold`, `ListHeaderComponent`,
  `ListEmptyComponent`, `ItemSeparatorComponent`, `stickyHeaderIndices`, `numColumns`, `inverted`,
  `onViewableItemsChanged`, `viewabilityConfig`, `refreshing` / `onRefresh`, `initialScrollIndex`.
- **Development loop.** Save a file and the running app updates in place, keeping state where it can.

## What is different

### Signals instead of re-rendering

In React, a component function runs again whenever its state or props change, returns a new element tree, and React
diffs it against the previous one. In nsh, **a component runs once**. State lives in signals, and only the parts of
the UI that read a signal update when it changes:

```tsx
// React Native
function Counter() {
  const [count, setCount] = useState(0);
  return (
    <Pressable onPress={() => setCount(count + 1)}>
      <Text>Count: {count}</Text>
    </Pressable>
  );
}
```

```tsx
// nsh
import { signal } from "nsh";

function Counter() {
  const count = signal(0);
  return (
    <view onPress={() => count.update((c) => c + 1)}>
      <text>Count: {count()}</text>
    </view>
  );
}
```

`count()` reads the signal; the compiled JSX turns `{count()}` into a binding that updates that text, and nothing
else, when `count` changes. Consequences:

- **No hooks and no rules of hooks.** Create signals, computed values and effects anywhere: in a component, in a
  condition, in a module. They are owned by the component (or effect) that created them and cleaned up with it.
- **No dependency arrays.** `computed` and `effect` track what they read automatically.
- **No `useCallback` / `useMemo` for identity.** Functions and objects created in a component are created once.
- **Props are live getters.** A prop bound to a signal is a getter that reads it. **Do not destructure props** at the
  top of a component: `const { title } = props` reads the value once and never updates. Use `props.title` where you
  need it, or `split(props, ["title"])` to separate props while keeping them live.
- **Conditionals and lists are elements.** `<if when={…} else={…}>` renders a branch per change of truthiness;
  `<each of={items()}>` keeps rows by key and updates only what changed. A `.map()` in JSX works but creates every row
  again on each change.

The model is the one of [SolidJS](https://www.solidjs.com/), with nsh's own API (`signal`, `computed`, `effect`,
`batch`, `untracked`, `cleanup`, `context`), and the reactive graph itself is native. See
[Reactivity](./reactivity.md).

### A native runtime, not a JS framework

A React Native bundle contains React, React Native's JS and your app; React reconciles in JS, and the results reach the
native side through the bridge or, in the New Architecture, JSI and Fabric's C++ renderer.

An nsh bundle contains **only your app's code**. The reactive graph, the renderer, the components of `nsh/ui/mobile`,
layout (Taffy), list virtualization, animation and every module you import are Rust, running on the engine thread. Your
compiled JSX calls the native renderer directly; every change is an operation pushed into a native queue, one JS turn
becomes one commit, and one batch of view operations crosses to the main thread. There is no bridge, no JSON
serialization and no virtual DOM.

The JS engine is QuickJS: small, fast to start, compiled ahead of time to bytecode by `nsh build`, without a JIT
(Hermes, React Native's default engine, has no JIT either). Since so little runs in JS, QuickJS is rarely the
bottleneck; heavy computation in your own code still costs more than in a JIT engine. See
[Architecture](./architecture.md) and [Performance](./performance.md).

### Components are Rust, and look the same everywhere

React Native's core components map to platform widgets (`Switch` is a `UISwitch` on iOS and a Material switch on
Android). nsh's runtime provides **capabilities, not widgets**: the host elements are primitives, and the components
of the mobile UI set (`Button`, `Card`, `TextField`, `Stepper`, `Sheet`, `SelectList`, ...) are written in Rust from
those primitives, themed, and look the same on iOS and Android. There is no `Switch`, `ActivityIndicator` or `Picker`
mapped to platform widgets; where a platform has something specific, it is in `nsh/ios`, `nsh/android` or in the
`ios={{…}}` / `android={{…}}` props.

### Styling: React Native styles plus CSS and Tailwind

Besides React Native's style properties, nsh supports CSS grid, gradients, filters, blend modes, outlines and CSS
transitions, and **Tailwind is built in**: `class="flex-row gap-2 p-4 bg-white dark:bg-zinc-900"` works without any
setup, with the theme configured in `nsh.json`. The build warns about unknown classes, with their position. See [Styling](./styling.md) and
[Tailwind](./tailwind.md).

### Layout engine

React Native uses Yoga; nsh uses [Taffy](https://github.com/DioxusLabs/taffy) with React Native's defaults (and grid).
One visible difference: a `minWidth` / `minHeight` that is unset or `auto` is 0, as in Yoga, so `flex: 1` text in a row
wraps instead of overflowing.

### Toolchain

| | React Native | nsh |
|---|---|---|
| Needs | Node.js, npm / yarn, Metro, Xcode project + CocoaPods, Gradle | the `nsh` binary (plus Xcode or the Android SDK to install on devices) |
| Bundler | Metro (JS) | rolldown + Oxc + nsh's JSX compiler (Rust) |
| Types | `@types/react`, `react-native` from `node_modules` | generated into `.nsh/types` by every command; no `node_modules` |
| Run in development | `npx react-native run-ios` / `run-android` (builds the native app) | `nsh run` (prebuilt development hosts, nothing to compile) |
| Ship | Xcode archive, Gradle `assembleRelease` | `nsh package` (`.ipa` / `.apk` from the prebuilt runtime) |
| Over-the-air updates | third-party services | [signed packages](./signed-packages.md) the host app verifies |
| Headless testing | Jest with mocks of native modules | [`nsh trace`](./testing-and-trace.md) runs the real runtime headless |

### Native code

In React Native, any app can add native modules and native components (Turbo Modules, Fabric components, Expo
modules), and npm offers thousands. In nsh, a standalone app built with `nsh package` contains only the prebuilt
runtime. Native views are added by an app that **embeds** nsh, which registers them for `nativeComponent()` (see
[Native components](./native-components.md)). That keeps standalone builds free of native compilation, but it is a real
limitation if you depend on native SDKs.

### Ecosystem

React Native has a large ecosystem: navigation libraries, state managers, UI kits, and native SDK wrappers. nsh has
its own navigation (`createNavigator`, `Stack`, `Screen`, `Modal`), its own components and web APIs (`fetch`, streams,
`URL`, `Blob`, `FormData`, `AbortController`, `TextEncoder`). Libraries built on React or React Native do not work in
nsh, and nsh refuses to bundle `solid-js` (its own reactive API replaces it); plain JS libraries that only need
standard JS and the web APIs nsh provides can be bundled.

### Security model

React Native apps run their JS with whatever the app can do. nsh runs every app in a VM with
[limits and capabilities](./sandbox-and-capabilities.md) enforced natively (allowed hosts, web view hosts, status bar),
which is what makes it practical to run third-party code inside a host app.

### Debugging

React Native has React DevTools and a JS debugger. nsh has logs forwarded to the terminal, an error overlay with
source-mapped stacks, per-commit timings and `nsh trace`; it has no step debugger or element inspector today. See
[Dev server and debugging](./dev-server-and-debugging.md).

## API mapping

### Components and elements

| React Native | nsh |
|---|---|
| `<View>` | `<view>` |
| `<Text>` | `<text>`; nested `<text>` is a span |
| `<Image source={{ uri }}>` | `<image src="https://…">` (or `src={importedImage}`, `src="asset:/…"`) |
| `<ScrollView>` | `<scroll>` |
| `<TextInput onChangeText>` | `<input onChangeText>` (also `TextField` in `nsh/ui/mobile`) |
| `<FlatList data renderItem>` | `<List each={data()}>{(item, index) => …}</List>` from `nsh/list` (`item` and `index` are accessors) |
| `<SectionList>` | `List` with header rows and `stickyHeaderIndices` |
| `<Pressable>`, `<TouchableOpacity>` | `onPress`, `onLongPress`, `onPressIn`, `onPressOut`, `activeOpacity`, `hitSlop` on any `<view>`; `android={{ ripple }}` |
| `<RefreshControl>` | `refreshing` / `onRefresh` on `<scroll>` and `List` |
| `<Modal>` | `Modal` from `nsh/ui/mobile` |
| `<SafeAreaView>` | `SafeAreaView` from `nsh/ui/mobile` |
| `<KeyboardAvoidingView>` | `KeyboardAvoidingView` from `nsh/ui/mobile` |
| `<StatusBar>` | `StatusBar` from `nsh/ui/mobile` |
| `<Button>` | `Button` from `nsh/ui/mobile` (variants, one look on both platforms) |
| `<InputAccessoryView>` | `InputAccessoryView` from `nsh/ios` |
| `react-native-webview` | `WebView` from `nsh/webview` |
| `react-native-video` | `Video` from `nsh/video` |
| `@react-navigation/native-stack` | `createNavigator`, `Stack`, `Screen` from `nsh/ui/mobile` |
| `requireNativeComponent` | `nativeComponent()` from `nsh/native` |
| `{cond && <X />}` | `<if when={cond()}><X /></if>` |
| `{items.map(…)}` | `<each of={items()}>{(item) => …}</each>` |

### React and hooks

| React | nsh (`import … from "nsh"`) |
|---|---|
| `useState(0)` | `const n = signal(0)`; read `n()`, write `n.set(1)` / `n.update((v) => v + 1)` |
| `useMemo(() => f(a, b), [a, b])` | `computed(() => f(a(), b()))` |
| `useEffect(() => { …; return cleanup }, [deps])` | `effect(() => { …; cleanup(() => …) })` (dependencies tracked) |
| `useEffect(() => …, [])` (on mount) | code in the component body, or an `effect` that reads no signals |
| `useCallback(fn, deps)` | `fn`: components run once |
| `useRef()` for a view | `let node: YNode; <view ref={node} />` or `ref={(n) => …}` |
| `createContext` / `useContext` | `const Theme = context(default)`; `<Theme.Provider value>`; read `Theme()` |
| `ReactDOM.unstable_batchedUpdates` | `batch(() => …)` |
| `AppRegistry.registerComponent` | `mount(App)` |
| `React.memo` | not needed |

### APIs

| React Native | nsh |
|---|---|
| `Animated.Value` | `createNativeValue()` from `nsh/animated` |
| `Animated.timing / spring / decay / sequence / parallel / stagger / loop` | the same names on `Animated` from `nsh/animated` (always native) |
| `Animated.event` (scroll) | `scrollValue={value}` on `<scroll>` / `List` |
| `Animated.event` (pan) | `pan={{ x, y }}` on a `<view>` |
| `LayoutAnimation`, `Easing` | `LayoutAnimation`, `Easing` from `nsh/animated` |
| `PanResponder` | `PanResponder` from `nsh/gesture` |
| `Dimensions`, `useWindowDimensions` | `windowSize()` from `nsh/device` |
| `useColorScheme`, `Appearance` | `colorScheme()` from `nsh/device` |
| `PixelRatio.getFontScale()` | `fontScale()` from `nsh/device` (also `PixelRatio`) |
| `Platform`, `PixelRatio`, `I18nManager` | the same names from `nsh/device` |
| `AccessibilityInfo` | `AccessibilityInfo`, `screenReaderEnabled()`, `reduceMotion()` from `nsh/accessibility` |
| `Alert.alert` | `Alert.alert` from `nsh/ui/mobile` (returns a promise of the chosen index) |
| `ActionSheetIOS` | `ActionSheet.show` from `nsh/ui/mobile` (both platforms) |
| `Keyboard.dismiss` | `Keyboard.dismiss` from `nsh/ui/mobile`; `keyboard()` for its height |
| `useSafeAreaInsets` (react-native-safe-area-context) | `useSafeAreaInsets()` / `safeArea()` from `nsh/ui/mobile` |
| `BackHandler`, `ToastAndroid` | `BackHandler`, `Toast.show` from `nsh/android` |
| `Image.getSize`, `Image.prefetch` | `Image.getSize`, `Image.prefetch` from `nsh/image` |
| `fetch`, `AbortController`, `URL`, `TextEncoder` | globals, as in browsers (see [Web APIs](../modules/web-apis.md)) |

Not provided today: persistent storage (`AsyncStorage`), `Linking`, `Clipboard`, `Share`, `Vibration`, `WebSocket`,
`Switch`, `ActivityIndicator`, `Picker`.

## When to choose which

nsh is a good fit when you want native views with less JS at runtime, a toolchain without Node.js, consistent
components on both platforms, or a sandbox for running downloaded or third-party code inside an existing app. React
Native is the better fit when you depend on its library ecosystem, on native SDKs wrapped as React Native modules, or on
a large community and long track record.

## See also

- [JSX and components](./jsx-and-components.md)
- [Reactivity](./reactivity.md)
- [Architecture](./architecture.md)
