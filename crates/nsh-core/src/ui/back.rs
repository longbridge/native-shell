//! The Android back button for Rust components ([`super::Context::on_back_press`];
//! TS: `BackHandler.addEventListener("hardwareBackPress", ...)`).
//!
//! The newest modal takes the button first, natively. Without one, the
//! root's `BackPress` event comes when the app listens: the components'
//! listeners run newest first until one returns true; then JS's, if it
//! listens. When no one handles it, the core sends [`crate::Command::Back`]:
//! the innermost stack pops (or asks, with `preventBack`), else the app
//! exits. The root listens while a listener of either side exists, so with
//! none the platform goes back on the spot, as `Command::Back` would.

use std::{any::Any, cell::RefCell, rc::Rc};

use super::element::ErasedCx;
use crate::NodeId;

/// A back listener with the component's type erased: it gets the
/// component's state and props, and returns whether it handled the press.
pub(crate) type BackListener = Rc<dyn Fn(&mut dyn Any, &dyn Any, &mut ErasedCx) -> bool>;

/// The components' back listeners, in the order they were added.
#[derive(Default)]
pub(crate) struct BackListeners {
    next: u64,
    pub(crate) list: Vec<(u64, NodeId, BackListener)>,
    /// The list went from empty to not or back since the core last looked.
    pub(crate) changed: bool,
}

impl BackListeners {
    pub(crate) fn add(&mut self, host: NodeId, listener: BackListener) -> u64 {
        self.next += 1;
        self.changed |= self.list.is_empty();
        self.list.push((self.next, host, listener));
        self.next
    }

    /// Removes listener `id`; returns it so it is dropped outside the
    /// borrow (it may own what removes others).
    pub(crate) fn remove(&mut self, id: u64) -> Option<BackListener> {
        let i = self.list.iter().position(|(i, ..)| *i == id)?;
        let (_, _, listener) = self.list.remove(i);
        self.changed |= self.list.is_empty();
        Some(listener)
    }

    /// Removes the listeners of component `host`, which is gone.
    pub(crate) fn remove_host(&mut self, host: NodeId) -> Vec<BackListener> {
        let mut gone = Vec::new();
        self.list.retain(|(_, h, l)| {
            let keep = *h != host;
            if !keep {
                gone.push(l.clone());
            }
            keep
        });
        self.changed |= !gone.is_empty() && self.list.is_empty();
        gone
    }

    /// The listeners newest first, with their components.
    pub(crate) fn newest_first(&self) -> Vec<(NodeId, BackListener)> {
        self.list
            .iter()
            .rev()
            .map(|(_, h, l)| (*h, l.clone()))
            .collect()
    }
}

/// A listener of the back button from [`super::Context::on_back_press`]
/// (TS: the `{ remove }` `BackHandler.addEventListener` returns): keep it in
/// a field while the component wants the button; dropping it removes the
/// listener. A component's listeners also end with it.
#[must_use = "a dropped subscription removes its listener: keep it in a field"]
pub struct BackSubscription {
    listeners: std::rc::Weak<RefCell<BackListeners>>,
    id: u64,
}

impl BackSubscription {
    pub(crate) fn new(listeners: &Rc<RefCell<BackListeners>>, id: u64) -> Self {
        Self {
            listeners: Rc::downgrade(listeners),
            id,
        }
    }
}

impl Drop for BackSubscription {
    fn drop(&mut self) {
        if let Some(listeners) = self.listeners.upgrade() {
            let removed = listeners.borrow_mut().remove(self.id);
            drop(removed);
        }
    }
}
