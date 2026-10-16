use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context as _, bail};
use serde::Deserialize;

use crate::{dev::DevSession, ui};

const APP_NAME: &str = "NshDev";
const BUNDLE_ID: &str = "com.longbridge.nsh.host";

/// Where `nsh run` runs the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub(crate) enum Target {
    /// The Mac Catalyst development host (an iPhone-sized window).
    Catalyst,
    /// The development host in an iOS simulator.
    Ios,
    /// The development host on an Android device or emulator.
    Android,
    /// The Linux (GTK) development host, in a window on this machine.
    Linux,
}

impl Target {
    /// The default: the host that runs on this machine (Mac Catalyst on
    /// macOS, the GTK host on Linux; Android elsewhere).
    pub(crate) fn native() -> Target {
        if cfg!(target_os = "macos") {
            Target::Catalyst
        } else if cfg!(target_os = "linux") {
            Target::Linux
        } else {
            Target::Android
        }
    }

    fn platform(self) -> crate::runtime::Platform {
        match self {
            Target::Catalyst => crate::runtime::Platform::MacCatalyst,
            Target::Linux => crate::runtime::Platform::Linux,
            Target::Ios | Target::Android => crate::runtime::Platform::IphoneSimulator,
        }
    }
}

/// Starts a dev session and runs the app in a development host, which
/// loads bundles from the dev server and reloads on change.
pub(crate) fn run(
    entry: &Path,
    port: u16,
    release: bool,
    trace: bool,
    target: Target,
    device: Option<&str>,
) -> anyhow::Result<()> {
    let work = dunce::canonicalize(entry)?
        .parent()
        .context("entry has no directory")?
        .join(".nsh");
    if target == Target::Android {
        return run_android(entry, port, release, &work, trace);
    }
    // The host a previous session left running would connect to this one
    // until it is replaced.
    let udid = match target {
        Target::Ios => {
            let udid = simulator(device)?;
            let _ = simctl(&["terminate", &udid, BUNDLE_ID]);
            Some(udid)
        }
        Target::Catalyst | Target::Android | Target::Linux => None,
    };
    let session = DevSession::start(entry, port, release)?;
    session.print_urls();
    let host = crate::runtime::file(target.platform(), false)?;
    let mut args = vec!["--dev-server".to_owned(), session.local_url()];
    if trace {
        args.push("--trace".to_owned());
    }
    match target {
        Target::Catalyst => {
            let app = package_catalyst(&work, &host)?;
            let exe = app.join("Contents/MacOS/nsh-ios-host");
            // Its own output (AppKit's notes) is kept for when it fails.
            let mut command = Command::new(&exe);
            command
                .args(&args)
                // A host that crashed must not ask to reopen its windows
                // next time (a modal dialog that blocks the window).
                .args(["-ApplePersistenceIgnoreState", "YES"]);
            run_local(&session, &mut command, "the Mac Catalyst host")
        }
        // Run in place: unlike Catalyst's, the GTK host needs no app
        // bundle around it.
        Target::Linux => {
            let mut command = Command::new(&host);
            command.args(&args);
            run_local(&session, &mut command, "the Linux host")
        }
        Target::Ios => {
            let app = package_ios(&work, &host)?;
            let udid = udid.expect("the simulator is picked before the session");
            let step = ui::Step::start("installing the development host in the simulator");
            simctl(&["install", &udid, &app.to_string_lossy()])?;
            step.finish();
            let mut launch = vec!["launch", "--terminate-running-process", &udid, BUNDLE_ID];
            launch.extend(args.iter().map(String::as_str));
            let step = ui::Step::start("launching the development host");
            simctl(&launch)?;
            step.finish();
            // The simulator shares the Mac's network: 127.0.0.1 is the dev
            // server, so a host left running would join the next session
            // of any app: Ctrl-C stops it too.
            let stoppable = ui::Stoppable::new();
            let result = session.watch(|| Ok(false));
            let _ = simctl(&["terminate", &udid, BUNDLE_ID]);
            drop(stoppable);
            result
        }
        Target::Android => unreachable!("handled above"),
    }
}

/// Runs `command`, a host in a window on this machine, until its window
/// closes or the session ends; `what` names it when it fails.
fn run_local(session: &DevSession, command: &mut Command, what: &str) -> anyhow::Result<()> {
    // Its own output is kept for when it fails.
    let (mut child, output) = ui::quiet(command)
        .with_context(|| format!("launch {}", command.get_program().to_string_lossy()))?;
    // A host left running would open a second window next time: Ctrl-C and
    // SIGTERM close it too.
    let stoppable = ui::Stoppable::new();
    let result = session.watch(|| match child.try_wait()? {
        Some(status) if status.success() => Ok(true),
        Some(status) => Err(ui::failure(
            format!("{what} exited with {status}"),
            output.text(),
        )),
        None => Ok(false),
    });
    let _ = child.kill();
    let _ = child.wait();
    drop(stoppable);
    result
}

/// Starts a dev session, builds and installs the Android dev host (when
/// it changed), forwards the dev server port to the device and starts the
/// host.
fn run_android(
    entry: &Path,
    port: u16,
    release: bool,
    work: &Path,
    trace: bool,
) -> anyhow::Result<()> {
    use crate::android::{self, Apk, Signing};
    let sdk = android::Sdk::find()?;
    let serial = android::device(&sdk)?;
    // As on iOS: the host of a previous session must not connect first.
    let _ = android::adb(&sdk, &serial, &["shell", "am", "force-stop", BUNDLE_ID]);
    let session = DevSession::start(entry, port, release)?;
    session.print_urls();
    let runtime = crate::runtime::android(true)?;
    let apk_path = work.join("android/NshDev.apk");
    fs::create_dir_all(apk_path.parent().context("apk dir")?)?;
    let stamp = work.join("android/runtime.stamp");
    let version = runtime_version(&runtime)?;
    if !apk_path.exists() || fs::read_to_string(&stamp).ok().as_deref() != Some(version.as_str()) {
        let step = ui::Step::start("building the Android development host");
        android::build_apk(
            &sdk,
            &Apk {
                package: BUNDLE_ID,
                label: APP_NAME,
                version_name: "0.1.0",
                version_code: 1,
                activity: "com.longbridge.nsh.DevActivity",
                dev: true,
                dark: false,
                runtime: &runtime,
                icon: None,
                assets: Vec::new(),
                signing: Signing::Debug,
            },
            &apk_path,
        )?;
        step.done("built the Android development host");
        let step = ui::Step::start(format!("installing it on {serial}"));
        android::adb(
            &sdk,
            &serial,
            &["install", "-r", "-t", &apk_path.to_string_lossy()],
        )?;
        step.done(format!(
            "installed the Android development host on {serial}"
        ));
        fs::write(&stamp, &version)?;
    }
    // The device reaches the dev server on its own 127.0.0.1, until the
    // run ends.
    let _reverse = android::Reverse::add(&sdk, &serial, session.port())?;
    let url = session.local_url();
    let mut start = vec![
        "shell",
        "am",
        "start",
        "-S",
        "-n",
        "com.longbridge.nsh.host/com.longbridge.nsh.DevActivity",
        "--es",
        "url",
        &url,
    ];
    if trace {
        start.extend(["--ez", "trace", "true"]);
    }
    let step = ui::Step::start("launching the development host");
    android::adb(&sdk, &serial, &start)?;
    step.finish();
    let result = session.watch(|| Ok(false));
    let _ = android::adb(&sdk, &serial, &["shell", "am", "force-stop", BUNDLE_ID]);
    result
}

/// Identifies a runtime build: the sizes and times of its files.
fn runtime_version(runtime: &Path) -> anyhow::Result<String> {
    let mut parts = Vec::new();
    let mut files = vec![runtime.join("classes.dex")];
    for entry in fs::read_dir(runtime.join("lib"))? {
        files.push(entry?.path().join("libnsh.so"));
    }
    for file in files {
        if let Ok(meta) = fs::metadata(&file) {
            let modified = meta
                .modified()?
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            parts.push(format!("{}:{}:{modified}", file.display(), meta.len()));
        }
    }
    Ok(parts.join("\n"))
}

/// Creates (or refreshes) `<work>/NshDev.app` around the host binary.
fn package_catalyst(work: &Path, host: &Path) -> anyhow::Result<PathBuf> {
    let app = work.join(format!("{APP_NAME}.app"));
    let contents = app.join("Contents");
    let macos = contents.join("MacOS");
    fs::create_dir_all(&macos)?;
    fs::write(contents.join("Info.plist"), info_plist(Target::Catalyst))?;
    install_binary(host, &macos.join("nsh-ios-host"))?;
    Ok(app)
}

/// Creates (or refreshes) `<work>/ios/NshDev.app`, a flat iOS app bundle,
/// signed ad hoc (the simulator requires a signature).
fn package_ios(work: &Path, host: &Path) -> anyhow::Result<PathBuf> {
    let app = work.join("ios").join(format!("{APP_NAME}.app"));
    fs::create_dir_all(&app)?;
    fs::write(app.join("Info.plist"), info_plist(Target::Ios))?;
    if install_binary(host, &app.join("nsh-ios-host"))? || !app.join("_CodeSignature").exists() {
        let status = Command::new("codesign")
            .args(["--force", "--sign", "-", "--timestamp=none"])
            .arg(&app)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("run codesign")?;
        if !status.success() {
            bail!("codesign failed for {}", app.display());
        }
    }
    Ok(app)
}

/// Copies the host binary if it changed. Returns true if it did.
fn install_binary(host: &Path, dest: &Path) -> anyhow::Result<bool> {
    if !is_stale(host, dest)? {
        return Ok(false);
    }
    // Copy then rename: overwriting in place would modify the file a
    // running host (another session) maps, and the kernel kills processes
    // whose code no longer matches its cached signature.
    let tmp = dest.with_extension("tmp");
    fs::copy(host, &tmp).with_context(|| format!("copy {}", host.display()))?;
    fs::rename(&tmp, dest)?;
    Ok(true)
}

fn is_stale(src: &Path, dest: &Path) -> anyhow::Result<bool> {
    let Ok(dest_meta) = fs::metadata(dest) else {
        return Ok(true);
    };
    let src_meta = fs::metadata(src)?;
    Ok(src_meta.len() != dest_meta.len() || src_meta.modified()? > dest_meta.modified()?)
}

pub(crate) fn simctl(args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("xcrun")
        .arg("simctl")
        .args(args)
        .output()
        .context("run xcrun simctl (is Xcode installed?)")?;
    if !output.status.success() {
        bail!(
            "simctl {} failed: {}",
            args.first().unwrap_or(&""),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[derive(Deserialize)]
struct DeviceList {
    devices: std::collections::BTreeMap<String, Vec<Device>>,
}

#[derive(Deserialize)]
struct Device {
    udid: String,
    name: String,
    state: String,
    #[serde(rename = "isAvailable", default)]
    available: bool,
}

/// Picks a simulator (`wanted` by name or UDID; else a booted one; else the
/// newest iPhone), boots it and shows the Simulator app.
pub(crate) fn simulator(wanted: Option<&str>) -> anyhow::Result<String> {
    let list: DeviceList =
        serde_json::from_str(&simctl(&["list", "devices", "available", "--json"])?)?;
    // Runtimes sort by version within a platform; newest last.
    let mut devices: Vec<&Device> = list
        .devices
        .iter()
        .filter(|(runtime, _)| runtime.contains("SimRuntime.iOS"))
        .flat_map(|(_, devices)| devices)
        .filter(|d| d.available)
        .collect();
    devices.reverse();
    let device = match wanted {
        Some(w) => devices
            .iter()
            .find(|d| d.udid == w || d.name == w)
            .with_context(|| format!("no available simulator named {w}"))?,
        None => devices
            .iter()
            .find(|d| d.state == "Booted")
            .or_else(|| {
                devices
                    .iter()
                    .find(|d| d.name.starts_with("iPhone") && d.name.contains("Pro"))
            })
            .or_else(|| devices.iter().find(|d| d.name.starts_with("iPhone")))
            .context(
                "no iOS simulator available: install one with `xcodebuild -downloadPlatform iOS`",
            )?,
    };
    let booting =
        (device.state != "Booted").then(|| ui::Step::start(format!("booting {}", device.name)));
    if booting.is_some() {
        simctl(&["boot", &device.udid])?;
    }
    // Apps launched before the boot completes are sent to the background
    // when SpringBoard comes up.
    simctl(&["bootstatus", &device.udid, "-b"])?;
    if let Some(step) = booting {
        step.done(format!("booted {}", device.name));
    }
    show_simulators();
    Ok(device.udid.clone())
}

/// Brings up the simulators' window: Simulator.app, or DeviceHub.app in
/// Xcode versions that replaced it.
fn show_simulators() {
    let quiet = |cmd: &mut Command| {
        cmd.stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    };
    if quiet(Command::new("open").args(["-b", "com.apple.iphonesimulator"])) {
        return;
    }
    let developer = Command::new("xcode-select").arg("-p").output();
    if let Ok(out) = developer {
        let developer = String::from_utf8_lossy(&out.stdout);
        let hub = Path::new(developer.trim()).join("../Applications/DeviceHub.app");
        if hub.exists() && quiet(Command::new("open").arg(&hub)) {
            return;
        }
    }
    ui::warn("no Simulator or DeviceHub app found: the simulator runs without a window");
}

fn info_plist(target: Target) -> String {
    let platform = match target {
        // 6 = Mac idiom ("Optimize for Mac"): 1:1 points instead of the 77%
        // iPad scaling.
        Target::Catalyst => {
            "<key>UIDeviceFamily</key><array><integer>6</integer></array>\n    \
             <key>LSMinimumSystemVersion</key><string>11.0</string>"
        }
        // A launch screen entry opts into the full screen (no letterboxing).
        Target::Ios | Target::Android | Target::Linux => {
            "<key>UIDeviceFamily</key><array><integer>1</integer><integer>2</integer></array>\n    \
             <key>MinimumOSVersion</key><string>15.0</string>\n    \
             <key>CFBundleSupportedPlatforms</key><array><string>iPhoneSimulator</string></array>\n    \
             <key>UILaunchScreen</key><dict/>\n    \
             <key>NSAppTransportSecurity</key><dict><key>NSAllowsLocalNetworking</key><true/></dict>"
        }
    };
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>nsh-ios-host</string>
    <key>CFBundleIdentifier</key><string>{BUNDLE_ID}</string>
    <key>CFBundleName</key><string>{APP_NAME}</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>0.1.0</string>
    <key>CFBundleVersion</key><string>1</string>
    <key>UIApplicationSupportsMultipleScenes</key><false/>
    {platform}
</dict>
</plist>
"#
    )
}
