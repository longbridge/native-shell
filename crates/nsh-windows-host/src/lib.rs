//! Development host on Windows: runs any nsh app in a Win32 window.
//!
//! - `nsh-windows-host --dev-server ws://127.0.0.1:8790/ws [--trace]` loads
//!   bundles pushed by `nsh dev` / `nsh run` and reloads on every rebuild.
//! - `nsh-windows-host --bundle <path> [--trace]` runs a built bundle (`.qbc`
//!   is QuickJS bytecode, anything else JS source).
//!
//! - `--script "wait 800; shot a.png; tap 100 200; wait 300; shot b.png;
//!   quit"` runs steps once the window shows: screenshots rendered
//!   offscreen and simulated input, to check the backend without touching
//!   the screen.
//! - `--size 390x844` sets the window size.
//!
//! Without arguments it connects to the dev server on this machine.
//!
//! A library so that an app with Rust components of its own (`nsh.json`
//! `"rust"`) builds the development host with them: its `main` calls
//! `nsh_platform_windows::set_ui_sets` and then [`main`].

/// Runs the development host (until its window closes).
#[cfg(windows)]
pub fn main() -> std::process::ExitCode {
    use std::{env, path::PathBuf};

    use nsh_platform_windows::{AppOptions, AppSource, AssetBase, Bundle, dev, run_app};

    dev::init_logging();
    let mut source = None;
    let mut options = AppOptions::default();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--dev-server" => source = args.next().map(AppSource::DevServer),
            "--bundle" => {
                let path = PathBuf::from(args.next().unwrap_or_default());
                // SAFETY: the developer names the file to run on the
                // command line of this development host.
                let bundle = match unsafe { Bundle::read_trusted(&path) } {
                    Ok(bundle) => bundle,
                    Err(err) => {
                        log::error!("cannot read bundle {}: {err}", path.display());
                        return std::process::ExitCode::FAILURE;
                    }
                };
                // `nsh build` copies imported assets next to the bundle.
                options.asset_base = path
                    .parent()
                    .map(|dir| AssetBase::Directory(dir.join("assets")));
                source = Some(AppSource::Bundle(bundle));
            }
            "--trace" => options.trace = true,
            // Steps run once the window shows (`nsh_platform_windows`'s
            // `script` module): offscreen screenshots and simulated input.
            "--script" => options.script = args.next(),
            "--size" => {
                if let Some((w, h)) = args.next().as_deref().and_then(|s| s.split_once('x')) {
                    options.size = (w.parse().unwrap_or(390), h.parse().unwrap_or(844));
                }
            }
            other => log::warn!("unknown argument {other}"),
        }
    }
    let source = source.unwrap_or_else(|| {
        use nsh_dev::protocol::{DEFAULT_PORT, WS_PATH};
        AppSource::DevServer(format!("ws://127.0.0.1:{DEFAULT_PORT}{WS_PATH}"))
    });
    options.title = "nsh".to_owned();
    run_app(source, options)
}

/// Not on Windows: says so and exits.
#[cfg(not(windows))]
pub fn main() -> std::process::ExitCode {
    eprintln!("nsh-windows-host runs on Windows");
    std::process::ExitCode::FAILURE
}
