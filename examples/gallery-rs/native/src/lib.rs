//! The Rust gallery (`examples/gallery-rs`): the app's TSX mounts one
//! component of this crate, [`Gallery`], and everything it shows is written
//! in Rust: the same demos as the TSX gallery (`examples/gallery`), on
//! nsh's Rust component API. The app imports the set as `gallery/native`
//! (`nsh.json` `"rust"`).

mod assets;
mod demos;
mod nav;
mod ui;

use nsh_core::ui::prelude::*;

use crate::nav::Routes;

/// The UI set of the gallery: what `App.tsx` imports from `gallery/native`.
#[native_module("gallery/native")]
fn native(registry: &mut Registry) {
    registry.register::<Gallery>();
    registry.assets(assets!());
}

/// The gallery: a native stack of its index and the screens pushed on it.
#[derive(Default)]
pub struct Gallery {
    _back: Option<BackSubscription>,
}

impl Component for Gallery {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        // The Android back button comes here before the stack pops; the
        // Navigation demo switches whether the gallery takes it.
        self._back = Some(cx.on_back_press(|_: &mut Self, cx| {
            cx.update_global::<demos::BackIntercept, _>(|intercept, _| {
                if intercept.on {
                    intercept.count += 1;
                }
                intercept.on
            })
        }));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let routes = cx.global::<Routes>().screens();
        let index = demos::index(cx);
        let pushed: Vec<_> = routes
            .into_iter()
            .enumerate()
            .map(|(depth, route)| {
                (route.screen)(cx)
                    .key(route.key)
                    // The user popped it natively: it and what is above it go.
                    .on_dismiss(cx.listener(move |_: &mut Self, _, cx| {
                        cx.update_global::<Routes, _>(|routes, _| routes.truncate(depth));
                    }))
            })
            .collect();
        view()
            .flex_1()
            .child(stack().flex_1().child(index).children(pushed))
    }
}
