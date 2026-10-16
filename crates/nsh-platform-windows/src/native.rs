//! Video, web views and the native components apps register.

use std::{cell::RefCell, rc::Rc, sync::Arc};

use nsh_core::{Command, EventKind, EventPayload, NativeProps, NodeId, Size};
use nsh_engine::EngineControl;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value as JsonValue;

use crate::{image::AssetBase, surface::Surface, view::NshView};

/// A native component: something the embedding app draws in a node's
/// view (`nativeComponent(name, ...)` in JS).
pub trait NativeComponent {
    /// Draws the component in its view's bounds (`size` points; the context
    /// is in the view's coordinates).
    fn draw(&self, dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext, size: (f32, f32)) {
        let _ = (dc, size);
    }

    /// Prop `name` changed (`None`: removed). Every prop the app set comes
    /// right after the component is made.
    fn set_prop(&mut self, name: &str, value: Option<&JsonValue>);

    /// JS called `ref.command(name, args)`.
    fn command(&mut self, name: &str, args: &JsonValue) {
        let _ = (name, args);
    }
}

/// What a component can tell its app. Cheap to clone; usable from any
/// thread.
#[derive(Clone)]
pub struct ComponentContext {
    id: NodeId,
    control: EngineControl,
}

impl std::fmt::Debug for ComponentContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComponentContext")
            .field("id", &self.id)
            .finish()
    }
}

impl ComponentContext {
    /// Emits event `name` with `data`: the handler `on<Name>` gets it.
    pub fn emit(&self, name: &str, data: JsonValue) {
        self.control.dispatch_event(
            self.id,
            EventKind::NativeEvent,
            EventPayload::Native {
                name: name.into(),
                data,
            },
        );
    }

    /// The component's own size (points): used where its style leaves a
    /// dimension open.
    pub fn set_size(&self, width: f32, height: f32) {
        self.control
            .set_component_size(self.id, Size::new(width, height));
    }
}

// Components may emit from any thread (their own callbacks, workers).
const _: () = {
    const fn send_sync<T: Send + Sync>() {}
    send_sync::<ComponentContext>();
};

/// Makes a component for an element, given its context.
pub type ComponentFactory = Rc<dyn Fn(ComponentContext) -> Box<dyn NativeComponent>>;

/// The components registered with a view (shared by its clones, so
/// registrations apply to elements created from then on).
#[derive(Clone, Default)]
pub struct Components {
    factories: Rc<RefCell<FxHashMap<Arc<str>, ComponentFactory>>>,
    /// Names already reported as unknown.
    missing: Rc<RefCell<FxHashSet<Arc<str>>>>,
}

impl std::fmt::Debug for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(self.factories.borrow().keys())
            .finish()
    }
}

impl Components {
    /// Registers `factory` under `name` (replacing an earlier one).
    pub fn register(&self, name: &str, factory: ComponentFactory) {
        self.factories.borrow_mut().insert(name.into(), factory);
        self.missing.borrow_mut().remove(name);
    }
}

/// The native controls of one surface.
#[derive(Default)]
pub(crate) struct NativeState {
    /// The components the app can use.
    components: RefCell<Components>,
    /// Where `asset:/` video sources are.
    asset_base: RefCell<Option<AssetBase>>,
}

impl NativeState {
    /// Uses `components` (an [`crate::AppOptions`]'s registry).
    pub(crate) fn set_components(&self, components: Components) {
        *self.components.borrow_mut() = components;
    }

    /// The registry, to register in.
    pub(crate) fn components(&self) -> Components {
        self.components.borrow().clone()
    }

    /// Where `asset:/` sources resolve from now on.
    pub(crate) fn set_asset_base(&self, base: Option<AssetBase>) {
        *self.asset_base.borrow_mut() = base;
    }

    /// Drops every control (the app stopped): media stops playing.
    pub(crate) fn clear(&self) {}
}

/// A web view, video or native component view was made.
pub(crate) fn create(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// Its node was deleted.
pub(crate) fn deleted(surface: &Rc<Surface>, view: &NshView) {
    let _ = (surface, view);
}

/// `NativeProps`.
pub(crate) fn props(surface: &Rc<Surface>, view: &NshView, props: &NativeProps) {
    let _ = (surface, view, props);
}

/// `Component`.
pub(crate) fn component(surface: &Rc<Surface>, view: &NshView, name: &Arc<str>) {
    let _ = (surface, view, name);
}

/// `ComponentProps`.
pub(crate) fn component_props(
    surface: &Rc<Surface>,
    view: &NshView,
    props: &[(Arc<str>, Option<JsonValue>)],
) {
    let _ = (surface, view, props);
}

/// A command for a web view, video or component.
pub(crate) fn command(surface: &Rc<Surface>, view: &NshView, command: &Command) {
    let _ = (surface, view, command);
}
