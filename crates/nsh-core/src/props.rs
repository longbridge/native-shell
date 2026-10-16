//! Non-style view properties: touch handling, accessibility, gesture values
//! and layout animations.

use std::sync::Arc;

use crate::{js::host::HostProps, ui::PropEnum};

/// Edge insets in points.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct EdgeInsets {
    /// Top.
    pub top: f32,
    /// Right.
    pub right: f32,
    /// Bottom.
    pub bottom: f32,
    /// Left.
    pub left: f32,
}

impl EdgeInsets {
    /// The same inset on every edge.
    pub const fn all(v: f32) -> Self {
        Self {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }

    /// True if every inset is zero.
    pub fn is_zero(&self) -> bool {
        *self == Self::default()
    }
}

/// Which touches a view and its children receive (`pointerEvents`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum PointerEvents {
    /// The view and its children receive touches (default).
    #[default]
    Auto,
    /// Neither the view nor its children receive touches.
    None,
    /// Only the children receive touches; the view itself is transparent to them.
    #[prop(rename = "box-none")]
    BoxNone,
    /// Only the view receives touches, never its children.
    #[prop(rename = "box-only")]
    BoxOnly,
}

/// What an element is, for assistive technologies (`accessibilityRole`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum AccessibilityRole {
    /// No particular role.
    #[default]
    None,
    /// A button.
    Button,
    /// A link.
    Link,
    /// A search field.
    Search,
    /// An image.
    Image,
    /// An image that acts as a button.
    #[prop(rename = "imagebutton")]
    ImageButton,
    /// A key of a keyboard.
    #[prop(rename = "keyboardkey")]
    KeyboardKey,
    /// Static text.
    Text,
    /// A value that can be adjusted (slider).
    Adjustable,
    /// A section header.
    Header,
    /// A summary of the app's state.
    Summary,
    /// An alert.
    Alert,
    /// A checkbox.
    Checkbox,
    /// A combo box.
    Combobox,
    /// A menu.
    Menu,
    /// A menu bar.
    Menubar,
    /// An item of a menu.
    Menuitem,
    /// A progress bar.
    Progressbar,
    /// A radio button.
    Radio,
    /// A group of radio buttons.
    Radiogroup,
    /// A scroll bar.
    Scrollbar,
    /// A spin button.
    Spinbutton,
    /// A switch.
    Switch,
    /// A tab.
    Tab,
    /// A list of tabs.
    Tablist,
    /// A timer.
    Timer,
    /// A toolbar.
    Toolbar,
    /// A list.
    List,
    /// A grid.
    Grid,
}

/// Whether a checkable element is checked (`accessibilityState.checked`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Checked {
    /// Not checked.
    False,
    /// Checked.
    True,
    /// Partially checked.
    Mixed,
}

/// Dynamic state of an element (`accessibilityState`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AccessibilityState {
    /// Disabled.
    pub disabled: bool,
    /// Selected.
    pub selected: bool,
    /// Checked state of checkable elements.
    pub checked: Option<Checked>,
    /// Busy (loading).
    pub busy: bool,
    /// Expanded or collapsed, for expandable elements.
    pub expanded: Option<bool>,
}

/// The value of a range or textual element (`accessibilityValue`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AccessibilityValue {
    /// Minimum of the range.
    pub min: Option<f32>,
    /// Maximum of the range.
    pub max: Option<f32>,
    /// Current value in the range.
    pub now: Option<f32>,
    /// A textual description of the value; wins over the range.
    pub text: Option<Arc<str>>,
}

/// How changes of an element are announced (Android `accessibilityLiveRegion`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub enum LiveRegion {
    /// Not announced.
    #[default]
    None,
    /// Announced when the user is idle.
    Polite,
    /// Announced right away.
    Assertive,
}

/// A custom action offered to assistive technologies.
#[derive(Debug, Clone, PartialEq)]
pub struct AccessibilityActionDef {
    /// Name reported back in `onAccessibilityAction` (`activate`,
    /// `increment`, `decrement` and `longpress` are standard actions).
    pub name: Arc<str>,
    /// Label shown to the user; required for custom actions.
    pub label: Option<Arc<str>>,
}

/// Accessibility of one view.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Accessibility {
    /// `accessible`: the view is one element for assistive technologies
    /// (its children are not exposed separately). `None` keeps the
    /// platform default: views with text or touch handlers are elements.
    pub accessible: Option<bool>,
    /// `accessibilityLabel`.
    pub label: Option<Arc<str>>,
    /// `accessibilityHint`.
    pub hint: Option<Arc<str>>,
    /// `accessibilityRole`.
    pub role: AccessibilityRole,
    /// `accessibilityState`.
    pub state: AccessibilityState,
    /// `accessibilityValue`.
    pub value: Option<AccessibilityValue>,
    /// `accessibilityElementsHidden` / `importantForAccessibility:
    /// "no-hide-descendants"`: hide the view and its subtree.
    pub hidden: bool,
    /// `accessibilityLiveRegion`.
    pub live_region: LiveRegion,
    /// `accessibilityActions`.
    pub actions: Vec<AccessibilityActionDef>,
}

impl Accessibility {
    /// True if nothing is set.
    pub fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

/// One accessibility property, as set from JS.
#[derive(Debug, Clone, PartialEq, HostProps)]
pub enum AccessibilityProp {
    /// `accessible`: the view is one element for assistive technologies (its
    /// children are not exposed separately).
    #[prop(js = "accessible")]
    Accessible(Option<bool>),
    /// `accessibilityLabel`: what screen readers read.
    #[prop(js = "accessibilityLabel")]
    Label(Option<Arc<str>>),
    /// `accessibilityHint`: what activating the element does.
    #[prop(js = "accessibilityHint")]
    Hint(Option<Arc<str>>),
    /// `accessibilityRole` / `role`: what the element is.
    #[prop(js = "accessibilityRole", js = "role", with = crate::js::host_values::role, reset = AccessibilityProp::Role(AccessibilityRole::None))]
    Role(AccessibilityRole),
    /// `accessibilityState`.
    #[prop(js = "accessibilityState")]
    State(AccessibilityState),
    /// `accessibilityValue`: the value of a range or textual element.
    #[prop(js = "accessibilityValue")]
    Value(Option<AccessibilityValue>),
    /// `accessibilityElementsHidden`: hides the view and its subtree from assistive
    /// technologies.
    #[prop(js = "accessibilityElementsHidden")]
    Hidden(bool),
    /// `importantForAccessibility`: `"auto"`, `"yes"`, `"no"` or
    /// `"no-hide-descendants"`.
    #[prop(js = "importantForAccessibility", with = crate::js::host_values::important, reset = AccessibilityProp::Important(None, false), ts = "\"auto\" | \"yes\" | \"no\" | \"no-hide-descendants\"")]
    Important(Option<bool>, bool),
    /// `accessibilityLiveRegion` (Android): how changes of the view are announced
    /// to TalkBack.
    #[prop(android = "accessibilityLiveRegion")]
    LiveRegion(LiveRegion),
    /// `accessibilityActions`: custom actions offered to assistive technologies
    /// (`onAccessibilityAction` reports them).
    #[prop(js = "accessibilityActions")]
    Actions(Vec<AccessibilityActionDef>),
}

/// Android's press ripple (`android={{ ripple }}`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Ripple {
    /// Ripple color; the theme's when `None`.
    pub color: Option<crate::style::Color>,
    /// The ripple is not clipped to the view.
    pub borderless: bool,
    /// Radius in points; the platform's when `None`.
    pub radius: Option<f32>,
    /// Drawn over the view's content (else behind it).
    pub foreground: bool,
}

/// Non-style properties of a view that only the platform needs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ViewProps {
    /// Which touches the view and its subtree receive.
    pub pointer_events: PointerEvents,
    /// Extends the touch area beyond the view's bounds.
    pub hit_slop: EdgeInsets,
    /// Opacity while pressed (native press feedback, no JS round trip).
    pub active_opacity: Option<f32>,
    /// Milliseconds before a press becomes a long press (platform default
    /// when `None`).
    pub long_press_delay: Option<f32>,
    /// Ripple while pressed (Android).
    pub ripple: Option<Ripple>,
    /// Accessibility.
    pub accessibility: Accessibility,
    /// `testID`: an identifier for UI tests.
    pub test_id: Option<Arc<str>>,
}

/// Native animation values driven by gestures on a view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GestureValues {
    /// Horizontal drag translation (`pan={{ x }}`).
    pub pan_x: Option<nsh_animation::NodeId>,
    /// Vertical drag translation (`pan={{ y }}`).
    pub pan_y: Option<nsh_animation::NodeId>,
    /// Pinch scale (`pinch={{ scale }}`), multiplied from its value when the
    /// pinch starts.
    pub pinch: Option<nsh_animation::NodeId>,
    /// Two-finger rotation in degrees (`rotation={{ angle }}`), added to its
    /// value when the rotation starts.
    pub rotation: Option<nsh_animation::NodeId>,
}

impl GestureValues {
    /// True if no value is driven.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

/// Timing curve of a layout animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutCurve {
    /// Constant speed.
    Linear,
    /// Accelerates.
    EaseIn,
    /// Decelerates.
    EaseOut,
    /// Accelerates, then decelerates.
    EaseInEaseOut,
    /// A spring with the given damping ratio (0..1).
    Spring {
        /// Damping ratio: 1 does not overshoot.
        damping: f32,
    },
}

/// The property animated when views appear or disappear.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutProperty {
    /// Fade in / out.
    Opacity,
    /// Grow / shrink from the center.
    Scale,
}

/// How one kind of change animates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutTransition {
    /// Curve.
    pub curve: LayoutCurve,
    /// Animated property of appearing / disappearing views.
    pub property: LayoutProperty,
    /// Duration in milliseconds.
    pub duration_ms: f32,
    /// Delay in milliseconds.
    pub delay_ms: f32,
}

/// Animates the layout changes of the next commit
/// (`LayoutAnimation.configureNext`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutAnimation {
    /// Views inserted by the commit.
    pub create: Option<LayoutTransition>,
    /// Views whose frame changes.
    pub update: Option<LayoutTransition>,
    /// Views removed by the commit.
    pub delete: Option<LayoutTransition>,
}

impl LayoutTransition {
    /// `curve` over `duration_ms`, appearing views fading.
    pub const fn new(curve: LayoutCurve, duration_ms: f32) -> Self {
        Self {
            curve,
            property: LayoutProperty::Opacity,
            duration_ms,
            delay_ms: 0.0,
        }
    }
}

impl LayoutAnimation {
    /// `transition` for every kind of change.
    pub const fn all(transition: LayoutTransition) -> Self {
        Self {
            create: Some(transition),
            update: Some(transition),
            delete: Some(transition),
        }
    }

    /// React Native's `LayoutAnimation.Presets.easeInEaseOut`: 300 ms,
    /// views fading in and out.
    pub const EASE_IN_EASE_OUT: Self =
        Self::all(LayoutTransition::new(LayoutCurve::EaseInEaseOut, 300.0));

    /// `Presets.linear`: 500 ms at constant speed.
    pub const LINEAR: Self = Self::all(LayoutTransition::new(LayoutCurve::Linear, 500.0));

    /// `Presets.spring`: frames spring (damping 0.4) over 700 ms, views
    /// fade linearly.
    pub const SPRING: Self = Self {
        create: Some(LayoutTransition::new(LayoutCurve::Linear, 700.0)),
        update: Some(LayoutTransition::new(
            LayoutCurve::Spring { damping: 0.4 },
            700.0,
        )),
        delete: Some(LayoutTransition::new(LayoutCurve::Linear, 700.0)),
    };
}
