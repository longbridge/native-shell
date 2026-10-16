//! Development tooling for nsh apps.
//!
//! A dev server (in the `nsh` CLI) watches the app sources, rebuilds on
//! change and pushes bundles over WebSocket to every connected dev host
//! (Mac Catalyst, simulators, devices). Hosts send their logs back so they
//! show up in the developer's terminal.
//!
//! - [`protocol`]: the wire format, shared by both sides.
//! - `client` (feature `client`): the host side.
//! - `server` (feature `server`): the CLI side.

pub mod protocol;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "server")]
mod server;

#[cfg(feature = "client")]
pub use crate::client::{DevClient, DevEvent, HostInfo};
#[cfg(feature = "server")]
pub use crate::server::{ClientEvent, ClientLog, DevServer};
