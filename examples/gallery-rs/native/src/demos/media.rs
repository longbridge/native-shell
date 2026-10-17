//! Web and video: WKWebView / WebView, AVPlayer / MediaPlayer (as the TSX gallery's `demos/Media.tsx`).

use std::time::{SystemTime, UNIX_EPOCH};

use nsh_core::ui::{AnyElement, prelude::*};

use crate::ui::legacy::{button, colors, page, section};

/// The inline page of the first web view: it posts to the app and shows
/// what the app posts to it.
const PAGE: &str = r#"<!doctype html><html><head><meta name="viewport" content="width=device-width"></head>
<body style="font-family: -apple-system, sans-serif; margin: 16px">
<h3>An inline page</h3>
<button onclick="window.ReactNativeWebView.postMessage('hello from the page')">Post to the app</button>
<p id="got">Nothing from the app yet.</p>
<script>window.addEventListener('message', (e) => { document.getElementById('got').textContent = 'The app says: ' + e.data; });</script>
</body></html>"#;

/// The web site of the second web view.
const SITE: &str = "https://en.m.wikipedia.org/wiki/React_Native";

/// The video.
const FLOWER: &str = "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4";

/// A disabled-looking button's color.
const INACTIVE: u32 = 0x8c959f;

/// The demo's page: an inline page, a web site and a video, with what they
/// tell the app.
#[derive(Debug, Default)]
pub(crate) struct Media {
    /// The inline page.
    web: Option<ElementRef>,
    /// The web site.
    site: Option<ElementRef>,
    /// The video.
    video: Option<ElementRef>,
    /// The web site's last navigation state.
    state: Option<WebNavigation>,
    /// The video shows the platform's controls.
    controls: bool,
    /// What the inline page posted last.
    message: String,
    /// The video plays (it starts paused).
    playing: bool,
    /// The video's position, `"1.2 / 5.0 s"`.
    progress: String,
}

impl Media {
    /// The line under the web site: its title (or URL) while it is known.
    fn site_status(&self) -> String {
        match &self.state {
            Some(s) => format!(
                "{}{}",
                if s.loading { "Loading " } else { "" },
                if s.title.is_empty() { &s.url } else { &s.title }
            ),
            None => "…".into(),
        }
    }
}

/// A row of controls.
fn controls(content: impl IntoIterator<Item = AnyElement>) -> AnyElement {
    view()
        .flex_row()
        .items_center()
        .gap_3()
        .children(content)
        .into_element()
}

impl Component for Media {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self.web = Some(cx.element_ref());
        self.site = Some(cx.element_ref());
        self.video = Some(cx.element_ref());
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        // Each ref is made in `setup`, before the first render.
        let [web, site, player] = [&self.web, &self.site, &self.video].map(|r| r.clone().unwrap());
        let state = self.state.as_ref();
        let can_go_back = state.is_some_and(|s| s.can_go_back);
        let can_go_forward = state.is_some_and(|s| s.can_go_forward);

        let inline = section(
            "Web view",
            [
                web_view_html(PAGE, None)
                    .element_ref(&web)
                    .h(px(200.0))
                    .rounded_lg()
                    .overflow_hidden()
                    .on_message(cx.listener(|this: &mut Self, data: &str, cx| {
                        this.message = data.to_owned();
                        cx.notify();
                    }))
                    .into_element(),
                controls([
                    button("Post to the page", colors::accent())
                        .on_press(cx.listener(move |_: &mut Self, _, _| {
                            let now = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default();
                            web.post_message(format!("hi #{}", now.as_millis() % 1000));
                        }))
                        .into_element(),
                    text(self.message.clone())
                        .flex_1()
                        .text_color(colors::muted())
                        .into_element(),
                ]),
            ],
        );

        let back = site.clone();
        let forward = site.clone();
        let website = section(
            "A web site",
            [
                web_view(SITE)
                    .element_ref(&site)
                    .h(px(320.0))
                    .rounded_lg()
                    .overflow_hidden()
                    .on_navigation_state_change(cx.listener(
                        |this: &mut Self, state: &WebNavigation, cx| {
                            this.state = Some(state.clone());
                            cx.notify();
                        },
                    ))
                    .into_element(),
                text(self.site_status())
                    .number_of_lines(1)
                    .text_color(colors::muted())
                    .text_size(px(13.0))
                    .into_element(),
                view()
                    .flex_row()
                    .gap_2()
                    .child(
                        button(
                            "Back",
                            if can_go_back {
                                colors::accent()
                            } else {
                                rgb(INACTIVE)
                            },
                        )
                        .on_press(cx.listener(move |_: &mut Self, _, _| back.go_back())),
                    )
                    .child(
                        button(
                            "Forward",
                            if can_go_forward {
                                colors::accent()
                            } else {
                                rgb(INACTIVE)
                            },
                        )
                        .on_press(cx.listener(move |_: &mut Self, _, _| forward.go_forward())),
                    )
                    .child(
                        button("Reload", colors::gray())
                            .on_press(cx.listener(move |_: &mut Self, _, _| site.reload())),
                    )
                    .into_element(),
            ],
        );

        let playback = section(
            "Video",
            [
                video(FLOWER)
                    .element_ref(&player)
                    .paused(!self.playing)
                    .muted(true)
                    .repeat(true)
                    .controls(self.controls)
                    .h(px(200.0))
                    .bg(rgb(0x000000))
                    .rounded_lg()
                    .overflow_hidden()
                    .on_progress(cx.listener(|this: &mut Self, e: &VideoProgress, cx| {
                        this.progress = format!("{:.1} / {:.1} s", e.current_time, e.duration);
                        cx.notify();
                    }))
                    .into_element(),
                controls([
                    button(
                        if self.playing { "Pause" } else { "Play" },
                        colors::accent(),
                    )
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        this.playing = !this.playing;
                        cx.notify();
                    }))
                    .into_element(),
                    button("Restart", colors::gray())
                        .on_press(cx.listener(move |_: &mut Self, _, _| player.seek(0.0)))
                        .into_element(),
                    button(
                        if self.controls {
                            "Own controls"
                        } else {
                            "System controls"
                        },
                        colors::purple(),
                    )
                    .on_press(cx.listener(|this: &mut Self, _, cx| {
                        this.controls = !this.controls;
                        cx.notify();
                    }))
                    .into_element(),
                    text(self.progress.clone())
                        .text_color(colors::muted())
                        .into_element(),
                ]),
            ],
        );

        view().flex_1().child(page([
            inline.into_element(),
            website.into_element(),
            playback.into_element(),
        ]))
    }
}
