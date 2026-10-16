//! `nsh/animated`: native values and the animation graph (`NativeNode`,
//! `NativeValue`, `Animated`'s drivers and composites), `Easing` and
//! `LayoutAnimation`. Everything runs on the UI thread, JS only describes
//! it.
//!
//! Operations are encoded as a few values (`ANIMATION_*`) and decoded into
//! `AnimationOp`s (`protocol::decode_animation_op`). Nodes and composites are
//! released when JS lets go of them (their finalizers queue the release); a
//! promise of a started animation settles when the core reports its end
//! (`Host::animations_finished`).

use std::{
    cell::{Cell, RefCell},
    ffi::{CStr, CString, c_int, c_void},
    ptr,
};

use nsh_animation::AnimationOp;
use nsh_core::js::protocol::*;
use rquickjs::{Ctx, Value, qjs};
use rustc_hash::FxHashMap;

use crate::{
    dom,
    modules::{Export, Item, Ts},
    reactive::{self, Native, arg},
};

type JsValue = qjs::JSValue;
type JsContext = qjs::JSContext;

const UNDEFINED: JsValue = qjs::JS_UNDEFINED;

/// The animations' part of a context's state.
pub(crate) struct Animations {
    classes: Cell<Option<Classes>>,
    /// The prototypes (owned): node, value, composite.
    protos: Cell<[JsValue; 3]>,
    /// Even ids: the core gives Rust components' nodes, animations and
    /// composites odd ones.
    next_node: Cell<u32>,
    next_animation: Cell<u32>,
    next_composite: Cell<u32>,
    /// Promises of started animations: id -> (resolve, composite (or
    /// undefined), callback (or undefined)), owned.
    ends: RefCell<FxHashMap<u32, (JsValue, JsValue, JsValue)>>,
}

#[derive(Clone, Copy)]
struct Classes {
    node: qjs::JSClassID,
    composite: qjs::JSClassID,
}

impl Default for Animations {
    fn default() -> Self {
        Animations {
            classes: Cell::new(None),
            protos: Cell::new([UNDEFINED; 3]),
            next_node: Cell::new(2),
            next_animation: Cell::new(2),
            next_composite: Cell::new(2),
            ends: RefCell::new(FxHashMap::default()),
        }
    }
}

impl Animations {
    /// Frees the values held (the state is unregistered).
    pub(crate) fn free_all(&mut self, raw: *mut JsContext) {
        // SAFETY: owned values; no JS runs from here on.
        unsafe {
            for p in self.protos.replace([UNDEFINED; 3]) {
                qjs::JS_FreeValue(raw, p);
            }
            for (_, (a, b, c)) in self.ends.borrow_mut().drain() {
                qjs::JS_FreeValue(raw, a);
                qjs::JS_FreeValue(raw, b);
                qjs::JS_FreeValue(raw, c);
            }
        }
    }
}

fn animations<'a>(ctx: *mut JsContext) -> &'a Animations {
    // SAFETY: a projection of the state's field, by shared reference.
    unsafe { &(*crate::state::state(ctx)).animations }
}

/// A node's data: its id and the nodes it reads (kept alive with it).
struct NodeData {
    id: u32,
    inputs: Vec<JsValue>,
}

/// A composite's data: its key, the running animation's id (0: none),
/// its spec and the values it animates (owned).
struct CompositeData {
    key: u32,
    running: u32,
    spec: JsValue,
    values: Vec<JsValue>,
}

unsafe extern "C" fn node_finalize(rt: *mut qjs::JSRuntime, obj: JsValue) {
    // SAFETY: the opaque of a node object is its boxed data.
    let p = unsafe { qjs::JS_GetOpaque(obj, qjs::JS_GetClassID(obj)) } as *mut NodeData;
    if p.is_null() {
        return;
    }
    let data = unsafe { Box::from_raw(p) };
    if let Some(state) = crate::state::of_runtime(rt) {
        // Not the op queue: a finalizer may run while it is borrowed.
        unsafe { &(*state).dom }.queue_animation_release(AnimationOp::Release { node: data.id });
    }
    for v in data.inputs {
        unsafe { qjs::JS_FreeValueRT(rt, v) };
    }
}

unsafe extern "C" fn node_mark(rt: *mut qjs::JSRuntime, obj: JsValue, mark: qjs::JS_MarkFunc) {
    // SAFETY: as in the finalizer; marking reads only.
    let p = unsafe { qjs::JS_GetOpaque(obj, qjs::JS_GetClassID(obj)) } as *const NodeData;
    if !p.is_null() {
        for &v in unsafe { &(*p).inputs } {
            unsafe { qjs::JS_MarkValue(rt, v, mark) };
        }
    }
}

unsafe extern "C" fn composite_finalize(rt: *mut qjs::JSRuntime, obj: JsValue) {
    // SAFETY: the opaque of a composite object is its boxed data.
    let p = unsafe { qjs::JS_GetOpaque(obj, qjs::JS_GetClassID(obj)) } as *mut CompositeData;
    if p.is_null() {
        return;
    }
    let data = unsafe { Box::from_raw(p) };
    if let Some(state) = crate::state::of_runtime(rt) {
        unsafe { &(*state).dom }
            .queue_animation_release(AnimationOp::ReleaseComposite { key: data.key });
    }
    unsafe { qjs::JS_FreeValueRT(rt, data.spec) };
    for v in data.values {
        unsafe { qjs::JS_FreeValueRT(rt, v) };
    }
}

unsafe extern "C" fn composite_mark(rt: *mut qjs::JSRuntime, obj: JsValue, mark: qjs::JS_MarkFunc) {
    // SAFETY: as in the finalizer.
    let p = unsafe { qjs::JS_GetOpaque(obj, qjs::JS_GetClassID(obj)) } as *const CompositeData;
    if !p.is_null() {
        unsafe { qjs::JS_MarkValue(rt, (*p).spec, mark) };
        for &v in unsafe { &(*p).values } {
            unsafe { qjs::JS_MarkValue(rt, v, mark) };
        }
    }
}

/// Registers the classes and prototypes (once per context).
fn classes(ctx: *mut JsContext) -> Classes {
    let a = animations(ctx);
    if let Some(c) = a.classes.get() {
        return c;
    }
    // SAFETY: registering classes on the context's runtime; prototypes
    // take their methods.
    unsafe {
        let rt = qjs::JS_GetRuntime(ctx);
        let class =
            |name: &'static CStr, finalizer: qjs::JSClassFinalizer, mark: qjs::JSClassGCMark| {
                let mut id: qjs::JSClassID = 0;
                qjs::JS_NewClassID(rt, &mut id);
                let def = qjs::JSClassDef {
                    class_name: name.as_ptr(),
                    finalizer,
                    gc_mark: mark,
                    call: None,
                    exotic: ptr::null_mut(),
                };
                qjs::JS_NewClass(rt, id, &def);
                id
            };
        let classes = Classes {
            node: class(c"NativeNode", Some(node_finalize), Some(node_mark)),
            composite: class(
                c"CompositeAnimation",
                Some(composite_finalize),
                Some(composite_mark),
            ),
        };
        let methods = |obj: JsValue, list: &[(&CStr, Native, c_int)]| {
            for &(name, f, length) in list {
                let func = reactive::new_function(ctx, name, f, length);
                qjs::JS_SetPropertyStr(ctx, obj, name.as_ptr(), func);
            }
        };
        let node_proto = qjs::JS_NewObject(ctx);
        methods(
            node_proto,
            &[
                (c"interpolate", node_interpolate, 1),
                (c"diffClamp", node_diff_clamp, 2),
                (c"add", node_add, 1),
                (c"sub", node_sub, 1),
                (c"mul", node_mul, 1),
                (c"div", node_div, 1),
                (c"mod", node_mod, 1),
            ],
        );
        let value_proto = qjs::JS_NewObjectProto(ctx, node_proto);
        methods(
            value_proto,
            &[
                (c"set", value_set, 1),
                (c"stop", value_stop, 0),
                (c"timing", value_timing, 2),
                (c"spring", value_spring, 2),
                (c"decay", value_decay, 1),
            ],
        );
        let composite_proto = qjs::JS_NewObject(ctx);
        methods(
            composite_proto,
            &[
                (c"start", composite_start, 1),
                (c"stop", composite_stop, 0),
                (c"reset", composite_reset, 0),
            ],
        );
        qjs::JS_SetClassProto(
            ctx,
            classes.composite,
            qjs::JS_DupValue(ctx, composite_proto),
        );
        a.protos.set([node_proto, value_proto, composite_proto]);
        a.classes.set(Some(classes));
        classes
    }
}

/// Decodes an encoded animation operation (`ANIMATION_*`, the node, its
/// values) and queues it. Takes `values`.
fn send(ctx: *mut JsContext, values: Vec<JsValue>) -> Result<(), JsValue> {
    let mut values = values;
    // SAFETY: an array taking the values, read by the decoder through a
    // context reference.
    let array = unsafe { qjs::JS_NewArrayFrom(ctx, values.len() as c_int, values.as_mut_ptr()) };
    let cx = unsafe { Ctx::from_raw(ptr::NonNull::new_unchecked(ctx)) };
    let value = unsafe { Value::from_raw(cx.clone(), array) };
    let Some(array) = value.as_array() else {
        return Ok(());
    };
    match crate::protocol::decode_animation_op(&cx, array) {
        Ok(op) => dom::push_op(ctx, nsh_core::Op::Animation(op)),
        Err(rquickjs::Error::Exception) => Err(qjs::JS_EXCEPTION),
        Err(e) => {
            let message = CString::new(format!("nsh/animated: {e}")).unwrap_or_default();
            // SAFETY: throws on a live context.
            Err(unsafe { qjs::JS_ThrowTypeError(ctx, c"%s".as_ptr(), message.as_ptr()) })
        }
    }
}

fn int(v: impl Into<f64>) -> JsValue {
    qjs::JS_NewFloat64(v.into())
}

fn string(ctx: *mut JsContext, s: &str) -> JsValue {
    // SAFETY: a new string.
    unsafe { qjs::JS_NewStringLen(ctx, s.as_ptr().cast(), s.len() as _) }
}

fn dup(ctx: *mut JsContext, v: JsValue) -> JsValue {
    // SAFETY: a live value.
    unsafe { qjs::JS_DupValue(ctx, v) }
}

fn free(ctx: *mut JsContext, v: JsValue) {
    // SAFETY: an owned value.
    unsafe { qjs::JS_FreeValue(ctx, v) };
}

fn get(ctx: *mut JsContext, obj: JsValue, key: &CStr) -> JsValue {
    // SAFETY: a property read of a live value.
    unsafe {
        if !qjs::JS_IsObject(obj) {
            return UNDEFINED;
        }
        qjs::JS_GetPropertyStr(ctx, obj, key.as_ptr())
    }
}

/// A new node object (`value`: a `NativeValue`) reading `inputs` (taken).
fn new_node(ctx: *mut JsContext, value: bool, inputs: Vec<JsValue>) -> (JsValue, u32) {
    let c = classes(ctx);
    let a = animations(ctx);
    let id = a.next_node.get();
    a.next_node.set(id + 2);
    let proto = a.protos.get()[if value { 1 } else { 0 }];
    let data = Box::into_raw(Box::new(NodeData { id, inputs }));
    // SAFETY: an object of the node class owning its data; `__nv` is what
    // the prop decoder reads.
    unsafe {
        let obj = qjs::JS_NewObjectProtoClass(ctx, proto, c.node);
        qjs::JS_SetOpaque(obj, data as *mut c_void);
        qjs::JS_SetPropertyStr(ctx, obj, c"__nv".as_ptr(), int(id));
        (obj, id)
    }
}

/// The id of node object `v`, if it is one.
fn node_id(ctx: *mut JsContext, v: JsValue) -> Option<u32> {
    let c = classes(ctx);
    // SAFETY: the opaque of a node object.
    let p = unsafe { qjs::JS_GetOpaque(v, c.node) } as *const NodeData;
    (!p.is_null()).then(|| unsafe { (*p).id })
}

fn this_node(ctx: *mut JsContext, this: JsValue) -> Result<u32, JsValue> {
    node_id(ctx, this).ok_or_else(|| {
        // SAFETY: throws on a live context.
        unsafe { qjs::JS_ThrowTypeError(ctx, c"not a native node".as_ptr()) }
    })
}

/// A node reading `inputs` made by encoded operation `sub` with `args`.
fn derived(ctx: *mut JsContext, inputs: Vec<JsValue>, sub: i32, args: Vec<JsValue>) -> JsValue {
    let (node, id) = new_node(ctx, false, inputs);
    let mut values = vec![int(sub), int(id)];
    values.extend(args);
    match send(ctx, values) {
        Ok(()) => node,
        Err(e) => {
            free(ctx, node);
            e
        }
    }
}

unsafe extern "C" fn node_interpolate(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let input = match this_node(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let config = arg(argc, argv, 0);
    let both = get(ctx, config, c"extrapolate");
    let both = if unsafe { qjs::JS_IsUndefined(both) } {
        string(ctx, "clamp")
    } else {
        both
    };
    let side = |key: &CStr| {
        let v = get(ctx, config, key);
        if unsafe { qjs::JS_IsUndefined(v) } {
            dup(ctx, both)
        } else {
            v
        }
    };
    let easing = get(ctx, config, c"easing");
    let easing = if unsafe { qjs::JS_IsUndefined(easing) } {
        string(ctx, "linear")
    } else {
        easing
    };
    let args = vec![
        int(input),
        get(ctx, config, c"inputRange"),
        get(ctx, config, c"outputRange"),
        side(c"extrapolateLeft"),
        side(c"extrapolateRight"),
        easing,
    ];
    free(ctx, both);
    derived(ctx, vec![dup(ctx, this)], ANIMATION_INTERPOLATE, args)
}

unsafe extern "C" fn node_diff_clamp(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let input = match this_node(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let args = vec![
        int(input),
        dup(ctx, arg(argc, argv, 0)),
        dup(ctx, arg(argc, argv, 1)),
    ];
    derived(ctx, vec![dup(ctx, this)], ANIMATION_DIFF_CLAMP, args)
}

/// `a op b`: a node of two operands (nodes or numbers).
fn math(ctx: *mut JsContext, op: &str, a: JsValue, b: JsValue) -> JsValue {
    let operand = |v: JsValue| match node_id(ctx, v) {
        // The decoder reads a node as its object.
        Some(_) => dup(ctx, v),
        None => dup(ctx, v),
    };
    let inputs: Vec<JsValue> = [a, b]
        .into_iter()
        .filter(|&v| node_id(ctx, v).is_some())
        .map(|v| dup(ctx, v))
        .collect();
    let args = vec![string(ctx, op), operand(a), operand(b)];
    derived(ctx, inputs, ANIMATION_MATH, args)
}

macro_rules! math_method {
    ($name:ident, $op:literal) => {
        unsafe extern "C" fn $name(
            ctx: *mut JsContext,
            this: JsValue,
            argc: c_int,
            argv: *mut JsValue,
        ) -> JsValue {
            if let Err(e) = this_node(ctx, this) {
                return e;
            }
            math(ctx, $op, this, arg(argc, argv, 0))
        }
    };
}

math_method!(node_add, "add");
math_method!(node_sub, "sub");
math_method!(node_mul, "mul");
math_method!(node_div, "div");
math_method!(node_mod, "mod");

/// `new NativeValue(value)` / `createNativeValue(value)`.
unsafe extern "C" fn native_value(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let mut v = 0f64;
    // SAFETY: converting a live value.
    unsafe { qjs::JS_ToFloat64(ctx, &mut v, arg(argc, argv, 0)) };
    let (node, id) = new_node(ctx, true, Vec::new());
    match send(
        ctx,
        vec![int(ANIMATION_VALUE), int(id), qjs::JS_NewFloat64(v)],
    ) {
        Ok(()) => node,
        Err(e) => {
            free(ctx, node);
            e
        }
    }
}

/// `new NativeNode()`: a node of its own (a pass-through input).
unsafe extern "C" fn native_node(
    ctx: *mut JsContext,
    _this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
) -> JsValue {
    new_node(ctx, false, Vec::new()).0
}

unsafe extern "C" fn value_set(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let id = match this_node(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    let v = dup(ctx, arg(argc, argv, 0));
    send(ctx, vec![int(ANIMATION_SET), int(id), v])
        .err()
        .unwrap_or(UNDEFINED)
}

unsafe extern "C" fn value_stop(
    ctx: *mut JsContext,
    this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
) -> JsValue {
    let id = match this_node(ctx, this) {
        Ok(id) => id,
        Err(e) => return e,
    };
    send(ctx, vec![int(ANIMATION_STOP), int(id)])
        .err()
        .unwrap_or(UNDEFINED)
}

/// A new promise and its resolve function (owned).
fn promise(ctx: *mut JsContext) -> (JsValue, JsValue) {
    let mut funcs = [UNDEFINED; 2];
    // SAFETY: a new promise; the reject function is not needed.
    unsafe {
        let p = qjs::JS_NewPromiseCapability(ctx, funcs.as_mut_ptr());
        qjs::JS_FreeValue(ctx, funcs[1]);
        (p, funcs[0])
    }
}

fn next_animation(ctx: *mut JsContext) -> u32 {
    let a = animations(ctx);
    let id = a.next_animation.get();
    a.next_animation.set(id + 2);
    id
}

/// Starts driver `config` (`{ type, to, ... }`, taken) on value `this`.
fn start(ctx: *mut JsContext, this: JsValue, config: JsValue) -> JsValue {
    let node = match this_node(ctx, this) {
        Ok(id) => id,
        Err(e) => {
            free(ctx, config);
            return e;
        }
    };
    let id = next_animation(ctx);
    if let Err(e) = send(ctx, vec![int(ANIMATION_START), int(node), int(id), config]) {
        return e;
    }
    let (p, resolve) = promise(ctx);
    animations(ctx)
        .ends
        .borrow_mut()
        .insert(id, (resolve, UNDEFINED, UNDEFINED));
    p
}

/// `{ type, to?, ...options }` (a copy of `options`).
fn driver_config(
    ctx: *mut JsContext,
    kind: &str,
    to: Option<JsValue>,
    options: JsValue,
) -> JsValue {
    let mut args = [options];
    // SAFETY: `merge` copies the options' properties into a new object.
    let config = crate::dom::merge_values(ctx, &mut args);
    unsafe {
        qjs::JS_SetPropertyStr(ctx, config, c"type".as_ptr(), string(ctx, kind));
        if let Some(to) = to {
            qjs::JS_SetPropertyStr(ctx, config, c"to".as_ptr(), dup(ctx, to));
        }
    }
    config
}

unsafe extern "C" fn value_timing(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let config = driver_config(ctx, "timing", Some(arg(argc, argv, 0)), arg(argc, argv, 1));
    start(ctx, this, config)
}

unsafe extern "C" fn value_spring(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let config = driver_config(ctx, "spring", Some(arg(argc, argv, 0)), arg(argc, argv, 1));
    start(ctx, this, config)
}

unsafe extern "C" fn value_decay(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let config = driver_config(ctx, "decay", None, arg(argc, argv, 0));
    start(ctx, this, config)
}

/// Settles animation `id`'s promise (`Host::animations_finished`); false
/// when it is not one of these.
pub(crate) fn finished(ctx: *mut JsContext, id: u32, done: bool) -> Result<bool, JsValue> {
    let Some((resolve, composite, callback)) = animations(ctx).ends.borrow_mut().remove(&id) else {
        return Ok(false);
    };
    let c = classes(ctx);
    // SAFETY: the composite's data, then calls of owned functions.
    unsafe {
        let p = qjs::JS_GetOpaque(composite, c.composite) as *mut CompositeData;
        if !p.is_null() && (*p).running == id {
            (*p).running = 0;
        }
        let mut result = Ok(true);
        if qjs::JS_IsFunction(ctx, callback) {
            let r = qjs::JS_NewObject(ctx);
            qjs::JS_SetPropertyStr(
                ctx,
                r,
                c"finished".as_ptr(),
                if done { qjs::JS_TRUE } else { qjs::JS_FALSE },
            );
            let mut args = [r];
            let v = qjs::JS_Call(ctx, callback, UNDEFINED, 1, args.as_mut_ptr());
            qjs::JS_FreeValue(ctx, r);
            if qjs::JS_IsException(v) {
                result = Err(v);
            } else {
                qjs::JS_FreeValue(ctx, v);
            }
        }
        let mut args = [if done { qjs::JS_TRUE } else { qjs::JS_FALSE }];
        let v = qjs::JS_Call(ctx, resolve, UNDEFINED, 1, args.as_mut_ptr());
        qjs::JS_FreeValue(ctx, v);
        for x in [resolve, composite, callback] {
            qjs::JS_FreeValue(ctx, x);
        }
        result
    }
}

// --- Composite animations --------------------------------------------------------

/// A composite of `spec` (taken) animating `values` (taken).
fn composite(ctx: *mut JsContext, spec: JsValue, values: Vec<JsValue>) -> JsValue {
    let c = classes(ctx);
    let a = animations(ctx);
    let key = a.next_composite.get();
    a.next_composite.set(key + 2);
    let data = Box::into_raw(Box::new(CompositeData {
        key,
        running: 0,
        spec,
        values,
    }));
    // SAFETY: an object of the composite class owning its data.
    unsafe {
        let obj = qjs::JS_NewObjectClass(ctx, c.composite);
        qjs::JS_SetOpaque(obj, data as *mut c_void);
        obj
    }
}

fn composite_data(ctx: *mut JsContext, v: JsValue) -> *mut CompositeData {
    let c = classes(ctx);
    // SAFETY: the opaque of a composite object.
    unsafe { qjs::JS_GetOpaque(v, c.composite) as *mut CompositeData }
}

unsafe extern "C" fn composite_stop(
    ctx: *mut JsContext,
    this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
) -> JsValue {
    let p = composite_data(ctx, this);
    // SAFETY: live composite data.
    if p.is_null() || unsafe { (*p).running } == 0 {
        return UNDEFINED;
    }
    let id = unsafe { (*p).running };
    send(ctx, vec![int(ANIMATION_STOP_ANIMATION), int(id)])
        .err()
        .unwrap_or(UNDEFINED)
}

unsafe extern "C" fn composite_reset(
    ctx: *mut JsContext,
    this: JsValue,
    _argc: c_int,
    _argv: *mut JsValue,
) -> JsValue {
    let p = composite_data(ctx, this);
    if p.is_null() {
        return UNDEFINED;
    }
    // SAFETY: live composite data.
    let key = unsafe { (*p).key };
    send(ctx, vec![int(ANIMATION_RESET_COMPOSITE), int(key)])
        .err()
        .unwrap_or(UNDEFINED)
}

/// `start(callback)`: runs the composite (stopping a run of it first); a
/// promise of whether it finished.
unsafe extern "C" fn composite_start(
    ctx: *mut JsContext,
    this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let p = composite_data(ctx, this);
    if p.is_null() {
        return UNDEFINED;
    }
    // SAFETY: live composite data, reached briefly.
    if unsafe { (*p).running } != 0 {
        let r = unsafe { composite_stop(ctx, this, 0, ptr::null_mut()) };
        if unsafe { qjs::JS_IsException(r) } {
            return r;
        }
    }
    let id = next_animation(ctx);
    let (key, spec) = unsafe {
        (*p).running = id;
        ((*p).key, qjs::JS_DupValue(ctx, (*p).spec))
    };
    if let Err(e) = send(
        ctx,
        vec![int(ANIMATION_START_COMPOSITE), int(id), int(key), spec],
    ) {
        return e;
    }
    let (promise, resolve) = promise(ctx);
    animations(ctx)
        .ends
        .borrow_mut()
        .insert(id, (resolve, dup(ctx, this), dup(ctx, arg(argc, argv, 0))));
    promise
}

/// The spec and values of composite `v` (dups), if it is one.
fn parts(ctx: *mut JsContext, v: JsValue) -> Option<(JsValue, Vec<JsValue>)> {
    let p = composite_data(ctx, v);
    // SAFETY: live composite data.
    (!p.is_null()).then(|| unsafe {
        (
            qjs::JS_DupValue(ctx, (*p).spec),
            (*p).values
                .iter()
                .map(|&x| qjs::JS_DupValue(ctx, x))
                .collect(),
        )
    })
}

fn object(ctx: *mut JsContext, fields: Vec<(&CStr, JsValue)>) -> JsValue {
    // SAFETY: a new object taking the values.
    unsafe {
        let o = qjs::JS_NewObject(ctx);
        for (k, v) in fields {
            qjs::JS_SetPropertyStr(ctx, o, k.as_ptr(), v);
        }
        o
    }
}

/// `Animated.timing / spring / decay(value, config)`: one driver as a
/// composite.
fn run_driver(ctx: *mut JsContext, kind: &str, value: JsValue, config: JsValue) -> JsValue {
    let Some(node) = node_id(ctx, value) else {
        // SAFETY: throws on a live context.
        return unsafe { qjs::JS_ThrowTypeError(ctx, c"not a native value".as_ptr()) };
    };
    let to = get(ctx, config, c"toValue");
    let has_to = !unsafe { qjs::JS_IsUndefined(to) };
    let spec = driver_config(ctx, kind, has_to.then_some(to), config);
    free(ctx, to);
    // SAFETY: the spec takes the fields.
    unsafe {
        let atom = qjs::JS_NewAtom(ctx, c"toValue".as_ptr());
        qjs::JS_DeleteProperty(ctx, spec, atom, 0);
        qjs::JS_FreeAtom(ctx, atom);
        qjs::JS_SetPropertyStr(ctx, spec, c"node".as_ptr(), int(node));
    }
    composite(ctx, spec, vec![dup(ctx, value)])
}

macro_rules! driver_fn {
    ($name:ident, $kind:literal) => {
        unsafe extern "C" fn $name(
            ctx: *mut JsContext,
            _this: JsValue,
            argc: c_int,
            argv: *mut JsValue,
        ) -> JsValue {
            run_driver(ctx, $kind, arg(argc, argv, 0), arg(argc, argv, 1))
        }
    };
}

driver_fn!(animated_timing, "timing");
driver_fn!(animated_spring, "spring");
driver_fn!(animated_decay, "decay");

/// A composite of `kind` over the composites of array `list`, with extra
/// fields.
fn group(ctx: *mut JsContext, kind: &str, list: JsValue, extra: Vec<(&CStr, JsValue)>) -> JsValue {
    let mut specs = Vec::new();
    let mut values = Vec::new();
    let mut len: i64 = 0;
    // SAFETY: reads of a live array; values got are freed.
    unsafe {
        if qjs::JS_IsArray(list) && qjs::JS_GetLength(ctx, list, &mut len) == 0 {
            for i in 0..len.max(0) {
                let c = qjs::JS_GetPropertyInt64(ctx, list, i);
                if let Some((spec, v)) = parts(ctx, c) {
                    specs.push(spec);
                    values.extend(v);
                }
                qjs::JS_FreeValue(ctx, c);
            }
        }
    }
    let children = unsafe { qjs::JS_NewArrayFrom(ctx, specs.len() as c_int, specs.as_mut_ptr()) };
    let mut fields = vec![(c"type", string(ctx, kind)), (c"children", children)];
    fields.extend(extra);
    composite(ctx, object(ctx, fields), values)
}

unsafe extern "C" fn animated_sequence(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    group(ctx, "sequence", arg(argc, argv, 0), Vec::new())
}

unsafe extern "C" fn animated_parallel(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let together = get(ctx, arg(argc, argv, 1), c"stopTogether");
    let together = if unsafe { qjs::JS_IsUndefined(together) } {
        qjs::JS_TRUE
    } else {
        together
    };
    group(
        ctx,
        "parallel",
        arg(argc, argv, 0),
        vec![(c"stopTogether", together)],
    )
}

unsafe extern "C" fn animated_delay(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let spec = object(
        ctx,
        vec![
            (c"type", string(ctx, "delay")),
            (c"ms", dup(ctx, arg(argc, argv, 0))),
        ],
    );
    composite(ctx, spec, Vec::new())
}

/// `stagger(ms, list)`: the animations starting `ms` apart.
unsafe extern "C" fn animated_stagger(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let mut ms = 0f64;
    // SAFETY: a conversion, reads of a live array; values are freed.
    unsafe { qjs::JS_ToFloat64(ctx, &mut ms, arg(argc, argv, 0)) };
    let list = arg(argc, argv, 1);
    let mut len: i64 = 0;
    let mut staged = Vec::new();
    unsafe {
        if qjs::JS_IsArray(list) && qjs::JS_GetLength(ctx, list, &mut len) == 0 {
            for i in 0..len.max(0) {
                let a = qjs::JS_GetPropertyInt64(ctx, list, i);
                let mut delay_args = [qjs::JS_NewFloat64(ms * i as f64)];
                let delay = animated_delay(ctx, UNDEFINED, 1, delay_args.as_mut_ptr());
                let mut pair = [delay, a];
                let seq_list = qjs::JS_NewArrayFrom(ctx, 2, pair.as_mut_ptr());
                staged.push(group(ctx, "sequence", seq_list, Vec::new()));
                qjs::JS_FreeValue(ctx, seq_list);
            }
        }
    }
    let all = unsafe { qjs::JS_NewArrayFrom(ctx, staged.len() as c_int, staged.as_mut_ptr()) };
    let r = group(ctx, "parallel", all, vec![(c"stopTogether", qjs::JS_TRUE)]);
    free(ctx, all);
    r
}

/// `loop(animation, { iterations, resetBeforeIteration })`.
unsafe extern "C" fn animated_loop(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let Some((spec, values)) = parts(ctx, arg(argc, argv, 0)) else {
        // SAFETY: throws on a live context.
        return unsafe { qjs::JS_ThrowTypeError(ctx, c"loop: an animation".as_ptr()) };
    };
    let config = arg(argc, argv, 1);
    let iterations = get(ctx, config, c"iterations");
    let iterations = if unsafe { qjs::JS_IsUndefined(iterations) } {
        qjs::JS_NewFloat64(-1.0)
    } else {
        iterations
    };
    let reset = get(ctx, config, c"resetBeforeIteration");
    let reset = if unsafe { qjs::JS_IsUndefined(reset) } {
        qjs::JS_TRUE
    } else {
        reset
    };
    let spec = object(
        ctx,
        vec![
            (c"type", string(ctx, "loop")),
            (c"child", spec),
            (c"iterations", iterations),
            (c"reset", reset),
        ],
    );
    composite(ctx, spec, values)
}

macro_rules! math_fn {
    ($name:ident, $op:literal) => {
        unsafe extern "C" fn $name(
            ctx: *mut JsContext,
            _this: JsValue,
            argc: c_int,
            argv: *mut JsValue,
        ) -> JsValue {
            let a = arg(argc, argv, 0);
            if node_id(ctx, a).is_none() {
                // SAFETY: throws on a live context.
                return unsafe { qjs::JS_ThrowTypeError(ctx, c"not a native node".as_ptr()) };
            }
            math(ctx, $op, a, arg(argc, argv, 1))
        }
    };
}

math_fn!(animated_add, "add");
math_fn!(animated_subtract, "sub");
math_fn!(animated_multiply, "mul");
math_fn!(animated_divide, "div");
math_fn!(animated_modulo, "mod");

unsafe extern "C" fn animated_diff_clamp(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let a = arg(argc, argv, 0);
    // SAFETY: the node method with `a` as `this`.
    unsafe { node_diff_clamp(ctx, a, argc - 1, argv.add(1)) }
}

fn functions(ctx: *mut JsContext, list: &[(&CStr, Native, c_int)]) -> JsValue {
    // SAFETY: a new object taking the functions.
    let o = unsafe { qjs::JS_NewObject(ctx) };
    for &(name, f, length) in list {
        let func = reactive::new_function(ctx, name, f, length);
        unsafe { qjs::JS_SetPropertyStr(ctx, o, name.as_ptr(), func) };
    }
    o
}

fn animated(ctx: *mut JsContext) -> JsValue {
    functions(
        ctx,
        &[
            (c"timing", animated_timing, 2),
            (c"spring", animated_spring, 2),
            (c"decay", animated_decay, 2),
            (c"sequence", animated_sequence, 1),
            (c"parallel", animated_parallel, 2),
            (c"stagger", animated_stagger, 2),
            (c"delay", animated_delay, 1),
            (c"loop", animated_loop, 2),
            (c"add", animated_add, 2),
            (c"subtract", animated_subtract, 2),
            (c"multiply", animated_multiply, 2),
            (c"divide", animated_divide, 2),
            (c"modulo", animated_modulo, 2),
            (c"diffClamp", animated_diff_clamp, 3),
        ],
    )
}

/// A constructor-or-function `name` whose objects have `proto`.
fn class_function(ctx: *mut JsContext, name: &CStr, f: Native, proto_index: usize) -> JsValue {
    classes(ctx);
    let proto = animations(ctx).protos.get()[proto_index];
    // SAFETY: a C function callable with or without `new`; its `prototype`
    // is the objects' (so `instanceof` works).
    unsafe {
        let func = qjs::JS_NewCFunction2(
            ctx,
            Some(f),
            name.as_ptr(),
            1,
            qjs::JSCFunctionEnum_JS_CFUNC_constructor_or_func,
            0,
        );
        qjs::JS_SetConstructor(ctx, func, proto);
        func
    }
}

// --- Easing and LayoutAnimation ------------------------------------------------------

/// The core's animation tables as JS (cached JSON, converted on use).
fn table(ctx: *mut JsContext, key: &str) -> JsValue {
    static TABLES: std::sync::OnceLock<nsh_core::JsonValue> = std::sync::OnceLock::new();
    let tables = TABLES.get_or_init(nsh_core::js::animation::tables);
    crate::host::json_to_js(ctx, &tables[key])
}

unsafe extern "C" fn easing_curve(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
    _magic: c_int,
    data: *mut JsValue,
) -> JsValue {
    // SAFETY: two data values: the curve's name and its default parameter.
    let (name, default) = unsafe { (*data, *data.add(1)) };
    let p = arg(argc, argv, 0);
    let param = if unsafe { qjs::JS_IsUndefined(p) } {
        dup(ctx, default)
    } else {
        dup(ctx, p)
    };
    object(
        ctx,
        vec![
            (c"curve", dup(ctx, name)),
            (c"param", param),
            (c"mode", string(ctx, "in")),
        ],
    )
}

/// `Easing.in / out / inOut(e)`: a curve in that mode (`out` also turns a
/// Bézier or the named in/out easings around).
unsafe extern "C" fn easing_mode(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
    magic: c_int,
    _data: *mut JsValue,
) -> JsValue {
    let e = arg(argc, argv, 0);
    let mode = match magic {
        0 => "in",
        1 => "out",
        _ => "inOut",
    };
    // SAFETY: inline checks and reads of a live value.
    unsafe {
        if qjs::JS_IsArray(e) && magic == 1 {
            let n = |i| {
                let v = qjs::JS_GetPropertyInt64(ctx, e, i);
                let mut x = 0f64;
                qjs::JS_ToFloat64(ctx, &mut x, v);
                qjs::JS_FreeValue(ctx, v);
                x
            };
            let mut values = [
                qjs::JS_NewFloat64(1.0 - n(2)),
                qjs::JS_NewFloat64(1.0 - n(3)),
                qjs::JS_NewFloat64(1.0 - n(0)),
                qjs::JS_NewFloat64(1.0 - n(1)),
            ];
            return qjs::JS_NewArrayFrom(ctx, 4, values.as_mut_ptr());
        }
        if qjs::JS_IsObject(e) && !qjs::JS_IsArray(e) {
            let mut args = [e];
            let copy = crate::dom::merge_values(ctx, &mut args);
            qjs::JS_SetPropertyStr(ctx, copy, c"mode".as_ptr(), string(ctx, mode));
            return copy;
        }
        if magic == 1 {
            match dom::string_arg(ctx, e).as_deref() {
                Some("easeIn") => return string(ctx, "easeOut"),
                Some("easeOut") => return string(ctx, "easeIn"),
                _ => {}
            }
        }
        qjs::JS_DupValue(ctx, e)
    }
}

unsafe extern "C" fn easing_bezier(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let mut values: Vec<JsValue> = (0..4).map(|i| dup(ctx, arg(argc, argv, i))).collect();
    // SAFETY: the array takes the values.
    unsafe { qjs::JS_NewArrayFrom(ctx, 4, values.as_mut_ptr()) }
}

fn easing(ctx: *mut JsContext) -> JsValue {
    // SAFETY: a new object taking its fields.
    let o = unsafe { qjs::JS_NewObject(ctx) };
    let curves = table(ctx, "curves");
    let mut len: i64 = 0;
    // SAFETY: reads of the table's arrays; values got are freed or moved.
    unsafe {
        qjs::JS_GetLength(ctx, curves, &mut len);
        for i in 0..len.max(0) {
            let c = qjs::JS_GetPropertyInt64(ctx, curves, i);
            let name = qjs::JS_GetPropertyInt64(ctx, c, 0);
            let param = qjs::JS_GetPropertyInt64(ctx, c, 1);
            let default = qjs::JS_GetPropertyInt64(ctx, c, 2);
            let key =
                CString::new(dom::string_arg(ctx, name).unwrap_or_default()).unwrap_or_default();
            let value = if qjs::JS_IsNull(param) {
                object(
                    ctx,
                    vec![
                        (c"curve", qjs::JS_DupValue(ctx, name)),
                        (c"mode", string(ctx, "in")),
                    ],
                )
            } else {
                let mut data = [name, default];
                qjs::JS_NewCFunctionData(ctx, Some(easing_curve), 1, 0, 2, data.as_mut_ptr())
            };
            qjs::JS_SetPropertyStr(ctx, o, key.as_ptr(), value);
            for v in [c, name, param, default] {
                qjs::JS_FreeValue(ctx, v);
            }
        }
        qjs::JS_FreeValue(ctx, curves);
        qjs::JS_SetPropertyStr(ctx, o, c"linear".as_ptr(), string(ctx, "linear"));
        qjs::JS_SetPropertyStr(ctx, o, c"ease".as_ptr(), string(ctx, "ease"));
        qjs::JS_SetPropertyStr(
            ctx,
            o,
            c"bezier".as_ptr(),
            reactive::new_function(ctx, c"bezier", easing_bezier, 4),
        );
        for (name, magic) in [(c"in", 0), (c"out", 1), (c"inOut", 2)] {
            let f = qjs::JS_NewCFunctionData(ctx, Some(easing_mode), 1, magic, 0, ptr::null_mut());
            qjs::JS_SetPropertyStr(ctx, o, name.as_ptr(), f);
        }
    }
    o
}

/// `LayoutAnimation.configureNext(config, onEnd)`: the next commit's
/// changes animate.
unsafe extern "C" fn configure_next(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let config = arg(argc, argv, 0);
    // SAFETY: a context reference for the decoder, which takes a dup.
    let cx = unsafe { Ctx::from_raw(ptr::NonNull::new_unchecked(ctx)) };
    let value = unsafe { Value::from_raw(cx, qjs::JS_DupValue(ctx, config)) };
    match crate::protocol::decode_command("layoutAnimation", value) {
        Ok(Some(command)) => {
            if let Err(e) = dom::push_op(ctx, nsh_core::Op::Command { id: 1, command }) {
                return e;
            }
        }
        Ok(None) => {}
        Err(_) => return qjs::JS_EXCEPTION,
    }
    let on_end = arg(argc, argv, 1);
    // SAFETY: an inline check; `setTimeout(onEnd, duration)`.
    unsafe {
        if qjs::JS_IsFunction(ctx, on_end) {
            let global = qjs::JS_GetGlobalObject(ctx);
            let set_timeout = qjs::JS_GetPropertyStr(ctx, global, c"setTimeout".as_ptr());
            let mut args = [on_end, get(ctx, config, c"duration")];
            let r = qjs::JS_Call(ctx, set_timeout, UNDEFINED, 2, args.as_mut_ptr());
            for v in [r, args[1], set_timeout, global] {
                qjs::JS_FreeValue(ctx, v);
            }
        }
    }
    UNDEFINED
}

/// `LayoutAnimation.create(duration, type, property)`.
unsafe extern "C" fn layout_create(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
) -> JsValue {
    let or = |i: usize, default: &str| {
        let v = arg(argc, argv, i);
        if unsafe { qjs::JS_IsUndefined(v) } {
            string(ctx, default)
        } else {
            dup(ctx, v)
        }
    };
    let (kind, property) = (or(1, "easeInEaseOut"), or(2, "opacity"));
    let phase = |with_property: bool| {
        let mut fields = vec![(c"type", dup(ctx, kind))];
        if with_property {
            fields.push((c"property", dup(ctx, property)));
        }
        object(ctx, fields)
    };
    let o = object(
        ctx,
        vec![
            (c"duration", dup(ctx, arg(argc, argv, 0))),
            (c"create", phase(true)),
            (c"update", phase(false)),
            (c"delete", phase(true)),
        ],
    );
    free(ctx, kind);
    free(ctx, property);
    o
}

/// `LayoutAnimation.easeInEaseOut / linear / spring(onEnd)`: a preset.
unsafe extern "C" fn layout_preset(
    ctx: *mut JsContext,
    _this: JsValue,
    argc: c_int,
    argv: *mut JsValue,
    magic: c_int,
    _data: *mut JsValue,
) -> JsValue {
    let presets = table(ctx, "presets");
    let name = match magic {
        0 => c"easeInEaseOut",
        1 => c"linear",
        _ => c"spring",
    };
    let preset = get(ctx, presets, name);
    let mut args = [preset, arg(argc, argv, 0)];
    // SAFETY: borrowed arguments.
    let r = unsafe { configure_next(ctx, UNDEFINED, 2, args.as_mut_ptr()) };
    free(ctx, preset);
    free(ctx, presets);
    r
}

/// An object naming each of `names` as itself.
fn names_object(ctx: *mut JsContext, names: JsValue) -> JsValue {
    // SAFETY: a new object; reads of a live array, values freed or moved.
    unsafe {
        let o = qjs::JS_NewObject(ctx);
        let mut len: i64 = 0;
        qjs::JS_GetLength(ctx, names, &mut len);
        for i in 0..len.max(0) {
            let n = qjs::JS_GetPropertyInt64(ctx, names, i);
            let key = CString::new(dom::string_arg(ctx, n).unwrap_or_default()).unwrap_or_default();
            qjs::JS_SetPropertyStr(ctx, o, key.as_ptr(), n);
        }
        qjs::JS_FreeValue(ctx, names);
        o
    }
}

fn layout_animation(ctx: *mut JsContext) -> JsValue {
    let o = functions(
        ctx,
        &[
            (c"configureNext", configure_next, 2),
            (c"create", layout_create, 3),
        ],
    );
    // SAFETY: the object takes its fields.
    unsafe {
        qjs::JS_SetPropertyStr(
            ctx,
            o,
            c"Types".as_ptr(),
            names_object(ctx, table(ctx, "types")),
        );
        qjs::JS_SetPropertyStr(
            ctx,
            o,
            c"Properties".as_ptr(),
            names_object(ctx, table(ctx, "properties")),
        );
        qjs::JS_SetPropertyStr(ctx, o, c"Presets".as_ptr(), table(ctx, "presets"));
        for (name, magic) in [(c"easeInEaseOut", 0), (c"linear", 1), (c"spring", 2)] {
            let f =
                qjs::JS_NewCFunctionData(ctx, Some(layout_preset), 1, magic, 0, ptr::null_mut());
            qjs::JS_SetPropertyStr(ctx, o, name.as_ptr(), f);
        }
    }
    o
}

/// `nsh/animated`.
pub(crate) const EXPORTS: &[Item] = &[
    (
        "NativeNode",
        Export::Value(|ctx| class_function(ctx, c"NativeNode", native_node, 0)),
        Ts::Class(
            "A node of the native animation graph: evaluated on the UI thread.",
            r#" {
  #private;
  constructor(inputs?: NativeNode[]);
  /** Maps this node through piecewise ranges: numbers, colors or angles
   * (`"45deg"`, `"1rad"`). Outside the input range it clamps by default;
   * `easing` shapes each segment. */
  interpolate(config: {
    inputRange: number[];
    outputRange: number[] | string[];
    extrapolate?: Extrapolation;
    extrapolateLeft?: Extrapolation;
    extrapolateRight?: Extrapolation;
    easing?: Easing;
  }): NativeNode;
  /** Follows this node's changes, kept within `min..max` (a header that
   * hides while content scrolls up and comes back as it scrolls down). */
  diffClamp(min: number, max: number): NativeNode;
  add(b: NativeNode | number): NativeNode;
  sub(b: NativeNode | number): NativeNode;
  mul(b: NativeNode | number): NativeNode;
  div(b: NativeNode | number): NativeNode;
  /** Modulo, never negative for a positive `b` (e.g. angles in 0..360). */
  mod(b: NativeNode | number): NativeNode;
}"#,
        ),
    ),
    (
        "NativeValue",
        Export::Value(|ctx| class_function(ctx, c"NativeValue", native_value, 1)),
        Ts::Class(
            "A settable, animatable native value.",
            r#" extends NativeNode {
  constructor(value: number);
  /** Jumps to `value`, stopping any animation. */
  set(value: number): void;
  /** Stops the running animation where it is. */
  stop(): void;
  /** Animates to `to` over `duration` ms. Resolves with `finished`. */
  timing(to: number, options?: { duration?: number; easing?: Easing; delay?: number }): Promise<boolean>;
  /** Springs to `to`. Resolves with `finished`. */
  spring(to: number, options?: { stiffness?: number; damping?: number; mass?: number; velocity?: number }): Promise<boolean>;
  /** Coasts from `velocity` (units per second) and slows down. */
  decay(options?: { velocity?: number; deceleration?: number }): Promise<boolean>;
}"#,
        ),
    ),
    (
        "createNativeValue",
        Export::Function(native_value, 1),
        Ts::Function(
            "Creates a native value. Bind it (or nodes derived from it) to style\nproperties such as `opacity`, `transform` and `backgroundColor`, to a\nscroll position (`scrollValue`) or to drags (`pan`).",
            &["(value?: number): NativeValue"],
        ),
    ),
    (
        "Animated",
        Export::Value(animated),
        Ts::Const(
            "Animations composed natively (React Native's `Animated`): each step\nstarts on the UI thread the moment the previous one ends.",
            "AnimatedModule",
        ),
    ),
    (
        "Easing",
        Export::Value(easing),
        Ts::Const(
            "Easing functions (React Native's `Easing`): curves in their \"in\" form,\nturned around with `out` / `inOut`.",
            "EasingModule",
        ),
    ),
    (
        "LayoutAnimation",
        Export::Value(layout_animation),
        Ts::Const(
            "Animates the layout changes of the next render natively: frames move,\nnew views fade in, removed views fade out.",
            "LayoutAnimationModule",
        ),
    ),
];

/// `nsh/animated`'s types (with the easing and layout animation types of
/// `nsh_core::js::animation::declarations`).
pub(crate) const TYPES: &str = r#"export type Extrapolation = "extend" | "clamp" | "identity";

/** Easing functions (React Native's `Easing`). */
export interface EasingModule extends EasingCurves {
  linear: Easing;
  ease: Easing;
  bezier(x1: number, y1: number, x2: number, y2: number): Easing;
  in(e: Easing): Easing;
  out(e: Easing): Easing;
  inOut(e: Easing): Easing;
}

/** An animation built with `Animated`: runs natively, steps and all. */
export interface CompositeAnimation {
  /** Runs it (again). Resolves (and calls `callback`) with whether it
   * finished rather than being stopped. */
  start(callback?: (result: { finished: boolean }) => void): Promise<boolean>;
  /** Stops it where it is. */
  stop(): void;
  /** Stops it and puts its values back where they were when it started. */
  reset(): void;
}

/** Animations composed natively (React Native's `Animated`). */
export interface AnimatedModule {
  timing(value: NativeValue, config: { toValue: number; duration?: number; easing?: Easing; delay?: number }): CompositeAnimation;
  spring(value: NativeValue, config: { toValue: number; stiffness?: number; damping?: number; mass?: number; velocity?: number }): CompositeAnimation;
  decay(value: NativeValue, config?: { velocity?: number; deceleration?: number }): CompositeAnimation;
  sequence(list: CompositeAnimation[]): CompositeAnimation;
  parallel(list: CompositeAnimation[], config?: { stopTogether?: boolean }): CompositeAnimation;
  stagger(ms: number, list: CompositeAnimation[]): CompositeAnimation;
  delay(ms: number): CompositeAnimation;
  loop(animation: CompositeAnimation, config?: { iterations?: number; resetBeforeIteration?: boolean }): CompositeAnimation;
  add(a: NativeNode, b: NativeNode | number): NativeNode;
  subtract(a: NativeNode, b: NativeNode | number): NativeNode;
  multiply(a: NativeNode, b: NativeNode | number): NativeNode;
  divide(a: NativeNode, b: NativeNode | number): NativeNode;
  modulo(a: NativeNode, b: NativeNode | number): NativeNode;
  diffClamp(a: NativeNode, min: number, max: number): NativeNode;
}

/** `LayoutAnimation`. */
export interface LayoutAnimationModule {
  /** Animates the changes of the next commit. `onEnd` runs when done. */
  configureNext(config: LayoutAnimationConfig, onEnd?: () => void): void;
  /** A config with `duration`, the same curve for all three phases. */
  create: (duration: number, type?: LayoutAnimationType, property?: LayoutAnimationProperty) => LayoutAnimationConfig;
  Types: LayoutAnimationTypes;
  Properties: LayoutAnimationProperties;
  Presets: LayoutAnimationPresets;
  easeInEaseOut(onEnd?: () => void): void;
  linear(onEnd?: () => void): void;
  spring(onEnd?: () => void): void;
}
"#;

#[cfg(test)]
mod tests {
    use nsh_core::Op;

    use crate::Host;

    #[test]
    fn animations_settle_their_promises_and_callbacks() {
        let host = Host::new().unwrap();
        host.eval_source(
            r#"
            import { Animated, NativeValue } from "nsh/animated";
            import { text } from "nsh:rt";
            const v = new NativeValue(0);
            globalThis.log = [];
            v.timing(1, { duration: 10 }).then((f) => log.push(`timing ${f}`));
            Animated.sequence([Animated.delay(5), Animated.timing(v, { toValue: 0 })])
                .start((r) => log.push(`callback ${r.finished}`))
                .then((f) => log.push(`sequence ${f}`));
            globalThis.report = () => text(log.join("|"));
            globalThis.keep = v;
            "#,
        )
        .unwrap();
        let mut ops = Vec::new();
        host.take_ops(&mut ops);
        let starts: Vec<u32> = ops
            .iter()
            .filter_map(|o| match o {
                Op::Animation(nsh_animation::AnimationOp::Start { animation, .. }) => {
                    Some(*animation)
                }
                Op::Animation(nsh_animation::AnimationOp::StartComposite { animation, .. }) => {
                    Some(*animation)
                }
                _ => None,
            })
            .collect();
        assert_eq!(starts, [2, 4], "{ops:?}");
        host.animations_finished(&[
            nsh_animation::Finished {
                animation: 2,
                finished: true,
            },
            nsh_animation::Finished {
                animation: 4,
                finished: false,
            },
        ]);
        host.eval_script("report.js", "report()").unwrap();
        let mut ops = Vec::new();
        host.take_ops(&mut ops);
        let log = ops.iter().find_map(|o| match o {
            Op::CreateText { text, .. } => Some(text.clone()),
            _ => None,
        });
        assert_eq!(
            log.as_deref(),
            // Callbacks run at once, promise reactions after (as in the JS
            // runtime).
            Some("callback false|timing true|sequence false")
        );
    }
}
