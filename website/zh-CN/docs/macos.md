---
title: macOS
description: 在 macOS 上用 AppKit 窗口运行 nsh 应用、打包为 .app、用 AppView 嵌入 Mac 应用，以及 macOS 后端不支持的功能。
order: 3.24
---

# macOS

nsh 应用也能作为原生 Mac 应用运行。macOS 后端用 AppKit 视图和 Core Animation 渲染应用，用 CoreText 排版文字；核心、响应式图、
渲染器、Taffy 布局和 `nsh/ui/mobile` 组件与 iOS、Android 上完全相同，因此同一个 bundle 可以在所有平台上运行。在 macOS 上，
`Platform.OS` 为 `"macos"`，并会应用 `desktop={{…}}` 属性。

它不是 [Mac Catalyst](./getting-started.md#mac-catalystmacos-上默认)。Mac Catalyst 仍是 macOS 上 `nsh run` 的默认目标：
它在 iPhone 尺寸的窗口中运行 iOS 后端（UIKit），行为与手机一致。macOS 后端的行为则是桌面应用：鼠标、触控板和键盘输入，
可调整大小的窗口，居中的表单和菜单栏。

## 环境要求

- 运行应用的机器需要 **macOS 14** 及以上。
- Xcode 命令行工具（`xcode-select --install`），用于从源码构建 nsh 的 macOS 运行时以及打包应用（`codesign`、`sips`）。
  在 Mac 上用 `cargo xtask dist` 制作的发行版把运行时放在 `runtime/macos/` 中。

## 运行应用

```sh
nsh run my-app --target macos
```

应用会在一个 AppKit 窗口（默认 1200×800）中打开，和其他目标一样支持热重载、在终端显示日志并提供错误浮层；Cmd+R 重新加载最新构建。
在源码仓库中，`nsh run` 会先构建开发宿主（`nsh-macos-host`）。停止 `nsh run` 会关闭窗口，关闭窗口也会结束 `nsh run`。
宿主也可以单独运行一个构建好的 bundle：

```sh
nsh build my-app --release -o build
dist/nsh/runtime/macos/nsh-macos-host --bundle build/bundle.qbc --size 1200x800   # 源码仓库中：target/debug/nsh-macos-host
```

`--size` 是窗口的初始尺寸；窗口可以调整大小。

## 打包

```sh
nsh package my-app --target macos [--run]
```

会写出一个应用包：

```text
out/<Name>.app/Contents/
  Info.plist
  MacOS/<Name>               可执行文件（nsh 的发布版运行器）
  Resources/nsh/bundle.qbc   编译后的应用
  Resources/nsh/assets/…     应用导入的资源
  Resources/nsh/app.json     窗口标题和应用的外观
  Resources/AppIcon.icns     图标（nsh.json 指定了图标时，用 sips 转换）
```

`.app` 使用 ad hoc 签名，因此可以在构建它的 Mac 上运行。要分发它，请用 Developer ID 签名并进行公证。`--run` 会在打包后打开应用。

## 嵌入 Mac 应用

`nsh-platform-macos` crate（位于 nsh 仓库中）为 Rust 编写的 Mac 应用提供 `AppView`：一个作为 `NSView` 的 nsh 引擎。每个
`AppView` 相互独立，可以同时运行多个。

```rust
use std::rc::Rc;
use nsh_platform_macos::{
    objc2::MainThreadMarker, objc2_app_kit::NSWindow, AppOptions, AppView, AssetBase, Bundle,
};

fn show(window: &NSWindow) -> Rc<AppView> {
    let mtm = MainThreadMarker::new().expect("main thread");
    let view = AppView::new(mtm, AppOptions {
        asset_base: Some(AssetBase::Directory("app/assets".into())),
        ..AppOptions::default()
    });
    // 在加载之前注册原生组件
    let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
    view.load(Bundle::source(code).into());
    window.setContentView(Some(&view.view()));
    // 窗口显示视图期间保留它；窗口关闭时调用 `stop`。
    view
}
```

`AppView::load` 会替换正在运行的内容；`stop` 结束应用。`asset_base` 指定 `asset:/` URI 从哪里解析（`AssetBase::Directory`
或 `AssetBase::Http`）。字节码 bundle（`bundle.qbc`）用 `unsafe { Bundle::bytecode(bytes) }` 加载：QuickJS 字节码是受信任的输入，
只能从你自己应用的文件中加载。如果应用只是一个 nsh 应用，`run_app(source, options)` 会在窗口中打开一个 `AppView`，带有标准菜单
（隐藏、退出、编辑、最小化、关闭），并运行到窗口关闭为止。

原生组件通过 `AppView::register_component` 注册，见[原生组件](./native-components.md#在-macos-上注册rust)。

## macOS 上的差异

运行环境是桌面环境：安全区和键盘高度为 0，`Platform.constants.interfaceIdiom` 为 `"desktop"`，`Platform.Version` 为 macOS 的版本号，
深色模式实时跟随系统外观。见[设备与环境](./device-and-environment.md)。

输入是桌面输入：鼠标、触控板（带惯性的滚动、捏合和旋转）和键盘。`selectable` 文本通过拖动、双击和三击选择，用 Cmd+C 复制。
Escape、Option+Left、Cmd+[ 和鼠标的后退键会在栈中返回，或请求模态视图关闭；`pageSheet` 和 `formSheet` 是窗口中居中的卡片，背后的内容变暗。
`Alert` 以 `NSAlert` 表单的形式显示在窗口上。

不支持或有所不同：

- **可缩放的滚动视图**（`zoomTo`、缩放相关属性）。
- **`mixBlendMode`**，以及大于 1 的 `brightness()`。
- **输入框**：内容类型与自动填充、键盘类型、自动大写、字间距；输入辅助视图从不显示（没有软键盘），`keyboardShouldPersistTaps` 被忽略。
- **`WebView` 的 `scrollEnabled`** 被忽略（macOS 的 Web 视图没有可关闭的滚动视图）。
- **无障碍**：自定义 `accessibilityActions`。
- **状态栏**和**触感反馈**（桌面上没有对应物）。

## 另见

- [CLI](./cli.md)：`nsh run --target macos`、`nsh package --target macos`
- [打包](./packaging.md)
- [平台特定代码](./platform-specific.md)
