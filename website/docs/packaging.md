---
title: Packaging
description: Build installable apps with nsh package for iOS devices, the iOS simulator and Android, with app metadata, icons and signing.
order: 29
---

# Packaging

`nsh package` turns an app into something you can install: a `.app` for the iOS simulator, a signed `.ipa` for iOS
devices, a signed `.apk` for Android, an ad hoc signed `.app` for macOS, or a directory and `.tar.gz` for Linux. It combines a release build of your app with the **prebuilt runtime** that
ships with nsh, so it needs no Rust toolchain, no Xcode project and no Gradle.

```sh
nsh package App.tsx --target ios-simulator --run     # build, install and launch in the simulator
nsh package App.tsx --target android --run           # build, install and launch on a device or emulator
nsh package App.tsx --target macos --run             # build and launch on this Mac
nsh package App.tsx --target linux --run             # build and launch on this machine
nsh package App.tsx --target ios --sign "Apple Development: Jane Doe (ABCDE12345)" --profile Dev.mobileprovision
```

The app is built in release mode (minified, compiled to QuickJS bytecode) exactly as `nsh build --release` would build
it. Output goes to `out/` unless you pass `--out`.

> [!NOTE]
> To put nsh *inside* an existing native app instead, see [Embedding in iOS apps](./embedding-ios.md) and
> [Embedding in Android apps](./embedding-android.md). To ship updates that apps download and verify, see
> [Signed packages](./signed-packages.md).

## App metadata: `nsh.json`

`nsh package` reads `nsh.json` next to the entry file. `nsh init` writes one:

```json
{
  "name": "Gallery",
  "id": "com.example.gallery",
  "version": "1.0.0",
  "build": 1,
  "icon": "icon.png"
}
```

| Key | Type | Default | Used for |
|---|---|---|---|
| `name` | string | the entry's directory name | The display name under the icon (`CFBundleDisplayName`, Android `android:label`) and the output file name. |
| `id` | string | `com.example.<name>` | The bundle identifier / Android package name. |
| `version` | string | `"1.0.0"` | The user-visible version (`CFBundleShortVersionString`, `versionName`). |
| `build` | number | `1` | The build number (`CFBundleVersion`, `versionCode`). Raise it for every release you upload. |
| `icon` | string | none | A square PNG, ideally 1024×1024, relative to `nsh.json`. |

`nsh.json` may also hold `tailwind` (see [Tailwind](./tailwind.md)) and `capabilities` (see
[Sandbox and capabilities](./sandbox-and-capabilities.md)); those are read by the bundler. Unknown keys are an error.

Every key can be overridden on the command line, which is handy in CI:

```sh
nsh package App.tsx --target android --app-version 1.4.0 --build 42
```

## Options

| Flag | Description |
|---|---|
| `--target <ios \| ios-simulator \| android \| macos \| linux>` | What to build for (required). |
| `-o, --out <dir>` | Output directory. Default `out`. |
| `--name <name>` | Display name. |
| `--id <id>` | Bundle identifier, such as `com.example.app`. |
| `--app-version <version>` | Version, such as `1.2.0`. |
| `--build <n>` | Build number. |
| `--icon <png>` | App icon: a square PNG, ideally 1024×1024. |
| `--sign <identity>` | iOS: the code signing identity for devices. |
| `--profile <file>` | iOS: the provisioning profile for devices (`.mobileprovision`). |
| `--keystore <file>` | Android: a release keystore (default: the debug key). |
| `--key-alias <alias>` | Android: the key alias in the keystore. |
| `--keystore-password <password>` | Android: the keystore password; also read from `NSH_KEYSTORE_PASSWORD`. |
| `--run` | Install and launch afterwards: in the iOS simulator, or on the Android device or emulator `adb` sees; macOS and Linux apps are launched on this machine. Not for `--target ios`. |

## iOS simulator

```sh
nsh package App.tsx --target ios-simulator --run
```

Writes `out/<Name>.app`, signed ad hoc (the simulator needs a signature but no identity). With `--run` it boots a
simulator if none is running, installs the app and launches it. Requires Xcode.

## iOS devices

```sh
nsh package App.tsx --target ios \
  --sign "Apple Distribution: Example Inc (ABCDE12345)" \
  --profile AppStore.mobileprovision
```

Writes `out/<Name>.ipa`, ready for TestFlight, App Store upload tools or direct installation (depending on the
profile). What happens:

- The provisioning profile is copied into the app as `embedded.mobileprovision`.
- The entitlements come from the profile. nsh checks that the profile's application identifier covers the app's `id`
  (wildcard profiles such as `TEAMID.*` or `TEAMID.com.example.*` work) and stops with an error otherwise.
- The app is signed with `codesign` using the identity, then zipped into an `.ipa` (`Payload/<Name>.app`).

Find your identities with:

```sh
security find-identity -v -p codesigning
```

Without `--sign` and `--profile`, nsh writes an unsigned `out/<Name>.app` and tells you so; it cannot be installed on a
device as it is.

The app supports iPhone and iPad, portrait and landscape (portrait upside down on iPad only), and needs iOS 15.0 or
newer.

## Android

```sh
nsh package App.tsx --target android --run
```

Writes `out/<Name>.apk`. The APK is assembled directly with the SDK's build tools (`aapt2`, `zipalign`, `apksigner`),
without Gradle. Requirements:

- The Android SDK, found through `ANDROID_HOME` (or `ANDROID_SDK_ROOT`, or `~/Library/Android/sdk`), with build tools
  and the `platforms;android-35` package. nsh tells you the `sdkmanager` command when one is missing.
- A JDK, to create the debug keystore the first time.

The app targets API 35 and needs API 28 (Android 9) or newer; it contains native code for `arm64-v8a` and `x86_64`.

### Signing for release

Without `--keystore`, the APK is signed with the standard debug key (`~/.android/debug.keystore`, created if missing),
and nsh warns that it is not for store releases. For a release, create a keystore once and pass it:

```sh
keytool -genkeypair -v -keystore release.jks -alias upload -keyalg RSA -keysize 2048 -validity 10000

NSH_KEYSTORE_PASSWORD=… nsh package App.tsx --target android \
  --keystore release.jks --key-alias upload
```

Prefer the environment variable over `--keystore-password` so the password does not end up in your shell history.

## macOS

```sh
nsh package App.tsx --target macos --run
```

Writes `out/<Name>.app`: the executable in `Contents/MacOS/`, and `nsh/bundle.qbc`, `nsh/assets/…` and `nsh/app.json`
(the window title and the app's appearance) in `Contents/Resources/`. The app is signed ad hoc and needs macOS 14 or
later; to distribute it, sign it with a Developer ID and notarize it. See [macOS](./macos.md#packaging).

## Linux

```sh
nsh package App.tsx --target linux --run
```

Writes `out/<Name>/` (the executable `<Name>`, `nsh/bundle.nsh`, `nsh/assets/…`, `nsh/app.json` with the window title
and the application id, `<id>.desktop` and the icon `<id>.png`) and the same as `out/<Name>.tar.gz`. Package on
Linux: the runner is built for the machine it is built on. GTK 4 (4.14 or later) and Pango must be installed where the
app runs. Installing it and the desktop entry: [Linux](./linux.md#packaging).

## Icons

Give one square PNG, ideally 1024×1024, in `nsh.json` (`"icon": "icon.png"`) or with `--icon`:

- **iOS:** compiled by Xcode's `actool` into the app's asset catalog (`Assets.car`), which derives every size iOS
  needs.
- **Android:** resized to 192×192 and used as the launcher icon (`mipmap-xxxhdpi`).
- **macOS:** converted by `sips` to `AppIcon.icns`.
- **Linux:** copied as `<id>.png`, which the `.desktop` file names as its `Icon`.

Without an icon, the system shows its default.

## What is inside

The package holds your app's bytecode, its `capabilities.json` (when it declares capabilities) and the assets it
imports, next to the prebuilt runtime:

| Target | Runtime | Your app |
|---|---|---|
| iOS | the `nsh-ios-app` executable | `<Name>.app/nsh/bundle.nsh`, `nsh/assets/…` |
| Android | `classes.dex`, `lib/<abi>/libnsh.so` (stored uncompressed, 16 KB aligned) | `assets/nsh/bundle.nsh`, `assets/nsh/assets/…` |
| macOS | the `nsh-macos-app` executable, renamed `<Name>` | `<Name>.app/Contents/Resources/nsh/bundle.nsh`, `nsh/assets/…` |
| Linux | the `nsh-linux-app` executable, renamed `<Name>` | `<Name>/nsh/bundle.nsh`, `nsh/assets/…` |

A packaged app registers no [native components](./native-components.md); for those, embed nsh in your own app.

## Finding the runtime

`nsh package` looks for the prebuilt runtime next to the CLI (`bin/../runtime` in the distribution). In a checkout of
nsh it uses `target/<triple>/<profile>` and `target/android/<profile>`, with the CLI's own profile (a debug CLI uses
debug runtimes): build them with `cargo build -p nsh-ios-app --target aarch64-apple-ios-sim` (or `aarch64-apple-ios`
for devices) and `cargo xtask android` (add `--release` for a release CLI); `nsh-macos-app` (on macOS) and `nsh-linux-app` (on Linux) are built for this
machine in `target/<profile>` (`cargo build -p nsh-macos-app`, `cargo build -p nsh-linux-app`). The environment variable `NSH_RUNTIME`
points the CLI at another directory laid out like the distribution's `runtime/` (`ios/iphoneos`,
`ios/iphonesimulator`, `android/release`, `macos`, `linux`, ...).

## See also

- [CLI](./cli.md): every command
- [Signed packages](./signed-packages.md): updates apps download
- [Project structure](./project-structure.md): `nsh.json` and the files of an app
