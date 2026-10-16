//! `Sheet`: a panel that slides up from the bottom over a dimmed backdrop,
//! dragged down to close. The drag and the springs run natively: the panel
//! follows the finger with no render per frame.

use nsh_core::ui::prelude::*;

/// A bottom sheet ([`SheetProps`]).
#[derive(Debug, Default)]
pub struct Sheet {
    /// How far down the panel is from where it rests.
    offset: Option<NativeValue>,
    /// Closing: the panel slides away, then the app is told.
    closing: Option<Task<()>>,
}

/// A panel that slides up from the bottom over a dimmed backdrop while
/// `open`; dragging it down, tapping the backdrop or the back button closes
/// it (`onClose`: set `open` to false).
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SheetProps {
    /// Shown.
    #[prop(default)]
    pub open: bool,
    /// The content.
    pub children: Children,
}

/// The events of a [`Sheet`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SheetEvent {
    /// The user closed it (it slid away).
    Close,
}

/// A drag further down than this, or faster, closes it.
const CLOSE_DISTANCE: f32 = 120.0;
const CLOSE_VELOCITY: f32 = 900.0;

impl Sheet {
    fn offset(&self) -> &NativeValue {
        self.offset.as_ref().expect("set up")
    }

    /// Slides in from below the screen.
    fn open(&mut self, cx: &mut Context<'_, Self>) {
        self.closing = None;
        let below = cx.environment().window.height;
        self.offset().set(below);
        self.offset().spring(
            0.0,
            Spring {
                stiffness: 280.0,
                damping: 30.0,
                ..Spring::default()
            },
        );
    }

    /// Slides away, then tells the app.
    fn close(&mut self, cx: &mut Context<'_, Self>) {
        if self.closing.is_some() {
            return;
        }
        let below = cx.environment().window.height;
        let end = self.offset().timing(below, 220.0, Easing::EaseIn);
        self.closing = Some(cx.spawn(|this, _| async move {
            // Ended or interrupted: closed either way.
            let _ = end.await;
            this.update(|this, cx| {
                this.closing = None;
                cx.emit(SheetEvent::Close);
            })
            .await;
        }));
    }
}

impl Component for Sheet {
    type Props = SheetProps;
    type Event = SheetEvent;

    fn setup(&mut self, props: &SheetProps, cx: &mut Context<'_, Self>) {
        self.offset = Some(cx.native_value(0.0));
        if props.open {
            self.open(cx);
        }
    }

    fn props_changed(&mut self, old: &SheetProps, new: &SheetProps, cx: &mut Context<'_, Self>) {
        if new.open && !old.open {
            self.open(cx);
        }
    }

    fn render(&self, props: &SheetProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        let offset = self.offset();
        let height = cx.window().height;
        let bottom = cx.safe_area().bottom;
        view().children(props.open.then(|| {
            modal()
                .transparent(true)
                .animation(ModalAnimation::None)
                .on_request_close(cx.listener(|this: &mut Self, _, cx| this.close(cx)))
                .child(
                    // The backdrop fades with the panel's position.
                    view()
                        .absolute()
                        .inset_0()
                        .bg(theme.overlay)
                        .bind_opacity(&offset.interpolate(&[0.0, height], &[1.0, 0.0]))
                        .label("Close")
                        .on_press(cx.listener(|this: &mut Self, _, cx| this.close(cx))),
                )
                .child(
                    view()
                        .absolute()
                        .left_0()
                        .right_0()
                        .bottom_0()
                        .pt_2()
                        .pb(px(bottom + 16.0))
                        .px_4()
                        .gap_3()
                        .rounded_t(px(theme.radius_lg.max(16.0)))
                        .bg(theme.background)
                        .shadow_lg()
                        .bind_translate_y(offset)
                        .pan_values(None, Some(offset))
                        .on_pan_end(cx.listener(|this: &mut Self, e: &PanEvent, cx| {
                            if e.translation_y > CLOSE_DISTANCE || e.velocity_y > CLOSE_VELOCITY {
                                this.close(cx);
                            } else {
                                this.offset().spring(
                                    0.0,
                                    Spring {
                                        stiffness: 280.0,
                                        damping: 30.0,
                                        velocity: Some(e.velocity_y),
                                        ..Spring::default()
                                    },
                                );
                            }
                        }))
                        // The grabber.
                        .child(
                            view()
                                .w(px(36.0))
                                .h(px(5.0))
                                .self_center()
                                .rounded_full()
                                .bg(theme.muted),
                        )
                        .child(props.children),
                )
        }))
    }
}
