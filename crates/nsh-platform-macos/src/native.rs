//! Native views: videos (AVFoundation), web views (WebKit) and the
//! components apps register ([`Components`]).

use std::{cell::RefCell, rc::Rc, sync::Arc};

use nsh_core::{Command, EventKind, EventPayload, JsonValue, NativeProps, NodeId, Size};
use nsh_engine::EngineControl;
use objc2::rc::Retained;
use objc2_app_kit::NSView;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{image::AssetBase, surface::Surface, view::NshView};

/// A native component made by a registered factory.
///
/// ```no_run
/// use nsh_platform_macos::{
///     NativeComponent,
///     objc2::{MainThreadMarker, rc::Retained},
///     objc2_app_kit::{NSTextField, NSView},
///     objc2_foundation::NSString,
/// };
///
/// /// A label the app shows with a `text` prop.
/// struct Badge(Retained<NSTextField>);
///
/// impl Badge {
///     fn new(mtm: MainThreadMarker) -> Self {
///         Badge(NSTextField::labelWithString(&NSString::new(), mtm))
///     }
/// }
///
/// impl NativeComponent for Badge {
///     fn view(&self) -> Retained<NSView> {
///         Retained::into_super(Retained::into_super(self.0.clone()))
///     }
///
///     fn set_prop(&mut self, name: &str, value: Option<&serde_json::Value>) {
///         if name == "text" {
///             let text = value.and_then(|v| v.as_str()).unwrap_or("");
///             self.0.setStringValue(&NSString::from_str(text));
///         }
///     }
/// }
/// ```
pub trait NativeComponent {
    /// Its view; it fills the container nsh lays out.
    fn view(&self) -> Retained<NSView>;

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

    /// The factory registered as `name`; `None` (logged once per name) if
    /// there is none.
    fn factory(&self, name: &Arc<str>) -> Option<ComponentFactory> {
        let factory = self.factories.borrow().get(name).cloned();
        if factory.is_none() && self.missing.borrow_mut().insert(name.clone()) {
            log::warn!(
                target: "js",
                "no native component is registered as `{name}`: it renders an empty view"
            );
        }
        factory
    }
}

/// The native controls of one surface.
#[derive(Default)]
pub(crate) struct NativeState {
    /// The components the app can use.
    components: RefCell<Components>,
    /// Where `asset:/` video sources are.
    asset_base: RefCell<Option<AssetBase>>,
    /// The controls, by node.
    hosts: RefCell<FxHashMap<NodeId, Host>>,
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
    pub(crate) fn clear(&self) {
        let hosts = std::mem::take(&mut *self.hosts.borrow_mut());
        drop(hosts);
    }
}

/// Sets up a web view, video or component container.
pub(crate) fn create(_surface: &Rc<Surface>, _view: &NshView) {}

/// A view was deleted.
pub(crate) fn deleted(surface: &Rc<Surface>, view: &NshView) {
    surface.native.hosts.borrow_mut().remove(&view.id());
}

/// A video's or web view's props.
pub(crate) fn props(_surface: &Rc<Surface>, _view: &NshView, _props: &NativeProps) {}

/// A component container got its component's name.
pub(crate) fn component(_surface: &Rc<Surface>, _view: &NshView, _name: &Arc<str>) {}

/// A component's props changed.
pub(crate) fn component_props(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _props: &[(Arc<str>, Option<JsonValue>)],
) {
}

/// Web view, video and component commands.
pub(crate) fn command(_surface: &Rc<Surface>, _view: &NshView, _command: &Command) {}

/// What a native view holds.
enum Host {
    /// A registered component; `None` until named, or when unknown.
    #[allow(dead_code)]
    Component(Option<Box<dyn NativeComponent>>),
}
