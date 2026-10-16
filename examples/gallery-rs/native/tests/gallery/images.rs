//! The Images demo on a headless engine: its sections render, the crate's
//! assets lay out at their natural size, its images report their events,
//! "Show" and "Reload" change what loads, `getSize` and `prefetch` show the
//! platform's answers.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{Command, EventKind, EventPayload, MountBatch, MountItem, PayloadValue, ROOT_ID};
use nsh_engine::Engine;

/// An image source's size in points, when known up front.
type NaturalSize = (f32, f32);

/// What is on screen, the sources of the images by node (with their
/// natural sizes) and the image information asked for.
#[derive(Default)]
struct Shown {
    screen: crate::common::Screen,
    images: Vec<(u32, String, Option<NaturalSize>)>,
    /// `Command::ImageInfo` requests: id and URI.
    infos: Vec<(u32, String)>,
}

impl Shown {
    fn drain(&mut self, rx: &Receiver<MountBatch>) {
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            self.screen.apply(&batch);
            for item in &batch.items {
                if let MountItem::Image {
                    id,
                    source: Some(source),
                    ..
                } = item
                {
                    self.images.retain(|i| i.0 != *id);
                    self.images.push((*id, source.uri.to_string(), source.size));
                }
                if let MountItem::Command {
                    id: ROOT_ID,
                    command: Command::ImageInfo { id, uri },
                } = item
                {
                    self.infos.push((*id, uri.clone()));
                }
            }
        }
    }

    fn image(&self, uri: &str) -> u32 {
        self.images
            .iter()
            .find(|i| i.1 == uri)
            .unwrap_or_else(|| panic!("no image {uri}: {:?}", self.images))
            .0
    }

    fn natural_size(&self, uri: &str) -> Option<NaturalSize> {
        self.images.iter().find(|i| i.1 == uri).and_then(|i| i.2)
    }

    fn shows(&self, text: &str) -> bool {
        self.screen.texts.iter().any(|t| t.1 == text)
    }

    fn press(&mut self, engine: &Engine, rx: &Receiver<MountBatch>, label: &str) {
        let text = self
            .screen
            .find(label)
            .unwrap_or_else(|| panic!("no {label}: {:?}", self.screen.texts));
        let target = self.screen.parents[&text];
        engine
            .control()
            .dispatch_event(target, EventKind::Press, EventPayload::None);
        self.drain(rx);
    }

    fn send(
        &mut self,
        engine: &Engine,
        rx: &Receiver<MountBatch>,
        id: u32,
        kind: EventKind,
        payload: EventPayload,
    ) {
        engine.control().dispatch_event(id, kind, payload);
        self.drain(rx);
    }
}

fn open() -> (Engine, Receiver<MountBatch>, Shown) {
    let (engine, rx) = crate::common::start();
    let mut shown = Shown::default();
    shown.drain(&rx);
    shown.press(&engine, &rx, "Images");
    assert!(
        shown.screen.titles.iter().any(|t| t.1 == "Images"),
        "Images opens: {:?}",
        shown.screen.titles
    );
    (engine, rx, shown)
}

#[test]
fn images_render_their_sections() {
    let (_engine, _rx, shown) = open();
    for text in [
        "RESIZE MODES (A 2:1 IMAGE IN SQUARES)",
        "cover",
        "center",
        "logo@2x.png",
        "#1a7f37",
        "spinner.webp",
        "fade 300ms",
        "blur 16",
        "Prefetch, then show: no download, no progress",
        "no load events yet",
    ] {
        assert!(shown.shows(text), "{text}: {:?}", shown.screen.texts);
    }
    // The crate's files, at the natural size `asset!` read (in points).
    assert_eq!(
        shown.natural_size("asset:/@gallery-native/logo@2x.png"),
        Some((64.0, 64.0))
    );
    assert_eq!(
        shown.natural_size("asset:/@gallery-native/bounce.gif"),
        Some((96.0, 96.0))
    );
    shown.image("asset:/@gallery-native/spinner.webp");
    shown.image("https://picsum.photos/seed/nsh-fade-1-1000/300/300");
}

#[test]
fn images_show_what_get_size_and_prefetch_answer() {
    let (engine, rx, mut shown) = open();
    let big = "https://picsum.photos/seed/nsh-progress/2400/1600";
    shown.press(&engine, &rx, "getSize");
    shown.press(&engine, &rx, "prefetch");
    let infos = shown.infos.clone();
    let [(size, size_uri), (prefetch, prefetch_uri)] = &infos[..] else {
        panic!("two requests: {infos:?}");
    };
    let (size, prefetch) = (*size, *prefetch);
    assert_eq!((size_uri.as_str(), prefetch_uri.as_str()), (big, big));
    let answer = |id: u32, size: Option<(f64, f64)>| {
        let mut entries = vec![("id", PayloadValue::Number(f64::from(id)))];
        match size {
            Some((width, height)) => entries.extend([
                ("width", PayloadValue::Number(width)),
                ("height", PayloadValue::Number(height)),
            ]),
            None => entries.push(("error", PayloadValue::Str("offline".into()))),
        }
        EventPayload::Map(entries)
    };
    shown.send(
        &engine,
        &rx,
        ROOT_ID,
        EventKind::ImageInfo,
        answer(size, Some((2400.0, 1600.0))),
    );
    assert!(shown.shows("2400×1600 px"), "{:?}", shown.screen.texts);
    shown.send(
        &engine,
        &rx,
        ROOT_ID,
        EventKind::ImageInfo,
        answer(prefetch, None),
    );
    assert!(
        shown.shows("2400×1600 px · failed"),
        "{:?}",
        shown.screen.texts
    );
}

#[test]
fn images_report_their_events() {
    let (engine, rx, mut shown) = open();
    let missing = shown.image("https://example.invalid/missing.png");
    shown.send(
        &engine,
        &rx,
        missing,
        EventKind::Error,
        EventPayload::Error {
            message: "not found".into(),
        },
    );
    assert!(
        shown.shows("onError: not found"),
        "{:?}",
        shown.screen.texts
    );

    let loads = shown.image("https://picsum.photos/seed/nsh-load/300/200");
    shown.send(
        &engine,
        &rx,
        loads,
        EventKind::LoadStart,
        EventPayload::None,
    );
    shown.send(
        &engine,
        &rx,
        loads,
        EventKind::Load,
        EventPayload::Image {
            width: 300.0,
            height: 200.0,
        },
    );
    shown.send(&engine, &rx, loads, EventKind::LoadEnd, EventPayload::None);
    assert!(shown.shows("onLoad: 300×200"), "{:?}", shown.screen.texts);
    assert!(
        shown.shows("onLoadStart → onLoad → onLoadEnd"),
        "{:?}",
        shown.screen.texts
    );
}

#[test]
fn show_and_reload_change_what_loads() {
    let (engine, rx, mut shown) = open();
    shown.press(&engine, &rx, "Show");
    assert!(shown.shows("Hide"), "{:?}", shown.screen.texts);
    let big = shown.image("https://picsum.photos/seed/nsh-progress/2400/1600");
    shown.send(
        &engine,
        &rx,
        big,
        EventKind::Progress,
        EventPayload::Map(vec![
            ("loaded", PayloadValue::Number(512.0)),
            ("total", PayloadValue::Number(1024.0)),
        ]),
    );
    assert!(shown.shows("50%"), "{:?}", shown.screen.texts);
    shown.send(
        &engine,
        &rx,
        big,
        EventKind::Load,
        EventPayload::Image {
            width: 2400.0,
            height: 1600.0,
        },
    );
    assert!(shown.shows("50% loaded"), "{:?}", shown.screen.texts);

    shown.press(&engine, &rx, "Reload");
    shown.image("https://picsum.photos/seed/nsh-fade-2-0/300/300");
}

#[test]
fn the_gallery_ships_its_assets() {
    use nsh_core::ui::AssetKind;
    // The set `#[native_module]` makes, as nsh's generated workspace gets it.
    let registry = gallery_native::__nsh_native_module();
    let files: Vec<_> = registry
        .asset_files()
        .map(|f| {
            (
                f.path(),
                f.kind,
                f.source
                    .ends_with(f.path().trim_start_matches("@gallery-native/")),
            )
        })
        .collect();
    assert_eq!(
        files,
        [
            ("@gallery-native/bounce.gif", AssetKind::Image, true),
            ("@gallery-native/fonts/OFL.txt", AssetKind::File, true),
            (
                "@gallery-native/fonts/Pacifico-Regular.ttf",
                AssetKind::Font,
                true
            ),
            ("@gallery-native/logo@2x.png", AssetKind::Image, true),
            ("@gallery-native/spinner.webp", AssetKind::Image, true),
        ]
    );
}
