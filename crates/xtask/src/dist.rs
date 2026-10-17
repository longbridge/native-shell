//! The distribution: everything an app developer needs, prebuilt.
//!
//! ```text
//! dist/nsh/bin/nsh                                             the CLI
//! dist/nsh/runtime/ios/iphoneos/nsh-ios-app                    packaged app runner
//! dist/nsh/runtime/ios/iphonesimulator/{nsh-ios-app,nsh-ios-host}
//! dist/nsh/runtime/ios/maccatalyst/nsh-ios-host                development host
//! dist/nsh/runtime/macos/{nsh-macos-host,nsh-macos-app}        development host, packaged app runner
//! dist/nsh/runtime/linux/{nsh-linux-host,nsh-linux-app}        development host, packaged app runner
//! dist/nsh/runtime/android/{release,dev}/                      classes.dex, lib/<abi>/libnsh.so
//! dist/nsh/embed/ios/{NativeShell,NativeShellDev}.xcframework  embedding in iOS apps
//! dist/nsh/embed/android/{nsh,nsh-dev}.aar                     embedding in Android apps
//! ```
//!
//! The iOS runtimes and SDK need Xcode: they are built on macOS only, as is
//! the macOS (AppKit) runtime, built for this machine. The
//! Linux runtime is built for this machine, on Linux only (where it runs,
//! GTK 4 and Pango must be installed).

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context as _;

use crate::{android, term};

const IOS: &str = "aarch64-apple-ios";
const IOS_SIM: &str = "aarch64-apple-ios-sim";
const CATALYST: &str = "aarch64-apple-ios-macabi";

pub(crate) fn dist() -> anyhow::Result<()> {
    let root = android::root();
    let out = root.join("dist/nsh");
    if out.exists() {
        fs::remove_dir_all(&out)?;
    }
    let release = root.join("target");

    // The CLI for this machine.
    cargo(&["build", "--release", "-p", "nsh-cli"], "nsh")?;
    copy(&release.join("release/nsh"), &out.join("bin/nsh"))?;

    if cfg!(target_os = "macos") {
        ios_runtimes(&release, &out)?;
        for (package, what) in [
            ("nsh-macos-host", "the development host for macOS"),
            ("nsh-macos-app", "the release runner for macOS"),
        ] {
            cargo(&["build", "--release", "-p", package], what)?;
            copy(
                &release.join("release").join(package),
                &out.join("runtime/macos").join(package),
            )?;
        }
    }
    if cfg!(target_os = "linux") {
        for (package, what) in [
            ("nsh-linux-host", "the development host for Linux"),
            ("nsh-linux-app", "the release runner for Linux"),
        ] {
            cargo(&["build", "--release", "-p", package], what)?;
            copy(
                &release.join("release").join(package),
                &out.join("runtime/linux").join(package),
            )?;
        }
    }

    // Android runtimes.
    let abis: Vec<String> = android::ABIS.iter().map(|a| a.to_string()).collect();
    for (dev, name) in [(false, "release"), (true, "dev")] {
        let built = android::build(true, dev, &abis)?;
        let dest = out.join("runtime/android").join(name);
        copy(&built.join("classes.dex"), &dest.join("classes.dex"))?;
        for abi in &abis {
            copy(
                &built.join("lib").join(abi).join("libnsh.so"),
                &dest.join("lib").join(abi).join("libnsh.so"),
            )?;
        }
        let aar = out
            .join("embed/android")
            .join(if dev { "nsh-dev.aar" } else { "nsh.aar" });
        make_aar(&built, &abis, dev, &aar)?;
        term::done(format!(
            "packed {}",
            aar.strip_prefix(&root).unwrap_or(&aar).display()
        ));
    }

    if cfg!(target_os = "macos") {
        // iOS embedding: a static framework per platform (Rust + the Swift
        // module) in an XCFramework.
        crate::ios::xcframeworks(&out.join("embed/ios"), &out.join("work"))?;
        fs::remove_dir_all(out.join("work"))?;
    }
    term::note(out.strip_prefix(&root).unwrap_or(&out).display());
    Ok(())
}

/// The iOS runtimes: release runners, development hosts.
fn ios_runtimes(release: &Path, out: &Path) -> anyhow::Result<()> {
    for (triple, dir) in [(IOS, "iphoneos"), (IOS_SIM, "iphonesimulator")] {
        cargo(
            &[
                "build",
                "--release",
                "-p",
                "nsh-ios-app",
                "--target",
                triple,
            ],
            &format!("the release runner for {triple}"),
        )?;
        copy(
            &release.join(triple).join("release/nsh-ios-app"),
            &out.join("runtime/ios").join(dir).join("nsh-ios-app"),
        )?;
    }
    for (triple, dir) in [(IOS_SIM, "iphonesimulator"), (CATALYST, "maccatalyst")] {
        cargo(
            &[
                "build",
                "--release",
                "-p",
                "nsh-ios-host",
                "--target",
                triple,
            ],
            &format!("the development host for {triple}"),
        )?;
        copy(
            &release.join(triple).join("release/nsh-ios-host"),
            &out.join("runtime/ios").join(dir).join("nsh-ios-host"),
        )?;
    }
    Ok(())
}

/// An Android library archive: the Java runtime and `libnsh.so` per ABI.
///
/// The development library allows cleartext HTTP (merged into the app's
/// manifest): it loads assets from the `nsh dev` server over plain HTTP.
fn make_aar(built: &Path, abis: &[String], dev: bool, aar: &Path) -> anyhow::Result<()> {
    let work = aar.with_extension("aar-work");
    if work.exists() {
        fs::remove_dir_all(&work)?;
    }
    fs::create_dir_all(&work)?;
    fs::write(
        work.join("AndroidManifest.xml"),
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="com.longbridge.nsh">
    <uses-sdk android:minSdkVersion="{}" />
    <uses-permission android:name="android.permission.INTERNET" />{}
</manifest>
"#,
            android::MIN_SDK,
            if dev {
                "\n    <application android:usesCleartextTraffic=\"true\" />"
            } else {
                ""
            }
        ),
    )?;
    fs::write(work.join("R.txt"), "")?;
    copy(&built.join("classes.jar"), &work.join("classes.jar"))?;
    for abi in abis {
        copy(
            &built.join("lib").join(abi).join("libnsh.so"),
            &work.join("jni").join(abi).join("libnsh.so"),
        )?;
    }
    if let Some(dir) = aar.parent() {
        fs::create_dir_all(dir)?;
    }
    let _ = fs::remove_file(aar);
    term::run(
        Command::new("zip")
            .current_dir(&work)
            .args(["-q", "-r"])
            .arg(aar.canonicalize().unwrap_or_else(|_| abs(aar)))
            .arg("."),
    )?;
    fs::remove_dir_all(&work)?;
    Ok(())
}

fn abs(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        env::current_dir().unwrap_or_default().join(p)
    }
}

/// `cargo <args>` in the workspace: the step "building <what>".
pub(crate) fn cargo(args: &[&str], what: &str) -> anyhow::Result<()> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    term::cargo(
        Command::new(cargo).current_dir(android::root()).args(args),
        what,
    )
}

pub(crate) fn copy(from: &Path, to: &Path) -> anyhow::Result<()> {
    if let Some(dir) = to.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::copy(from, to).with_context(|| format!("copy {} -> {}", from.display(), to.display()))?;
    Ok(())
}
