//! Image views: loading (`asset:/`, files, HTTP, `data:`), decoding into
//! textures, drawing with the resize mode, tint, blur and placeholder.
//!
//! Loads run on a small pool of worker threads: read the file or download
//! (ureq; `onProgress` at most every 100ms), decode (GDK's own PNG / JPEG /
//! TIFF loaders, then the `image` crate for GIF and WebP, whose first frame
//! is shown: animated images do not play) and hand the texture to the main
//! thread with `MainContext::invoke`. Textures are kept in a main-thread
//! cache keyed by the resolved location (so `asset:/` URIs of surfaces with
//! different asset bases never mix), bounded by bytes and evicting the
//! least recently used; concurrent requests for one location share one
//! load. Images decode at their full size: GSK scales them when drawing
//! (trilinear filtering when shrinking a lot).
//!
//! `blurRadius` is a GSK blur at draw time, in points; the image is drawn
//! enlarged under a clip to its own rectangle so the blur's edges stay
//! opaque. `tintColor` draws the color through the image's alpha.
//! `defaultSource` shows until the image arrives (and stays on error); an
//! image that had to load then fades in over `fadeDuration` (default 0, as
//! iOS).

use std::{
    cell::{Cell, RefCell},
    io::Read,
    path::PathBuf,
    rc::Rc,
    sync::OnceLock,
    time::{Duration, Instant},
};

use gtk4::{self as gtk, gdk, glib, graphene, gsk, prelude::*};
use nsh_core::{
    Command, EventKind, EventPayload, ImageOptions, ImageSource, NodeId, PayloadValue, ResizeMode,
};
use rustc_hash::FxHashMap;

use crate::{
    surface::Surface,
    view::{NshView, ViewState},
};

/// Decoded textures kept in memory, in bytes (4 per pixel).
const CACHE_BYTES: usize = 128 << 20;

/// The largest download or file read.
const MAX_BYTES: u64 = 64 << 20;

/// Loader threads: reads, downloads and decodes run there.
const WORKERS: usize = 4;

/// Progress reports at most this often (a download reports its end too).
const PROGRESS_INTERVAL: Duration = Duration::from_millis(100);

/// Where bundled (`asset:/…`) files are.
#[derive(Debug, Clone)]
pub enum AssetBase {
    /// Served over HTTP (the dev server): `http://host:port/assets/`.
    Http(String),
    /// A directory on disk.
    Directory(std::path::PathBuf),
}

/// Reads `asset:/…` files for the engine (fonts), on the engine thread.
pub(crate) struct AssetReader(pub(crate) Option<AssetBase>);

impl nsh_engine::Assets for AssetReader {
    fn read(&self, uri: &str) -> Result<Vec<u8>, String> {
        if !uri.starts_with("asset:/") {
            return Err(format!("not an asset URI: {uri}"));
        }
        match resolve(uri, self.0.as_ref()) {
            Some(location) => read(&location, &mut |_, _| {}),
            None => Err(format!("{uri}: no asset location configured")),
        }
    }
}

/// Where this surface's images resolve `asset:/` URIs.
pub(crate) fn set_asset_base(surface: &Rc<Surface>, base: Option<AssetBase>) {
    *surface.asset_base.borrow_mut() = base;
}

/// Where an image's bytes come from.
#[derive(Debug, Clone, PartialEq)]
enum Location {
    /// `http(s)://`.
    Url(String),
    /// A file on disk.
    File(PathBuf),
    /// A `data:` URI (decoded on the worker).
    Data(String),
}

impl Location {
    /// The cache key: the same location is the same image, whatever URI
    /// named it.
    fn key(&self) -> String {
        match self {
            Location::Url(url) => url.clone(),
            Location::File(path) => format!("file://{}", path.display()),
            Location::Data(uri) => uri.clone(),
        }
    }
}

/// Resolves `uri` (`asset:/…` against `base`, `http(s)://`, `file://`, an
/// absolute path, `data:`).
fn resolve(uri: &str, base: Option<&AssetBase>) -> Option<Location> {
    if let Some(path) = uri.strip_prefix("asset:/") {
        let path = path.trim_start_matches('/');
        return match base? {
            AssetBase::Http(base) => {
                let sep = if base.ends_with('/') { "" } else { "/" };
                Some(Location::Url(format!("{base}{sep}{path}")))
            }
            AssetBase::Directory(dir) => Some(Location::File(dir.join(path))),
        };
    }
    if uri.starts_with("http://") || uri.starts_with("https://") {
        return Some(Location::Url(uri.to_owned()));
    }
    if uri.starts_with("data:") {
        return Some(Location::Data(uri.to_owned()));
    }
    if uri.starts_with("file:") {
        return glib::filename_from_uri(uri)
            .ok()
            .map(|(path, _)| Location::File(path));
    }
    uri.starts_with('/').then(|| Location::File(uri.into()))
}

/// The bytes of a `data:` URI (base64 or percent-encoded).
fn data_uri(uri: &str) -> Result<Vec<u8>, String> {
    let rest = uri.strip_prefix("data:").ok_or("not a data URI")?;
    let (meta, data) = rest.split_once(',').ok_or("malformed data URI")?;
    if meta.ends_with(";base64") {
        let clean: String = data.chars().filter(|c| !c.is_ascii_whitespace()).collect();
        return Ok(glib::base64_decode(&clean));
    }
    let bytes = data.as_bytes();
    let hex = |b: u8| (b as char).to_digit(16);
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match (
            bytes[i],
            bytes.get(i + 1).and_then(|&b| hex(b)),
            bytes.get(i + 2).and_then(|&b| hex(b)),
        ) {
            (b'%', Some(h), Some(l)) => {
                out.push((h * 16 + l) as u8);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    Ok(out)
}

/// The HTTP agent all downloads share.
fn agent() -> &'static ureq::Agent {
    static AGENT: OnceLock<ureq::Agent> = OnceLock::new();
    AGENT.get_or_init(|| {
        ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(60)))
            .build()
            .into()
    })
}

/// Reads the bytes at `location` (blocking), calling `progress(loaded,
/// total)` (total 0: unknown) as a download goes.
fn read(location: &Location, progress: &mut dyn FnMut(u64, u64)) -> Result<Vec<u8>, String> {
    match location {
        Location::File(path) => std::fs::read(path).map_err(|e| format!("{}: {e}", path.display())),
        Location::Data(uri) => data_uri(uri),
        Location::Url(url) => {
            let mut response = agent().get(url).call().map_err(|e| format!("{url}: {e}"))?;
            let total = response.body().content_length().unwrap_or(0);
            let mut reader = response.body_mut().with_config().limit(MAX_BYTES).reader();
            let mut bytes = Vec::with_capacity(total.min(MAX_BYTES) as usize);
            let mut chunk = [0u8; 16 << 10];
            loop {
                let n = reader.read(&mut chunk).map_err(|e| format!("{url}: {e}"))?;
                if n == 0 {
                    break;
                }
                bytes.extend_from_slice(&chunk[..n]);
                progress(bytes.len() as u64, total);
            }
            Ok(bytes)
        }
    }
}

/// Decodes an encoded image (any thread): GDK's loaders first, then the
/// `image` crate (GIF, WebP: their first frame).
fn decode(bytes: Vec<u8>) -> Result<gdk::Texture, String> {
    let bytes = glib::Bytes::from_owned(bytes);
    let gdk_error = match gdk::Texture::from_bytes(&bytes) {
        Ok(texture) => return Ok(texture),
        Err(e) => e,
    };
    let image = image::load_from_memory(&bytes).map_err(|e| format!("{gdk_error}; {e}"))?;
    let rgba = image.into_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = glib::Bytes::from_owned(rgba.into_raw());
    Ok(gdk::MemoryTexture::new(
        width as i32,
        height as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &pixels,
        width as usize * 4,
    )
    .upcast())
}

/// One load for the workers.
struct Job {
    key: String,
    location: Location,
    /// Someone listens to `onProgress`.
    progress: bool,
}

/// Queues `job` on the worker pool (started on first use).
fn spawn(job: Job) {
    static POOL: OnceLock<crossbeam_channel::Sender<Job>> = OnceLock::new();
    let tx = POOL.get_or_init(|| {
        let (tx, rx) = crossbeam_channel::unbounded::<Job>();
        for i in 0..WORKERS {
            let rx = rx.clone();
            let spawned = std::thread::Builder::new()
                .name(format!("nsh-image-{i}"))
                .spawn(move || {
                    for job in rx {
                        work(job);
                    }
                });
            if let Err(e) = spawned {
                log::error!("cannot start an image loader thread: {e}");
            }
        }
        tx
    });
    let _ = tx.send(job);
}

/// Runs one job on a worker and hands the result to the main thread.
fn work(job: Job) {
    let Job {
        key,
        location,
        progress,
    } = job;
    let mut last: Option<Instant> = None;
    let mut report = |loaded: u64, total: u64| {
        let now = Instant::now();
        let done = total > 0 && loaded >= total;
        if !progress || !(done || last.is_none_or(|t| now - t >= PROGRESS_INTERVAL)) {
            return;
        }
        last = Some(now);
        let key = key.clone();
        glib::MainContext::default().invoke(move || {
            LOADER.with(|l| l.progress(&key, loaded, total));
        });
    };
    let result = read(&location, &mut report).and_then(decode);
    glib::MainContext::default().invoke(move || finish(key, result));
}

/// Called with a load's texture or error.
type Done = Box<dyn FnOnce(Result<gdk::Texture, String>)>;

/// Called with a download's bytes so far and its size (0: unknown).
type Progress = Box<dyn Fn(u64, u64)>;

/// Someone waiting for a load.
struct Waiter {
    done: Done,
    progress: Option<Progress>,
}

/// A cached texture.
struct Entry {
    texture: gdk::Texture,
    bytes: usize,
    used: u64,
}

/// The main thread's texture cache and running loads.
#[derive(Default)]
struct Loader {
    cache: RefCell<FxHashMap<String, Entry>>,
    bytes: Cell<usize>,
    clock: Cell<u64>,
    inflight: RefCell<FxHashMap<String, Vec<Waiter>>>,
}

thread_local! {
    static LOADER: Loader = Loader::default();
}

impl Loader {
    fn tick(&self) -> u64 {
        let now = self.clock.get() + 1;
        self.clock.set(now);
        now
    }

    fn cached(&self, key: &str) -> Option<gdk::Texture> {
        let mut cache = self.cache.borrow_mut();
        let entry = cache.get_mut(key)?;
        entry.used = self.tick();
        Some(entry.texture.clone())
    }

    fn insert(&self, key: String, texture: gdk::Texture) {
        let bytes = texture.width().max(0) as usize * texture.height().max(0) as usize * 4;
        if bytes > CACHE_BYTES / 2 {
            return;
        }
        let used = self.tick();
        let mut cache = self.cache.borrow_mut();
        let mut total = self.bytes.get() + bytes;
        let entry = Entry {
            texture,
            bytes,
            used,
        };
        if let Some(old) = cache.insert(key, entry) {
            total -= old.bytes;
        }
        // Rare, and the cache holds hundreds of entries at most: a scan for
        // the oldest beats keeping an ordered index up to date on every hit.
        while total > CACHE_BYTES {
            let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, e)| e.used)
                .map(|(k, _)| k.clone())
            else {
                break;
            };
            if let Some(e) = cache.remove(&oldest) {
                total -= e.bytes;
            }
        }
        self.bytes.set(total);
    }

    fn progress(&self, key: &str, loaded: u64, total: u64) {
        if let Some(waiters) = self.inflight.borrow().get(key) {
            for p in waiters.iter().filter_map(|w| w.progress.as_ref()) {
                p(loaded, total);
            }
        }
    }
}

/// A load finished (main thread): caches it and calls the waiters.
fn finish(key: String, result: Result<gdk::Texture, String>) {
    let waiters = LOADER.with(|l| {
        if let Ok(texture) = &result {
            l.insert(key.clone(), texture.clone());
        }
        l.inflight.borrow_mut().remove(&key).unwrap_or_default()
    });
    for w in waiters {
        (w.done)(result.clone());
    }
}

/// Loads `uri` for `surface`; `done` runs on the main thread. Returns
/// whether it already ran (cached, or a source that cannot load).
fn fetch(surface: &Surface, uri: &str, progress: Option<Progress>, done: Done) -> bool {
    let Some(location) = resolve(uri, surface.asset_base.borrow().as_ref()) else {
        done(Err(format!("unsupported image source `{uri}`")));
        return true;
    };
    let key = location.key();
    if let Some(texture) = LOADER.with(|l| l.cached(&key)) {
        done(Ok(texture));
        return true;
    }
    let listens = progress.is_some();
    let start = LOADER.with(|l| {
        let mut inflight = l.inflight.borrow_mut();
        let waiters = inflight.entry(key.clone()).or_default();
        waiters.push(Waiter { done, progress });
        waiters.len() == 1
    });
    if start {
        spawn(Job {
            key,
            location,
            progress: listens,
        });
    }
    false
}

/// Loads `uri` (`https://…`, `asset:/…`, a file path, `data:`) off the
/// main thread, through the texture cache; `done` runs on the main thread.
///
/// A cached texture is handed over before this returns: callers must not
/// hold a borrow of view state that `done` takes.
pub(crate) fn load(
    surface: &Rc<Surface>,
    uri: &str,
    done: impl FnOnce(Result<gdk::Texture, String>) + 'static,
) {
    fetch(surface, uri, None, Box::new(done));
}

/// The texture of `uri` if it is cached (to draw without waiting).
#[allow(dead_code)]
pub(crate) fn cached(surface: &Surface, uri: &str) -> Option<gdk::Texture> {
    let key = resolve(uri, surface.asset_base.borrow().as_ref())?.key();
    LOADER.with(|l| l.cached(&key))
}

/// `Command::ImageInfo` (`Image.getSize`, `Image.prefetch`): loads the
/// image into the texture cache and reports its size in pixels.
pub(crate) fn command(surface: &Rc<Surface>, _id: NodeId, command: &Command) {
    let Command::ImageInfo { id, uri } = command else {
        return;
    };
    let request = *id;
    let weak = Rc::downgrade(surface);
    load(surface, uri, move |result| {
        let Some(surface) = weak.upgrade() else {
            return;
        };
        let mut fields = vec![("id", PayloadValue::Number(request as f64))];
        match result {
            Ok(t) => {
                fields.push(("width", PayloadValue::Number(t.width() as f64)));
                fields.push(("height", PayloadValue::Number(t.height() as f64)));
            }
            Err(message) => fields.push(("error", PayloadValue::Str(message))),
        }
        surface.event(
            nsh_core::ROOT_ID,
            EventKind::ImageInfo,
            EventPayload::Map(fields),
        );
    });
}

/// A fade-in in progress.
#[derive(Debug, Clone, Copy)]
struct Fade {
    /// Frame time it started (µs); set on the first tick.
    start: Option<i64>,
    /// Duration in µs.
    duration: i64,
}

/// What an image view shows.
#[derive(Default)]
pub(crate) struct ImageState {
    source: Option<ImageSource>,
    mode: ResizeMode,
    options: ImageOptions,
    /// The source's texture, once loaded.
    image: Option<gdk::Texture>,
    /// The placeholder's texture, until the image shows.
    placeholder: Option<gdk::Texture>,
    /// Bumped on every source change: older loads are stale.
    generation: u64,
    /// The image's opacity while it fades in.
    alpha: f32,
    fade: Option<Fade>,
}

/// Sets up an image view.
pub(crate) fn create(view: &NshView) {
    view.state_mut().image = Some(ImageState::default());
}

/// An image view's source changed.
pub(crate) fn set(
    surface: &Rc<Surface>,
    view: &NshView,
    source: Option<&ImageSource>,
    mode: ResizeMode,
    options: &ImageOptions,
) {
    let (generation, needs_image, events, id) = {
        let mut s = view.state_mut();
        let (events, id) = (s.events, s.id);
        let image = s.image.get_or_insert_with(ImageState::default);
        let same_source = image.source.as_ref() == source;
        let same_placeholder = image.options.placeholder == options.placeholder;
        image.mode = mode;
        image.options = options.clone();
        if same_source && same_placeholder {
            drop(s);
            view.changed();
            return;
        }
        image.generation += 1;
        if !same_source {
            image.source = source.cloned();
            image.image = None;
            image.fade = None;
        }
        if !same_placeholder {
            image.placeholder = None;
        }
        (image.generation, image.image.is_none(), events, id)
    };
    view.changed();
    let weak = view.downgrade();
    // The image first: when it is cached, the placeholder is not needed.
    if let Some(source) = source.filter(|_| needs_image) {
        let progress = events.contains(EventKind::Progress).then(|| {
            let surface = Rc::downgrade(surface);
            let weak = weak.clone();
            Box::new(move |loaded: u64, total: u64| {
                let current = weak.upgrade().is_some_and(|v| {
                    v.state()
                        .image
                        .as_ref()
                        .is_some_and(|i| i.generation == generation)
                });
                if let (true, Some(surface)) = (current, surface.upgrade()) {
                    surface.event(
                        id,
                        EventKind::Progress,
                        EventPayload::Map(vec![
                            ("loaded", PayloadValue::Number(loaded as f64)),
                            ("total", PayloadValue::Number(total as f64)),
                        ]),
                    );
                }
            }) as Progress
        });
        // Fades only what had to wait.
        let waited = Rc::new(Cell::new(false));
        let done = {
            let surface = Rc::downgrade(surface);
            let weak = weak.clone();
            let waited = waited.clone();
            move |result: Result<gdk::Texture, String>| {
                if let (Some(view), Some(surface)) = (weak.upgrade(), surface.upgrade()) {
                    loaded(&surface, &view, generation, result, waited.get());
                }
            }
        };
        let immediate = fetch(surface, &source.uri, progress, Box::new(done));
        waited.set(true);
        if !immediate && events.contains(EventKind::LoadStart) {
            surface.event(id, EventKind::LoadStart, EventPayload::None);
        }
    }
    let wants_placeholder = view
        .state()
        .image
        .as_ref()
        .is_some_and(|i| i.image.is_none() && i.placeholder.is_none());
    if let Some(placeholder) = options.placeholder.as_ref().filter(|_| wants_placeholder) {
        load(surface, &placeholder.uri, move |result| {
            let Some(view) = weak.upgrade() else { return };
            match result {
                Ok(texture) => {
                    if let Some(i) = view.state_mut().image.as_mut()
                        && i.generation == generation
                    {
                        i.placeholder = Some(texture);
                    }
                    view.changed();
                }
                Err(message) => log::warn!("image placeholder failed: {message}"),
            }
        });
    }
}

/// The source of `view` loaded (or failed); `waited`: it was not cached,
/// so it fades in (when the app asked for a fade).
fn loaded(
    surface: &Surface,
    view: &NshView,
    generation: u64,
    result: Result<gdk::Texture, String>,
    waited: bool,
) {
    let (events, id, fades) = {
        let mut s = view.state_mut();
        let (events, id) = (s.events, s.id);
        let Some(image) = s.image.as_mut().filter(|i| i.generation == generation) else {
            return;
        };
        match &result {
            Ok(texture) => {
                image.image = Some(texture.clone());
                let ms = image.options.fade_ms.unwrap_or(0.0);
                image.alpha = 1.0;
                image.fade = None;
                if waited && ms > 0.0 {
                    image.alpha = 0.0;
                    image.fade = Some(Fade {
                        start: None,
                        duration: (ms * 1000.0) as i64,
                    });
                }
            }
            Err(message) => log::warn!("image failed: {message}"),
        }
        (events, id, image.fade.is_some())
    };
    if fades {
        view.add_tick_callback(|view, clock| fade_tick(view, clock.frame_time()));
    }
    view.changed();
    match &result {
        Ok(texture) if events.contains(EventKind::Load) => surface.event(
            id,
            EventKind::Load,
            EventPayload::Image {
                width: texture.width() as f32,
                height: texture.height() as f32,
            },
        ),
        Err(message) if events.contains(EventKind::Error) => surface.event(
            id,
            EventKind::Error,
            EventPayload::Error {
                message: message.clone(),
            },
        ),
        _ => {}
    }
    if events.contains(EventKind::LoadEnd) {
        surface.event(id, EventKind::LoadEnd, EventPayload::None);
    }
}

/// Advances a fade-in to frame time `now` (µs).
fn fade_tick(view: &NshView, now: i64) -> glib::ControlFlow {
    let running = {
        let mut s = view.state_mut();
        let Some(image) = s.image.as_mut() else {
            return glib::ControlFlow::Break;
        };
        let Some(fade) = image.fade.as_mut() else {
            return glib::ControlFlow::Break;
        };
        let start = *fade.start.get_or_insert(now);
        let t = ((now - start) as f32 / fade.duration.max(1) as f32).min(1.0);
        image.alpha = t;
        if t >= 1.0 {
            image.fade = None;
        }
        t < 1.0
    };
    view.changed();
    if running {
        glib::ControlFlow::Continue
    } else {
        glib::ControlFlow::Break
    }
}

/// `logo@2x.png` -> 2: the pixel density a file name declares (1 without).
fn density(uri: &str) -> f32 {
    let name = uri.rsplit('/').next().unwrap_or(uri);
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);
    stem.rsplit_once('@')
        .and_then(|(_, s)| s.strip_suffix('x'))
        .and_then(|n| n.parse::<f32>().ok())
        .filter(|n| *n > 0.0)
        .unwrap_or(1.0)
}

/// The natural size in points of `texture` loaded from `source`: the size
/// the source declares, or its pixels over the file name's density.
fn natural(texture: &gdk::Texture, source: Option<&ImageSource>) -> (f32, f32) {
    if let Some(size) = source
        .and_then(|s| s.size)
        .filter(|(w, h)| *w > 0.0 && *h > 0.0)
    {
        return size;
    }
    let d = source.map_or(1.0, |s| density(&s.uri));
    (texture.width() as f32 / d, texture.height() as f32 / d)
}

/// Where an image of natural size `natural` goes in a box of `bounds`
/// (x, y, width, height), as React Native fits it; it may extend past the
/// box (`cover`), which the view clips.
fn fit(mode: ResizeMode, natural: (f32, f32), bounds: (f32, f32)) -> (f32, f32, f32, f32) {
    let (nw, nh) = natural;
    let (bw, bh) = bounds;
    if nw <= 0.0 || nh <= 0.0 || matches!(mode, ResizeMode::Stretch) {
        return (0.0, 0.0, bw, bh);
    }
    let contain = (bw / nw).min(bh / nh);
    let scale = match mode {
        ResizeMode::Cover => (bw / nw).max(bh / nh),
        ResizeMode::Contain => contain,
        // Natural size, shrunk to fit when larger.
        ResizeMode::Center => contain.min(1.0),
        ResizeMode::Stretch => 1.0,
    };
    let (w, h) = (nw * scale, nh * scale);
    ((bw - w) / 2.0, (bh - h) / 2.0, w, h)
}

/// Draws `texture` at `rect` (points); mipmaps when it shrinks more than
/// a 2× screen would.
fn draw_texture(snapshot: &gtk::Snapshot, texture: &gdk::Texture, rect: &graphene::Rect) {
    let filter = if texture.width() as f32 > rect.width() * 2.0 {
        gsk::ScalingFilter::Trilinear
    } else {
        gsk::ScalingFilter::Linear
    };
    snapshot.append_scaled_texture(texture, filter, rect);
}

/// Draws one picture with the view's mode, tint and blur.
fn draw_one(
    snapshot: &gtk::Snapshot,
    state: &ViewState,
    image: &ImageState,
    texture: &gdk::Texture,
    source: Option<&ImageSource>,
) {
    let (w, h) = (state.frame.width, state.frame.height);
    let (x, y, fw, fh) = fit(image.mode, natural(texture, source), (w, h));
    let mut rect = graphene::Rect::new(x, y, fw, fh);
    let blur = image.options.blur_radius.max(0.0);
    if blur > 0.0 {
        // Enlarged under a clip to its own rectangle: the blur's
        // transparent fringe falls outside.
        let clip = rect
            .intersection(&graphene::Rect::new(0.0, 0.0, w, h))
            .unwrap_or(rect);
        snapshot.push_clip(&clip);
        snapshot.push_blur(blur as f64);
        let grow = 2.0 * blur;
        rect = graphene::Rect::new(x - grow, y - grow, fw + 2.0 * grow, fh + 2.0 * grow);
    }
    match state.style.tint_color {
        Some(tint) => {
            // The mask is recorded first, then the source.
            snapshot.push_mask(gsk::MaskMode::Alpha);
            draw_texture(snapshot, texture, &rect);
            snapshot.pop();
            snapshot.append_color(&crate::decor::rgba(tint), &rect);
            snapshot.pop();
        }
        None => draw_texture(snapshot, texture, &rect),
    }
    if blur > 0.0 {
        snapshot.pop();
        snapshot.pop();
    }
}

/// Draws an image view's picture.
pub(crate) fn draw(snapshot: &gtk::Snapshot, state: &ViewState) {
    let Some(image) = &state.image else { return };
    let fading = image.fade.is_some();
    if let Some(placeholder) = &image.placeholder
        && (image.image.is_none() || fading)
    {
        let source = image.options.placeholder.as_ref();
        draw_one(snapshot, state, image, placeholder, source);
    }
    let Some(texture) = &image.image else { return };
    let alpha = if fading { image.alpha } else { 1.0 };
    if alpha <= 0.0 {
        return;
    }
    if alpha < 1.0 {
        snapshot.push_opacity(alpha as f64);
    }
    draw_one(snapshot, state, image, texture, image.source.as_ref());
    if alpha < 1.0 {
        snapshot.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_uris() {
        let dir = AssetBase::Directory("/app/assets".into());
        let http = AssetBase::Http("http://10.0.0.2:8080/assets/".into());
        assert_eq!(
            resolve("asset:/img/a.png", Some(&dir)),
            Some(Location::File("/app/assets/img/a.png".into()))
        );
        assert_eq!(
            resolve("asset:/a.png", Some(&http)),
            Some(Location::Url("http://10.0.0.2:8080/assets/a.png".into()))
        );
        assert_eq!(resolve("asset:/a.png", None), None);
        assert_eq!(
            resolve("https://x.org/a.jpg", None),
            Some(Location::Url("https://x.org/a.jpg".into()))
        );
        assert_eq!(
            resolve("file:///home/a%20b.png", None),
            Some(Location::File("/home/a b.png".into()))
        );
        assert_eq!(
            resolve("/home/a.png", None),
            Some(Location::File("/home/a.png".into()))
        );
        assert_eq!(resolve("relative.png", None), None);
        assert!(matches!(
            resolve("data:image/png;base64,AA==", None),
            Some(Location::Data(_))
        ));
    }

    #[test]
    fn decodes_data_uris() {
        assert_eq!(data_uri("data:image/png;base64,aGk=").unwrap(), b"hi");
        assert_eq!(data_uri("data:text/plain,a%20b%zz").unwrap(), b"a b%zz");
        assert!(data_uri("data:nocomma").is_err());
    }

    #[test]
    fn reads_densities() {
        assert_eq!(density("asset:/logo@2x.png"), 2.0);
        assert_eq!(density("asset:/a@3x.webp"), 3.0);
        assert_eq!(density("https://x.org/me@home/a.png"), 1.0);
        assert_eq!(density("a.png"), 1.0);
    }

    #[test]
    fn fits_images() {
        let n = (200.0, 100.0);
        let b = (100.0, 100.0);
        assert_eq!(fit(ResizeMode::Cover, n, b), (-50.0, 0.0, 200.0, 100.0));
        assert_eq!(fit(ResizeMode::Contain, n, b), (0.0, 25.0, 100.0, 50.0));
        assert_eq!(fit(ResizeMode::Stretch, n, b), (0.0, 0.0, 100.0, 100.0));
        assert_eq!(fit(ResizeMode::Center, n, b), (0.0, 25.0, 100.0, 50.0));
        assert_eq!(
            fit(ResizeMode::Center, n, (400.0, 400.0)),
            (100.0, 150.0, 200.0, 100.0)
        );
    }
}
