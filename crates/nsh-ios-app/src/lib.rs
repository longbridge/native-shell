//! The executable of a packaged nsh app (`nsh package`): runs the bundle
//! shipped inside the app (`nsh/bundle.nsh`, assets in `nsh/assets`).
//!
//! A library so that an app with Rust components of its own (`nsh.json`
//! `"rust"`) builds the release runner with them: its `main` calls
//! `nsh_platform_ios::set_ui_sets` and then [`main`].

/// Runs the release runner (until the app quits).
#[cfg(target_os = "ios")]
pub fn main() {
    use nsh_platform_ios::{AppOptions, AppSource, AssetBase, Bundle, run_app};

    struct Logger;
    impl log::Log for Logger {
        fn enabled(&self, m: &log::Metadata<'_>) -> bool {
            m.level() <= log::Level::Warn || m.target() == "js"
        }
        fn log(&self, r: &log::Record<'_>) {
            if self.enabled(r.metadata()) {
                eprintln!("{} {}: {}", r.level(), r.target(), r.args());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: Logger = Logger;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Info);

    // The app bundle directory holds the executable.
    let dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join("nsh")))
        .unwrap_or_default();
    // SAFETY: the bytecode is a resource of this (signed) app.
    let bundle = match unsafe { Bundle::read_trusted(&dir.join("bundle.nsh")) } {
        Ok(bundle) => bundle,
        Err(err) => {
            log::error!("cannot read the app bundle in {}: {err}", dir.display());
            std::process::exit(1);
        }
    };
    run_app(
        AppSource::Bundle(bundle),
        AppOptions {
            trace: false,
            asset_base: Some(AssetBase::Directory(dir.join("assets"))),
        },
    );
}

/// Not on UIKit targets: says so and exits.
#[cfg(not(target_os = "ios"))]
pub fn main() {
    eprintln!("nsh-ios-app is the executable of packaged iOS apps; build it for an iOS target");
    std::process::exit(1);
}
