//! Photos: a long recycled list of rows with images, pull to refresh,
//! loading more at the end, a hero that fades and slides under the list
//! and a compact bar that fades in, all driven natively by the list's
//! scroll offset (as the TSX gallery's `demos/Photos.tsx`).

use std::{rc::Rc, time::Duration};

use nsh_core::ui::prelude::*;

use crate::ui::{Badge, Finish, Finished, IconName, PressProps, Stat, Tone, colors, icon, radius};

/// Photos loaded per page.
const PAGE: usize = 200;

/// A photo of the list.
#[derive(Debug, Clone)]
struct Photo {
    id: usize,
    title: String,
    author: &'static str,
    tone: Tone,
    views: usize,
}

/// `count` photos, numbered from `from`.
fn make_photos(from: usize, count: usize) -> impl Iterator<Item = Photo> {
    const AUTHORS: [&str; 5] = ["Ada", "Linus", "Grace", "Ken", "Barbara"];
    const TONES: [Tone; 5] = [Tone::Accent, Tone::Info, Tone::Warn, Tone::Up, Tone::Danger];
    const PLACES: [&str; 9] = [
        "Harbor", "Ridge", "Old town", "Dunes", "Canal", "Orchard", "Quarry", "Pier", "Meadow",
    ];
    (from..from + count).map(|id| Photo {
        id,
        title: format!("{} {id:04}", PLACES[id % PLACES.len()]),
        author: AUTHORS[id % AUTHORS.len()],
        tone: TONES[id % TONES.len()],
        views: (id * 7919) % 9800 + 120,
    })
}

/// 1234 → "1,234".
fn grouped(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// 1234 → "1.2k".
fn compact(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

/// One row: the picture, its title, author and views, its number.
fn row(photo: &Photo) -> impl IntoElement + use<> {
    crate::ui::Press::element(PressProps {
        scale: 0.985,
        label: Some(photo.title.clone().into()),
        ..PressProps::default()
    })
    .child(
        view()
            .flex_row()
            .items_center()
            .gap_3()
            .px_4()
            .h(px(76.0))
            .child(
                image(format!("https://picsum.photos/seed/{}/112/112", photo.id))
                    .size(px(56.0))
                    .rounded(px(radius::INSET))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(colors::BORDER)
                    .bg(colors::RAISED)
                    .finish(Finish::Raised),
            )
            .child(
                view()
                    .flex_1()
                    .gap_1()
                    .self_stretch()
                    .justify_center()
                    .border_b_1()
                    .border_b_color(colors::LINE)
                    .child(
                        text(photo.title.clone())
                            .number_of_lines(1)
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap(px(6.0))
                            .child(view().size(px(6.0)).rounded(px(3.0)).bg(photo.tone.color()))
                            .child(
                                text(photo.author)
                                    .text_size(px(13.0))
                                    .text_color(colors::MUTED),
                            )
                            .child(
                                text(format!("· {} views", compact(photo.views)))
                                    .text_size(px(13.0))
                                    .text_color(colors::FAINT)
                                    .tabular_nums(),
                            ),
                    ),
            )
            .child(
                text(format!("#{}", photo.id))
                    .text_size(px(13.0))
                    .text_color(colors::FAINT)
                    .tabular_nums(),
            ),
    )
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Photos {
    photos: Rc<Vec<Photo>>,
    refreshing: bool,
    refreshes: u32,
    /// The refresh in progress (dropping it cancels it).
    refresh: Option<Task<()>>,
    /// The list's scroll offset, fed natively: the hero's parallax and
    /// fade and the compact bar run without a render per frame.
    scroll_y: Option<NativeValue>,
}

impl Photos {
    /// The list's header: a card of figures and a hint, fading out and
    /// moving at half speed as the list slides over it.
    fn hero(&self, scroll_y: &NativeValue) -> impl IntoElement + use<> {
        let count = self.photos.len();
        view()
            .px_4()
            .pt_2()
            .pb_4()
            .bind_opacity(&scroll_y.interpolate(&[0.0, 150.0], &[1.0, 0.0]))
            .bind_translate_y(&scroll_y.interpolate(&[-200.0, 0.0, 300.0], &[0.0, 0.0, 150.0]))
            .child(
                view()
                    .bg(colors::SURFACE)
                    .rounded(px(radius::CARD))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(colors::BORDER)
                    .p_4()
                    .gap_3()
                    .finish(Finish::Card)
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .gap_3()
                            .child(
                                view()
                                    .size(px(40.0))
                                    .rounded(px(12.0))
                                    .border_curve(BorderCurve::Continuous)
                                    .bg(colors::RAISED)
                                    .border_1()
                                    .border_color(colors::BORDER)
                                    .items_center()
                                    .justify_center()
                                    .finish(Finish::Raised)
                                    .child(image(crate::assets::LOGO).size(px(24.0))),
                            )
                            .child(
                                view()
                                    .flex_1()
                                    .gap(px(2.0))
                                    .child(
                                        text("Photo stream")
                                            .text_size(px(17.0))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(colors::TEXT),
                                    )
                                    .child(
                                        text("A recycled list of remote images")
                                            .text_size(px(13.0))
                                            .text_color(colors::MUTED),
                                    ),
                            )
                            .child(Badge::new("Remote").tone(Tone::Info)),
                    )
                    .child(
                        text(grouped(count))
                            .text_size(px(40.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT)
                            .tabular_nums()
                            .letter_spacing(-1.0)
                            .child(
                                text(" photos")
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(colors::ACCENT),
                            ),
                    )
                    .child(
                        view()
                            .flex_row()
                            .border_t_1()
                            .border_t_color(colors::LINE)
                            .pt(px(10.0))
                            .child(Stat::new("Pages", count.div_ceil(PAGE).to_string()).width(0.33))
                            .child(Stat::new("Per page", PAGE.to_string()).width(0.33))
                            .child({
                                let stat =
                                    Stat::new("Refreshes", self.refreshes.to_string()).width(0.33);
                                if self.refreshes > 0 {
                                    stat.tone(Tone::Accent)
                                } else {
                                    stat
                                }
                            }),
                    ),
            )
            .child(
                view()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .pt_3()
                    .child(icon(IconName::ArrowDown, 14.0, colors::FAINT))
                    .child(
                        text(format!(
                            "Pull to refresh. Scrolling to the end loads {PAGE} more."
                        ))
                        .flex_1()
                        .text_size(px(13.0))
                        .text_color(colors::FAINT),
                    ),
            )
    }
}

impl Component for Photos {
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
                this.refreshes += 1;
                this.refreshing = false;
                cx.notify();
            }));
        });
        let load_more = cx.listener(|this: &mut Self, _, cx| {
            let from = this.photos.len() + 1;
            log::info!("loading more after {}", this.photos.len());
            Rc::make_mut(&mut this.photos).extend(make_photos(from, PAGE));
            cx.notify();
        });
        view()
            .flex_1()
            .bg(colors::BACKGROUND)
            .child(
                virtual_list(photos.len(), move |at| row(&photos[at.index()]))
                    .estimated_item_size(76.0)
                    .flex_1()
                    .content_padding_bottom(24.0)
                    .scroll_value(scroll_y)
                    .refreshing(self.refreshing)
                    .on_refresh(refresh)
                    .on_end_reached(load_more)
                    .header(self.hero(scroll_y))
                    .footer(
                        text(format!("Loading {PAGE} more…"))
                            .text_center()
                            .text_size(px(13.0))
                            .text_color(colors::FAINT)
                            .p_4(),
                    ),
            )
            // The compact bar fades in once the hero has scrolled away.
            .child(
                view()
                    .pointer_events(PointerEvents::None)
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(44.0))
                    .px_4()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .bg(colors::BACKGROUND)
                    .border_b_1()
                    .border_b_color(colors::BORDER)
                    .bind_opacity(&scroll_y.interpolate(&[140.0, 190.0], &[0.0, 1.0]))
                    .bind_translate_y(&scroll_y.interpolate(&[140.0, 190.0], &[-8.0, 0.0]))
                    .child(
                        text("Photo stream")
                            .flex_1()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    )
                    .child(
                        text(format!("{} photos", grouped(self.photos.len())))
                            .text_size(px(13.0))
                            .text_color(colors::MUTED)
                            .tabular_nums(),
                    ),
            )
    }
}
