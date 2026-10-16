use std::{
    path::Path,
    sync::{Arc, OnceLock},
    time::Duration,
};

use anyhow::{Context as _, bail};
use crossbeam_channel::{Receiver, Sender};
use nsh_bundler::BundleOptions;
use nsh_core::{ApproxTextEngine, EventKind, MountBatch, MountItem, Size};
use nsh_engine::{Bundle, Engine, EngineConfig, EngineControl, MountSink};

use crate::{build::BuildError, ui};

struct ChannelSink {
    batches: Sender<MountBatch>,
    /// Set once the engine runs; drives `requestAnimationFrame` at 60 Hz.
    control: Arc<OnceLock<EngineControl>>,
}

impl MountSink for ChannelSink {
    fn submit(&self, batch: MountBatch) {
        let _ = self.batches.send(batch);
    }

    fn request_frame(&self) {
        let control = self.control.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_micros(16_667));
            if let Some(control) = control.get() {
                control.frame();
            }
        });
    }
}

/// Runs the app headless, taps the first pressable view `taps` times and
/// prints every mount batch.
pub(crate) fn trace(
    entry: &Path,
    taps: u32,
    idle_ms: u64,
    scroll: Option<f32>,
    width: f32,
    height: f32,
) -> anyhow::Result<()> {
    let bundle = nsh_bundler::bundle(&BundleOptions {
        entry: entry.to_path_buf(),
        minify: false,
        ui_sets: crate::ui_sets().to_vec(),
    })
    .map_err(|e| BuildError::from(e).into_error(entry))?;
    // Exact numbers: traces are compared.
    ui::done(format!(
        "bundled {} {}",
        ui::path(entry),
        ui::dim(format!(
            "({} B in {:.1} ms)",
            bundle.code.len(),
            bundle.elapsed_ms
        ))
    ));
    let bytecode = nsh_js::compile_bytecode_with(&bundle.code, false, crate::ui_sets())?;
    // SAFETY: compiled just now by this runtime.
    let bytecode = unsafe { Bundle::bytecode(bytecode) };

    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: nsh_engine::App::Js(match bundle.source_map {
            Some(map) => bytecode.with_source_map(map),
            None => bytecode,
        }),
        viewport: Size::new(width, height),
        trace: true,
        network: Some(std::sync::Arc::new(crate::net::UreqNetwork::new())),
        assets: Some(std::sync::Arc::new(crate::net::DirectoryAssets(
            entry.parent().unwrap_or(Path::new(".")).to_path_buf(),
        ))),
        environment: Default::default(),
        header_heights: (44.0, 96.0),
        limits: nsh_engine::Limits::default(),
        ui_sets: crate::ui_sets().to_vec(),
        runtime: None,
    };
    let control = Arc::new(OnceLock::new());
    let sink = ChannelSink {
        batches: tx,
        control: control.clone(),
    };
    let engine = Engine::start(config, || ApproxTextEngine, sink)?;
    let _ = control.set(engine.control());

    let first = next(&rx).context("the app rendered nothing")?;
    print_batch("initial render", &first);
    let pressable = first.items.iter().find_map(|item| match item {
        MountItem::Events { id, events } if events.contains(EventKind::Press) => Some(*id),
        _ => None,
    });
    let scroll_view = first.items.iter().find_map(|item| match item {
        MountItem::Create {
            id,
            kind: nsh_core::ViewKind::Scroll,
        } => Some(*id),
        _ => None,
    });
    if let (Some(distance), Some(id)) = (scroll, scroll_view) {
        let mut y = 0.0;
        while y < distance {
            y += 10.0;
            // The simulated platform applied every scroll the core asked for.
            engine.control().scroll(id, 0.0, y, u32::MAX);
            // Frames that need nothing produce no batch.
            if let Ok(batch) = rx.recv_timeout(Duration::from_millis(50)) {
                print_batch(&format!("scroll to {y}"), &batch);
            }
        }
    }
    // Batches emitted while idling (e.g. releases after GC) are printed too.
    while let Ok(batch) = rx.recv_timeout(Duration::from_millis(idle_ms)) {
        print_batch("while idle", &batch);
    }
    match pressable {
        Some(id) => {
            for i in 1..=taps {
                engine
                    .control()
                    .dispatch_event(id, EventKind::Press, nsh_core::EventPayload::None);
                match next(&rx) {
                    Some(batch) => print_batch(&format!("tap #{i} on view {id}"), &batch),
                    None => bail!("tap #{i} produced no update"),
                }
            }
        }
        None if taps > 0 => ui::warn("no pressable view found"),
        None => {}
    }
    Ok(())
}

fn next(rx: &Receiver<MountBatch>) -> Option<MountBatch> {
    rx.recv_timeout(Duration::from_secs(5)).ok()
}

fn print_batch(title: &str, batch: &MountBatch) {
    println!("── {title}: {} items", batch.items.len());
    for item in &batch.items {
        match item {
            MountItem::Text {
                id, text, content, ..
            } => println!(
                "   Text {{ id: {id}, text: {:?}, content: {}x{} @ {},{} }}",
                text.text, content.width, content.height, content.x, content.y
            ),
            MountItem::Frame { id, frame } => println!(
                "   Frame {{ id: {id}, x: {}, y: {}, w: {}, h: {} }}",
                frame.x, frame.y, frame.width, frame.height
            ),
            other => println!("   {other:?}"),
        }
    }
}
