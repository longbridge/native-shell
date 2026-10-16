//! What the gallery's headless tests share: an engine running the
//! gallery, what it shows, presses.

#![allow(dead_code, unreachable_pub)]

use std::{sync::Arc, time::Duration};

use crossbeam_channel::Receiver;
use nsh_core::{
    ApproxTextEngine, AttributedText, EventKind, EventPayload, MountBatch, MountItem, Size,
    ui::Root,
};
use nsh_engine::{App, Engine, EngineConfig, Environment, MountSink};

pub struct Sink(crossbeam_channel::Sender<MountBatch>);

impl MountSink for Sink {
    fn submit(&self, batch: MountBatch) {
        let _ = self.0.send(batch);
    }
}

/// What is on screen: texts by node and screens' titles.
#[derive(Default)]
pub struct Screen {
    pub texts: Vec<(u32, String)>,
    /// The text blocks as built: spans, links, attachments.
    pub blocks: Vec<(u32, Arc<AttributedText>)>,
    pub titles: Vec<(u32, String)>,
    pub parents: std::collections::HashMap<u32, u32>,
}

impl Screen {
    pub fn apply(&mut self, batch: &MountBatch) {
        for item in &batch.items {
            match item {
                MountItem::Text { id, text, .. } => {
                    self.texts.retain(|t| t.0 != *id);
                    self.texts.push((*id, text.text.clone()));
                    self.blocks.retain(|t| t.0 != *id);
                    self.blocks.push((*id, text.clone()));
                }
                MountItem::ScreenConfig { id, config } => {
                    self.titles.retain(|t| t.0 != *id);
                    if let Some(title) = &config.title {
                        self.titles.push((*id, title.to_string()));
                    }
                }
                MountItem::Insert { parent, child, .. } => {
                    self.parents.insert(*child, *parent);
                }
                MountItem::Delete { id } => {
                    self.texts.retain(|t| t.0 != *id);
                    self.blocks.retain(|t| t.0 != *id);
                    self.titles.retain(|t| t.0 != *id);
                }
                _ => {}
            }
        }
    }

    /// The pressable node around text `label` (the text or a parent).
    pub fn find(&self, label: &str) -> Option<u32> {
        self.texts.iter().find(|t| t.1 == label).map(|t| t.0)
    }

    /// The text block showing `text`.
    pub fn block(&self, text: &str) -> Option<&AttributedText> {
        self.blocks.iter().find(|b| b.1.text == text).map(|b| &*b.1)
    }

    /// The span node of the link `label` in a text block.
    pub fn link(&self, label: &str) -> Option<u32> {
        self.blocks.iter().find_map(|(_, block)| {
            let units: Vec<u16> = block.text.encode_utf16().collect();
            block.links.iter().find_map(|l| {
                let range = units.get(l.start as usize..l.end as usize)?;
                (String::from_utf16_lossy(range) == label).then_some(l.node)
            })
        })
    }
}

pub fn drain(rx: &Receiver<MountBatch>, screen: &mut Screen) {
    while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
        screen.apply(&batch);
    }
}

pub fn logger() {
    struct L;
    impl log::Log for L {
        fn enabled(&self, m: &log::Metadata<'_>) -> bool {
            m.level() <= log::Level::Warn
        }
        fn log(&self, r: &log::Record<'_>) {
            if self.enabled(r.metadata()) {
                eprintln!("{} {}: {}", r.level(), r.target(), r.args());
            }
        }
        fn flush(&self) {}
    }
    static LOGGER: L = L;
    let _ = log::set_logger(&LOGGER);
    log::set_max_level(log::LevelFilter::Warn);
}

/// A headless engine running the gallery, and its batches.
pub fn start() -> (nsh_engine::Engine, Receiver<MountBatch>) {
    logger();
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<gallery_native::Gallery>()),
        viewport: Size::new(390.0, 844.0),
        trace: false,
        network: None,
        assets: None,
        environment: Environment::default(),
        header_heights: (44.0, 96.0),
        limits: nsh_engine::Limits::default(),
        ui_sets: Vec::new(),
        runtime: None,
    };
    let engine = Engine::start(config, || ApproxTextEngine, Sink(tx)).unwrap();
    (engine, rx)
}

/// Presses the parent of the text `label` (a row or a button), and waits
/// for what it changes.
pub fn press(engine: &Engine, rx: &Receiver<MountBatch>, screen: &mut Screen, label: &str) {
    let text = screen
        .find(label)
        .unwrap_or_else(|| panic!("no {label}: {:?}", screen.texts));
    let target = screen.parents[&text];
    engine
        .control()
        .dispatch_event(target, EventKind::Press, EventPayload::None);
    drain(rx, screen);
}

/// The gallery, with demo `name` opened from the index.
pub fn open(name: &str) -> (Engine, Receiver<MountBatch>, Screen) {
    let (engine, rx) = start();
    let mut screen = Screen::default();
    drain(&rx, &mut screen);
    press(&engine, &rx, &mut screen, name);
    assert!(
        screen.titles.iter().any(|t| t.1 == name),
        "{name} opens: {:?}",
        screen.titles
    );
    (engine, rx, screen)
}

/// Whether text `text` is on screen.
pub fn shows(screen: &Screen, text: &str) -> bool {
    screen.texts.iter().any(|t| t.1 == text)
}

/// Presses the text `label` itself (a pressable text: a link, a chip, a
/// tab), and waits for what it changes.
pub fn press_text(engine: &Engine, rx: &Receiver<MountBatch>, screen: &mut Screen, label: &str) {
    let text = screen
        .find(label)
        .unwrap_or_else(|| panic!("no {label}: {:?}", screen.texts));
    engine
        .control()
        .dispatch_event(text, EventKind::Press, EventPayload::None);
    drain(rx, screen);
}

/// Sends `kind` to the link `label` inside a text block (its span), and
/// waits for what it changes.
pub fn press_link(
    engine: &Engine,
    rx: &Receiver<MountBatch>,
    screen: &mut Screen,
    label: &str,
    kind: EventKind,
) {
    let link = screen
        .link(label)
        .unwrap_or_else(|| panic!("no link {label}: {:?}", screen.texts));
    engine
        .control()
        .dispatch_event(link, kind, EventPayload::None);
    drain(rx, screen);
}
