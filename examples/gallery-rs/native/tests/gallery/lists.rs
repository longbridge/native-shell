//! The Lists demo on a headless engine: its tabs switch, the grid's
//! header, footer and visible rows, the sticky section headers, the chat
//! sends, dismisses the keyboard and loads older messages, the empty list
//! fills and clears.

use nsh_core::{EventKind, EventPayload, KeyboardDismissMode, MountItem, PayloadValue};

use crate::common::{drain, press_text, shows};

#[test]
fn lists_switch_tabs_and_change_their_rows() {
    let (engine, rx, mut screen) = crate::common::open("Lists");
    for label in [
        "Grid",
        "Sections",
        "Chat",
        "More",
        // The core reports the rows it shows.
        "Visible 0–14",
        "Item 60",
        "End",
        "0",
        "1",
        "— the end —",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    // The header scrolls with the rows: it is in the grid's list.
    let grid = screen
        .scroll_of(screen.find("0").unwrap())
        .expect("the grid");
    assert_eq!(
        screen.scroll_of(screen.find("Item 60").unwrap()),
        Some(grid)
    );
    engine.control().dispatch_event(
        grid,
        EventKind::ViewableItemsChanged,
        EventPayload::Map(vec![
            ("first", PayloadValue::Number(0.0)),
            ("last", PayloadValue::Number(8.0)),
        ]),
    );
    drain(&rx, &mut screen);
    assert!(shows(&screen, "Visible 0–8"), "{:?}", screen.texts);

    press_text(&engine, &rx, &mut screen, "Sections");
    for label in ["A", "Ada", "Alan", "B", "Barbara"] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    assert!(!shows(&screen, "Item 60"), "the grid is gone");
    let contacts = screen.scroll_of(screen.find("Ada").unwrap()).unwrap();
    assert!(screen.sticky.contains(&contacts), "the letters stick");

    // The chat: type, then send.
    press_text(&engine, &rx, &mut screen, "Chat");
    assert!(shows(&screen, "Message 30"), "{:?}", screen.texts);
    let send = screen.find("Send").expect("a send button");
    let bar = screen.parents[&send];
    let input = screen
        .parents
        .iter()
        .find(|&(&child, &parent)| parent == bar && child != send)
        .map(|(&child, _)| child)
        .expect("the message input");
    engine.control().dispatch_event(
        input,
        EventKind::ChangeText,
        EventPayload::Text {
            text: "Hello there".into(),
            count: 1,
        },
    );
    crate::common::drain(&rx, &mut screen);
    press_text(&engine, &rx, &mut screen, "Send");
    assert!(shows(&screen, "Hello there"), "{:?}", screen.texts);
    let chat = screen
        .scroll_of(screen.find("Message 30").unwrap())
        .unwrap();
    assert!(
        matches!(
            screen.scrolls.get(&chat),
            Some(MountItem::ScrollConfig {
                keyboard_dismiss: KeyboardDismissMode::Interactive,
                ..
            })
        ),
        "{:?}",
        screen.scrolls.get(&chat)
    );
    // Near its end (the oldest messages), older ones load.
    engine
        .control()
        .dispatch_event(chat, EventKind::EndReached, EventPayload::None);
    drain(&rx, &mut screen);
    engine.control().scroll(chat, 0.0, 0.0, u32::MAX);
    drain(&rx, &mut screen);
    assert!(
        screen.texts.iter().any(|t| t.1.starts_with("Older")),
        "{:?}",
        screen.texts
    );

    // The empty list: Add fills it, Clear empties it.
    press_text(&engine, &rx, &mut screen, "More");
    for label in [
        "Pull down to refresh (refreshed 0 times)",
        "Today",
        "A horizontal list of 1000 recycled cards",
        "An empty list (0 items)",
        "Nothing here yet: tap Add",
    ] {
        assert!(shows(&screen, label), "{label}: {:?}", screen.texts);
    }
    press_text(&engine, &rx, &mut screen, "Add");
    press_text(&engine, &rx, &mut screen, "Add");
    assert!(
        shows(&screen, "An empty list (2 items)"),
        "{:?}",
        screen.texts
    );
    assert!(shows(&screen, "Item 1") && shows(&screen, "Item 2"));
    assert!(!shows(&screen, "Nothing here yet: tap Add"));
    // A separator after the first row, in its cell.
    let first = screen.parents[&screen.find("Item 1").unwrap()];
    let in_first = screen.parents.values().filter(|&&p| p == first).count();
    assert_eq!(in_first, 2, "the row and a separator");
    press_text(&engine, &rx, &mut screen, "Clear");
    assert!(
        shows(&screen, "An empty list (0 items)"),
        "{:?}",
        screen.texts
    );
    assert!(shows(&screen, "Nothing here yet: tap Add"));
}
