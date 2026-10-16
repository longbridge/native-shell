//! Photos: a long recycled list of rows with images, pull to refresh,
//! loading more at the end, and a header that fades and lifts as the list
//! scrolls, natively (as the TSX gallery's `demos/Photos.tsx`).
//!
//! Not ported: the header's logo (Rust components have no asset imports:
//! `logo@2x.png` is bundled only when JS imports it).

use std::{rc::Rc, time::Duration};

use nsh_core::ui::{View, prelude::*};

/// A photo of the list.
#[derive(Debug, Clone)]
struct Photo {
    id: usize,
    title: String,
    author: &'static str,
}

/// `count` photos, numbered from `from`.
fn make_photos(from: usize, count: usize) -> impl Iterator<Item = Photo> {
    const AUTHORS: [&str; 5] = ["Ada", "Linus", "Grace", "Ken", "Barbara"];
    (from..from + count).map(|id| Photo {
        id,
        title: format!("Photo #{id}"),
        author: AUTHORS[id % 5],
    })
}

/// One row: the picture, its title and author.
fn row(photo: &Photo) -> View {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .px_4()
        .py_2()
        .child(
            image(format!("https://picsum.photos/seed/{}/96/96", photo.id))
                .size(px(48.0))
                .rounded(px(8.0))
                .bg(rgb(0xe1e4e8)),
        )
        .child(
            view()
                .flex_1()
                .gap(px(2.0))
                .child(
                    text(photo.title.clone())
                        .text_size(px(16.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(0x1f2328)),
                )
                .child(
                    text(format!("by {}", photo.author))
                        .text_size(px(13.0))
                        .text_color(rgb(0x6e7781)),
                ),
        )
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Photos {
    photos: Rc<Vec<Photo>>,
    refreshing: bool,
    /// The refresh in progress (dropping it cancels it).
    refresh: Option<Task<()>>,
    /// The list's scroll offset, fed natively: the header fades without a
    /// render per frame.
    scroll_y: Option<NativeValue>,
}

impl Component for Photos {
    const NAME: &'static str = "gallery.photos";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.photos = Rc::new(make_photos(1, 1000).collect());
        self.scroll_y = Some(cx.native_value(0.0));
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let photos = self.photos.clone();
        let scroll_y = self.scroll_y.as_ref().expect("set up");
        let refresh = cx.listener(|this: &mut Self, _, cx| {
            this.refreshing = true;
            cx.notify();
            this.refresh = Some(cx.set_timeout(Duration::from_millis(800), |this, cx| {
                this.photos = Rc::new(make_photos(1, 1000).collect());
                this.refreshing = false;
                cx.notify();
            }));
        });
        let load_more = cx.listener(|this: &mut Self, _, cx| {
            let from = this.photos.len() + 1;
            Rc::make_mut(&mut this.photos).extend(make_photos(from, 200));
            cx.notify();
        });
        view()
            .flex_1()
            .child(
                view()
                    .px_4()
                    .py(px(10.0))
                    .flex_row()
                    .items_center()
                    .gap(px(10.0))
                    .bind_opacity(&scroll_y.interpolate(&[0.0, 120.0], &[1.0, 0.35]))
                    .bind_translate_y(&scroll_y.interpolate(&[0.0, 120.0], &[0.0, -6.0]))
                    .child(
                        text(format!("{} photos", self.photos.len()))
                            .text_size(px(22.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x1f2328)),
                    ),
            )
            .child(
                virtual_list(photos.len(), move |i, _| row(&photos[i]))
                    .estimated_item_size(64.0)
                    .flex_1()
                    .bg(rgb(0xffffff))
                    .scroll_value(scroll_y)
                    .refreshing(self.refreshing)
                    .on_refresh(refresh)
                    .on_end_reached(load_more),
            )
    }
}
