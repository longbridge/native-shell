//! Locates the prebuilt runtime: native executables and libraries that
//! apps are packaged with, so building an app never needs Rust.
//!
//! Distribution layout (`cargo xtask dist`), next to `bin/nsh`:
//!
//! ```text
//! runtime/ios/iphoneos/nsh-ios-app           release runner (device)
//! runtime/ios/iphonesimulator/nsh-ios-app    release runner (simulator)
//! runtime/ios/iphonesimulator/nsh-ios-host   development host (simulator)
//! runtime/ios/maccatalyst/nsh-ios-host       development host (Mac Catalyst)
//! runtime/linux/nsh-linux-host               development host (Linux, GTK)
//! runtime/linux/nsh-linux-app                release runner (Linux)
//! runtime/macos/nsh-macos-host               development host (macOS, AppKit)
//! runtime/macos/nsh-macos-app                release runner (macOS)
//! runtime/windows/nsh-windows-host.exe       development host (Windows)
//! runtime/windows/nsh-windows-app.exe        release runner (Windows)
//! runtime/gpui/nsh-gpui-host[.exe]           development host (GPUI, this OS)
//! runtime/gpui/nsh-gpui-app[.exe]            release runner (GPUI, this OS)
//! runtime/android/release/             classes.dex + lib/<abi>/libnsh.so
//! runtime/android/dev/                 the same with development support
//! ```
//!
//! In a cargo checkout, the same files come from `target/<triple>/<profile>`
//! (with the profile of this CLI; Linux's, macOS's, Windows' and GPUI's
//! from `target/<profile>`, as they are built for this machine), and are
//! built with cargo first, so a run never uses a missing or stale host.
//! `NSH_RUNTIME` overrides the root. An app with a Rust crate of its own
//! gets hosts built for it instead (`rust_app.rs`).

use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context as _, bail};

/// A runtime platform directory and its cargo target triple.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Platform {
    /// iOS devices.
    IphoneOs,
    /// The iOS simulator.
    IphoneSimulator,
    /// Mac Catalyst.
    MacCatalyst,
    /// Linux (GTK), on this machine.
    Linux,
    /// macOS (AppKit), on this machine.
    MacOs,
    /// Windows (Win32), on this machine.
    Windows,
    /// GPUI, on this machine (macOS, Windows or Linux).
    Gpui,
}

impl Platform {
    fn dir(self) -> &'static str {
        match self {
            Platform::IphoneOs => "ios/iphoneos",
            Platform::IphoneSimulator => "ios/iphonesimulator",
            Platform::MacCatalyst => "ios/maccatalyst",
            Platform::Linux => "linux",
            Platform::MacOs => "macos",
            Platform::Windows => "windows",
            Platform::Gpui => "gpui",
        }
    }

    /// The cargo target triple (`None`: this machine's).
    pub(crate) fn triple(self) -> Option<&'static str> {
        match self {
            Platform::IphoneOs => Some("aarch64-apple-ios"),
            Platform::IphoneSimulator => Some("aarch64-apple-ios-sim"),
            Platform::MacCatalyst => Some("aarch64-apple-ios-macabi"),
            Platform::Linux | Platform::MacOs | Platform::Windows | Platform::Gpui => None,
        }
    }

    /// The development host (`runner` false) or release runner binary:
    /// its package, and its file without the executable suffix.
    pub(crate) fn binary(self, runner: bool) -> &'static str {
        match (self, runner) {
            (Platform::Linux, false) => "nsh-linux-host",
            (Platform::Linux, true) => "nsh-linux-app",
            (Platform::MacOs, false) => "nsh-macos-host",
            (Platform::MacOs, true) => "nsh-macos-app",
            (Platform::Windows, false) => "nsh-windows-host",
            (Platform::Windows, true) => "nsh-windows-app",
            (Platform::Gpui, false) => "nsh-gpui-host",
            (Platform::Gpui, true) => "nsh-gpui-app",
            (_, false) => "nsh-ios-host",
            (_, true) => "nsh-ios-app",
        }
    }

    /// The file of [`Platform::binary`]: with this machine's executable
    /// suffix (`.exe`) when it is built for this machine.
    pub(crate) fn file_name(self, runner: bool) -> String {
        let suffix = if self.triple().is_none() {
            env::consts::EXE_SUFFIX
        } else {
            ""
        };
        format!("{}{suffix}", self.binary(runner))
    }

    /// The development host (`runner` false) or release runner for it, as
    /// steps name it: "the development host for the iOS simulator".
    pub(crate) fn host(self, runner: bool) -> String {
        let what = if runner {
            "release runner"
        } else {
            "development host"
        };
        let platform = match self {
            Platform::IphoneOs => "iOS devices",
            Platform::IphoneSimulator => "the iOS simulator",
            Platform::MacCatalyst => "Mac Catalyst",
            Platform::Linux => "Linux",
            Platform::MacOs => "macOS",
            Platform::Windows => "Windows",
            Platform::Gpui => "GPUI",
        };
        format!("the {what} for {platform}")
    }
}

/// This `nsh`, through symlinks: an installer links `~/.local/bin/nsh` (or
/// `/opt/homebrew/bin/nsh`) to the distribution's `bin/nsh`, whose runtimes
/// are next to the real file. macOS reports the path as it was run, Linux
/// the resolved one; on Windows nothing links it, and canonical paths there
/// are `\\?\` ones that other tools reject.
pub(crate) fn cli_exe() -> std::io::Result<PathBuf> {
    let exe = env::current_exe()?;
    if cfg!(windows) {
        return Ok(exe);
    }
    Ok(exe.canonicalize().unwrap_or(exe))
}

/// The Android runtime directory (`classes.dex`, `lib/<abi>/libnsh.so`),
/// with development support or not.
pub(crate) fn android(dev: bool) -> anyhow::Result<PathBuf> {
    let prebuilt = prebuilt_android(dev)?;
    match crate::rust_app::current() {
        // The app's own library with the prebuilt Java runtime.
        Some(app) => app.android(&prebuilt, dev),
        None => Ok(prebuilt),
    }
}

fn prebuilt_android(dev: bool) -> anyhow::Result<PathBuf> {
    let variant = if dev { "dev" } else { "release" };
    if let Some(root) = env::var_os("NSH_RUNTIME") {
        return android_existing(PathBuf::from(root).join("android").join(variant), dev);
    }
    let exe = cli_exe()?;
    let bin = exe.parent().context("cli has no parent directory")?;
    let dist = bin.join("../runtime/android").join(variant);
    if dist.join("classes.dex").exists() {
        return Ok(dist);
    }
    // Cargo checkout: `cargo xtask android [--release] [--dev]` output.
    let profile = bin
        .file_name()
        .context("cli profile dir")?
        .to_string_lossy();
    if let Some(root) = checkout(bin) {
        let mut args = vec!["xtask", "android"];
        if profile == "release" {
            args.push("--release");
        }
        if dev {
            args.push("--dev");
        }
        let what = if dev {
            "the Android runtime with development support"
        } else {
            "the Android runtime"
        };
        cargo(&root, &args, what)?;
    }
    let name = if dev {
        format!("{profile}-dev")
    } else {
        profile.into_owned()
    };
    let built = bin
        .parent()
        .context("cli is not in a target directory")?
        .join("android")
        .join(name);
    android_existing(built, dev)
}

fn android_existing(dir: PathBuf, dev: bool) -> anyhow::Result<PathBuf> {
    if dir.join("classes.dex").exists() {
        return Ok(dir);
    }
    bail!(
        "the Android runtime is not at {}\nin a nsh checkout, build it with: cargo xtask android{}",
        dir.display(),
        if dev { " --dev" } else { "" }
    )
}

/// Path of the development host (`runner` false) or release runner for
/// `platform` ([`Platform::binary`]): for an app with a Rust crate, its own
/// build.
pub(crate) fn file(platform: Platform, runner: bool) -> anyhow::Result<PathBuf> {
    if let Some(app) = crate::rust_app::current() {
        return app.runtime(platform, runner);
    }
    let name = platform.binary(runner);
    let file_name = platform.file_name(runner);
    if let Some(root) = env::var_os("NSH_RUNTIME") {
        return existing(
            PathBuf::from(root).join(platform.dir()).join(&file_name),
            platform,
            name,
        );
    }
    let exe = cli_exe()?;
    let bin = exe.parent().context("cli has no parent directory")?;
    // Distribution: bin/nsh + runtime/.
    let dist = bin.join("../runtime").join(platform.dir()).join(&file_name);
    if dist.exists() {
        return Ok(dist);
    }
    // Cargo checkout: target/<profile>/nsh -> target/<triple>/<profile>/<name>.
    let profile = bin.file_name().context("cli profile dir")?;
    if let Some(root) = checkout(bin) {
        let mut args = vec!["build", "-p", name];
        if let Some(triple) = platform.triple() {
            args.extend(["--target", triple]);
        }
        match profile.to_str() {
            Some("debug") => {}
            Some("release") => args.push("--release"),
            Some(other) => args.extend(["--profile", other]),
            None => bail!("cli profile dir is not UTF-8"),
        }
        // The runner is "release" as opposed to the development host; the
        // build follows the CLI's own profile, so say which one it is.
        let what = match profile.to_str() {
            Some("release") => platform.host(runner),
            Some(other) => format!("{} ({other} build)", platform.host(runner)),
            None => platform.host(runner),
        };
        cargo(&root, &args, &what)?;
    }
    let mut built = bin
        .parent()
        .context("cli is not in a target directory")?
        .to_path_buf();
    if let Some(triple) = platform.triple() {
        built.push(triple);
    }
    built.push(profile);
    built.push(file_name);
    existing(built, platform, name)
}

/// The workspace of a cargo checkout of nsh this CLI was built in
/// (`<root>/target/<profile>/nsh`), if it is one.
fn checkout(bin: &Path) -> Option<PathBuf> {
    let root = bin.parent()?.parent()?;
    root.join("crates/nsh-ios-host/Cargo.toml")
        .exists()
        .then(|| root.to_owned())
}

/// Runs cargo in the checkout `root` (the one that runs this CLI under
/// `cargo run`, else `cargo` from the path): the step "building <what>".
fn cargo(root: &Path, args: &[&str], what: &str) -> anyhow::Result<()> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    crate::ui::cargo(Command::new(&cargo).args(args).current_dir(root), what)
}

fn existing(path: PathBuf, platform: Platform, name: &str) -> anyhow::Result<PathBuf> {
    if path.exists() {
        return Ok(path);
    }
    let target = platform
        .triple()
        .map(|triple| format!(" --target {triple}"))
        .unwrap_or_default();
    bail!(
        "{name} for {} not found at {}\nin a nsh checkout, build it with: cargo build -p {name}{target}",
        platform.dir(),
        path.display(),
    )
}

#[cfg(test)]
mod tests {
    use super::Platform;

    #[test]
    fn gpui_runtimes_are_built_for_this_machine() {
        assert_eq!(Platform::Gpui.dir(), "gpui");
        assert_eq!(Platform::Gpui.triple(), None);
        assert_eq!(Platform::Gpui.binary(false), "nsh-gpui-host");
        assert_eq!(Platform::Gpui.binary(true), "nsh-gpui-app");
        assert_eq!(
            Platform::Gpui.file_name(true),
            format!("nsh-gpui-app{}", std::env::consts::EXE_SUFFIX)
        );
        assert_eq!(Platform::Gpui.host(false), "the development host for GPUI");
    }
}
