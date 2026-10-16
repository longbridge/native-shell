//! Async tasks of components ([`super::Context::spawn`]): futures run on the
//! engine thread (they may hold the component's `!Send` state) by a small
//! executor of the core, polled inside a tokio runtime's context, so the
//! tokio ecosystem works in them: `tokio::time` for timers, network and
//! file IO, channels; `Send` work goes to the runtime's threads
//! ([`AsyncCx::background`]). Wakers may fire on any thread: they wake the
//! engine. A task changes its component's state with
//! [`WeakComponent::update`], applied by the core between polls (no shared
//! borrows of the component). Dropping a [`Task`] cancels it; a
//! component's tasks end with it.

use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    future::Future,
    marker::PhantomData,
    pin::Pin,
    rc::{Rc, Weak},
    sync::{Arc, Mutex},
    task::{Context as TaskContext, Poll, Wake, Waker},
    time::Duration,
};

use rustc_hash::FxHashMap;
use tokio::runtime::Handle;

use super::{Component, Context, Prefetch, ToastDuration, element::ErasedCx, services};
use crate::{Command, Dialog, JsonValue, NodeId, ROOT_ID, StatusBarSetting, time::Instant};

type LocalFuture = Pin<Box<dyn Future<Output = ()>>>;

/// Called when a task is woken (from any thread): the engine wakes up.
pub(crate) type Notify = Arc<dyn Fn() + Send + Sync>;

/// Tasks woken since the last run.
#[derive(Default)]
struct Ready {
    ids: Mutex<Vec<u64>>,
    notify: Mutex<Option<Notify>>,
}

struct TaskWaker {
    id: u64,
    ready: Arc<Ready>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.ready.ids.lock().expect("ready lock").push(self.id);
        let notify = self.ready.notify.lock().expect("notify lock").clone();
        if let Some(notify) = notify {
            notify();
        }
    }
}

/// A change of a component's state a task asked for; `None` when the
/// component is gone.
pub(crate) type UpdateFn =
    Box<dyn for<'a> FnOnce(Option<(&'a mut dyn Any, &'a dyn Any, &'a mut ErasedCx)>)>;

pub(crate) struct Update {
    pub(crate) host: NodeId,
    pub(crate) run: UpdateFn,
}

/// What a component or a task asks of the platform, applied by the core.
pub(crate) enum Action {
    Command(NodeId, Command),
    /// A dialog; its answer resolves the slot.
    Dialog(Dialog, Rc<RefCell<Output<i32>>>),
    /// The result of a module function JS waits for.
    Return(u64, Result<JsonValue, String>),
    /// The pixel size of the image at the URI (its bytes stay loaded); the
    /// answer resolves the slot.
    ImageInfo(String, AnswerSlot<Result<ImageSize, String>>),
    /// Reads the system clipboard; the answer resolves the slot.
    ReadClipboard(ClipboardRead),
    /// An operation on the native animation graph.
    Animation(nsh_animation::AnimationOp),
    /// Starts an animation; its end resolves the slot.
    Animate {
        op: nsh_animation::AnimationOp,
        animation: nsh_animation::AnimationId,
        end: AnswerSlot<bool>,
    },
    /// A component's own list reports the rows viewable now (they stayed
    /// so for its minimum view time).
    ReportViewable(NodeId),
    /// The start of the next display frame resolves the slot.
    Frame(AnswerSlot<Instant>),
    /// Changes the layout direction's policy.
    Direction(super::services::Direction),
    /// Sets the app's appearance.
    Appearance(crate::platform::Appearance),
}

/// A clipboard read waiting for the platform's answer.
pub(crate) enum ClipboardRead {
    /// The text (`""` when it holds none).
    Text(AnswerSlot<String>),
    /// Whether it holds text.
    Has(AnswerSlot<bool>),
}

/// An image's size in pixels (`Image.getSize`).
#[derive(Debug, Clone, Copy, Default, PartialEq, super::JsObject)]
pub struct ImageSize {
    /// Width.
    pub width: f64,
    /// Height.
    pub height: f64,
}

/// The platform services components and tasks use ([`super::Context`],
/// [`AsyncCx`]): commands for the surface's root.
pub(crate) fn status_bar(patch: StatusBarSetting) -> Action {
    Action::Command(ROOT_ID, Command::SetStatusBar(patch))
}

struct Entry {
    host: NodeId,
    /// Out while it is polled.
    future: Option<LocalFuture>,
}

#[derive(Default)]
struct Inner {
    tasks: FxHashMap<u64, Entry>,
    next_task: u64,
    updates: VecDeque<Update>,
    actions: VecDeque<Action>,
    failures: Vec<(NodeId, String)>,
}

/// The executor of one core's component tasks.
#[derive(Default)]
pub(crate) struct Executor {
    inner: RefCell<Inner>,
    ready: Arc<Ready>,
    /// The runtime tasks are polled in the context of.
    runtime: RefCell<Option<Handle>>,
    /// Native animation nodes and animations made so far (their ids are
    /// odd: JS's are even).
    animation_ids: std::cell::Cell<u32>,
    /// Live derived nodes, for renders deriving them again.
    derived: RefCell<super::animated::Derived>,
    /// The components' listeners of the back button.
    pub(crate) back: Rc<RefCell<super::back::BackListeners>>,
}

impl Executor {
    /// The live node `key` (a node-making op, node 0) made.
    pub(crate) fn derived(
        &self,
        key: &nsh_animation::AnimationOp,
    ) -> Option<Rc<super::animated::Node>> {
        self.derived.borrow_mut().get(key)
    }

    pub(crate) fn remember_derived(
        &self,
        key: nsh_animation::AnimationOp,
        node: &Rc<super::animated::Node>,
    ) {
        self.derived.borrow_mut().0.push((key, Rc::downgrade(node)));
    }

    /// A new id for a native animation node or animation.
    pub(crate) fn next_animation_id(&self) -> u32 {
        let n = self.animation_ids.get();
        self.animation_ids.set(n + 1);
        2 * n + 1
    }

    pub(crate) fn set_runtime(&self, runtime: Handle) {
        *self.runtime.borrow_mut() = Some(runtime);
    }

    pub(crate) fn set_notify(&self, notify: Notify) {
        *self.ready.notify.lock().expect("notify lock") = Some(notify);
    }

    fn spawn(&self, host: NodeId, future: LocalFuture) -> u64 {
        let id = {
            let mut inner = self.inner.borrow_mut();
            inner.next_task += 1;
            let id = inner.next_task;
            inner.tasks.insert(
                id,
                Entry {
                    host,
                    future: Some(future),
                },
            );
            id
        };
        // Polled at the next run, not inside the code that spawned it.
        self.ready.ids.lock().expect("ready lock").push(id);
        let notify = self.ready.notify.lock().expect("notify lock").clone();
        if let Some(notify) = notify {
            notify();
        }
        id
    }

    fn cancel(&self, id: u64) {
        let removed = self.inner.borrow_mut().tasks.remove(&id);
        // Dropped outside the borrow: the future may own other tasks.
        drop(removed);
    }

    /// Cancels the tasks of component `host`.
    pub(crate) fn cancel_host(&self, host: NodeId) {
        let removed: Vec<Entry> = {
            let mut inner = self.inner.borrow_mut();
            let ids: Vec<u64> = inner
                .tasks
                .iter()
                .filter(|(_, e)| e.host == host)
                .map(|(&id, _)| id)
                .collect();
            ids.iter().filter_map(|id| inner.tasks.remove(id)).collect()
        };
        drop(removed);
    }

    /// Polls the woken tasks; returns whether there were any.
    pub(crate) fn poll_ready(&self) -> bool {
        let mut ids = std::mem::take(&mut *self.ready.ids.lock().expect("ready lock"));
        if ids.is_empty() {
            return false;
        }
        ids.sort_unstable();
        ids.dedup();
        // tokio's timers, IO and `spawn` find their runtime here.
        let runtime = self.runtime.borrow().clone();
        let _context = runtime.as_ref().map(Handle::enter);
        for id in ids {
            let future = match self.inner.borrow_mut().tasks.get_mut(&id) {
                Some(entry) => entry.future.take(),
                None => None,
            };
            let Some(mut future) = future else { continue };
            let waker = Waker::from(Arc::new(TaskWaker {
                id,
                ready: self.ready.clone(),
            }));
            let done = match super::component::caught(|| {
                future
                    .as_mut()
                    .poll(&mut TaskContext::from_waker(&waker))
                    .is_ready()
            }) {
                Ok(done) => done,
                // A task that panicked ends; its component failed.
                Err(message) => {
                    let mut inner = self.inner.borrow_mut();
                    if let Some(entry) = inner.tasks.get(&id) {
                        let host = entry.host;
                        inner.failures.push((host, message));
                    }
                    true
                }
            };
            if done {
                self.inner.borrow_mut().tasks.remove(&id);
                drop(future);
            } else if let Some(entry) = self.inner.borrow_mut().tasks.get_mut(&id) {
                entry.future = Some(future);
            }
        }
        true
    }

    #[cfg(test)]
    pub(crate) fn task_count(&self) -> usize {
        self.inner.borrow().tasks.len()
    }

    /// The tasks that panicked since, by component.
    pub(crate) fn take_failures(&self) -> Vec<(NodeId, String)> {
        std::mem::take(&mut self.inner.borrow_mut().failures)
    }

    /// The state changes tasks asked for.
    pub(crate) fn take_updates(&self) -> VecDeque<Update> {
        std::mem::take(&mut self.inner.borrow_mut().updates)
    }

    /// What tasks asked of the platform.
    pub(crate) fn take_actions(&self) -> VecDeque<Action> {
        std::mem::take(&mut self.inner.borrow_mut().actions)
    }

    pub(crate) fn act(&self, action: Action) {
        self.inner.borrow_mut().actions.push_back(action);
        let notify = self.ready.notify.lock().expect("notify lock").clone();
        if let Some(notify) = notify {
            notify();
        }
    }
}

/// A running task ([`super::Context::spawn`]). Dropping it cancels the
/// task (TS's `clearTimeout`, for timers); [`Task::detach`] lets it run
/// on its own. Awaited in another task, it gives the task's output.
#[must_use = "a dropped task is cancelled: keep it, or detach it"]
pub struct Task<T> {
    executor: Weak<Executor>,
    id: u64,
    output: Rc<RefCell<Output<T>>>,
    detached: bool,
}

/// A value a future waits for, and the waker to call when it comes.
pub(crate) struct Output<T> {
    pub(crate) value: Option<T>,
    pub(crate) waker: Option<Waker>,
}

/// Where an [`Answer`] comes from.
pub(crate) type AnswerSlot<T> = Rc<RefCell<Output<T>>>;

impl<T> Default for Output<T> {
    fn default() -> Self {
        Self {
            value: None,
            waker: None,
        }
    }
}

impl<T> Output<T> {
    /// Delivers `value`, waking the future waiting for it.
    pub(crate) fn deliver(&mut self, value: T) {
        self.value = Some(value);
        if let Some(waker) = self.waker.take() {
            waker.wake();
        }
    }
}

/// The answer the platform gives to a request (a dialog, an image's size),
/// awaited in a task.
#[must_use = "an answer does nothing unless awaited"]
pub struct Answer<T> {
    slot: Rc<RefCell<Output<T>>>,
}

/// The answer to a dialog ([`super::Context::dialog`]): the index of the
/// button pressed, -1 when it was dismissed.
pub type DialogAnswer = Answer<i32>;

impl<T> Answer<T> {
    pub(crate) fn new() -> (Self, Rc<RefCell<Output<T>>>) {
        let slot = Rc::new(RefCell::new(Output::default()));
        (Self { slot: slot.clone() }, slot)
    }
}

impl<T> std::fmt::Debug for Answer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Answer").finish_non_exhaustive()
    }
}

impl<T> Future for Answer<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<T> {
        let mut slot = self.slot.borrow_mut();
        match slot.value.take() {
            Some(value) => Poll::Ready(value),
            None => {
                slot.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl<T> std::fmt::Debug for Task<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

impl<T> Task<T> {
    /// Lets the task run to its end without this handle (it still ends
    /// with its component).
    pub fn detach(mut self) {
        self.detached = true;
    }
}

impl<T> Drop for Task<T> {
    fn drop(&mut self) {
        if !self.detached
            && let Some(executor) = self.executor.upgrade()
        {
            executor.cancel(self.id);
        }
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<T> {
        let mut output = self.output.borrow_mut();
        match output.value.take() {
            Some(value) => Poll::Ready(value),
            None => {
                output.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// What a task's code gets besides its component: timers and background
/// work on tokio (any tokio API works in a task as well), and the
/// platform's services.
#[derive(Clone, Debug)]
pub struct AsyncCx {
    executor: Weak<Executor>,
}

impl AsyncCx {
    pub(crate) fn new(executor: Weak<Executor>) -> Self {
        Self { executor }
    }

    pub(crate) fn act(&self, action: Action) {
        if let Some(executor) = self.executor.upgrade() {
            executor.act(action);
        }
    }

    /// Shows `dialog` (an alert, or with `sheet` an action sheet).
    pub fn dialog(&self, dialog: Dialog) -> DialogAnswer {
        let (answer, slot) = DialogAnswer::new();
        self.act(Action::Dialog(dialog, slot));
        answer
    }

    /// Unfocuses the focused input, hiding the software keyboard.
    pub fn dismiss_keyboard(&self) {
        self.act(Action::Command(ROOT_ID, Command::Blur));
    }

    /// Changes what functions set for the status bar (the fields `patch`
    /// sets), under the settings components render.
    pub fn set_status_bar(&self, patch: StatusBarSetting) {
        self.act(status_bar(patch));
    }

    /// The screen reader reads `message` (TS:
    /// `AccessibilityInfo.announceForAccessibility`).
    pub fn announce_for_accessibility(&self, message: impl Into<String>) {
        self.act(services::announce(message.into()));
    }

    /// Shows `message` at the bottom of the screen (Android's toasts, TS:
    /// `Toast.show`); nothing happens elsewhere.
    pub fn toast(&self, message: impl Into<String>, duration: ToastDuration) {
        self.act(services::toast(message.into(), duration));
    }

    /// The size in pixels of the image at `uri` (TS: `Image.getSize`); its
    /// bytes stay loaded for the views that show it next.
    pub fn image_size(&self, uri: impl Into<String>) -> Answer<Result<ImageSize, String>> {
        let (answer, action) = services::image_size(uri.into());
        self.act(action);
        answer
    }

    /// Loads the image at `uri` ahead of showing it (TS: `Image.prefetch`).
    pub fn prefetch_image(&self, uri: impl Into<String>) -> Prefetch {
        let (answer, action) = services::prefetch(uri.into());
        self.act(action);
        answer
    }

    /// Puts `text` on the system clipboard (TS: `Clipboard.setString`).
    pub fn set_clipboard(&self, text: impl Into<String>) {
        self.act(services::set_clipboard(text.into()));
    }

    /// The text on the system clipboard, `""` when it holds none (TS:
    /// `Clipboard.getString`). On iOS, reading it may show the system's
    /// paste notice.
    pub fn clipboard_string(&self) -> Answer<String> {
        let (answer, slot) = Answer::new();
        self.act(Action::ReadClipboard(ClipboardRead::Text(slot)));
        answer
    }

    /// Whether the system clipboard holds text, without reading it (TS:
    /// `Clipboard.hasString`).
    pub fn clipboard_has_string(&self) -> Answer<bool> {
        let (answer, slot) = Answer::new();
        self.act(Action::ReadClipboard(ClipboardRead::Has(slot)));
        answer
    }

    /// Completes when the next display frame starts, with its time (TS:
    /// what `requestAnimationFrame` waits for). Awaited in a loop, it runs
    /// once per frame.
    pub fn next_frame(&self) -> Answer<Instant> {
        let (answer, action) = services::frame();
        self.act(action);
        answer
    }

    /// Shows the surface in `appearance` whatever the system's (TS:
    /// `setColorScheme`): [`crate::ui::RenderCx::dark`] (the theme's
    /// mode), `dark:` classes and the platform's controls follow it;
    /// [`crate::platform::Appearance::System`] follows the system again.
    pub fn set_color_scheme(&self, appearance: crate::platform::Appearance) {
        self.act(Action::Appearance(appearance));
    }

    /// Lays the surface out right to left whatever the user's language
    /// (TS: `I18nManager.forceRTL`), at once; `false` follows the language
    /// again.
    pub fn force_rtl(&self, force: bool) {
        self.act(Action::Direction(services::Direction::Force(force)));
    }

    /// Whether a right-to-left language lays the surface out right to left
    /// (TS: `I18nManager.allowRTL`; true by default).
    pub fn allow_rtl(&self, allow: bool) {
        self.act(Action::Direction(services::Direction::Allow(allow)));
    }

    /// Completes after `delay` (TS: what `setTimeout` waits for):
    /// `tokio::time::sleep`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn timer(&self, delay: Duration) -> tokio::time::Sleep {
        tokio::time::sleep(delay)
    }

    /// Completes at `at`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn timer_until(&self, at: Instant) -> tokio::time::Sleep {
        tokio::time::sleep_until(at.into())
    }

    /// Completes after `delay` (TS: what `setTimeout` waits for).
    #[cfg(target_arch = "wasm32")]
    pub fn timer(&self, delay: Duration) -> crate::time::Sleep {
        crate::time::sleep(delay)
    }

    /// Completes at `at`.
    #[cfg(target_arch = "wasm32")]
    pub fn timer_until(&self, at: Instant) -> crate::time::Sleep {
        crate::time::sleep_until(at)
    }

    /// Runs `future` on the runtime's threads (`tokio::spawn`): `Send`
    /// work that should not hold the engine thread. Await the handle for
    /// its output. Not in WebAssembly, which has no threads.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn background<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(future)
    }

    /// Runs blocking `f` on the runtime's blocking threads
    /// (`tokio::task::spawn_blocking`). Not in WebAssembly.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn blocking<R: Send + 'static>(
        &self,
        f: impl FnOnce() -> R + Send + 'static,
    ) -> tokio::task::JoinHandle<R> {
        tokio::task::spawn_blocking(f)
    }
}

/// A component as its tasks see it: it may be gone by the time they run.
pub struct WeakComponent<C> {
    host: NodeId,
    executor: Weak<Executor>,
    _component: PhantomData<fn(&mut C)>,
}

impl<C> std::fmt::Debug for WeakComponent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakComponent")
            .field("host", &self.host)
            .finish_non_exhaustive()
    }
}

impl<C> Clone for WeakComponent<C> {
    fn clone(&self) -> Self {
        Self {
            host: self.host,
            executor: self.executor.clone(),
            _component: PhantomData,
        }
    }
}

impl<C: Component> WeakComponent<C> {
    pub(crate) fn new(host: NodeId, executor: Weak<Executor>) -> Self {
        Self {
            host,
            executor,
            _component: PhantomData,
        }
    }

    /// Changes the component's state: `f` runs on the engine thread right
    /// after this poll, with the component and a [`Context`]
    /// (`cx.notify()` renders it again). Await it for `f`'s result, `None`
    /// when the component is gone; not awaited, it still runs.
    pub fn update<R: 'static>(
        &self,
        f: impl FnOnce(&mut C, &mut Context<'_, C>) -> R + 'static,
    ) -> Updated<R> {
        let slot = Rc::new(RefCell::new(Output {
            value: None,
            waker: None,
        }));
        let result = slot.clone();
        if let Some(executor) = self.executor.upgrade() {
            executor.inner.borrow_mut().updates.push_back(Update {
                host: self.host,
                run: Box::new(move |target| {
                    let value = target.and_then(|(state, props, cx)| {
                        let props = props.downcast_ref::<C::Props>()?;
                        let state = state.downcast_mut::<C>()?;
                        Some(f(state, &mut Context::new(cx, props)))
                    });
                    let mut slot = result.borrow_mut();
                    slot.value = Some(value);
                    if let Some(waker) = slot.waker.take() {
                        waker.wake();
                    }
                }),
            });
        } else {
            slot.borrow_mut().value = Some(None);
        }
        Updated { slot }
    }
}

/// An update's result ([`WeakComponent::update`]).
#[must_use = "the update runs anyway; await it for its result"]
pub struct Updated<R> {
    slot: Rc<RefCell<Output<Option<R>>>>,
}

impl<R> std::fmt::Debug for Updated<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Updated").finish_non_exhaustive()
    }
}

impl<R> Future for Updated<R> {
    type Output = Option<R>;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<R>> {
        let mut slot = self.slot.borrow_mut();
        match slot.value.take() {
            Some(value) => Poll::Ready(value),
            None => {
                slot.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

/// Starts `future` as a task of `host`, to run to its end.
pub(crate) fn spawn_raw(executor: &Rc<Executor>, host: NodeId, future: LocalFuture) -> u64 {
    executor.spawn(host, future)
}

/// Applies `action` after `delay`, as a task of `host` (dropping the task
/// cancels it).
pub(crate) fn act_after(
    executor: &Rc<Executor>,
    host: NodeId,
    delay: Duration,
    action: Action,
) -> Task<()> {
    let weak = Rc::downgrade(executor);
    let cx = AsyncCx::new(weak.clone());
    let id = executor.spawn(
        host,
        Box::pin(async move {
            tokio::time::sleep(delay).await;
            cx.act(action);
        }),
    );
    Task {
        executor: weak,
        id,
        output: Rc::new(RefCell::new(Output::default())),
        detached: false,
    }
}

/// Starts `f(this, cx)` as a task of component `host`.
pub(crate) fn spawn<C, R, Fut>(
    executor: &Rc<Executor>,
    host: NodeId,
    f: impl FnOnce(WeakComponent<C>, AsyncCx) -> Fut,
) -> Task<R>
where
    C: Component,
    R: 'static,
    Fut: Future<Output = R> + 'static,
{
    let weak = Rc::downgrade(executor);
    let this = WeakComponent {
        host,
        executor: weak.clone(),
        _component: PhantomData,
    };
    let future = f(this, AsyncCx::new(weak.clone()));
    let output = Rc::new(RefCell::new(Output {
        value: None,
        waker: None,
    }));
    let done = output.clone();
    let id = executor.spawn(
        host,
        Box::pin(async move {
            let value = future.await;
            let mut output = done.borrow_mut();
            output.value = Some(value);
            if let Some(waker) = output.waker.take() {
                waker.wake();
            }
        }),
    );
    Task {
        executor: weak,
        id,
        output,
        detached: false,
    }
}
