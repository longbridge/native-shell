//! `host.switch`: a GPUI Kit `Switch` inside the nsh app, as a native
//! component. nsh lays its box out like any view; the switch draws, takes
//! its own clicks and keys, and tells JS (`onChange`, `{ checked }`).

use std::{cell::Cell, rc::Rc};

use gpui_kit::{AnyElement, App, IntoElement, Window, component::switch::Switch};
use nsh_platform_gpui::{ComponentContext, NativeComponent};
use serde_json::{Value, json};

/// The registered name, which JS declares with `nativeComponent`.
pub const NAME: &str = "host.switch";

/// One switch.
pub struct HostSwitch {
    cx: ComponentContext,
    label: String,
    /// Controlled by JS (`checked`), and changed at once by a click so the
    /// switch does not wait for JS to answer.
    checked: Rc<Cell<bool>>,
}

impl HostSwitch {
    /// The component's factory (`AppView::register_component`).
    pub fn make(cx: ComponentContext) -> Box<dyn NativeComponent> {
        Box::new(HostSwitch {
            cx,
            label: String::new(),
            checked: Rc::default(),
        })
    }
}

impl NativeComponent for HostSwitch {
    fn render(&mut self, _window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let (cx, checked) = (self.cx.clone(), self.checked.clone());
        Some(
            Switch::new("host-switch")
                .checked(self.checked.get())
                .label(self.label.clone())
                .on_change(move |value: &bool, _, _| {
                    checked.set(*value);
                    cx.emit("change", json!({ "checked": value }));
                    cx.redraw();
                })
                .into_any_element(),
        )
    }

    fn set_prop(&mut self, name: &str, value: Option<&Value>) {
        match name {
            "label" => self.label = value.and_then(Value::as_str).unwrap_or("").to_owned(),
            "checked" => self
                .checked
                .set(value.and_then(Value::as_bool).unwrap_or(false)),
            _ => {}
        }
    }
}
