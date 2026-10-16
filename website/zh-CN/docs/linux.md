---
title: Linux
description: 在 Linux 上用 GTK 4 窗口运行 nsh 应用、打包应用、用 AppView 嵌入 GTK 应用，以及 Linux 后端不支持的功能。
order: 3.25
---

# Linux

nsh 应用也能在 Linux 上运行。Linux 后端用 GTK 4 控件渲染应用，用 Pango 排版文字；核心、响应式图、渲染器、Taffy 布局和
`nsh/ui/mobile` 组件与 iOS、Android 上完全相同，因此同一个 bundle 可以在所有平台上运行。在 Linux 上，`Platform.OS` 为
`"linux"`，并会应用 `desktop={{…}}` 属性。

## 环境要求

- **GTK 4.14** 及以上。构建 nsh 的 Linux 运行时（从源码构建，或在 Linux 上运行 `cargo xtask dist`）
  还需要它们的开发包和 clang：Debian 与 Ubuntu（24.04 及以上）为 `libgtk-4-dev clang`，Arch 为 `gtk4 clang`，Fedora 为
  `gtk4-devel clang`。
- 应用要播放的视频格式对应的 **GStreamer** 插件（GTK 通过 GStreamer 播放视频）。

## 运行应用

在 Linux 上，`nsh run` 会在一个 GTK 窗口（默认 390×844）中打开应用，和其他目标一样支持热重载、在终端显示日志并提供错误浮层：

```sh
nsh run my-app                 # 在 Linux 上默认就是 --target linux
```

在源码仓库中，`nsh run` 会先构建开发宿主（`nsh-linux-host`）；在 Linux 上制作的发行版把它放在 `runtime/linux/` 中。
Ctrl-C 会关闭窗口，关闭窗口也会结束 `nsh run`。宿主也可以单独运行一个构建好的 bundle：

```sh
nsh build my-app --release -o build
dist/nsh/runtime/linux/nsh-linux-host --bundle build/bundle.qbc --size 1024x768   # 源码仓库中：target/debug/nsh-linux-host
```

`--size` 是窗口的默认尺寸；平铺式窗口管理器可能会给窗口另一个尺寸。

## 打包

```sh
nsh package my-app --target linux [--run]
```

会写出一个目录，以及同样内容的压缩包：

```text
out/<Name>/
  <Name>                 可执行文件（nsh 的发布版运行器）
  nsh/bundle.qbc         编译后的应用
  nsh/assets/…           应用导入的资源
  nsh/app.json           窗口标题和应用 id
  <id>.desktop           桌面条目（StartupWMClass 为应用 id）
  <id>.png               图标（nsh.json 指定了图标时）
out/<Name>.tar.gz
```

可执行文件会在自身旁边找到 `nsh/`（指向它的符号链接也可以，例如放在 `~/.local/bin` 中）。运行应用的机器上必须装有 GTK 4
和 Pango。要让应用出现在桌面的启动器中，把可执行文件（或其链接）放到 `PATH` 中，把 `.desktop` 文件放到
`~/.local/share/applications`，把图标放到图标主题目录中。`--run` 会在打包后启动应用。

## 嵌入 GTK 应用

`nsh-platform-linux` crate（位于 nsh 仓库中）为 Rust 编写的 GTK 应用提供 `AppView`：一个作为 GTK 控件的 nsh 引擎。每个
`AppView` 相互独立，可以同时运行多个。

```rust
use std::rc::Rc;
use nsh_platform_linux::{gtk4 as gtk, AppOptions, AppView, AssetBase, Bundle};
use gtk::prelude::*;

fn main() -> gtk::glib::ExitCode {
    let app = gtk::Application::builder().application_id("com.example.host").build();
    app.connect_activate(|app| {
        let window = gtk::ApplicationWindow::builder().application(app).default_width(390).default_height(844).build();
        let view = AppView::new(AppOptions {
            asset_base: Some(AssetBase::Directory("app/assets".into())),
            ..AppOptions::default()
        });
        // 在加载之前注册原生组件
        let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
        view.load(Bundle::source(code).into());
        window.set_child(Some(&view.widget()));
        // 窗口持有视图；关闭窗口即停止应用。
        let keep = std::cell::RefCell::new(Some(Rc::clone(&view)));
        window.connect_destroy(move |_| if let Some(view) = keep.take() { view.stop() });
        window.present();
    });
    app.run()
}
```

`AppView::load` 会替换正在运行的内容；`stop` 结束应用。`asset_base` 指定 `asset:/` URI 从哪里解析（`AssetBase::Directory`
或 `AssetBase::Http`）。字节码 bundle（`bundle.qbc`）用 `unsafe { Bundle::bytecode(bytes) }` 加载：QuickJS 字节码是受信任的输入，
只能从你自己应用的文件中加载。

原生组件通过 `AppView::register_component` 注册，见[原生组件](./native-components.md#在-linux-上注册rust)。

## Linux 上的差异

运行环境是桌面环境：安全区和键盘高度为 0，`Platform.constants.interfaceIdiom` 为 `"desktop"`，`Platform.Version` 为内核的主、次版本号，
深色模式实时跟随桌面的设置（通过 XDG 设置门户）。见[设备与环境](./device-and-environment.md)。

不支持：

- **`WebView`**：它需要 WebKitGTK；Web 视图只显示一个中性的占位视图。
- **视频的 `rate`**：GTK 只能以正常速度播放。`onEnd` 只对不循环的视频触发。
- **可缩放的滚动视图**（`zoomTo`、缩放相关属性）。
- **`mixBlendMode` 和 `backdropFilter`**（背景色仍会绘制）；`borderCurve: "continuous"` 绘制为圆形圆角。
- **可选择的文本**、**输入辅助视图**和**状态栏**（桌面上没有对应物）。
- **无障碍**：`accessible` 分组、自定义 `accessibilityActions`；实时区域改为播报。

## 另见

- [CLI](./cli.md)：`nsh run --target linux`、`nsh package --target linux`
- [打包](./packaging.md)
- [平台特定代码](./platform-specific.md)
