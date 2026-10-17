//! The Form demo on a headless engine: its fields render, leaving or
//! submitting checks them, the password's strength follows the text, the
//! dismiss modes and the sheet change what it shows, the message bar
//! pushes and sends.

use std::time::Duration;

use nsh_core::{Command, EventKind, EventPayload, MountItem, PayloadValue};
use nsh_engine::Engine;

use crate::common::{Screen, drain, press, shows};

/// The input of the field labelled `label`: in the box next to the label's
/// row, listening to text changes.
fn field_input(screen: &Screen, label: &str) -> u32 {
    let label = screen.find(label).unwrap_or_else(|| panic!("no {label}"));
    let row = screen.parents[&label];
    let field = screen.parents[&row];
    screen
        .parents
        .iter()
        .filter(|&(_, &parent)| parent != field)
        .map(|(&child, &parent)| (child, parent))
        .find(|&(child, parent)| {
            screen.parents.get(&parent) == Some(&field)
                && screen
                    .events
                    .get(&child)
                    .is_some_and(|e| e.contains(EventKind::ChangeText))
        })
        .map(|(child, _)| child)
        .expect("the field's input")
}

/// Types `text` into `input`.
fn type_text(
    engine: &Engine,
    rx: &crossbeam_channel::Receiver<nsh_core::MountBatch>,
    screen: &mut Screen,
    input: u32,
    text: &str,
) {
    engine.control().dispatch_event(
        input,
        EventKind::ChangeText,
        EventPayload::Text {
            text: text.into(),
            count: 1,
        },
    );
    drain(rx, screen);
}

#[test]
fn form_renders_and_submits() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    for label in [
        "Keyboard",
        "keyboardDismissMode while dragging",
        "0 pt",
        "Drag the form with the keyboard up",
        "on-drag",
        "Account",
        "autoComplete",
        "E-mail",
        "email-address",
        "Empty",
        "Verification code",
        "0/6",
        "Shipping address",
        "0–0",
        "\"\"",
        "Keys you press show here",
        "Notes",
        "0/280",
        "Grows with its text",
        "No caret",
        "caretHidden",
        "Create the account",
        "More keyboards",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // Submitting an empty form flags its three required fields.
    press(&engine, &rx, &mut screen, "Submit");
    for label in [
        "3 fields need attention",
        "Check",
        "Enter your name",
        "Enter your e-mail",
        "Use at least 8 characters",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // Fixing them clears the errors; submitting again creates the account.
    let name = field_input(&screen, "Name");
    type_text(&engine, &rx, &mut screen, name, "Ada");
    assert!(!shows(&screen, "Enter your name"), "{:?}", screen.texts);
    let email = field_input(&screen, "E-mail");
    type_text(&engine, &rx, &mut screen, email, "ada@x");
    assert!(
        shows(&screen, "That address needs an @ and a domain"),
        "{:?}",
        screen.texts
    );
    type_text(&engine, &rx, &mut screen, email, "ada@example.com");
    let password = field_input(&screen, "Password");
    // Three letters are not even weak.
    type_text(&engine, &rx, &mut screen, password, "abc");
    assert!(shows(&screen, "Empty"), "{:?}", screen.texts);
    type_text(&engine, &rx, &mut screen, password, "abcdefgh");
    assert!(shows(&screen, "Weak"), "{:?}", screen.texts);
    type_text(&engine, &rx, &mut screen, password, "Abcdefg1!");
    assert!(shows(&screen, "Strong"), "{:?}", screen.texts);
    press(&engine, &rx, &mut screen, "Submit");
    assert!(
        shows(&screen, "Account created for Ada <ada@example.com>"),
        "{:?}",
        screen.texts
    );
    assert!(shows(&screen, "Sent"), "{:?}", screen.texts);
}

#[test]
fn form_switches_modes_and_opens_the_sheet() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    press(&engine, &rx, &mut screen, "Interactive");
    assert!(shows(&screen, "interactive"), "{:?}", screen.texts);
    // Leaving the name empty flags it.
    let name = field_input(&screen, "Name");
    engine
        .control()
        .dispatch_event(name, EventKind::Blur, EventPayload::None);
    drain(&rx, &mut screen);
    assert!(shows(&screen, "Enter your name"), "{:?}", screen.texts);
    press(&engine, &rx, &mut screen, "Inputs in a sheet");
    assert!(shows(&screen, "A sheet"), "{:?}", screen.texts);
    assert!(shows(&screen, "Seventh"), "{:?}", screen.texts);
    press(&engine, &rx, &mut screen, "Done");
    assert!(!shows(&screen, "A sheet"), "{:?}", screen.texts);
}

#[test]
fn form_pushes_the_message_bar() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    press(&engine, &rx, &mut screen, "A message bar at the bottom");
    assert!(
        screen.titles.iter().any(|t| t.1 == "Message bar"),
        "{:?}",
        screen.titles
    );
    for label in [
        "Today",
        "The bar below sits outside the scroll view.",
        "KeyboardAvoidingView lifts it with the keyboard. Say something.",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // Return sends the draft.
    let input = *screen
        .events
        .iter()
        .find(|(id, e)| {
            e.contains(EventKind::SubmitEditing) && screen.find("Today").is_some_and(|t| **id > t)
        })
        .expect("the message input")
        .0;
    type_text(&engine, &rx, &mut screen, input, "Hello there");
    engine.control().dispatch_event(
        input,
        EventKind::SubmitEditing,
        EventPayload::Text {
            text: "Hello there".into(),
            count: 1,
        },
    );
    drain(&rx, &mut screen);
    assert!(shows(&screen, "Hello there"), "{:?}", screen.texts);
}

#[test]
fn form_selects_through_the_controlled_selection() {
    let (engine, rx, mut screen) = crate::common::open("Form");
    let selections = |label: &str, screen: &mut Screen| {
        let text = screen.find(label).expect(label);
        engine.control().dispatch_event(
            screen.press_target(text),
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
            ("end", PayloadValue::Number(6.0)),
        ]),
    );
    drain(&rx, &mut screen);
    assert!(shows(&screen, "0–6"), "{:?}", screen.texts);
    assert!(shows(&screen, "\"Select\""), "{:?}", screen.texts);
    assert_eq!(selections("Select all", &mut screen), [(input, 0, 24)]);
    // Keys show as chips.
    engine.control().dispatch_event(
        input,
        EventKind::KeyPress,
        EventPayload::Map(vec![("key", PayloadValue::Str("a".into()))]),
    );
    drain(&rx, &mut screen);
    assert!(
        !shows(&screen, "Keys you press show here"),
        "{:?}",
        screen.texts
    );
}
