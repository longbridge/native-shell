//! `SelectList`: a scrolling list of options, one of them chosen. The rows
//! are the component's own (a virtual list rendered in Rust), so thousands
//! of options cost what the visible ones do.

use std::rc::Rc;

use nsh_core::ui::prelude::*;

/// A list of options ([`SelectListProps`]).
#[derive(Debug, Default)]
pub struct SelectList {
    /// The options, shared with the rows.
    options: Rc<Vec<SharedString>>,
    /// The chosen one: the app's when it controls it, else our own.
    chosen: Option<u32>,
}

/// A scrolling list of options with a check mark on the chosen one. With
/// `value` the app controls the choice (it updates `value` from
/// `onSelect`); without, the list keeps its own.
#[derive(Debug, Clone, PartialEq, Props)]
pub struct SelectListProps {
    /// The options.
    pub options: Vec<SharedString>,
    /// The index of the chosen option, when the app controls it.
    pub value: Option<u32>,
}

/// The events of a [`SelectList`].
#[derive(Debug, Clone, PartialEq, Event)]
pub enum SelectListEvent {
    /// An option was chosen (its index).
    Select(u32),
}

impl Component for SelectList {
    type Props = SelectListProps;
    type Event = SelectListEvent;

    fn setup(&mut self, props: &SelectListProps, _cx: &mut Context<'_, Self>) {
        self.options = Rc::new(props.options.clone());
    }

    fn props_changed(
        &mut self,
        old: &SelectListProps,
        new: &SelectListProps,
        _cx: &mut Context<'_, Self>,
    ) {
        if new.options != old.options {
            self.options = Rc::new(new.options.clone());
        }
    }

    fn render(&self, props: &SelectListProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let theme = cx.theme();
        // The rows outlive the render: they keep the colors.
        let (background, active, border, foreground, primary) = (
            theme.background,
            theme.list_active,
            theme.border,
            theme.foreground,
            theme.primary,
        );
        let chosen = props.value.or(self.chosen);
        let options = self.options.clone();
        let select = cx.listener(|this: &mut Self, index: &u32, cx| {
            if cx.props().value.is_none() {
                this.chosen = Some(*index);
                cx.notify();
            }
            cx.emit(SelectListEvent::Select(*index));
        });
        view().flex_1().child(
            virtual_list(options.len(), move |i, at| {
                let index = i as u32;
                let on = chosen == Some(index);
                view()
                    .flex_row()
                    .items_center()
                    .min_h(px(48.0))
                    .px_4()
                    .gap_3()
                    .bg(background)
                    .active(|s| s.bg(active))
                    .when(!at.is_last(), |v| v.border_b_1().border_color(border))
                    .role(AccessibilityRole::Button)
                    .accessibility_state(AccessibilityState {
                        selected: on,
                        ..Default::default()
                    })
                    .on_press(select.clone().map(move |_: &()| index))
                    .child(
                        text(options[i].clone())
                            .flex_1()
                            .text_base()
                            .text_color(foreground),
                    )
                    .when(on, |v| v.child(text("✓").text_base().text_color(primary)))
            })
            .estimated_item_size(48.0)
            .keys(self.options.iter().cloned())
            .flex_1(),
        )
    }
}
