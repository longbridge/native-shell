//! `NshView`: the AppKit view of one node.
//!
//! The core lays everything out, so a view never measures itself: it sits
//! at the frame the core gave it ([`NshView::place`]: the frame's origin,
//! a sticky shift, mirrored lists, and the view's transform on its layer).
//! Views are flipped (y down) and layer-backed; scroll views move their
//! children by the bounds origin of their [`container`](NshView::container)
//! (as `NSClipView` scrolls), so a scroll is one call.
//!
//! What a view looks like is mostly Core Animation: its layer's background,
//! corners, border and shadow, and sublayers for the rest
//! ([`crate::decor`]). Text and canvas drawings are drawn into the layer by
//! `drawRect:` (CoreText / Core Graphics); other views use `updateLayer`
//! and get no backing store.
//!
//! Children are subviews of the view, or of its container when it has one:
//! scroll views always do, and a view that clips its children and casts a
//! shadow does (clipping a layer clips its shadow too). Native AppKit views
//! a view hosts (a text field, a header) are [`ViewState::natives`]: they
//! sit above the children, at their own rectangles.
//!
//! The root view of a surface takes every mouse event over an app's view
//! (`hitTest:`) and hands it to [`crate::input`], which picks the view
//! itself (transforms, `overflow`, `pointerEvents`, hit slop); hosted
//! native views get their own events.

use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Weak,
};

use nsh_core::{
    EventMask, MAX_TRANSFORMS, Matrix, NodeId, PointerEvents, ROOT_ID, Rect, ViewKind, ViewProps,
    ViewStyle,
};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
    rc::Retained, runtime::AnyObject,
};
use objc2_app_kit::{
    NSEvent, NSGraphicsContext, NSResponder, NSTrackingArea, NSTrackingAreaOptions, NSView,
    NSViewLayerContentsRedrawPolicy, NSWindowOrderingMode,
};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect};
use objc2_quartz_core::CATransform3D;

use crate::surface::Surface;

/// Values native animations set on a view; they win over the style's.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct AnimValues {
    /// Animated opacity.
    pub(crate) opacity: Option<f32>,
    /// Animated background color.
    pub(crate) background: Option<nsh_core::Color>,
    /// Animated entries of the transform list (by position).
    pub(crate) transform: [Option<f32>; MAX_TRANSFORMS],
    /// Animated border color (every side).
    pub(crate) border_color: Option<nsh_core::Color>,
    /// Animated text color (a text view's runs, an input's text).
    pub(crate) color: Option<nsh_core::Color>,
    /// Animated shadow color.
    pub(crate) shadow_color: Option<nsh_core::Color>,
    /// Animated shadow opacity.
    pub(crate) shadow_opacity: Option<f32>,
    /// Animated shadow blur radius.
    pub(crate) shadow_radius: Option<f32>,
}

/// Scroll position and extent of a [`ViewKind::Scroll`] view.
#[derive(Debug, Default, Clone)]
pub(crate) struct ScrollState {
    /// Current offset (points; content coordinates of the viewport's top
    /// left).
    pub(crate) x: f32,
    /// Vertical offset.
    pub(crate) y: f32,
    /// Size of the scrollable content.
    pub(crate) content: (f32, f32),
    /// Scrolls horizontally.
    pub(crate) horizontal: bool,
    /// Shows the scroll indicator.
    pub(crate) indicator: bool,
    /// How it scrolls (paging, snapping, insets...).
    pub(crate) behavior: nsh_core::ScrollBehavior,
    /// Scrolls the core asked for (`ScrollBy`, `scrollTo`) that were
    /// applied: reported with each offset.
    pub(crate) applied: u32,
    /// The sticky cells: (cell, start, end).
    pub(crate) sticky: Vec<nsh_core::StickyCell>,
    /// A horizontal list laid out right to left: children are placed
    /// mirrored (offsets stay left to right).
    pub(crate) mirrored: bool,
    /// Gestures, glides and the indicator ([`crate::scroll`]).
    pub(crate) live: crate::scroll::Live,
}

impl ScrollState {
    /// The largest offsets.
    pub(crate) fn max(&self, viewport: (f32, f32)) -> (f32, f32) {
        let inset = self.behavior.content_inset;
        (
            (self.content.0 + inset.left + inset.right - viewport.0).max(0.0),
            (self.content.1 + inset.top + inset.bottom - viewport.1).max(0.0),
        )
    }
}

/// A native AppKit view a view hosts at `rect` (view coordinates).
#[derive(Debug, Clone)]
pub(crate) struct Native {
    /// The view (a subview of the node's view).
    pub(crate) view: Retained<NSView>,
    /// Where it goes.
    pub(crate) rect: Rect,
}

/// Everything the mounter set on one view.
pub(crate) struct ViewState {
    /// Node id.
    pub(crate) id: NodeId,
    /// What it is.
    pub(crate) kind: ViewKind,
    /// Frame in the parent (points).
    pub(crate) frame: Rect,
    /// Visual style.
    pub(crate) style: Box<ViewStyle>,
    /// Native animation values.
    pub(crate) anim: AnimValues,
    /// Events JS listens to.
    pub(crate) events: EventMask,
    /// Touch, accessibility and test properties.
    pub(crate) props: Box<ViewProps>,
    /// States to report (`TrackStates`), and the current ones.
    pub(crate) tracked: u8,
    /// Current interaction states (pressed, hovered, focused).
    pub(crate) states: u8,
    /// Extra translation (sticky list cells, screen transitions).
    pub(crate) shift: (f32, f32),
    /// Extra opacity (press feedback, screen and modal transitions).
    pub(crate) fade: f32,
    /// Scroll state (scroll views).
    pub(crate) scroll: Option<ScrollState>,
    /// Text content (text views).
    pub(crate) text: Option<crate::text::DrawnText>,
    /// Image content (image views).
    pub(crate) image: Option<crate::image::ImageState>,
    /// Canvas content.
    pub(crate) canvas: Option<crate::canvas::CanvasState>,
    /// Text field (input views).
    pub(crate) input: Option<std::rc::Rc<crate::text_input::InputState>>,
    /// Hosted AppKit views.
    pub(crate) natives: Vec<Native>,
    /// The area descendants cover outside the bounds (`MountItem::Overflow`):
    /// hit testing looks there too.
    pub(crate) overflow: Option<Rect>,
    /// Running layout / style transitions and keyframes.
    pub(crate) live: Option<Box<crate::transition::Live>>,
    /// The layers decoration added ([`crate::decor`]).
    pub(crate) decor: crate::decor::Decor,
    /// What accessibility last gave AppKit.
    pub(crate) access: crate::access::Applied,
}

impl ViewState {
    fn new(id: NodeId, kind: ViewKind) -> Self {
        ViewState {
            id,
            kind,
            frame: Rect::default(),
            style: Box::default(),
            anim: AnimValues::default(),
            events: EventMask::default(),
            props: Box::default(),
            tracked: 0,
            states: 0,
            shift: (0.0, 0.0),
            fade: 1.0,
            scroll: matches!(kind, ViewKind::Scroll).then(ScrollState::default),
            text: None,
            image: None,
            canvas: None,
            input: None,
            natives: Vec::new(),
            overflow: None,
            live: None,
            decor: Default::default(),
            access: Default::default(),
        }
    }

    /// The opacity it is drawn with.
    pub(crate) fn opacity(&self) -> f32 {
        let live = self.live.as_ref().map_or(1.0, |l| l.opacity);
        self.anim.opacity.unwrap_or(self.style.opacity) * self.fade * live
    }

    /// Its shadow, with animated color, opacity and radius.
    pub(crate) fn shadow(&self) -> nsh_core::Shadow {
        let mut s = self.style.shadow;
        let a = &self.anim;
        s.color = a.shadow_color.unwrap_or(s.color);
        s.opacity = a.shadow_opacity.unwrap_or(s.opacity);
        s.radius = a.shadow_radius.unwrap_or(s.radius);
        s
    }

    /// Its borders, with an animated color.
    pub(crate) fn borders(&self) -> nsh_core::Borders {
        let mut b = self.style.borders;
        if let Some(c) = self.anim.border_color {
            b.colors = [c; 4];
        }
        b
    }

    /// Its background color, animated or not.
    pub(crate) fn background(&self) -> nsh_core::Color {
        self.anim.background.unwrap_or(self.style.background_color)
    }

    /// The corner radii (top left, top right, bottom right, bottom left),
    /// clamped as CSS clamps them to the box.
    pub(crate) fn radii(&self) -> [f32; 4] {
        let s = &self.style;
        let mut r: [f32; 4] =
            std::array::from_fn(|i| s.corner_radius[i].unwrap_or(s.border_radius));
        let (w, h) = (self.frame.width, self.frame.height);
        let sums = [r[0] + r[1], r[1] + r[2], r[2] + r[3], r[3] + r[0]];
        let lens = [w, h, w, h];
        let f = sums
            .iter()
            .zip(lens)
            .filter(|(s, _)| **s > 0.0)
            .map(|(s, l)| l / s)
            .fold(1.0f32, f32::min);
        if f < 1.0 {
            r.iter_mut().for_each(|v| *v *= f);
        }
        r
    }

    /// True if its children are clipped to its rounded bounds.
    pub(crate) fn clips(&self) -> bool {
        self.style.overflow_hidden
            || matches!(
                self.kind,
                ViewKind::Scroll | ViewKind::Image | ViewKind::Stack | ViewKind::Input
            )
    }

    /// True if it draws content with `drawRect:` (text, canvas).
    pub(crate) fn draws(&self) -> bool {
        self.text.is_some() || self.canvas.is_some()
    }

    /// Its transform (static entries, overridden by animated ones) around
    /// its transform origin, as a row-vector matrix about its top left
    /// corner (the layer's anchor), or `None` when it has none.
    pub(crate) fn transform(&self) -> Option<Matrix> {
        let t = &self.style.transform;
        let animated = self.anim.transform.iter().any(Option::is_some);
        let extra = self.live.as_ref().and_then(|l| l.extra());
        if extra.is_none() && (t.ops().is_empty() || (!animated && t.is_identity())) {
            return None;
        }
        let (w, h) = (self.frame.width, self.frame.height);
        let mut m = t.matrix(|i| self.anim.transform.get(i).copied().flatten());
        if let Some(extra) = extra {
            m = extra.apply(m, h);
        }
        let o = self.style.transform_origin;
        let (ox, oy, oz) = (o.x.resolve(w) as f64, o.y.resolve(h) as f64, o.z as f64);
        // To the origin, transform, and back (row vectors).
        Some(
            translation(-ox, -oy, -oz)
                .mul(&m)
                .mul(&translation(ox, oy, oz)),
        )
    }
}

/// A translation matrix (row vectors, as Core Animation).
pub(crate) fn translation(x: f64, y: f64, z: f64) -> Matrix {
    let mut m = Matrix::IDENTITY;
    (m.0[12], m.0[13], m.0[14]) = (x, y, z);
    m
}

/// `m` as a Core Animation matrix.
pub(crate) fn ca_transform(m: &Matrix) -> CATransform3D {
    let m = m.0;
    CATransform3D {
        m11: m[0],
        m12: m[1],
        m13: m[2],
        m14: m[3],
        m21: m[4],
        m22: m[5],
        m23: m[6],
        m24: m[7],
        m31: m[8],
        m32: m[9],
        m33: m[10],
        m34: m[11],
        m41: m[12],
        m42: m[13],
        m43: m[14],
        m44: m[15],
    }
}

/// The identity transform.
pub(crate) const IDENTITY: CATransform3D = CATransform3D {
    m11: 1.0,
    m12: 0.0,
    m13: 0.0,
    m14: 0.0,
    m21: 0.0,
    m22: 1.0,
    m23: 0.0,
    m24: 0.0,
    m31: 0.0,
    m32: 0.0,
    m33: 1.0,
    m34: 0.0,
    m41: 0.0,
    m42: 0.0,
    m43: 0.0,
    m44: 1.0,
};

/// A rectangle in points.
pub(crate) fn ns_rect(x: f32, y: f32, width: f32, height: f32) -> NSRect {
    CGRect::new(
        CGPoint::new(x as CGFloat, y as CGFloat),
        CGSize::new(width.max(0.0) as CGFloat, height.max(0.0) as CGFloat),
    )
}

/// Ivars of [`ViewObj`].
pub(crate) struct Ivars {
    state: RefCell<ViewState>,
    /// Holds the children (scroll views; views that clip their children
    /// and cast a shadow).
    container: RefCell<Option<Retained<Container>>>,
    /// The surface (root views only): mouse, keyboard and resizes go there.
    surface: RefCell<Weak<Surface>>,
    /// Set once the root's tracking area is installed.
    tracking: Cell<bool>,
}

define_class!(
    /// The AppKit class of [`NshView`].
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    #[name = "NshView"]
    pub(crate) struct ViewObj;

    unsafe impl NSObjectProtocol for ViewObj {}

    impl ViewObj {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(wantsUpdateLayer))]
        fn wants_update_layer(&self) -> bool {
            self.ivars().state.try_borrow().is_ok_and(|s| !s.draws())
        }

        #[unsafe(method(updateLayer))]
        fn update_layer(&self) {}

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let Some(context) = NSGraphicsContext::currentContext() else {
                return;
            };
            let cg = context.CGContext();
            let Ok(state) = self.ivars().state.try_borrow() else {
                return;
            };
            crate::text::draw(&cg, &state);
            crate::canvas::draw(&cg, &state);
        }

        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, point: NSPoint) -> Option<Retained<NSView>> {
            // SAFETY: NSView's own hit testing.
            let hit: Option<Retained<NSView>> = unsafe { msg_send![super(self), hitTest: point] };
            // The root takes every event over the app's own views and
            // picks the view itself (`crate::input`); hosted AppKit views
            // get theirs.
            let root = self.ivars().surface.borrow().strong_count() > 0;
            match hit {
                Some(v) if root && is_ours(&v) => Some(Retained::from(&**self)),
                other => other,
            }
        }

        #[unsafe(method(acceptsFirstMouse:))]
        fn accepts_first_mouse(&self, _event: Option<&NSEvent>) -> bool {
            true
        }

        #[unsafe(method(acceptsFirstResponder))]
        fn accepts_first_responder(&self) -> bool {
            self.ivars().surface.borrow().strong_count() > 0
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(rightMouseDown:))]
        fn right_mouse_down(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(rightMouseDragged:))]
        fn right_mouse_dragged(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(rightMouseUp:))]
        fn right_mouse_up(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(otherMouseDown:))]
        fn other_mouse_down(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(otherMouseDragged:))]
        fn other_mouse_dragged(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(otherMouseUp:))]
        fn other_mouse_up(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            match self.surface() {
                Some(surface) => crate::input::event(&surface, event),
                // SAFETY: NSResponder's own handling (passes it up).
                None => unsafe { msg_send![super(self), scrollWheel: event] },
            }
        }

        #[unsafe(method(magnifyWithEvent:))]
        fn magnify(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(rotateWithEvent:))]
        fn rotate(&self, event: &NSEvent) {
            self.pointer(event);
        }

        #[unsafe(method(keyDown:))]
        fn key_down(&self, event: &NSEvent) {
            let handled = self
                .surface()
                .is_some_and(|surface| crate::input::key(&surface, event, true));
            if !handled {
                // SAFETY: NSResponder's own handling (beeps at the end).
                unsafe { msg_send![super(self), keyDown: event] }
            }
        }

        #[unsafe(method(keyUp:))]
        fn key_up(&self, event: &NSEvent) {
            let handled = self
                .surface()
                .is_some_and(|surface| crate::input::key(&surface, event, false));
            if !handled {
                // SAFETY: NSResponder's own handling.
                unsafe { msg_send![super(self), keyUp: event] }
            }
        }

        #[unsafe(method(setFrameSize:))]
        fn set_frame_size(&self, size: CGSize) {
            // SAFETY: NSView's own resize.
            unsafe { msg_send![super(self), setFrameSize: size] }
            if let Some(surface) = self.surface() {
                surface.resized(size.width as f32, size.height as f32);
            }
        }

        #[unsafe(method(viewDidMoveToWindow))]
        fn view_did_move_to_window(&self) {
            if let Some(surface) = self.surface() {
                self.install_tracking();
                crate::system::window_changed(&surface);
            }
        }

        #[unsafe(method(viewDidChangeEffectiveAppearance))]
        fn view_did_change_effective_appearance(&self) {
            if let Some(surface) = self.surface() {
                crate::system::appearance_changed(&surface);
            }
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn view_did_change_backing_properties(&self) {
            if let Some(surface) = self.surface() {
                crate::system::window_changed(&surface);
            }
        }
    }
);

impl ViewObj {
    fn surface(&self) -> Option<std::rc::Rc<Surface>> {
        self.ivars().surface.borrow().upgrade()
    }

    fn pointer(&self, event: &NSEvent) {
        if let Some(surface) = self.surface() {
            crate::input::event(&surface, event);
        }
    }

    /// Mouse moves, enters and exits over the root (hover, cursors).
    fn install_tracking(&self) {
        if self.ivars().tracking.replace(true) {
            return;
        }
        let options = NSTrackingAreaOptions::MouseMoved
            | NSTrackingAreaOptions::MouseEnteredAndExited
            | NSTrackingAreaOptions::ActiveAlways
            | NSTrackingAreaOptions::InVisibleRect;
        // SAFETY: the owner (this view) outlives the tracking area, which
        // it holds itself.
        let area = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                NSTrackingArea::alloc(),
                NSRect::ZERO,
                options,
                Some(self),
                None,
            )
        };
        self.addTrackingArea(&area);
    }
}

define_class!(
    /// Holds a view's children ([`NshView::container`]): flipped, the
    /// view's size, scrolled by its bounds origin.
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "NshContainer"]
    pub(crate) struct Container;

    unsafe impl NSObjectProtocol for Container {}

    impl Container {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(wantsUpdateLayer))]
        fn wants_update_layer(&self) -> bool {
            true
        }

        #[unsafe(method(updateLayer))]
        fn update_layer(&self) {}
    }
);

/// Whether `view` is an app's view (or a container of one), as opposed to
/// a hosted AppKit view.
pub(crate) fn is_ours(view: &NSView) -> bool {
    view.downcast_ref::<ViewObj>().is_some() || view.downcast_ref::<Container>().is_some()
}

/// `view`'s superview.
pub(crate) fn superview(view: &NSView) -> Option<Retained<NSView>> {
    // SAFETY: a plain getter, on the main thread (views are main-thread
    // only); AppKit keeps the superview alive while `view` is in it.
    unsafe { view.superview() }
}

/// Makes `view` layer-backed with the policies every view of ours uses.
fn setup(view: &NSView) {
    view.setWantsLayer(true);
    view.setLayerContentsRedrawPolicy(NSViewLayerContentsRedrawPolicy::OnSetNeedsDisplay);
    if let Some(layer) = view.layer() {
        layer.setAnchorPoint(CGPoint::new(0.0, 0.0));
    }
}

/// The view of one node: a cheap handle (cloning retains).
#[derive(Clone)]
pub(crate) struct NshView(Retained<ViewObj>);

impl std::fmt::Debug for NshView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.state();
        write!(f, "NshView({} {:?})", s.id, s.kind)
    }
}

impl PartialEq for NshView {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(&*self.0, &*other.0)
    }
}

impl Eq for NshView {}

impl std::ops::Deref for NshView {
    type Target = ViewObj;

    fn deref(&self) -> &ViewObj {
        &self.0
    }
}

impl NshView {
    /// A new, detached view of node `id`.
    pub(crate) fn new(mtm: MainThreadMarker, id: NodeId, kind: ViewKind) -> Self {
        let obj = mtm.alloc::<ViewObj>().set_ivars(Ivars {
            state: RefCell::new(ViewState::new(id, kind)),
            container: RefCell::new(None),
            surface: RefCell::new(Weak::new()),
            tracking: Cell::new(false),
        });
        // SAFETY: NSView's designated initializer.
        let obj: Retained<ViewObj> = unsafe { msg_send![super(obj), initWithFrame: NSRect::ZERO] };
        setup(&obj);
        let view = NshView(obj);
        if kind == ViewKind::Scroll {
            view.ensure_container();
        }
        view
    }

    /// Makes this the root view of `surface`: it takes the surface's mouse
    /// and keyboard events and reports its size.
    pub(crate) fn set_surface(&self, surface: Weak<Surface>) {
        *self.ivars().surface.borrow_mut() = surface;
    }

    /// From an AppKit view of ours, if it is one.
    pub(crate) fn from_ns(view: &NSView) -> Option<NshView> {
        view.downcast_ref::<ViewObj>().map(|v| NshView(v.retain()))
    }

    /// The AppKit view.
    pub(crate) fn ns(&self) -> &NSView {
        &self.0
    }

    /// Its state.
    pub(crate) fn state(&self) -> Ref<'_, ViewState> {
        self.ivars().state.borrow()
    }

    /// Its state, to change. Call [`NshView::changed`] or
    /// [`NshView::moved`] afterwards.
    pub(crate) fn state_mut(&self) -> RefMut<'_, ViewState> {
        self.ivars().state.borrow_mut()
    }

    /// Node id.
    pub(crate) fn id(&self) -> NodeId {
        self.state().id
    }

    /// Kind.
    pub(crate) fn kind(&self) -> ViewKind {
        self.state().kind
    }

    /// The view that holds its children: its container, or itself.
    pub(crate) fn container(&self) -> Retained<NSView> {
        match &*self.ivars().container.borrow() {
            Some(c) => Retained::from(&**c),
            None => Retained::from(self.ns()),
        }
    }

    /// Its container, if it has one.
    pub(crate) fn container_view(&self) -> Option<Retained<NSView>> {
        self.ivars()
            .container
            .borrow()
            .as_ref()
            .map(|c| Retained::from(&**c))
    }

    /// Gives it a container (moving its children into it), if it has none.
    pub(crate) fn ensure_container(&self) -> Retained<NSView> {
        if let Some(c) = self.container_view() {
            return c;
        }
        let mtm = self.mtm();
        // SAFETY: NSView's designated initializer.
        let c: Retained<Container> =
            unsafe { msg_send![Container::alloc(mtm), initWithFrame: self.ns().bounds()] };
        setup(&c);
        let children = self.child_views();
        let first_native = self.ns().subviews().iter().find(|v| !is_ours(v));
        match first_native {
            Some(native) => self.ns().addSubview_positioned_relativeTo(
                &c,
                NSWindowOrderingMode::Below,
                Some(&native),
            ),
            None => self.ns().addSubview(&c),
        }
        for child in children {
            c.addSubview(child.ns());
        }
        *self.ivars().container.borrow_mut() = Some(c.clone());
        self.size_container();
        Retained::from(&*c as &NSView)
    }

    /// Sizes the container to the view's bounds.
    fn size_container(&self) {
        if let Some(c) = &*self.ivars().container.borrow() {
            let f = self.state().frame;
            let origin = c.bounds().origin;
            c.setFrame(ns_rect(0.0, 0.0, f.width, f.height));
            c.setBoundsOrigin(origin);
        }
    }

    /// Sets the origin of its container's bounds: the scroll offset.
    pub(crate) fn set_bounds_origin(&self, x: f32, y: f32) {
        let c = self.ensure_container();
        c.setBoundsOrigin(NSPoint::new(x as CGFloat, y as CGFloat));
    }

    /// Something it draws changed: content (text, canvas) is drawn again
    /// and the decoration applied again.
    pub(crate) fn changed(&self) {
        crate::decor::apply(self);
        if self.state().draws() {
            self.ns().setNeedsDisplay(true);
        }
    }

    /// Its frame, transform or shift changed: places it again.
    pub(crate) fn moved(&self) {
        self.place();
        crate::decor::apply(self);
    }

    /// Places the view at its frame (with its parent's mirroring and its
    /// shift) and sets its layer's transform, opacity and visibility.
    pub(crate) fn place(&self) {
        let mirror = self.parent_view().and_then(|p| {
            let s = p.state();
            s.scroll
                .as_ref()
                .is_some_and(|sc| sc.mirrored)
                .then_some(s.frame.width)
        });
        let (rect, transform, opacity, hidden, root) = {
            let s = self.state();
            let f = s.frame;
            let mut x = f.x + s.shift.0;
            if let Some(w) = mirror {
                x = w - x - f.width;
            }
            let y = f.y + s.shift.1;
            (
                ns_rect(x, y, f.width, f.height),
                s.transform(),
                s.opacity(),
                s.style.hidden || crate::decor::backface_away(&s),
                s.id == ROOT_ID && self.ivars().surface.borrow().strong_count() > 0,
            )
        };
        if !root && self.ns().frame() != rect {
            self.ns().setFrame(rect);
            self.size_container();
            self.place_natives();
        }
        if let Some(layer) = self.ns().layer() {
            // AppKit sets a layer-backed view's anchor, position and
            // transform when it moves the view: set them again after.
            layer.setAnchorPoint(CGPoint::new(0.0, 0.0));
            layer.setPosition(rect.origin);
            let t = transform.as_ref().map_or(IDENTITY, ca_transform);
            layer.setTransform(t);
            layer.setOpacity(opacity.clamp(0.0, 1.0));
            layer.setHidden(hidden);
        }
        if self.state().draws() {
            self.ns().setNeedsDisplay(true);
        }
    }

    /// Hosted native views at their rectangles.
    fn place_natives(&self) {
        let natives = self.state().natives.clone();
        for n in natives {
            let r = n.rect;
            n.view.setFrame(ns_rect(r.x, r.y, r.width, r.height));
        }
    }

    /// The view's parent view, if it is attached to one.
    pub(crate) fn parent_view(&self) -> Option<NshView> {
        let mut sup = superview(self.ns())?;
        if sup.downcast_ref::<Container>().is_some() {
            sup = superview(&sup)?;
        }
        NshView::from_ns(&sup)
    }

    /// Whether it is in a parent view.
    pub(crate) fn has_parent(&self) -> bool {
        superview(self.ns()).is_some()
    }

    /// Its child views, bottom to top.
    pub(crate) fn child_views(&self) -> Vec<NshView> {
        self.container()
            .subviews()
            .iter()
            .filter_map(|v| NshView::from_ns(&v))
            .collect()
    }

    /// Inserts it into `parent` below `before` (on top of the other
    /// children without one; hosted native views stay above).
    pub(crate) fn insert_before(&self, parent: &NshView, before: Option<&NshView>) {
        let container = parent.container();
        let above = before
            .map(|b| Retained::from(b.ns()))
            .or_else(|| container.subviews().iter().find(|v| !is_ours(v)));
        match above {
            Some(above) => container.addSubview_positioned_relativeTo(
                self.ns(),
                NSWindowOrderingMode::Below,
                Some(&above),
            ),
            None => container.addSubview(self.ns()),
        }
        self.place();
    }

    /// Takes it out of its parent.
    pub(crate) fn unparent(&self) {
        self.ns().removeFromSuperview();
    }

    /// Hosts `view` at `rect` (or moves it there).
    pub(crate) fn set_native(&self, view: &NSView, rect: Rect) {
        {
            let mut s = self.state_mut();
            match s.natives.iter_mut().find(|n| &*n.view == view) {
                Some(n) => n.rect = rect,
                None => s.natives.push(Native {
                    view: Retained::from(view),
                    rect,
                }),
            }
        }
        if superview(view).as_deref() != Some(self.ns()) {
            self.ns().addSubview(view);
        }
        view.setFrame(ns_rect(rect.x, rect.y, rect.width, rect.height));
    }

    /// Stops hosting `view`.
    pub(crate) fn remove_native(&self, view: &NSView) {
        self.state_mut().natives.retain(|n| &*n.view != view);
        if superview(view).as_deref() == Some(self.ns()) {
            view.removeFromSuperview();
        }
    }

    /// Whether the pointer may target it (`pointerEvents` and visibility).
    pub(crate) fn can_target(&self) -> bool {
        let s = self.state();
        !s.style.hidden && !matches!(s.props.pointer_events, PointerEvents::None)
    }

    /// An object for `msg_send!` calls AppKit's bindings lack.
    #[allow(dead_code)]
    pub(crate) fn as_object(&self) -> &AnyObject {
        let view: &NSView = self.ns();
        view.as_ref()
    }
}
