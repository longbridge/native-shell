//! Development support: the dev server client, log forwarding, the status
//! line and the error overlay.

use std::rc::Rc;

use crate::app::AppView;

/// Installs the development logger (stderr, and the dev server once
/// connected).
pub fn init_logging() {
    struct Logger;
    impl log::Log for Logger {
        fn enabled(&self, m: &log::Metadata<'_>) -> bool {
            m.level() <= log::Level::Info
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
}

/// A connection to a dev server.
pub(crate) struct DevSession {}

/// Connects `view` to the dev server at `url`.
pub(crate) fn start(view: &Rc<AppView>, url: String) {
    let _ = (view, url);
}
