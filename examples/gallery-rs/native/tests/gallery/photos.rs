//! The Photos demo on a headless engine: its rows render, the list feeds
//! the header's native value, more load at the end, a pull refreshes.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{EventKind, EventPayload, MountBatch, MountItem};
use nsh_engine::Engine;

use crate::common::shows;

/// Sends `kind` to every view above the text `label` (the one listening
/// handles it), and waits for what it changes.
fn dispatch_above(
    engine: &Engine,
    rx: &Receiver<MountBatch>,
    screen: &mut crate::common::Screen,
    label: &str,
    kind: EventKind,
) {
    let mut node = screen
        .find(label)
        .unwrap_or_else(|| panic!("no {label}: {:?}", screen.texts));
    while let Some(&parent) = screen.parents.get(&node) {
        engine
            .control()
            .dispatch_event(parent, kind, EventPayload::None);
        node = parent;
    }
    crate::common::drain(rx, screen);
}

#[test]
fn photos_load_more_and_refresh() {
    let (engine, rx, mut screen) = crate::common::open("Photos");
    for label in [
        "1000 photos",
        "Photo #1",
        "by Linus",
        "Photo #2",
        "by Grace",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }

    // The list feeds the header's fade natively.
    let list = screen
        .scroll_of(screen.find("Photo #1").unwrap())
        .expect("a list");
    assert!(
        matches!(
            screen.scrolls.get(&list),
            Some(MountItem::ScrollConfig {
                scroll_value: Some(_),
                ..
            })
        ),
        "{:?}",
        screen.scrolls.get(&list)
    );

    // The end of the list loads 200 more.
    dispatch_above(&engine, &rx, &mut screen, "Photo #1", EventKind::EndReached);
    assert!(shows(&screen, "1200 photos"), "{:?}", screen.texts);

    // A pull refreshes: back to the first 1000 after a moment.
    dispatch_above(&engine, &rx, &mut screen, "Photo #1", EventKind::Refresh);
    std::thread::sleep(Duration::from_millis(900));
    crate::common::drain(&rx, &mut screen);
    assert!(shows(&screen, "1000 photos"), "{:?}", screen.texts);
}
