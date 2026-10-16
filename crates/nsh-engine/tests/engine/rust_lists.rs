//! Rust components' lists on the engine: a list's own header, footer,
//! empty view and separators, viewable rows.

use std::{collections::HashMap, time::Duration};

use crossbeam_channel::Receiver;
use nsh_core::{
    ApproxTextEngine, EventKind, EventPayload, MountBatch, MountItem, PayloadValue, Size,
    ui::{Root, prelude::*},
};
use nsh_engine::{App, Engine, EngineConfig, Environment};

use crate::ChannelSink;

fn start(root: Root) -> (Engine, Receiver<MountBatch>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(root),
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
    (
        Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap(),
        rx,
    )
}

/// What the batches built: texts, parents, kinds, and the list cells'
/// slots.
#[derive(Default)]
struct Tree {
    texts: Vec<(u32, String)>,
    parents: HashMap<u32, u32>,
    kinds: HashMap<u32, String>,
    /// Every text shown, in order.
    history: Vec<String>,
    commands: Vec<(u32, nsh_core::Command)>,
}

impl Tree {
    fn apply(&mut self, batch: &MountBatch) {
        for item in &batch.items {
            match item {
                MountItem::Text { id, text, .. } => {
                    self.history.push(text.text.clone());
                    self.texts.retain(|t| t.0 != *id);
                    self.texts.push((*id, text.text.clone()));
                }
                MountItem::Insert { parent, child, .. } => {
                    self.parents.insert(*child, *parent);
                }
                MountItem::Create { id, kind, .. } => {
                    self.kinds.insert(*id, format!("{kind:?}"));
                }
                MountItem::Delete { id } => {
                    self.texts.retain(|t| t.0 != *id);
                    self.parents.remove(id);
                }
                MountItem::Command { id, command } => self.commands.push((*id, command.clone())),
                _ => {}
            }
        }
    }

    fn drain(&mut self, rx: &Receiver<MountBatch>) {
        while let Ok(batch) = rx.recv_timeout(Duration::from_millis(300)) {
            self.apply(&batch);
        }
    }

    fn find(&self, label: &str) -> Option<u32> {
        self.texts.iter().find(|t| t.1 == label).map(|t| t.0)
    }

    fn shows(&self, label: &str) -> bool {
        self.find(label).is_some()
    }

    /// The list `id` is in (its nearest scroll view) and its cell there.
    fn cell(&self, mut id: u32) -> Option<(u32, u32)> {
        loop {
            let parent = *self.parents.get(&id)?;
            if self.kinds.get(&parent).is_some_and(|k| k == "Scroll") {
                return Some((parent, id));
            }
            id = parent;
        }
    }
}

/// A list of its own rows with a header, a footer, an empty view and
/// separators.
#[derive(Default)]
struct Own {
    count: usize,
    header_taps: u32,
    visible: String,
    /// Emptied once at the end.
    emptied: bool,
}

impl Component for Own {
    const NAME: &'static str = "app.own_list";
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.count = 50;
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .child(
                virtual_list(self.count, |i, _| text(format!("row {i}")).h(px(40.0)))
                    .estimated_item_size(40.0)
                    .flex_1()
                    .header(
                        text(format!("header {}", self.header_taps)).on_press(cx.listener(
                            |this: &mut Self, _, cx| {
                                this.header_taps += 1;
                                cx.notify();
                            },
                        )),
                    )
                    .footer(text("footer"))
                    .empty(
                        text("nothing").on_press(cx.listener(|this: &mut Self, _, cx| {
                            this.count = 3;
                            cx.notify();
                        })),
                    )
                    .separator(|| view().h(px(1.0)).bg(rgb(0xeeeeee)))
                    .sticky_header_indices([0])
                    .keyboard_dismiss_mode(KeyboardDismissMode::OnDrag)
                    .end_reached_threshold(0.5)
                    .initial_scroll_index(2)
                    .minimum_view_time(Duration::from_millis(100))
                    .on_viewable_items_changed(cx.listener(
                        |this: &mut Self, rows: &ViewableItems, cx| {
                            let rows = rows.indices();
                            this.visible = format!("{}-{}", rows.start(), rows.end());
                            cx.notify();
                        },
                    ))
                    .on_end_reached(cx.listener(|this: &mut Self, _, cx| {
                        if !this.emptied {
                            this.emptied = true;
                            this.count = 0;
                            cx.notify();
                        }
                    })),
            )
            .child(text(format!("visible {}", self.visible)))
    }
}

#[test]
fn rust_lists_have_headers_footers_separators_and_empty_views() {
    let (engine, rx) = start(Root::of::<Own>());
    let mut tree = Tree::default();
    let first = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    // Their own props, set as JS sets them.
    let list = first
        .items
        .iter()
        .find_map(|i| match i {
            MountItem::Create { id, kind, .. } if format!("{kind:?}") == "Scroll" => Some(*id),
            _ => None,
        })
        .expect("a list");
    tree.apply(&first);
    tree.drain(&rx);
    for label in ["header 0", "footer", "row 0", "row 1"] {
        assert!(tree.shows(label), "{label}: {:?}", tree.texts);
    }
    assert!(!tree.shows("nothing"), "the empty view waits for no rows");
    assert!(
        tree.commands
            .iter()
            .any(|(id, c)| *id == list && matches!(c, nsh_core::Command::ScrollTo { .. })),
        "the initial row is scrolled to: {:?}",
        tree.commands
    );
    // The header and the footer are cells of the list, not rows.
    let header = tree.find("header 0").unwrap();
    let (header_list, header_cell) = tree.cell(header).expect("in a cell");
    assert_eq!(header_list, list);
    // A row and its separator share the cell.
    let row = tree.find("row 0").unwrap();
    let (_, cell) = tree.cell(row).unwrap();
    assert_ne!(cell, header_cell);
    // The header is the component's: its state changes, its node stays.
    engine
        .control()
        .dispatch_event(header, EventKind::Press, EventPayload::None);
    tree.drain(&rx);
    assert_eq!(tree.find("header 1"), Some(header), "{:?}", tree.texts);
    // Rows and their separators share the cells, but the last row's.
    let children = |cell: u32| tree.parents.values().filter(|&&p| p == cell).count();
    assert_eq!(children(cell), 2, "a row and its separator");
    // Viewable rows reach the component once they stayed long enough.
    let viewable = |first: f64, last: f64| {
        engine.control().dispatch_event(
            list,
            EventKind::ViewableItemsChanged,
            EventPayload::Map(vec![
                ("first", PayloadValue::Number(first)),
                ("last", PayloadValue::Number(last)),
            ]),
        );
    };
    viewable(2.0, 9.0);
    viewable(3.0, 10.0);
    tree.drain(&rx);
    assert!(tree.shows("visible 3-10"), "{:?}", tree.texts);
    assert!(
        !tree.history.iter().any(|t| t == "visible 2-9"),
        "a range left within the minimum view time is not reported"
    );
    // The end reached empties it: the empty view shows, and is the
    // component's.
    engine
        .control()
        .dispatch_event(list, EventKind::EndReached, EventPayload::None);
    tree.drain(&rx);
    assert!(tree.shows("nothing"), "{:?}", tree.texts);
    let nothing = tree.find("nothing").unwrap();
    engine
        .control()
        .dispatch_event(nothing, EventKind::Press, EventPayload::None);
    tree.drain(&rx);
    assert!(
        tree.shows("row 2") && !tree.shows("nothing"),
        "{:?}",
        tree.texts
    );
}
