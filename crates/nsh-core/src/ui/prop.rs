//! A component's prop as the component reads it ([`Prop`]): a `Copy` handle
//! of a value or a binding in the reactive graph.

use std::{any::Any, fmt, marker::PhantomData, rc::Rc};

use super::{
    IntoReactive, Memo, Reactive, ReadSignal, Signal, computed,
    reactive::{Node, with_data},
    signal,
};

/// A prop of a component that runs once (a field of its `#[derive(Props)]`
/// struct): read it in a binding, `props.max.get()`, and the binding runs
/// again when it changes. `Copy`, like a signal: closures capture it as it
/// is.
///
/// A Rust parent passes a value, a signal, a memo or a closure (`.into()`,
/// [`Prop::bind`]); JS passes values, one prop at a time. In the
/// component's instance each prop is a memo of its own, so a new value from
/// JS or from a parent that renders again reaches only what read it.
pub struct Prop<T: 'static> {
    node: Node,
    /// Made from a value (its own signal): a parent that renders again
    /// passes its value on, not the binding.
    given_value: bool,
    _t: PhantomData<fn() -> T>,
}

impl<T> Clone for Prop<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Prop<T> {}

/// The same node.
impl<T> PartialEq for Prop<T> {
    fn eq(&self, other: &Self) -> bool {
        self.node == other.node
    }
}

impl<T> fmt::Debug for Prop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prop({:?})", self.node)
    }
}

impl<T: Clone + PartialEq + Default + 'static> Default for Prop<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Clone + PartialEq + 'static> From<T> for Prop<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl From<&'static str> for Prop<super::SharedString> {
    fn from(value: &'static str) -> Self {
        Self::new(value.into())
    }
}

impl From<String> for Prop<super::SharedString> {
    fn from(value: String) -> Self {
        Self::new(value.into())
    }
}

/// An optional text given as text (`label: "Name".into()`).
impl From<&'static str> for Prop<Option<super::SharedString>> {
    fn from(value: &'static str) -> Self {
        Self::new(Some(value.into()))
    }
}

impl From<String> for Prop<Option<super::SharedString>> {
    fn from(value: String) -> Self {
        Self::new(Some(value.into()))
    }
}

impl From<super::SharedString> for Prop<Option<super::SharedString>> {
    fn from(value: super::SharedString) -> Self {
        Self::new(Some(value))
    }
}

macro_rules! from_handles {
    ($($handle:ident),*) => {$(
        impl<T: Clone + PartialEq + 'static> From<$handle<T>> for Prop<T> {
            fn from(value: $handle<T>) -> Self {
                Self::of(value.node(), false)
            }
        }
    )*};
}

from_handles!(Signal, ReadSignal, Memo);

impl<T: Clone + PartialEq + 'static> From<Reactive<T>> for Prop<T> {
    fn from(value: Reactive<T>) -> Self {
        match value {
            Reactive::Static(v) => Self::new(v),
            Reactive::Bound(f) => Self::bind(move || f()),
        }
    }
}

impl<T: Clone + PartialEq + 'static> Prop<T> {
    /// A prop given `value` (a signal of the current owner).
    pub fn new(value: T) -> Self {
        Self::of(signal(value).node(), true)
    }

    /// A prop bound to `f` (a memo of the current owner): run again when
    /// what it reads changes.
    pub fn bind(f: impl Fn() -> T + 'static) -> Self {
        Self::of(computed(f).node(), false)
    }

    fn of(node: Node, given_value: bool) -> Self {
        Self {
            node,
            given_value,
            _t: PhantomData,
        }
    }

    /// The value, subscribing the running computation.
    #[track_caller]
    pub fn get(&self) -> T {
        self.with(T::clone)
    }

    /// `f` of the value, subscribing the running computation.
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        self.node.with::<T, R>("a prop", f)
    }

    /// The value, without subscribing.
    #[track_caller]
    pub fn get_untracked(&self) -> T {
        super::untracked(|| self.get())
    }

    /// `f` of the value, without subscribing.
    #[track_caller]
    pub fn with_untracked<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        super::untracked(|| self.with(f))
    }

    /// The prop of an instance: a memo of its own, of a value that JS or a
    /// parent rendering again replaces, or of a binding a parent passed.
    /// Made under the current owner (the instance's).
    #[doc(hidden)]
    pub fn __hold(&self) -> Self {
        let value = signal(self.get_untracked());
        let binding: Signal<Option<Prop<T>>> = signal((!self.given_value).then_some(*self));
        let held = computed(move || match binding.get() {
            Some(prop) => prop.get(),
            None => value.get(),
        });
        let node = held.node();
        with_data(|d| {
            d.held
                .insert(node, Box::new(Held { value, binding }) as Box<dyn Any>);
        });
        // The registration goes with the instance.
        super::cleanup(move || {
            with_data(|d| d.held.remove(&node));
        });
        Self::of(node, false)
    }

    /// Takes `value` (a static one, or a binding): an instance's held prop
    /// replaces what it reads, updating its readers; a prop not held yet (an
    /// instance's before its view is made) becomes it. Whether it changed.
    #[doc(hidden)]
    pub fn __set(&mut self, value: Reactive<T>) -> bool {
        match Held::<T>::of(self.node) {
            Some(held) => match value {
                Reactive::Static(v) => {
                    let changed = held.binding.with_untracked(Option::is_some)
                        || held.value.with_untracked(|old| *old != v);
                    super::batch(|| {
                        held.binding.set(None);
                        held.value.set(v);
                    });
                    changed
                }
                Reactive::Bound(f) => {
                    held.binding.set(Some(Self::bind(move || f())));
                    true
                }
            },
            None => {
                let new = Self::from(value);
                let changed = !new.given_value || self.get_untracked() != new.get_untracked();
                *self = new;
                changed
            }
        }
    }

    /// An instance's held prop takes what `new` gives: its value when it
    /// was given a value (a parent that renders again makes new props each
    /// time), else its binding.
    #[doc(hidden)]
    pub fn __update_from(&mut self, new: &Self) -> bool {
        if new.given_value {
            return self.__set(Reactive::Static(new.get_untracked()));
        }
        let same = Held::<T>::of(self.node)
            .is_some_and(|h| h.binding.with_untracked(|b| *b == Some(*new)));
        if same {
            return false;
        }
        let new = *new;
        match Held::<T>::of(self.node) {
            Some(held) => {
                held.binding.set(Some(new));
                true
            }
            None => {
                *self = new;
                true
            }
        }
    }
}

impl Prop<Option<super::Slot>> {
    /// The slot where the app fills it, styled by `style`: it comes and
    /// goes as the app fills and empties it.
    pub fn place(
        self,
        style: impl Fn(super::SlotElement) -> super::SlotElement + 'static,
    ) -> super::AnyElement {
        super::dynamic(move || self.get().map(|slot| style(slot.element())))
    }
}

/// What a held prop's memo reads: a value and, when a parent passed one, a
/// binding.
struct Held<T: 'static> {
    value: Signal<T>,
    binding: Signal<Option<Prop<T>>>,
}

impl<T: 'static> Held<T> {
    fn of(node: Node) -> Option<Self> {
        with_data(|d| {
            d.held
                .get(&node)
                .and_then(|h| h.downcast_ref::<Held<T>>())
                .map(|h| Held {
                    value: h.value,
                    binding: h.binding,
                })
        })
    }
}

/// A prop passed on, to an element's setter or a child's prop: bound to it
/// (`text(props.title)`).
impl<T: Clone + PartialEq + 'static> IntoReactive<T> for Prop<T> {
    fn into_reactive(self) -> Reactive<T> {
        Reactive::Bound(Rc::new(move || self.get()))
    }
}
