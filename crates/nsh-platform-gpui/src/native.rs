//! Web views, videos and native components. (Being ported: see
//! `nsh-platform-windows/src/native.rs`.)

use std::{cell::RefCell, rc::Rc, sync::Arc};

use nsh_core::{Command, NativeProps, NodeId};
use rustc_hash::FxHashMap;
use serde_json::Value as JsonValue;

use crate::{image::AssetBase, surface::Surface, view::NshView};

/// A native component an app registers (`nsh/native`).
pub trait NativeComponent {
    /// A prop changed (`None`: removed).
    fn set_prop(&mut self, name: &str, value: Option<&JsonValue>) {
        let _ = (name, value);
    }
}

/// What a component can do: emit events, report its size.
pub struct ComponentContext {
    /// Its node.
    pub id: NodeId,
}

/// Makes a component for an element.
pub type ComponentFactory = Rc<dyn Fn(ComponentContext) -> Box<dyn NativeComponent>>;

/// The components registered with a view.
#[derive(Clone, Default)]
pub struct Components {
    factories: Rc<RefCell<FxHashMap<Arc<str>, ComponentFactory>>>,
}

impl std::fmt::Debug for Components {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.factories.borrow().keys())
            .finish()
    }
}

impl Components {
    /// Registers component `name`.
    pub fn register(&self, name: &str, factory: ComponentFactory) {
        self.factories.borrow_mut().insert(Arc::from(name), factory);
    }
}

/// Native content of a surface.
#[derive(Default)]
pub(crate) struct NativeState {
    components: RefCell<Components>,
}

impl NativeState {
    pub(crate) fn set_components(&self, components: Components) {
        *self.components.borrow_mut() = components;
    }
    pub(crate) fn components(&self) -> Components {
        self.components.borrow().clone()
    }
    pub(crate) fn set_asset_base(&self, _base: Option<AssetBase>) {}
    pub(crate) fn clear(&self) {}
}

/// A web view, video or native view was made.
pub(crate) fn create(_surface: &Rc<Surface>, _view: &NshView) {}
/// A view was deleted.
pub(crate) fn deleted(_surface: &Rc<Surface>, _view: &NshView) {}
/// A view was resized.
pub(crate) fn resized(_view: &NshView) {}
/// `NativeProps`.
pub(crate) fn props(_surface: &Rc<Surface>, _view: &NshView, _props: &NativeProps) {}
/// `Component`.
pub(crate) fn component(_surface: &Rc<Surface>, _view: &NshView, _name: &Arc<str>) {}
/// `ComponentProps`.
pub(crate) fn component_props(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _props: &[(Arc<str>, Option<JsonValue>)],
) {
}
/// Web view, video and component commands.
pub(crate) fn command(_surface: &Rc<Surface>, _view: &NshView, _command: &Command) {}
