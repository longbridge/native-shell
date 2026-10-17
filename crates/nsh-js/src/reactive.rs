//! The reactive graph JS runs on: signals, memos, effects, owners and
//! context, kept in Rust and called through QuickJS's C API.
//!
//! Reading a signal or a memo is a call of a native callable object (a few
//! nanoseconds, cheaper than calling a JS function); writing one marks its
//! dependants and runs the effects in Rust; only the user's closures run in
//! the interpreter. The scheduling follows Solid 1.9 (`runUpdates`, the
//! update and effect queues, stale owners updated before their children),
//! which the runtime and apps rely on.
//!
//! Lifetimes: a signal lives as long as its JS object (the object's GC mark
//! reports the value, so cycles through it are collected); memos and
//! effects belong to the owner they were made under and are disposed with
//! it; a memo's object keeps reading its last value after that. Nothing
//! here holds a Rust reference across a call into JS (which may re-enter)
//! or across freeing a JS value (which may finalize a signal or memo object,
//! whose finalizer updates the graph): the graph is reached through the
//! context's opaque pointer, every access is short, and values leave their
//! node before they are freed.

use std::{
    ffi::{CStr, c_int, c_void},
    ptr,
    rc::Rc,
};

#[cfg(test)]
use rquickjs::Ctx;
use rquickjs::qjs;

type JsValue = qjs::JSValue;
type JsContext = qjs::JSContext;

const UNDEFINED: JsValue = qjs::JS_UNDEFINED;
/// No node (no listener, no owner).
pub(crate) const NONE: u32 = u32::MAX;
/// The owner of nodes made in a root without `dispose`: never disposed.
pub(crate) const UNOWNED: u32 = u32::MAX - 1;

const CLEAN: u8 = 0;
const STALE: u8 = 1;
const PENDING: u8 = 2;

/// Runs of computations in one flush before it counts as an infinite loop.
const MAX_RUNS: u64 = 1_000_000;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    Signal,
    Memo,
    /// A render effect (runs as soon as possible, before user effects).
    Effect,
    /// A user effect (`createEffect`: after the render effects).
    UserEffect,
    Root,
    /// A freed slot.
    Free,
}

/// When a signal or memo counts as changed.
enum Equals {
    /// `===` (the default).
    Strict,
    /// Always (`equals: false`).
    Never,
    /// `equals(previous, next)`.
    Custom(JsValue),
    /// The same truthiness (a condition: `<If when>`).
    Truthiness,
}

/// A context value set by a provider, and the frame it shadows.
struct Frame {
    parent: Option<Rc<Frame>>,
    key: JsValue,
    value: JsValue,
    ctx: *mut JsContext,
}

impl Drop for Frame {
    fn drop(&mut self) {
        // SAFETY: frames only live in a graph, which is torn down before
        // its context.
        unsafe {
            qjs::JS_FreeValue(self.ctx, self.key);
            qjs::JS_FreeValue(self.ctx, self.value);
        }
    }
}

struct Node {
    kind: Kind,
    /// Bumped when the slot is reused: handles of the old node go stale.
    generation: u32,
    state: u8,
    value: JsValue,
    func: JsValue,
    equals: Equals,
    /// The exec round of the last run (`None`: never ran).
    updated_at: Option<u64>,
    /// (source, its index in the source's observers)
    sources: Vec<(u32, u32)>,
    /// (observer, its index in the observer's sources)
    observers: Vec<(u32, u32)>,
    /// The owner (for a root, the one it was made under: context and
    /// `runTop` look through it, but it does not own the root).
    owner: u32,
    owned: Vec<u32>,
    cleanups: Vec<JsValue>,
    context: Option<Rc<Frame>>,
    /// A JS object (signal, memo) refers to the node: its slot is freed
    /// when the object is collected.
    handle: bool,
    /// Disposed: runs no more (a memo keeps its value for its object).
    disposed: bool,
}

impl Node {
    fn new(kind: Kind) -> Self {
        Node {
            kind,
            generation: 1,
            state: CLEAN,
            value: UNDEFINED,
            func: UNDEFINED,
            equals: Equals::Strict,
            updated_at: None,
            sources: Vec::new(),
            observers: Vec::new(),
            owner: NONE,
            owned: Vec::new(),
            cleanups: Vec::new(),
            context: None,
            handle: false,
            disposed: false,
        }
    }

    fn is_computation(&self) -> bool {
        matches!(self.kind, Kind::Memo | Kind::Effect | Kind::UserEffect)
    }
}

/// JS values a node held, freed after the graph is no longer borrowed.
#[must_use]
struct Held {
    values: [JsValue; 3],
    context: Option<Rc<Frame>>,
}

impl Held {
    fn free(self, ctx: *mut JsContext) {
        for v in self.values {
            // SAFETY: values the node owned; no graph borrow is live.
            unsafe { qjs::JS_FreeValue(ctx, v) };
        }
        drop(self.context);
    }
}

/// The class ids of the graph's JS objects.
#[derive(Clone, Copy)]
struct Classes {
    signal: qjs::JSClassID,
    setter: qjs::JSClassID,
    memo: qjs::JSClassID,
}

pub(crate) struct Graph {
    ctx: *mut JsContext,
    classes: Classes,
    nodes: Vec<Node>,
    free: Vec<u32>,
    listener: u32,
    pub(crate) owner: u32,
    /// Solid's `Updates`: computations to run in this round (`None`: not
    /// in a round).
    updates: Option<Vec<u32>>,
    /// Solid's `Effects`.
    effects: Option<Vec<u32>>,
    exec: u64,
    runs: u64,
    /// The first exception of a round, thrown when the round ends.
    error: Option<JsValue>,
}

/// The graph of `ctx`.
///
/// # Safety
///
/// `ctx` has a state ([`crate::state::install`]), and the caller holds no
/// other reference to the graph.
#[inline]
pub(crate) unsafe fn graph<'a>(ctx: *mut JsContext) -> &'a mut Graph {
    // SAFETY: as the caller promises; a projection of the state's field.
    unsafe { &mut (*crate::state::state(ctx)).graph }
}

/// The graph of runtime `rt` (finalizers, GC marks), unless it is being
/// torn down.
fn graph_of_runtime<'a>(rt: *mut qjs::JSRuntime) -> Option<&'a mut Graph> {
    // SAFETY: a registered state is live; finalizers and marks run during
    // GC, when natives hold no reference to the graph.
    crate::state::of_runtime(rt).map(|p| unsafe { &mut (*p).graph })
}

/// Packs a node id and its generation into an opaque pointer (never null:
/// generations start at 1).
#[cfg(target_pointer_width = "64")]
fn pack(id: u32, generation: u32) -> *mut c_void {
    (((generation as u64) << 32) | id as u64) as usize as *mut c_void
}

#[cfg(target_pointer_width = "64")]
fn unpack(p: *mut c_void) -> (u32, u32) {
    let v = p as usize as u64;
    (v as u32, (v >> 32) as u32)
}

/// The bits of a generation an opaque pointer keeps.
#[cfg(target_pointer_width = "64")]
const GENERATION_MASK: u32 = u32::MAX;

/// 32-bit pointers (WebAssembly) hold 20 bits of id and the low 11 bits of
/// the generation, under a set top bit (never null). Stale handles are
/// still caught unless a node was reused a multiple of 2048 times.
#[cfg(target_pointer_width = "32")]
const GENERATION_MASK: u32 = 0x7ff;

#[cfg(target_pointer_width = "32")]
fn pack(id: u32, generation: u32) -> *mut c_void {
    debug_assert!(id < 1 << 20, "node ids fit in 20 bits");
    ((1 << 31) | ((generation & GENERATION_MASK) << 20) | (id & 0xf_ffff)) as usize as *mut c_void
}

#[cfg(target_pointer_width = "32")]
fn unpack(p: *mut c_void) -> (u32, u32) {
    let v = p as usize as u32;
    (v & 0xf_ffff, (v >> 20) & GENERATION_MASK)
}

impl Graph {
    fn alloc(&mut self, kind: Kind) -> u32 {
        match self.free.pop() {
            Some(id) => {
                let generation = self.nodes[id as usize].generation.wrapping_add(1).max(1);
                self.nodes[id as usize] = Node {
                    generation,
                    ..Node::new(kind)
                };
                id
            }
            None => {
                self.nodes.push(Node::new(kind));
                (self.nodes.len() - 1) as u32
            }
        }
    }

    /// Makes a computation under the current owner.
    fn computation(&mut self, kind: Kind, func: JsValue, value: JsValue, state: u8) -> u32 {
        let id = self.alloc(kind);
        let owner = self.owner;
        let context = self.context_of(owner);
        let n = &mut self.nodes[id as usize];
        n.func = func;
        n.value = value;
        n.state = state;
        n.owner = owner;
        n.context = context;
        if owner != NONE && owner != UNOWNED {
            self.nodes[owner as usize].owned.push(id);
        }
        id
    }

    fn context_of(&self, owner: u32) -> Option<Rc<Frame>> {
        if owner == NONE || owner == UNOWNED {
            return None;
        }
        self.nodes[owner as usize].context.clone()
    }

    /// Whether handle (`id`, `generation`) still names its node.
    fn live(&self, id: u32, generation: u32) -> bool {
        self.nodes.get(id as usize).is_some_and(|n| {
            n.generation & GENERATION_MASK == generation & GENERATION_MASK && n.kind != Kind::Free
        })
    }

    fn track(&mut self, id: u32) {
        let l = self.listener;
        if l == NONE || l == UNOWNED || self.nodes[l as usize].disposed {
            return;
        }
        // Repeated reads of the same source make one edge.
        if self.nodes[l as usize]
            .sources
            .last()
            .is_some_and(|&(s, _)| s == id)
        {
            return;
        }
        let slot = self.nodes[id as usize].observers.len() as u32;
        let back = self.nodes[l as usize].sources.len() as u32;
        self.nodes[l as usize].sources.push((id, slot));
        self.nodes[id as usize].observers.push((l, back));
    }

    fn unlink_sources(&mut self, id: u32) {
        while let Some((src, slot)) = self.nodes[id as usize].sources.pop() {
            let obs = &mut self.nodes[src as usize].observers;
            let Some(last) = obs.pop() else { continue };
            if (slot as usize) < obs.len() {
                obs[slot as usize] = last;
                self.nodes[last.0 as usize].sources[last.1 as usize].1 = slot;
            }
        }
    }

    fn unlink_observers(&mut self, id: u32) {
        while let Some((o, slot)) = self.nodes[id as usize].observers.pop() {
            let srcs = &mut self.nodes[o as usize].sources;
            let Some(last) = srcs.pop() else { continue };
            if (slot as usize) < srcs.len() {
                srcs[slot as usize] = last;
                self.nodes[last.0 as usize].observers[last.1 as usize].1 = slot;
            }
        }
    }

    /// Frees slot `id`: what it held, to be freed by the caller once no
    /// borrow of the graph is live.
    fn release(&mut self, id: u32) -> Held {
        let n = &mut self.nodes[id as usize];
        let held = Held {
            values: [
                std::mem::replace(&mut n.value, UNDEFINED),
                std::mem::replace(&mut n.func, UNDEFINED),
                match std::mem::replace(&mut n.equals, Equals::Strict) {
                    Equals::Custom(f) => f,
                    _ => UNDEFINED,
                },
            ],
            context: n.context.take(),
        };
        n.kind = Kind::Free;
        n.handle = false;
        n.disposed = true;
        self.free.push(id);
        held
    }

    fn queue_update(&mut self, id: u32) {
        match &mut self.updates {
            Some(q) => q.push(id),
            None => self.updates = Some(vec![id]),
        }
    }

    fn queue_effect(&mut self, id: u32) {
        match &mut self.effects {
            Some(q) => q.push(id),
            None => self.effects = Some(vec![id]),
        }
    }

    /// Solid's `markDownstream`.
    fn mark_downstream(&mut self, id: u32) {
        let mut i = 0;
        while i < self.nodes[id as usize].observers.len() {
            let o = self.nodes[id as usize].observers[i].0;
            i += 1;
            if self.nodes[o as usize].state == CLEAN && !self.nodes[o as usize].disposed {
                self.nodes[o as usize].state = PENDING;
                if self.nodes[o as usize].kind == Kind::Memo {
                    self.queue_update(o);
                } else {
                    self.queue_effect(o);
                }
                if !self.nodes[o as usize].observers.is_empty() {
                    self.mark_downstream(o);
                }
            }
        }
    }
}

/// An exception of a JS call: kept as the round's error (the first wins).
fn take_exception(ctx: *mut JsContext) {
    // SAFETY: an exception is pending.
    let exception = unsafe { qjs::JS_GetException(ctx) };
    // SAFETY: no other reference to the graph is live here.
    let g = unsafe { graph(ctx) };
    if g.error.is_none() {
        g.error = Some(exception);
    } else {
        // SAFETY: ours.
        unsafe { qjs::JS_FreeValue(ctx, exception) };
    }
}

/// Calls `f` with `args`, keeping an exception as the round's error.
/// `Err` when it threw.
fn call(ctx: *mut JsContext, f: JsValue, args: &mut [JsValue]) -> Result<JsValue, ()> {
    // SAFETY: `f` and `args` are live values; the call may re-enter the
    // graph, which holds no reference here.
    let r = unsafe { qjs::JS_Call(ctx, f, UNDEFINED, args.len() as c_int, args.as_mut_ptr()) };
    // SAFETY: inline tag check.
    if unsafe { qjs::JS_IsException(r) } {
        take_exception(ctx);
        Err(())
    } else {
        Ok(r)
    }
}

/// Solid's `runUpdates`: runs `f`, then the queued computations and
/// effects (unless a round is running already: they join it). `init` (a
/// root, `runWithOwner`): what `f` makes runs at once rather than queued;
/// only effects wait for the end.
fn run_updates<R>(ctx: *mut JsContext, init: bool, f: impl FnOnce() -> R) -> R {
    // SAFETY: short borrows, none across `f`.
    let g = unsafe { graph(ctx) };
    if g.updates.is_some() {
        return f();
    }
    if !init {
        g.updates = Some(Vec::new());
    }
    let wait = if g.effects.is_some() {
        true
    } else {
        g.effects = Some(Vec::new());
        false
    };
    g.exec += 1;
    let r = f();
    complete_updates(ctx, wait);
    r
}

/// Solid's `completeUpdates`.
fn complete_updates(ctx: *mut JsContext, wait: bool) {
    let mut i = 0;
    loop {
        // SAFETY: short borrows.
        let g = unsafe { graph(ctx) };
        let Some(next) = g.updates.as_ref().and_then(|q| q.get(i).copied()) else {
            break;
        };
        i += 1;
        if !count_run(ctx) {
            break;
        }
        run_top(ctx, next);
    }
    // SAFETY: short borrow.
    let g = unsafe { graph(ctx) };
    g.updates = None;
    if wait {
        return;
    }
    let effects = g.effects.take().unwrap_or_default();
    if !effects.is_empty() {
        run_updates(ctx, false, || run_effects(ctx, &effects));
    }
}

/// Counts a run; false (with an error) once a flush runs away.
fn count_run(ctx: *mut JsContext) -> bool {
    // SAFETY: short borrow.
    let g = unsafe { graph(ctx) };
    g.runs += 1;
    if g.runs <= MAX_RUNS {
        return true;
    }
    g.updates = Some(Vec::new());
    g.effects = None;
    if g.error.is_none() {
        let message = c"potential infinite loop: an effect keeps changing what it reads";
        // SAFETY: a new error object.
        unsafe {
            let error = qjs::JS_NewError(ctx);
            let bytes = message.to_bytes();
            let text = qjs::JS_NewStringLen(ctx, bytes.as_ptr().cast(), bytes.len() as _);
            qjs::JS_SetPropertyStr(ctx, error, c"message".as_ptr(), text);
            g.error = Some(error);
        }
    }
    false
}

/// Solid's `runEffects` with user effects after the render effects.
fn run_effects(ctx: *mut JsContext, effects: &[u32]) {
    for user in [false, true] {
        for &e in effects {
            // SAFETY: short borrow.
            let kind = unsafe { graph(ctx) }.nodes[e as usize].kind;
            if (kind == Kind::UserEffect) == user && count_run(ctx) {
                run_top(ctx, e);
            }
        }
    }
}

/// Solid's `runTop`: updates the node, its stale owners first.
fn run_top(ctx: *mut JsContext, id: u32) {
    // SAFETY: short borrows, none across JS.
    let g = unsafe { graph(ctx) };
    let node = &g.nodes[id as usize];
    if node.state == CLEAN || node.disposed || !node.is_computation() {
        return;
    }
    if node.state == PENDING {
        return look_upstream(ctx, id, NONE);
    }
    let exec = g.exec;
    let mut ancestors = vec![id];
    let mut at = node.owner;
    while at != NONE && at != UNOWNED {
        let n = &g.nodes[at as usize];
        if n.updated_at.is_some_and(|t| t >= exec) {
            break;
        }
        if n.state != CLEAN && n.is_computation() {
            ancestors.push(at);
        }
        at = n.owner;
    }
    for &node in ancestors.iter().rev() {
        // SAFETY: short borrow.
        let n = &unsafe { graph(ctx) }.nodes[node as usize];
        if n.disposed {
            continue;
        }
        if n.state == STALE {
            update_computation(ctx, node);
        } else if n.state == PENDING {
            let g = unsafe { graph(ctx) };
            let updates = g.updates.take();
            run_updates(ctx, false, || look_upstream(ctx, node, ancestors[0]));
            // SAFETY: short borrow.
            unsafe { graph(ctx) }.updates = updates;
        }
    }
}

/// Solid's `lookUpstream`: brings the stale memos `id` reads up to date.
fn look_upstream(ctx: *mut JsContext, id: u32, ignore: u32) {
    // SAFETY: short borrows, none across JS.
    unsafe { graph(ctx) }.nodes[id as usize].state = CLEAN;
    let mut i = 0;
    loop {
        let g = unsafe { graph(ctx) };
        let Some(&(source, _)) = g.nodes[id as usize].sources.get(i) else {
            break;
        };
        i += 1;
        let s = &g.nodes[source as usize];
        if s.kind != Kind::Memo {
            continue;
        }
        if s.state == STALE {
            if source != ignore && s.updated_at.is_none_or(|t| t < g.exec) {
                run_top(ctx, source);
            }
        } else if s.state == PENDING {
            look_upstream(ctx, source, ignore);
        }
    }
}

/// Solid's `updateComputation`: cleans the node and runs it again.
fn update_computation(ctx: *mut JsContext, id: u32) {
    // SAFETY: short borrow.
    let g = unsafe { graph(ctx) };
    if g.nodes[id as usize].disposed || g.nodes[id as usize].kind == Kind::Free {
        return;
    }
    clean_node(ctx, id);
    let exec = unsafe { graph(ctx) }.exec;
    run_computation(ctx, id, exec);
}

/// Solid's `runComputation`.
fn run_computation(ctx: *mut JsContext, id: u32, time: u64) {
    // SAFETY: short borrows, none across JS.
    let g = unsafe { graph(ctx) };
    let n = &g.nodes[id as usize];
    // SAFETY: dups of values the node keeps.
    let (func, prev) = unsafe {
        (
            qjs::JS_DupValue(ctx, n.func),
            qjs::JS_DupValue(ctx, n.value),
        )
    };
    let (listener, owner) = (g.listener, g.owner);
    g.listener = id;
    g.owner = id;
    let mut args = [prev];
    let result = call(ctx, func, &mut args);
    // SAFETY: our dups.
    unsafe {
        qjs::JS_FreeValue(ctx, func);
        qjs::JS_FreeValue(ctx, args[0]);
    }
    let g = unsafe { graph(ctx) };
    g.listener = listener;
    g.owner = owner;
    let Ok(next) = result else {
        let n = &mut g.nodes[id as usize];
        if n.kind == Kind::Memo {
            n.state = STALE;
            let owned = std::mem::take(&mut n.owned);
            for c in owned {
                dispose(ctx, c);
            }
        }
        unsafe { graph(ctx) }.nodes[id as usize].updated_at = Some(time + 1);
        return;
    };
    let n = &mut g.nodes[id as usize];
    if n.disposed {
        // Disposed while running (its owner re-ran).
        unsafe { qjs::JS_FreeValue(ctx, next) };
        return;
    }
    if n.updated_at.is_none_or(|t| t <= time) {
        let notify = n.updated_at.is_some() && n.kind == Kind::Memo;
        n.updated_at = Some(time);
        if notify {
            write(ctx, id, next);
        } else {
            let old = std::mem::replace(&mut n.value, next);
            // SAFETY: the node's previous value; the borrow ended.
            unsafe { qjs::JS_FreeValue(ctx, old) };
        }
    } else {
        unsafe { qjs::JS_FreeValue(ctx, next) };
    }
}

/// Whether `next` counts as a change from node `id`'s value.
fn changed(ctx: *mut JsContext, id: u32, next: JsValue) -> bool {
    // SAFETY: short borrows, none across JS.
    let n = &unsafe { graph(ctx) }.nodes[id as usize];
    match n.equals {
        Equals::Strict => !unsafe { qjs::JS_IsStrictEqual(ctx, n.value, next) },
        Equals::Never => true,
        Equals::Truthiness => unsafe {
            (qjs::JS_ToBool(ctx, n.value) == 1) != (qjs::JS_ToBool(ctx, next) == 1)
        },
        Equals::Custom(f) => {
            let (f, prev) = unsafe { (qjs::JS_DupValue(ctx, f), qjs::JS_DupValue(ctx, n.value)) };
            let mut args = [prev, unsafe { qjs::JS_DupValue(ctx, next) }];
            let r = call(ctx, f, &mut args);
            unsafe {
                qjs::JS_FreeValue(ctx, f);
                qjs::JS_FreeValue(ctx, args[0]);
                qjs::JS_FreeValue(ctx, args[1]);
            }
            match r {
                Ok(same) => {
                    let same = unsafe { qjs::JS_ToBool(ctx, same) } == 1;
                    !same
                }
                Err(()) => true,
            }
        }
    }
}

/// Solid's `writeSignal`: takes `next` (owned).
fn write(ctx: *mut JsContext, id: u32, next: JsValue) {
    if !changed(ctx, id, next) {
        // SAFETY: owned.
        unsafe { qjs::JS_FreeValue(ctx, next) };
        return;
    }
    // SAFETY: short borrows, none across JS or a free.
    let g = unsafe { graph(ctx) };
    let old = std::mem::replace(&mut g.nodes[id as usize].value, next);
    let observed = !g.nodes[id as usize].observers.is_empty();
    unsafe { qjs::JS_FreeValue(ctx, old) };
    if !observed {
        return;
    }
    run_updates(ctx, false, || {
        let g = unsafe { graph(ctx) };
        let mut i = 0;
        while i < g.nodes[id as usize].observers.len() {
            let o = g.nodes[id as usize].observers[i].0;
            i += 1;
            if g.nodes[o as usize].disposed {
                continue;
            }
            if g.nodes[o as usize].state == CLEAN {
                if g.nodes[o as usize].kind == Kind::Memo {
                    g.queue_update(o);
                } else {
                    g.queue_effect(o);
                }
                if !g.nodes[o as usize].observers.is_empty() {
                    g.mark_downstream(o);
                }
            }
            g.nodes[o as usize].state = STALE;
        }
    });
}

/// Solid's `readSignal`: brings a memo up to date, tracks, and returns a
/// dup of the value.
fn read(ctx: *mut JsContext, id: u32) -> JsValue {
    // SAFETY: short borrows, none across JS.
    let g = unsafe { graph(ctx) };
    let n = &g.nodes[id as usize];
    if n.kind == Kind::Memo && !n.disposed && n.state != CLEAN {
        if n.state == STALE {
            update_computation(ctx, id);
        } else {
            let g = unsafe { graph(ctx) };
            let updates = g.updates.take();
            run_updates(ctx, false, || look_upstream(ctx, id, NONE));
            unsafe { graph(ctx) }.updates = updates;
        }
    }
    let g = unsafe { graph(ctx) };
    if !g.nodes[id as usize].disposed {
        g.track(id);
    }
    unsafe { qjs::JS_DupValue(ctx, g.nodes[id as usize].value) }
}

/// Solid's `cleanNode`: unlinks the node's sources, disposes what it owns
/// and runs its cleanups.
fn clean_node(ctx: *mut JsContext, id: u32) {
    // SAFETY: short borrows, none across JS.
    let g = unsafe { graph(ctx) };
    g.unlink_sources(id);
    while let Some(c) = unsafe { graph(ctx) }.nodes[id as usize].owned.pop() {
        dispose(ctx, c);
    }
    while let Some(f) = unsafe { graph(ctx) }.nodes[id as usize].cleanups.pop() {
        let _ = call(ctx, f, &mut []).map(|r| unsafe { qjs::JS_FreeValue(ctx, r) });
        unsafe { qjs::JS_FreeValue(ctx, f) };
    }
    unsafe { graph(ctx) }.nodes[id as usize].state = CLEAN;
}

/// Disposes node `id`: cleans it, unlinks it and frees it (a memo with an
/// object keeps its value until the object goes).
fn dispose(ctx: *mut JsContext, id: u32) {
    // SAFETY: short borrows, none across JS.
    let g = unsafe { graph(ctx) };
    if g.nodes[id as usize].disposed {
        return;
    }
    clean_node(ctx, id);
    let g = unsafe { graph(ctx) };
    g.unlink_observers(id);
    let n = &mut g.nodes[id as usize];
    n.disposed = true;
    n.state = CLEAN;
    if n.handle {
        let func = std::mem::replace(&mut n.func, UNDEFINED);
        let context = n.context.take();
        unsafe { qjs::JS_FreeValue(ctx, func) };
        drop(context);
    } else {
        g.release(id).free(ctx);
    }
}

// --- The JS objects -------------------------------------------------------

/// The node a signal / setter / memo object names, if it is still live.
unsafe fn node_of(ctx: *mut JsContext, obj: JsValue, class: qjs::JSClassID) -> Option<u32> {
    // SAFETY: an object of `class` holds a packed handle.
    let p = unsafe { qjs::JS_GetOpaque(obj, class) };
    if p.is_null() {
        return None;
    }
    let (id, generation) = unpack(p);
    unsafe { graph(ctx) }.live(id, generation).then_some(id)
}

unsafe extern "C" fn signal_call(
    ctx: *mut JsContext,
    func: JsValue,
    _this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
    _flags: c_int,
) -> JsValue {
    let class = unsafe { graph(ctx) }.classes.signal;
    let memo = unsafe { graph(ctx) }.classes.memo;
    let id = unsafe { node_of(ctx, func, class).or_else(|| node_of(ctx, func, memo)) };
    match id {
        Some(id) => {
            let value = read(ctx, id);
            finish(ctx, value)
        }
        None => UNDEFINED,
    }
}

unsafe extern "C" fn setter_call(
    ctx: *mut JsContext,
    func: JsValue,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
    _flags: c_int,
) -> JsValue {
    let class = unsafe { graph(ctx) }.classes.setter;
    let Some(id) = (unsafe { node_of(ctx, func, class) }) else {
        return UNDEFINED;
    };
    let mut value = if argc > 0 {
        unsafe { qjs::JS_DupValue(ctx, *argv) }
    } else {
        UNDEFINED
    };
    // An updater gets the previous value.
    if unsafe { qjs::JS_IsFunction(ctx, value) } {
        let prev = unsafe { qjs::JS_DupValue(ctx, graph(ctx).nodes[id as usize].value) };
        let mut args = [prev];
        let r = unsafe { qjs::JS_Call(ctx, value, UNDEFINED, 1, args.as_mut_ptr()) };
        unsafe {
            qjs::JS_FreeValue(ctx, args[0]);
            qjs::JS_FreeValue(ctx, value);
        }
        if unsafe { qjs::JS_IsException(r) } {
            return r;
        }
        value = r;
    }
    let returned = unsafe { qjs::JS_DupValue(ctx, value) };
    write(ctx, id, value);
    finish(ctx, returned)
}

/// Returns `value`, or throws the error a round kept.
fn finish(ctx: *mut JsContext, value: JsValue) -> JsValue {
    // SAFETY: short borrow.
    let g = unsafe { graph(ctx) };
    if g.updates.is_some() {
        // Still in a round: its end throws.
        return value;
    }
    g.runs = 0;
    match g.error.take() {
        Some(error) => unsafe {
            qjs::JS_FreeValue(ctx, value);
            qjs::JS_Throw(ctx, error)
        },
        None => value,
    }
}

unsafe extern "C" fn finalize(rt: *mut qjs::JSRuntime, obj: JsValue) {
    let Some(g) = graph_of_runtime(rt) else {
        return;
    };
    let class = unsafe { qjs::JS_GetClassID(obj) };
    if class == g.classes.setter {
        return;
    }
    let p = unsafe { qjs::JS_GetOpaque(obj, class) };
    if p.is_null() {
        return;
    }
    let (id, generation) = unpack(p);
    if !g.live(id, generation) {
        return;
    }
    let n = &mut g.nodes[id as usize];
    n.handle = false;
    if n.kind == Kind::Signal || n.disposed {
        g.unlink_observers(id);
        // Freed now, as QuickJS's cycle collection expects of finalizers.
        g.release(id).free(g.ctx);
    }
}

unsafe extern "C" fn gc_mark(rt: *mut qjs::JSRuntime, obj: JsValue, mark: qjs::JS_MarkFunc) {
    let Some(g) = graph_of_runtime(rt) else {
        return;
    };
    let class = unsafe { qjs::JS_GetClassID(obj) };
    let p = unsafe { qjs::JS_GetOpaque(obj, class) };
    if p.is_null() {
        return;
    }
    let (id, generation) = unpack(p);
    if g.live(id, generation) {
        let n = &g.nodes[id as usize];
        unsafe { qjs::JS_MarkValue(rt, n.value, mark) };
        if let Equals::Custom(f) = n.equals {
            unsafe { qjs::JS_MarkValue(rt, f, mark) };
        }
    }
}

/// A new object of `class` naming node `id`.
fn object(ctx: *mut JsContext, class: qjs::JSClassID, id: u32) -> JsValue {
    // SAFETY: a registered class; the opaque is a packed handle.
    unsafe {
        let generation = graph(ctx).nodes[id as usize].generation;
        let obj = qjs::JS_NewObjectClass(ctx, class);
        qjs::JS_SetOpaque(obj, pack(id, generation));
        obj
    }
}

// --- Natives --------------------------------------------------------------

pub(crate) fn arg(argc: c_int, argv: *mut JsValue, i: usize) -> JsValue {
    if (i as c_int) < argc {
        // SAFETY: QuickJS passes `argc` values.
        unsafe { *argv.add(i) }
    } else {
        UNDEFINED
    }
}

/// `equals` of an options object.
fn equals_option(ctx: *mut JsContext, options: JsValue) -> Equals {
    // SAFETY: reading a property of a live value.
    unsafe {
        if !qjs::JS_IsObject(options) {
            return Equals::Strict;
        }
        let e = qjs::JS_GetPropertyStr(ctx, options, c"equals".as_ptr());
        if qjs::JS_IsFunction(ctx, e) {
            Equals::Custom(e)
        } else {
            let never = qjs::JS_IsBool(e) && !qjs::JS_VALUE_GET_BOOL(e);
            qjs::JS_FreeValue(ctx, e);
            if never { Equals::Never } else { Equals::Strict }
        }
    }
}

pub(crate) type Native =
    unsafe extern "C" fn(*mut JsContext, JsValue, c_int, *mut JsValue) -> JsValue;

/// `createSignal(value, options)`: `[read, write]`.
#[cfg(test)]
unsafe extern "C" fn create_signal(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let g = unsafe { graph(ctx) };
    let id = g.alloc(Kind::Signal);
    let value = unsafe { qjs::JS_DupValue(ctx, arg(argc, argv, 0)) };
    let equals = equals_option(ctx, arg(argc, argv, 1));
    let g = unsafe { graph(ctx) };
    let n = &mut g.nodes[id as usize];
    n.value = value;
    n.equals = equals;
    n.handle = true;
    let classes = g.classes;
    let mut pair = [
        object(ctx, classes.signal, id),
        object(ctx, classes.setter, id),
    ];
    unsafe { qjs::JS_NewArrayFrom(ctx, 2, pair.as_mut_ptr()) }
}

/// `createMemo(fn, value, options)`.
unsafe extern "C" fn create_memo(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let (func, value) = unsafe {
        (
            qjs::JS_DupValue(ctx, arg(argc, argv, 0)),
            qjs::JS_DupValue(ctx, arg(argc, argv, 1)),
        )
    };
    let equals = equals_option(ctx, arg(argc, argv, 2));
    let g = unsafe { graph(ctx) };
    let id = g.computation(Kind::Memo, func, value, STALE);
    g.nodes[id as usize].equals = equals;
    g.nodes[id as usize].handle = true;
    let obj = object(ctx, g.classes.memo, id);
    // At once, inside a round too: what is made while rendering reads it
    // right away (as in Solid without transitions).
    update_computation(ctx, id);
    finish(ctx, obj)
}

fn create_effect(ctx: *mut JsContext, argc: c_int, argv: *mut JsValue, kind: Kind) -> JsValue {
    let (func, value) = unsafe {
        (
            qjs::JS_DupValue(ctx, arg(argc, argv, 0)),
            qjs::JS_DupValue(ctx, arg(argc, argv, 1)),
        )
    };
    let g = unsafe { graph(ctx) };
    let id = g.computation(kind, func, value, STALE);
    if kind == Kind::UserEffect {
        // After the render effects of the round, or now outside one.
        if g.effects.is_some() {
            g.queue_effect(id);
        } else {
            update_computation(ctx, id);
        }
    } else {
        // A render effect runs at once, inside a round too: a provider
        // returns what its effect rendered.
        update_computation(ctx, id);
    }
    finish(ctx, UNDEFINED)
}

/// `createRenderEffect(fn, value)`.
#[cfg(test)]
unsafe extern "C" fn create_render_effect(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    create_effect(ctx, argc, argv, Kind::Effect)
}

/// `createEffect(fn, value)`.
unsafe extern "C" fn create_user_effect(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    create_effect(ctx, argc, argv, Kind::UserEffect)
}

/// The owner a handle names (`getOwner()`'s value), `UNOWNED` for others.
#[cfg(test)]
fn owner_of(ctx: *mut JsContext, handle: JsValue) -> Option<u32> {
    // SAFETY: inline tag checks.
    unsafe {
        if qjs::JS_IsUndefined(handle) {
            return None;
        }
        let mut v = 0f64;
        if !qjs::JS_IsNumber(handle) || qjs::JS_ToFloat64(ctx, &mut v, handle) != 0 {
            return Some(UNOWNED);
        }
        let bits = v as u64;
        let (id, generation) = (bits as u32, (bits >> 32) as u32);
        let g = graph(ctx);
        Some(
            if g.live(id, generation) && !g.nodes[id as usize].disposed {
                id
            } else {
                UNOWNED
            },
        )
    }
}

#[cfg(test)]
unsafe extern "C" fn dispose_root(
    ctx: *mut JsContext,
    _this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
    _magic: c_int,
    data: *mut JsValue,
) -> JsValue {
    let handle = unsafe { *data };
    if let Some(id) = owner_of(ctx, handle).filter(|&id| id != UNOWNED) {
        // Untracked: disposing reads nothing.
        let g = unsafe { graph(ctx) };
        let listener = std::mem::replace(&mut g.listener, NONE);
        dispose(ctx, id);
        unsafe { graph(ctx) }.listener = listener;
    }
    finish(ctx, UNDEFINED)
}

/// `createRoot(fn, owner)`: `fn(dispose)` (a root without the parameter
/// is never disposed and owns nothing).
#[cfg(test)]
unsafe extern "C" fn create_root(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let func = arg(argc, argv, 0);
    let detached = owner_of(ctx, arg(argc, argv, 1));
    let g = unsafe { graph(ctx) };
    let current = detached.unwrap_or(g.owner);
    let mut length = 0i32;
    unsafe {
        let l = qjs::JS_GetPropertyStr(ctx, func, c"length".as_ptr());
        qjs::JS_ToInt32(ctx, &mut length, l);
        qjs::JS_FreeValue(ctx, l);
    }
    let (root, args) = if length == 0 {
        (UNOWNED, Vec::new())
    } else {
        let g = unsafe { graph(ctx) };
        let id = g.alloc(Kind::Root);
        let context = g.context_of(current);
        let n = &mut g.nodes[id as usize];
        n.owner = current;
        n.context = context;
        let mut data = [handle_value(ctx, id)];
        let dispose = unsafe {
            qjs::JS_NewCFunctionData(ctx, Some(dispose_root), 0, 0, 1, data.as_mut_ptr())
        };
        (id, vec![dispose])
    };
    let g = unsafe { graph(ctx) };
    let (listener, owner) = (g.listener, g.owner);
    g.owner = root;
    g.listener = NONE;
    let mut args = args;
    let func = unsafe { qjs::JS_DupValue(ctx, func) };
    let result = run_updates(ctx, true, || call(ctx, func, &mut args));
    unsafe {
        qjs::JS_FreeValue(ctx, func);
        for a in args {
            qjs::JS_FreeValue(ctx, a);
        }
    }
    let g = unsafe { graph(ctx) };
    g.listener = listener;
    g.owner = owner;
    finish(ctx, result.unwrap_or(UNDEFINED))
}

/// An owner handle: a number packing the node and its generation.
#[cfg(test)]
fn handle_value(ctx: *mut JsContext, id: u32) -> JsValue {
    let generation = unsafe { graph(ctx) }.nodes[id as usize].generation;
    qjs::JS_NewFloat64((((generation as u64) << 32) | id as u64) as f64)
}

/// `untrack(fn)`.
pub(crate) unsafe extern "C" fn untrack(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let g = unsafe { graph(ctx) };
    let listener = std::mem::replace(&mut g.listener, NONE);
    let r = unsafe { qjs::JS_Call(ctx, arg(argc, argv, 0), UNDEFINED, 0, ptr::null_mut()) };
    unsafe { graph(ctx) }.listener = listener;
    r
}

/// `batch(fn)`.
pub(crate) unsafe extern "C" fn batch(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let func = unsafe { qjs::JS_DupValue(ctx, arg(argc, argv, 0)) };
    let r = run_updates(ctx, false, || call(ctx, func, &mut []));
    unsafe { qjs::JS_FreeValue(ctx, func) };
    finish(ctx, r.unwrap_or(UNDEFINED))
}

/// `onCleanup(fn)`: runs before the owner runs again or is disposed.
pub(crate) unsafe extern "C" fn on_cleanup(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let f = arg(argc, argv, 0);
    let g = unsafe { graph(ctx) };
    if g.owner != NONE && g.owner != UNOWNED {
        let f = unsafe { qjs::JS_DupValue(ctx, f) };
        g.nodes[g.owner as usize].cleanups.push(f);
    }
    unsafe { qjs::JS_DupValue(ctx, f) }
}

/// `getOwner()`: a handle, or `undefined`.
#[cfg(test)]
unsafe extern "C" fn get_owner(
    ctx: *mut JsContext,
    _this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
) -> JsValue {
    let owner = unsafe { graph(ctx) }.owner;
    if owner == NONE || owner == UNOWNED {
        UNDEFINED
    } else {
        handle_value(ctx, owner)
    }
}

/// `runWithOwner(owner, fn)`.
#[cfg(test)]
unsafe extern "C" fn run_with_owner(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let owner = owner_of(ctx, arg(argc, argv, 0)).unwrap_or(UNOWNED);
    let g = unsafe { graph(ctx) };
    let (prev_owner, prev_listener) = (g.owner, g.listener);
    g.owner = owner;
    g.listener = NONE;
    let func = unsafe { qjs::JS_DupValue(ctx, arg(argc, argv, 1)) };
    let r = run_updates(ctx, true, || call(ctx, func, &mut []));
    unsafe { qjs::JS_FreeValue(ctx, func) };
    let g = unsafe { graph(ctx) };
    g.owner = prev_owner;
    g.listener = prev_listener;
    finish(ctx, r.unwrap_or(UNDEFINED))
}

/// `setContext(key, value)`: the current owner and what is made under it
/// from now on see `value` for `key`.
unsafe extern "C" fn set_context(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let g = unsafe { graph(ctx) };
    let owner = g.owner;
    if owner == NONE || owner == UNOWNED {
        return UNDEFINED;
    }
    let parent = g.nodes[owner as usize].context.clone();
    let frame = unsafe {
        Frame {
            parent,
            key: qjs::JS_DupValue(ctx, arg(argc, argv, 0)),
            value: qjs::JS_DupValue(ctx, arg(argc, argv, 1)),
            ctx,
        }
    };
    let old = g.nodes[owner as usize].context.replace(Rc::new(frame));
    drop(old);
    UNDEFINED
}

/// `useContext(key)`: the nearest value set for `key`, or `undefined`.
#[cfg(test)]
unsafe extern "C" fn use_context(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let key = arg(argc, argv, 0);
    let g = unsafe { graph(ctx) };
    let mut frame = g.context_of(g.owner);
    while let Some(f) = frame {
        if unsafe { qjs::JS_IsStrictEqual(ctx, f.key, key) } {
            return unsafe { qjs::JS_DupValue(ctx, f.value) };
        }
        frame = f.parent.clone();
    }
    UNDEFINED
}

impl Graph {
    /// A graph for context `raw`, with its classes registered.
    pub(crate) fn new(raw: *mut JsContext) -> Graph {
        // SAFETY: registering classes on the context's runtime.
        unsafe {
            let rt = qjs::JS_GetRuntime(raw);
            let function_proto = qjs::JS_GetFunctionProto(raw);
            let class =
                |name: &'static CStr, call: qjs::JSClassCall, mark: bool, proto: JsValue| {
                    let mut id: qjs::JSClassID = 0;
                    qjs::JS_NewClassID(rt, &mut id);
                    let def = qjs::JSClassDef {
                        class_name: name.as_ptr(),
                        finalizer: Some(finalize),
                        gc_mark: if mark { Some(gc_mark) } else { None },
                        call,
                        exotic: ptr::null_mut(),
                    };
                    qjs::JS_NewClass(rt, id, &def);
                    // Calls of their objects are function calls (`typeof` is
                    // "function"); `proto` is taken.
                    qjs::JS_SetClassProto(raw, id, proto);
                    id
                };
            // A signal's methods: `count.set(1)`, `count.update(n => n + 1)`.
            let signal_proto = qjs::JS_NewObjectProto(raw, function_proto);
            for (name, f, length) in [
                (c"set", signal_set as Native, 1),
                (c"update", signal_update as Native, 1),
            ] {
                let func = new_function(raw, name, f, length);
                qjs::JS_SetPropertyStr(raw, signal_proto, name.as_ptr(), func);
            }
            let classes = Classes {
                signal: class(c"Signal", Some(signal_call), true, signal_proto),
                setter: class(
                    c"SignalSetter",
                    Some(setter_call),
                    false,
                    qjs::JS_DupValue(raw, function_proto),
                ),
                memo: class(
                    c"Memo",
                    Some(signal_call),
                    true,
                    qjs::JS_DupValue(raw, function_proto),
                ),
            };
            qjs::JS_FreeValue(raw, function_proto);
            Graph {
                ctx: raw,
                classes,
                nodes: Vec::new(),
                free: Vec::new(),
                listener: NONE,
                owner: NONE,
                updates: None,
                effects: None,
                exec: 0,
                runs: 0,
                error: None,
            }
        }
    }

    /// Frees every JS value the graph holds (the state is unregistered:
    /// finalizers that run meanwhile leave the graph alone).
    pub(crate) fn free_all(&mut self, raw: *mut JsContext) {
        // SAFETY: the graph's own values; no JS runs from here on.
        unsafe {
            for n in &mut self.nodes {
                if n.kind == Kind::Free {
                    continue;
                }
                qjs::JS_FreeValue(raw, std::mem::replace(&mut n.value, UNDEFINED));
                qjs::JS_FreeValue(raw, std::mem::replace(&mut n.func, UNDEFINED));
                if let Equals::Custom(f) = std::mem::replace(&mut n.equals, Equals::Strict) {
                    qjs::JS_FreeValue(raw, f);
                }
                for f in n.cleanups.drain(..) {
                    qjs::JS_FreeValue(raw, f);
                }
                n.context = None;
                n.kind = Kind::Free;
            }
            if let Some(e) = self.error.take() {
                qjs::JS_FreeValue(raw, e);
            }
        }
    }
}

/// A native function named `name`.
pub(crate) fn new_function(ctx: *mut JsContext, name: &CStr, f: Native, length: c_int) -> JsValue {
    // SAFETY: a plain C function.
    unsafe {
        qjs::JS_NewCFunction2(
            ctx,
            Some(f),
            name.as_ptr(),
            length,
            qjs::JSCFunctionEnum_JS_CFUNC_generic,
            0,
        )
    }
}

/// The graph's primitives as `globalThis.__nsh_reactive`, for tests that
/// drive owners and roots directly.
#[cfg(test)]
pub(crate) fn install_globals(ctx: &Ctx<'_>) {
    let raw = ctx.as_raw().as_ptr();
    // SAFETY: natives on the global object.
    unsafe {
        let obj = qjs::JS_NewObject(raw);
        let natives: [(&CStr, Native, c_int); 13] = [
            (c"createSignal", create_signal, 2),
            (c"createMemo", create_memo, 3),
            (c"createRenderEffect", create_render_effect, 2),
            (c"createEffect", create_user_effect, 2),
            (c"createRoot", create_root, 2),
            (c"untrack", untrack, 1),
            (c"batch", batch, 1),
            (c"onCleanup", on_cleanup, 1),
            (c"getOwner", get_owner, 0),
            (c"runWithOwner", run_with_owner, 2),
            (c"setContext", set_context, 2),
            (c"useContext", use_context, 1),
            (c"isAccessor", is_accessor, 1),
        ];
        for (name, f, length) in natives {
            let func = new_function(raw, name, f, length);
            qjs::JS_SetPropertyStr(raw, obj, name.as_ptr(), func);
        }
        let global = qjs::JS_GetGlobalObject(raw);
        qjs::JS_SetPropertyStr(raw, global, c"__nsh_reactive".as_ptr(), obj);
        qjs::JS_FreeValue(raw, global);
    }
}

/// `isAccessor(value)`: whether `value` is a signal or memo object.
#[cfg(test)]
unsafe extern "C" fn is_accessor(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let v = arg(argc, argv, 0);
    let g = unsafe { graph(ctx) };
    let class = unsafe { qjs::JS_GetClassID(v) };
    let yes = class == g.classes.signal || class == g.classes.memo;
    if yes { qjs::JS_TRUE } else { qjs::JS_FALSE }
}

// --- `nsh`'s API ------------------------------------------------------------

/// `signal(value, options)`: a signal object, read by calling it.
pub(crate) unsafe extern "C" fn signal(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let g = unsafe { graph(ctx) };
    let id = g.alloc(Kind::Signal);
    let value = unsafe { qjs::JS_DupValue(ctx, arg(argc, argv, 0)) };
    let equals = equals_option(ctx, arg(argc, argv, 1));
    let g = unsafe { graph(ctx) };
    let n = &mut g.nodes[id as usize];
    n.value = value;
    n.equals = equals;
    n.handle = true;
    object(ctx, g.classes.signal, id)
}

/// The signal `this` names, or a thrown `TypeError`.
fn this_signal(ctx: *mut JsContext, this: JsValue) -> Result<u32, JsValue> {
    let class = unsafe { graph(ctx) }.classes.signal;
    match unsafe { node_of(ctx, this, class) } {
        Some(id) => Ok(id),
        // SAFETY: throwing on a live context.
        None => Err(unsafe { qjs::JS_ThrowTypeError(ctx, c"not a live signal".as_ptr()) }),
    }
}

/// `signal.set(value)`.
unsafe extern "C" fn signal_set(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let id = match this_signal(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let value = unsafe { qjs::JS_DupValue(ctx, arg(argc, argv, 0)) };
    write(ctx, id, value);
    finish(ctx, UNDEFINED)
}

/// `signal.update(fn)`: sets `fn(current value)`.
unsafe extern "C" fn signal_update(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let id = match this_signal(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let f = arg(argc, argv, 0);
    let prev = unsafe { qjs::JS_DupValue(ctx, graph(ctx).nodes[id as usize].value) };
    let mut args = [prev];
    // SAFETY: a live value; the graph is not borrowed across the call.
    let next = unsafe { qjs::JS_Call(ctx, f, UNDEFINED, 1, args.as_mut_ptr()) };
    unsafe { qjs::JS_FreeValue(ctx, args[0]) };
    if unsafe { qjs::JS_IsException(next) } {
        return next;
    }
    write(ctx, id, next);
    finish(ctx, UNDEFINED)
}

/// `computed(fn, options)`: a memo object, read by calling it.
pub(crate) unsafe extern "C" fn computed(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let mut args = [arg(argc, argv, 0), UNDEFINED, arg(argc, argv, 1)];
    // SAFETY: borrowed arguments, as `createMemo` takes them.
    unsafe { create_memo(ctx, UNDEFINED, 3, args.as_mut_ptr()) }
}

/// `effect(fn)`: runs `fn` after rendering, and again when what it read
/// changes.
pub(crate) unsafe extern "C" fn effect(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let mut args = [arg(argc, argv, 0)];
    // SAFETY: as above.
    unsafe { create_user_effect(ctx, UNDEFINED, 1, args.as_mut_ptr()) }
}

// --- For the renderer -------------------------------------------------------

/// Makes a render effect running `func` (owned) under the current owner,
/// runs it, and returns the value of that first run (owned).
pub(crate) fn render_effect(ctx: *mut JsContext, func: JsValue) -> JsValue {
    let g = unsafe { graph(ctx) };
    let id = g.computation(Kind::Effect, func, UNDEFINED, STALE);
    update_computation(ctx, id);
    let n = &unsafe { graph(ctx) }.nodes[id as usize];
    // SAFETY: a dup of the node's value (a disposed node holds undefined).
    unsafe { qjs::JS_DupValue(ctx, n.value) }
}

/// A new signal object holding `value` (borrowed).
pub(crate) fn new_signal(ctx: *mut JsContext, value: JsValue) -> JsValue {
    let mut args = [value];
    // SAFETY: borrowed arguments, as `signal` takes them.
    unsafe { signal(ctx, UNDEFINED, 1, args.as_mut_ptr()) }
}

/// Writes `value` (owned) to signal object `obj` (does nothing for a dead
/// one).
pub(crate) fn set_signal(ctx: *mut JsContext, obj: JsValue, value: JsValue) {
    match this_signal(ctx, obj) {
        Ok(id) => write(ctx, id, value),
        Err(e) => unsafe {
            // Not reachable for the renderer's own signals; drop the error.
            qjs::JS_FreeValue(ctx, e);
            qjs::JS_FreeValue(ctx, qjs::JS_GetException(ctx));
            qjs::JS_FreeValue(ctx, value);
        },
    }
}

/// The number signal object `obj` holds (untracked), if it holds one.
pub(crate) fn peek_number(ctx: *mut JsContext, obj: JsValue) -> Option<f64> {
    let v = peek(ctx, obj);
    // SAFETY: an inline check and a conversion of a live value.
    unsafe {
        if !qjs::JS_IsNumber(v) {
            return None;
        }
        let mut n = 0f64;
        qjs::JS_ToFloat64(ctx, &mut n, v);
        Some(n)
    }
}

/// The current value of signal object `obj` (borrowed; untracked), if live.
pub(crate) fn peek(ctx: *mut JsContext, obj: JsValue) -> JsValue {
    let class = unsafe { graph(ctx) }.classes.signal;
    match unsafe { node_of(ctx, obj, class) } {
        Some(id) => unsafe { graph(ctx) }.nodes[id as usize].value,
        None => UNDEFINED,
    }
}

/// Makes a memo of `func` (owned) under the current owner and returns its
/// object.
pub(crate) fn memo(ctx: *mut JsContext, func: JsValue, equals_truthy: bool) -> JsValue {
    let g = unsafe { graph(ctx) };
    let id = g.computation(Kind::Memo, func, UNDEFINED, STALE);
    if equals_truthy {
        g.nodes[id as usize].equals = Equals::Truthiness;
    }
    g.nodes[id as usize].handle = true;
    let obj = object(ctx, g.classes.memo, id);
    update_computation(ctx, id);
    obj
}

/// Runs `f` as one batch: its writes apply together at the end.
pub(crate) fn batch_with<R>(ctx: *mut JsContext, f: impl FnOnce() -> R) -> R {
    run_updates(ctx, false, f)
}

/// Runs `f` with no listener (untracked).
pub(crate) fn untracked_with<R>(ctx: *mut JsContext, f: impl FnOnce() -> R) -> R {
    let g = unsafe { graph(ctx) };
    let listener = std::mem::replace(&mut g.listener, NONE);
    let r = f();
    unsafe { graph(ctx) }.listener = listener;
    r
}

/// Runs `f` in a new root owner, under the current one (context passes
/// through it); returns the root and `f`'s result. The root is disposed
/// with [`dispose_owner`].
pub(crate) fn with_root<R>(ctx: *mut JsContext, f: impl FnOnce() -> R) -> (u32, R) {
    let g = unsafe { graph(ctx) };
    let current = g.owner;
    let id = g.alloc(Kind::Root);
    let context = g.context_of(current);
    let n = &mut g.nodes[id as usize];
    n.owner = current;
    n.context = context;
    let g = unsafe { graph(ctx) };
    let (listener, owner) = (g.listener, g.owner);
    g.owner = id;
    g.listener = NONE;
    let r = run_updates(ctx, true, f);
    let g = unsafe { graph(ctx) };
    g.listener = listener;
    g.owner = owner;
    (id, r)
}

/// Whether something owns what runs now (a component, an effect, a
/// root): its cleanups run when it goes.
pub(crate) fn has_owner(ctx: *mut JsContext) -> bool {
    let owner = unsafe { graph(ctx) }.owner;
    owner != NONE && owner != UNOWNED
}

/// The current owner (for [`with_root_under`] later).
pub(crate) fn current_owner(ctx: *mut JsContext) -> u32 {
    unsafe { graph(ctx) }.owner
}

/// [`with_root`] as if run under `owner` (from [`current_owner`]): rows a
/// list makes later see the list's context.
pub(crate) fn with_root_under<R>(
    ctx: *mut JsContext,
    owner: u32,
    f: impl FnOnce() -> R,
) -> (u32, R) {
    let g = unsafe { graph(ctx) };
    let live = owner != NONE && owner != UNOWNED && !g.nodes[owner as usize].disposed;
    let previous = std::mem::replace(&mut g.owner, if live { owner } else { NONE });
    let r = with_root(ctx, f);
    unsafe { graph(ctx) }.owner = previous;
    r
}

/// Disposes owner `id` (a root from [`with_root`]), untracked.
pub(crate) fn dispose_owner(ctx: *mut JsContext, id: u32) {
    untracked_with(ctx, || dispose(ctx, id));
}

/// Sets `key` to `value` (both borrowed) in the current owner's context.
pub(crate) fn provide(ctx: *mut JsContext, key: JsValue, value: JsValue) {
    let mut args = [key, value];
    // SAFETY: borrowed arguments.
    unsafe { set_context(ctx, UNDEFINED, 2, args.as_mut_ptr()) };
}

/// The value of `key` in the current owner's context (owned), if set.
pub(crate) fn lookup(ctx: *mut JsContext, key: JsValue) -> Option<JsValue> {
    let g = unsafe { graph(ctx) };
    let mut frame = g.context_of(g.owner);
    while let Some(f) = frame {
        if unsafe { qjs::JS_IsStrictEqual(ctx, f.key, key) } {
            return Some(unsafe { qjs::JS_DupValue(ctx, f.value) });
        }
        frame = f.parent.clone();
    }
    None
}

/// Throws the error the last round kept, if any (natives that ran JS
/// through the graph end with this).
pub(crate) fn finish_value(ctx: *mut JsContext, value: JsValue) -> JsValue {
    finish(ctx, value)
}

#[cfg(test)]
mod tests {
    use rquickjs::{Context, Runtime};

    /// Runs `code` with the natives as `R` and returns its value as JSON.
    fn run(code: &str) -> String {
        let rt = Runtime::new().unwrap();
        let ctx = Context::full(&rt).unwrap();
        ctx.with(|ctx| {
            crate::state::install(&ctx, std::rc::Rc::default(), &[]);
            super::install_globals(&ctx);
            let out: String = ctx
                .eval(format!(
                    "(() => {{ const R = globalThis.__nsh_reactive; const out = (() => {{ {code} }})(); return JSON.stringify(out); }})()"
                ))
                .unwrap_or_else(|e| match e {
                    rquickjs::Error::Exception => format!("threw {:?}", ctx.catch()),
                    e => format!("error {e}"),
                });
            crate::state::uninstall(&ctx);
            out
        })
    }

    #[test]
    fn memos_are_glitch_free_and_lazy_reads_are_current() {
        // A diamond: the effect sees each change once, consistent.
        assert_eq!(
            run(r#"
                const [a, setA] = R.createSignal(1);
                const b = R.createMemo(() => a() * 2);
                const c = R.createMemo(() => a() + 1);
                const log = [];
                R.createRoot(() => R.createRenderEffect(() => log.push(`${b()}/${c()}`)));
                setA(2);
                // Read inside a batch: memos are current before it ends.
                R.batch(() => { setA(3); log.push(`read ${b()}`); });
                return log;
            "#),
            r#"["2/2","4/3","read 6","6/4"]"#
        );
    }

    #[test]
    fn batch_untrack_and_equality() {
        assert_eq!(
            run(r#"
                const [a, setA] = R.createSignal(0);
                const [b, setB] = R.createSignal(0);
                const [c, setC] = R.createSignal({}, { equals: false });
                const [d, setD] = R.createSignal(1, { equals: (x, y) => x % 2 === y % 2 });
                const log = [];
                R.createRoot(() => {
                    R.createRenderEffect(() => log.push(`ab ${a()} ${R.untrack(b)}`));
                    R.createRenderEffect(() => { c(); log.push("c"); });
                    R.createRenderEffect(() => log.push(`d ${d()}`));
                });
                R.batch(() => { setA(1); setA(2); setB(5); });
                setB(6);          // untracked: nothing
                setC(c());        // equals: false: runs again
                setD(3);          // same parity: nothing
                setD(4);
                return log;
            "#),
            r#"["ab 0 0","c","d 1","ab 2 5","c","d 4"]"#
        );
    }

    #[test]
    fn owners_clean_up_and_dispose_what_they_own() {
        assert_eq!(
            run(r#"
                const [show, setShow] = R.createSignal(true);
                const [n, setN] = R.createSignal(0);
                const log = [];
                const dispose = R.createRoot((dispose) => {
                    R.createRenderEffect(() => {
                        if (!show()) return;
                        R.onCleanup(() => log.push("outer cleanup"));
                        R.createRenderEffect(() => {
                            log.push(`inner ${n()}`);
                            R.onCleanup(() => log.push(`inner cleanup ${n()}`));
                        });
                    });
                    return dispose;
                });
                setN(1);
                setShow(false);   // disposes the inner effect
                setN(2);          // nothing runs
                setShow(true);
                dispose();
                setN(3);          // disposed: nothing
                return log;
            "#),
            // Cleanups read the signal when they run.
            r#"["inner 0","inner cleanup 1","inner 1","inner cleanup 1","outer cleanup","inner 2","inner cleanup 2","outer cleanup"]"#
        );
    }

    #[test]
    fn context_owners_and_user_effects() {
        assert_eq!(
            run(r#"
                const key = Symbol("k");
                const log = [];
                let owner;
                // A root with `dispose` (one without is unowned: no context).
                R.createRoot((dispose) => {
                    R.setContext(key, "root");
                    R.createRenderEffect(() => {
                        log.push(`render ${R.useContext(key)}`);
                        R.setContext(key, "inner");
                        owner = R.getOwner();
                    });
                    R.createEffect(() => log.push("user after render"));
                    R.createRenderEffect(() => log.push("render 2"));
                });
                R.runWithOwner(owner, () => log.push(`with owner ${R.useContext(key)}`));
                log.push(`outside ${R.useContext(key)}`);
                return log;
            "#),
            r#"["render root","render 2","user after render","with owner inner","outside undefined"]"#
        );
    }

    #[test]
    fn a_throwing_effect_throws_from_the_write_and_the_graph_goes_on() {
        assert_eq!(
            run(r#"
                const [a, setA] = R.createSignal(0);
                const log = [];
                R.createRoot(() => {
                    R.createRenderEffect(() => { if (a() === 1) throw new Error("boom"); log.push(`ok ${a()}`); });
                    R.createRenderEffect(() => log.push(`other ${a()}`));
                });
                try { setA(1); } catch (e) { log.push(`caught ${e.message}`); }
                setA(2);
                return log;
            "#),
            r#"["ok 0","other 0","other 1","caught boom","ok 2","other 2"]"#
        );
    }

    #[test]
    fn unreachable_signals_are_collected() {
        let out = run(r#"
            const [s, set] = R.createSignal(0);
            let last;
            for (let i = 0; i < 20000; i++) {
                const [g, w] = R.createSignal({ i });
                // A cycle through the value: the signal holds a closure over itself.
                w({ get: g });
                last = g;
            }
            return last().get === last;
        "#);
        assert_eq!(out, "true");
        // Collected (also during the loop): slots are reused, none is left.
        let rt = rquickjs::Runtime::new().unwrap();
        let ctx = rquickjs::Context::full(&rt).unwrap();
        // The nodes the state itself keeps (the environment's).
        let live = |raw| {
            // SAFETY: the graph `install` made; no JS runs meanwhile.
            let g = unsafe { super::graph(raw) };
            (g.nodes.len() - g.free.len(), g.nodes.len())
        };
        let before = ctx.with(|ctx| {
            crate::state::install(&ctx, std::rc::Rc::default(), &[]);
            super::install_globals(&ctx);
            let before = live(ctx.as_raw().as_ptr()).0;
            let _: () = ctx
                .eval("const R = globalThis.__nsh_reactive; for (let i = 0; i < 5000; i++) { const [g, w] = R.createSignal(0); w({ g }); }")
                .unwrap();
            before
        });
        rt.run_gc();
        let (after, slots) = ctx.with(|ctx| {
            let counts = live(ctx.as_raw().as_ptr());
            crate::state::uninstall(&ctx);
            counts
        });
        assert!(slots < 5000, "{slots} nodes for 5000 signals");
        assert_eq!(after, before, "every signal collected");
    }

    #[test]
    fn effects_made_untracked_in_a_root_still_track() {
        // The renderer's shape: `render` (a root), `createComponent`
        // (untrack), `insert` (a render effect reading an accessor).
        assert_eq!(
            run(r#"
                const [c, setC] = R.createSignal(0);
                const log = [];
                R.createRoot((dispose) => {
                    R.untrack(() => {
                        R.createRenderEffect((cur) => { const v = c(); log.push(v); return v; });
                    });
                });
                setC(1);
                return log;
            "#),
            "[0,1]"
        );
    }

    #[test]
    fn a_memo_bumped_in_a_batch_updates_its_readers() {
        // Hot reload's component proxies: a memo of a version signal.
        assert_eq!(
            run(r#"
                const [v, setV] = R.createSignal(0);
                let fn = () => "a";
                const log = [];
                R.createRoot((dispose) => {
                    const m = R.createMemo(() => { v(); return R.untrack(() => fn()); });
                    R.createRenderEffect(() => log.push(m()));
                });
                fn = () => "b";
                R.batch(() => setV((x) => x + 1));
                return log;
            "#),
            r#"["a","b"]"#
        );
    }

    #[test]
    fn computations_made_during_an_update_run_at_once() {
        // A provider made while a list renders a new item returns what its
        // render effect produced, so the effect must not wait in the queue.
        assert_eq!(
            run(r#"
                const [items, setItems] = R.createSignal([0]);
                const log = [];
                R.createRoot(() => {
                    R.createRenderEffect(() => {
                        for (const i of items()) {
                            let made;
                            R.createRenderEffect(() => { made = i; });
                            const m = R.createMemo(() => i * 10);
                            log.push([made, m()]);
                        }
                    });
                });
                setItems([0, 1]);
                return log;
            "#),
            r#"[[0,0],[0,0],[1,10]]"#
        );
    }

    #[test]
    fn effects_follow_signals() {
        assert_eq!(
            run(r#"
                const [c, setC] = R.createSignal(0);
                const log = [];
                R.createRoot(() => { R.createRenderEffect(() => log.push(c())); });
                setC(1);
                setC((n) => n + 1);
                setC(2);
                return log;
            "#),
            "[0,1,2]"
        );
    }
}
