//! `host.switch`: a GPUI Kit `Switch` inside the nsh app, as a native
//! component. nsh lays its box out like any view, at the size the switch
//! reports (its track and label); the switch draws, takes its own clicks
//! and keys (Space while focused), and tells JS (`onChange`, `{ checked }`).

use std::{cell::Cell, rc::Rc};

use gpui_kit::{
    AnyElement, App, IntoElement, TextRun, Window,
    component::{ActiveTheme, switch::Switch},
    px,
};
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
    /// The size last reported to nsh.
    reported: (f32, f32),
}

/// The switch's track and the gap to its label (GPUI Kit's medium size).
const TRACK: (f32, f32) = (36.0, 20.0);
const GAP: f32 = 8.0;

impl HostSwitch {
    /// The component's factory (`AppView::register_component`).
    pub fn make(cx: ComponentContext) -> Box<dyn NativeComponent> {
        Box::new(HostSwitch {
            cx,
            label: String::new(),
            checked: Rc::default(),
            reported: (0.0, 0.0),
        })
    }
}

impl NativeComponent for HostSwitch {
    fn render(&mut self, window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        // Its natural size: nsh lays the box out at it (the app's style
        // leaves both dimensions open).
        let font_size = cx.theme().font_size;
        let run = TextRun {
            len: self.label.len(),
            font: window.text_style().font(),
            color: cx.theme().foreground,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let label =
            window
                .text_system()
                .shape_line(self.label.clone().into(), font_size, &[run], None);
        let line_height = f32::from(window.line_height().max(px(TRACK.1)));
        let wanted = (
            (TRACK.0 + GAP + f32::from(label.width)).ceil(),
            line_height.ceil(),
        );
        if wanted != self.reported {
            self.reported = wanted;
            self.cx.set_size(wanted.0, wanted.1);
        }
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
