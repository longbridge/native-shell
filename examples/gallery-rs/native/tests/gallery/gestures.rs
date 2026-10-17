//! The Gestures demo on a headless engine: press events are logged and
//! counted, taps reach the button under the overlay, touches are logged
//! with their fingers, the ball is dragged as the touch responder, the
//! photo takes pinches and rotations, labelled items report presses.

use nsh_core::{Command, ROOT_ID, TouchPhase};

use crate::common::{Screen, shows, touch};

/// How often the text `text` is on screen.
fn count(screen: &Screen, text: &str) -> usize {
    screen.texts.iter().filter(|t| t.1 == text).count()
}

#[test]
fn gestures_log_presses_and_touches() {
    let (engine, rx, mut screen) = crate::common::open("Gestures");
    for text in [
        "Pan, pinch and rotation",
        "Press events",
        "No events yet",
        "No touches yet",
        "0 down",
        "Idle",
        "Touch here, try two fingers",
        "0 taps",
        "hitSlop={30}",
        "pointerEvents=\"none\"",
        "Long press after 250 ms",
        "pressIn",
        "longPress",
        "Starts the song",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }

    // Taps go through the hatched overlay to the button.
    let zeros = count(&screen, "0");
    crate::common::press(&engine, &rx, &mut screen, "Button under an overlay");
    assert_eq!(count(&screen, "0"), zeros - 1, "{:?}", screen.texts);
    assert_eq!(count(&screen, "1"), 1, "{:?}", screen.texts);

    crate::common::press(&engine, &rx, &mut screen, "Tap, hold, or slide off");
    assert!(shows(&screen, "press"), "{:?}", screen.texts);
    assert_eq!(count(&screen, "1"), 2, "counted: {:?}", screen.texts);

    // A labelled item shows its label in the card's badge.
    assert_eq!(count(&screen, "Like"), 1);
    crate::common::press(&engine, &rx, &mut screen, "Like");
    assert_eq!(count(&screen, "Like"), 2, "{:?}", screen.texts);

    let label = screen.find("Touch here, try two fingers").unwrap();
    let area = screen.parents[&label];
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::Start,
        &[area],
        (10.4, 20.6),
        1000.0,
    );
    assert!(
        shows(&screen, "10, 21 · page 110, 121 · 1 down"),
        "{:?}",
        screen.texts
    );
    assert!(shows(&screen, "start") && shows(&screen, "1 down"));
    assert!(!shows(&screen, "Touch here, try two fingers"));
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::End,
        &[area],
        (12.0, 22.0),
        1016.0,
    );
    assert!(
        shows(&screen, "12, 22 · page 112, 122 · 0 down"),
        "{:?}",
        screen.texts
    );
    assert!(shows(&screen, "end") && shows(&screen, "Touch here, try two fingers"));
}

#[test]
fn the_ball_claims_its_touches_and_the_photo_pinches_and_rotates() {
    let (engine, rx, mut screen) = crate::common::open("Gestures");
    for text in [
        "PanResponder",
        "Positions go through Rust; the page does not scroll",
        "Pinch and rotate with two fingers (Option-drag in the simulator).",
        "1.00×",
        "0°",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }
    // The photo: drags, pinches and rotations drive its values natively.
    let card = screen
        .gestures
        .values()
        .find(|g| g.pan_x.is_some())
        .expect("the photo's gesture values");
    assert!(card.pinch.is_some() && card.rotation.is_some(), "{card:?}");

    // The ball: the touch listener that is not the touch pad.
    let area = screen.parents[&screen.find("Touch here, try two fingers").unwrap()];
    let ball = *screen
        .touch_listeners()
        .iter()
        .find(|&&id| id != area)
        .expect("the ball listens to touches");
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::Start,
        &[ball],
        (30.0, 30.0),
        0.0,
    );
    assert!(shows(&screen, "Dragging"), "{:?}", screen.texts);
    assert_eq!(
        screen.commands.last(),
        Some(&(ball, Command::SetJsResponder { block: true })),
        "the page stops scrolling"
    );
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::Move,
        &[ball],
        (50.0, 30.0),
        16.0,
    );
    assert!(shows(&screen, "20, 0"), "{:?}", screen.texts);
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::End,
        &[ball],
        (50.0, 30.0),
        32.0,
    );
    assert!(shows(&screen, "0.88, 0.00 pt/ms"), "{:?}", screen.texts);
    assert_eq!(
        screen.commands.last(),
        Some(&(ROOT_ID, Command::ClearJsResponder))
    );
    crate::common::press(&engine, &rx, &mut screen, "Center");
    assert!(shows(&screen, "0, 0"), "{:?}", screen.texts);
}
