//! The Mobile UI demo on a headless engine: the header's components and
//! the first rows of the `ListView` render, the header's buttons and
//! steppers change its state, which stays while the list scrolls far
//! away, and the header asks a stepper for its value through its ref.

use nsh_core::{EventKind, EventPayload};

use crate::common::{Screen, drain, shows};

/// The ancestors of `id`, nearest first.
fn ancestors(screen: &Screen, mut id: u32) -> Vec<u32> {
    let mut out = Vec::new();
    while let Some(&parent) = screen.parents.get(&id) {
        out.push(parent);
        id = parent;
    }
    out
}

#[test]
fn mobile_ui_renders_and_answers_presses() {
    let (engine, rx, mut screen) = crate::common::open("Mobile UI");
    for label in [
        "Name (controlled; × clears and focuses)",
        "Hello, Ada",
        "SelectList: 500 rows of Rust data, chosen: none",
        "City 1",
        "Group: each child is a row",
        "Notifications",
        "Controlled: 2",
        "Ask the stepper, then reset it",
        "Inset rows",
        // The first rows below the header.
        "SYM000",
        "-2.50%",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // The header's buttons are nested components: their events change the
    // header's state.
    crate::common::press(&engine, &rx, &mut screen, "Tap");
    crate::common::press(&engine, &rx, &mut screen, "Tap");
    let taps = screen.find("Taps").unwrap();
    let row = screen.parents[&taps];
    let count = screen
        .texts
        .iter()
        .find(|t| t.0 != taps && screen.parents.get(&t.0) == Some(&row))
        .map(|t| t.1.clone());
    assert_eq!(count.as_deref(), Some("2"), "{:?}", screen.texts);

    // The select list's rows are the select list's own.
    crate::common::press(&engine, &rx, &mut screen, "City 3");
    assert!(
        shows(&screen, "SelectList: 500 rows of Rust data, chosen: City 3"),
        "{:?}",
        screen.texts
    );

    // The stepper with its own value: one step up, then asked for its
    // value and reset through its ref.
    let five = screen.find("5").expect("the own stepper's value");
    let stepper = screen.parents[&five];
    let plus = screen
        .texts
        .iter()
        .filter(|t| t.1 == "+")
        .map(|t| screen.parents[&t.0])
        .find(|button| screen.parents.get(button) == Some(&stepper))
        .expect("its + button");
    for kind in [EventKind::PressIn, EventKind::PressOut] {
        engine
            .control()
            .dispatch_event(plus, kind, EventPayload::None);
        drain(&rx, &mut screen);
    }
    assert_eq!(
        screen
            .texts
            .iter()
            .find(|t| t.0 == five)
            .map(|t| t.1.as_str()),
        Some("6")
    );
    crate::common::press(&engine, &rx, &mut screen, "Ask the stepper, then reset it");
    assert!(shows(&screen, "It was 6; reset to 5"), "{:?}", screen.texts);
    assert_eq!(
        screen
            .texts
            .iter()
            .find(|t| t.0 == five)
            .map(|t| t.1.as_str()),
        Some("5")
    );

    // The header is the list's own (not a recycled row): its state stays
    // while the list scrolls far away and back.
    let header = ancestors(&screen, screen.find("Notifications").unwrap());
    let row = ancestors(&screen, screen.find("SYM000").unwrap());
    let list = *header.iter().find(|a| row.contains(a)).expect("one list");
    engine.control().scroll(list, 0.0, 12000.0, u32::MAX);
    drain(&rx, &mut screen);
    assert!(
        screen.texts.iter().any(|t| t.1.starts_with("SYM2")),
        "rows far down: {:?}",
        screen.texts
    );
    engine.control().scroll(list, 0.0, 0.0, u32::MAX);
    drain(&rx, &mut screen);
    assert!(shows(&screen, "It was 6; reset to 5"), "{:?}", screen.texts);

    crate::common::press(&engine, &rx, &mut screen, "Inset rows");
    assert!(shows(&screen, "Plain rows"), "{:?}", screen.texts);
}
