---
title: 简介
description: Native Shell 是什么：让原生 iOS、Android 应用在发布之后加载 UI 模块的 Shell，独立应用是次要用途。
order: 1
---

# 简介

Native Shell（简称 nsh）是为**由 AI 生成、动态下发的原生界面**而设计的。模型用 TSX 写出一个小模块，已经发布的原生 iOS 或 Android 应用在运行时加载它，用平台自己的视图显示，并运行在应用掌控的沙箱里。同一套机制也让原生应用整体具备扩展能力——插件、功能、整个页面都可以在发布之后下发。手机客户端是首要目标；桌面客户端随后跟上，通过正在开发中的 GPUI 后端实现。

> [!IMPORTANT]
> nsh 不是用 TypeScript 编写整个应用的框架。应用仍然是原生应用，照原来的方式构建和发布；nsh 是嵌在其中的一层，负责那些在发布之后才生成、修改或新增的界面。

## 为什么做 nsh

**首要目标：由 AI 生成的界面。** 应用里的助手可以用一个界面而不是一段文字来回答。模型写出一个小的 TSX 模块（一个表单、一张图表、一张预订卡片），服务器检查、构建并签名后，应用下载并在 `NshAppView` 中显示：是原生界面，运行在沙箱里，只能做你授予的事，而且不需要更新应用。nsh 围绕这个场景设计，整体架构都是 AI 友好的：

- **模型擅长的格式。** 一个模块就是一个 TSX 文件，用到信号、少量宿主元素和 Tailwind 类——都是模型在 Web 上早已熟悉的词汇，不需要构建配置，也无需 NPM。
- **精确的、自动生成的契约。** 每个元素、属性、样式键和模块都有从 Rust 源码生成的 TypeScript 声明（`.nsh/types`），生成的代码可以做类型检查，也可以把准确的 API 交给模型，而不是让它去猜。本文档也以纯文本形式提供给模型（[`llms.txt`](/llms.txt)、[`llms-full.txt`](/llms-full.txt)）。
- **下发之前就能验证。** `nsh trace` 可以无界面、不需要设备地运行模块，报告错误和它生成的视图；构建错误附带代码定位。生成流程可以编译、运行、读取结果再重试。编译器也能作为库在进程内运行（`nsh_bundler::compile_module`），甚至在浏览器里运行（[在线试用](/zh-CN/playground)）。
- **不可信也能安全运行。** 生成的代码就是不可信代码。每个模块运行在独立的 QuickJS 虚拟机里，受内存、调用时长和节点数限制，只能使用宿主授予的能力，并以签名包的形式到达、先经应用校验。
- **原生，却无需写原生代码。** 产出的是真正的 UIKit 和 Android 视图，拥有系统的文本、输入和无障碍能力，但模型从不需要写 Swift、Kotlin 或 Java，应用也不需要发新版本。

<div class="diagram-scroll">
<img class="diagram diagram-light" src="/diagrams/ai-flow-light.svg" alt="从提示词到原生界面：模型写出 TSX 模块；服务器构建、检查并签名；跨过信任边界后，应用校验包，在设定了限制和能力的沙箱里运行，并显示 UIKit 或 Android 视图。">
<img class="diagram diagram-dark" src="/diagrams/ai-flow-dark.svg" alt="从提示词到原生界面：模型写出 TSX 模块；服务器构建、检查并签名；跨过信任边界后，应用校验包，在设定了限制和能力的沙箱里运行，并显示 UIKit 或 Android 视图。">
</div>

**其次：扩展原生应用，动态下发界面。** 同一套机制适用于任何模块，不只是生成的模块。应用在需要显示模块的地方嵌入一个 `NshAppView`（iOS）或 `NshView`（Android），nsh 在其中运行模块：

- **发布之后再下发。** 模块是应用在运行时下载的签名包（`.nshpkg`）。应用在运行它之前，会用自己信任的密钥校验签名，并检查吊销列表（[签名包](./signed-packages.md)）。改动无需发新版本，也无需重新提交应用商店审核。
- **宿主始终掌控。** 每个模块运行在独立的沙箱虚拟机中，受内存、时间和节点数限制，只能访问宿主授予的能力：网络主机、网页、状态栏（[沙箱与能力](./sandbox-and-capabilities.md)）。多个模块可以同时运行，没有全局单例。
- **与应用其余部分浑然一体。** 模块渲染真正的 UIKit 和 Android 视图，文本、输入、无障碍和滚动都是系统自己的。宿主还可以注册自己的原生视图——地图、播放器、设计系统组件——供模块使用（[原生组件](./native-components.md)）。

**再次：独立应用。** 同样的模块也可以组成一个完整的应用。`nsh package` 用预编译的运行时把它们打成可安装的 iOS 或 Android 应用，这也是试用 nsh 最快的方式：[快速上手](./getting-started.md)就是这样做的。这是受支持的用法，但不是 nsh 设计的出发点。

## 为什么在 GPUI Shell 之外还要做 Native Shell

[长桥（Longbridge）](https://longbridge.com)把 nsh 作为手机客户端的插件层来建设，桌面客户端也需要同样的一层。[GPUI Shell](https://gpui-kit.com/shell) 让 [GPUI](https://gpui-kit.com) 桌面应用可以用 JavaScript 扩展，最初的想法也是把它用到手机上。但它并不适合，因为它的模块由 GPUI 自己在 GPU 上绘制，而不是使用平台的视图：

- **系统能力都得重新实现。** 嵌在原生页面里的自绘界面，没有平台的文本选区，也没有选区弹出的菜单（拷贝、分享、翻译、查询、全选），更没有系统输入法、无障碍，以及每一代 iOS 带来的新系统视觉效果。在 GPU 渲染器里复刻这些是一项永远做不完的大工程，而且复刻品总会和真正的系统行为走样。
- **手机团队得接手一套新的 UI 技术栈。** 要在现有手机客户端里大规模运营一层自绘界面，iOS 和 Android 团队就得接手一个陌生的渲染器，以及它在各种系统版本和设备上的兼容性问题和缺陷。

因此 nsh 用 UIKit 和 Android 视图来渲染模块：模块和应用其余部分用的是同一套视图，表现自然一致：同一种模块格式、同一个运行时，现在运行在 UIKit 和 Android 视图上，GPUI 后端完成后也运行在桌面客户端上——一个模块写一次，就能在三端的客户端里运行。

两者的定位不同：GPUI Shell 用于扩展 GPUI 桌面应用；Native Shell 用于扩展原生手机应用，并通过 GPUI 后端覆盖桌面客户端。nsh 也有自己的桌面后端：[macOS](./macos.md) 上用 AppKit，[Linux](./linux.md) 上用 GTK 4，[Windows](./windows.md) 上用 Direct2D 绘制的 Win32 窗口。

## 一个模块

模块用 TypeScript 和 JSX 编写：

```tsx
import { mount, signal, Text, View } from "nsh";

function Counter() {
  const count = signal(0);
  return (
    <View style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 12 }}>
      <Text style={{ fontSize: 48, fontWeight: "700" }}>{count()}</Text>
      <View
        onPress={() => count.update((n) => n + 1)}
        activeOpacity={0.6}
        class="rounded-lg bg-blue-600 px-5 py-3"
      >
        <Text class="text-white text-lg">Tap</Text>
      </View>
    </View>
  );
}

mount(Counter);
```

`<View>` 会变成 `UIView` 或 Android 的 `ViewGroup`，`<Text>` 变成原生文本视图，按钮的按压反馈（`activeOpacity`）在主线程上完成，不需要往返 JavaScript。点击时，`count.update` 只修改一个信号，只有读取 `count()` 的那段文本会更新：组件函数不会重新执行。

宿主应用只需几行原生代码就能显示它（[iOS](./embedding-ios.md)、[Android](./embedding-android.md)）：

```swift
let app = NshAppView()                         // 一个 UIView：可以放在任何地方
try app.grantCapabilities(#"{ "network": { "hosts": ["api.example.com"] } }"#)
let info = try await app.loadPackage(at: downloadedFile, trust: trust)   // 先校验，再运行
```

只有模块自己的代码是 JavaScript。它调用的一切——响应式图、渲染器、布局、文本，以及它导入的每个模块——都是用 Rust 实现的原生代码。

## 模块如何运行

一个模块（在 CLI 里称为「应用」）是一组带入口的 `.tsx` / `.ts` 文件，入口通常是 `App.tsx`。工具链只有一个 `nsh` 可执行文件：

1. **构建。** `nsh` 打包模块的各个文件（借助 rolldown 和 Oxc），并用自己的编译器编译 JSX。对 `nsh` 与 `nsh/*` 的导入保留为导入：运行时以原生方式提供它们，因此 bundle 里只有你的代码。bundle 会被编译为 QuickJS 字节码。
2. **运行。** 在设备上，引擎线程用 [QuickJS](https://github.com/quickjs-ng/quickjs) 执行字节码。编译后的 JSX 直接调用原生渲染器；一次 JavaScript 执行轮次中的变更被收集起来，由 [Taffy](https://github.com/DioxusLabs/taffy)（flexbox 与 CSS grid）完成布局，再以每帧一批视图操作的形式交给主线程。
3. **渲染。** 主线程把这批操作应用到原生视图上。文本在引擎线程上测量和排版一次（Apple 平台用 CoreText，Android 用 `StaticLayout`），主线程只负责绘制。

由于 JavaScript 只描述界面、处理事件，解释器里的工作量很小。解析样式、解析 Tailwind 类名、差异比较、布局和文本处理都在 Rust 中完成。

> [!NOTE]
> 主线程从不等待 JavaScript。系统同步提出的问题（这个输入框还能再接受一个字符吗、用户可以侧滑返回吗、这个面板可以关闭吗）由原生层根据你事先设置的 props 直接回答，例如 `maxLength`、`preventBack` 或 `dismissible`。事件随后才会到达你的代码。

## 你能得到什么

- **嵌入。** `NshAppView`（Swift）和 `NshView`（Java/Kotlin）可以把模块放进原生页面的任意位置，数量不限（[iOS](./embedding-ios.md)、[Android](./embedding-android.md)）。
- **签名、可吊销的下发。** Ed25519 签名的包，发布密钥由离线根密钥认证，支持吊销列表（[签名包](./signed-packages.md)）。
- **由宿主配置的沙箱。** 每个视图单独设置虚拟机限制和授予的能力（[沙箱与能力](./sandbox-and-capabilities.md)）。
- **模块可以使用宿主的视图。** 用 Swift、Java 或 Rust 注册原生组件（[原生组件](./native-components.md)）。
- **原生视图。** 嵌入原生地图、WebView 或视频播放器没有层级问题，无障碍使用平台自己的树，文本输入是系统的输入。
- **细粒度响应式。** 与 Solid 类似的[信号](./reactivity.md)，在原生层实现。组件只运行一次，只有读取了变化信号的表达式会重新执行。
- **样式与 Tailwind**，在原生层解析：flexbox、CSS grid、阴影、渐变、滤镜、过渡（[样式](./styling.md)、[Tailwind](./tailwind.md)）。
- **移动端 UI 组件集**（`nsh/ui/mobile`），组件用 Rust 编写，在 iOS 和 Android 上外观一致：[Button](../components/button.md)、[Card](../components/card.md)、[ListView](../components/list-view.md)、[导航](../components/navigation.md) 等。
- **单一可执行文件的工具链。** 不需要 Node.js，也无需 NPM；`nsh` 负责构建模块、生成类型声明、以热模块替换的方式提供开发服务、打包和签名。

## 运行平台

| 平台 | 状态 | 说明 |
|---|---|---|
| iOS | 已支持（iOS 15.0+） | 真机和模拟器；通过 `NativeShell.xcframework` 嵌入 |
| Android | 已支持（API 28，Android 9+） | `arm64-v8a` 和 `x86_64`；通过 `nsh.aar` 嵌入 |
| Mac Catalyst | 仅用于开发 | `nsh run` 在 Mac 上打开的 iPhone 尺寸窗口 |
| Web（WebAssembly） | 仅用于演示 | 运行本站的示例，用 DOM 而不是原生视图绘制（[Web](./web.md)）；不是可发布的目标平台 |
| 桌面（GPUI） | 开发中 | [GPUI](https://gpui-kit.com) 后端，届时可以用同样的模块和运行时开发专门的桌面客户端 |

<div class="diagram-scroll">
<img class="diagram diagram-light" src="/diagrams/platforms-light.svg" alt="一个模块，多种宿主：同一个 bundle 和 Rust 运行时，运行在 iOS（UIKit，已支持）、Android（Android 视图，已支持）、桌面（GPUI，开发中）和 Web（WebAssembly 渲染到 DOM，仅用于本站演示）。">
<img class="diagram diagram-dark" src="/diagrams/platforms-dark.svg" alt="一个模块，多种宿主：同一个 bundle 和 Rust 运行时，运行在 iOS（UIKit，已支持）、Android（Android 视图，已支持）、桌面（GPUI，开发中）和 Web（WebAssembly 渲染到 DOM，仅用于本站演示）。">
</div>

## 与其他方案的比较

在手机客户端里显示发布之后才到来的界面，常见做法有 WebView、用 JSON 描述的服务端驱动 UI，以及小程序容器。WebView 渲染的是网页，nsh 模块渲染的是原生视图；服务端驱动的协议只能重新组合内置组件，nsh 模块有自己的状态和逻辑；而且它运行在宿主用代码配置的沙箱里。[方案对比](./comparison.md)逐一做了比较。

## 下一步

- [在 iOS 应用中嵌入](./embedding-ios.md)和[在 Android 应用中嵌入](./embedding-android.md)：在你的应用里承载模块。
- [签名包](./signed-packages.md)和[沙箱与能力](./sandbox-and-capabilities.md)：下发模块，并决定它们能做什么。
- [安装](./installation.md)和[快速上手](./getting-started.md)：安装 `nsh`，编写第一个模块（以独立应用运行）。
- [JSX 与组件](./jsx-and-components.md)和[响应式](./reactivity.md)：其余一切都建立在这两个概念之上。
