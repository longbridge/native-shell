//! The performance monitor's HUD (`--fps`, [`crate::AppOptions::fps`],
//! [`crate::AppView::set_fps_monitor`]): a panel at the top right of an
//! [`AppView`] that shows what its surface's frames and commits cost
//! ([`nsh_engine::perf`]), redrawn twice a second.
//!
//! - **FPS**: frames shown over the last second (display refreshes in which
//!   the surface changed: an idle app reads low, which is not slowness);
//!   `MAX` is what the main thread could keep up with (one over the mean
//!   frame cost, capped at the display's rate), then the display link's
//!   rate.
//! - **MAIN**: main-thread time per shown frame (applying batches, Core
//!   Animation's commit and the drawing it does, animations, input), its
//!   95th percentile, and the share of frames over one refresh (`drop`).
//! - **JS** / **CORE**: per commit, the app's code and the core's work
//!   (apply, render, text, layout, diff); commits per second and mount
//!   items per commit.
//! - **HOP**: how long batches waited between the engine thread and the
//!   main thread; the engine thread's busy share.
//! - **CPU** (100 is one core), **GPU** (this process' share, from the IO
//!   registry, [`crate::gpu`]), **MEM** (the physical footprint).
//! - The trace: the last 120 frames' main-thread cost, against the frame
//!   budget (the line), green within half of it, amber within it, red over.
//!
//! The panel lets the mouse through. It measures nothing while hidden: the
//! surface's monitor is removed with it.

use std::{
    cell::RefCell,
    rc::{Rc, Weak},
    sync::Arc,
    time::Duration,
};

use nsh_engine::perf::{PerfMonitor, PerfSnapshot, ProcessSampler, Stat};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, rc::Retained};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSColor, NSFont, NSFontWeightRegular, NSGraphicsContext,
    NSResponder, NSTextField, NSView,
};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGContext;
use objc2_foundation::{NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use rustc_hash::FxHashMap;

use crate::{app::AppView, surface::Surface};

/// How often the readings are redrawn.
const INTERVAL_MS: u64 = 500;
/// The panel's width and its distance from the view's edges (points).
const WIDTH: CGFloat = 262.0;
const MARGIN: CGFloat = 10.0;
const PADDING: CGFloat = 8.0;
const GRAPH_HEIGHT: CGFloat = 36.0;

thread_local! {
    /// The HUDs of this (main) thread by key: the timer finds its HUD by
    /// key, as dispatched closures must be `Send`.
    static HUDS: RefCell<FxHashMap<u64, Weak<Hud>>> = RefCell::default();
}

static NEXT_KEY: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

define_class!(
    /// The panel: lets the pointer through to the app.
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "NshFpsPanel"]
    struct Panel;

    unsafe impl NSObjectProtocol for Panel {}

    impl Panel {
        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
            None
        }
    }
);

/// What the trace draws: frame costs (ms) and the budget (ms).
#[derive(Default)]
struct Trace {
    frames: Vec<f32>,
    budget: f32,
}

define_class!(
    /// The frame-time trace.
    #[unsafe(super(NSView, NSResponder, NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Trace>]
    #[name = "NshFpsTrace"]
    struct TraceView;

    unsafe impl NSObjectProtocol for TraceView {}

    impl TraceView {
        #[unsafe(method_id(hitTest:))]
        fn hit_test(&self, _point: NSPoint) -> Option<Retained<NSView>> {
            None
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let Some(context) = NSGraphicsContext::currentContext() else {
                return;
            };
            draw_trace(&context.CGContext(), self.bounds(), &self.ivars().borrow());
        }
    }
);

/// Bars of the frames' cost, newest at the right; the scale is twice the
/// budget, its line at half height.
fn draw_trace(c: &CGContext, bounds: NSRect, trace: &Trace) {
    let (w, h) = (bounds.size.width, bounds.size.height);
    let c = Some(c);
    CGContext::set_rgb_fill_color(c, 1.0, 1.0, 1.0, 0.08);
    CGContext::fill_rect(c, CGRect::new(CGPoint::ZERO, CGSize::new(w, h)));
    let budget = trace.budget.max(0.1) as CGFloat;
    let scale = h / (2.0 * budget);
    let slots = 120.0;
    let bar = w / slots;
    let start = slots as usize - trace.frames.len().min(slots as usize);
    for (i, &ms) in trace
        .frames
        .iter()
        .rev()
        .take(slots as usize)
        .rev()
        .enumerate()
    {
        let ms = ms as CGFloat;
        let (r, g, b) = if ms <= budget / 2.0 {
            (0.30, 0.85, 0.45)
        } else if ms <= budget {
            (0.98, 0.75, 0.25)
        } else {
            (0.98, 0.35, 0.33)
        };
        CGContext::set_rgb_fill_color(c, r, g, b, 0.9);
        let height = (ms * scale).clamp(1.0, h);
        let x = (start + i) as CGFloat * bar;
        CGContext::fill_rect(
            c,
            CGRect::new(
                CGPoint::new(x, 0.0),
                CGSize::new((bar - 0.5).max(0.5), height),
            ),
        );
    }
    CGContext::set_rgb_fill_color(c, 1.0, 1.0, 1.0, 0.45);
    CGContext::fill_rect(
        c,
        CGRect::new(CGPoint::new(0.0, (h / 2.0).round()), CGSize::new(w, 1.0)),
    );
}

/// The HUD of one app view.
pub(crate) struct Hud {
    key: u64,
    surface: Weak<Surface>,
    monitor: Arc<PerfMonitor>,
    panel: Retained<NSView>,
    label: Retained<NSTextField>,
    trace: Retained<TraceView>,
    process: RefCell<ProcessSampler>,
    gpu: RefCell<Option<crate::gpu::GpuProbe>>,
    /// Also log the readings once a second (`--fps --trace`), and how many
    /// updates ran.
    log: bool,
    updates: std::cell::Cell<u64>,
}

impl Hud {
    /// Shows a HUD over `view` and starts measuring its surface.
    pub(crate) fn show(view: &AppView, log: bool) -> Rc<Hud> {
        let mtm = MainThreadMarker::new().expect("main thread");
        let surface = view.surface();
        let monitor = Arc::new(PerfMonitor::new());
        // SAFETY: NSView's designated initializer.
        let panel: Retained<Panel> =
            unsafe { msg_send![Panel::alloc(mtm), initWithFrame: NSRect::ZERO] };
        let panel: Retained<NSView> = Retained::into_super(panel);
        panel.setWantsLayer(true);
        if let Some(layer) = panel.layer() {
            let fill = NSColor::colorWithSRGBRed_green_blue_alpha(0.07, 0.07, 0.08, 0.86);
            layer.setBackgroundColor(Some(&fill.CGColor()));
            layer.setCornerRadius(8.0);
        }
        let label = NSTextField::labelWithString(&NSString::new(), mtm);
        // SAFETY: AppKit's font weight constant.
        let weight = unsafe { NSFontWeightRegular };
        label.setFont(Some(&NSFont::monospacedSystemFontOfSize_weight(
            10.5, weight,
        )));
        label.setTextColor(Some(&NSColor::colorWithSRGBRed_green_blue_alpha(
            0.92, 0.93, 0.95, 1.0,
        )));
        let trace = mtm.alloc::<TraceView>().set_ivars(RefCell::default());
        // SAFETY: NSView's designated initializer.
        let trace: Retained<TraceView> =
            unsafe { msg_send![super(trace), initWithFrame: NSRect::ZERO] };
        panel.addSubview(&label);
        panel.addSubview(&trace);
        view.container().addSubview(&panel);
        panel.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewMinXMargin | NSAutoresizingMaskOptions::ViewMinYMargin,
        );
        let hud = Rc::new(Hud {
            key: NEXT_KEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            surface: Rc::downgrade(surface),
            monitor: monitor.clone(),
            panel,
            label,
            trace,
            process: RefCell::new(ProcessSampler::new()),
            gpu: RefCell::new(crate::gpu::GpuProbe::new()),
            log,
            updates: std::cell::Cell::new(0),
        });
        HUDS.with(|h| h.borrow_mut().insert(hud.key, Rc::downgrade(&hud)));
        surface.set_perf_monitor(Some(monitor));
        hud.update();
        schedule(hud.key);
        hud
    }

    /// The monitor it shows.
    pub(crate) fn monitor(&self) -> &Arc<PerfMonitor> {
        &self.monitor
    }

    /// Samples the process and redraws the readings.
    fn update(&self) {
        let mut process = self.process.borrow_mut().sample();
        process.gpu = self.gpu.borrow_mut().as_mut().and_then(|g| g.sample());
        self.monitor.set_process(process);
        let snapshot = self.monitor.snapshot();
        let n = self.updates.get() + 1;
        self.updates.set(n);
        if self.log && n.is_multiple_of(2) {
            log::info!(target: "nsh::perf", "{}", log_line(&snapshot));
        }
        self.label
            .setStringValue(&NSString::from_str(&readings(&snapshot)));
        let budget = 1000.0 / snapshot.refresh_rate.unwrap_or(60.0);
        *self.trace.ivars().borrow_mut() = Trace {
            frames: snapshot
                .frames
                .iter()
                .map(|d| d.as_secs_f32() * 1000.0)
                .collect(),
            budget,
        };
        self.layout();
        self.trace.setNeedsDisplay(true);
    }

    /// Sizes the panel to its text, at the top right of its superview (not
    /// flipped: y grows upwards).
    fn layout(&self) {
        let Some(parent) = crate::view::superview(&self.panel) else {
            return;
        };
        let fit = self.label.fittingSize();
        let height = PADDING * 3.0 + fit.height + GRAPH_HEIGHT;
        let bounds = parent.bounds();
        self.panel.setFrame(NSRect::new(
            NSPoint::new(
                bounds.size.width - WIDTH - MARGIN,
                bounds.size.height - height - MARGIN,
            ),
            NSSize::new(WIDTH, height),
        ));
        self.trace.setFrame(NSRect::new(
            NSPoint::new(PADDING, PADDING),
            NSSize::new(WIDTH - 2.0 * PADDING, GRAPH_HEIGHT),
        ));
        self.label.setFrame(NSRect::new(
            NSPoint::new(PADDING, PADDING * 2.0 + GRAPH_HEIGHT),
            NSSize::new(WIDTH - 2.0 * PADDING, fit.height),
        ));
    }
}

impl Drop for Hud {
    fn drop(&mut self) {
        self.panel.removeFromSuperview();
        if let Some(surface) = self.surface.upgrade() {
            surface.set_perf_monitor(None);
        }
        let key = self.key;
        let _ = HUDS.try_with(|h| h.borrow_mut().remove(&key));
    }
}

/// Redraws HUD `key` every [`INTERVAL_MS`] while it exists.
fn schedule(key: u64) {
    let when = dispatch2::DispatchTime::NOW.time((INTERVAL_MS * 1_000_000) as i64);
    let result = dispatch2::DispatchQueue::main().after(when, move || {
        let Some(hud) = HUDS.with(|h| h.borrow().get(&key).and_then(Weak::upgrade)) else {
            return;
        };
        hud.update();
        schedule(key);
    });
    if let Err(err) = result {
        log::error!("cannot schedule the performance monitor: {err:?}");
    }
}

fn ms(d: Duration) -> f32 {
    d.as_secs_f32() * 1000.0
}

fn stat(s: &Stat) -> String {
    format!("{:6.2}  p95 {:6.2}", ms(s.mean), ms(s.p95))
}

/// One line of the readings, for logs: milliseconds as `mean/p50/p95/max`.
fn log_line(s: &PerfSnapshot) -> String {
    let st = |s: &Stat| {
        format!(
            "{:.2}/{:.2}/{:.2}/{:.2}",
            ms(s.mean),
            ms(s.p50),
            ms(s.p95),
            ms(s.max)
        )
    };
    format!(
        "fps {:.0} ({:.0} Hz) tick {} frame-gap {} main {} drop {:.0}% js {} core {} {:.0} commits/s {:.0} items hop {} cpu {:.0}% gpu {:.0}%",
        s.fps,
        s.refresh_rate.unwrap_or(0.0),
        st(&s.tick_interval),
        st(&s.frame_interval),
        st(&s.frame),
        s.dropped * 100.0,
        st(&s.js),
        st(&s.layout),
        s.commits,
        s.items,
        st(&s.latency),
        s.process.cpu.unwrap_or(0.0),
        s.process.gpu.unwrap_or(0.0),
    )
}

/// The panel's text.
fn readings(s: &PerfSnapshot) -> String {
    let rate = s
        .refresh_rate
        .map_or_else(|| "-".to_owned(), |r| format!("{r:.0} Hz"));
    let percent = |v: Option<f32>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.0}%"));
    let memory = s
        .process
        .memory
        .map_or_else(|| "-".to_owned(), |m| format!("{:.0} MB", m as f64 / 1e6));
    format!(
        "{:4.0} FPS   max {:.0} · {rate}\n\
         TICK {}  gap {:.1}\n\
         MAIN {}  drop {:.0}%\n\
         JS   {}  {:.0}/s\n\
         CORE {}  {:.0} items\n\
         HOP  {}  eng {:.0}%\n\
         CPU {}  GPU {}  MEM {memory}",
        s.fps,
        s.max_fps,
        stat(&s.tick_interval),
        ms(s.frame_interval.p95),
        stat(&s.frame),
        s.dropped * 100.0,
        stat(&s.js),
        s.commits,
        stat(&s.layout),
        s.items,
        stat(&s.latency),
        s.engine_busy,
        percent(s.process.cpu),
        percent(s.process.gpu),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings_of_an_empty_snapshot() {
        let text = readings(&PerfSnapshot::default());
        assert!(text.starts_with("   0 FPS   max 0 · -"));
        assert!(text.contains("CPU -  GPU -  MEM -"));
    }
}
