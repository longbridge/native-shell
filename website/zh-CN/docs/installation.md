---
title: 安装
description: 从预构建的发行包或源码仓库获取 nsh 命令，以及各目标平台所需的 Xcode、Android 和 GTK 工具。
order: 2
---

# 安装

nsh 的所有操作都通过一个命令 `nsh` 完成：打包和编译应用、运行开发服务器、启动开发宿主、打包可安装的应用。你可以使用无需 Rust 的预构建发行包，也可以从仓库源码构建。

除了 `nsh` 本身，每个目标平台还需要平台自己的工具：iOS 和 Mac Catalyst 需要 Xcode，macOS 需要 Xcode 命令行工具，Android 需要 Android SDK，Linux 需要 GTK 4。

## 预构建发行包

发行包是一个目录 `dist/nsh`，包含 CLI 以及它打包应用时使用的预构建原生运行时：

```sh
dist/nsh/bin/nsh                                             # the CLI
dist/nsh/runtime/ios/iphoneos/nsh-ios-app                    # runs packaged apps on iOS devices
dist/nsh/runtime/ios/iphonesimulator/{nsh-ios-app,nsh-ios-host}
dist/nsh/runtime/ios/maccatalyst/nsh-ios-host                # the Mac Catalyst development host
dist/nsh/runtime/android/{release,dev}/                      # classes.dex, lib/<abi>/libnsh.so
dist/nsh/runtime/macos/{nsh-macos-host,nsh-macos-app}        # the macOS (AppKit) development host and release runner
dist/nsh/runtime/linux/{nsh-linux-host,nsh-linux-app}        # the Linux development host and release runner
dist/nsh/embed/ios/{NativeShell,NativeShellDev}.xcframework  # embedding in iOS apps
dist/nsh/embed/android/{nsh,nsh-dev}.aar                     # embedding in Android apps
```

把 `dist/nsh/bin` 加入 `PATH`（或直接调用 `dist/nsh/bin/nsh`），并保持目录完整：`nsh` 会在自身旁边的 `bin/../runtime` 中查找运行时。

```sh
export PATH="$PWD/dist/nsh/bin:$PATH"
nsh --version
```

发行包由源码仓库中的 `cargo xtask dist` 生成（见下文）。其中的 `nsh` 可执行文件是为该机器构建的。在装有 Xcode 的 Mac 上，它包含 iOS 和 macOS 运行时（Apple Silicon）；在 Linux 上，它改为包含为该机器构建的 Linux 运行时。两者都包含 Android 运行时。

## 从源码构建

你需要通过 [rustup](https://rustup.rs) 安装 Rust。仓库中的 `rust-toolchain.toml` 会选择 stable 工具链，并在首次使用时安装 iOS 和 Android 目标。

```sh
git clone https://github.com/longbridge/native-shell.git
cd native-shell
cargo build -p nsh-cli          # ./target/debug/nsh
```

然后为每个要运行的目标构建开发宿主，构建一次即可，更新源码后再重新构建：

```sh
cargo build -p nsh-ios-host --target aarch64-apple-ios-macabi   # nsh run (Mac Catalyst window)
cargo build -p nsh-ios-host --target aarch64-apple-ios-sim      # nsh run --target ios
cargo xtask android --dev                                       # nsh run --target android
cargo build -p nsh-macos-host                                   # nsh run --target macos (built for you in a checkout)
cargo build -p nsh-linux-host                                   # nsh run on Linux (built for you in a checkout)
```

如果要使用 `nsh package`，还需要构建发布版运行器：

```sh
cargo build -p nsh-ios-app --target aarch64-apple-ios-sim       # nsh package --target ios-simulator
cargo build -p nsh-ios-app --target aarch64-apple-ios           # nsh package --target ios
cargo xtask android                                             # nsh package --target android
cargo build -p nsh-macos-app                                    # nsh package --target macos
cargo build -p nsh-linux-app                                    # nsh package --target linux
```

从源码构建的 CLI 会按自身的 profile 在 `target/<triple>/<profile>`（macOS 和 Linux 运行时为本机构建，位于 `target/<profile>`）和 `target/android/<profile>[-dev]` 中查找运行时：`./target/debug/nsh` 使用 debug 运行时，`./target/release/nsh` 使用 release 运行时。请用相同的 profile 构建两者（若希望应用更快，给所有命令都加上 `--release`）。

> [!TIP]
> 设置 `NSH_RUNTIME=/path/to/runtime` 后，无论 CLI 位于何处，`nsh` 都会使用这个目录作为运行时，其结构与发行包的 `runtime/` 相同。

## 平台工具

### iOS 与 Mac Catalyst

- **Xcode**（及其命令行工具）。`nsh run` 用 `xcrun simctl` 启动模拟器并安装开发宿主；`nsh package` 用 `actool` 处理应用图标，用 `codesign` 签名。
- 一个 iOS 模拟器运行时。如果还没有安装，可以用 `xcodebuild -downloadPlatform iOS` 安装。
- 应用支持 iOS 15.0 及以上版本。

### Android

- **Android SDK**，通过 `ANDROID_HOME`、`ANDROID_SDK_ROOT` 或 `~/Library/Android/sdk` 查找，需包含：
  - `platforms;android-35` 和较新的 `build-tools`（nsh 使用其中的 `aapt2`、`zipalign` 和 `apksigner`，不使用 Gradle）；
  - `platform-tools`（`adb`）。
- **JDK**（首次打包时 `keytool` 会在 `~/.android/debug.keystore` 创建调试密钥库，`apksigner` 也需要 Java）。
- 一个正在运行的模拟器，或通过 `adb` 连接的设备。应用要求 Android 9（API 28）及以上。

```sh
sdkmanager "platforms;android-35" "build-tools;35.0.1" "platform-tools"
```

### macOS

- **Xcode 命令行工具**（`xcode-select --install`）：构建运行时需要其中的 SDK 和 clang，打包应用需要 `codesign` 和 `sips`。
- 应用要求 macOS 14 及以上。

见 [macOS](./macos.md)。

### Linux

- **GTK 4.14** 及以上；构建运行时还需要它们的开发包和 clang：`libgtk-4-dev clang`（Debian、Ubuntu 24.04 及以上）、`gtk4 clang`（Arch）、`gtk4-devel clang`（Fedora）。
- 应用要播放的视频格式对应的 **GStreamer** 插件。

见 [Linux](./linux.md)。

### 构建 Android 运行时

从源码构建 Android 运行时（`cargo xtask android`）还需要 SDK 中的 **NDK**（`ndk;<version>`；如果已安装的最新版本不完整，请设置 `ANDROID_NDK_HOME`）以及 `javac`。

## 各平台可用的命令

| 命令 | macOS | Linux |
|---|---|---|
| `nsh init`, `nsh build`, `nsh trace`, `nsh dev` | 可以 | 可以 |
| `nsh run`（Mac Catalyst，macOS 上的默认目标） | 可以 | 不可以 |
| `nsh run --target macos`, `nsh package --target macos` | 可以 | 不可以 |
| `nsh run --target linux`（Linux 上的默认目标）, `nsh package --target linux` | 不可以 | 可以 |
| `nsh run --target ios`, `nsh package --target ios` / `ios-simulator` | 可以，需要 Xcode | 不可以 |
| `nsh run --target android`, `nsh package --target android` | 可以 | 可以，需从源码构建 |
| `nsh keygen`, `certify`, `publish`, `verify`, `revoke` | 可以 | 可以 |

在 Linux 上，请从源码构建 CLI 和 Android 运行时（`cargo build -p nsh-cli` 与 `cargo xtask android --dev`）；会自动使用 NDK 的 Linux 工具链。构建 Android 运行时只支持 macOS 和 Linux 主机。

## 编辑器配置

nsh 无需 NPM。`nsh init` 会写入 `tsconfig.json`，并把 nsh 提供的所有内容的类型声明写入 `.nsh/types`，之后每个构建应用的命令都会让它们保持最新。任何使用 TypeScript 语言服务的编辑器（VS Code、Zed、Neovim、WebStorm）都能借此对应用做类型检查。参见 [项目结构](./project-structure.md)。

## 下一步

继续阅读 [快速开始](./getting-started.md)，创建并运行一个应用。
