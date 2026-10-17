//! The Animations demo on a headless engine, with the platform's animation
//! graph played here: its sections render, the compositions run natively
//! and report finishing, stopping and resetting (back to where the run
//! started), the interpolation's chips and the layout's button follow
//! their state.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    EventKind, EventPayload, MountBatch, MountItem,
    nsh_animation::{AnimationValue, Finished, Graph, NodeId},
};
use nsh_engine::Engine;

use crate::common::{Screen, shows};

/// The gallery, what it shows and the platform's graph.
struct Platform {
    engine: Engine,
    rx: Receiver<MountBatch>,
    screen: Screen,
    graph: Graph,
    finished: Vec<Finished>,
    /// The graph's clock, in seconds.
    now: f64,
    /// The nodes bound to views' properties.
    bound: Vec<NodeId>,
}

impl Platform {
    /// The gallery with the Animations demo open, every graph operation
    /// applied.
    fn open() -> Self {
        let (engine, rx) = crate::common::start();
        let mut p = Self {
            engine,
            rx,
            screen: Screen::default(),
            graph: Graph::default(),
            finished: Vec::new(),
            now: 0.0,
            bound: Vec::new(),
        };
        p.pump();
        let row = p
            .screen
            .press_target(p.screen.find("Animations").expect("its row"));
        p.dispatch(row);
        assert!(
            p.screen.titles.iter().any(|t| t.1 == "Animations"),
            "{:?}",
            p.screen.titles
        );
        p
    }

    /// Applies batches until the engine is quiet, handing it the ends of
    /// animations.
    fn pump(&mut self) {
        loop {
            if !self.finished.is_empty() {
                let finished = std::mem::take(&mut self.finished);
                self.engine.control().animations_finished(finished);
            }
            let Ok(batch) = self.rx.recv_timeout(Duration::from_millis(300)) else {
                break;
            };
            self.screen.apply(&batch);
            for item in batch.items {
                match item {
                    MountItem::Animation(op) => self.graph.apply(op, &mut self.finished),
                    MountItem::AnimationBindings { bindings, .. } => {
                        for (_, node) in bindings {
                            if !self.bound.contains(&node) {
                                self.bound.push(node);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn dispatch(&mut self, id: u32) {
        self.engine
            .control()
            .dispatch_event(id, EventKind::Press, EventPayload::None);
        self.pump();
        // What came in starts at the graph's next frame.
        self.graph.tick(self.now, &mut self.finished);
        self.pump();
    }

    /// Presses the text `label` (a chip).
    fn press_text(&mut self, label: &str) {
        let id = self
            .screen
            .find(label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", self.screen.texts));
        self.dispatch(id);
    }

    /// Presses the parent of the text `label` (a button).
    fn press(&mut self, label: &str) {
        let text = self
            .screen
            .find(label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", self.screen.texts));
        self.dispatch(self.screen.press_target(text));
    }

    /// Runs the graph's frames (60 Hz) for `secs`.
    fn advance(&mut self, secs: f64) {
        let end = self.now + secs;
        while self.now < end {
            self.now += 1.0 / 60.0;
            self.graph.tick(self.now, &mut self.finished);
        }
        self.pump();
    }

    /// What the bound properties are now.
    fn bound_values(&self) -> Vec<AnimationValue> {
        self.bound.iter().map(|&n| self.graph.eval(n)).collect()
    }

    fn shows(&self, text: &str) -> bool {
        shows(&self.screen, text)
    }
}

#[test]
fn animations_render_and_react() {
    let mut p = Platform::open();
    for text in [
        "EASING CURVES",
        "SEQUENCE, PARALLEL, STAGGER, LOOP",
        "SPRINGS",
        "INTERPOLATION",
        "DIFFCLAMP: A HEADER THAT HIDES ON SCROLL",
        "LAYOUT ANIMATION",
        "bounce out",
        "wobbly",
        "idle",
        "Scroll up to hide me",
        "Row 30",
        "t = 0 ●",
    ] {
        assert!(p.shows(text), "{text}: {:?}", p.screen.texts);
    }

    // A composition runs to its end natively.
    p.press_text("Stagger");
    assert!(p.shows("stagger…"), "{:?}", p.screen.texts);
    p.advance(3.0);
    assert!(p.shows("stagger: finished"), "{:?}", p.screen.texts);

    let rest = p.bound_values();

    // A loop goes on until stopped, where it is; a reset puts it back.
    p.press_text("Loop");
    p.advance(1.0);
    assert!(p.shows("loop…"), "{:?}", p.screen.texts);
    p.press_text("Stop");
    assert!(p.shows("loop: stopped"), "{:?}", p.screen.texts);
    assert_ne!(p.bound_values(), rest, "partway");
    p.press_text("Reset");
    assert_eq!(p.bound_values(), rest);

    // A reset stops a run going on and puts its values back where the run
    // started.
    p.press_text("Sequence");
    assert!(p.shows("sequence…"), "{:?}", p.screen.texts);
    // Turned, waited, fading.
    p.advance(0.9);
    assert_ne!(p.bound_values(), rest, "partway");
    p.press_text("Reset");
    assert!(p.shows("sequence: stopped"), "{:?}", p.screen.texts);
    assert_eq!(p.bound_values(), rest);

    p.press_text("t = 2");
    assert!(
        p.shows("t = 2 ●") && p.shows("t = 0"),
        "{:?}",
        p.screen.texts
    );

    p.press("Column");
    assert!(p.shows("Row"), "{:?}", p.screen.texts);
    p.press("Row");
    assert!(p.shows("Column"), "{:?}", p.screen.texts);
}
