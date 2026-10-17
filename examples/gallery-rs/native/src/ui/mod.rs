//! What the demos share (as the TSX gallery's `ui.tsx` and `icons.tsx`):
//! the gallery's look ([`theme`]: colors for both appearances, tones,
//! radii, motion, finishes), its icons, and the building blocks every demo
//! is made of.
//!
//! - Stateful, native motion (components: `X::element(XProps { .. })`,
//!   their events through `.on_event(..)`, or the helper functions):
//!   [`Press`] / [`press`], [`Appear`] / [`appear`], [`Toggle`] /
//!   [`toggle`], [`Segmented`] / [`segmented`], [`Icon`] / [`icon`].
//! - Builders (`Card::new().title(..).child(..)`, styled at their root):
//!   [`Card`], [`Section`], [`Inset`], [`Row`], [`Figure`], [`Badge`],
//!   [`Stat`], [`Meter`], [`IconTile`], [`Button`], [`RoundButton`]; and
//!   [`caption`], [`legend`], [`code`], [`header`].

pub(crate) mod legacy;

// The demos use these as they move to the shared design; until each
// piece has a user, it is expected to be unused (the expectation then
// fails, and goes).
#[expect(dead_code, reason = "used by the demos as they are restyled")]
mod animated;
#[expect(dead_code, reason = "used by the demos as they are restyled")]
mod controls;
mod icons;
#[expect(dead_code, reason = "used by the demos as they are restyled")]
mod parts;
#[expect(dead_code, reason = "used by the demos as they are restyled")]
mod theme;

#[expect(unused_imports, reason = "used by the demos as they are restyled")]
pub(crate) use self::{
    animated::{
        Appear, AppearProps, Press, PressEvent, PressProps, PressRole, appear, press, press_with,
    },
    controls::{
        Segment, Segmented, SegmentedEvent, SegmentedProps, SegmentedSize, Toggle, ToggleEvent,
        ToggleProps, segmented, toggle,
    },
    icons::{Icon, IconName, IconProps, Tint, icon},
    parts::{
        Badge, Button, ButtonVariant, Card, Figure, IconTile, Inset, Meter, RoundButton, Row,
        Section, Stat, caption, code, header, legend,
    },
    theme::{Finish, Finished, MONO, Tone, colors, inset_shadow, motion, radius, rgb_a, tints},
};
