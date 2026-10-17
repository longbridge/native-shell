//! Images: resize modes, corners, tint, GIF / WebP, placeholders and fades,
//! blur, sizes and prefetching, load events (as the TSX gallery's
//! `demos/Images.tsx`).

use nsh_core::{
    Size,
    ui::{AnyElement, Image, ImageProgress, ImageSize, prelude::*},
};

use crate::{
    assets::{BOUNCE, LOGO, SPINNER},
    ui::{
        Badge, Button, ButtonVariant, Card, Finish, Finished, IconName, MONO, Meter, Tone, appear,
        caption, colors, radius,
    },
};

/// A fixed photo (in color), large enough to stay sharp at 3x.
const WIDE: &str = "https://picsum.photos/id/1015/1200/600";

/// Photo `id` at `w` × `h` pixels.
fn photo(id: u32, w: u32, h: u32) -> String {
    format!("https://picsum.photos/id/{id}/{w}/{h}")
}

/// A large photo (`image_size`, `prefetch_image`, progress).
fn big() -> String {
    photo(1018, 2400, 1600)
}

/// Every photo's frame: rounded, a hairline border, a raised fill while it
/// loads.
fn frame(image: Image) -> Image {
    image
        .rounded(px(radius::CONTROL))
        .border_curve(BorderCurve::Continuous)
        .border_1()
        .border_color(colors::BORDER)
        .bg(colors::RAISED)
}

/// A quiet label under a tile (`code` for API values).
fn note(content: impl Into<SharedString>, code: bool) -> AnyElement {
    text(content.into())
        .number_of_lines(1)
        .text_size(px(13.0))
        .text_color(colors::MUTED)
        .when(code, |t| t.font_family(MONO))
        .tabular_nums()
        .text_center()
        .into_element()
}

/// A tile on the inset: something centered on a raised square, a note
/// below.
fn tile(note_text: &str, code: bool, height: f32, content: impl IntoElement) -> AnyElement {
    view()
        .flex_1()
        .gap_2()
        .child(
            view()
                .h(px(height))
                .rounded(px(radius::CONTROL))
                .border_curve(BorderCurve::Continuous)
                .bg(colors::RAISED)
                .finish(Finish::Raised)
                .border_1()
                .border_color(colors::BORDER)
                .items_center()
                .justify_center()
                .child(content),
        )
        .child(note(note_text.to_owned(), code))
        .into_element()
}

/// The demo's page.
#[derive(Debug)]
pub(crate) struct Images {
    /// The load events of the first image of "Events".
    events: Vec<&'static str>,
    /// What the first image of "Events" reported.
    loaded: String,
    /// What the image that cannot load reported.
    failed: String,
    /// Asks for images that are not cached yet ("Reload").
    seed: u32,
    /// The large image shows.
    show: bool,
    /// What `image_size` answered for the large image.
    size: String,
    /// What `prefetch_image` answered.
    prefetched: String,
    /// The large image's download progress, 0 to 1.
    progress: f64,
    /// The large image's load.
    status: &'static str,
    /// Waits for `image_size`'s answer.
    sizing: Option<Task<()>>,
    /// Waits for `prefetch_image`'s answer.
    prefetching: Option<Task<()>>,
}

impl Default for Images {
    fn default() -> Self {
        Self {
            events: Vec::new(),
            loaded: "…".to_owned(),
            failed: "Loading…".to_owned(),
            seed: 1,
            show: false,
            size: "—".to_owned(),
            prefetched: "—".to_owned(),
            progress: 0.0,
            status: "not shown",
            sizing: None,
            prefetching: None,
        }
    }
}

impl Images {
    /// A 2:1 image in squares, one per resize mode.
    fn resize_modes() -> AnyElement {
        let rows = [
            [
                (ResizeMode::Cover, "cover"),
                (ResizeMode::Contain, "contain"),
            ],
            [
                (ResizeMode::Stretch, "stretch"),
                (ResizeMode::Center, "center"),
            ],
        ]
        .map(|row| {
            view().flex_row().gap_3().children(row.map(|(mode, name)| {
                view()
                    .flex_1()
                    .gap_2()
                    .child(frame(image(WIDE).resize_mode(mode)).h(px(140.0)))
                    .child(note(name, true))
            }))
        });
        Card::new()
            .icon(IconName::Image)
            .title("Resize modes")
            .subtitle("A 2:1 image in square frames")
            .children(rows)
            .into_element()
    }

    fn corners() -> AnyElement {
        let square = || frame(image(WIDE)).size(px(88.0));
        let item = |image: Image, label: &'static str| {
            view()
                .items_center()
                .gap_2()
                .child(image)
                .child(note(label, false))
        };
        Card::new()
            .icon(IconName::Styles)
            .title("Corners and borders")
            .subtitle("Radii per corner, borders on the image")
            .child(
                view()
                    .flex_row()
                    .justify_between()
                    .child(item(square().rounded(px(44.0)), "Circle"))
                    .child(item(
                        square()
                            .rounded(px(18.0))
                            .border_2()
                            .border_color(colors::ACCENT),
                        "Border",
                    ))
                    .child(item(
                        square()
                            .rounded(px(0.0))
                            .rounded_tl(px(40.0))
                            .rounded_br(px(40.0)),
                        "Two corners",
                    )),
            )
            .into_element()
    }

    fn assets() -> AnyElement {
        // At its natural size (64 × 64 points: `@2x` of 128 pixels).
        let tints = [
            ("Accent", Tone::Accent),
            ("Info", Tone::Info),
            ("Danger", Tone::Danger),
        ]
        .map(|(label, tone)| tile(label, false, 76.0, image(LOGO).tint_color(tone.color())));
        Card::new()
            .icon(IconName::Star)
            .title("Assets and tint")
            .subtitle("logo@2x.png, tinted natively")
            .child(
                view()
                    .flex_row()
                    .gap_3()
                    .child(tile("Original", false, 76.0, image(LOGO)))
                    .children(tints),
            )
            .into_element()
    }

    fn animated() -> AnyElement {
        Card::new()
            .icon(IconName::Animation)
            .title("Animated GIF and WebP")
            .subtitle("Decoded and played natively")
            .child(
                view()
                    .flex_row()
                    .gap_3()
                    .child(tile("bounce.gif", true, 96.0, image(BOUNCE)))
                    .child(tile("spinner.webp", true, 96.0, image(SPINNER)))
                    .child(tile(
                        "Scaled, round",
                        false,
                        96.0,
                        image(BOUNCE).size(px(48.0)).rounded(px(24.0)),
                    )),
            )
            .into_element()
    }

    /// Network images behind a placeholder, fading in; "Reload" asks for
    /// images that are not cached yet.
    fn placeholders(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        // A new size per reload: a URL not in the cache yet.
        let side = 600 + self.seed;
        let images = [(1043, 0u32), (1080, 300), (1069, 1000)].map(|(id, ms)| {
            view()
                .flex_1()
                .gap_2()
                .child(
                    frame(
                        image(photo(id, side, side))
                            .placeholder(LOGO)
                            .fade_duration(ms as f32)
                            .resize_mode(ResizeMode::Cover),
                    )
                    .h(px(100.0)),
                )
                .child(note(format!("{ms} ms"), false))
        });
        let reload = cx.listener(|this: &mut Self, _, cx| {
            this.seed += 1;
            cx.notify();
        });
        Card::new()
            .icon(IconName::Photos)
            .title("Placeholder and fade")
            .subtitle("defaultSource, then a fade")
            .right(Button::new("Reload", reload).variant(ButtonVariant::Secondary))
            .child(view().flex_row().gap_3().children(images))
            .child(caption("Loads", Some(self.seed.to_string())))
            .into_element()
    }

    fn blur() -> AnyElement {
        let images = [0u32, 2, 6, 16].map(|r| {
            view()
                .flex_1()
                .gap_2()
                .child(frame(image(WIDE).blur_radius(r as f32)).h(px(76.0)))
                .child(note(
                    if r == 0 {
                        "None".to_owned()
                    } else {
                        format!("{r} pt")
                    },
                    false,
                ))
        });
        Card::new()
            .icon(IconName::Spark)
            .title("Blur radius")
            .subtitle("blurRadius, applied while decoding")
            .child(view().flex_row().gap_2().children(images))
            .into_element()
    }

    fn info(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let get_size = cx.listener(|this: &mut Self, _, cx| {
            let size = cx.image_size(big());
            this.sizing = Some(cx.spawn(|this, _| async move {
                let size = size.await;
                this.update(move |this, cx| {
                    this.size = match size {
                        Ok(ImageSize { width, height }) => format!("{width}×{height} px"),
                        Err(error) => error,
                    };
                    cx.notify();
                })
                .await;
            }));
        });
        let prefetch = cx.listener(|this: &mut Self, _, cx| {
            this.prefetched = "loading…".to_owned();
            cx.notify();
            let loaded = cx.prefetch_image(big());
            this.prefetching = Some(cx.spawn(|this, _| async move {
                let loaded = loaded.await;
                this.update(move |this, cx| {
                    this.prefetched = if loaded { "in memory" } else { "failed" }.to_owned();
                    cx.notify();
                })
                .await;
            }));
        });
        let toggle = cx.listener(|this: &mut Self, _, cx| {
            this.show = !this.show;
            cx.notify();
        });
        let shown = if self.show {
            appear(0)
                .child(
                    frame(image(big()))
                        .h(px(170.0))
                        .on_load_start(cx.listener(|this: &mut Self, _, cx| {
                            this.status = "loading";
                            cx.notify();
                        }))
                        .on_progress(cx.listener(|this: &mut Self, e: &ImageProgress, cx| {
                            this.progress = e.loaded / e.total.max(1.0);
                            cx.notify();
                        }))
                        .on_load(cx.listener(|this: &mut Self, _: &Size, cx| {
                            this.status = if this.progress == 0.0 {
                                "loaded (no download)"
                            } else {
                                "loaded"
                            };
                            this.progress = 1.0;
                            cx.notify();
                        })),
                )
                .into_element()
        } else {
            note("Prefetch, then show: no download, no progress", false)
        };
        Card::new()
            .icon(IconName::Search)
            .title("Size, prefetch, progress")
            .subtitle("Image.getSize · Image.prefetch · onProgress")
            .child(
                view()
                    .flex_row()
                    .gap_2()
                    .child(
                        Button::new("Size", get_size)
                            .variant(ButtonVariant::Secondary)
                            .flex_1(),
                    )
                    .child(
                        Button::new("Prefetch", prefetch)
                            .variant(ButtonVariant::Secondary)
                            .flex_1(),
                    )
                    .child(
                        Button::new(if self.show { "Hide" } else { "Show" }, toggle)
                            .variant(if self.show {
                                ButtonVariant::Secondary
                            } else {
                                ButtonVariant::Accent
                            })
                            .flex_1(),
                    ),
            )
            .child(
                view()
                    .gap_2()
                    .child(caption("getSize", Some(self.size.clone())))
                    .child(caption("prefetch", Some(self.prefetched.clone())))
                    .child(caption(
                        "onProgress",
                        Some(format!(
                            "{}% · {}",
                            (self.progress * 100.0).round(),
                            self.status
                        )),
                    ))
                    .child(
                        Meter::new(self.progress as f32).tone(if self.progress >= 1.0 {
                            Tone::Up
                        } else {
                            Tone::Accent
                        }),
                    ),
            )
            .child(shown)
            .into_element()
    }

    fn events(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let log = |line: &'static str| {
            cx.listener(move |this: &mut Self, _: &(), cx| {
                this.events.push(line);
                cx.notify();
            })
        };
        let thumb = |image: Image| frame(image).w(px(96.0)).h(px(64.0));
        let loads = thumb(image(photo(1084, 600, 400)))
            .on_load_start(log("onLoadStart"))
            .on_load(cx.listener(|this: &mut Self, size: &Size, cx| {
                this.events.push("onLoad");
                this.loaded = format!("{}×{}", size.width, size.height);
                cx.notify();
            }))
            .on_error(cx.listener(|this: &mut Self, message: &str, cx| {
                this.loaded = message.to_owned();
                cx.notify();
            }))
            .on_load_end(log("onLoadEnd"));
        let fails = thumb(image("https://example.invalid/missing.png"))
            .on_error(cx.listener(|this: &mut Self, message: &str, cx| {
                this.failed = message.to_owned();
                cx.notify();
            }))
            .absolute()
            .left(px(-1.0))
            .top(px(-1.0))
            .border(0.0);
        let label = |host: &'static str, status: AnyElement| {
            view()
                .flex_1()
                .gap_1()
                .child(text(host).text_size(px(15.0)).text_color(colors::TEXT))
                .child(status)
        };
        let events = if self.events.is_empty() {
            vec![note("No load events yet", false)]
        } else {
            self.events
                .iter()
                .map(|&e| {
                    Badge::new(e)
                        .tone(if e == "onLoad" {
                            Tone::Up
                        } else {
                            Tone::Neutral
                        })
                        .into_element()
                })
                .collect()
        };
        Card::new()
            .icon(IconName::Bell)
            .title("Events")
            .subtitle("Load start, load, error, load end")
            .child(
                view()
                    .flex_row()
                    .gap_3()
                    .items_center()
                    .child(loads)
                    .child(label(
                        "picsum.photos",
                        text(format!("Loaded, {} px", self.loaded))
                            .text_size(px(13.0))
                            .text_color(colors::UP)
                            .tabular_nums()
                            .into_element(),
                    )),
            )
            .child(
                view()
                    .flex_row()
                    .gap_3()
                    .items_center()
                    .border_t_1()
                    .border_t_color(colors::LINE)
                    .pt_3()
                    .child(
                        view()
                            .w(px(96.0))
                            .h(px(64.0))
                            .rounded(px(radius::CONTROL))
                            .border_curve(BorderCurve::Continuous)
                            .border_1()
                            .border_color(colors::DANGER.opacity(0.4))
                            .bg(colors::RAISED)
                            .items_center()
                            .justify_center()
                            .child(fails)
                            .child(text("×").text_size(px(20.0)).text_color(colors::DANGER)),
                    )
                    .child(label(
                        "example.invalid",
                        text(self.failed.clone())
                            .number_of_lines(3)
                            .text_size(px(13.0))
                            .text_color(colors::DANGER)
                            .into_element(),
                    )),
            )
            .child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap_2()
                    .items_center()
                    .children(events),
            )
            .into_element()
    }
}

impl Component for Images {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let cards = [
            Self::resize_modes(),
            Self::corners(),
            Self::assets(),
            Self::animated(),
            self.placeholders(cx),
            Self::blur(),
            self.info(cx),
            self.events(cx),
        ];
        // A component's root is a view: the page scrolls inside it.
        view().flex_1().child(
            scroll().flex_1().bg(colors::BACKGROUND).child(
                view().p_4().pb(px(40.0)).gap_4().children(
                    cards
                        .into_iter()
                        .enumerate()
                        .map(|(i, card)| appear(i as u32).child(card)),
                ),
            ),
        )
    }
}
