//! The Text demo on a headless engine: its sentences are one paragraph
//! each (spans, links, inline views), links and the button inside a
//! sentence take presses.

use nsh_core::{EventKind, TextDecoration};

use crate::common::{press_link, shows};

/// The sentence with links.
const SENTENCE: &str = "By continuing you agree to the Terms of service and the Privacy policy, which mention nested links too.";

#[test]
fn text_renders_its_samples() {
    let (_engine, _rx, screen) = crate::common::open("Text");
    for text in [
        SENTENCE,
        "Pressed: Tap a link in the sentence",
        "LIVE",
        "♥ 3",
        "Custom fonts",
        "Light Bold Italic Both Spaced",
        "Underline Strikethrough Both",
        "UPPERCASE Capitalized Words Here",
        "Text shadow",
        "System text size (fontScale 1.00)",
        "numberOfLines=1 head / middle / clip",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }
    // The image, the badge and the button wrap with the words.
    let inline = screen
        .block(
            "Made with \u{fffc} nsh. Status \u{fffc} and a button inside the sentence \u{fffc} that takes presses of its own.",
        )
        .expect("one paragraph");
    assert_eq!(inline.attachments.len(), 3);
    let links = &screen.block(SENTENCE).expect("one paragraph").links;
    assert_eq!(links.len(), 3, "{links:?}");
    // "Both" is underlined and struck through; the strikethrough is red.
    let decorations = screen.block("Underline Strikethrough Both").unwrap();
    let both = decorations.spans.last().unwrap();
    assert_eq!(
        both.style.decoration,
        TextDecoration {
            underline: true,
            line_through: true
        }
    );
    let struck = &decorations.spans[2];
    assert_eq!(
        struck.style.decoration_color,
        Some(nsh_core::Color(0xcf22_2eff))
    );
    let shadow = screen
        .block("Text shadow")
        .unwrap()
        .shadow
        .expect("a shadow");
    assert_eq!((shadow.radius, shadow.offset_x), (4.0, 2.0));
}

#[test]
fn links_and_the_inline_button_take_presses() {
    let (engine, rx, mut screen) = crate::common::open("Text");
    press_link(
        &engine,
        &rx,
        &mut screen,
        "Privacy policy",
        EventKind::Press,
    );
    assert!(
        shows(&screen, "Pressed: Privacy policy"),
        "{:?}",
        screen.texts
    );
    press_link(&engine, &rx, &mut screen, "links", EventKind::Press);
    assert!(
        shows(&screen, "Pressed: a link inside bold text"),
        "{:?}",
        screen.texts
    );
    press_link(
        &engine,
        &rx,
        &mut screen,
        "Terms of service",
        EventKind::LongPress,
    );
    assert!(
        shows(&screen, "Pressed: Terms (long press)"),
        "{:?}",
        screen.texts
    );
    // The sentence around the links.
    crate::common::press_text(&engine, &rx, &mut screen, SENTENCE);
    assert!(
        shows(&screen, "Pressed: the sentence (not a link)"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "♥ 3");
    assert!(shows(&screen, "♥ 4"), "{:?}", screen.texts);
}
