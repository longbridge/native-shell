//! The Canvas demo on a headless engine: its sections render, its buttons
//! draw again, its drawings use gradients, SVG paths and measured labels,
//! and the sketch claims its touches.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    Command, EventKind, EventPayload, MountBatch, MountItem, TouchPhase,
    canvas::{CanvasOp, GradientShape, Num, Paint, PathOp},
};
use nsh_engine::Engine;

/// Presses the parent of the text `label` and returns the texts the
/// canvases drew in what it changed.
fn press_drawing(
    engine: &Engine,
    rx: &Receiver<MountBatch>,
    screen: &mut crate::common::Screen,
    label: &str,
) -> Vec<String> {
    let text = screen
        .find(label)
        .unwrap_or_else(|| panic!("no {label}: {:?}", screen.texts));
    let target = screen.press_target(text);
    engine
        .control()
        .dispatch_event(target, EventKind::Press, EventPayload::None);
    let mut drawn = Vec::new();
    while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
        screen.apply(&batch);
        for item in &batch.items {
            if let MountItem::Canvas { drawing, .. } = item {
                drawn.extend(drawing.ops.iter().filter_map(|op| match op {
                    CanvasOp::FillText { text, .. } => Some(text.to_string()),
                    _ => None,
                }));
            }
        }
    }
    drawn
}

#[test]
fn canvas_renders_its_sections() {
    let (_engine, _rx, screen) = crate::common::open("Canvas");
    for text in [
        "NATIVE ANIMATION (PROGRESS RING)",
        "CHART",
        "TRANSFORMS (A CLOCK)",
        "PATH2D (SVG PATH DATA)",
        "heart",
        "check",
        "IMAGES, COMPOSITING, SHADOWS",
        "SKETCH (TOUCH → DRAW)",
        "Random",
        "New data",
        "Clear",
    ] {
        assert!(
            screen.texts.iter().any(|t| t.1 == text),
            "{text}: {:?}",
            screen.texts
        );
    }
}

#[test]
fn full_draws_the_ring_again() {
    let (engine, rx, mut screen) = crate::common::open("Canvas");
    let drawn = press_drawing(&engine, &rx, &mut screen, "Full");
    assert!(drawn.iter().any(|t| t == "100%"), "{drawn:?}");
}

/// The drawing whose operations `pick` finds, among the canvases.
fn drawing_with(
    screen: &crate::common::Screen,
    pick: impl Fn(&CanvasOp) -> bool,
) -> Option<(u32, &nsh_core::canvas::Drawing)> {
    screen
        .drawings
        .iter()
        .find(|(_, d)| d.ops.iter().any(&pick))
        .map(|(id, d)| (*id, &**d))
}

#[test]
fn canvases_draw_gradients_paths_and_measured_labels() {
    let (_engine, _rx, screen) = crate::common::open("Canvas");
    let gradient = |kind: fn(&GradientShape) -> bool| {
        move |op: &CanvasOp| match op {
            CanvasOp::FillStyle(Paint::Gradient(g)) | CanvasOp::StrokeStyle(Paint::Gradient(g)) => {
                kind(&g.shape)
            }
            _ => false,
        }
    };
    // The ring strokes a conic gradient, the orange ball fills a radial
    // one, the chart's area a linear one.
    let conic_stroke = |op: &CanvasOp| matches!(op, CanvasOp::StrokeStyle(Paint::Gradient(g)) if matches!(g.shape, GradientShape::Conic(_)));
    assert!(drawing_with(&screen, conic_stroke).is_some(), "the ring");
    assert!(drawing_with(&screen, gradient(|s| matches!(s, GradientShape::Radial(_)))).is_some());
    let (_, chart) = drawing_with(&screen, gradient(|s| matches!(s, GradientShape::Linear(_))))
        .expect("the chart");
    // Its labels end 6 pt left of the plot: each placed by its width.
    let ends: Vec<(String, f32)> = chart
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::FillText {
                text,
                x: Num::Const(x),
                ..
            } => Some((text.to_string(), *x)),
            _ => None,
        })
        .collect();
    assert_eq!(ends.len(), 5, "{ends:?}");
    assert!(
        ends[0].1 > ends[4].1,
        "\"0\" starts right of \"100\": {ends:?}"
    );
    // The check icon strokes its SVG path.
    assert!(
        drawing_with(&screen, |op| matches!(op, CanvasOp::Stroke(Some(_)))).is_some(),
        "a stroked path"
    );
}

#[test]
fn the_sketch_claims_its_touches_and_draws_strokes() {
    let (engine, rx, mut screen) = crate::common::open("Canvas");
    let pad = *screen
        .touch_listeners()
        .first()
        .expect("the sketch listens to touches");
    for (phase, at, t) in [
        (TouchPhase::Start, (10.0, 10.0), 0.0),
        (TouchPhase::Move, (40.0, 30.0), 16.0),
    ] {
        crate::common::touch(&engine, &rx, &mut screen, phase, &[pad], at, t);
    }
    assert_eq!(
        screen.commands.last(),
        Some(&(pad, Command::SetJsResponder { block: true })),
        "the page does not scroll while drawing"
    );
    let points: Vec<(f32, f32)> = screen.drawings[&pad]
        .ops
        .iter()
        .filter_map(|op| match op {
            CanvasOp::Path(PathOp::MoveTo(Num::Const(x), Num::Const(y)))
            | CanvasOp::Path(PathOp::LineTo(Num::Const(x), Num::Const(y))) => Some((*x, *y)),
            _ => None,
        })
        .collect();
    assert_eq!(points, [(10.0, 10.0), (40.0, 30.0)]);
    crate::common::touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::End,
        &[pad],
        (40.0, 30.0),
        32.0,
    );
    assert_eq!(
        screen.commands.last(),
        Some(&(nsh_core::ROOT_ID, Command::ClearJsResponder))
    );
}
