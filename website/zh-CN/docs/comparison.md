---
title: 与 React Native 对比
description: nsh 与 React Native 的共同点、不同之处（用 signal 代替重新渲染、原生运行时、Rust 组件、单一二进制的工具链），以及常用 React Native API 在 nsh 中的对应。
order: 36
---

# 与 React Native 对比

如果你熟悉 React Native，nsh 会让你感到亲切：应用用 TSX 编写，渲染真正的原生视图，样式沿用 React Native 的属性名和
flexbox 默认值，许多 API（`Animated`、`PanResponder`、`Platform`、`Alert`、列表的 props）也参照 React Native 设计。
但在这层表面之下，两者的工作方式不同，而这些差异决定了你如何编写组件。本页先逐一介绍异同，再列出常用 React Native API
在 nsh 中的对应。

React Native 是判断应该具备哪些能力的参考，而不是 nsh 试图兼容的 API：nsh 没有兼容层，React Native 代码也无法原样运行。

## 相似之处

- **TSX 和组件。** UI 写成返回 JSX 的函数，由少数几个宿主元素（`<view>`、`<text>`、`<image>`、`<scroll>`、
  `<input>` 等）组合而成。
- **原生视图。** 一切都用 UIKit 和 Android 视图渲染，而不是 canvas 或 web view，因此文本、滚动、输入框和无障碍的行为都
  与平台一致。
- **样式。** 样式对象使用 React Native 的属性名（`backgroundColor`、`paddingHorizontal`、`borderRadius`、
  `shadowOpacity`、`transform: [...]`）和 React Native 的 flexbox 默认值：`flexDirection: "column"`、`flexShrink: 0`、
  `position: "relative"`。
- **熟悉的 API。** `Animated`（timing、spring、decay、sequence、parallel、interpolate）、`LayoutAnimation`、`Easing`、
  `PanResponder`、`Platform`、`PixelRatio`、`I18nManager`、`AccessibilityInfo`、`Alert`、`Keyboard`、`StatusBar`、
  `SafeAreaView`、`KeyboardAvoidingView`、`BackHandler`、`ToastAndroid` 的 `show`：思路相同，选项名也常常相同。
- **列表。** `List` 沿用 `FlatList` 的术语：`onEndReached`、`onEndReachedThreshold`、`ListHeaderComponent`、
  `ListEmptyComponent`、`ItemSeparatorComponent`、`stickyHeaderIndices`、`numColumns`、`inverted`、
  `onViewableItemsChanged`、`viewabilityConfig`、`refreshing` / `onRefresh`、`initialScrollIndex`。
- **开发循环。** 保存文件后，正在运行的应用会就地更新，并尽可能保留状态。

## 不同之处

### 用 signal 代替重新渲染

在 React 中，组件函数会在其 state 或 props 变化时重新运行，返回一棵新的元素树，再由 React 与上一棵树做 diff。在 nsh
中，**组件只运行一次**。状态存放在 signal 中，signal 变化时，只有读取它的那部分 UI 会更新：

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

`count()` 读取 signal；编译后的 JSX 会把 `{count()}` 变成一个绑定，在 `count` 变化时只更新那段文本，别的都不动。由此
带来的结果是：

- **没有 hook，也没有 hook 规则。** 可以在任何地方创建 signal、computed 值和 effect：组件中、条件分支中、模块中。它们
  归创建它们的组件（或 effect）所有，并随之一起清理。
- **没有依赖数组。** `computed` 和 `effect` 会自动追踪它们读取的内容。
- **不需要为保持引用一致而用 `useCallback` / `useMemo`。** 组件中创建的函数和对象只会创建一次。
- **props 是实时的 getter。** 绑定到 signal 的 prop 是一个读取该 signal 的 getter。**不要在组件顶部解构 props**：
  `const { title } = props` 只读取一次值，之后再也不会更新。请在需要的地方使用 `props.title`，或用
  `split(props, ["title"])` 拆分 props，同时保持它们的实时性。
- **条件和列表是元素。** `<if when={…} else={…}>` 在真假值每次变化时渲染对应分支；`<each of={items()}>` 按 key 保留
  行，只更新变化的部分。JSX 中的 `.map()` 可以用，但每次变化都会重新创建所有行。

这一模型与 [SolidJS](https://www.solidjs.com/) 相同，但使用 nsh 自己的 API（`signal`、`computed`、`effect`、`batch`、
`untracked`、`cleanup`、`context`），而且响应式图本身是原生的。参见[响应式](./reactivity.md)。

### 原生运行时，而非 JS 框架

React Native 的 bundle 包含 React、React Native 的 JS 和你的应用；React 在 JS 中做协调（reconcile），结果通过 bridge
送到原生侧，在新架构中则通过 JSI 和 Fabric 的 C++ 渲染器。

nsh 的 bundle **只包含你的应用代码**。响应式图、渲染器、`nsh/ui/mobile` 的组件、布局（Taffy）、列表虚拟化、动画以及你
导入的每个模块都是 Rust，运行在引擎线程上。编译后的 JSX 直接调用原生渲染器；每个变化都是推入原生队列的一个操作，一个
JS 轮次成为一次 commit，一批视图操作跨到主线程。没有 bridge，没有 JSON 序列化，也没有虚拟 DOM。

JS 引擎是 QuickJS：体积小、启动快，由 `nsh build` 预先编译为字节码，没有 JIT（React Native 的默认引擎 Hermes 同样没有
JIT）。由于在 JS 中运行的东西很少，QuickJS 很少成为瓶颈；不过你自己代码中的繁重计算仍然比在 JIT 引擎中代价更高。参见
[架构](./architecture.md)和[性能](./performance.md)。

### 组件是 Rust，并且在各平台上外观一致

React Native 的核心组件映射到平台控件（`Switch` 在 iOS 上是 `UISwitch`，在 Android 上是 Material 开关）。nsh 的运行时
提供的是**能力，而不是控件**：宿主元素是基础原语，移动端 UI 集的组件（`Button`、`Card`、`TextField`、`Stepper`、
`Sheet`、`SelectList` 等）用 Rust 基于这些原语编写，支持主题，在 iOS 和 Android 上外观一致。没有映射到平台控件的
`Switch`、`ActivityIndicator` 或 `Picker`；平台特有的功能放在 `nsh/ios`、`nsh/android` 或 `ios={{…}}` /
`android={{…}}` props 中。

### 样式：React Native 样式加上 CSS 和 Tailwind

除了 React Native 的样式属性，nsh 还支持 CSS grid、渐变、滤镜、混合模式、outline 和 CSS 过渡，并且**内置 Tailwind**：
`class="flex-row gap-2 p-4 bg-white dark:bg-zinc-900"` 无需任何配置即可使用，主题在 `nsh.json` 中配置。构建时会对未知
的类名给出警告，并附带其位置。参见[样式](./styling.md)和 [Tailwind](./tailwind.md)。

### 布局引擎

React Native 使用 Yoga；nsh 使用 [Taffy](https://github.com/DioxusLabs/taffy)，并采用 React Native 的默认值（另外支持
grid）。一个可见的差异：未设置或为 `auto` 的 `minWidth` / `minHeight` 视为 0，与 Yoga 相同，所以行中 `flex: 1` 的文本
会换行而不是溢出。

### 工具链

| | React Native | nsh |
|---|---|---|
| 需要 | Node.js、npm / yarn、Metro、Xcode 项目 + CocoaPods、Gradle | `nsh` 二进制文件（要安装到设备上还需 Xcode 或 Android SDK） |
| 打包器 | Metro（JS） | rolldown + Oxc + nsh 的 JSX 编译器（Rust） |
| 类型 | 来自 `node_modules` 的 `@types/react`、`react-native` | 每个命令都会生成到 `.nsh/types`；没有 `node_modules` |
| 开发时运行 | `npx react-native run-ios` / `run-android`（构建原生应用） | `nsh run`（预构建的开发宿主，无需编译） |
| 发布 | Xcode archive、Gradle `assembleRelease` | `nsh package`（基于预构建运行时生成 `.ipa` / `.apk`） |
| 热更新（OTA） | 第三方服务 | 由宿主应用验证的[签名包](./signed-packages.md) |
| 无界面测试 | Jest 加原生模块的 mock | [`nsh trace`](./testing-and-trace.md) 无界面运行真实的运行时 |

### 原生代码

在 React Native 中，任何应用都可以添加原生模块和原生组件（Turbo Modules、Fabric 组件、Expo 模块），npm 上有成千上万个。
在 nsh 中，用 `nsh package` 构建的独立应用只包含预构建的运行时。原生视图由**嵌入** nsh 的应用添加，由它注册给
`nativeComponent()` 使用（参见[原生组件](./native-components.md)）。这让独立构建无需原生编译，但如果你依赖原生 SDK，
这确实是一个限制。

### 生态

React Native 拥有庞大的生态：导航库、状态管理、UI 套件和原生 SDK 的封装。nsh 有自己的导航（`createNavigator`、`Stack`、
`Screen`、`Modal`）、自己的组件和 Web API（`fetch`、stream、`URL`、`Blob`、`FormData`、`AbortController`、
`TextEncoder`）。基于 React 或 React Native 的库无法在 nsh 中运行，nsh 也拒绝打包 `solid-js`（它自己的响应式 API 取代了
它）；只依赖标准 JS 和 nsh 所提供 Web API 的纯 JS 库可以被打包。

### 安全模型

React Native 应用的 JS 拥有应用本身的全部权限。nsh 则把每个应用都运行在一个虚拟机中，由原生侧强制执行
[限制和能力](./sandbox-and-capabilities.md)（允许的主机、web view 主机、状态栏），正因如此，在宿主应用中运行第三方代码
才切实可行。

### 调试

React Native 有 React DevTools 和 JS 调试器。nsh 有转发到终端的日志、带 source map 堆栈的错误浮层、每次 commit 的耗时
以及 `nsh trace`；目前还没有单步调试器或元素检查器。参见[开发服务器与调试](./dev-server-and-debugging.md)。

## API 对应

### 组件和元素

| React Native | nsh |
|---|---|
| `<View>` | `<view>` |
| `<Text>` | `<text>`；嵌套的 `<text>` 是 span |
| `<Image source={{ uri }}>` | `<image src="https://…">`（或 `src={importedImage}`、`src="asset:/…"`） |
| `<ScrollView>` | `<scroll>` |
| `<TextInput onChangeText>` | `<input onChangeText>`（`nsh/ui/mobile` 中还有 `TextField`） |
| `<FlatList data renderItem>` | `nsh/list` 中的 `<List each={data()}>{(item, index) => …}</List>`（`item` 和 `index` 是 accessor） |
| `<SectionList>` | 带标题行和 `stickyHeaderIndices` 的 `List` |
| `<Pressable>`、`<TouchableOpacity>` | 任意 `<view>` 上的 `onPress`、`onLongPress`、`onPressIn`、`onPressOut`、`activeOpacity`、`hitSlop`；`android={{ ripple }}` |
| `<RefreshControl>` | `<scroll>` 和 `List` 上的 `refreshing` / `onRefresh` |
| `<Modal>` | `nsh/ui/mobile` 中的 `Modal` |
| `<SafeAreaView>` | `nsh/ui/mobile` 中的 `SafeAreaView` |
| `<KeyboardAvoidingView>` | `nsh/ui/mobile` 中的 `KeyboardAvoidingView` |
| `<StatusBar>` | `nsh/ui/mobile` 中的 `StatusBar` |
| `<Button>` | `nsh/ui/mobile` 中的 `Button`（多种变体，两个平台外观一致） |
| `<InputAccessoryView>` | `nsh/ios` 中的 `InputAccessoryView` |
| `react-native-webview` | `nsh/webview` 中的 `WebView` |
| `react-native-video` | `nsh/video` 中的 `Video` |
| `@react-navigation/native-stack` | `nsh/ui/mobile` 中的 `createNavigator`、`Stack`、`Screen` |
| `requireNativeComponent` | `nsh/native` 中的 `nativeComponent()` |
| `{cond && <X />}` | `<if when={cond()}><X /></if>` |
| `{items.map(…)}` | `<each of={items()}>{(item) => …}</each>` |

### React 和 hook

| React | nsh（`import … from "nsh"`） |
|---|---|
| `useState(0)` | `const n = signal(0)`；读取用 `n()`，写入用 `n.set(1)` / `n.update((v) => v + 1)` |
| `useMemo(() => f(a, b), [a, b])` | `computed(() => f(a(), b()))` |
| `useEffect(() => { …; return cleanup }, [deps])` | `effect(() => { …; cleanup(() => …) })`（依赖自动追踪） |
| `useEffect(() => …, [])`（挂载时） | 写在组件函数体中，或使用不读取任何 signal 的 `effect` |
| `useCallback(fn, deps)` | `fn`：组件只运行一次 |
| 用于视图的 `useRef()` | `let node: YNode; <view ref={node} />` 或 `ref={(n) => …}` |
| `createContext` / `useContext` | `const Theme = context(default)`；`<Theme.Provider value>`；读取用 `Theme()` |
| `ReactDOM.unstable_batchedUpdates` | `batch(() => …)` |
| `AppRegistry.registerComponent` | `mount(App)` |
| `React.memo` | 不需要 |

### API

| React Native | nsh |
|---|---|
| `Animated.Value` | `nsh/animated` 中的 `createNativeValue()` |
| `Animated.timing / spring / decay / sequence / parallel / stagger / loop` | `nsh/animated` 中 `Animated` 上的同名方法（始终原生执行） |
| `Animated.event`（滚动） | `<scroll>` / `List` 上的 `scrollValue={value}` |
| `Animated.event`（拖动） | `<view>` 上的 `pan={{ x, y }}` |
| `LayoutAnimation`、`Easing` | `nsh/animated` 中的 `LayoutAnimation`、`Easing` |
| `PanResponder` | `nsh/gesture` 中的 `PanResponder` |
| `Dimensions`、`useWindowDimensions` | `nsh/device` 中的 `windowSize()` |
| `useColorScheme`、`Appearance` | `nsh/device` 中的 `colorScheme()` |
| `PixelRatio.getFontScale()` | `nsh/device` 中的 `fontScale()`（也有 `PixelRatio`） |
| `Platform`、`PixelRatio`、`I18nManager` | `nsh/device` 中的同名 API |
| `AccessibilityInfo` | `nsh/accessibility` 中的 `AccessibilityInfo`、`screenReaderEnabled()`、`reduceMotion()` |
| `Alert.alert` | `nsh/ui/mobile` 中的 `Alert.alert`（返回所选索引的 promise） |
| `ActionSheetIOS` | `nsh/ui/mobile` 中的 `ActionSheet.show`（两个平台均可用） |
| `Keyboard.dismiss` | `nsh/ui/mobile` 中的 `Keyboard.dismiss`；用 `keyboard()` 获取其高度 |
| `useSafeAreaInsets`（react-native-safe-area-context） | `nsh/ui/mobile` 中的 `useSafeAreaInsets()` / `safeArea()` |
| `BackHandler`、`ToastAndroid` | `nsh/android` 中的 `BackHandler`、`Toast.show` |
| `Image.getSize`、`Image.prefetch` | `nsh/image` 中的 `Image.getSize`、`Image.prefetch` |
| `fetch`、`AbortController`、`URL`、`TextEncoder` | 全局对象，与浏览器中一样（参见 [Web API](../modules/web-apis.md)） |

目前尚未提供：持久化存储（`AsyncStorage`）、`Linking`、`Clipboard`、`Share`、`Vibration`、`WebSocket`、`Switch`、
`ActivityIndicator`、`Picker`。

## 如何选择

如果你希望使用原生视图且运行时 JS 更少、工具链不依赖 Node.js、两个平台上组件外观一致，或者需要在现有应用中以沙箱方式
运行下载的或第三方的代码，nsh 很合适。如果你依赖 React Native 的库生态、封装成 React Native 模块的原生 SDK，或者看重
庞大的社区和长期的积累，React Native 是更好的选择。

## 另请参阅

- [JSX 与组件](./jsx-and-components.md)
- [响应式](./reactivity.md)
- [架构](./architecture.md)
