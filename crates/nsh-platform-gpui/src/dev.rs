//! Development host support. (Being ported: see
//! `nsh-platform-linux/src/dev.rs`.)

use std::io::Write;

use gpui::{App, Entity};

use crate::app::AppView;

struct DevLogger;

impl log::Log for DevLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.target().starts_with("nsh")
            || metadata.target() == "js"
            || metadata.level() <= log::Level::Warn
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            let _ = writeln!(
                std::io::stderr().lock(),
                "{:>5} {} {}",
                record.level(),
                record.target(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Installs a logger that prints to stderr. Call first thing in a
/// development host.
pub fn init_logging() {
    static LOGGER: DevLogger = DevLogger;
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

/// A connection to a dev server.
pub(crate) struct DevSession;

/// Connects `view` to the dev server at `url`.
pub(crate) fn start(_view: &Entity<AppView>, url: String, _cx: &mut App) {
    log::error!("dev server {url}: not supported yet");
}
