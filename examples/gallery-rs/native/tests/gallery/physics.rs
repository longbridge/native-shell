//! The Physics demo on a headless engine: a release springs the card back
//! with the drag's velocity and is counted, a tap pulses it, a long press
//! resets the count.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    EventKind, EventPayload, MountBatch, MountItem,
    nsh_animation::{AnimationOp, Driver},
};
use nsh_engine::Engine;

use crate::common::shows;

/// Sends `kind` with `payload` to the parent of the text `label`; the
/// drivers the animations it started have.
fn dispatch(
    engine: &Engine,
    rx: &Receiver<MountBatch>,
    screen: &mut crate::common::Screen,
    label: &str,
    kind: EventKind,
    payload: EventPayload,
) -> Vec<Driver> {
    let text = screen.find(label).unwrap_or_else(|| panic!("no {label}"));
    let target = screen.press_target(text);
    engine.control().dispatch_event(target, kind, payload);
    let mut started = Vec::new();
    while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
        screen.apply(&batch);
        for item in &batch.items {
            if let MountItem::Animation(AnimationOp::Start { driver, .. }) = item {
                started.push(*driver);
            }
        }
    }
    started
}

#[test]
fn physics_counts_releases() {
    let (engine, rx, mut screen) = crate::common::open("Physics");
    assert!(shows(
        &screen,
        "Drag the card, tap it to pulse, long-press to reset"
    ));
    assert!(shows(&screen, "Released 0 times"), "{:?}", screen.texts);

    let release = EventPayload::Pan {
        translation_x: 80.0,
        translation_y: 20.0,
        velocity_x: 300.0,
        velocity_y: -100.0,
        value_x: Some(80.0),
        value_y: Some(20.0),
    };
    let springs = dispatch(
        &engine,
        &rx,
        &mut screen,
        "Drag me",
        EventKind::PanEnd,
        release.clone(),
    );
    // Back to the middle, with the drag's velocity.
    let spring = |velocity| Driver::Spring {
        to: 0.0,
        stiffness: 180.0,
        damping: 14.0,
        mass: 1.0,
        velocity: Some(velocity),
    };
    assert_eq!(springs, [spring(300.0), spring(-100.0)]);
    dispatch(
        &engine,
        &rx,
        &mut screen,
        "Drag me",
        EventKind::PanEnd,
        release,
    );
    assert!(shows(&screen, "Released 2 times"), "{:?}", screen.texts);

    // A tap pulses (natively; the count stays): up first.
    let pulse = dispatch(
        &engine,
        &rx,
        &mut screen,
        "Drag me",
        EventKind::Press,
        EventPayload::None,
    );
    assert!(
        matches!(
            pulse[..],
            [Driver::Timing {
                to: 1.0,
                duration_ms: 120.0,
                ..
            }]
        ),
        "{pulse:?}"
    );
    assert!(shows(&screen, "Released 2 times"), "{:?}", screen.texts);

    dispatch(
        &engine,
        &rx,
        &mut screen,
        "Drag me",
        EventKind::LongPress,
        EventPayload::None,
    );
    assert!(shows(&screen, "Released 0 times"), "{:?}", screen.texts);
}
