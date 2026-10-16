//! One surface: an engine thread and the views it mounts on the main
//! thread.
//!
//! The engine thread hands each commit to [`Sink`], which queues it and
//! wakes the main thread once (`DispatchQueue::main`); the main thread
//! drains the queue and applies every batch in order ([`Surface::drain`]).
//! Native animations and JS frame requests run on the root view's display
//! link while there is something to do.

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crossbeam_channel::{Receiver, Sender};
use dispatch2::DispatchQueue;
use nsh_core::{EventKind, EventPayload, MountBatch, MountItem, NodeId, ROOT_ID, Size};
use nsh_engine::{Engine, EngineConfig, EngineControl, Environment, MountSink};
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained, sel,
};
use objc2_foundation::{NSObject, NSObjectProtocol};
use objc2_quartz_core::{CADisplayLink, CATransaction};
use rustc_hash::FxHashMap;

use crate::{animation::UiAnimation, mount::Mounter, text::CoreTextEngine, view::NshView};

/// What the engine thread sends the main thread.
enum Message {
    Batch(MountBatch),
    Error(String),
    Frame,
    HotUpdateFailed,
}

thread_local! {
    /// The surfaces of this (main) thread, by key: the sink wakes one by
    /// its key, as dispatched closures must be `Send`.
    static SURFACES: RefCell<FxHashMap<u64, Weak<Surface>>> = RefCell::default();
}

static NEXT_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// The surface with `key`, if it still exists.
fn find(key: u64) -> Option<Rc<Surface>> {
    SURFACES.with(|s| s.borrow().get(&key).and_then(Weak::upgrade))
}

/// Forwards batches from the engine thread to the main thread.
struct Sink {
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
            DispatchQueue::main().exec_async(move || {
                if let Some(surface) = find(key) {
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

define_class!(
    /// The display link's target: ticks the surface with its key.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = u64]
    #[name = "NshTicker"]
    struct Ticker;

    unsafe impl NSObjectProtocol for Ticker {}

    impl Ticker {
        #[unsafe(method(tick:))]
        fn tick(&self, link: &CADisplayLink) {
            match find(*self.ivars()) {
                Some(surface) => surface.ticked(link),
                None => link.invalidate(),
            }
        }
    }
);

/// Called when the app reports a fatal error (development hosts show it).
pub(crate) type ErrorHandler = Box<dyn Fn(&str)>;

/// One engine and its views.
pub(crate) struct Surface {
    key: u64,
    pub(crate) mtm: MainThreadMarker,
    /// The root view (node [`ROOT_ID`]): the surface's view.
    pub(crate) root: NshView,
    /// Views by node id, and what mounting needs between items.
    pub(crate) mounter: RefCell<Mounter>,
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
    /// The environment the engine was last told about.
    pub(crate) environment: Cell<Environment>,
    /// The app's appearance (`nsh.json`, `setColorScheme`): the color
    /// scheme is the system's resolved with it ([`crate::system`]).
    pub(crate) appearance: Cell<nsh_core::platform::Appearance>,
    /// JS asked for a display frame.
    js_frame: Cell<bool>,
    /// Frame callbacks of the platform's own animations (scroll glides,
    /// transitions): called on every frame while they return true.
    frame_callbacks: RefCell<Vec<FrameCallback>>,
    /// The display link, while it runs.
    link: RefCell<Option<Retained<CADisplayLink>>>,
    on_error: RefCell<Option<ErrorHandler>>,
    on_hot_update_failed: RefCell<Option<Box<dyn Fn()>>>,
    /// Where `asset:/` image URIs resolve ([`crate::image::set_asset_base`]).
    pub(crate) asset_base: RefCell<Option<crate::image::AssetBase>>,
    /// Videos, web views and registered native components.
    pub(crate) native: crate::native::NativeState,
}

/// A per-frame callback ([`Surface::add_frame_callback`]): gets the frame
/// time in seconds, returns whether it wants the next frame.
pub(crate) type FrameCallback = Box<dyn FnMut(&Rc<Surface>, f64) -> bool>;

impl Surface {
    /// A surface with no app yet.
    pub(crate) fn new(mtm: MainThreadMarker, environment: Environment) -> Rc<Surface> {
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);
        let root = NshView::new(mtm, ROOT_ID, nsh_core::ViewKind::View);
        let surface = Rc::new(Surface {
            key,
            mtm,
            root: root.clone(),
            mounter: RefCell::new(Mounter::new(root.clone())),
            anim: RefCell::new(UiAnimation::default()),
            nav: RefCell::default(),
            input: Default::default(),
            engine: RefCell::new(None),
            control: RefCell::new(None),
            rx: RefCell::new(None),
            pending: RefCell::new(Arc::new(AtomicBool::new(false))),
            environment: Cell::new(environment),
            appearance: Cell::default(),
            js_frame: Cell::new(false),
            frame_callbacks: RefCell::default(),
            link: RefCell::new(None),
            on_error: RefCell::new(None),
            on_hot_update_failed: RefCell::new(None),
            asset_base: RefCell::new(None),
            native: Default::default(),
        });
        SURFACES.with(|s| s.borrow_mut().insert(key, Rc::downgrade(&surface)));
        root.set_surface(Rc::downgrade(&surface));
        surface
    }

    /// The surface's key ([`Surface::with_key`]).
    pub(crate) fn key(&self) -> u64 {
        self.key
    }

    /// The surface with `key`, for callbacks that must be `Send`.
    pub(crate) fn with_key(key: u64) -> Option<Rc<Surface>> {
        find(key)
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
        match Engine::start(config, CoreTextEngine::default, sink) {
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
        self.js_frame.set(false);
        self.frame_callbacks.borrow_mut().clear();
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

    /// Shows fatal app errors with `f` (development hosts).
    #[cfg(feature = "dev")]
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

    /// The root view got a new size (the window's content area).
    pub(crate) fn resized(&self, width: f32, height: f32) {
        {
            let mut s = self.root.state_mut();
            if s.frame.width == width && s.frame.height == height {
                return;
            }
            s.frame.width = width;
            s.frame.height = height;
        }
        let mut env = self.environment.get();
        env.window = (width, height);
        self.environment.set(env);
        self.control(|c| {
            c.resize(Size::new(width, height));
            c.set_environment(env);
        });
        crate::navigation::surface_resized(self, width, height);
        crate::system::surface_resized(self, width, height);
    }

    /// Applies what the engine sent.
    fn drain(self: &Rc<Self>) {
        self.pending.borrow().store(false, Ordering::Release);
        let messages: Vec<Message> = match &*self.rx.borrow() {
            Some(rx) => rx.try_iter().collect(),
            None => return,
        };
        let mut applied = false;
        quiet(|| {
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
        });
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

    /// Calls `f` on every display frame while it returns true.
    pub(crate) fn add_frame_callback(
        self: &Rc<Self>,
        f: impl FnMut(&Rc<Surface>, f64) -> bool + 'static,
    ) {
        self.frame_callbacks.borrow_mut().push(Box::new(f));
        self.ensure_ticking();
    }

    fn wants_frames(&self) -> bool {
        self.js_frame.get()
            || self.anim.borrow().animating()
            || !self.frame_callbacks.borrow().is_empty()
    }

    /// Runs the display link while animations run or JS wants a frame.
    pub(crate) fn ensure_ticking(self: &Rc<Self>) {
        if !self.wants_frames() {
            return;
        }
        if let Some(link) = &*self.link.borrow() {
            link.setPaused(false);
            return;
        }
        let ticker = Ticker::alloc(self.mtm).set_ivars(self.key);
        // SAFETY: NSObject's initializer.
        let ticker: Retained<Ticker> = unsafe { msg_send![super(ticker), init] };
        // SAFETY: `tick:` takes the display link, as the selector requires;
        // the link retains its target.
        let link = unsafe {
            self.root
                .ns()
                .displayLinkWithTarget_selector(&ticker, sel!(tick:))
        };
        // SAFETY: the main run loop exists while the app runs.
        unsafe {
            link.addToRunLoop_forMode(
                &objc2_foundation::NSRunLoop::mainRunLoop(),
                objc2_foundation::NSRunLoopCommonModes,
            )
        };
        *self.link.borrow_mut() = Some(link);
    }

    fn ticked(self: &Rc<Self>, link: &CADisplayLink) {
        let now = link.timestamp();
        quiet(|| self.tick(now));
        if !self.wants_frames() {
            link.setPaused(true);
        }
    }

    fn tick(self: &Rc<Self>, now: f64) {
        if self.anim.borrow().animating() {
            self.anim.borrow_mut().tick(now);
            self.write_animations();
        }
        let callbacks = std::mem::take(&mut *self.frame_callbacks.borrow_mut());
        let mut kept: Vec<FrameCallback> = callbacks
            .into_iter()
            .filter_map(|mut f| f(self, now).then_some(f))
            .collect();
        {
            let mut added = self.frame_callbacks.borrow_mut();
            kept.append(&mut added);
            *added = kept;
        }
        if self.js_frame.replace(false) {
            self.control(|c| c.frame());
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        let key = self.key;
        if let Some(link) = self.link.borrow_mut().take() {
            link.invalidate();
        }
        let _ = SURFACES.try_with(|s| s.borrow_mut().remove(&key));
    }
}

/// Runs `f` with Core Animation's implicit animations off: layers change
/// at once, as the core said (animations are explicit).
pub(crate) fn quiet<R>(f: impl FnOnce() -> R) -> R {
    CATransaction::begin();
    CATransaction::setDisableActions(true);
    let r = f();
    CATransaction::commit();
    r
}
