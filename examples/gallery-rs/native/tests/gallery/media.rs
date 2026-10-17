//! The Web and video demo: two web views and a video, the buttons driving
//! them and what they tell the app.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    Command, EventKind, EventPayload, MountBatch, MountItem, NodeId, PayloadValue, ViewKind,
};
use nsh_engine::Engine;

use crate::common::shows;

/// What the gallery shows, the nodes of its web views and video, and the
/// commands sent to nodes.
#[derive(Default)]
struct Media {
    screen: crate::common::Screen,
    web_views: Vec<NodeId>,
    videos: Vec<NodeId>,
    commands: Vec<(NodeId, Command)>,
}

impl Media {
    fn drain(&mut self, rx: &Receiver<MountBatch>) {
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            self.screen.apply(&batch);
            for item in &batch.items {
                match item {
                    MountItem::Create {
                        id,
                        kind: ViewKind::WebView,
                    } => self.web_views.push(*id),
                    MountItem::Create {
                        id,
                        kind: ViewKind::Video,
                    } => self.videos.push(*id),
                    MountItem::Command { id, command } => {
                        self.commands.push((*id, command.clone()))
                    }
                    _ => {}
                }
            }
        }
    }

    fn press(&mut self, engine: &Engine, rx: &Receiver<MountBatch>, label: &str) {
        let text = self
            .screen
            .find(label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", self.screen.texts));
        engine.control().dispatch_event(
            self.screen.press_target(text),
            EventKind::Press,
            EventPayload::None,
        );
        self.drain(rx);
    }

    fn event(
        &mut self,
        engine: &Engine,
        rx: &Receiver<MountBatch>,
        node: NodeId,
        kind: EventKind,
        payload: Vec<(&'static str, PayloadValue)>,
    ) {
        engine
            .control()
            .dispatch_event(node, kind, EventPayload::Map(payload));
        self.drain(rx);
    }
}

/// The gallery with the demo open: the index first, then the demo's
/// mount items, so the views it creates are seen.
fn open() -> (Engine, Receiver<MountBatch>, Media) {
    let (engine, rx) = crate::common::start();
    let mut media = Media::default();
    media.drain(&rx);
    media.press(&engine, &rx, "Web and video");
    assert!(
        media.screen.titles.iter().any(|t| t.1 == "Web and video"),
        "{:?}",
        media.screen.titles
    );
    (engine, rx, media)
}

#[test]
fn media_shows_its_sections_and_toggles_the_video() {
    let (engine, rx, mut media) = open();
    for text in [
        "WEB VIEW",
        "A WEB SITE",
        "VIDEO",
        "Post to the page",
        "…",
        "Reload",
    ] {
        assert!(
            shows(&media.screen, text),
            "{text}: {:?}",
            media.screen.texts
        );
    }
    assert_eq!(media.web_views.len(), 2);
    assert_eq!(media.videos.len(), 1);

    media.press(&engine, &rx, "Play");
    assert!(shows(&media.screen, "Pause"), "{:?}", media.screen.texts);
    media.press(&engine, &rx, "System controls");
    assert!(
        shows(&media.screen, "Own controls"),
        "{:?}",
        media.screen.texts
    );

    let video = media.videos[0];
    media.press(&engine, &rx, "Restart");
    assert!(
        media
            .commands
            .iter()
            .any(|(id, c)| *id == video && *c == Command::Seek(0.0)),
        "{:?}",
        media.commands
    );
    media.event(
        &engine,
        &rx,
        video,
        EventKind::Progress,
        vec![
            ("currentTime", PayloadValue::Number(1.5)),
            ("duration", PayloadValue::Number(5.0)),
        ],
    );
    assert!(
        shows(&media.screen, "1.5 / 5.0 s"),
        "{:?}",
        media.screen.texts
    );
}

#[test]
fn media_talks_to_its_web_views() {
    let (engine, rx, mut media) = open();
    let (page, site) = (media.web_views[0], media.web_views[1]);

    media.event(
        &engine,
        &rx,
        page,
        EventKind::Message,
        vec![("data", PayloadValue::Str("hello from the page".into()))],
    );
    assert!(
        shows(&media.screen, "hello from the page"),
        "{:?}",
        media.screen.texts
    );

    media.press(&engine, &rx, "Post to the page");
    assert!(
        media
            .commands
            .iter()
            .any(|(id, c)| *id == page
                && matches!(c, Command::PostMessage(m) if m.starts_with("hi #"))),
        "{:?}",
        media.commands
    );

    media.event(
        &engine,
        &rx,
        site,
        EventKind::NavigationStateChange,
        vec![
            (
                "url",
                PayloadValue::Str("https://en.m.wikipedia.org/wiki/React_Native".into()),
            ),
            ("title", PayloadValue::Str("React Native".into())),
            ("loading", PayloadValue::Bool(true)),
            ("canGoBack", PayloadValue::Bool(true)),
            ("canGoForward", PayloadValue::Bool(false)),
        ],
    );
    assert!(
        shows(&media.screen, "Loading React Native"),
        "{:?}",
        media.screen.texts
    );
    media.press(&engine, &rx, "Back");
    assert!(
        media
            .commands
            .iter()
            .any(|(id, c)| *id == site && *c == Command::GoBack),
        "{:?}",
        media.commands
    );
}
