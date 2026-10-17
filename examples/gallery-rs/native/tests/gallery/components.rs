//! The Components demo on a headless engine: its sections render, the
//! mobile UI's buttons and the press feedback cards answer presses, the
//! keyboard avoiding view's screen pushes, the platform's rows and the
//! screen reader's buttons, the input accessory and the modal's options.

use std::time::Duration;

use nsh_core::{Command, EventKind, EventPayload, MountItem, ROOT_ID, ViewKind};

use crate::common::shows;

#[test]
fn components_render_and_answer_presses() {
    let (engine, rx, mut screen) = crate::common::open("Components");
    for label in [
        "NSH/UI/MOBILE",
        "Primary (0)",
        "Card with slots",
        "An outlined card, children only.",
        "PRESS FEEDBACK",
        "Last pressed: nothing yet",
        "Open a sheet",
        "LAYOUT ANIMATION",
        "0 characters",
        "MODAL OPTIONS",
        "statusBarTranslucent: false",
        "Platform.OS",
        "ios",
        "Platform.Version",
        "isPad / isTV / isMacCatalyst",
        "false / false / false",
        "ios branch",
        "PixelRatio.get()",
        "roundToNearestPixel(10.3)",
        "10",
        "reduceMotion()",
        "The line above is one device pixel tall.",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // A nested Rust component's event reaches this demo's state.
    crate::common::press(&engine, &rx, &mut screen, "Primary (0)");
    crate::common::press(&engine, &rx, &mut screen, "Secondary");
    assert!(shows(&screen, "Primary (2)"), "{:?}", screen.texts);
    // Its slot's content: the footer's button.
    crate::common::press(&engine, &rx, &mut screen, "Reset");
    assert!(shows(&screen, "Primary (0)"), "{:?}", screen.texts);

    crate::common::press(
        &engine,
        &rx,
        &mut screen,
        "activeOpacity: fades while pressed",
    );
    assert!(
        shows(&screen, "Last pressed: activeOpacity"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "Disable");
    assert!(
        shows(&screen, "disabled: takes no presses") && shows(&screen, "Enable"),
        "{:?}",
        screen.texts
    );

    // Layout animation: the boxes 1, 2, 3; Add puts 4 first (the index
    // under the demo shows a 4 of its own).
    let fours = |screen: &crate::common::Screen| screen.texts.iter().filter(|t| t.1 == "4").count();
    let before = fours(&screen);
    crate::common::press(&engine, &rx, &mut screen, "Add");
    assert_eq!(fours(&screen), before + 1, "{:?}", screen.texts);

    crate::common::press(&engine, &rx, &mut screen, "statusBarTranslucent: false");
    assert!(shows(&screen, "statusBarTranslucent: true"));
    crate::common::press(&engine, &rx, &mut screen, "Open");
    assert!(
        shows(
            &screen,
            "The yellow background starts under the status bar on Android."
        ),
        "{:?}",
        screen.texts
    );
}

#[test]
fn components_push_the_keyboard_avoiding_view() {
    let (engine, rx, mut screen) = crate::common::open("Components");
    crate::common::press(&engine, &rx, &mut screen, "padding / height / position");
    assert!(
        screen.titles.iter().any(|t| t.1 == "KeyboardAvoidingView"),
        "{:?}",
        screen.titles
    );
    assert!(shows(&screen, "behavior=\"padding\""), "{:?}", screen.texts);
    crate::common::press(&engine, &rx, &mut screen, "position");
    assert!(
        shows(&screen, "behavior=\"position\""),
        "{:?}",
        screen.texts
    );
}

#[test]
fn components_announce_and_move_the_readers_focus() {
    let (engine, rx, mut screen) = crate::common::open("Components");
    let note = screen
        .find("With VoiceOver or TalkBack on, the buttons above speak and move the reader here.")
        .expect("the note");
    let commands = |label: &str, screen: &mut crate::common::Screen| {
        let text = screen.find(label).expect(label);
        engine.control().dispatch_event(
            screen.press_target(text),
            EventKind::Press,
            EventPayload::None,
        );
        let mut commands = Vec::new();
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            screen.apply(&batch);
            for item in &batch.items {
                if let MountItem::Command { id, command } = item {
                    commands.push((*id, command.clone()));
                }
            }
        }
        commands
    };
    assert!(
        commands("Announce", &mut screen)
            .contains(&(ROOT_ID, Command::Announce("Announcement 1".into()))),
    );
    assert!(
        commands("Announce", &mut screen)
            .contains(&(ROOT_ID, Command::Announce("Announcement 2".into()))),
    );
    assert!(commands("Focus the note", &mut screen).contains(&(note, Command::AccessibilityFocus)));
}

#[test]
fn components_put_the_accessory_bar_over_the_keyboard() {
    let (engine, rx) = crate::common::start();
    let mut screen = crate::common::Screen::default();
    let mut kinds = Vec::new();
    let mut collect = |screen: &mut crate::common::Screen| {
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            screen.apply(&batch);
            for item in &batch.items {
                if let MountItem::Create { id, kind, .. } = item {
                    kinds.push((*id, *kind));
                }
            }
        }
    };
    collect(&mut screen);
    let text = screen.find("Components").expect("the index's row");
    engine.control().dispatch_event(
        screen.press_target(text),
        EventKind::Press,
        EventPayload::None,
    );
    collect(&mut screen);
    // The bar is an input accessory (iOS): its content is mounted in it.
    let accessory = kinds
        .iter()
        .find(|(_, kind)| *kind == ViewKind::InputAccessory)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no input accessory: {kinds:?}"));
    let counter = screen.find("0 characters").expect("the counter");
    let mut parent = screen.parents[&counter];
    while parent != accessory {
        parent = *screen
            .parents
            .get(&parent)
            .unwrap_or_else(|| panic!("the counter is not in the accessory"));
    }
}
