//! A Rust app: the engine mounts its root component with no bundle and no
//! VM; it renders, handles presses and runs its tasks.

use std::time::Duration;

use crossbeam_channel::Receiver;
use nsh_core::{
    ApproxTextEngine, EventKind, EventPayload, MountBatch, MountItem, Size,
    ui::{Root, prelude::*},
};
use nsh_engine::{App, Engine, EngineConfig, Environment};

use crate::ChannelSink;

#[derive(Default)]
struct Counter {
    count: u32,
    ticks: u32,
    _timer: Option<Task<()>>,
}

#[derive(Clone, PartialEq, Props)]
struct CounterProps {
    #[prop(default = "Count".into())]
    label: SharedString,
}

impl Component for Counter {
    type Props = CounterProps;
    type Event = NoEvent;

    fn setup(&mut self, _: &CounterProps, cx: &mut Context<'_, Self>) {
        self._timer = Some(
            cx.set_timeout(Duration::from_millis(10), |this: &mut Self, cx| {
                this.ticks += 1;
                cx.notify();
            }),
        );
    }

    fn render(&self, props: &CounterProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .on_press(cx.listener(|this: &mut Self, _, cx| {
                this.count += 1;
                cx.notify();
            }))
            .child(text(format!(
                "{} {} ({} ticks)",
                props.label, self.count, self.ticks
            )))
    }
}

fn start() -> (Engine, Receiver<MountBatch>) {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::new::<Counter>(|| CounterProps {
            label: "Taps".into(),
        })),
        viewport: Size::new(390.0, 844.0),
        trace: false,
        network: None,
        assets: None,
        environment: Environment::default(),
        header_heights: (44.0, 96.0),
        limits: nsh_engine::Limits::default(),
        // Nothing registered: Rust renders its components by type.
        ui_sets: Vec::new(),
        runtime: None,
    };
    (
        Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap(),
        rx,
    )
}

/// The text shown after the batches received within `wait`, and the view
/// it is in.
fn shown(rx: &Receiver<MountBatch>, wait: Duration, until: &str) -> Option<(String, u32)> {
    let deadline = std::time::Instant::now() + wait;
    let mut last = None;
    let mut host = None;
    while let Ok(batch) = rx.recv_deadline(deadline) {
        for item in &batch.items {
            match item {
                MountItem::Text { text, .. } => last = Some(text.text.clone()),
                MountItem::Insert {
                    parent: nsh_core::ROOT_ID,
                    child,
                    ..
                } => host = Some(*child),
                _ => {}
            }
        }
        if last.as_deref() == Some(until) {
            break;
        }
    }
    Some((last?, host.unwrap_or(0)))
}

#[test]
fn a_rust_app_renders_handles_presses_and_runs_tasks() {
    let (engine, rx) = start();
    let (text, host) = shown(&rx, Duration::from_secs(5), "Taps 0 (0 ticks)").unwrap();
    assert_eq!(text, "Taps 0 (0 ticks)");
    assert_ne!(host, 0, "the root component is in the root view");
    // The timer task wakes the engine and renders again.
    let (text, _) = shown(&rx, Duration::from_secs(5), "Taps 0 (1 ticks)").unwrap();
    assert_eq!(text, "Taps 0 (1 ticks)");
    engine
        .control()
        .dispatch_event(host, EventKind::Press, EventPayload::None);
    let (text, _) = shown(&rx, Duration::from_secs(5), "Taps 1 (1 ticks)").unwrap();
    assert_eq!(text, "Taps 1 (1 ticks)");
}

/// A Rust navigation stack: a screen per route, keyed by it.
#[derive(Default)]
struct Navigation {
    routes: Vec<u32>,
}

impl Component for Navigation {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), _: &mut Context<'_, Self>) {
        self.routes = vec![0, 1];
    }

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        // A component's root is a view (its host node).
        view()
            .flex_1()
            .child(stack().flex_1().children(self.routes.iter().map(|&route| {
                screen()
                    .key(route.to_string())
                    .title(format!("page {route}"))
                    .header_right(text("more"))
                    .on_dismiss(cx.listener(move |this: &mut Self, _, cx| {
                        this.routes.retain(|r| *r < route);
                        cx.notify();
                    }))
                    .child(text(format!("content {route}")))
            })))
    }
}

#[test]
fn rust_components_navigate_with_native_stacks() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<Navigation>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let first = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let screens: Vec<(u32, String, Option<u32>)> = first
        .items
        .iter()
        .filter_map(|i| match i {
            MountItem::ScreenConfig { id, config } => Some((
                *id,
                config.title.as_deref().unwrap_or_default().to_owned(),
                config.right,
            )),
            _ => None,
        })
        .collect();
    let titles: Vec<&str> = screens.iter().map(|s| s.1.as_str()).collect();
    assert!(
        titles.contains(&"page 0") && titles.contains(&"page 1"),
        "{titles:?}"
    );
    assert!(
        screens.iter().all(|s| s.2.is_some()),
        "a header item each: {screens:?}"
    );
    let kinds: Vec<_> = first
        .items
        .iter()
        .filter_map(|i| match i {
            MountItem::Create { kind, .. } => Some(format!("{kind:?}")),
            _ => None,
        })
        .collect();
    assert!(kinds.iter().any(|k| k == "Stack"), "{kinds:?}");
    // The user pops the top screen natively: its route goes.
    let top = screens.iter().find(|s| s.1 == "page 1").unwrap().0;
    engine
        .control()
        .dispatch_event(top, EventKind::Dismiss, EventPayload::None);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut deleted = false;
    while let Ok(batch) = rx.recv_deadline(deadline) {
        if batch
            .items
            .iter()
            .any(|i| matches!(i, MountItem::Delete { id } if *id == top))
        {
            deleted = true;
            break;
        }
    }
    assert!(deleted, "the popped screen goes");
}

/// Rust content in a Rust component of the mobile UI: a card's header
/// and body.
#[derive(Default)]
struct WithCard {
    taps: u32,
}

impl Component for WithCard {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view().flex_1().child(
            nsh_ui_mobile::Card::element(nsh_ui_mobile::CardProps::default())
                .fill("header", text("card header"))
                .child(
                    text(format!("card body {}", self.taps)).on_press(cx.listener(
                        |this: &mut Self, _, cx| {
                            this.taps += 1;
                            cx.notify();
                        },
                    )),
                ),
        )
    }
}

#[test]
fn rust_components_pass_content_to_rust_components() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<WithCard>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let first = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let texts: Vec<(u32, String)> = first
        .items
        .iter()
        .filter_map(|i| match i {
            MountItem::Text { id, text, .. } => Some((*id, text.text.clone())),
            _ => None,
        })
        .collect();
    let names: Vec<&str> = texts.iter().map(|t| t.1.as_str()).collect();
    assert!(
        names.contains(&"card header") && names.contains(&"card body 0"),
        "{names:?}"
    );
    // Both are in the tree (placed by the card), not left aside.
    let parents: std::collections::HashMap<u32, u32> = first
        .items
        .iter()
        .filter_map(|i| match i {
            MountItem::Insert { parent, child, .. } => Some((*child, *parent)),
            _ => None,
        })
        .collect();
    let attached = |mut id: u32| loop {
        if id == nsh_core::ROOT_ID {
            return true;
        }
        match parents.get(&id) {
            Some(&p) => id = p,
            None => return false,
        }
    };
    for (id, name) in &texts {
        assert!(attached(*id), "{name} is in the tree");
    }
    // Where the card's render places them: the header in its slot's
    // holder, the body in the card's padded box (not in the host).
    let id_of = |name: &str| texts.iter().find(|t| t.1 == name).unwrap().0;
    let card = parents[&parents[&id_of("card header")]];
    let body_box = parents[&id_of("card body 0")];
    assert_ne!(body_box, card, "the body is in the card's box");
    assert_eq!(parents[&body_box], card);
    // The body renders again with the parent's state.
    let body = texts.iter().find(|t| t.1 == "card body 0").unwrap().0;
    engine
        .control()
        .dispatch_event(body, EventKind::Press, EventPayload::None);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let mut updated = false;
    while let Ok(batch) = rx.recv_deadline(deadline) {
        if batch.items.iter().any(|i| {
            matches!(i, MountItem::Text { id, text, .. } if *id == body && text.text == "card body 1")
        }) {
            updated = true;
            break;
        }
    }
    assert!(updated, "the same text node shows the new count");
}

/// A Rust component takes the Android back button while it listens; when
/// it no longer handles it, the platform goes back.
#[derive(Default)]
struct BackCounter {
    presses: u32,
    _back: Option<BackSubscription>,
}

impl Component for BackCounter {
    type Props = ();
    type Event = NoEvent;

    fn setup(&mut self, _: &(), cx: &mut Context<'_, Self>) {
        self._back = Some(cx.on_back_press(|this: &mut Self, cx| {
            this.presses += 1;
            cx.notify();
            this.presses < 2
        }));
    }

    fn render(&self, _: &(), _: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .child(text(format!("presses {}", self.presses)))
    }
}

#[test]
fn rust_components_take_the_back_button() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<BackCounter>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let first = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(
        first.items.iter().any(|i| matches!(
            i,
            MountItem::Events { id: nsh_core::ROOT_ID, events } if events.contains(EventKind::BackPress)
        )),
        "the root wants the back button"
    );
    let press = |until: &dyn Fn(&MountItem) -> bool| {
        engine.control().dispatch_event(
            nsh_core::ROOT_ID,
            EventKind::BackPress,
            EventPayload::None,
        );
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while let Ok(batch) = rx.recv_deadline(deadline) {
            if batch.items.iter().any(until) {
                return true;
            }
        }
        false
    };
    // Handled: it does not go back.
    assert!(press(
        &|i| matches!(i, MountItem::Text { text, .. } if text.text == "presses 1")
    ));
    // Not handled, and JS does not listen: the platform goes back.
    assert!(press(&|i| matches!(
        i,
        MountItem::Command {
            command: nsh_core::Command::Back,
            ..
        }
    )));
}

/// The core's events (a layout) and throttled scroll events reach a Rust
/// component with no JS.
#[derive(Default)]
struct Measured {
    width: f32,
    scrolled: f32,
}

impl Component for Measured {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .on_layout(cx.listener(|this: &mut Self, frame: &nsh_core::Rect, cx| {
                this.width = frame.width;
                cx.notify();
            }))
            .child(text(format!(
                "width {} scrolled {}",
                self.width, self.scrolled
            )))
            .child(
                scroll()
                    .flex_1()
                    .on_scroll(cx.listener(|this: &mut Self, e: &ScrollEvent, cx| {
                        this.scrolled = e.y;
                        cx.notify();
                    }))
                    .child(view().h(px(2000.0))),
            )
    }
}

#[test]
fn rust_components_hear_layouts_and_scrolls() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<Measured>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    // Until `until` shows; the scroll view's id on the way.
    let wait = |until: &str, scroll: &mut Option<u32>| {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while let Ok(batch) = rx.recv_deadline(deadline) {
            let mut done = false;
            for item in &batch.items {
                match item {
                    MountItem::Create {
                        id,
                        kind: nsh_core::ViewKind::Scroll,
                    } => *scroll = Some(*id),
                    MountItem::Text { text, .. } if text.text == until => done = true,
                    _ => {}
                }
            }
            if done {
                return true;
            }
        }
        false
    };
    let mut scroll = None;
    assert!(wait("width 390 scrolled 0", &mut scroll), "the layout came");
    let scroll = scroll.expect("a scroll view");
    engine.control().scroll(scroll, 0.0, 120.0, 0);
    assert!(wait("width 390 scrolled 120", &mut None), "the scroll came");
}

/// A nested component's events in a row of a component's own list reach
/// the component whose list it is.
#[derive(Default)]
struct ButtonRows {
    pressed: Option<usize>,
}

impl Component for ButtonRows {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let press = cx.listener(|this: &mut Self, row: &usize, cx| {
            this.pressed = Some(*row);
            cx.notify();
        });
        view()
            .flex_1()
            .child(text(format!("pressed {:?}", self.pressed)))
            .child(
                virtual_list(20, move |i, _| {
                    nsh_ui_mobile::Button::element(nsh_ui_mobile::ButtonProps {
                        title: format!("row {i}").into(),
                        ..Default::default()
                    })
                    .on_event(press.clone().map(move |_: &nsh_ui_mobile::ButtonEvent| i))
                })
                .flex_1(),
            )
    }
}

#[test]
fn nested_components_in_list_rows_reach_the_list_owner() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<ButtonRows>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let mut parents = std::collections::HashMap::new();
    let mut texts = Vec::new();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while let Ok(batch) = rx.recv_deadline(deadline) {
        for item in &batch.items {
            match item {
                MountItem::Insert { parent, child, .. } => {
                    parents.insert(*child, *parent);
                }
                MountItem::Text { id, text, .. } => texts.push((*id, text.text.clone())),
                _ => {}
            }
        }
        if texts.iter().any(|t| t.1 == "row 3") {
            break;
        }
    }
    let label = texts.iter().find(|t| t.1 == "row 3").expect("row 3").0;
    // The button's root (its pressable) holds the label.
    engine
        .control()
        .dispatch_event(parents[&label], EventKind::Press, EventPayload::None);
    let (text, _) = shown(&rx, Duration::from_secs(5), "pressed Some(3)").unwrap();
    assert_eq!(text, "pressed Some(3)");
}

/// Content a nested component does not place (a closed sheet's) is not on
/// screen; it is once the component places it.
#[derive(Default)]
struct WithSheet {
    open: bool,
}

impl Component for WithSheet {
    type Props = ();
    type Event = NoEvent;

    fn render(&self, _: &(), cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        view()
            .flex_1()
            .child(text("open").on_press(cx.listener(|this: &mut Self, _, cx| {
                this.open = true;
                cx.notify();
            })))
            .child(
                nsh_ui_mobile::Sheet::element(nsh_ui_mobile::SheetProps {
                    open: self.open,
                    ..Default::default()
                })
                .child(text("sheet body")),
            )
    }
}

#[test]
fn content_a_component_does_not_place_is_not_shown() {
    let (tx, rx) = crossbeam_channel::unbounded();
    let config = EngineConfig {
        app: App::Rust(Root::of::<WithSheet>()),
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
    let engine = Engine::start(config, || ApproxTextEngine, ChannelSink(tx)).unwrap();
    let mut parents: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut texts = Vec::new();
    let mut collect = |wait: Duration| {
        let deadline = std::time::Instant::now() + wait;
        while let Ok(batch) = rx.recv_deadline(deadline) {
            for item in &batch.items {
                match item {
                    MountItem::Insert { parent, child, .. } => {
                        parents.insert(*child, *parent);
                    }
                    MountItem::Remove { id } | MountItem::Delete { id } => {
                        parents.remove(id);
                    }
                    MountItem::Text { id, text, .. } => texts.push((*id, text.text.clone())),
                    _ => {}
                }
            }
        }
        (parents.clone(), texts.clone())
    };
    let attached = |parents: &std::collections::HashMap<u32, u32>, mut id: u32| loop {
        if id == nsh_core::ROOT_ID {
            return true;
        }
        match parents.get(&id) {
            Some(&p) => id = p,
            None => return false,
        }
    };
    let (p, t) = collect(Duration::from_millis(300));
    let body = t.iter().find(|t| t.1 == "sheet body").map(|t| t.0);
    assert!(
        body.is_none_or(|b| !attached(&p, b)),
        "a closed sheet shows nothing"
    );
    let open = t.iter().find(|t| t.1 == "open").unwrap().0;
    engine
        .control()
        .dispatch_event(open, EventKind::Press, EventPayload::None);
    let (p, t) = collect(Duration::from_millis(300));
    let body = t.iter().find(|t| t.1 == "sheet body").unwrap().0;
    assert!(attached(&p, body), "the open sheet shows its content");
}
