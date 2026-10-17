//! The Styles demo on a headless engine: its cards render with their
//! shadows, corners, borders and transforms; the clipping card toggles;
//! "Flip" lays the surface out right to left and back.

use std::collections::HashMap;

use nsh_core::{BorderStyle, EventKind, EventPayload, MountItem, TransformKind, ViewStyle};

#[test]
fn styles_render_their_cards() {
    let (_engine, _rx, screen) = crate::common::open("Styles");
    for text in [
        "Shadow",
        "Top corners",
        "Mixed radii",
        "Clipped (tap)",
        "Per-side borders",
        "zIndex (tap)",
        "rotateY + perspective",
        "skewX (iOS)",
        "Front (tap)",
        "Back: backfaceVisibility",
        "Left to right",
        "12",
    ] {
        assert!(
            screen.texts.iter().any(|t| t.1 == text),
            "{text}: {:?}",
            screen.texts
        );
    }
}

#[test]
fn styles_send_the_cards_styles() {
    let (engine, rx) = crate::common::start();
    let mut screen = crate::common::Screen::default();
    crate::common::drain(&rx, &mut screen);
    let text = screen.find("Styles").expect("the demo in the index");
    engine.control().dispatch_event(
        screen.press_target(text),
        EventKind::Press,
        EventPayload::None,
    );
    let mut styles: HashMap<u32, ViewStyle> = HashMap::new();
    while let Ok(batch) = rx.recv_timeout(std::time::Duration::from_millis(300)) {
        for item in &batch.items {
            if let MountItem::ViewStyle { id, style } = item {
                styles.insert(*id, (**style).clone());
            }
        }
    }
    let any = |f: &dyn Fn(&ViewStyle) -> bool| styles.values().any(f);
    // "Mixed radii": a blue shadow and three corners of their own.
    assert!(any(&|s| s.shadow.opacity == 0.35
        && s.shadow.radius == 10.0
        && (s.shadow.offset_x, s.shadow.offset_y) == (4.0, 8.0)
        && s.corner_radius[..3] == [Some(44.0), Some(6.0), Some(28.0)]));
    // Per-side borders.
    assert!(any(&|s| s.borders.widths == [6.0, 2.0, 4.0, 1.0]));
    assert!(any(&|s| s.borders.style == BorderStyle::Dashed));
    assert!(any(&|s| s.borders.style == BorderStyle::Dotted));
    assert!(any(&|s| s.continuous_corners));
    assert!(any(&|s| s.backface_hidden));
    let has =
        |s: &ViewStyle, kind: TransformKind| s.transform.ops().iter().any(|op| op.kind == kind);
    assert!(any(
        &|s| has(s, TransformKind::Perspective) && has(s, TransformKind::RotateY)
    ));
    assert!(any(&|s| has(s, TransformKind::SkewX)));
    assert!(any(
        &|s| s.transform_origin.x == nsh_core::OriginLen::Fraction(0.0)
    ));
}

#[test]
fn the_clipping_card_toggles() {
    let (engine, rx, mut screen) = crate::common::open("Styles");
    crate::common::press(&engine, &rx, &mut screen, "Clipped (tap)");
    assert!(
        screen.texts.iter().any(|t| t.1 == "Visible (tap)"),
        "{:?}",
        screen.texts
    );
    crate::common::press(&engine, &rx, &mut screen, "Visible (tap)");
    assert!(
        screen.texts.iter().any(|t| t.1 == "Clipped (tap)"),
        "{:?}",
        screen.texts
    );
}

#[test]
fn flip_lays_the_surface_out_right_to_left() {
    let (engine, rx, mut screen) = crate::common::open("Styles");
    let flip = screen.find("Flip").expect("the Flip button");
    let mut directions = Vec::new();
    let mut press = |screen: &mut crate::common::Screen| {
        engine
            .control()
            .dispatch_event(flip, EventKind::Press, EventPayload::None);
        while let Ok(batch) = rx.recv_timeout(std::time::Duration::from_millis(300)) {
            screen.apply(&batch);
            for item in &batch.items {
                if let MountItem::LayoutDirection { rtl } = item {
                    directions.push(*rtl);
                }
            }
        }
    };
    press(&mut screen);
    assert!(
        screen.texts.iter().any(|t| t.1 == "Right to left"),
        "{:?}",
        screen.texts
    );
    press(&mut screen);
    assert!(
        screen.texts.iter().any(|t| t.1 == "Left to right"),
        "{:?}",
        screen.texts
    );
    assert_eq!(directions, [true, false]);
}
