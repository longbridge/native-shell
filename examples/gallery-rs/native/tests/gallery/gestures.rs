//! The Gestures demo on a headless engine: press events are logged, taps
//! reach the button under the overlay, touches are logged with their
//! fingers, the ball is dragged as the touch responder, the card takes
//! pinches and rotations.

use nsh_core::{Command, ROOT_ID, TouchPhase};

use crate::common::{shows, touch};

#[test]
fn gestures_log_presses_and_touches() {
    let (engine, rx, mut screen) = crate::common::open("Gestures");
    for text in [
        "PRESS EVENTS",
        "No events yet",
        "No touches yet",
        "idle",
        "Touch here (try two fingers)",
        "The dot takes taps 30 pt around it: 0",
        "Ripple, long press 250 ms",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }

    crate::common::press(&engine, &rx, &mut screen, "Tap, hold, or slide off");
    assert!(shows(&screen, "press"), "{:?}", screen.texts);
    crate::common::press(
        &engine,
        &rx,
        &mut screen,
        "Like (labelled for VoiceOver / TalkBack)",
    );
    assert!(shows(&screen, "Like · press"), "{:?}", screen.texts);

    crate::common::press(&engine, &rx, &mut screen, "Button under an overlay: 0");
    assert!(
        shows(&screen, "Button under an overlay: 1"),
        "{:?}",
        screen.texts
    );

    let label = screen.find("Touch here (try two fingers)").unwrap();
    let area = screen.parents[&label];
    let at = (10.4, 20.6);
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::Start,
        &[area],
        at,
        1000.0,
    );
    assert!(
        shows(&screen, "start at 10,21 (page 110,121), 1 down"),
        "{:?}",
        screen.texts
    );
    let at = (12.0, 22.0);
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::End,
        &[area],
        at,
        1016.0,
    );
    assert!(
        shows(
            &screen,
            "end at 12,22 (page 112,122), 0 down\nstart at 10,21 (page 110,121), 1 down"
        ),
        "{:?}",
        screen.texts
    );
}

#[test]
fn the_ball_claims_its_touches_and_the_card_pinches_and_rotates() {
    let (engine, rx, mut screen) = crate::common::open("Gestures");
    for text in [
        "PAN, PINCH AND ROTATION",
        "PANRESPONDER (DRAG THE BALL; THE PAGE DOES NOT SCROLL)",
        "Drag with one finger; pinch and rotate with two (Option-drag in the simulator).",
        "The dashed overlay has pointerEvents=\"none\": taps reach the button.",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }
    // The card: drags, pinches and rotations drive its values natively.
    let card = screen
        .gestures
        .values()
        .find(|g| g.pan_x.is_some())
        .expect("the card's gesture values");
    assert!(card.pinch.is_some() && card.rotation.is_some(), "{card:?}");

    // The ball: the touch listener that is not the touch log's area.
    let area = screen.parents[&screen.find("Touch here (try two fingers)").unwrap()];
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
    assert!(shows(&screen, "dragging"), "{:?}", screen.texts);
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
    touch(
        &engine,
        &rx,
        &mut screen,
        TouchPhase::End,
        &[ball],
        (50.0, 30.0),
        32.0,
    );
    assert!(
        shows(&screen, "released at 0.88, 0.00 pt/ms"),
        "{:?}",
        screen.texts
    );
    assert_eq!(
        screen.commands.last(),
        Some(&(ROOT_ID, Command::ClearJsResponder))
    );
}
