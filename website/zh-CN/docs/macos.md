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

### 性能监视器

`--fps`（`nsh run my-app --target macos --fps`、宿主的 `--fps`、打包后应用的 `open -a MyApp --args --fps`，嵌入时用
`AppOptions { fps: true, .. }` / `AppView::set_fps_monitor(true)`）会在应用右上角显示一个每秒刷新两次的面板：

| 行 | 测量内容 |
|---|---|
| `FPS` | 最近一秒显示的帧数：应用有变化的屏幕刷新次数（空闲的应用读数低，这不代表慢）。`max` 是主线程能跟上的帧率，之后是显示器的刷新率（ProMotion 屏幕为 120 Hz）。 |
| `TICK` | 屏幕刷新实际到达的间隔（均值、p95），以及 `gap`：显示帧之间间隔的 p95。没有迟到时为一次刷新的时长（120 Hz 下 8.3 ms）。 |
| `MAIN` | 每帧的主线程耗时（应用引擎的变更、绘制文本和画布、动画、输入）、它的 95 分位数，以及 `drop`：超过一次刷新时长的帧所占比例。 |
| `JS` / `CORE` | 每次 commit：你的代码（事件处理、定时器、列表行）和 nsh 核心（布局、文本、diff）的耗时；每秒 commit 数、每次 commit 的原生视图变更数。 |
| `HOP` | 变更在引擎线程和主线程之间等待的时间；引擎线程的繁忙程度。 |
| `CPU` / `GPU` / `MEM` | 进程：CPU（100% 为一个核心）、它占用的 GPU 比例、内存占用（活动监视器中的数值）。 |

下方的柱状图是最近 120 帧的主线程耗时，与一次刷新的时长（横线）对比：在一半以内为绿色，在其以内为琥珀色，超过为红色。
同时加上 `--trace` 时，宿主还会每秒把读数记录到日志一次（毫秒，依次为 mean/p50/p95/max）。请测量 release 构建；监视器关闭时没有任何开销。

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
点击 `Input` 的任意位置（包括内边距）都会把光标放到最近的位置。
`Alert` 和 `ActionSheet` 以 `NSAlert` 表单的形式显示在窗口上：不超过两个按钮的提示框按钮排成一行（取消按钮在最左边，最后一个普通按钮是
Return 的默认按钮）；更多按钮以及操作表的选项按顺序竖排，取消按钮在最后，没有默认按钮；Escape 按下取消按钮。

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
