//! The gallery on a headless engine: its index, and the screens pushed.

#[test]
fn every_demo_opens_from_the_index() {
    for name in [
        "Stocks",
        "AI Chat",
        "Photos",
        "Lists",
        "Scrolling",
        "Navigation",
        "Form",
        "Gestures",
        "Physics",
        "Animations",
        "Canvas",
        "Styles",
        "Typed styles",
        "Text",
        "Images",
        "Mobile UI",
        "Components",
        "Web and video",
        "System",
    ] {
        // `open` checks that the demo's screen is pushed with its title.
        let (_engine, _rx, screen) = crate::common::open(name);
        if name == "Stocks" {
            assert!(
                crate::common::shows(&screen, "Composite"),
                "{name}: {:?}",
                screen.texts
            );
        }
        if name == "AI Chat" {
            assert!(
                crate::common::shows(&screen, "How can I help?"),
                "{name}: {:?}",
                screen.texts
            );
        }
    }
}

#[test]
fn the_index_groups_the_demos() {
    let (_engine, rx) = crate::common::start();
    let mut screen = crate::common::Screen::default();
    crate::common::drain(&rx, &mut screen);
    for (title, count) in [
        ("APPS", "3"),
        ("LAYOUT AND LISTS", "4"),
        ("TOUCH AND MOTION", "3"),
        ("DRAWING AND TYPE", "5"),
        ("PLATFORM", "4"),
    ] {
        let at = screen.find(title).unwrap_or_else(|| panic!("no {title}"));
        // The count is the title's sibling.
        let row = screen.parents[&at];
        assert!(
            screen
                .texts
                .iter()
                .any(|(id, t)| t == count && screen.parents.get(id) == Some(&row)),
            "{title}: {count}"
        );
    }
    // Each row has its icon and its chevron drawn.
    assert!(screen.drawings.len() >= 2 * 19, "{}", screen.drawings.len());
}

#[test]
fn navigation_pushes_its_screens() {
    let (engine, rx, mut screen) = crate::common::open("Navigation");
    for label in [
        "Native stack",
        "Top",
        "Bottom",
        "Screens",
        "ios.headerLargeTitle",
        "Modals",
        "See-through",
        "Back button",
        "Off: the back button pops this screen.",
    ] {
        assert!(
            crate::common::shows(&screen, label),
            "{label}: {:?}",
            screen.texts
        );
    }
    for (label, title) in [
        ("Large title", "Large title"),
        ("Custom header items", "Custom"),
        ("Unsaved changes", "Unsaved changes"),
        ("A deep stack", "Level 1"),
        ("Deeper", "Level 2"),
    ] {
        crate::common::press(&engine, &rx, &mut screen, label);
        assert!(
            screen.titles.iter().any(|t| t.1 == title),
            "{label}: {:?}",
            screen.titles
        );
    }
    assert!(
        crate::common::shows(&screen, "2 deep in the stack"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "To the index");
    assert!(
        !screen.titles.iter().any(|t| t.1 == "Navigation"),
        "{:?}",
        screen.titles
    );
}

#[test]
fn navigation_presents_its_modals() {
    let (engine, rx, mut screen) = crate::common::open("Navigation");
    crate::common::press(&engine, &rx, &mut screen, "Page sheet");
    for label in [
        "pageSheet",
        "Close with the button, the back button (Android) or a swipe down (iOS sheets).",
        "the modal's",
    ] {
        assert!(
            crate::common::shows(&screen, label),
            "{label}: {:?}",
            screen.texts
        );
    }
    crate::common::press(&engine, &rx, &mut screen, "Close");
    assert!(
        !crate::common::shows(&screen, "pageSheet"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "Dialog");
    assert!(
        crate::common::shows(&screen, "A dialog"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "OK");
    assert!(
        !crate::common::shows(&screen, "A dialog"),
        "{:?}",
        screen.texts
    );
}
