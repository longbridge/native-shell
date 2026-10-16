use std::{
    io,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use tokio::sync::broadcast;

use crate::protocol::{
    BundleFormat, ClientMessage, LogLevel, PROTOCOL_VERSION, ServerMessage, WS_PATH,
};

/// What a dev host did, for the server's owner to show ([`DevServer::start`]).
#[derive(Debug, Clone)]
pub enum ClientEvent {
    /// A host connected.
    Connected {
        /// Device name from the client's hello.
        device: String,
        /// The build it was sent first: the latest bundle (`None`: none
        /// yet, or the latest build failed).
        build: Option<u64>,
    },
    /// A host's connection ended.
    Disconnected {
        /// Device name from the client's hello.
        device: String,
    },
    /// A log line from a host.
    Log(ClientLog),
}

/// A log line received from a dev host.
#[derive(Debug, Clone)]
pub struct ClientLog {
    /// Device name from the client's hello.
    pub device: String,
    /// Severity.
    pub level: LogLevel,
    /// Source (`js` or a Rust module path).
    pub target: String,
    /// The message.
    pub message: String,
}

struct Bundle {
    version: u64,
    bytecode: axum::body::Bytes,
    source: axum::body::Bytes,
    source_map: Option<axum::body::Bytes>,
    capabilities: Option<String>,
}

struct Patch {
    version: u64,
    base: u64,
    code: axum::body::Bytes,
    source_map: Option<axum::body::Bytes>,
}

#[derive(Clone)]
enum Event {
    Building,
    Failed(Arc<str>),
    Patch(Arc<Patch>),
    Bundle(Arc<Bundle>),
}

struct Shared {
    /// Directory served at `/assets/` (the app's source directory).
    assets_root: Option<std::path::PathBuf>,
    /// Files served at `/assets/<path>` from elsewhere (the files the UI
    /// sets' crates ship), by path.
    files: Mutex<std::collections::HashMap<String, std::path::PathBuf>>,
    /// The state a newly connected client must see first.
    latest: Mutex<Option<Event>>,
    events: broadcast::Sender<Event>,
    bytecode_version: String,
    /// Hosts connected now.
    clients: AtomicUsize,
    on_event: Box<dyn Fn(ClientEvent) + Send + Sync>,
}

/// The dev server: serves the latest bundle to dev hosts and pushes
/// rebuilds to them. Dropping it stops the server.
pub struct DevServer {
    shared: Arc<Shared>,
    addr: SocketAddr,
    next_version: Mutex<u64>,
    _runtime: tokio::runtime::Runtime,
}

impl DevServer {
    /// Starts listening on `addr`. `bytecode_version` is the format of the
    /// bytecode this server publishes; clients reporting another version get
    /// JS source instead. `on_event` receives what hosts do (connecting,
    /// disconnecting, their log lines), on the server's threads.
    pub fn start(
        addr: SocketAddr,
        bytecode_version: impl Into<String>,
        assets_root: Option<std::path::PathBuf>,
        on_event: impl Fn(ClientEvent) + Send + Sync + 'static,
    ) -> io::Result<DevServer> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("nsh-dev-server")
            .enable_all()
            .build()?;
        let shared = Arc::new(Shared {
            assets_root: assets_root.and_then(|p| p.canonicalize().ok()),
            files: Mutex::default(),
            latest: Mutex::new(None),
            events: broadcast::channel(16).0,
            bytecode_version: bytecode_version.into(),
            clients: AtomicUsize::new(0),
            on_event: Box::new(on_event),
        });
        let listener = runtime.block_on(tokio::net::TcpListener::bind(addr))?;
        let addr = listener.local_addr()?;
        let app = Router::new()
            .route(WS_PATH, get(upgrade))
            .route("/bundle.js", get(bundle_source))
            .route("/assets/{*path}", get(asset))
            .with_state(shared.clone());
        runtime.spawn(async move {
            if let Err(err) = axum::serve(listener, app).await {
                log::error!("dev server stopped: {err}");
            }
        });
        Ok(DevServer {
            shared,
            addr,
            next_version: Mutex::new(0),
            _runtime: runtime,
        })
    }

    /// Serves each `(path, file)` at `/assets/<path>` (`asset:/<path>` on
    /// device), besides the assets root: files that live elsewhere, as the
    /// crates of Rust components ship them.
    pub fn serve_files(&self, files: impl IntoIterator<Item = (String, std::path::PathBuf)>) {
        self.shared.files.lock().expect("files lock").extend(files);
    }

    /// The bound address (useful with port 0).
    pub fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// The number of hosts connected now.
    pub fn clients(&self) -> usize {
        self.shared.clients.load(Ordering::Relaxed)
    }

    /// Announces that a rebuild started.
    pub fn building(&self) {
        let _ = self.shared.events.send(Event::Building);
    }

    /// Publishes a new bundle (and its source map) to every client. With a
    /// `patch` (a hot update script from the previous bundle, and its source
    /// map), clients running the previous bundle update in place.
    pub fn publish(
        &self,
        bytecode: Vec<u8>,
        source: String,
        source_map: Option<String>,
        capabilities: Option<String>,
        patch: Option<(String, Option<String>)>,
    ) {
        let version = {
            let mut next = self.next_version.lock().expect("version lock");
            *next += 1;
            *next
        };
        if let Some((code, map)) = patch
            && version > 1
        {
            // Only for clients connected now: new clients get the bundle.
            let _ = self.shared.events.send(Event::Patch(Arc::new(Patch {
                version,
                base: version - 1,
                code: code.into(),
                source_map: map.map(Into::into),
            })));
        }
        self.set(Event::Bundle(Arc::new(Bundle {
            version,
            bytecode: bytecode.into(),
            source: source.into(),
            source_map: source_map.map(Into::into),
            capabilities,
        })));
    }

    /// Publishes a build error; clients keep running the last good bundle
    /// and show the error.
    pub fn publish_error(&self, message: impl Into<Arc<str>>) {
        self.set(Event::Failed(message.into()));
    }

    fn set(&self, event: Event) {
        *self.shared.latest.lock().expect("latest lock") = Some(event.clone());
        let _ = self.shared.events.send(event);
    }
}

async fn bundle_source(State(shared): State<Arc<Shared>>) -> Response {
    let latest = shared.latest.lock().expect("latest lock").clone();
    match latest {
        Some(Event::Bundle(bundle)) => (
            [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
            bundle.source.clone(),
        )
            .into_response(),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

/// Serves an app asset (`asset:/<path>` on device). Only the files
/// [`DevServer::serve_files`] named and files inside the assets root are
/// reachable.
async fn asset(
    State(shared): State<Arc<Shared>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response {
    let named = shared.files.lock().expect("files lock").get(&path).cloned();
    let file = match named {
        Some(file) => file,
        None => {
            let Some(root) = &shared.assets_root else {
                return StatusCode::NOT_FOUND.into_response();
            };
            let Ok(file) = root.join(&path).canonicalize() else {
                return StatusCode::NOT_FOUND.into_response();
            };
            if !file.starts_with(root) {
                return StatusCode::FORBIDDEN.into_response();
            }
            file
        }
    };
    let content_type = match file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("heic") => "image/heic",
        _ => "application/octet-stream",
    };
    match tokio::fs::read(&file).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, content_type)], bytes).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn upgrade(ws: WebSocketUpgrade, State(shared): State<Arc<Shared>>) -> Response {
    // Bundles are ~100 KB of bytecode; allow generous frames.
    ws.max_message_size(64 << 20)
        .on_upgrade(move |socket| async move {
            if let Err(err) = serve_client(socket, shared).await {
                log::debug!("dev client disconnected: {err}");
            }
        })
}

type BoxError = Box<dyn std::error::Error + Send + Sync>;

async fn serve_client(mut socket: WebSocket, shared: Arc<Shared>) -> Result<(), BoxError> {
    let hello = tokio::time::timeout(Duration::from_secs(5), socket.recv())
        .await
        .map_err(|_| "no hello")?
        .ok_or("closed before hello")??;
    let Message::Text(text) = hello else {
        return Err("expected hello".into());
    };
    let ClientMessage::Hello {
        protocol,
        bytecode_version,
        device,
    } = serde_json::from_str(&text)?
    else {
        return Err("expected hello".into());
    };
    if protocol != PROTOCOL_VERSION {
        return Err(
            format!("{device}: protocol {protocol}, server speaks {PROTOCOL_VERSION}").into(),
        );
    }
    let format = if bytecode_version == shared.bytecode_version {
        BundleFormat::Bytecode
    } else {
        log::warn!(
            "{device}: bytecode {bytecode_version} != {}, sending JS source",
            shared.bytecode_version
        );
        BundleFormat::Source
    };
    send(
        &mut socket,
        &ServerMessage::Hello {
            protocol: PROTOCOL_VERSION,
        },
    )
    .await?;

    // Subscribe before reading the latest state so no event is missed.
    let mut events = shared.events.subscribe();
    let latest = shared.latest.lock().expect("latest lock").clone();
    let build = match &latest {
        Some(Event::Bundle(bundle)) => Some(bundle.version),
        _ => None,
    };
    let _connection = Connection::new(&shared, &device, build);
    if let Some(event) = latest {
        send_event(&mut socket, &event, format).await?;
    }
    loop {
        tokio::select! {
            event = events.recv() => match event {
                Ok(event) => send_event(&mut socket, &event, format).await?,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    let latest = shared.latest.lock().expect("latest lock").clone();
                    if let Some(event) = latest {
                        send_event(&mut socket, &event, format).await?;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => return Ok(()),
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Text(text))) => {
                    if let Ok(ClientMessage::Log { level, target, message }) = serde_json::from_str(&text) {
                        (shared.on_event)(ClientEvent::Log(ClientLog { device: device.clone(), level, target, message }));
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(err)) => return Err(err.into()),
                None => return Ok(()),
            },
        }
    }
}

/// A host connected after its hello: counted and reported until its
/// session ends, however it ends.
struct Connection<'a> {
    shared: &'a Shared,
    device: &'a str,
}

impl<'a> Connection<'a> {
    fn new(shared: &'a Shared, device: &'a str, build: Option<u64>) -> Self {
        shared.clients.fetch_add(1, Ordering::Relaxed);
        (shared.on_event)(ClientEvent::Connected {
            device: device.to_owned(),
            build,
        });
        Connection { shared, device }
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        self.shared.clients.fetch_sub(1, Ordering::Relaxed);
        (self.shared.on_event)(ClientEvent::Disconnected {
            device: self.device.to_owned(),
        });
    }
}

async fn send(socket: &mut WebSocket, msg: &ServerMessage) -> Result<(), BoxError> {
    socket
        .send(Message::Text(serde_json::to_string(msg)?.into()))
        .await?;
    Ok(())
}

async fn send_event(
    socket: &mut WebSocket,
    event: &Event,
    format: BundleFormat,
) -> Result<(), BoxError> {
    match event {
        Event::Building => send(socket, &ServerMessage::Building).await,
        Event::Failed(message) => {
            send(
                socket,
                &ServerMessage::BuildFailed {
                    message: message.to_string(),
                },
            )
            .await
        }
        Event::Patch(patch) => {
            send(
                socket,
                &ServerMessage::Patch {
                    version: patch.version,
                    base: patch.base,
                    source_map: patch.source_map.is_some(),
                },
            )
            .await?;
            socket.send(Message::Binary(patch.code.clone())).await?;
            if let Some(map) = &patch.source_map {
                socket.send(Message::Binary(map.clone())).await?;
            }
            Ok(())
        }
        Event::Bundle(bundle) => {
            send(
                socket,
                &ServerMessage::Bundle {
                    version: bundle.version,
                    format,
                    source_map: bundle.source_map.is_some(),
                    capabilities: bundle.capabilities.clone(),
                },
            )
            .await?;
            let payload = match format {
                BundleFormat::Bytecode => bundle.bytecode.clone(),
                BundleFormat::Source => bundle.source.clone(),
            };
            socket.send(Message::Binary(payload)).await?;
            if let Some(map) = &bundle.source_map {
                socket.send(Message::Binary(map.clone())).await?;
            }
            Ok(())
        }
    }
}
