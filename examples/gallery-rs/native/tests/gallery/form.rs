//! The Form demo on a headless engine: its inputs render, submitting and
//! the dismiss modes change what it shows, the message bar pushes.

use std::time::Duration;

use nsh_core::{Command, EventKind, EventPayload, MountItem, PayloadValue};

use crate::common::{press_text, shows};

#[test]
fn form_renders_and_submits() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    for label in [
        "Keyboard 0 pt",
        "ACCOUNT",
        "E-mail",
        "Code (6 digits)",
        "Selection 0–0: \"\"",
        "Keys: –",
        "Notes (multiline, grows, 280 max)",
        "0/280",
        "caretHidden",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    crate::common::press(&engine, &rx, &mut screen, "Submit");
    assert!(shows(&screen, "Nobody <no e-mail>"), "{:?}", screen.texts);
    // The modes are texts taking presses; one changes the scroll view's
    // mode and renders again (the keyboard line stays).
    press_text(&engine, &rx, &mut screen, "interactive");
    assert!(shows(&screen, "Keyboard 0 pt"));
    crate::common::press(&engine, &rx, &mut screen, "Inputs in a sheet");
    assert!(shows(&screen, "A sheet"), "{:?}", screen.texts);
    assert!(shows(&screen, "Seventh"), "{:?}", screen.texts);
}

#[test]
fn form_pushes_the_message_bar() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    crate::common::press(&engine, &rx, &mut screen, "A message bar at the bottom");
    assert!(
        screen.titles.iter().any(|t| t.1 == "Message bar"),
        "{:?}",
        screen.titles
    );
    assert!(shows(&screen, "Send"), "{:?}", screen.texts);
}

#[test]
fn form_selects_through_the_controlled_selection() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    let selections = |label: &str, screen: &mut crate::common::Screen| {
        let text = screen.find(label).expect(label);
        engine.control().dispatch_event(
            screen.parents[&text],
            EventKind::Press,
            EventPayload::None,
        );
        let mut selections = Vec::new();
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            screen.apply(&batch);
            for item in &batch.items {
                if let MountItem::Command {
                    id,
                    command: Command::SetSelection { start, end },
                } = item
                {
                    selections.push((*id, *start, *end));
                }
            }
        }
        selections
    };
    // "Select part of this text": 24 UTF-16 units.
    let all = selections("Select all", &mut screen);
    let [(input, 0, 24)] = all[..] else {
        panic!("{all:?}");
    };
    assert_eq!(selections("Caret to start", &mut screen), [(input, 0, 0)]);
    // The input reports where the selection went; "Select all" selects
    // again.
    engine.control().dispatch_event(
        input,
        EventKind::SelectionChange,
        EventPayload::Map(vec![
            ("start", PayloadValue::Number(0.0)),
            ("end", PayloadValue::Number(0.0)),
        ]),
    );
    crate::common::drain(&rx, &mut screen);
    assert_eq!(selections("Select all", &mut screen), [(input, 0, 24)]);
}
