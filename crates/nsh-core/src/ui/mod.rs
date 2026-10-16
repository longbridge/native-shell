//! Components written in Rust: the UI sets' components (`nsh-ui-mobile`),
//! run by the core on the engine thread.
//!
//! A [`Component`] renders a tree of elements ([`view`], [`text`],
//! [`image`], other components) styled with typed methods ([`Styled`]:
//! `.px_4()`, `.rounded_xl()`, `.bg(theme.primary)`) that lower into the
//! same rules as Tailwind classes. The core expands the tree into nodes of
//! its own, diffs it against the previous render and lays it out in the same
//! pass as everything else, so a component sizes like any view, with no
//! round trip. Events on its elements go to the component first
//! ([`RenderCx::listener`]); it decides what reaches JS ([`Context::emit`])
//! and calls [`Context::notify`] to render again. State lives in the
//! component (`Self`), set up from the first props ([`Component::setup`]),
//! changed by listeners, timers and async tasks ([`Context::spawn`]); a render
//! subscribes to what it reads of the environment ([`RenderCx::window`]).
//!
//! Props are a struct (`#[derive(Props)]`), events an enum
//! (`#[derive(Event)]`); their JS face (`nsh-js`) and TypeScript
//! declarations ([`declarations`]) are made from them. In JS a component is one host node:
//! the root of its expansion. The app's `class` is appended to the root's
//! styles and its `style` stays the root's inline style. Colors come from
//! the [`Theme`] (gpui-kit's fields), with a light and a dark value each.
//!
//! Node ids: JS allocates even ids, the core odd ones for the nodes it
//! expands, so both stay dense. How the core runs components:
//! `docs/dev/components.md`.

pub(crate) mod animated;
mod back;
mod component;
mod element;
mod element_ref;
pub(crate) mod env;
mod events;
pub(crate) mod host;
mod module;
pub mod palette;
pub(crate) mod status_bar;
mod style;
pub(crate) mod task;
mod theme;
mod value;
pub(crate) mod virtual_list;

pub use nsh_anim::{Curve, EaseMode, Easing, Extrapolate, Output as AnimOutput};
pub use nsh_macros::{Event, JsObject, PropEnum, Props, methods, module};

pub use self::{
    animated::{
        AnimOperand, Animated, Animation, AnimationEnd, Interpolation, NativeNode, NativeValue,
        Spring, Timing,
    },
    back::BackSubscription,
    component::{
        Component, ComponentError, ComponentSpec, Event, EventSpec, Method, MethodSpec, NoEvent,
        Param, PropKind, PropSpec, Props, Registry, Root, declarations,
    },
    element::{
        Accessible, AnyElement, Attrs, Child, Children, ComponentElement, Context, ElNode, Element,
        FluentBuilder, Image, Interactive, IntoElement, Items, ListConfig, ListElement, Listener,
        ParentElement, Position, RenderCx, Slot, SlotElement, Style, Text, View, image, text, view,
    },
    element_ref::ElementRef,
    env::Environment,
    events::{
        GestureEvent, NativeEvent, PanEvent, ScrollEvent, ScrollOffset, Selection, VideoInfo,
        VideoProgress, WebNavigation,
    },
    host::{
        CanvasArg, CanvasView, Input, Modal, NativeView, Painter, Screen, Scroll, Stack, Video,
        WebView, canvas, input, modal, native, screen, scroll, stack, video, web_view,
    },
    module::{
        Function, FunctionCall, FunctionSpec, Module, ModuleCx, ModuleFuture, ModuleParam,
        ModuleSpec,
    },
    status_bar::StatusBarEntry,
    style::{
        AdaptiveColor, BorderCurve, FontWeight, Length, StyleRefinement, Styled, WritingDirection,
        px, relative, rgb, rgba,
    },
    task::{Answer, AsyncCx, DialogAnswer, ImageSize, Task, Updated, WeakComponent},
    theme::{Theme, ThemeColors, ThemeError},
    value::{JsValue, NodeRef, PropError, SharedString, TsType, TsTypes},
    virtual_list::{ListLayout, VirtualList, virtual_list},
};
pub(crate) use self::{
    back::BackListener,
    component::{AnyInstance, caught},
    element::{ElKind, ErasedCx, Globals, PlaceFn, Provided, RenderReads, RenderScope},
    task::{Action, AnswerSlot, Executor},
};
pub use crate::{
    AccessibilityRole, AccessibilityState, AutoCapitalize, AutoFill, BlendMode, BoxShadow, Dialog,
    DialogButton, DialogButtonStyle, EdgeInsets, EllipsizeMode, EventKind, EventPayload,
    KeyboardDismissMode, KeyboardTaps, KeyboardType, LayoutAnimation, LayoutCurve, LayoutProperty,
    LayoutTransition, ModalAnimation, ModalPresentation, PointerEvents, ResizeMode, ReturnKeyType,
    ScreenAnimation, SnapAlignment, StatusBarConfig, StatusBarSetting, StatusBarStyle, TouchEvent,
    TransformKind, TransformOrigin,
    style::{Color, OriginLen},
};

/// Everything a component's module needs: `use nsh_core::ui::prelude::*;`.
pub mod prelude {
    pub use super::{
        AccessibilityRole,
        AccessibilityState,
        Accessible,
        AdaptiveColor,
        AnimOutput,
        Animated,
        Animation,
        AnyElement,
        AsyncCx,
        // Host elements and their settings.
        AutoCapitalize,
        BackSubscription,
        BlendMode,
        BorderCurve,
        BoxShadow,
        CanvasView,
        Child,
        Children,
        Color,
        Component,
        ComponentElement,
        ComponentError,
        Context,
        Curve,
        DialogAnswer,
        EaseMode,
        // Native animation values.
        Easing,
        EdgeInsets,
        Element,
        ElementRef,
        Environment,
        Event,
        EventKind,
        EventPayload,
        Extrapolate,
        FluentBuilder,
        FontWeight,
        // Typed events.
        GestureEvent,
        Input,
        Interactive,
        Interpolation,
        IntoElement,
        Items,
        JsObject,
        JsValue,
        KeyboardType,
        Length,
        ListLayout,
        Listener,
        Method,
        Modal,
        ModalAnimation,
        ModalPresentation,
        ModuleCx,
        NativeEvent,
        NativeNode,
        NativeValue,
        NativeView,
        NoEvent,
        Painter,
        PanEvent,
        ParentElement,
        PointerEvents,
        Position,
        PropEnum,
        Props,
        RenderCx,
        ResizeMode,
        ReturnKeyType,
        Screen,
        ScreenAnimation,
        Scroll,
        ScrollEvent,
        ScrollOffset,
        Selection,
        SharedString,
        Slot,
        Spring,
        Stack,
        Styled,
        Task,
        Text,
        Theme,
        Timing,
        Video,
        VideoInfo,
        VideoProgress,
        View,
        VirtualList,
        WeakComponent,
        WebNavigation,
        WebView,
        WritingDirection,
        canvas,
        image,
        input,
        methods,
        modal,
        module,
        native,
        px,
        relative,
        rgb,
        rgba,
        screen,
        scroll,
        stack,
        text,
        video,
        view,
        virtual_list,
        web_view,
    };
}

/// What the derive macros' code names.
#[doc(hidden)]
pub mod __private {
    pub use serde_json::Map as JsonMap;
}
