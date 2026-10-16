//! The Scrolling demo on a headless engine: its sections render, the
//! cards snap on a recycled list, drags and offsets are logged, the
//! toggles change how the box scrolls, the zoom shows what the scroll view
//! reports, the feed and the keyboard taps choice change what shows.

use crossbeam_channel::Receiver;
use nsh_core::{EventKind, EventPayload, MountBatch, SnapAlignment};
use nsh_engine::Engine;

use crate::common::{Screen, press_text, shows};

/// Presses the switch in the row labeled `label`.
fn toggle(engine: &Engine, rx: &Receiver<MountBatch>, screen: &mut Screen, label: &str) {
    let text = screen
        .find(label)
        .unwrap_or_else(|| panic!("no {label}: {:?}", screen.texts));
    let row = screen.parents[&text];
    let switch = screen
        .parents
        .iter()
        .find(|&(&child, &parent)| parent == row && child != text)
        .map(|(&child, _)| child)
        .expect("a switch");
    engine
        .control()
        .dispatch_event(switch, EventKind::Press, EventPayload::None);
    crate::common::drain(rx, screen);
}

#[test]
fn scrolling_sections_react() {
    let (engine, rx, mut screen) = crate::common::open("Scrolling");
    for label in [
        "PAGING",
        "Page 1",
        "SNAPPING (SNAPTOINTERVAL)",
        "Card 1",
        "center",
        "ZOOM (PINCH, TAP A TILE)",
        "Tile 1",
        "Zoom 1.00×",
        "EVENTS AND BEHAVIOR",
        "Line 1",
        "Scroll the box",
        "No drags yet",
        "Scroll enabled",
        "Bounces",
        "One card per fling",
        "Bounces past the limits",
        "PREPENDING KEEPS THE POSITION",
        "Post 1",
        "Pressed 0 times",
        "A tap outside the input only hides the keyboard; the button does not get it.",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }

    // The cards snap on a recycled list; one per fling once toggled.
    let cards = screen
        .scroll_of(screen.find("Card 1").unwrap())
        .expect("the cards");
    let snap = |screen: &crate::common::Screen| {
        let b = &screen.behaviors[&cards];
        (
            b.snap_interval,
            b.snap_alignment,
            b.deceleration,
            b.one_step,
        )
    };
    assert_eq!(snap(&screen), (232.0, SnapAlignment::Center, 0.99, false));
    toggle(&engine, &rx, &mut screen, "One card per fling");
    assert!(snap(&screen).3);
    press_text(&engine, &rx, &mut screen, "start");
    assert_eq!(snap(&screen).1, SnapAlignment::Start);

    // A drag on the events' box is logged.
    let line = screen.find("Line 1").expect("a line");
    let scroll = screen.parents[&screen.parents[&line]];
    engine.control().dispatch_event(
        scroll,
        EventKind::ScrollBeginDrag,
        EventPayload::Scroll { x: 0.0, y: 12.0 },
    );
    crate::common::drain(&rx, &mut screen);
    assert!(shows(&screen, "begin drag @ 12"), "{:?}", screen.texts);
    // Its offset, as `on_scroll` reports it.
    engine.control().scroll(scroll, 0.0, 40.0, u32::MAX);
    crate::common::drain(&rx, &mut screen);
    assert!(
        screen.texts.iter().any(|t| t.1.starts_with("y 40 of ")),
        "{:?}",
        screen.texts
    );
    assert!(screen.behaviors[&scroll].enabled);
    toggle(&engine, &rx, &mut screen, "Scroll enabled");
    assert!(!screen.behaviors[&scroll].enabled, "scrolling turned off");

    // The zoom label shows the zoom the scroll view reports.
    let tile = screen.find("Tile 1").unwrap();
    let zoom = screen.scroll_of(tile).expect("the zooming scroll view");
    crate::common::press(&engine, &rx, &mut screen, "2×");
    // The platform reports the zoom, then the offset it moved to.
    engine.control().zoom(zoom, 2.0);
    engine.control().scroll(zoom, 40.0, 30.0, u32::MAX);
    crate::common::drain(&rx, &mut screen);
    assert!(shows(&screen, "Zoom 2.00×"), "{:?}", screen.texts);
    assert!(screen.behaviors[&zoom].bounces_zoom);
    toggle(&engine, &rx, &mut screen, "Bounces past the limits");
    assert!(!screen.behaviors[&zoom].bounces_zoom);

    crate::common::press(&engine, &rx, &mut screen, "Add 3 posts on top");
    assert!(shows(&screen, "New 1"), "{:?}", screen.texts);

    press_text(&engine, &rx, &mut screen, "handled");
    assert!(
        shows(
            &screen,
            "The button gets the tap and the keyboard stays; a tap on empty space hides it."
        ),
        "{:?}",
        screen.texts
    );

    crate::common::press(&engine, &rx, &mut screen, "Pressed 0 times");
    assert!(shows(&screen, "Pressed 1 times"), "{:?}", screen.texts);
}
