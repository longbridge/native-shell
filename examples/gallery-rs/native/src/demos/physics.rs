//! Physics: a card dragged natively that springs back with the drag's
//! velocity, pulses when tapped and counts its releases (as the TSX
//! gallery's `demos/Physics.tsx`).

use nsh_core::ui::prelude::*;

/// The springs that bring the card back.
const BACK: Spring = Spring {
    stiffness: 180.0,
    damping: 14.0,
    mass: 1.0,
    velocity: None,
};

/// The demo's page: the card, its hint and how often it was released.
#[derive(Debug, Default)]
pub(crate) struct Physics {
    /// The drag's horizontal translation (the drag moves it natively).
    x: Option<NativeValue>,
    /// The drag's vertical translation.
    y: Option<NativeValue>,
    /// The pulse: 0 at rest, 1 at its peak.
    pressed: Option<NativeValue>,
    /// How often the card was let go.
    releases: u32,
    /// The pulse running.
    pulse: Option<Task<()>>,
}

impl Physics {
    fn values(&self) -> (&NativeValue, &NativeValue, &NativeValue) {
        match (&self.x, &self.y, &self.pressed) {
            (Some(x), Some(y), Some(pressed)) => (x, y, pressed),
            _ => unreachable!("set up"),
        }
    }
}

impl Component for Physics {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.x = Some(cx.native_value(0.0));
        self.y = Some(cx.native_value(0.0));
        self.pressed = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let (x, y, pressed) = self.values();
        let muted = rgb(0x6e7781);
        view()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_6()
            .child(
                text("Drag the card, tap it to pulse, long-press to reset")
                    .text_size(px(15.0))
                    .text_color(muted),
            )
            .child(
                view()
                    .size(px(160.0))
                    .rounded(px(28.0))
                    .items_center()
                    .justify_center()
                    .pan_values(Some(x), Some(y))
                    .bind_background(
                        &pressed.interpolate_colors(&[0.0, 1.0], &[rgb(0x8250df), rgb(0xbf3989)]),
                    )
                    .bind_translate_x(x)
                    .bind_translate_y(y)
                    .bind_scale(&pressed.interpolate(&[0.0, 1.0], &[1.0, 1.12]))
                    .bind_rotate(&x.interpolate(&[-200.0, 200.0], &[-12.0, 12.0]))
                    .on_pan_end(cx.listener(|this: &mut Self, e: &PanEvent, cx| {
                        this.releases += 1;
                        let (x, y, _) = this.values();
                        x.spring(
                            0.0,
                            Spring {
                                velocity: Some(e.velocity_x),
                                ..BACK
                            },
                        );
                        y.spring(
                            0.0,
                            Spring {
                                velocity: Some(e.velocity_y),
                                ..BACK
                            },
                        );
                        cx.notify();
                    }))
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        let pressed = this.values().2.clone();
                        this.pulse = Some(cx.spawn(|_, _| async move {
                            // Up, then down whether or not the way up was
                            // interrupted.
                            let _ = pressed.timing(1.0, 120.0, Easing::EaseOut).await;
                            let _ = pressed.timing(0.0, 260.0, Easing::EaseInOut).await;
                        }));
                    }))
                    .on_long_press(cx.listener(|this: &mut Self, _, cx| {
                        this.releases = 0;
                        cx.notify();
                    }))
                    .child(
                        text("Drag me")
                            .text_color(rgb(0xffffff))
                            .text_size(px(20.0))
                            .font_weight(FontWeight::BOLD),
                    ),
            )
            .child(
                text(format!("Released {} times", self.releases))
                    .text_size(px(13.0))
                    .text_color(muted),
            )
    }
}
