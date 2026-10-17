//! The gallery's building blocks without state of their own (as the TSX
//! gallery's `ui.tsx`): builders that render to elements, GPUI-style.
//! Those with content take it with `.child(..)` / `.children(..)`; those
//! with a root to place (a card in a row) take [`Styled`] methods for it.

use nsh_core::ui::{Screen, StyleRefinement, prelude::*};

use super::{
    animated::{PressProps, press_with},
    icons::{IconName, icon},
    theme::{Finish, Finished, MONO, Tone, colors, radius},
};

/// Delegates [`Styled`] to the builder's root view, and gives it
/// [`FluentBuilder`].
macro_rules! styled_root {
    ($($name:ident),*) => {$(
        impl Styled for $name {
            fn style(&mut self) -> &mut StyleRefinement {
                Styled::style(&mut self.root)
            }
        }

        impl FluentBuilder for $name {}
    )*};
}

/// Takes content: `.child(..)`, `.children(..)`.
macro_rules! content {
    ($($name:ident),*) => {$(
        impl $name {
            /// Adds content.
            pub(crate) fn child(mut self, child: impl IntoElement) -> Self {
                self.content.push(child.into_element());
                self
            }

            /// Adds content (an `Option` adds one or none).
            pub(crate) fn children(
                mut self,
                children: impl IntoIterator<Item = impl IntoElement>,
            ) -> Self {
                self.content.extend(children.into_iter().map(IntoElement::into_element));
                self
            }
        }
    )*};
}

/// The gallery's header colors on a screen (TS `{...header}`): the page's
/// background, the text color for the title, the accent for the back
/// button. Read in the render, so the header follows the appearance.
pub(crate) fn header<C: Component>(screen: Screen, cx: &RenderCx<'_, C>) -> Screen {
    screen
        .header_background_color(cx.color(colors::BACKGROUND))
        .header_title_color(cx.color(colors::TEXT))
        .header_tint_color(cx.color(colors::ACCENT))
}

/// A rounded icon tile.
pub(crate) struct IconTile {
    name: IconName,
    size: f32,
    tint: AdaptiveColor,
    root: View,
}

impl IconTile {
    /// Tile of `name`, 40 pt, the glyph in the text color.
    pub(crate) fn new(name: IconName) -> Self {
        Self {
            name,
            size: 40.0,
            tint: colors::TEXT,
            root: view(),
        }
    }

    /// Its width and height.
    pub(crate) fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The glyph's color.
    pub(crate) fn tint(mut self, tint: impl Into<AdaptiveColor>) -> Self {
        self.tint = tint.into();
        self
    }
}

impl IntoElement for IconTile {
    fn into_element(self) -> AnyElement {
        let size = self.size;
        self.root
            .size(px(size))
            .rounded(px(size * 0.3))
            .border_curve(BorderCurve::Continuous)
            .border_1()
            .border_color(colors::BORDER)
            .bg(colors::RAISED)
            .finish(Finish::Raised)
            .items_center()
            .justify_center()
            .child(icon(self.name, size * 0.5, self.tint))
            .into_element()
    }
}

/// A card: an optional icon, a title and a subtitle, something on the
/// right (a badge, an action), then the content on an inset surface (or
/// bare with [`Card::flush`]).
pub(crate) struct Card {
    icon: Option<IconName>,
    title: Option<SharedString>,
    subtitle: Option<SharedString>,
    right: Option<AnyElement>,
    flush: bool,
    content: Vec<AnyElement>,
    root: View,
}

impl Card {
    /// An empty card.
    pub(crate) fn new() -> Self {
        Self {
            icon: None,
            title: None,
            subtitle: None,
            right: None,
            flush: false,
            content: Vec::new(),
            root: view(),
        }
    }

    /// Its icon, before the title.
    pub(crate) fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Its title (without one, no header row).
    pub(crate) fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The line under the title.
    pub(crate) fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// What sits right of the title.
    pub(crate) fn right(mut self, right: impl IntoElement) -> Self {
        self.right = Some(right.into_element());
        self
    }

    /// The content sits on the card itself, not on an inset.
    pub(crate) fn flush(mut self) -> Self {
        self.flush = true;
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

/// A card's or a section's surface.
fn card_surface(root: View) -> View {
    root.bg(colors::SURFACE)
        .rounded(px(radius::CARD))
        .border_curve(BorderCurve::Continuous)
        .border_1()
        .border_color(colors::BORDER)
        .p(px(5.0))
        .finish(Finish::Card)
}

impl IntoElement for Card {
    fn into_element(self) -> AnyElement {
        let header = self.title.map(|title| {
            view()
                .flex_row()
                .items_center()
                .gap_3()
                .px(px(11.0))
                .pt(px(11.0))
                .pb(px(12.0))
                .children(self.icon.map(IconTile::new))
                .child(
                    view()
                        .flex_1()
                        .gap(px(2.0))
                        .child(
                            text(title)
                                .text_size(px(17.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors::TEXT),
                        )
                        .children(self.subtitle.map(|subtitle| {
                            text(subtitle)
                                .number_of_lines(2)
                                .text_size(px(13.0))
                                .text_color(colors::MUTED)
                        })),
                )
                .children(self.right)
        });
        let surface = card_surface(self.root).children(header);
        if self.flush {
            surface.children(self.content)
        } else {
            surface.child(Inset::new().children(self.content))
        }
        .into_element()
    }
}

/// A titled card of controls (the demos' sections).
pub(crate) struct Section {
    title: SharedString,
    note: Option<SharedString>,
    content: Vec<AnyElement>,
    root: View,
}

impl Section {
    /// A section titled `title`.
    pub(crate) fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            note: None,
            content: Vec::new(),
            root: view(),
        }
    }

    /// A quiet note right of the title.
    pub(crate) fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }
}

impl IntoElement for Section {
    fn into_element(self) -> AnyElement {
        card_surface(self.root)
            .child(
                view()
                    .flex_row()
                    .items_baseline()
                    .px(px(11.0))
                    .py(px(10.0))
                    .gap_2()
                    .child(
                        text(self.title)
                            .flex_1()
                            .text_size(px(15.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors::TEXT),
                    )
                    .children(self.note.map(|note| {
                        text(note)
                            .text_size(px(13.0))
                            .text_color(colors::FAINT)
                            .tabular_nums()
                    })),
            )
            .child(Inset::new().children(self.content))
            .into_element()
    }
}

/// The darker surface inside a card.
pub(crate) struct Inset {
    gap: f32,
    padding: f32,
    content: Vec<AnyElement>,
    root: View,
}

impl Inset {
    /// An inset with 14 pt padding and 12 pt between its children.
    pub(crate) fn new() -> Self {
        Self {
            gap: 12.0,
            padding: 14.0,
            content: Vec::new(),
            root: view(),
        }
    }

    /// The space between its children.
    pub(crate) fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Its padding.
    pub(crate) fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl Default for Inset {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoElement for Inset {
    fn into_element(self) -> AnyElement {
        self.root
            .bg(colors::INSET)
            .rounded(px(radius::INSET))
            .border_curve(BorderCurve::Continuous)
            .border_1()
            .border_color(colors::LINE)
            .finish(Finish::Inset)
            .p(px(self.padding))
            .gap(px(self.gap))
            .children(self.content)
            .into_element()
    }
}

/// A caption row: a label on the left, a quieter note on the right.
pub(crate) fn caption(
    label: impl Into<SharedString>,
    note: Option<impl Into<SharedString>>,
) -> View {
    view()
        .flex_row()
        .items_baseline()
        .gap_2()
        .child(
            text(label.into())
                .flex_1()
                .text_size(px(14.0))
                .text_color(colors::MUTED),
        )
        .children(note.map(|note| {
            text(note.into())
                .text_size(px(13.0))
                .text_color(colors::FAINT)
                .tabular_nums()
        }))
}

/// A large figure with a tinted note after it ("38 / 1k  would pass").
pub(crate) struct Figure {
    value: SharedString,
    unit: Option<SharedString>,
    note: Option<SharedString>,
    tone: Tone,
}

impl Figure {
    /// The figure `value`.
    pub(crate) fn new(value: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            unit: None,
            note: None,
            tone: Tone::Accent,
        }
    }

    /// Its unit, quieter, after it.
    pub(crate) fn unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// A note after it, in `tone`'s color (the accent by default).
    pub(crate) fn note(mut self, note: impl Into<SharedString>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// The note's tone.
    pub(crate) fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
}

impl IntoElement for Figure {
    fn into_element(self) -> AnyElement {
        text(self.value)
            .text_size(px(32.0))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(colors::TEXT)
            .tabular_nums()
            .letter_spacing(-0.5)
            .children(self.unit.map(|unit| {
                text(format!(" {unit}"))
                    .text_size(px(17.0))
                    .font_weight(FontWeight::NORMAL)
                    .text_color(colors::MUTED)
            }))
            .children(self.note.map(|note| {
                text(format!(" {note}"))
                    .text_size(px(15.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(self.tone.color())
            }))
            .into_element()
    }
}

/// A small label in a tinted capsule.
pub(crate) struct Badge {
    label: SharedString,
    tone: Tone,
    outline: bool,
}

impl Badge {
    /// A neutral badge `label`.
    pub(crate) fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            tone: Tone::Neutral,
            outline: false,
        }
    }

    /// Its tone.
    pub(crate) fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// A rim in the tone's color instead of a fill.
    pub(crate) fn outline(mut self) -> Self {
        self.outline = true;
        self
    }
}

impl IntoElement for Badge {
    fn into_element(self) -> AnyElement {
        let tone = self.tone;
        let fg = if tone == Tone::Neutral {
            colors::TEXT
        } else {
            tone.color()
        };
        view()
            .px(px(10.0))
            .h(px(26.0))
            .rounded(px(13.0))
            .justify_center()
            .border_1()
            .map(|v| {
                if self.outline {
                    v.border_color(tone.color())
                } else {
                    v.bg(tone.tint())
                        .finish_shadow(Finish::Chip)
                        .border_color(fg.opacity(0.1))
                }
            })
            .child(
                text(self.label)
                    .text_size(px(12.0))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg),
            )
            .into_element()
    }
}

/// A label above a value, for grids of figures.
pub(crate) struct Stat {
    label: SharedString,
    value: SharedString,
    tone: Option<Tone>,
    width: Option<f32>,
}

impl Stat {
    /// `value`, labelled `label`.
    pub(crate) fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone: None,
            width: None,
        }
    }

    /// The value in `tone`'s color.
    pub(crate) fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }

    /// Its width, a fraction of the row (`0.5`: two per row).
    pub(crate) fn width(mut self, fraction: f32) -> Self {
        self.width = Some(fraction);
        self
    }
}

impl IntoElement for Stat {
    fn into_element(self) -> AnyElement {
        view()
            .when_some(self.width, |v, w| v.w(relative(w)))
            .gap(px(3.0))
            .py_1()
            .child(
                text(self.label)
                    .text_size(px(12.0))
                    .text_color(colors::MUTED),
            )
            .child(
                text(self.value)
                    .text_size(px(15.0))
                    .text_color(self.tone.map_or(colors::TEXT, Tone::color))
                    .tabular_nums(),
            )
            .into_element()
    }
}

/// A colored square and a label (and a value), for chart legends.
pub(crate) fn legend(
    color: impl Into<AdaptiveColor>,
    label: impl Into<SharedString>,
    value: Option<impl Into<SharedString>>,
) -> View {
    view()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(view().size(px(9.0)).rounded(px(2.5)).bg(color.into()))
        .child(
            text(label.into())
                .text_size(px(13.0))
                .text_color(colors::MUTED)
                .children(value.map(|value| {
                    text(format!(" {}", value.into()))
                        .text_color(colors::TEXT)
                        .tabular_nums()
                })),
        )
}

/// A row inside an inset, with a hairline above all but the first.
pub(crate) struct Row {
    first: bool,
    on_press: Option<Listener<()>>,
    content: Vec<AnyElement>,
    root: View,
}

impl Row {
    /// A row; `first` leaves the hairline out.
    pub(crate) fn new(first: bool) -> Self {
        Self {
            first,
            on_press: None,
            content: Vec::new(),
            root: view(),
        }
    }

    /// Pressable: it lights up under the finger.
    pub(crate) fn on_press(mut self, listener: Listener<()>) -> Self {
        self.on_press = Some(listener);
        self
    }
}

impl IntoElement for Row {
    fn into_element(self) -> AnyElement {
        self.root
            .min_h(px(48.0))
            .flex_row()
            .items_center()
            .gap_3()
            .when(!self.first, |v| v.border_t_1().border_t_color(colors::LINE))
            .when_some(self.on_press, |v, listener| {
                v.on_press(listener).active(|s| s.bg(colors::RAISED))
            })
            .children(self.content)
            .into_element()
    }
}

/// A [`Button`]'s look.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum ButtonVariant {
    /// Light on the dark page, dark on the light one: the main action.
    #[default]
    Primary,
    /// A raised control.
    Secondary,
    /// Citrus.
    Accent,
    /// Only a rim.
    Ghost,
}

/// A button, 44 pt tall: `Primary` is light (the main action),
/// `Secondary` dark, `Accent` citrus. [`Button::color`] fills it with
/// that color instead.
pub(crate) struct Button {
    label: SharedString,
    variant: ButtonVariant,
    color: Option<AdaptiveColor>,
    icon: Option<IconName>,
    disabled: bool,
    on_press: Listener<()>,
    root: View,
}

impl Button {
    /// A primary button `label` calling `on_press`.
    pub(crate) fn new(label: impl Into<SharedString>, on_press: Listener<()>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            color: None,
            icon: None,
            disabled: false,
            on_press,
            root: view(),
        }
    }

    /// Its look.
    pub(crate) fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Filled with `color` (white text) instead of a variant's look.
    pub(crate) fn color(mut self, color: impl Into<AdaptiveColor>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// An icon before the label.
    pub(crate) fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Takes no presses.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl IntoElement for Button {
    fn into_element(self) -> AnyElement {
        let variant = self.variant;
        let (bg, fg) = match (self.color, variant) {
            (Some(color), _) => (color, AdaptiveColor::from(rgb(0xffffff))),
            (None, ButtonVariant::Primary) => (colors::LIGHT, colors::ON_LIGHT),
            (None, ButtonVariant::Accent) => (colors::ACCENT_FILL, colors::ON_ACCENT),
            (None, ButtonVariant::Ghost) => (AdaptiveColor::TRANSPARENT, colors::TEXT),
            (None, ButtonVariant::Secondary) => (colors::RAISED, colors::TEXT),
        };
        let finish = match variant {
            _ if self.color.is_some() => None,
            ButtonVariant::Primary => Some(Finish::Light),
            ButtonVariant::Accent => Some(Finish::Accent),
            ButtonVariant::Secondary => Some(Finish::Raised),
            ButtonVariant::Ghost => None,
        };
        let rim = matches!(variant, ButtonVariant::Secondary | ButtonVariant::Ghost);
        // The root (the press target) takes the caller's styles: `flex_1`
        // shares a row.
        press_with(
            PressProps {
                disabled: self.disabled,
                label: Some(self.label.clone()),
                ..PressProps::default()
            },
            self.on_press,
        )
        .map(|p| apply_root(p, self.root))
        .child(
            view()
                .h(px(44.0))
                .px(px(18.0))
                .rounded(px(radius::CONTROL))
                .border_curve(BorderCurve::Continuous)
                .flex_row()
                .items_center()
                .justify_center()
                .gap_2()
                .bg(bg)
                .when_some(finish, |v, f| v.finish(f))
                .when(rim, |v| v.border_1().border_color(colors::BORDER))
                .children(self.icon.map(|name| icon(name, 16.0, fg)))
                .child(
                    text(self.label)
                        .text_size(px(15.0))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(fg),
                ),
        )
        .into_element()
    }
}

/// A round icon button (44 pt): light by default, [`RoundButton::secondary`]
/// dark.
pub(crate) struct RoundButton {
    icon: IconName,
    label: SharedString,
    secondary: bool,
    size: f32,
    disabled: bool,
    on_press: Listener<()>,
    root: View,
}

impl RoundButton {
    /// A button of `icon`, labelled `label` for assistive technologies,
    /// calling `on_press`.
    pub(crate) fn new(
        icon: IconName,
        label: impl Into<SharedString>,
        on_press: Listener<()>,
    ) -> Self {
        Self {
            icon,
            label: label.into(),
            secondary: false,
            size: 44.0,
            disabled: false,
            on_press,
            root: view(),
        }
    }

    /// Dark, on lit glass.
    pub(crate) fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    /// Its width and height.
    pub(crate) fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Takes no presses.
    pub(crate) fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl IntoElement for RoundButton {
    fn into_element(self) -> AnyElement {
        let size = self.size;
        let primary = !self.secondary;
        press_with(
            PressProps {
                scale: 0.92,
                disabled: self.disabled,
                label: Some(self.label),
                ..PressProps::default()
            },
            self.on_press,
        )
        .map(|p| apply_root(p, self.root))
        .child(
            view()
                .size(px(size))
                .rounded(px(size / 2.0))
                .items_center()
                .justify_center()
                .map(|v| {
                    if primary {
                        v.bg(colors::LIGHT).finish(Finish::Light)
                    } else {
                        v.bg(colors::RAISED).finish(Finish::Pill).border_1()
                    }
                })
                .child(icon(
                    self.icon,
                    size * 0.42,
                    if primary {
                        colors::ON_LIGHT
                    } else {
                        colors::TEXT
                    },
                )),
        )
        .into_element()
    }
}

/// `root`'s styles on `element` (a builder's root that is a component).
fn apply_root<E: Styled>(mut element: E, mut root: View) -> E {
    let styles = std::mem::take(Styled::style(&mut root));
    *Styled::style(&mut element) = styles;
    element
}

/// A horizontal meter with an optional marker (a limit).
pub(crate) struct Meter {
    value: f32,
    tone: Tone,
    marker: Option<f32>,
}

impl Meter {
    /// Filled to `value` (0 to 1), in the accent.
    pub(crate) fn new(value: f32) -> Self {
        Self {
            value,
            tone: Tone::Accent,
            marker: None,
        }
    }

    /// The fill's tone.
    pub(crate) fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// A mark at `at` (0 to 1).
    pub(crate) fn marker(mut self, at: f32) -> Self {
        self.marker = Some(at);
        self
    }
}

impl IntoElement for Meter {
    fn into_element(self) -> AnyElement {
        view()
            .h(px(6.0))
            .rounded(px(3.0))
            .bg(colors::RAISED)
            .child(
                view()
                    .w(relative(self.value.clamp(0.0, 1.0)))
                    .h(px(6.0))
                    .rounded(px(3.0))
                    .bg(self.tone.color()),
            )
            .children(self.marker.map(|at| {
                view()
                    .absolute()
                    .left(relative(at))
                    .top(px(-3.0))
                    .w(px(2.0))
                    .h(px(12.0))
                    .rounded(px(1.0))
                    .bg(colors::TEXT)
            }))
            .into_element()
    }
}

styled_root!(IconTile, Card, Section, Inset, Row, Button, RoundButton);
content!(Card, Section, Inset, Row);

/// Text in the monospaced family (TS `fontFamily: mono`): API names and
/// identifiers.
pub(crate) fn code(content: impl Into<SharedString>) -> Text {
    text(content.into()).font_family(MONO)
}
