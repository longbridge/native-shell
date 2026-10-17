//! Components that run once ([`super::Component::view`]): their instance
//! and their [`Scope`].
//!
//! The instance makes the component's view once, under an owner of its
//! own, with its props held in signals: a prop from JS or from a parent that
//! renders again updates what read it. The core mounts the view with its
//! bindings and never asks for it again; releasing the host disposes the
//! owner, and with it everything the component made.

use std::{any::Any, marker::PhantomData};

use super::{
    Component, ComponentError, Environment, Event, IntoElement, Owner, PropError, Props,
    element::{AnyEvent, ElNode, ErasedCx, RenderReads, RenderScope},
};
use crate::{JsonValue, NodeId};

/// What a component that runs once has of its own: it emits its events
/// (TS: calling an `on…` prop). `Copy`: closures capture it as it is.
pub struct Scope<E> {
    host: NodeId,
    _event: PhantomData<fn(E)>,
}

impl<E> Clone for Scope<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for Scope<E> {}

impl<E> std::fmt::Debug for Scope<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Scope({})", self.host)
    }
}

impl<E: Event> Scope<E> {
    /// Sends `event` to the app (its `on<Event>` handler), or to the
    /// component that rendered this one.
    pub fn emit(&self, event: E) {
        super::reactive::emit(self.host, Box::new(event));
    }

    /// Makes `api` what the component's ref does in JS (its methods) for as
    /// long as the view lives; the component names it in its spec
    /// (`#[component(methods = StepperApi)]`).
    pub fn methods(&self, api: impl super::Api) {
        super::reactive::register_api(self.host, api);
    }

    /// The edges system UI or a header covers where the component is: its
    /// screen's (below the header) or modal's, else the safe area (TS:
    /// `useSafeAreaInsets()`). Reading subscribes.
    pub fn insets(&self) -> crate::EdgeInsets {
        super::ambient::insets_of(self.host)
    }
}

/// The instance of a component that runs once.
pub(crate) struct OnceInstance<C: Component> {
    /// The props: as set before the view is made, held afterwards.
    props: C::Props,
    /// What owns the props set before the view is made (they go once the
    /// view holds their values).
    initial: Option<Owner>,
    /// The component's owner, once its view is made.
    owner: Option<Owner>,
    /// Where it was made (its context comes from there).
    parent: Owner,
    host: NodeId,
    _component: PhantomData<fn() -> C>,
}

impl<C: Component> OnceInstance<C> {
    pub(crate) fn new() -> Self {
        let (initial, props) = super::create_root(C::Props::initial);
        Self {
            props,
            initial: Some(initial),
            owner: None,
            parent: Owner::detached(),
            host: 0,
            _component: PhantomData,
        }
    }
}

impl<C: Component> super::component::AnyInstance for OnceInstance<C> {
    fn set_prop(&mut self, name: &str, value: Option<JsonValue>) -> Result<bool, PropError> {
        match self.initial {
            Some(initial) => initial.run(|| self.props.set(name, value)),
            None => self.props.set(name, value),
        }
    }

    fn set_props(&mut self, props: &dyn Any) -> bool {
        match props.downcast_ref::<C::Props>() {
            // Before the view: the props it starts with; after: held props
            // update their readers.
            Some(p) if self.owner.is_none() => {
                self.props = p.clone();
                false
            }
            Some(p) => {
                let changed = self.props.update_from(p);
                if changed {
                    log::warn!(
                        target: "js",
                        "{}: a prop that is not a `Prop` changed; it runs once and keeps the first",
                        super::component::name_of::<C>()
                    );
                }
                false
            }
            None => false,
        }
    }

    fn set_slot(&mut self, name: &'static str, filled: bool) -> bool {
        self.props
            .set_slot(name, filled.then(|| super::Slot::new(name)))
    }

    fn set_children(&mut self, present: bool) -> bool {
        self.props.set_children(super::Children::new(present))
    }

    fn update(&mut self, _: &mut ErasedCx) {}

    fn cleanup(&mut self, _: &mut ErasedCx) {
        for owner in [self.owner.take(), self.initial.take()]
            .into_iter()
            .flatten()
        {
            owner.dispose();
        }
    }

    fn environment_changed(&mut self, _: &Environment, _: &Environment, _: &mut ErasedCx) {}

    fn render(&mut self, _: &RenderScope<'_>) -> (ElNode, RenderReads) {
        if self.owner.is_some() {
            log::error!(target: "js", "a component that runs once was asked for its view again");
            return (ElNode::empty_view(), RenderReads::default());
        }
        let scope = Scope {
            host: self.host,
            _event: PhantomData,
        };
        let props = &mut self.props;
        let (owner, node) = self.parent.run(|| {
            super::create_root(|| {
                let held = props.held();
                *props = held.clone();
                C::view(held, scope).into_element().0
            })
        });
        self.owner = Some(owner);
        if let Some(initial) = self.initial.take() {
            initial.dispose();
        }
        (node, RenderReads::default())
    }

    fn catch_error(&mut self, _: &ComponentError, _: &mut ErasedCx) -> bool {
        false
    }

    fn name(&self) -> &'static str {
        super::component::name_of::<C>()
    }

    fn parts(&mut self) -> (&mut dyn Any, &dyn Any) {
        (&mut self.host, &self.props)
    }

    fn call(
        &mut self,
        name: &str,
        args: Vec<JsonValue>,
        _: &mut ErasedCx,
    ) -> Option<Result<JsonValue, PropError>> {
        super::reactive::call_api(self.host, name, args)
    }

    fn adopt(&mut self, parent: Owner) {
        if self.owner.is_none() && !self.parent.is_live() {
            self.parent = parent;
        }
    }

    fn failure_owner(&self) -> Option<Owner> {
        Some(self.owner.unwrap_or(self.parent))
    }

    fn runs_once(&self) -> Option<Owner> {
        Some(self.owner.unwrap_or_else(Owner::detached))
    }

    fn place(&mut self, host: NodeId, parent: Owner) {
        self.host = host;
        self.parent = parent;
    }
}

/// An event a component that runs once emitted, for the core.
pub(crate) type Emitted = (NodeId, Box<dyn AnyEvent>);
