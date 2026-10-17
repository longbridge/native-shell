//! Drawing a surface's view tree with GPUI.
//!
//! [`SurfaceElement`] is the GPUI element of one surface: it fills the
//! space its parent gives it, reports that size to the engine, takes the
//! pointer and keys over it ([`crate::events`]) and paints the whole tree
//! in its `paint` through [`draw_view`], which draws one view and its
//! subtree in this order:
//!
//! 1. shadows and background ([`crate::decor::draw_background`]),
//! 2. the content, clipped to the bounds: text, image, canvas,
//! 3. the children (clipped to the bounds when the view clips),
//! 4. the hosted items ([`crate::view::Native`]) and the scroll indicator,
//! 5. the border ([`crate::decor::draw_border`]).
//!
//! GPUI paints primitives (quads, paths, glyphs, images) in window
//! coordinates (logical pixels, which are nsh's points), with no world
//! transform and no group opacity. [`DrawCx`] keeps both itself: the
//! current transform maps the view's coordinates to the window's
//! (translations and scales are exact; rotations and skews draw at their
//! bounds, paths and glyph runs aside), and opacity multiplies into every
//! color, as GPUI's own element opacity does. Clips are rectangles in
//! window coordinates (GPUI's content masks), intersected as they nest.

use std::rc::Rc;

use gpui::{
    App, Bounds, ContentMask, Corners, Edges, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, PaintQuad, Pixels, Point,
    Window, point, px, size,
};
use nsh_core::{Color, Rect};

use crate::{
    geom::Mat4,
    surface::Surface,
    view::{Hosted, NshView},
};

/// A color as GPUI's.
pub(crate) fn hsla(c: Color) -> Hsla {
    let (r, g, b, a) = c.to_f64();
    gpui::Rgba {
        r: r as f32,
        g: g as f32,
        b: b as f32,
        a: a as f32,
    }
    .into()
}

/// A rectangle in window coordinates.
pub(crate) fn bounds(x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
    Bounds::new(point(px(x), px(y)), size(px(w), px(h)))
}

/// Saved state of [`DrawCx::push_clip_rect`] / [`DrawCx::push_opacity`].
#[derive(Clone, Copy)]
struct Saved {
    clip: Bounds<Pixels>,
    opacity: f32,
}

/// The state of one drawing pass.
pub(crate) struct DrawCx<'a> {
    /// The window painted into (in its paint phase).
    pub(crate) window: &'a mut Window,
    /// The app.
    pub(crate) app: &'a mut App,
    /// The transform of the view being drawn (its coordinates to the
    /// window's, points).
    pub(crate) transform: Mat4,
    /// Pixels per point.
    pub(crate) scale: f32,
    /// The surface draws in its dark appearance.
    pub(crate) dark: bool,
    /// The frame's time (seconds, the animation clock).
    pub(crate) now: f64,
    /// The opacity everything is drawn with.
    pub(crate) opacity: f32,
    /// The clip in window coordinates.
    pub(crate) clip: Bounds<Pixels>,
    saved: Vec<Saved>,
    /// Hosted items drawn in this pass, with their window bounds.
    pub(crate) drawn: Vec<(Rc<dyn Hosted>, Bounds<Pixels>)>,
}

impl<'a> DrawCx<'a> {
    /// A pass over `clip` (window coordinates).
    pub(crate) fn new(
        window: &'a mut Window,
        app: &'a mut App,
        clip: Bounds<Pixels>,
        dark: bool,
        now: f64,
    ) -> Self {
        let scale = window.scale_factor();
        DrawCx {
            window,
            app,
            transform: Mat4::new(),
            scale,
            dark,
            now,
            opacity: 1.0,
            clip,
            saved: Vec::new(),
            drawn: Vec::new(),
        }
    }

    /// Sets the transform (view coordinates to window coordinates).
    pub(crate) fn set_transform(&mut self, m: Mat4) {
        self.transform = m;
    }

    /// True when the transform keeps rectangles axis-aligned.
    pub(crate) fn axis_aligned(&self) -> bool {
        let [_, b, c, _, _, _] = self.transform.affine();
        b.abs() < 1e-6 && c.abs() < 1e-6
    }

    /// The scale of the transform (the larger of its axes'): sizes that
    /// do not follow rectangles (radii, font sizes, line widths) scale by
    /// it.
    pub(crate) fn transform_scale(&self) -> f32 {
        let [a, b, c, d, _, _] = self.transform.affine();
        (a * a + b * b).sqrt().max((c * c + d * d).sqrt())
    }

    /// Where `(x, y)` of the current coordinates lands in the window.
    pub(crate) fn map(&self, x: f32, y: f32) -> Point<Pixels> {
        let [a, b, c, d, tx, ty] = self.transform.affine();
        point(px(a * x + c * y + tx), px(b * x + d * y + ty))
    }

    /// The window bounds of the rectangle (current coordinates): exact
    /// when [`DrawCx::axis_aligned`], else its bounding box.
    pub(crate) fn rect(&self, x: f32, y: f32, w: f32, h: f32) -> Bounds<Pixels> {
        match self.transform.bounds(x, y, w, h) {
            Some((bx, by, bw, bh)) => bounds(bx, by, bw, bh),
            None => bounds(0.0, 0.0, 0.0, 0.0),
        }
    }

    /// `color` with the pass's opacity.
    pub(crate) fn color(&self, color: Color) -> Hsla {
        let mut c = hsla(color);
        c.a *= self.opacity;
        c
    }

    /// Draws what follows at `opacity` (multiplied into the colors).
    pub(crate) fn push_opacity(&mut self, opacity: f32) {
        self.save();
        self.opacity *= opacity;
    }

    /// Clips what follows to the rectangle (current coordinates; its
    /// bounds when the transform rotates).
    pub(crate) fn push_clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.save();
        let r = self.rect(x, y, w, h);
        self.clip = self.clip.intersect(&r);
    }

    /// Clips what follows to the rounded rectangle `rect` (GPUI's masks
    /// are rectangles: the corners are not clipped).
    pub(crate) fn push_clip_rounded(&mut self, rect: Rect, _radii: [f32; 4]) {
        self.push_clip_rect(rect.x, rect.y, rect.width, rect.height);
    }

    fn save(&mut self) {
        self.saved.push(Saved {
            clip: self.clip,
            opacity: self.opacity,
        });
    }

    /// Pops the last push.
    pub(crate) fn pop(&mut self) {
        match self.saved.pop() {
            Some(s) => {
                self.clip = s.clip;
                self.opacity = s.opacity;
            }
            None => log::error!("render: pop without push"),
        }
    }

    /// True when nothing drawn now can show (clipped away, transparent).
    pub(crate) fn culled(&self) -> bool {
        self.opacity <= 0.0 || self.clip.size.width <= px(0.0) || self.clip.size.height <= px(0.0)
    }

    /// Runs `f` with the window inside the current clip.
    pub(crate) fn paint<R>(&mut self, f: impl FnOnce(&mut Window, &mut App) -> R) -> R {
        let mask = ContentMask { bounds: self.clip };
        let app = &mut *self.app;
        self.window
            .with_content_mask(Some(mask), |window| f(window, app))
    }

    /// Fills the rounded rectangle (current coordinates) with `color`.
    pub(crate) fn fill_rounded(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radius: f32,
        color: Color,
    ) {
        self.fill_rect(crate::geom::rect(x, y, w, h), [radius; 4], color);
    }

    /// Fills `rect` (current coordinates) with corner radii (clockwise
    /// from the top left) in `color`. Rotated rectangles are filled as
    /// paths (without their corners).
    pub(crate) fn fill_rect(&mut self, rect: Rect, radii: [f32; 4], color: Color) {
        if color.0 & 0xff == 0 || self.culled() {
            return;
        }
        let c = self.color(color);
        if self.axis_aligned() {
            let b = self.rect(rect.x, rect.y, rect.width, rect.height);
            let k = self.transform_scale();
            let quad = gpui::fill(b, c).corner_radii(corners(radii, k));
            self.paint(|window, _| window.paint_quad(quad));
        } else {
            self.fill_polygon(
                &[
                    (rect.x, rect.y),
                    (rect.x + rect.width, rect.y),
                    (rect.x + rect.width, rect.y + rect.height),
                    (rect.x, rect.y + rect.height),
                ],
                color,
            );
        }
    }

    /// Fills the polygon (current coordinates) with `color`.
    pub(crate) fn fill_polygon(&mut self, points: &[(f32, f32)], color: Color) {
        if points.len() < 3 || color.0 & 0xff == 0 || self.culled() {
            return;
        }
        let mut path = gpui::Path::new(self.map(points[0].0, points[0].1));
        for p in &points[1..] {
            path.line_to(self.map(p.0, p.1));
        }
        let c = self.color(color);
        self.paint(|window, _| window.paint_path(path, c));
    }

    /// Paints `quad` (window coordinates) inside the clip.
    pub(crate) fn paint_quad(&mut self, quad: PaintQuad) {
        if self.culled() {
            return;
        }
        self.paint(|window, _| window.paint_quad(quad));
    }
}

/// Corner radii (clockwise from the top left) as GPUI's, scaled by `k`.
pub(crate) fn corners(radii: [f32; 4], k: f32) -> Corners<Pixels> {
    Corners {
        top_left: px(radii[0] * k),
        top_right: px(radii[1] * k),
        bottom_right: px(radii[2] * k),
        bottom_left: px(radii[3] * k),
    }
}

/// Edge widths (top, right, bottom, left) as GPUI's, scaled by `k`.
pub(crate) fn edges(widths: [f32; 4], k: f32) -> Edges<Pixels> {
    Edges {
        top: px(widths[0] * k),
        right: px(widths[1] * k),
        bottom: px(widths[2] * k),
        left: px(widths[3] * k),
    }
}

/// Draws `view` and its subtree; `parent` is the transform of the
/// parent's coordinates and `origin` where the view goes in them.
pub(crate) fn draw_view(cx: &mut DrawCx<'_>, view: &NshView, parent: Mat4, origin: (f32, f32)) {
    let state = view.state();
    if state.style.hidden || state.concealed {
        return;
    }
    let opacity = state.opacity();
    if opacity <= 0.0 {
        return;
    }
    let mut local = Mat4::new().translate(origin.0, origin.1);
    if let Some(t) = state.transform() {
        local = Mat4::mul(&t, &local);
    }
    let world = Mat4::mul(&local, &parent);
    if crate::decor::backface_away(&state, &world) {
        return;
    }
    drop(state);
    cx.set_transform(world);
    cx.push_opacity(opacity);
    draw_content(cx, view, world);
    cx.pop();
}

/// Draws what `view` draws and its subtree at `world` (set on the
/// context): shadows and background, the content clipped to the bounds,
/// the children, the hosted items, the scroll indicator, the border and
/// outline.
pub(crate) fn draw_content(cx: &mut DrawCx<'_>, view: &NshView, world: Mat4) {
    let state = view.state();
    crate::decor::draw_background(cx, &state);
    let clip = state.clips();
    let (w, h) = (state.frame.width, state.frame.height);
    if clip {
        cx.push_clip_rounded(crate::geom::rect(0.0, 0.0, w, h), state.radii());
        crate::text::draw_shadow(cx, &state);
    } else {
        // Text shadows spread past the view, as CSS's do.
        crate::text::draw_shadow(cx, &state);
        // The view's own content stays inside it, as a UIKit view's drawing
        // stays in its backing store.
        cx.push_clip_rect(0.0, 0.0, w, h);
    }
    crate::text::draw(cx, &state);
    crate::image::draw(cx, &state);
    crate::canvas::draw(cx, &state);
    if !clip {
        cx.pop();
    }
    let natives = state.natives.clone();
    let culled = clip && cx.culled();
    drop(state);
    if !culled {
        view.with_children(|children| {
            for child in children {
                let origin = {
                    let s = child.state();
                    view.state().child_origin(s.frame, s.shift)
                };
                draw_view(cx, child, world, origin);
            }
        });
    }
    cx.set_transform(world);
    for native in &natives {
        let r = native.rect;
        let m = Mat4::new().translate(r.x, r.y).then(&world);
        cx.set_transform(m);
        native.item.draw(cx, (r.width, r.height));
        let b = cx.rect(0.0, 0.0, r.width, r.height);
        cx.drawn.push((native.item.clone(), b));
    }
    cx.set_transform(world);
    let state = view.state();
    crate::scroll::draw_indicator(cx, &state);
    if clip {
        cx.pop();
    }
    crate::decor::draw_border(cx, &state);
}

/// The GPUI element of a surface: fills its parent's space and draws the
/// surface's view tree.
pub(crate) struct SurfaceElement {
    pub(crate) surface: Rc<Surface>,
    pub(crate) focus: gpui::FocusHandle,
    pub(crate) view: gpui::WeakEntity<crate::app::AppView>,
}

impl IntoElement for SurfaceElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SurfaceElement {
    type RequestLayoutState = ();
    type PrepaintState = Hitbox;

    fn id(&self) -> Option<ElementId> {
        // An id gives the element a node in GPUI's accessibility tree.
        Some("nsh-surface".into())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = gpui::Style::default();
        style.size.width = gpui::relative(1.0).into();
        style.size.height = gpui::relative(1.0).into();
        style.flex_grow = 1.0;
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let surface = &self.surface;
        surface.drawing();
        surface
            .origin
            .set((f32::from(bounds.origin.x), f32::from(bounds.origin.y)));
        surface.resized(
            f32::from(bounds.size.width),
            f32::from(bounds.size.height),
            window.scale_factor(),
        );
        crate::system::window_prepaint(surface, window, cx);
        if surface.wants_frames() {
            surface.tick();
            window.request_animation_frame();
        }
        // Changes made by the tick are in this frame.
        surface.drawing();
        crate::access::prepaint(surface, &self.focus, window, cx);
        window.set_focus_handle(&self.focus, cx);
        window.insert_hitbox(bounds, HitboxBehavior::BlockMouse)
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _layout: &mut Self::RequestLayoutState,
        hitbox: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let surface = self.surface.clone();
        let dark = crate::system::dark(&surface);
        // Embedded with a theme, the surface is drawn on the host's
        // background.
        let background = match &*surface.host_theme.borrow() {
            Some(theme) if dark => theme.background.dark,
            Some(theme) => theme.background.light,
            None => crate::system::window_background(&surface),
        };
        if background.0 & 0xff != 0 {
            window.paint_quad(gpui::fill(bounds, hsla(background)));
        }
        let now = surface.now();
        let drawn = {
            let mut dcx = DrawCx::new(window, cx, bounds, dark, now);
            let origin = (f32::from(bounds.origin.x), f32::from(bounds.origin.y));
            draw_view(&mut dcx, &surface.root, Mat4::new(), origin);
            std::mem::take(&mut dcx.drawn)
        };
        surface.items_placed(drawn);
        crate::events::install(
            &surface,
            &self.focus,
            &self.view,
            hitbox,
            bounds,
            window,
            cx,
        );
        crate::access::paint(&surface, window, cx);
    }

    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        Some(crate::access::ROLE)
    }

    fn a11y_synthetic_children(
        &mut self,
        _hitbox: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        crate::access::synthetic(&self.surface, builder);
    }
}
