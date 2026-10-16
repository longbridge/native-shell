use std::{
    net::{Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

use anyhow::Context as _;
use notify::{RecursiveMode, Watcher};
use nsh_dev::{DevServer, protocol::WS_PATH};

use crate::{build, logger};

/// File events arriving within this window are coalesced into one rebuild.
const DEBOUNCE: Duration = Duration::from_millis(30);
/// Ports tried after the requested one when it is taken.
const PORT_ATTEMPTS: u16 = 10;

/// A running dev server plus the source watcher feeding it.
pub(crate) struct DevSession {
    entry: PathBuf,
    src_dir: PathBuf,
    release: bool,
    server: DevServer,
    /// The modules of the last good development build: updates are
    /// computed against it.
    hot: std::cell::RefCell<Option<nsh_bundler::HotBundle>>,
}

impl DevSession {
    /// Starts the server and publishes the first build (or its error).
    pub(crate) fn start(entry: &Path, port: u16, release: bool) -> anyhow::Result<DevSession> {
        let entry = entry
            .canonicalize()
            .with_context(|| format!("entry not found: {}", entry.display()))?;
        let src_dir = entry
            .parent()
            .context("entry has no directory")?
            .to_path_buf();
        let server = start_server(port, &src_dir)?;
        server.serve_files(crate::build::crate_assets().map(|a| (a.path, a.source)));
        let session = DevSession {
            entry,
            src_dir,
            release,
            server,
            hot: Default::default(),
        };
        session.rebuild();
        Ok(session)
    }

    /// WebSocket URL for hosts on this machine.
    /// The port the dev server listens on.
    pub(crate) fn port(&self) -> u16 {
        self.server.local_addr().port()
    }

    pub(crate) fn local_url(&self) -> String {
        format!(
            "ws://127.0.0.1:{}{WS_PATH}",
            self.server.local_addr().port()
        )
    }

    /// Prints where hosts can connect.
    pub(crate) fn print_urls(&self) {
        let port = self.server.local_addr().port();
        log::info!("dev server: {}", self.local_url());
        for ip in lan_ips() {
            log::info!("devices on this network: ws://{ip}:{port}{WS_PATH}");
        }
    }

    /// Rebuilds on source changes until `stop` returns true.
    pub(crate) fn watch(
        &self,
        mut stop: impl FnMut() -> anyhow::Result<bool>,
    ) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
                if let Ok(event) = event {
                    let _ = tx.send(event);
                }
            })?;
        watcher.watch(&self.src_dir, RecursiveMode::Recursive)?;
        log::info!("watching {}", self.src_dir.display());
        // The app's Rust code: a change builds the app's hosts again.
        let rust = crate::rust_app::current();
        if let Some(app) = &rust
            && !app.crate_dir().starts_with(&self.src_dir)
        {
            watcher.watch(app.crate_dir(), RecursiveMode::Recursive)?;
        }
        loop {
            if stop()? {
                return Ok(());
            }
            let Ok(event) = rx.recv_timeout(Duration::from_millis(200)) else {
                continue;
            };
            if let Some(app) = &rust
                && event.paths.iter().any(|p| app.is_rust(p))
            {
                return Err(crate::rust_app::Restart.into());
            }
            if !is_source(&event) {
                continue;
            }
            let deadline = Instant::now() + DEBOUNCE;
            while rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .is_ok()
            {}
            self.rebuild();
        }
    }

    fn rebuild(&self) {
        self.server.building();
        if self.release {
            match build::compile(&self.entry, true) {
                Ok(built) => {
                    let capabilities = built.capabilities_json();
                    self.server.publish(
                        built.bytecode,
                        built.code,
                        built.source_map,
                        capabilities,
                        None,
                    )
                }
                Err(err) => self.failed(err),
            }
            return;
        }
        match build::compile_hot(&self.entry) {
            Ok((built, hot)) => {
                // Other capabilities: the app starts again under them.
                let patch = self
                    .hot
                    .borrow()
                    .as_ref()
                    .filter(|previous| previous.capabilities == hot.capabilities)
                    .map(|previous| hot.patch(previous));
                let patch = match patch {
                    Some(nsh_bundler::HotPatch::Update { code, source_map }) => {
                        Some((code, source_map))
                    }
                    // Nothing to replace; hosts still clear a build error.
                    Some(nsh_bundler::HotPatch::Unchanged) => Some(("true".to_owned(), None)),
                    Some(nsh_bundler::HotPatch::Reload) | None => None,
                };
                let capabilities = built.capabilities_json();
                self.server.publish(
                    built.bytecode,
                    built.code,
                    built.source_map,
                    capabilities,
                    patch,
                );
                *self.hot.borrow_mut() = Some(hot);
            }
            Err(err) => self.failed(err),
        }
    }

    fn failed(&self, err: build::BuildError) {
        log::error!("build failed:\n{}", err.colored);
        self.server.publish_error(err.plain);
    }
}

fn start_server(port: u16, assets_root: &Path) -> anyhow::Result<DevServer> {
    let mut last_err = None;
    for port in port..port.saturating_add(PORT_ATTEMPTS) {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let root = Some(assets_root.to_path_buf());
        match DevServer::start(addr, nsh_js::BYTECODE_VERSION, root, |log| {
            logger::print_client(&log)
        }) {
            Ok(server) => return Ok(server),
            Err(err) => last_err = Some(err),
        }
    }
    Err(last_err.expect("at least one attempt")).context("start dev server")
}

fn is_source(event: &notify::Event) -> bool {
    event.paths.iter().any(|p| {
        !p.components()
            .any(|c| c.as_os_str() == ".nsh" || c.as_os_str() == "node_modules")
            && matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("ts" | "tsx" | "js" | "jsx" | "json")
            )
    })
}

/// Private IPv4 addresses of physical network interfaces (`en*`), where
/// devices on the same Wi-Fi or Ethernet can reach this machine. VPN and
/// proxy tunnels are skipped.
fn lan_ips() -> Vec<Ipv4Addr> {
    let mut ips = Vec::new();
    let mut list: *mut libc::ifaddrs = std::ptr::null_mut();
    // SAFETY: getifaddrs fills a linked list that we walk read-only and free
    // with freeifaddrs.
    unsafe {
        if libc::getifaddrs(&mut list) != 0 {
            return ips;
        }
        let mut cur = list;
        while let Some(ifa) = cur.as_ref() {
            cur = ifa.ifa_next;
            if ifa.ifa_addr.is_null() || i32::from((*ifa.ifa_addr).sa_family) != libc::AF_INET {
                continue;
            }
            let name = std::ffi::CStr::from_ptr(ifa.ifa_name).to_string_lossy();
            if !name.starts_with("en") {
                continue;
            }
            let addr = &*(ifa.ifa_addr as *const libc::sockaddr_in);
            let ip = Ipv4Addr::from(u32::from_be(addr.sin_addr.s_addr));
            if ip.is_private() && !ips.contains(&ip) {
                ips.push(ip);
            }
        }
        libc::freeifaddrs(list);
    }
    ips
}
