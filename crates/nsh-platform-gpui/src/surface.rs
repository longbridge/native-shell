//! One surface: an engine thread and the views it mounts on GPUI's main
//! thread.
//!
//! The engine thread hands each commit to [`Sink`], which queues it and
//! sends one wake-up to the main thread ([`crate::rt::invoke`]); the main
//! thread drains the queue and applies every batch in order
//! ([`Surface::drain`]). Drawing is coalesced: changes mark the surface
//! invalid, and [`flush_all`] (which ends every main-thread entry point)
//! notifies the GPUI view showing it, which draws the tree in its next
//! frame ([`crate::render`]). Native animations and JS frame requests tick
//! on the window's frames ([`Surface::tick`], called while drawing) while
//! there is something to do.

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender};
use nsh_core::{EventKind, EventPayload, MountBatch, MountItem, NodeId, ROOT_ID, Size};
use nsh_engine::{Engine, EngineConfig, EngineControl, Environment, MountSink};
use rustc_hash::FxHashMap;

use crate::{
    animation::UiAnimation,
    mount::Mounter,
    text::GpuiTextEngine,
    view::{Hosted, NshView},
};

/// What the engine thread sends the main thread.
enum Message {
    Batch(MountBatch),
    Error(String),
    Frame,
    HotUpdateFailed,
}

thread_local! {
    /// The surfaces of this (main) thread, by key.
    static SURFACES: RefCell<FxHashMap<u64, Weak<Surface>>> = RefCell::default();
}

static NEXT_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Forwards batches from the engine thread to the main thread.
struct Sink {
    /// The surface to wake ([`Surface::by_key`]).
    key: u64,
    tx: Sender<Message>,
    /// A wake-up is on its way: no need for another.
    pending: Arc<AtomicBool>,
}

impl Sink {
    fn send(&self, message: Message) {
        let _ = self.tx.send(message);
        if !self.pending.swap(true, Ordering::AcqRel) {
            let key = self.key;
            crate::rt::invoke(move || {
                if let Some(surface) = Surface::by_key(key) {
                    surface.drain();
                }
            });
        }
    }
}

impl MountSink for Sink {
    fn submit(&self, batch: MountBatch) {
        self.send(Message::Batch(batch));
    }

    fn report_error(&self, message: &str) {
        self.send(Message::Error(message.to_owned()));
    }

    fn request_frame(&self) {
        self.send(Message::Frame);
    }

    fn hot_update_failed(&self) {
        self.send(Message::HotUpdateFailed);
    }
}

/// Called on every frame with the frame's time until it returns false
/// ([`Surface::on_frame`]).
pub(crate) type FrameClient = Box<dyn FnMut(&Rc<Surface>, f64) -> bool>;

/// Called with the new size (points) when the surface is resized.
pub(crate) type ResizeHandler = Box<dyn Fn(f32, f32)>;

/// A window shortcut: true if it took the key (down).
pub(crate) type ShortcutHandler = Box<dyn Fn(&crate::input::KeyEvent) -> bool>;

/// Called when the app reports a fatal error (development hosts show it).
pub(crate) type ErrorHandler = Box<dyn Fn(&str)>;

/// One engine and its views.
pub(crate) struct Surface {
    key: u64,
    /// The root view (node [`ROOT_ID`]): fills the window.
    pub(crate) root: NshView,
    /// Views by node id, and what mounting needs between items.
    pub(crate) mounter: RefCell<Mounter>,
    /// The views by node id (the mounter's, readable while it applies a
    /// batch).
    views: Rc<crate::mount::Views>,
    /// The native animation graph.
    pub(crate) anim: RefCell<UiAnimation>,
    /// Stacks, screens and modals.
    pub(crate) nav: RefCell<crate::navigation::Nav>,
    /// Pointer input state ([`crate::input`]).
    pub(crate) input: crate::input::Input,
    engine: RefCell<Option<Engine>>,
    control: RefCell<Option<EngineControl>>,
    rx: RefCell<Option<Receiver<Message>>>,
    pending: RefCell<Arc<AtomicBool>>,
    /// Pixels per point (the window's scale factor).
    scale: Cell<f32>,
    /// The GPUI view drawing the surface ([`crate::app::AppView`]): notified
    /// when the surface changed.
    pub(crate) entity: Cell<Option<gpui::EntityId>>,
    /// The GPUI text system (fonts the engine thread shapes with, and the
    /// main thread draws with).
    pub(crate) text_system: RefCell<Option<Arc<gpui::TextSystem>>>,
    /// Where the surface was drawn last, in window coordinates (points):
    /// input arrives in window coordinates.
    pub(crate) origin: Cell<(f32, f32)>,
    /// A draw is requested.
    invalid: Cell<bool>,
    /// Mouse buttons pressed on the surface (bit per button): their moves
    /// and releases are the surface's wherever they happen.
    pub(crate) input_pressed: Cell<u8>,
    /// The mouse is over the surface.
    pub(crate) input_hovered: Cell<bool>,
    /// The environment the engine was last told about.
    pub(crate) environment: Cell<Environment>,
    /// The app's appearance (`nsh.json`, `setColorScheme`): the color
    /// scheme is the system's resolved with it ([`crate::system`]).
    pub(crate) appearance: Cell<nsh_core::platform::Appearance>,
    /// JS asked for a display frame.
    js_frame: Cell<bool>,
    /// Extra frame clients (transitions, glides, spinners): called on every
    /// frame until they return false ([`Surface::on_frame`]).
    frame_clients: RefCell<Vec<FrameClient>>,
    /// The time of the current frame (seconds since the surface was made).
    now: Cell<f64>,
    epoch: std::time::Instant,
    on_error: RefCell<Option<ErrorHandler>>,
    on_hot_update_failed: RefCell<Option<Box<dyn Fn()>>>,
    /// Where `asset:/` image URIs resolve ([`crate::image::set_asset_base`]).
    pub(crate) asset_base: RefCell<Option<crate::image::AssetBase>>,
    /// Videos, web views and registered native components.
    pub(crate) native: crate::native::NativeState,
    /// The app's keyboard shortcuts ([`crate::keymap`]).
    pub(crate) keymap: crate::keymap::KeymapState,
    /// Hosted items drawn by the last frame ([`Hosted::placed`]).
    placed: RefCell<Vec<Rc<dyn Hosted>>>,
    /// The window's chrome: title bar, menus ([`crate::chrome`]).
    pub(crate) chrome: crate::chrome::Chrome,
    /// Called after the surface is resized (development overlays).
    pub(crate) on_resized: RefCell<Vec<ResizeHandler>>,
    /// Keys the window takes before anything focused sees them (the
    /// standalone app's full screen, the dev host's reload).
    pub(crate) shortcuts: RefCell<Vec<ShortcutHandler>>,
    /// The embedding app's theme ([`crate::AppView::set_theme`]): sent to
    /// every engine the surface starts, before its first frame.
    pub(crate) host_theme: RefCell<Option<nsh_engine::Theme>>,
    /// The embedding app's appearance (dark or not), standing in for the
    /// window's ([`crate::AppView::set_appearance`]).
    pub(crate) host_dark: Cell<Option<bool>>,
}

impl Surface {
    /// A surface with no app and no window yet.
    pub(crate) fn new(environment: Environment) -> Rc<Surface> {
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
        let root = NshView::new(ROOT_ID, nsh_core::ViewKind::View);
        let mounter = Mounter::new(root.clone());
        let surface = Rc::new(Surface {
            key,
            root: root.clone(),
            views: mounter.views.clone(),
            mounter: RefCell::new(mounter),
            anim: RefCell::new(UiAnimation::default()),
            nav: RefCell::default(),
            input: Default::default(),
            engine: RefCell::new(None),
            control: RefCell::new(None),
            rx: RefCell::new(None),
            pending: RefCell::new(Arc::new(AtomicBool::new(false))),
            scale: Cell::new(1.0),
            entity: Cell::new(None),
            text_system: RefCell::new(None),
            origin: Cell::new((0.0, 0.0)),
            invalid: Cell::new(false),
            input_pressed: Cell::new(0),
            input_hovered: Cell::new(false),
            environment: Cell::new(environment),
            appearance: Cell::default(),
            js_frame: Cell::new(false),
            frame_clients: RefCell::default(),
            now: Cell::new(0.0),
            epoch: std::time::Instant::now(),
            on_error: RefCell::new(None),
            on_hot_update_failed: RefCell::new(None),
            asset_base: RefCell::new(None),
            native: Default::default(),
            keymap: Default::default(),
            placed: RefCell::default(),
            on_resized: RefCell::default(),
            shortcuts: RefCell::default(),
            chrome: Default::default(),
            host_theme: RefCell::new(None),
            host_dark: Cell::new(None),
        });
        SURFACES.with(|s| s.borrow_mut().insert(key, Rc::downgrade(&surface)));
        let weak = Rc::downgrade(&surface);
        root.set_invalidate(Rc::new(move || {
            if let Some(surface) = weak.upgrade() {
                surface.invalidate();
            }
        }));
        surface
    }

    /// The surface with `key`, if it is alive.
    pub(crate) fn by_key(key: u64) -> Option<Rc<Surface>> {
        SURFACES.with(|s| s.borrow().get(&key).and_then(Weak::upgrade))
    }

    /// Its key ([`Surface::by_key`]).
    pub(crate) fn key(&self) -> u64 {
        self.key
    }

    /// Pixels per point.
    pub(crate) fn scale(&self) -> f32 {
        self.scale.get()
    }

    /// The surface's size in points.
    pub(crate) fn size(&self) -> (f32, f32) {
        let s = self.root.state();
        (s.frame.width, s.frame.height)
    }

    /// The current frame's time (seconds, the animation clock).
    pub(crate) fn now(&self) -> f64 {
        self.now.get()
    }

    /// Seconds since the surface was made (the animation clock).
    pub(crate) fn clock(&self) -> f64 {
        self.epoch.elapsed().as_secs_f64()
    }

    /// Starts `config`'s app, replacing what ran.
    pub(crate) fn start(self: &Rc<Self>, mut config: EngineConfig) {
        self.stop();
        let env = self.environment.get();
        config.viewport = Size::new(env.window.0, env.window.1);
        config.environment = env;
        let (tx, rx) = crossbeam_channel::unbounded();
        let pending = Arc::new(AtomicBool::new(false));
        let sink = Sink {
            key: self.key,
            tx,
            pending: pending.clone(),
        };
        *self.rx.borrow_mut() = Some(rx);
        *self.pending.borrow_mut() = pending;
        let Some(text_system) = self.text_system.borrow().clone() else {
            log::error!("cannot start the engine: the surface is not in a GPUI app yet");
            return;
        };
        let family = crate::text::ui_family();
        let theme = self.host_theme.borrow().clone();
        match Engine::start_with(
            config,
            move || GpuiTextEngine::new(text_system, family),
            sink,
            |control| {
                if theme.is_some() {
                    control.set_theme(theme);
                }
            },
        ) {
            Ok(engine) => {
                *self.control.borrow_mut() = Some(engine.control());
                *self.engine.borrow_mut() = Some(engine);
            }
            Err(err) => log::error!("cannot start the engine: {err}"),
        }
    }

    /// Stops the app and removes its views.
    pub(crate) fn stop(&self) {
        // Dropping the engine joins its thread.
        self.control.borrow_mut().take();
        self.engine.borrow_mut().take();
        self.rx.borrow_mut().take();
        self.mounter.borrow_mut().clear();
        self.native.clear();
        crate::system::clear(self);
        *self.anim.borrow_mut() = UiAnimation::default();
        self.nav.borrow_mut().reset();
        crate::input::reset(self);
        crate::keymap::clear(self);
        self.js_frame.set(false);
        self.frame_clients.borrow_mut().clear();
        self.invalidate();
    }

    /// Calls `f` with the engine's control, if an app runs.
    pub(crate) fn control(&self, f: impl FnOnce(&EngineControl)) {
        if let Some(control) = &*self.control.borrow() {
            f(control);
        }
    }

    /// Sends an event to JS.
    pub(crate) fn event(&self, id: NodeId, kind: EventKind, payload: EventPayload) {
        self.control(|c| c.dispatch_event(id, kind, payload));
    }

    /// The view of node `id`.
    pub(crate) fn view(&self, id: NodeId) -> Option<NshView> {
        self.views.get(id)
    }

    /// Shows fatal app errors with `f` (development hosts, embedding apps).
    pub(crate) fn set_error_handler(&self, f: ErrorHandler) {
        *self.on_error.borrow_mut() = Some(f);
    }

    /// Runs `f` when a hot update could not be applied in place.
    #[cfg(feature = "dev")]
    pub(crate) fn set_hot_update_failed(&self, f: impl Fn() + 'static) {
        *self.on_hot_update_failed.borrow_mut() = Some(Box::new(f));
    }

    /// Updates the environment and tells the engine.
    pub(crate) fn set_environment(&self, environment: Environment) {
        if self.environment.replace(environment) != environment {
            self.control(|c| c.set_environment(environment));
            crate::system::environment_applied(self);
        }
    }

    /// The surface is drawn `width` × `height` points big at `scale`
    /// pixels per point.
    pub(crate) fn resized(self: &Rc<Self>, width: f32, height: f32, scale: f32) {
        let scale_changed = (self.scale.replace(scale) - scale).abs() > f32::EPSILON;
        {
            let mut s = self.root.state_mut();
            if !scale_changed && s.frame.width == width && s.frame.height == height {
                return;
            }
            s.frame.width = width;
            s.frame.height = height;
        }
        let mut env = self.environment.get();
        env.window = (width, height);
        env.pixel_ratio = scale;
        self.environment.set(env);
        self.control(|c| {
            c.resize(Size::new(width, height));
            c.set_environment(env);
        });
        crate::navigation::surface_resized(self, width, height);
        crate::system::surface_resized(self, width, height);
        for f in self.on_resized.borrow().iter() {
            f(width, height);
        }
        self.invalidate();
    }

    /// Asks for the surface to be drawn again (once, however often asked
    /// before it is): [`flush_all`] notifies its view.
    pub(crate) fn invalidate(&self) {
        self.invalid.set(true);
    }

    /// The surface is being drawn: what changed so far is in this frame.
    pub(crate) fn drawing(&self) {
        self.invalid.set(false);
    }

    /// Hosted items drawn by the frame just drawn, with their window
    /// bounds: tells them, and the ones no longer drawn.
    pub(crate) fn items_placed(&self, drawn: Vec<(Rc<dyn Hosted>, gpui::Bounds<gpui::Pixels>)>) {
        let mut placed = self.placed.borrow_mut();
        for old in placed.iter() {
            if !drawn.iter().any(|(item, _)| Rc::ptr_eq(item, old)) {
                old.placed(None);
            }
        }
        placed.clear();
        for (item, rect) in drawn {
            item.placed(Some(rect));
            placed.push(item);
        }
    }

    /// Applies what the engine sent.
    pub(crate) fn drain(self: &Rc<Self>) {
        self.pending.borrow().store(false, Ordering::Release);
        let messages: Vec<Message> = match &*self.rx.borrow() {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };
        let mut applied = false;
        for message in messages {
            match message {
                Message::Batch(batch) => {
                    self.apply(&batch);
                    applied = true;
                }
                Message::Error(message) => {
                    log::error!("{message}");
                    if let Some(f) = &*self.on_error.borrow() {
                        f(&message);
                    }
                }
                Message::Frame => self.js_frame.set(true),
                Message::HotUpdateFailed => {
                    if let Some(f) = &*self.on_hot_update_failed.borrow() {
                        f();
                    }
                }
            }
        }
        if applied {
            self.write_animations();
        }
        self.ensure_ticking();
    }

    fn apply(self: &Rc<Self>, batch: &MountBatch) {
        crate::access::prepare(batch);
        for item in &batch.items {
            self.anim.borrow_mut().apply(item);
            if !matches!(
                item,
                MountItem::Animation(_) | MountItem::AnimationBindings { .. }
            ) {
                self.mounter.borrow_mut().apply(self, item);
            }
        }
        self.mounter.borrow_mut().end_batch(self);
        crate::access::batch_applied(self);
    }

    /// Writes the animated values to the views and reports finished
    /// animations.
    pub(crate) fn write_animations(&self) {
        let finished = {
            let mut anim = self.anim.borrow_mut();
            anim.write(&self.mounter.borrow());
            std::mem::take(&mut anim.finished)
        };
        if !finished.is_empty() {
            self.control(|c| c.animations_finished(finished));
        }
    }

    /// Calls `f` on every frame (with the frame's time) until it returns
    /// false: transitions, scroll glides, spinners.
    pub(crate) fn on_frame(self: &Rc<Self>, f: impl FnMut(&Rc<Surface>, f64) -> bool + 'static) {
        self.frame_clients.borrow_mut().push(Box::new(f));
        self.ensure_ticking();
    }

    /// Something animates or JS wants a frame: the view keeps drawing
    /// frames ([`crate::render`]).
    pub(crate) fn wants_frames(&self) -> bool {
        self.js_frame.get()
            || self.anim.borrow().animating()
            || !self.frame_clients.borrow().is_empty()
    }

    /// Draws frames while animations run or JS wants a frame.
    pub(crate) fn ensure_ticking(self: &Rc<Self>) {
        if self.wants_frames() {
            self.invalidate();
        }
    }

    /// One animation frame (called while the surface is drawn, before its
    /// views are).
    pub(crate) fn tick(self: &Rc<Self>) {
        let now = self.clock();
        self.now.set(now);
        if self.anim.borrow().animating() {
            self.anim.borrow_mut().tick(now);
            self.write_animations();
        }
        if self.js_frame.replace(false) {
            self.control(|c| c.frame());
        }
        let mut clients = std::mem::take(&mut *self.frame_clients.borrow_mut());
        clients.retain_mut(|f| f(self, now));
        // Clients added while running stay after the ones kept.
        let mut added = std::mem::take(&mut *self.frame_clients.borrow_mut());
        clients.append(&mut added);
        *self.frame_clients.borrow_mut() = clients;
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        let key = self.key;
        let _ = SURFACES.try_with(|s| s.borrow_mut().remove(&key));
    }
}

/// Notifies the views of this thread's surfaces that changed.
pub(crate) fn flush_all(cx: &mut gpui::App) {
    let ids: Vec<gpui::EntityId> = SURFACES.with(|s| {
        s.borrow()
            .values()
            .filter_map(Weak::upgrade)
            .filter(|surface| surface.invalid.get())
            .filter_map(|surface| surface.entity.get())
            .collect()
    });
    for id in ids {
        cx.notify(id);
    }
}
