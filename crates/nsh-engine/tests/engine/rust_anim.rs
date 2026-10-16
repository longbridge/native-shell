//! A Rust component's composed animations on a headless engine, with the
//! platform's part played by an animation graph here: a composite goes out
//! as one operation and runs step after step in the graph (no engine round
//! trips between steps); its end, its stop and its reset come back to the
//! component's task.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    ApproxTextEngine, EventKind, EventPayload, MountBatch, MountItem, NodeId, Size,
    nsh_anim::{AnimOp, Finished, Graph},
    ui::{Root, prelude::*},
};
use nsh_engine::{App, Engine, EngineConfig, Environment};

use crate::ChannelSink;

/// A value, a jump (a delayed timing, a wait, a timing back) and a pulse
/// looping until stopped.
#[derive(Default)]
struct Jumper {
    x: Option<NativeValue>,
    jump: Option<Animation>,
    pulse: Option<Animation>,
    state: String,
}

impl Jumper {
    /// Runs `animation` (`jump` or `pulse`), reporting how it ended.
    fn run(&mut self, name: &'static str, cx: &mut Context<'_, Self>) {
        let animation = if name == "jump" {
            &self.jump
        } else {
            &self.pulse
        };
        let end = animation.as_ref().expect("set up").start();
        cx.spawn(move |this, _| async move {
            let finished = end.await;
            this.update(move |this, cx| {
                let how = if finished { "finished" } else { "stopped" };
                this.state = format!("{name}: {how}");
                cx.notify();
            })
            .await;
        })
        .detach();
        self.state = format!("{name}…");
        cx.notify();
    }

    fn each(&self, f: impl Fn(&Animation)) {
        self.jump.iter().chain(&self.pulse).for_each(f);
    }
}

impl Component for Jumper {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        let x = cx.native_value(0.0);
        let linear = |ms| Timing::new(ms, Easing::Linear);
        self.jump = Some(Animation::sequence([
            Animation::timing(
                &x,
                100.0,
                Timing {
                    delay_ms: 50.0,
                    ..linear(200.0)
                },
            ),
            Animation::delay(100.0),
            Animation::timing(&x, 0.0, linear(200.0)),
        ]));
        self.pulse = Some(
            Animation::sequence([
                Animation::timing(&x, 50.0, linear(100.0)),
                Animation::timing(&x, 0.0, linear(100.0)),
            ])
            .looped(None),
        );
        self.x = Some(x);
        self.state = "idle".into();
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .bind_translate_x(self.x.as_ref().expect("set up"))
            .child(text(self.state.clone()))
            .child(
                text("Jump").on_press(cx.listener(|this: &mut Self, _, cx| this.run("jump", cx))),
            )
            .child(
                text("Pulse").on_press(cx.listener(|this: &mut Self, _, cx| this.run("pulse", cx))),
            )
            .child(
                text("Stop")
                    .on_press(cx.listener(|this: &mut Self, _, _| this.each(Animation::stop))),
            )
            .child(
                text("Reset")
                    .on_press(cx.listener(|this: &mut Self, _, _| this.each(Animation::reset))),
            )
    }
}

/// The engine, and the platform's part: what it shows and its graph.
struct Platform {
    engine: Engine,
    rx: Receiver<MountBatch>,
    texts: Vec<(NodeId, String)>,
    graph: Graph,
    finished: Vec<Finished>,
    /// The graph's clock, in seconds.
    now: f64,
    /// The graph operations received.
    ops: Vec<AnimOp>,
    /// The value moved (bound to the root view's translation).
    x: Option<u32>,
}

impl Platform {
    fn start() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let config = EngineConfig {
            app: App::Rust(Root::of::<Jumper>()),
            viewport: Size::new(390.0, 844.0),
            trace: false,
            network: None,
            assets: None,
            environment: Environment::default(),
            header_heights: (44.0, 96.0),
            limits: nsh_engine::Limits::default(),
            ui_sets: Vec::new(),
            runtime: None,
        };
        let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
        let mut platform = Self {
            engine,
            rx,
            texts: Vec::new(),
            graph: Graph::default(),
            finished: Vec::new(),
            now: 0.0,
            ops: Vec::new(),
            x: None,
        };
        platform.pump();
        platform
    }

    /// Applies the batches until the engine is quiet, reporting the ends of
    /// animations.
    fn pump(&mut self) {
        loop {
            self.report();
            let Ok(batch) = self.rx.recv_timeout(Duration::from_millis(200)) else {
                break;
            };
            for item in batch.items {
                match item {
                    MountItem::Text { id, text, .. } => {
                        self.texts.retain(|t| t.0 != id);
                        self.texts.push((id, text.text.clone()));
                    }
                    MountItem::AnimBindings { bindings, .. } => {
                        self.x = bindings.first().map(|b| b.1);
                    }
                    MountItem::Anim(op) => {
                        self.ops.push(op.clone());
                        self.graph.apply(op, &mut self.finished);
                    }
                    _ => {}
                }
            }
        }
    }

    fn report(&mut self) {
        if !self.finished.is_empty() {
            let finished = std::mem::take(&mut self.finished);
            self.engine.control().animations_finished(finished);
        }
    }

    /// Runs the graph's frames (60 Hz) for `secs`, then hands the engine
    /// what ended.
    fn advance(&mut self, secs: f64) {
        let end = self.now + secs;
        while self.now < end - 1e-9 {
            self.graph.tick(self.now, &mut self.finished);
            self.now += 1.0 / 60.0;
        }
        self.graph.tick(end, &mut self.finished);
        self.now = end;
        self.pump();
    }

    fn press(&mut self, label: &str) {
        let id = self
            .texts
            .iter()
            .find(|t| t.1 == label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", self.texts))
            .0;
        self.engine
            .control()
            .dispatch_event(id, EventKind::Press, EventPayload::None);
        self.pump();
        // The graph starts what came in at its next frame.
        self.graph.tick(self.now, &mut self.finished);
    }

    fn shows(&self, text: &str) -> bool {
        self.texts.iter().any(|t| t.1 == text)
    }

    fn x(&self) -> f32 {
        self.graph.value(self.x.expect("bound")).unwrap()
    }
}

#[test]
fn rust_composites_run_natively_and_report_their_ends() {
    let mut p = Platform::start();
    assert!(p.shows("idle"), "{:?}", p.texts);
    assert!(p.x.is_some(), "the value is bound");

    p.ops.clear();
    p.press("Jump");
    assert!(p.shows("jump…"), "{:?}", p.texts);
    // Its first step waits for its delay.
    p.advance(0.04);
    assert_eq!(p.x(), 0.0);
    p.advance(0.11);
    assert!((p.x() - 50.0).abs() < 1.0, "halfway: {}", p.x());
    p.advance(0.45);
    assert_eq!(p.x(), 0.0, "back");
    assert!(p.shows("jump: finished"), "{:?}", p.texts);
    // One operation started it; its steps ran in the graph.
    let starts = p
        .ops
        .iter()
        .filter(|op| matches!(op, AnimOp::StartComposite { .. } | AnimOp::Start { .. }))
        .count();
    assert_eq!(starts, 1, "{:?}", p.ops);

    // Reset: stopped, and back where the run started.
    p.press("Jump");
    p.advance(0.15);
    assert!(p.x() > 10.0, "{}", p.x());
    p.press("Reset");
    assert!(p.shows("jump: stopped"), "{:?}", p.texts);
    assert_eq!(p.x(), 0.0);

    // Stop: stopped where it is.
    p.press("Pulse");
    assert!(p.shows("pulse…"), "{:?}", p.texts);
    p.advance(0.5);
    assert!(p.shows("pulse…"), "it loops: {:?}", p.texts);
    let at = p.x();
    assert!(at > 0.0, "{at}");
    p.press("Stop");
    assert!(p.shows("pulse: stopped"), "{:?}", p.texts);
    p.advance(0.2);
    assert_eq!(p.x(), at, "it stays");
}
