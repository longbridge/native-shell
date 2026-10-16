//! The iOS embedding SDK: `NativeShell.xcframework` (and `NativeShellDev`
//! with development support).
//!
//! Each platform slice is a static `NativeShell.framework` whose binary
//! holds the Rust runtime (`nsh-platform-ios` with feature `embed`, built as
//! a static library) and the `NativeShell` Swift module
//! (`crates/nsh-platform-ios/swift`), compiled with library evolution so apps
//! built with other Swift compilers can import it (`.swiftinterface`). The C
//! ABI (`NativeShellC`) is only used to compile the Swift module and is not
//! shipped.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context as _;

use crate::{
    android,
    dist::{cargo, copy},
    term::{self, run},
};

/// The lowest iOS the frameworks support (as `.cargo/config.toml`).
const MIN_IOS: &str = "15.0";

/// A platform slice.
struct Slice {
    /// Rust target.
    rust: &'static str,
    /// Swift target, without the OS version.
    swift: (&'static str, &'static str),
    /// SDK name for `xcrun --sdk`.
    sdk: &'static str,
    /// Mac Catalyst frameworks use the macOS (versioned) bundle layout.
    versioned: bool,
}

const SLICES: [Slice; 3] = [
    Slice {
        rust: "aarch64-apple-ios",
        swift: ("arm64-apple-ios", ""),
        sdk: "iphoneos",
        versioned: false,
    },
    Slice {
        rust: "aarch64-apple-ios-sim",
        swift: ("arm64-apple-ios", "-simulator"),
        sdk: "iphonesimulator",
        versioned: false,
    },
    Slice {
        rust: "aarch64-apple-ios-macabi",
        swift: ("arm64-apple-ios", "-macabi"),
        sdk: "macosx",
        versioned: true,
    },
];

/// Builds `NativeShell.xcframework` and `NativeShellDev.xcframework` into
/// `out`; intermediate files go to `work`.
pub(crate) fn xcframeworks(out: &Path, work: &Path) -> anyhow::Result<()> {
    let root = android::root();
    let crate_dir = root.join("crates/nsh-platform-ios");
    // The Swift module, once per slice (the same for both variants).
    let step = term::Step::start("compiling the NativeShell Swift module");
    let mut swift = Vec::new();
    for slice in &SLICES {
        let dir = work.join("swift").join(slice.rust);
        compile_swift(slice, &crate_dir, &dir)?;
        swift.push(dir);
    }
    step.done("compiled the NativeShell Swift module");
    for (dev, name) in [(false, "NativeShell"), (true, "NativeShellDev")] {
        let mut frameworks = Vec::new();
        for (slice, swift) in SLICES.iter().zip(&swift) {
            let features = if dev { "embed,dev" } else { "embed" };
            cargo(
                &[
                    "rustc",
                    "--release",
                    "-p",
                    "nsh-platform-ios",
                    "--target",
                    slice.rust,
                    "--features",
                    features,
                    "--crate-type",
                    "staticlib",
                ],
                &format!("the runtime of {name} for {}", slice.rust),
            )?;
            let rust_lib = root
                .join("target")
                .join(slice.rust)
                .join("release/libnsh_platform_ios.a");
            let framework = work
                .join(name)
                .join(slice.rust)
                .join("NativeShell.framework");
            make_framework(slice, &rust_lib, swift, &framework)?;
            frameworks.push(framework);
        }
        let xcframework = out.join(format!("{name}.xcframework"));
        if xcframework.exists() {
            fs::remove_dir_all(&xcframework)?;
        }
        let mut cmd = Command::new("xcodebuild");
        cmd.arg("-create-xcframework");
        for framework in &frameworks {
            cmd.arg("-framework").arg(framework);
        }
        cmd.arg("-output").arg(&xcframework);
        run(&mut cmd)?;
        term::done(format!("packed {name}.xcframework"));
    }
    Ok(())
}

fn sdk_path(sdk: &str) -> anyhow::Result<String> {
    let output = Command::new("xcrun")
        .args(["--sdk", sdk, "--show-sdk-path"])
        .output()
        .context("run xcrun")?;
    anyhow::ensure!(output.status.success(), "no {sdk} SDK (install Xcode)");
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}

/// Compiles the Swift module into `dir`: `libNativeShellSwift.a` and the
/// module files named by the target (`arm64-apple-ios-simulator.*`).
fn compile_swift(slice: &Slice, crate_dir: &Path, dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dir)?;
    let (arch_os, env) = slice.swift;
    let target = format!("{arch_os}{MIN_IOS}{env}");
    let module = dir.join(format!("{arch_os}{env}"));
    let sdk = sdk_path(slice.sdk)?;
    let mut swiftc = Command::new("xcrun");
    swiftc.arg("swiftc");
    if slice.versioned {
        // Mac Catalyst: UIKit and its Swift overlays live in the macOS SDK's
        // iOSSupport.
        let support = format!("{sdk}/System/iOSSupport");
        swiftc
            .arg("-Fsystem")
            .arg(format!("{support}/System/Library/Frameworks"))
            .arg("-I")
            .arg(format!("{support}/usr/lib/swift"));
    }
    run(swiftc
        .args(["-parse-as-library", "-module-name", "NativeShell"])
        .args(["-target", &target, "-sdk", &sdk])
        .args(["-swift-version", "5", "-enable-library-evolution"])
        .args([
            "-O",
            "-whole-module-optimization",
            "-emit-library",
            "-static",
        ])
        .arg("-emit-module-path")
        .arg(module.with_extension("swiftmodule"))
        .arg("-emit-module-interface-path")
        .arg(module.with_extension("swiftinterface"))
        .arg("-I")
        .arg(crate_dir.join("include"))
        .arg(crate_dir.join("swift/NativeShell.swift"))
        .arg("-o")
        .arg(dir.join("libNativeShellSwift.a")))
}

/// A static framework: the Rust and Swift libraries in one binary, the
/// Swift module and an Info.plist.
fn make_framework(
    slice: &Slice,
    rust_lib: &Path,
    swift: &Path,
    framework: &Path,
) -> anyhow::Result<()> {
    if framework.exists() {
        fs::remove_dir_all(framework)?;
    }
    let contents = if slice.versioned {
        framework.join("Versions/A")
    } else {
        framework.to_path_buf()
    };
    let resources = if slice.versioned {
        contents.join("Resources")
    } else {
        contents.clone()
    };
    fs::create_dir_all(&resources)?;
    run(Command::new("xcrun")
        .args(["libtool", "-static", "-no_warning_for_no_symbols", "-o"])
        .arg(contents.join("NativeShell"))
        .arg(rust_lib)
        .arg(swift.join("libNativeShellSwift.a")))?;
    let modules = contents.join("Modules/NativeShell.swiftmodule");
    fs::create_dir_all(&modules)?;
    for entry in fs::read_dir(swift)? {
        let path = entry?.path();
        let keep = path
            .extension()
            .is_some_and(|e| e == "swiftinterface" || e == "swiftmodule" || e == "swiftdoc");
        let private = path.to_string_lossy().ends_with(".private.swiftinterface");
        if keep && !private {
            copy(&path, &modules.join(path.file_name().expect("file")))?;
        }
    }
    fs::write(resources.join("Info.plist"), info_plist(slice))?;
    if slice.versioned {
        let link = |target: &str, name: &str| -> anyhow::Result<()> {
            std::os::unix::fs::symlink(target, framework.join(name))
                .with_context(|| format!("link {name}"))
        };
        std::os::unix::fs::symlink("A", framework.join("Versions/Current"))?;
        link("Versions/Current/NativeShell", "NativeShell")?;
        link("Versions/Current/Modules", "Modules")?;
        link("Versions/Current/Resources", "Resources")?;
    }
    Ok(())
}

fn info_plist(slice: &Slice) -> String {
    let platform = match slice.sdk {
        "iphoneos" => "iPhoneOS",
        "iphonesimulator" => "iPhoneSimulator",
        _ => "MacOSX",
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>NativeShell</string>
    <key>CFBundleIdentifier</key><string>com.longbridge.nsh.NativeShell</string>
    <key>CFBundleName</key><string>NativeShell</string>
    <key>CFBundlePackageType</key><string>FMWK</string>
    <key>CFBundleShortVersionString</key><string>{version}</string>
    <key>CFBundleVersion</key><string>{version}</string>
    <key>CFBundleSupportedPlatforms</key><array><string>{platform}</string></array>
    <key>MinimumOSVersion</key><string>{MIN_IOS}</string>
</dict>
</plist>
"#,
        version = env!("CARGO_PKG_VERSION"),
    )
}

/// `cargo xtask ios-embed`: only the XCFrameworks, into
/// `dist/nsh/embed/ios`.
pub(crate) fn build() -> anyhow::Result<PathBuf> {
    let root = android::root();
    let out = root.join("dist/nsh/embed/ios");
    let work = root.join("target/ios-embed");
    xcframeworks(&out, &work)?;
    Ok(out)
}
