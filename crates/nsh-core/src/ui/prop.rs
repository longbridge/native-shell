//! A component's prop as the component reads it ([`Prop`]): what the app or
//! a Rust parent gave, a value or a binding.

use std::{fmt, rc::Rc};

use super::{IntoReactive, Memo, Reactive, ReadSignal, Signal, SignalEquals, signal_with};

/// A prop of a component that runs once (a field of its `#[derive(Props)]`
/// struct): read it in a binding, `props.max.get()`, and the binding runs
/// again when it changes.
///
/// A Rust parent passes a value, a signal, a memo or a closure (`busy.into()`,
/// `(move || count.get() > 9).into()`); JS passes values, one prop at a time.
/// In the component's instance each prop is held in a signal of its own, so
/// a new value from JS or from a parent that renders again reaches only what
/// read that prop.
pub struct Prop<T: 'static>(Inner<T>);

enum Inner<T: 'static> {
    /// What a parent gave.
    Given(Reactive<T>),
    /// The instance's: what it was given, held to be replaced.
    Held(Signal<Held<T>>),
}

/// What a held prop holds.
struct Held<T>(Reactive<T>);

impl<T: 'static> Clone for Prop<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(match &self.0 {
            Inner::Given(r) => Inner::Given(r.clone()),
            Inner::Held(s) => Inner::Held(*s),
        })
    }
}

impl<T: fmt::Debug + 'static> fmt::Debug for Prop<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Inner::Given(r) => f.debug_tuple("Prop").field(r).finish(),
            Inner::Held(s) => f.debug_tuple("Prop").field(s).finish(),
        }
    }
}

/// The same value given (a static one equal, a binding the same one), or the
/// same held signal.
impl<T: PartialEq + 'static> PartialEq for Prop<T> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Inner::Given(a), Inner::Given(b)) => same(a, b),
            (Inner::Held(a), Inner::Held(b)) => a == b,
            _ => false,
        }
    }
}

fn same<T: PartialEq>(a: &Reactive<T>, b: &Reactive<T>) -> bool {
    match (a, b) {
        (Reactive::Static(a), Reactive::Static(b)) => a == b,
        (Reactive::Bound(a), Reactive::Bound(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

impl<T: Default + 'static> Default for Prop<T> {
    fn default() -> Self {
        Self(Inner::Given(Reactive::Static(T::default())))
    }
}

impl<T: 'static> From<T> for Prop<T> {
    fn from(value: T) -> Self {
        Self(Inner::Given(Reactive::Static(value)))
    }
}

impl<T: 'static> From<Reactive<T>> for Prop<T> {
    fn from(value: Reactive<T>) -> Self {
        Self(Inner::Given(value))
    }
}

macro_rules! from_handles {
    ($($handle:ident),*) => {$(
        impl<T: Clone + 'static> From<$handle<T>> for Prop<T> {
            fn from(value: $handle<T>) -> Self {
                Self(Inner::Given(value.into_reactive()))
            }
        }
    )*};
}

from_handles!(Signal, Memo, ReadSignal);

impl From<&'static str> for Prop<super::SharedString> {
    fn from(value: &'static str) -> Self {
        Self(Inner::Given(Reactive::Static(value.into())))
    }
}

impl From<String> for Prop<super::SharedString> {
    fn from(value: String) -> Self {
        Self(Inner::Given(Reactive::Static(value.into())))
    }
}

/// A parent's prop passed on to a child.
impl<T: Clone + 'static> From<&Prop<T>> for Prop<T> {
    fn from(value: &Prop<T>) -> Self {
        Self(Inner::Given(value.reactive()))
    }
}

impl<T: Clone + 'static> Prop<T> {
    /// A prop given `value`.
    pub fn new(value: T) -> Self {
        Self(Inner::Given(Reactive::Static(value)))
    }

    /// A prop bound to `f`, run again when what it reads changes (a value
    /// read in several places is better a [`super::computed`] memo).
    pub fn bind(f: impl Fn() -> T + 'static) -> Self {
        Self(Inner::Given(Reactive::Bound(Rc::new(f))))
    }

    /// The value, subscribing the running computation.
    pub fn get(&self) -> T {
        match &self.0 {
            Inner::Given(r) => r.get(),
            Inner::Held(s) => {
                let r = s.with(|h| h.0.clone());
                r.get()
            }
        }
    }

    /// `f` of the value, subscribing the running computation.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        match &self.0 {
            Inner::Given(Reactive::Static(v)) => f(v),
            _ => f(&self.get()),
        }
    }

    /// The value, without subscribing.
    pub fn get_untracked(&self) -> T {
        super::untracked(|| self.get())
    }

    /// As a [`Reactive`] (to pass on to a child, or to bind an element's
    /// setter to it).
    pub fn reactive(&self) -> Reactive<T> {
        match &self.0 {
            Inner::Given(r) => r.clone(),
            Inner::Held(s) => {
                let s = *s;
                Reactive::Bound(Rc::new(move || {
                    let r = s.with(|h| h.0.clone());
                    r.get()
                }))
            }
        }
    }

    /// The prop held in a signal of the current owner (an instance's).
    #[doc(hidden)]
    pub fn __hold(&self) -> Self {
        Self(Inner::Held(signal_with(
            Held(self.reactive_given()),
            SignalEquals::Never,
        )))
    }

    /// What it was given (a held one: what it holds).
    fn reactive_given(&self) -> Reactive<T> {
        match &self.0 {
            Inner::Given(r) => r.clone(),
            Inner::Held(s) => s.with_untracked(|h| h.0.clone()),
        }
    }

    /// Takes `value` (unless it is the same): a held prop replaces what it
    /// holds, updating its readers; one not held yet (an instance's before
    /// its view is made) is given it. Whether it changed.
    #[doc(hidden)]
    pub fn __set(&mut self, value: Reactive<T>) -> bool
    where
        T: PartialEq,
    {
        match &mut self.0 {
            Inner::Held(s) => {
                if s.with_untracked(|h| same(&h.0, &value)) {
                    return false;
                }
                s.set(Held(value));
                true
            }
            Inner::Given(given) => {
                if same(given, &value) {
                    return false;
                }
                *given = value;
                true
            }
        }
    }

    /// Replaces what a held prop holds with what `new` was given.
    #[doc(hidden)]
    pub fn __update_from(&mut self, new: &Self) -> bool
    where
        T: PartialEq,
    {
        self.__set(new.reactive_given())
    }
}

/// A prop passed on, to an element's setter or a child's prop: bound to it
/// (`text(&props.title)`).
impl<T: Clone + 'static> IntoReactive<T> for &Prop<T> {
    fn into_reactive(self) -> Reactive<T> {
        self.reactive()
    }
}
