//! The platform's services for Rust components on the engine: the fonts
//! the UI sets' crates ship are registered before the first render.

use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, Sender};
use nsh_core::{
    ApproxTextEngine, AttributedText, MountBatch, MountItem, Size, TextEngine, TextLayout,
    ui::{AssetFile, AssetKind, Registry, Root, prelude::*},
};
use nsh_engine::{App, Assets, Engine, EngineConfig, Environment, MountSink};

/// Hands the batches to the test.
struct Batches(Sender<MountBatch>);

impl MountSink for Batches {
    fn submit(&self, batch: MountBatch) {
        let _ = self.0.send(batch);
    }
}

fn config(app: App) -> EngineConfig {
    EngineConfig {
        app,
        viewport: Size::new(390.0, 844.0),
        trace: false,
        network: None,
        assets: None,
        environment: Environment::default(),
        header_heights: (44.0, 96.0),
        limits: nsh_engine::Limits::default(),
        ui_sets: Vec::new(),
        runtime: None,
    }
}

/// Waits until a batch shows `until` (or 5 seconds passed).
fn until(rx: &Receiver<MountBatch>, until: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while let Ok(batch) = rx.recv_deadline(deadline) {
        if batch
            .items
            .iter()
            .any(|item| matches!(item, MountItem::Text { text, .. } if text.text == until))
        {
            break;
        }
    }
}

/// Records the fonts registered.
struct Fonts(ApproxTextEngine, Arc<Mutex<Vec<Vec<u8>>>>);

impl TextEngine for Fonts {
    type Prepared = <ApproxTextEngine as TextEngine>::Prepared;

    fn prepare(&self, text: &AttributedText) -> Self::Prepared {
        self.0.prepare(text)
    }

    fn measure(&self, prepared: &Self::Prepared, max_width: Option<f32>) -> Size {
        self.0.measure(prepared, max_width)
    }

    fn layout(&self, prepared: &Self::Prepared, size: Size) -> TextLayout {
        self.0.layout(prepared, size)
    }

    fn register_font(&self, data: &[u8]) -> Result<(), String> {
        self.1.lock().unwrap().push(data.to_vec());
        Ok(())
    }
}

/// The files of a crate: a font, an image.
static FILES: &[AssetFile] = &[
    AssetFile {
        uri: "asset:/@app/fonts/Script.ttf",
        source: "/nowhere/fonts/Script.ttf",
        kind: AssetKind::Font,
    },
    AssetFile {
        uri: "asset:/@app/logo.png",
        source: "/nowhere/logo.png",
        kind: AssetKind::Image,
    },
];

/// Reads the font, and records what was read.
struct CrateAssets(Arc<Mutex<Vec<String>>>);

impl Assets for CrateAssets {
    fn read(&self, uri: &str) -> Result<Vec<u8>, String> {
        self.0.lock().unwrap().push(uri.to_owned());
        Ok(uri.as_bytes().to_vec())
    }
}

#[derive(Default)]
struct Label;

impl Component for Label {
    const NAME: &'static str = "app.label";
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().child(text("ready").font_family("Script"))
    }
}

#[test]
fn the_fonts_ui_sets_ship_are_registered_before_the_first_render() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let read = Arc::new(Mutex::new(Vec::new()));
    let fonts = Arc::new(Mutex::new(Vec::new()));
    let mut set = Registry::ui_set("app/native");
    set.assets(FILES);
    let config = EngineConfig {
        assets: Some(Arc::new(CrateAssets(read.clone()))),
        ui_sets: vec![set],
        ..config(App::Rust(Root::of::<Label>()))
    };
    let registered = fonts.clone();
    let _engine = Engine::start(
        config,
        move || Fonts(ApproxTextEngine, registered),
        Batches(tx),
    )
    .unwrap();
    until(&rx, "ready");
    // Only the font is read (images load where they show).
    assert_eq!(*read.lock().unwrap(), ["asset:/@app/fonts/Script.ttf"]);
    assert_eq!(
        *fonts.lock().unwrap(),
        [b"asset:/@app/fonts/Script.ttf".to_vec()]
    );
}
