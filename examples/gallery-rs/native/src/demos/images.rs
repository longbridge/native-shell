//! Images: resize modes, tint, GIF / WebP, placeholders, blur, fade, events
//! (as the TSX gallery's `demos/Images.tsx`).

use nsh_core::{
    Size,
    ui::{AnyElement, Image, ImageProgress, ImageSize, Text, prelude::*},
};

use crate::{
    assets::{BOUNCE, LOGO, SPINNER},
    ui::{button, colors, page, section},
};

/// A 2:1 network image.
const WIDE: &str = "https://picsum.photos/seed/nsh-wide/480/240";
/// A large network image (`Image.getSize`, `Image.prefetch`).
const BIG: &str = "https://picsum.photos/seed/nsh-progress/2400/1600";

/// The background of images still loading.
fn loading() -> Color {
    rgb(0xeaeef2)
}

fn caption(content: impl Into<SharedString>) -> AnyElement {
    text(content)
        .text_size(px(12.0))
        .text_color(colors::muted())
        .text_center()
        .into_element()
}

/// An item of a row: content over a caption.
fn captioned(content: impl IntoElement, label: impl Into<SharedString>) -> AnyElement {
    view()
        .gap_1()
        .items_center()
        .child(content)
        .child(caption(label))
        .into_element()
}

/// The demo's page.
#[derive(Debug, Default)]
pub(crate) struct Images {
    /// The load events of the first image of "Events".
    events: Vec<String>,
    /// What the first image of "Events" reported.
    loaded: Option<String>,
    /// What the image that cannot load reported.
    failed: Option<String>,
    /// Asks for images that are not cached yet ("Reload").
    seed: u32,
    /// The large image shows.
    show: bool,
    /// What `image_size` answered for the large image.
    size: String,
    /// What `prefetch_image` answered.
    prefetched: String,
    /// The large image's download progress and load.
    progress: String,
    /// Waits for `image_size`'s answer.
    sizing: Option<Task<()>>,
    /// Waits for `prefetch_image`'s answer.
    prefetching: Option<Task<()>>,
}

impl Images {
    fn resize_modes() -> AnyElement {
        let modes = [
            (ResizeMode::Cover, "cover"),
            (ResizeMode::Contain, "contain"),
            (ResizeMode::Stretch, "stretch"),
            (ResizeMode::Center, "center"),
        ]
        .map(|(mode, name)| {
            view()
                .gap_1()
                .child(image(WIDE).resize_mode(mode).size(px(140.0)).bg(loading()))
                .child(caption(name))
                .into_element()
        });
        section(
            "Resize modes (a 2:1 image in squares)",
            [view()
                .flex_row()
                .flex_wrap()
                .gap_3()
                .justify_between()
                .children(modes)
                .into_element()],
        )
        .into_element()
    }

    fn corners() -> AnyElement {
        section(
            "Corners and borders",
            [view()
                .flex_row()
                .gap_3()
                .justify_center()
                .child(image(WIDE).size(px(90.0)).rounded(px(45.0)))
                .child(
                    image(WIDE)
                        .size(px(90.0))
                        .rounded(px(16.0))
                        .border(3.0)
                        .border_color(colors::purple()),
                )
                .child(
                    image(WIDE)
                        .size(px(90.0))
                        .rounded_tl(px(40.0))
                        .rounded_br(px(40.0)),
                )
                .into_element()],
        )
        .into_element()
    }

    fn assets() -> AnyElement {
        // At its natural size (64 × 64 points: `@2x` of 128 pixels).
        let tints = [
            (0xcf222e, "#cf222e"),
            (0x1a7f37, "#1a7f37"),
            (0x0969da, "#0969da"),
        ]
        .map(|(tint, name)| captioned(image(LOGO).tint_color(rgb(tint)), name));
        section(
            "Assets and tint",
            [view()
                .flex_row()
                .gap_4()
                .items_center()
                .justify_center()
                .child(captioned(image(LOGO), "logo@2x.png"))
                .children(tints)
                .into_element()],
        )
        .into_element()
    }

    fn animated() -> AnyElement {
        section(
            "Animated GIF and WebP",
            [view()
                .flex_row()
                .gap_6()
                .justify_center()
                .child(captioned(image(BOUNCE), "bounce.gif"))
                .child(captioned(image(SPINNER), "spinner.webp"))
                .child(captioned(
                    image(BOUNCE).size(px(48.0)).rounded(px(24.0)),
                    "scaled, round",
                ))
                .into_element()],
        )
        .into_element()
    }

    /// Network images behind a placeholder, fading in; "Reload" asks for
    /// images that are not cached yet.
    fn placeholders(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let seed = self.seed + 1;
        let images = [0u32, 300, 1000].map(|fade| {
            captioned(
                image(format!(
                    "https://picsum.photos/seed/nsh-fade-{seed}-{fade}/300/300"
                ))
                .placeholder(LOGO)
                .fade_duration(fade as f32)
                .resize_mode(ResizeMode::Cover)
                .size(px(100.0))
                .rounded(px(12.0))
                .bg(loading()),
                format!("fade {fade}ms"),
            )
        });
        section(
            "Placeholder (defaultSource) and fade in",
            [view()
                .gap(px(10.0))
                .child(view().flex_row().justify_between().children(images))
                .child(button("Reload", colors::accent()).on_press(cx.listener(
                    |this: &mut Self, _, cx| {
                        this.seed += 1;
                        cx.notify();
                    },
                )))
                .into_element()],
        )
        .into_element()
    }

    fn blur() -> AnyElement {
        let images = [0u32, 2, 6, 16].map(|radius| {
            captioned(
                image(WIDE)
                    .blur_radius(radius as f32)
                    .size(px(76.0))
                    .rounded(px(8.0)),
                format!("blur {radius}"),
            )
        });
        section(
            "Blur radius",
            [view()
                .flex_row()
                .justify_between()
                .children(images)
                .into_element()],
        )
        .into_element()
    }

    fn info(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let status = [&self.size, &self.prefetched, &self.progress]
            .into_iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" · ");
        let status = if status.is_empty() {
            "Prefetch, then show: no download, no progress".to_owned()
        } else {
            status
        };
        let big = self.show.then(|| {
            image(BIG)
                .h(px(160.0))
                .rounded(px(10.0))
                .bg(loading())
                .on_progress(cx.listener(|this: &mut Self, e: &ImageProgress, cx| {
                    this.progress = format!("{}%", (e.loaded / e.total.max(1.0) * 100.0).round());
                    cx.notify();
                }))
                .on_load(cx.listener(|this: &mut Self, _: &Size, cx| {
                    this.progress = if this.progress.is_empty() {
                        "loaded (no download)".to_owned()
                    } else {
                        format!("{} loaded", this.progress)
                    };
                    cx.notify();
                }))
        });
        section(
            "Image.getSize, Image.prefetch, onProgress",
            [view()
                .gap(px(10.0))
                .child(
                    view()
                        .flex_row()
                        .gap_2()
                        .child(button("getSize", colors::accent()).on_press(cx.listener(
                            |this: &mut Self, _, cx| {
                                let size = cx.image_size(BIG);
                                let task = cx.spawn(|this, _| async move {
                                    let size = size.await;
                                    this.update(move |this, cx| {
                                        this.size = match size {
                                            Ok(ImageSize { width, height }) => {
                                                format!("{width}×{height} px")
                                            }
                                            Err(error) => error,
                                        };
                                        cx.notify();
                                    })
                                    .await;
                                });
                                this.sizing = Some(task);
                            },
                        )))
                        .child(button("prefetch", colors::purple()).on_press(cx.listener(
                            |this: &mut Self, _, cx| {
                                let loaded = cx.prefetch_image(BIG);
                                let task = cx.spawn(|this, _| async move {
                                    let loaded = loaded.await;
                                    this.update(move |this, cx| {
                                        this.prefetched =
                                            if loaded { "prefetched" } else { "failed" }.to_owned();
                                        cx.notify();
                                    })
                                    .await;
                                });
                                this.prefetching = Some(task);
                            },
                        )))
                        .child(
                            button(if self.show { "Hide" } else { "Show" }, rgb(0x1a7f37))
                                .on_press(cx.listener(|this: &mut Self, _, cx| {
                                    this.show = !this.show;
                                    cx.notify();
                                })),
                        ),
                )
                .child(text(status).text_color(colors::muted()))
                .children(big)
                .into_element()],
        )
        .into_element()
    }

    fn events(&self, cx: &RenderCx<'_, Self>) -> AnyElement {
        let log = |line: &'static str| {
            cx.listener(move |this: &mut Self, _: &(), cx| {
                this.events.push(line.to_owned());
                cx.notify();
            })
        };
        let loads = image("https://picsum.photos/seed/nsh-load/300/200")
            .w(px(90.0))
            .h(px(60.0))
            .rounded(px(8.0))
            .bg(loading())
            .on_load_start(log("onLoadStart"))
            .on_load(cx.listener(|this: &mut Self, size: &Size, cx| {
                this.events.push("onLoad".to_owned());
                this.loaded = Some(format!("onLoad: {}×{}", size.width, size.height));
                cx.notify();
            }))
            .on_error(cx.listener(|this: &mut Self, message: &str, cx| {
                this.loaded = Some(format!("onError: {message}"));
                cx.notify();
            }))
            .on_load_end(log("onLoadEnd"));
        let fails = image("https://example.invalid/missing.png")
            .w(px(90.0))
            .h(px(60.0))
            .rounded(px(8.0))
            .bg(loading())
            .on_error(cx.listener(|this: &mut Self, message: &str, cx| {
                this.failed = Some(format!("onError: {message}"));
                cx.notify();
            }));
        let row = |image: Image, status: Text| {
            view()
                .flex_row()
                .gap_3()
                .items_center()
                .child(image)
                .child(status.flex_1())
                .into_element()
        };
        let pending = || "loading…".to_owned();
        section(
            "Events",
            [
                row(loads, text(self.loaded.clone().unwrap_or_else(pending))),
                row(
                    fails,
                    text(self.failed.clone().unwrap_or_else(pending)).number_of_lines(3),
                ),
                caption(if self.events.is_empty() {
                    "no load events yet".to_owned()
                } else {
                    self.events.join(" → ")
                }),
            ],
        )
        .into_element()
    }
}

impl Component for Images {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().flex_1().child(page([
            Self::resize_modes(),
            Self::corners(),
            Self::assets(),
            Self::animated(),
            self.placeholders(cx),
            Self::blur(),
            self.info(cx),
            self.events(cx),
        ]))
    }
}
