//! Modules: functions a UI set exports to JS without a component
//! (`Alert.alert(...)`, `Keyboard.dismiss()`), written in Rust with
//! `#[module]` and generated into the set's JS module like its components.
//! A call is an op (ordered with the props set before it); a function
//! returning a value gives JS a `Promise`, an `async fn` runs as a task.

use std::{any::TypeId, future::Future, pin::Pin};

use super::{
    AsyncCx, DialogAnswer, Environment, PropError, TsType, TsTypes,
    element::ErasedCx,
    task::{self, Action},
};
use crate::{Command, Dialog, JsonValue, LayoutAnimation, ROOT_ID, StatusBarSetting};

/// A JS object of functions (`#[module]` implements it).
pub trait Module {
    /// Its JS name: the type's own (`#[module] impl Alert`: `Alert`), which
    /// `#[module]` writes; JS calls its functions by it (`Alert.alert`).
    const NAME: &'static str;
    /// Documentation (the `impl` block's doc comment).
    const DOC: &'static str;
    /// Its functions.
    const FUNCTIONS: &'static [Function];
    /// Its functions are exported one by one (`setStatusBarBackgroundColor`)
    /// rather than as an object (`#[module(flat)]`).
    const FLAT: bool = false;
}

/// A module function's parameter: its JS name, TypeScript type, and
/// whether it may be left out.
pub type ModuleParam = (&'static str, TsType, bool);

/// The future of an `async` module function: its result, or the error the
/// `Promise` rejects with.
pub type ModuleFuture = Pin<Box<dyn Future<Output = Result<JsonValue, String>>>>;

/// How a module function runs.
pub enum FunctionCall {
    /// At once, with a [`ModuleCx`]; an error rejects the `Promise`.
    Sync(fn(Vec<JsonValue>, &mut ModuleCx<'_>) -> Result<JsonValue, PropError>),
    /// As a task (its arguments checked first).
    Async(fn(Vec<JsonValue>, AsyncCx) -> Result<ModuleFuture, PropError>),
}

/// A function of a module (`#[module]` makes them).
pub struct Function {
    /// Its JS name (`alert`).
    pub name: &'static str,
    /// Documentation.
    pub doc: &'static str,
    /// Its parameters.
    pub params: &'static [ModuleParam],
    /// The TypeScript type of its result; `None`: nothing to wait for.
    pub returns: Option<TsType>,
    /// How it runs.
    pub call: FunctionCall,
}

/// One function, for the glue and its declarations.
#[derive(Debug, Clone)]
pub struct FunctionSpec {
    /// Its JS name.
    pub name: &'static str,
    /// Documentation.
    pub doc: &'static str,
    /// Its parameters: JS name, TypeScript type, optional.
    pub params: Vec<(&'static str, String, bool)>,
    /// The TypeScript type of its result.
    pub returns: Option<String>,
}

/// What a module is to JS.
#[derive(Debug, Clone)]
pub struct ModuleSpec {
    /// Its JS name.
    pub name: &'static str,
    /// Documentation.
    pub doc: &'static str,
    /// Its functions.
    pub functions: Vec<FunctionSpec>,
    /// Exported one by one ([`Module::FLAT`]).
    pub flat: bool,
    /// The named TypeScript types they use.
    pub types: TsTypes,
}

impl ModuleSpec {
    pub(crate) fn of<M: Module>() -> Self {
        let mut types = TsTypes::default();
        let functions = M::FUNCTIONS
            .iter()
            .map(|f| FunctionSpec {
                name: f.name,
                doc: f.doc,
                params: f
                    .params
                    .iter()
                    .map(|(n, ts, optional)| (*n, ts(&mut types), *optional))
                    .collect(),
                returns: f.returns.map(|ts| ts(&mut types)),
            })
            .collect();
        Self {
            name: M::NAME,
            doc: M::DOC,
            functions,
            flat: M::FLAT,
            types,
        }
    }
}

/// The context of a module function: the environment, the surface's
/// globals and the platform's services.
pub struct ModuleCx<'a> {
    inner: &'a mut ErasedCx,
}

impl<'a> ModuleCx<'a> {
    pub(crate) fn new(inner: &'a mut ErasedCx) -> Self {
        Self { inner }
    }

    /// The environment now.
    pub fn environment(&self) -> &Environment {
        &self.inner.environment
    }

    /// Runs `f` on the surface's `G` (made with `Default` the first time),
    /// the state a module shares with its components.
    pub fn update_global<G: Default + 'static, R>(
        &mut self,
        f: impl FnOnce(&mut G, &mut Self) -> R,
    ) -> R {
        let id = TypeId::of::<G>();
        let taken = self.inner.globals.borrow_mut().remove(&id);
        let mut global = taken
            .and_then(|g| g.downcast::<G>().ok())
            .unwrap_or_default();
        let result = f(&mut global, self);
        self.inner.globals.borrow_mut().insert(id, global);
        if !self.inner.changed_globals.contains(&id) {
            self.inner.changed_globals.push(id);
        }
        result
    }

    /// Shows `dialog`; await the answer in an `async` function.
    pub fn dialog(&mut self, dialog: Dialog) -> DialogAnswer {
        let (answer, slot) = DialogAnswer::new();
        self.inner.actions.push(Action::Dialog(dialog, slot));
        answer
    }

    /// Unfocuses the focused input, hiding the software keyboard.
    pub fn dismiss_keyboard(&mut self) {
        self.inner
            .actions
            .push(Action::Command(ROOT_ID, Command::Blur));
    }

    /// Changes what functions set for the status bar (the fields `patch`
    /// sets), under the settings components render.
    pub fn set_status_bar(&mut self, patch: StatusBarSetting) {
        self.inner.actions.push(task::status_bar(patch));
    }

    /// Animates the layout changes of the next commit.
    pub fn animate_layout(&mut self, animation: LayoutAnimation) {
        self.inner.actions.push(Action::Command(
            ROOT_ID,
            Command::LayoutAnimation(animation),
        ));
    }

    /// The screen reader reads `message` (TS:
    /// `AccessibilityInfo.announceForAccessibility`).
    pub fn announce_for_accessibility(&mut self, message: impl Into<String>) {
        self.inner
            .actions
            .push(super::services::announce(message.into()));
    }

    /// Shows `message` at the bottom of the screen (Android's toasts);
    /// nothing happens elsewhere.
    pub fn toast(&mut self, message: impl Into<String>, duration: super::ToastDuration) {
        self.inner
            .actions
            .push(super::services::toast(message.into(), duration));
    }

    /// Lays the surface out right to left whatever the user's language
    /// (TS: `I18nManager.forceRTL`); `false` follows the language again.
    pub fn force_rtl(&mut self, force: bool) {
        self.inner
            .actions
            .push(Action::Direction(super::services::Direction::Force(force)));
    }

    /// Whether a right-to-left language lays the surface out right to left
    /// (TS: `I18nManager.allowRTL`; true by default).
    pub fn allow_rtl(&mut self, allow: bool) {
        self.inner
            .actions
            .push(Action::Direction(super::services::Direction::Allow(allow)));
    }

    /// Asks the platform for `action` (the runtime's own modules).
    pub(crate) fn act(&mut self, action: Action) {
        self.inner.actions.push(action);
    }
}
