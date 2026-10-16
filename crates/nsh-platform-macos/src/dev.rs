//! Development support: the dev server client, log forwarding and the
//! error overlay.

use std::rc::Rc;

use crate::app::AppView;

/// Installs a logger for development hosts.
pub fn init_logging() {}

/// A connection to a dev server.
pub(crate) struct DevSession {}

/// Connects `view` to the dev server at `url`.
pub(crate) fn start(_view: &Rc<AppView>, url: String) {
    log::error!("dev server connections are not supported yet: {url}");
}
