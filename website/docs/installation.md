---
title: Installation
description: Get the nsh command, from a prebuilt distribution or a source checkout, and the Xcode and Android tools each target needs.
order: 2
---

# Installation

Everything nsh does goes through one command, `nsh`. It bundles and compiles apps, runs the dev server, launches development hosts and packages installable apps. You can use it from a prebuilt distribution, which needs no Rust, or build it from a checkout of the repository.

On top of `nsh`, each target needs the platform's own tools: Xcode for iOS and Mac Catalyst, the Android SDK for Android.

## The prebuilt distribution

The distribution is a directory, `dist/nsh`, with the CLI and the prebuilt native runtimes it packages apps with:

```sh
dist/nsh/bin/nsh                                             # the CLI
dist/nsh/runtime/ios/iphoneos/nsh-ios-app                    # runs packaged apps on iOS devices
dist/nsh/runtime/ios/iphonesimulator/{nsh-ios-app,nsh-ios-host}
dist/nsh/runtime/ios/maccatalyst/nsh-ios-host                # the Mac Catalyst development host
dist/nsh/runtime/android/{release,dev}/                      # classes.dex, lib/<abi>/libnsh.so
dist/nsh/embed/ios/{NativeShell,NativeShellDev}.xcframework  # embedding in iOS apps
dist/nsh/embed/android/{nsh,nsh-dev}.aar                     # embedding in Android apps
```

Put `dist/nsh/bin` on your `PATH` (or call `dist/nsh/bin/nsh` directly) and keep the directory together: `nsh` finds the runtimes next to itself, in `bin/../runtime`.

```sh
export PATH="$PWD/dist/nsh/bin:$PATH"
nsh --version
```

The distribution is made with `cargo xtask dist` in a checkout (below). It builds the iOS runtimes, so it has to run on a Mac with Xcode, and the `nsh` binary it contains is built for that machine (Apple Silicon).

## Building from a checkout

You need Rust through [rustup](https://rustup.rs). The repository's `rust-toolchain.toml` selects the stable toolchain and installs the iOS and Android targets on first use.

```sh
git clone https://github.com/longbridge/native-shell.git
cd native-shell
cargo build -p nsh-cli          # ./target/debug/nsh
```

Then build the development host of each target you want to run on, once, and again after you update the checkout:

```sh
cargo build -p nsh-ios-host --target aarch64-apple-ios-macabi   # nsh run (Mac Catalyst window)
cargo build -p nsh-ios-host --target aarch64-apple-ios-sim      # nsh run --target ios
cargo xtask android --dev                                       # nsh run --target android
```

For `nsh package`, build the release runners as well:

```sh
cargo build -p nsh-ios-app --target aarch64-apple-ios-sim       # nsh package --target ios-simulator
cargo build -p nsh-ios-app --target aarch64-apple-ios           # nsh package --target ios
cargo xtask android                                             # nsh package --target android
```

A CLI built from a checkout looks for the runtimes in `target/<triple>/<profile>` and `target/android/<profile>[-dev]`, using its own profile: `./target/debug/nsh` uses debug runtimes, `./target/release/nsh` release ones. Build both with the same profile (add `--release` to all of them for faster apps).

> [!TIP]
> `NSH_RUNTIME=/path/to/runtime` makes `nsh` use a runtime directory laid out like the distribution's `runtime/`, wherever the CLI itself lives.

## Platform tools

### iOS and Mac Catalyst

- **Xcode** (with its command line tools). `nsh run` uses `xcrun simctl` to boot simulators and install the development host; `nsh package` uses `actool` for app icons and `codesign` for signing.
- An iOS simulator runtime. If none is installed, install one with `xcodebuild -downloadPlatform iOS`.
- Apps target iOS 15.0 and later.

### Android

- **The Android SDK**, found through `ANDROID_HOME`, `ANDROID_SDK_ROOT` or `~/Library/Android/sdk`, with:
  - `platforms;android-35` and a recent `build-tools` (nsh uses `aapt2`, `zipalign` and `apksigner` from it, no Gradle);
  - `platform-tools` (`adb`).
- **A JDK** (`keytool` creates the debug keystore in `~/.android/debug.keystore` the first time you package, `apksigner` needs Java).
- A running emulator or a device connected over `adb`. Apps need Android 9 (API 28) or later.

```sh
sdkmanager "platforms;android-35" "build-tools;35.0.1" "platform-tools"
```

Building the Android runtime from a checkout (`cargo xtask android`) additionally needs an **NDK** from the SDK (`ndk;<version>`; set `ANDROID_NDK_HOME` if the newest one installed is incomplete) and `javac`.

## What works where

| Command | macOS | Linux |
|---|---|---|
| `nsh init`, `nsh build`, `nsh trace`, `nsh dev` | Yes | Yes |
| `nsh run` (Mac Catalyst, the default) | Yes | No |
| `nsh run --target ios`, `nsh package --target ios` / `ios-simulator` | Yes, with Xcode | No |
| `nsh run --target android`, `nsh package --target android` | Yes | Yes, from a checkout |
| `nsh keygen`, `certify`, `publish`, `verify`, `revoke` | Yes | Yes |

On Linux, build the CLI and the Android runtime from a checkout (`cargo build -p nsh-cli` and `cargo xtask android --dev`); the NDK's Linux toolchain is used automatically. Building the Android runtime supports macOS and Linux hosts only.

## Editor setup

nsh needs no NPM. `nsh init` writes a `tsconfig.json` and the type declarations of everything nsh provides into `.nsh/types`, and every command that builds the app keeps them up to date. Any editor that uses the TypeScript language server (VS Code, Zed, Neovim, WebStorm) type-checks the app with them. See [Project structure](./project-structure.md#tsconfigjson-and-nshtypes).

## Next steps

Continue with [Getting started](./getting-started.md) to create an app and run it.
