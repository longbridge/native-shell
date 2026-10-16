//! Apps with Rust components of their own: `"rust": "<crate dir>"` in
//! `nsh.json` names a library crate whose `registry()` returns the
//! `nsh_core::ui::Registry` of its components and modules (a UI set, by the
//! specifier the app imports it as).
//!
//! The prebuilt toolchain cannot know such components, so `nsh` builds the
//! app its own: a Cargo workspace in `.nsh/rust` with the command line
//! (`nsh_cli::main_with` of the mobile UI and the app's set), the iOS
//! development host and release runner, and the Android library, each
//! linking the app's crate. A command on the app runs again in the app's
//! command line ([`reexec`]), which builds the hosts it needs
//! ([`RustApp::ios`], [`RustApp::android`]) instead of taking the prebuilt
//! ones. The workspace builds against nsh's sources (the checkout this
//! `nsh` was built in, or `NSH_SOURCE`), sharing their `Cargo.lock` and
//! target directory, so only the app's code and what links it compile.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context as _, bail};

use crate::runtime::Platform;

/// Set in the app's command line: which app it is for (its workspace).
const APP_ENV: &str = "NSH_RUST_APP";

/// Set in the app's command line when it runs again after a change
/// ([`RESTART`]): a failed build then waits for the next change instead of
/// ending the development session.
const RESTARTED_ENV: &str = "NSH_RUST_RESTARTED";

/// The exit code of the app's command line when the app's Rust code changed
/// during `nsh dev` / `nsh run`: `nsh` builds it again and runs the command
/// anew ([`reexec`]).
pub(crate) const RESTART: i32 = 75;

/// The error a development session ends with when the app's Rust code
/// changed ([`RESTART`]).
#[derive(Debug)]
pub(crate) struct Restart;

impl std::fmt::Display for Restart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the app's Rust code changed")
    }
}

impl std::error::Error for Restart {}

/// An app with a Rust crate of its own.
#[derive(Debug, Clone)]
pub(crate) struct RustApp {
    /// The generated workspace (`<app>/.nsh/rust`).
    workspace: PathBuf,
    /// nsh's sources.
    source: PathBuf,
    /// The app crate's package name (`gallery-native`).
    package: String,
    /// The app crate's directory.
    crate_dir: PathBuf,
}

/// The `"rust"` crate of the app whose entry is `entry`, if it has one.
pub(crate) fn of(entry: &Path) -> anyhow::Result<Option<RustApp>> {
    let entry = entry
        .canonicalize()
        .with_context(|| format!("entry not found: {}", entry.display()))?;
    let dir = entry.parent().context("entry has no directory")?;
    let Ok(text) = fs::read_to_string(dir.join("nsh.json")) else {
        return Ok(None);
    };
    let config: serde_json::Value = serde_json::from_str(&text).context("nsh.json is not JSON")?;
    let Some(rust) = config.get("rust") else {
        return Ok(None);
    };
    let rust = rust
        .as_str()
        .context("nsh.json: \"rust\" is the path of the app's crate")?;
    let crate_dir = dir
        .join(rust)
        .canonicalize()
        .with_context(|| format!("nsh.json: \"rust\": no crate at {rust}"))?;
    let package = package_name(&crate_dir.join("Cargo.toml"))?;
    let app = RustApp {
        workspace: dir.join(".nsh/rust"),
        source: source()?,
        package,
        crate_dir,
    };
    app.write_workspace()?;
    Ok(Some(app))
}

/// The app this command line runs for (`None`: the prebuilt `nsh`).
pub(crate) fn current() -> Option<RustApp> {
    let workspace = PathBuf::from(env::var_os(APP_ENV)?);
    let text = fs::read_to_string(workspace.join("app.json")).ok()?;
    let app: serde_json::Value = serde_json::from_str(&text).ok()?;
    Some(RustApp {
        workspace,
        source: PathBuf::from(app.get("source")?.as_str()?),
        package: app.get("package")?.as_str()?.to_owned(),
        crate_dir: PathBuf::from(app.get("crate")?.as_str()?),
    })
}

/// When this is the prebuilt `nsh` and the command's app has a Rust crate:
/// builds the app's command line and runs the command in it.
pub(crate) fn reexec(entry: Option<&Path>) -> anyhow::Result<Option<std::process::ExitStatus>> {
    if env::var_os(APP_ENV).is_some() {
        return Ok(None);
    }
    let Some(entry) = entry else {
        return Ok(None);
    };
    let Some(app) = of(entry)? else {
        return Ok(None);
    };
    let mut restarted = false;
    loop {
        log::info!(target: "nsh::rust", "building the command line of {}", app.package);
        let cli = match app.build(&app.bin("cli"), None, false, &[]) {
            Ok(cli) => cli,
            Err(e) if restarted => {
                app.wait_after(&e)?;
                continue;
            }
            Err(e) => return Err(e),
        };
        let mut command = Command::new(&cli);
        command
            .args(env::args_os().skip(1))
            .env(APP_ENV, &app.workspace);
        if restarted {
            command.env(RESTARTED_ENV, "1");
        }
        let status = command
            .status()
            .with_context(|| format!("run {}", cli.display()))?;
        if status.code() != Some(RESTART) {
            return Ok(Some(status));
        }
        restarted = true;
        log::info!(target: "nsh::rust", "the app's Rust code changed: building it again");
    }
}

/// nsh's sources: `NSH_SOURCE`, or the checkout this `nsh` was built in.
fn source() -> anyhow::Result<PathBuf> {
    if let Some(dir) = env::var_os("NSH_SOURCE") {
        return Ok(PathBuf::from(dir));
    }
    // target/<profile>/nsh (or target/<triple>/<profile>/...).
    let exe = env::current_exe()?;
    for dir in exe.ancestors().skip(1) {
        if dir.join("crates/nsh-cli/Cargo.toml").exists() {
            return Ok(dir.to_path_buf());
        }
    }
    bail!(
        "an app with a Rust crate (nsh.json \"rust\") builds against nsh's sources: set NSH_SOURCE to a nsh checkout"
    )
}

/// The `name` of the `[package]` of `manifest`.
fn package_name(manifest: &Path) -> anyhow::Result<String> {
    let text = fs::read_to_string(manifest)
        .with_context(|| format!("nsh.json: \"rust\": no {}", manifest.display()))?;
    let mut in_package = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
        } else if in_package
            && let Some(value) = line.strip_prefix("name")
            && let Some(value) = value.trim_start().strip_prefix('=')
        {
            return Ok(value.trim().trim_matches('"').to_owned());
        }
    }
    bail!("{}: no [package] name", manifest.display())
}

/// Writes `path` when its contents differ (Cargo rebuilds on changes).
fn write(path: &Path, contents: &str) -> anyhow::Result<()> {
    if fs::read_to_string(path).is_ok_and(|old| old == contents) {
        return Ok(());
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(path, contents).with_context(|| format!("write {}", path.display()))
}

impl RustApp {
    /// Its binaries' names: unique per app, as they share nsh's target
    /// directory (`nsh-app-gallery-native-cli`).
    fn bin(&self, what: &str) -> String {
        format!("nsh-app-{}-{what}", self.package)
    }

    /// Whether `path` is part of the app's Rust code (its sources and
    /// manifest, and the files of its `assets/` directory: the crate reads
    /// them when it builds).
    pub(crate) fn is_rust(&self, path: &Path) -> bool {
        let Ok(rest) = path.strip_prefix(&self.crate_dir) else {
            return false;
        };
        !rest.starts_with("target")
            && (rest.starts_with("assets")
                || path.extension().is_some_and(|e| e == "rs")
                || path.file_name().is_some_and(|n| n == "Cargo.toml"))
    }

    /// The app crate's directory (watched in development).
    pub(crate) fn crate_dir(&self) -> &Path {
        &self.crate_dir
    }

    /// Reports `error` (a build of the code just saved failed) and waits for
    /// the app's Rust code to change again.
    fn wait_after(&self, error: &anyhow::Error) -> anyhow::Result<()> {
        use notify::{RecursiveMode, Watcher};

        log::error!(target: "nsh::rust", "{error:#}: fix the code and save, nsh builds it again");
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    let _ = tx.send(event);
                }
            })?;
        watcher.watch(&self.crate_dir, RecursiveMode::Recursive)?;
        for event in &rx {
            if event.paths.iter().any(|p| self.is_rust(p)) {
                // Editors save in several writes: let them settle.
                std::thread::sleep(std::time::Duration::from_millis(100));
                while rx.try_recv().is_ok() {}
                return Ok(());
            }
        }
        bail!("stopped watching {}", self.crate_dir.display())
    }

    /// The generated workspace.
    fn write_workspace(&self) -> anyhow::Result<()> {
        let crate_dir = &self.crate_dir;
        let src = |c: &str| {
            let path = self.source.join("crates").join(c);
            format!("{{ path = {:?} }}", path.to_string_lossy())
        };
        let app = format!(
            "app = {{ package = {:?}, path = {:?} }}",
            self.package,
            crate_dir.to_string_lossy()
        );
        let package = |name: &str| {
            format!(
                "[package]\nname = {:?}\nversion = \"0.0.0\"\nedition = \"2024\"\npublish = false\n",
                self.bin(name)
            )
        };
        let sets = "vec![nsh_ui_mobile::registry(), app::registry()]";
        let ws = &self.workspace;
        write(
            &ws.join("Cargo.toml"),
            "# Generated by nsh for the app's Rust crate (nsh.json \"rust\"): do not edit.\n\
             [workspace]\nresolver = \"3\"\nmembers = [\"cli\", \"ios-host\", \"ios-app\", \"android\"]\n",
        )?;
        write(
            &ws.join("app.json"),
            &serde_json::json!({
                "source": self.source.to_string_lossy(),
                "package": self.package,
                "crate": self.crate_dir.to_string_lossy(),
            })
            .to_string(),
        )?;
        // The same versions as nsh's own build.
        let lock = ws.join("Cargo.lock");
        if !lock.exists() {
            fs::copy(self.source.join("Cargo.lock"), &lock).context("copy nsh's Cargo.lock")?;
        }
        write(
            &ws.join("cli/Cargo.toml"),
            &format!(
                "{}\n[[bin]]\nname = {:?}\npath = \"main.rs\"\n\n[dependencies]\nnsh-cli = {}\nnsh-ui-mobile = {}\n{app}\n",
                package("cli"),
                self.bin("cli"),
                src("nsh-cli"),
                src("nsh-ui-mobile"),
            ),
        )?;
        write(
            &ws.join("cli/main.rs"),
            &format!(
                "//! `nsh` for this app: the mobile UI and the app's Rust components.\n\n\
                 fn main() -> std::process::ExitCode {{\n    match nsh_cli::main_with({sets}) {{\n        \
                 Ok(()) => std::process::ExitCode::SUCCESS,\n        Err(err) => {{\n            \
                 eprintln!(\"Error: {{err:?}}\");\n            std::process::ExitCode::FAILURE\n        }}\n    }}\n}}\n"
            ),
        )?;
        for (dir, lib, what) in [
            ("ios-host", "nsh-ios-host", "development host"),
            ("ios-app", "nsh-ios-app", "release runner"),
        ] {
            write(
                &ws.join(dir).join("Cargo.toml"),
                &format!(
                    "{}\n[[bin]]\nname = {:?}\npath = \"main.rs\"\n\n[dependencies]\n{lib} = {}\nnsh-ui-mobile = {}\n{app}\n\n\
                     [target.'cfg(target_os = \"ios\")'.dependencies]\nnsh-platform-ios = {}\n",
                    package(dir),
                    self.bin(dir),
                    src(lib),
                    src("nsh-ui-mobile"),
                    src("nsh-platform-ios"),
                ),
            )?;
            let lib = lib.replace('-', "_");
            write(
                &ws.join(dir).join("main.rs"),
                &format!(
                    "//! The iOS {what} with the mobile UI and the app's Rust components.\n\n\
                     fn main() {{\n    #[cfg(target_os = \"ios\")]\n    nsh_platform_ios::set_ui_sets({sets});\n    {lib}::main();\n}}\n"
                ),
            )?;
        }
        write(
            &ws.join("android/Cargo.toml"),
            &format!(
                "{}\n[lib]\nname = \"nsh_app\"\npath = \"lib.rs\"\ncrate-type = [\"cdylib\"]\n\n[features]\ndev = [\"nsh-platform-android/dev\"]\n\n\
                 [dependencies]\nnsh-platform-android = {}\nnsh-ui-mobile = {}\n{app}\n",
                package("android"),
                src("nsh-platform-android"),
                src("nsh-ui-mobile"),
            ),
        )?;
        write(
            &ws.join("android/lib.rs"),
            &format!(
                "//! The app's `libnsh.so`: the Android runtime with the mobile UI and the\n//! app's Rust components.\n\n\
                 #[cfg(target_os = \"android\")]\nnsh::ui_sets!({sets});\n"
            ),
        )?;
        Ok(())
    }

    /// The target directory: nsh's own (its dependencies are built there
    /// already).
    fn target_dir(&self) -> PathBuf {
        self.source.join("target")
    }

    /// `cargo build`s package `package` (for `triple`, else the host) and
    /// returns its artifact `artifact` (default: the binary named as the
    /// package).
    fn build(
        &self,
        package: &str,
        triple: Option<&str>,
        release: bool,
        extra: &[&str],
    ) -> anyhow::Result<PathBuf> {
        self.build_with(
            Command::new(cargo()),
            package,
            triple,
            release,
            extra,
            package,
        )
    }

    fn build_with(
        &self,
        mut cmd: Command,
        package: &str,
        triple: Option<&str>,
        release: bool,
        extra: &[&str],
        artifact: &str,
    ) -> anyhow::Result<PathBuf> {
        cmd.arg("build")
            .arg("--manifest-path")
            .arg(self.workspace.join("Cargo.toml"))
            .args(["-p", package])
            .args(extra)
            .env("CARGO_TARGET_DIR", self.target_dir());
        if let Some(triple) = triple {
            cmd.args(["--target", triple]);
        }
        if release {
            cmd.arg("--release");
        }
        let status = cmd.status().context("run cargo")?;
        if !status.success() {
            let error = anyhow::anyhow!("building {package} failed");
            // A development session that ran: a mistake in the code just
            // saved waits for the fix (the command line runs again).
            if env::var_os(RESTARTED_ENV).is_some() {
                self.wait_after(&error)?;
                return Err(Restart.into());
            }
            return Err(error);
        }
        let mut path = self.target_dir();
        if let Some(triple) = triple {
            path.push(triple);
        }
        path.push(if release { "release" } else { "debug" });
        path.push(artifact);
        Ok(path)
    }

    /// The iOS development host (`runner` false) or release runner of the
    /// app, built for `platform`.
    pub(crate) fn ios(&self, platform: Platform, runner: bool) -> anyhow::Result<PathBuf> {
        let (what, name) = if runner {
            ("ios-app", "release runner")
        } else {
            ("ios-host", "development host")
        };
        log::info!(target: "nsh::rust", "building the iOS {name} of {}", self.package);
        self.build(&self.bin(what), Some(platform.triple()), runner, &[])
    }

    /// The Android runtime of the app: the prebuilt runtime's `classes.dex`
    /// with the app's `libnsh.so` for each of its ABIs.
    pub(crate) fn android(&self, prebuilt: &Path, dev: bool) -> anyhow::Result<PathBuf> {
        let sdk = crate::android::Sdk::find()?;
        let out = self.workspace.join(if dev {
            "android-dev"
        } else {
            "android-release"
        });
        fs::create_dir_all(&out)?;
        install(&prebuilt.join("classes.dex"), &out.join("classes.dex"))?;
        for entry in fs::read_dir(prebuilt.join("lib"))? {
            let abi = entry?.file_name().to_string_lossy().into_owned();
            let triple = match abi.as_str() {
                "arm64-v8a" => "aarch64-linux-android",
                "armeabi-v7a" => "armv7-linux-androideabi",
                "x86_64" => "x86_64-linux-android",
                "x86" => "i686-linux-android",
                other => bail!("unknown Android ABI {other}"),
            };
            log::info!(target: "nsh::rust", "building libnsh.so ({abi}) of {}", self.package);
            let features: &[&str] = if dev { &["--features", "dev"] } else { &[] };
            let lib = self.build_with(
                sdk.ndk_cargo(triple)?,
                &self.bin("android"),
                Some(triple),
                !dev,
                features,
                "libnsh_app.so",
            )?;
            let dir = out.join("lib").join(&abi);
            fs::create_dir_all(&dir)?;
            install(&lib, &dir.join("libnsh.so"))?;
        }
        Ok(out)
    }
}

/// Copies `from` to `to` when it is newer.
fn install(from: &Path, to: &Path) -> anyhow::Result<()> {
    let modified = |p: &Path| fs::metadata(p).and_then(|m| m.modified()).ok();
    if modified(to).is_some() && modified(to) >= modified(from) {
        return Ok(());
    }
    fs::copy(from, to).with_context(|| format!("copy {} to {}", from.display(), to.display()))?;
    Ok(())
}

/// The cargo to run.
fn cargo() -> std::ffi::OsString {
    env::var_os("CARGO").unwrap_or_else(|| "cargo".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_crates_assets_are_its_code() {
        let app = super::RustApp {
            workspace: "/app/.nsh/rust".into(),
            source: "/nsh".into(),
            package: "app-native".into(),
            crate_dir: "/app/native".into(),
        };
        for (path, rust) in [
            ("/app/native/src/lib.rs", true),
            ("/app/native/Cargo.toml", true),
            // `asset!` reads them when the crate builds.
            ("/app/native/assets/logo@2x.png", true),
            ("/app/native/assets/fonts/Script.ttf", true),
            ("/app/native/target/debug/build.rs", false),
            ("/app/native/README.md", false),
            ("/app/App.tsx", false),
        ] {
            assert_eq!(app.is_rust(std::path::Path::new(path)), rust, "{path}");
        }
    }

    #[test]
    fn package_names_come_from_the_package_table() {
        let dir = std::env::temp_dir().join(format!("nsh-rust-app-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            "[workspace]\nname = \"no\"\n\n[package]\nname = \"gallery-native\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        assert_eq!(super::package_name(&manifest).unwrap(), "gallery-native");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
