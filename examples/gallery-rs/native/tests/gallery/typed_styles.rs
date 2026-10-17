//! The Typed styles demo: the Tailwind demo's cards in typed styles, its
//! buttons' state, and the effects the typed styles send natively.

use std::collections::HashMap;

use nsh_core::{EventKind, EventPayload, MountItem, ViewStyle};

use crate::common::shows;

#[test]
fn typed_styles_shows_every_card() {
    let (_engine, _rx, screen) = crate::common::open("Typed styles");
    for text in [
        "BUTTONS: ACTIVE, HOVER, TRANSITION, DISABLED",
        "CARDS: SHADOW, RING, GROUP, GRADIENT",
        "FLEX, GRID, BREAKPOINTS",
        "PALETTE",
        "SHADOWS, RINGS, OUTLINES",
        "GRADIENTS",
        "TYPOGRAPHY",
        "FILTERS, BACKDROP BLUR, BLEND MODES",
        "ANIMATIONS AND TRANSFORMS",
        "FIRST:, EVEN:, LAST:",
        "INPUTS",
        "Group press",
        "Gradient card",
        "col-span-2",
        "emerald",
        "2xl",
        "Display",
        "CAPS",
        "blur-sm",
        "bounce",
        "Archive",
    ] {
        assert!(shows(&screen, text), "{text}: {:?}", screen.texts);
    }
}

#[test]
fn typed_styles_buttons_keep_their_state() {
    let (engine, rx, mut screen) = crate::common::open("Typed styles");
    crate::common::press(&engine, &rx, &mut screen, "Pressed 0×");
    crate::common::press(&engine, &rx, &mut screen, "Pressed 1×");
    assert!(shows(&screen, "Pressed 2×"), "{:?}", screen.texts);
    crate::common::press(&engine, &rx, &mut screen, "Reset");
    assert!(shows(&screen, "Pressed 0×"), "{:?}", screen.texts);

    crate::common::press(&engine, &rx, &mut screen, "Enabled");
    assert!(shows(&screen, "Disabled"), "{:?}", screen.texts);
    crate::common::press(&engine, &rx, &mut screen, "Enable again");
    assert!(shows(&screen, "Enabled"), "{:?}", screen.texts);
}

#[test]
fn typed_styles_send_gradients_filters_and_animations_natively() {
    let (engine, rx) = crate::common::start();
    let mut screen = crate::common::Screen::default();
    crate::common::drain(&rx, &mut screen);
    let text = screen.find("Typed styles").expect("the demo in the index");
    engine.control().dispatch_event(
        screen.press_target(text),
        EventKind::Press,
        EventPayload::None,
    );
    let mut styles: HashMap<u32, ViewStyle> = HashMap::new();
    let mut tracked = 0;
    while let Ok(batch) = rx.recv_timeout(std::time::Duration::from_millis(300)) {
        for item in &batch.items {
            match item {
                MountItem::ViewStyle { id, style } => {
                    styles.insert(*id, (**style).clone());
                }
                MountItem::TrackStates { .. } => tracked += 1,
                _ => {}
            }
        }
    }
    let count = |f: &dyn Fn(&ViewStyle) -> bool| styles.values().filter(|s| f(s)).count();
    let effects = |s: &ViewStyle| s.effects.clone().unwrap_or_default();
    // The four gradients and the gradient card: no canvas, no layout.
    assert_eq!(count(&|s| !effects(s).background_images.is_empty()), 5);
    assert_eq!(count(&|s| !effects(s).backdrop_filters.is_empty()), 1);
    assert_eq!(
        count(&|s| s.animation.is_some()),
        4,
        "spin, ping, pulse, bounce"
    );
    assert_eq!(count(&|s| s.blend_mode == nsh_core::BlendMode::Multiply), 1);
    assert!(count(&|s| s.transition.is_some()) >= 4);
    // Rings and inset shadows are box shadows; outlines their own.
    assert!(count(&|s| effects(s).box_shadows.iter().any(|b| b.inset)) >= 1);
    assert_eq!(count(&|s| s.outline.width == 2.0), 1);
    assert!(tracked > 0, "press states are reported natively");
}
