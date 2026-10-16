//! Drawing a surface's view tree with Direct2D.
//!
//! [`frame`] draws the whole tree of a surface into its window's swap
//! chain and presents it; [`snapshot`] draws it into an offscreen bitmap
//! and reads the pixels back (`--script` screenshots). Both go through
//! [`draw_view`], which draws one view and its subtree in this order:
//!
//! 1. shadows and background ([`crate::decor::draw_background`]),
//! 2. the content, clipped to the bounds: text, image, canvas,
//! 3. the children (clipped to the rounded bounds when the view clips),
//! 4. the hosted items ([`crate::view::Native`]) and the scroll indicator,
//! 5. the border ([`crate::decor::draw_border`]),
//!
//! all inside an opacity layer when the view is translucent. A view whose
//! transform is projective (perspective, `rotateX` with a perspective) is
//! drawn into an offscreen bitmap first and mapped with
//! `DrawBitmap`'s perspective transform; every other transform is a 3x2
//! world transform.
//!
//! Coordinates are points (Direct2D DIPs: the context's DPI is the
//! window's scale times 96).

use std::rc::Rc;

use nsh_core::{Color, Rect};
use windows::Win32::{
    Foundation::{D2DERR_RECREATE_TARGET, RECT},
    Graphics::{
        Direct2D::{
            Common::{
                D2D_POINT_2U, D2D_RECT_F, D2D_RECT_U, D2D_SIZE_F, D2D_SIZE_U,
                D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_FIGURE_BEGIN_FILLED,
                D2D1_FIGURE_END_CLOSED, D2D1_PIXEL_FORMAT,
            },
            D2D1_ANTIALIAS_MODE_ALIASED, D2D1_ANTIALIAS_MODE_PER_PRIMITIVE, D2D1_ARC_SEGMENT,
            D2D1_ARC_SIZE_SMALL, D2D1_BITMAP_OPTIONS_CANNOT_DRAW, D2D1_BITMAP_OPTIONS_CPU_READ,
            D2D1_BITMAP_OPTIONS_TARGET, D2D1_BITMAP_PROPERTIES1, D2D1_INTERPOLATION_MODE_LINEAR,
            D2D1_LAYER_OPTIONS1_NONE, D2D1_LAYER_PARAMETERS1, D2D1_MAP_OPTIONS_READ,
            D2D1_ROUNDED_RECT, D2D1_SWEEP_DIRECTION_CLOCKWISE,
            ID2D1Bitmap1, ID2D1DeviceContext, ID2D1Geometry, ID2D1SolidColorBrush,
        },
        Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
    },
};
use windows_numerics::Matrix3x2;

use crate::{
    geom::Mat4,
    gfx::{self, Gfx},
    surface::Surface,
    view::{Hosted, NshView},
};

/// A color as Direct2D's (straight alpha; Direct2D premultiplies).
pub(crate) fn color_f(c: Color) -> D2D1_COLOR_F {
    let (r, g, b, a) = c.to_f64();
    D2D1_COLOR_F {
        r: r as f32,
        g: g as f32,
        b: b as f32,
        a: a as f32,
    }
}

/// A rectangle as Direct2D's.
pub(crate) fn rect_f(x: f32, y: f32, w: f32, h: f32) -> D2D_RECT_F {
    D2D_RECT_F {
        left: x,
        top: y,
        right: x + w,
        bottom: y + h,
    }
}

/// An unbounded rectangle (layer content bounds).
pub(crate) const INFINITE: D2D_RECT_F = D2D_RECT_F {
    left: -1.0e7,
    top: -1.0e7,
    right: 1.0e7,
    bottom: 1.0e7,
};

/// What was pushed on the context, to pop it the same way.
enum Pushed {
    Layer,
    Clip,
}

/// The state of one drawing pass.
pub(crate) struct DrawCx<'a> {
    /// The devices.
    pub(crate) gfx: &'a Gfx,
    /// The context drawn with (the same as `gfx.dc`).
    pub(crate) dc: &'a ID2D1DeviceContext,
    /// The world transform of the view being drawn (its coordinates to
    /// the target's DIPs).
    pub(crate) transform: Mat4,
    /// Pixels per point.
    pub(crate) scale: f32,
    /// The surface draws in its dark appearance.
    pub(crate) dark: bool,
    /// The frame's time (seconds, the animation clock).
    pub(crate) now: f64,
    brush: ID2D1SolidColorBrush,
    pushed: Vec<Pushed>,
    /// Hosted items drawn in this pass, with their window rectangles.
    pub(crate) drawn: Vec<(Rc<dyn Hosted>, RECT)>,
    /// True when drawing offscreen (a snapshot): child windows are not
    /// moved.
    pub(crate) offscreen: bool,
}

impl<'a> DrawCx<'a> {
    /// A pass with `dc` (its target set, between `BeginDraw` and
    /// `EndDraw`).
    pub(crate) fn new(gfx: &'a Gfx, scale: f32, dark: bool, now: f64) -> windows::core::Result<Self> {
        let black = color_f(Color::BLACK);
        // SAFETY: a COM call on the live context.
        let brush = unsafe { gfx.dc.CreateSolidColorBrush(&black, None)? };
        Ok(DrawCx {
            gfx,
            dc: &gfx.dc,
            transform: Mat4::new(),
            scale,
            dark,
            now,
            brush,
            pushed: Vec::new(),
            drawn: Vec::new(),
            offscreen: false,
        })
    }

    /// The pass's solid brush, set to `color`.
    pub(crate) fn solid(&self, color: Color) -> &ID2D1SolidColorBrush {
        // SAFETY: a COM call on a live brush.
        unsafe { self.brush.SetColor(&color_f(color)) };
        &self.brush
    }

    /// Sets the world transform (`m` must be affine: see
    /// [`Mat4::is_affine`]).
    pub(crate) fn set_transform(&mut self, m: Mat4) {
        self.transform = m;
        let t = m.to_3x2();
        // SAFETY: a COM call on the live context.
        unsafe { self.dc.SetTransform(&t) };
    }

    /// Draws what follows at `opacity`.
    pub(crate) fn push_opacity(&mut self, opacity: f32) {
        let params = D2D1_LAYER_PARAMETERS1 {
            contentBounds: INFINITE,
            geometricMask: std::mem::ManuallyDrop::new(None),
            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            maskTransform: Matrix3x2::identity(),
            opacity,
            opacityBrush: std::mem::ManuallyDrop::new(None),
            layerOptions: D2D1_LAYER_OPTIONS1_NONE,
        };
        // SAFETY: valid parameters; popped by `pop`.
        unsafe { self.dc.PushLayer(&params, None) };
        self.pushed.push(Pushed::Layer);
    }

    /// Clips what follows to `geometry` (in the current coordinates).
    pub(crate) fn push_clip_geometry(&mut self, geometry: &ID2D1Geometry) {
        let params = D2D1_LAYER_PARAMETERS1 {
            contentBounds: INFINITE,
            geometricMask: std::mem::ManuallyDrop::new(Some(geometry.clone())),
            maskAntialiasMode: D2D1_ANTIALIAS_MODE_PER_PRIMITIVE,
            maskTransform: Matrix3x2::identity(),
            opacity: 1.0,
            opacityBrush: std::mem::ManuallyDrop::new(None),
            layerOptions: D2D1_LAYER_OPTIONS1_NONE,
        };
        // SAFETY: valid parameters; popped by `pop`.
        unsafe { self.dc.PushLayer(&params, None) };
        // The parameters own a reference to the mask: release it.
        let mut params = params;
        // SAFETY: the field is not used afterwards.
        unsafe { std::mem::ManuallyDrop::drop(&mut params.geometricMask) };
        self.pushed.push(Pushed::Layer);
    }

    /// Clips what follows to the rectangle (current coordinates). Exact
    /// when the transform keeps rectangles axis-aligned; otherwise a
    /// geometric layer.
    pub(crate) fn push_clip_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let t = self.transform.to_3x2();
        if t.M12 == 0.0 && t.M21 == 0.0 {
            // Snapped to pixels when the edges fall on them, as UIKit's
            // `masksToBounds` of a pixel-aligned layer.
            // SAFETY: popped by `pop`.
            unsafe {
                self.dc
                    .PushAxisAlignedClip(&rect_f(x, y, w, h), D2D1_ANTIALIAS_MODE_ALIASED)
            };
            self.pushed.push(Pushed::Clip);
        } else if let Some(g) = self.rounded_geometry(crate::geom::rect(x, y, w, h), [0.0; 4]) {
            self.push_clip_geometry(&g);
        } else {
            self.push_opacity(1.0);
        }
    }

    /// Clips what follows to the rounded rectangle `rect` with `radii`
    /// (clockwise from the top left).
    pub(crate) fn push_clip_rounded(&mut self, rect: Rect, radii: [f32; 4]) {
        if radii.iter().all(|r| *r <= 0.0) {
            return self.push_clip_rect(rect.x, rect.y, rect.width, rect.height);
        }
        match self.rounded_geometry(rect, radii) {
            Some(g) => self.push_clip_geometry(&g),
            None => self.push_clip_rect(rect.x, rect.y, rect.width, rect.height),
        }
    }

    /// Pops the last push.
    pub(crate) fn pop(&mut self) {
        match self.pushed.pop() {
            // SAFETY: matches the push.
            Some(Pushed::Layer) => unsafe { self.dc.PopLayer() },
            // SAFETY: as above.
            Some(Pushed::Clip) => unsafe { self.dc.PopAxisAlignedClip() },
            None => log::error!("render: pop without push"),
        }
    }

    /// The rounded rectangle `rect` with corner radii clockwise from the
    /// top left, as a geometry.
    pub(crate) fn rounded_geometry(&self, rect: Rect, radii: [f32; 4]) -> Option<ID2D1Geometry> {
        rounded_geometry(self.gfx, rect, radii)
    }
}

/// The rounded rectangle `rect` with corner radii clockwise from the top
/// left (each clamped to half the box), as a geometry.
pub(crate) fn rounded_geometry(gfx: &Gfx, rect: Rect, radii: [f32; 4]) -> Option<ID2D1Geometry> {
    use windows::core::Interface;
    let Rect {
        x,
        y,
        width: w,
        height: h,
    } = rect;
    let max = (w.min(h) / 2.0).max(0.0);
    let r = radii.map(|r| r.clamp(0.0, max));
    if r[0] == r[1] && r[1] == r[2] && r[2] == r[3] {
        let rr = D2D1_ROUNDED_RECT {
            rect: rect_f(x, y, w, h),
            radiusX: r[0],
            radiusY: r[0],
        };
        // SAFETY: COM calls on the live factory.
        return unsafe { gfx.factory.CreateRoundedRectangleGeometry(&rr) }
            .ok()
            .and_then(|g| g.cast().ok());
    }
    // SAFETY: COM calls on live objects; the sink is closed before use.
    unsafe {
        let path = gfx.factory.CreatePathGeometry().ok()?;
        let sink = path.Open().ok()?;
        let p = |px: f32, py: f32| windows_numerics::Vector2 { X: px, Y: py };
        let arc = |to: windows_numerics::Vector2, radius: f32| D2D1_ARC_SEGMENT {
            point: to,
            size: D2D_SIZE_F {
                width: radius,
                height: radius,
            },
            rotationAngle: 0.0,
            sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
            arcSize: D2D1_ARC_SIZE_SMALL,
        };
        sink.BeginFigure(p(x + r[0], y), D2D1_FIGURE_BEGIN_FILLED);
        sink.AddLine(p(x + w - r[1], y));
        if r[1] > 0.0 {
            sink.AddArc(&arc(p(x + w, y + r[1]), r[1]));
        }
        sink.AddLine(p(x + w, y + h - r[2]));
        if r[2] > 0.0 {
            sink.AddArc(&arc(p(x + w - r[2], y + h), r[2]));
        }
        sink.AddLine(p(x + r[3], y + h));
        if r[3] > 0.0 {
            sink.AddArc(&arc(p(x, y + h - r[3]), r[3]));
        }
        sink.AddLine(p(x, y + r[0]));
        if r[0] > 0.0 {
            sink.AddArc(&arc(p(x + r[0], y), r[0]));
        }
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        sink.Close().ok()?;
        path.cast().ok()
    }
}

/// Draws `view` and its subtree; `parent` is the world transform of the
/// parent's coordinates and `origin` where the view goes in them.
pub(crate) fn draw_view(cx: &mut DrawCx<'_>, view: &NshView, parent: Mat4, origin: (f32, f32)) {
    let state = view.state();
    if state.style.hidden || state.concealed || crate::decor::backface_away(&state, &parent) {
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
    if !world.is_affine() {
        drop(state);
        return draw_projected(cx, view, world, opacity);
    }
    cx.set_transform(world);
    if opacity < 1.0 {
        cx.push_opacity(opacity);
    }
    let filters = crate::decor::push_filters(cx, &state);
    crate::decor::draw_background(cx, &state);
    let clip = state.clips();
    let (w, h) = (state.frame.width, state.frame.height);
    if clip {
        cx.push_clip_rounded(crate::geom::rect(0.0, 0.0, w, h), state.radii());
    } else {
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
    drop(state);
    view.with_children(|children| {
        for child in children {
            let origin = {
                let s = child.state();
                view.state().child_origin(s.frame, s.shift)
            };
            draw_view(cx, child, world, origin);
        }
    });
    cx.set_transform(world);
    for native in &natives {
        let r = native.rect;
        let m = Mat4::new().translate(r.x, r.y).then(&world);
        cx.set_transform(m);
        native.item.draw(cx, (r.width, r.height));
        if let Some((x, y, bw, bh)) = m.bounds(0.0, 0.0, r.width, r.height) {
            let s = cx.scale;
            let rect = RECT {
                left: (x * s).round() as i32,
                top: (y * s).round() as i32,
                right: ((x + bw) * s).round() as i32,
                bottom: ((y + bh) * s).round() as i32,
            };
            cx.drawn.push((native.item.clone(), rect));
        }
    }
    cx.set_transform(world);
    let state = view.state();
    crate::scroll::draw_indicator(cx, &state);
    if clip {
        cx.pop();
    }
    crate::decor::draw_border(cx, &state);
    for _ in 0..filters {
        cx.pop();
    }
    if opacity < 1.0 {
        cx.pop();
    }
}

/// Draws `view` (with a projective `world` transform) into an offscreen
/// bitmap, then maps the bitmap with Direct2D's perspective transform.
fn draw_projected(cx: &mut DrawCx<'_>, view: &NshView, world: Mat4, opacity: f32) {
    let (w, h, overflow) = {
        let s = view.state();
        (s.frame.width, s.frame.height, s.overflow)
    };
    // What the subtree covers, in the view's coordinates.
    let area = overflow.unwrap_or(crate::geom::rect(0.0, 0.0, w, h));
    let scale = cx.scale;
    let px = (
        (area.width * scale).ceil().clamp(1.0, 8192.0) as u32,
        (area.height * scale).ceil().clamp(1.0, 8192.0) as u32,
    );
    let props = gfx::target_properties(scale, true);
    // SAFETY: COM calls on the live context; the previous target is
    // restored before returning.
    let result = unsafe {
        (|| -> windows::core::Result<()> {
            let bitmap = cx.dc.CreateBitmap(
                D2D_SIZE_U {
                    width: px.0,
                    height: px.1,
                },
                None,
                0,
                &D2D1_BITMAP_PROPERTIES1 {
                    bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
                    ..props
                },
            )?;
            let previous = cx.dc.GetTarget()?;
            let saved = (cx.transform, std::mem::take(&mut cx.pushed));
            cx.dc.SetTarget(&bitmap);
            cx.dc.Clear(Some(&color_f(Color::TRANSPARENT)));
            // The subtree as if untransformed, at the bitmap's origin.
            let base = Mat4::new().translate(-area.x, -area.y);
            {
                let s = view.state();
                let mut flat = view.clone();
                drop(s);
                draw_flat(cx, &mut flat, base);
            }
            cx.dc.SetTarget(&previous);
            cx.transform = saved.0;
            cx.pushed = saved.1;
            cx.dc.SetTransform(&Matrix3x2::identity());
            // The bitmap's DIPs map to the view's coordinates (offset by
            // the area's origin), then through the view's world transform.
            let m = Mat4::new().translate(area.x, area.y).then(&world);
            let dest = rect_f(0.0, 0.0, area.width, area.height);
            cx.dc.DrawBitmap(
                &bitmap,
                Some(&dest),
                opacity,
                D2D1_INTERPOLATION_MODE_LINEAR,
                None,
                Some(&m.to_d2d()),
            );
            Ok(())
        })()
    };
    if let Err(err) = result {
        log::debug!("render: projected view not drawn: {err}");
    }
}

/// Draws `view` with its transform replaced by `base` (the inside of
/// [`draw_projected`]), at full opacity.
fn draw_flat(cx: &mut DrawCx<'_>, view: &mut NshView, base: Mat4) {
    let (saved_style_transform, saved_opacity, saved_anim) = {
        let mut s = view.state_mut();
        let t = std::mem::take(&mut s.style.transform);
        let o = std::mem::replace(&mut s.style.opacity, 1.0);
        let a = std::mem::take(&mut s.anim);
        (t, o, a)
    };
    let saved_live = view.state_mut().live.take();
    let saved_fade = std::mem::replace(&mut view.state_mut().fade, 1.0);
    draw_view(cx, view, base, (0.0, 0.0));
    let mut s = view.state_mut();
    s.style.transform = saved_style_transform;
    s.style.opacity = saved_opacity;
    s.anim = saved_anim;
    s.live = saved_live;
    s.fade = saved_fade;
}

/// Draws the surface's window: the whole tree, then presents it.
pub(crate) fn frame(surface: &Rc<Surface>) {
    let Ok(gfx) = gfx::get() else { return };
    let hwnd = surface.hwnd();
    if hwnd.is_invalid() {
        return;
    }
    let scale = surface.scale();
    let size = surface.pixel_size();
    if size.0 == 0 || size.1 == 0 {
        return;
    }
    let mut swap = surface.swap.borrow_mut();
    if swap.as_ref().is_some_and(|s| s.stale()) {
        *swap = None;
    }
    if swap.is_none() {
        match gfx::SwapChain::new(&gfx, hwnd, size) {
            Ok(s) => *swap = Some(s),
            Err(err) => {
                log::error!("cannot create the swap chain: {err}");
                return;
            }
        }
    }
    let Some(chain) = swap.as_mut() else { return };
    if let Err(err) = chain.resize(&gfx, size) {
        log::error!("cannot resize the swap chain: {err}");
        *swap = None;
        return;
    }
    if (surface.target_scale.get() - scale).abs() > f32::EPSILON {
        chain.drop_target();
        surface.target_scale.set(scale);
    }
    let target = match chain.target(&gfx, scale) {
        Ok(t) => t,
        Err(err) => {
            log::error!("cannot get the back buffer: {err}");
            return;
        }
    };
    let drawn = draw_into(surface, &gfx, &target, scale, false);
    match drawn {
        Ok(drawn) => {
            if let Err(err) = chain.present() {
                log::warn!("present failed: {err}");
                drop(swap);
                device_lost(surface);
                return;
            }
            drop(swap);
            surface.items_placed(drawn);
        }
        Err(err) if err.code() == D2DERR_RECREATE_TARGET => {
            drop(swap);
            device_lost(surface);
        }
        Err(err) => log::error!("drawing failed: {err}"),
    }
}

fn device_lost(surface: &Rc<Surface>) {
    *surface.swap.borrow_mut() = None;
    gfx::lost();
    surface.invalidate();
}

/// Draws the tree into `target` (its DPI set for `scale`); returns the
/// hosted items drawn and their window rectangles.
fn draw_into(
    surface: &Rc<Surface>,
    gfx: &Gfx,
    target: &ID2D1Bitmap1,
    scale: f32,
    offscreen: bool,
) -> windows::core::Result<Vec<(Rc<dyn Hosted>, RECT)>> {
    let dc = &gfx.dc;
    // SAFETY: COM calls on the live context and target; `EndDraw` always
    // follows `BeginDraw`.
    unsafe {
        dc.SetTarget(target);
        dc.SetDpi(96.0 * scale, 96.0 * scale);
        dc.BeginDraw();
        dc.SetTransform(&Matrix3x2::identity());
        dc.Clear(Some(&color_f(crate::system::window_background(surface))));
    }
    let mut cx = match DrawCx::new(gfx, scale, crate::system::dark(surface), surface.now()) {
        Ok(cx) => cx,
        Err(err) => {
            // SAFETY: closes the `BeginDraw` above.
            let _ = unsafe { dc.EndDraw(None, None) };
            return Err(err);
        }
    };
    cx.offscreen = offscreen;
    let root = surface.root.clone();
    draw_view(&mut cx, &root, Mat4::new(), (0.0, 0.0));
    let drawn = std::mem::take(&mut cx.drawn);
    drop(cx);
    // SAFETY: closes the `BeginDraw` above; the target is unset so that
    // the swap chain can be resized later.
    let result = unsafe { dc.EndDraw(None, None) };
    // SAFETY: as above.
    unsafe { dc.SetTarget(None) };
    result.map(|()| drawn)
}

/// Pixels of a snapshot: BGRA, premultiplied, `width * 4` bytes per row.
pub(crate) struct Pixels {
    /// Width in pixels.
    pub(crate) width: u32,
    /// Height in pixels.
    pub(crate) height: u32,
    /// The rows.
    pub(crate) data: Vec<u8>,
}

/// Draws the surface offscreen at `scale` and reads the pixels back,
/// whatever covers the window.
pub(crate) fn snapshot(surface: &Rc<Surface>, scale: f32) -> windows::core::Result<Pixels> {
    let gfx = gfx::get()?;
    let (w, h) = surface.size();
    let px = (
        (w * scale).round().max(1.0) as u32,
        (h * scale).round().max(1.0) as u32,
    );
    let size = D2D_SIZE_U {
        width: px.0,
        height: px.1,
    };
    let format = D2D1_PIXEL_FORMAT {
        format: DXGI_FORMAT_B8G8R8A8_UNORM,
        alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
    };
    // SAFETY: COM calls on the live context; the mapped memory is copied
    // before `Unmap`.
    unsafe {
        let target = gfx.dc.CreateBitmap(
            size,
            None,
            0,
            &D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: format,
                dpiX: 96.0 * scale,
                dpiY: 96.0 * scale,
                bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
                colorContext: std::mem::ManuallyDrop::new(None),
            },
        )?;
        let drawn = draw_into(surface, &gfx, &target, scale, true)?;
        drop(drawn);
        let read = gfx.dc.CreateBitmap(
            size,
            None,
            0,
            &D2D1_BITMAP_PROPERTIES1 {
                pixelFormat: format,
                dpiX: 96.0 * scale,
                dpiY: 96.0 * scale,
                bitmapOptions: D2D1_BITMAP_OPTIONS_CPU_READ | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
                colorContext: std::mem::ManuallyDrop::new(None),
            },
        )?;
        read.CopyFromBitmap(
            Some(&D2D_POINT_2U { x: 0, y: 0 }),
            &target,
            Some(&D2D_RECT_U {
                left: 0,
                top: 0,
                right: px.0,
                bottom: px.1,
            }),
        )?;
        let mapped = read.Map(D2D1_MAP_OPTIONS_READ)?;
        let mut data = Vec::with_capacity((px.0 * px.1 * 4) as usize);
        for row in 0..px.1 {
            let start = mapped.bits.add((row * mapped.pitch) as usize);
            data.extend_from_slice(std::slice::from_raw_parts(start, (px.0 * 4) as usize));
        }
        read.Unmap()?;
        Ok(Pixels {
            width: px.0,
            height: px.1,
            data,
        })
    }
}
