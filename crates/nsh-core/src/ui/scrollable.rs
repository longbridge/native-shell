//! How scroll views and lists scroll ([`Scrollable`]): the behavior props
//! and scroll events `Scroll` and `List` share in TS
//! (`ScrollingHostProps`).

use super::{
    Element, Listener, NativeValue, ScrollEvent, ScrollOffset,
    events::{self, Decode},
    host,
};
use crate::{
    EdgeInsets, EventKind, KeyboardDismissMode, KeyboardTaps, Prop, ScrollProp, SnapAlignment,
};

/// How an element scrolls: [`super::Scroll`], [`super::VirtualList`] and
/// [`super::ListElement`] (the app's rows a component places). The
/// direction and the indicator are [`super::Scroll::horizontal`] /
/// [`super::ListLayout::horizontal`] and their `shows_indicator`.
pub trait Scrollable: Element {
    #[doc(hidden)]
    fn set_scroll_prop(mut self, prop: Prop) -> Self {
        host::set(self.node(), prop);
        self
    }

    #[doc(hidden)]
    fn listen_scroll<T: ?Sized + ToOwned + 'static>(
        mut self,
        kind: EventKind,
        listener: Listener<T>,
        decode: Decode<T>,
    ) -> Self {
        self.node()
            .handlers
            .push((kind, events::typed(listener, decode)));
        self
    }

    /// Stops on multiples of its size (TS: `pagingEnabled`).
    fn paging(self, paging: bool) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::Paging(paging)))
    }

    /// Stops on multiples of `points` (TS: `snapToInterval`; 0: off).
    fn snap_interval(self, points: f32) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::SnapInterval(points)))
    }

    /// Stops at these offsets (TS: `snapToOffsets`).
    fn snap_offsets(self, offsets: Vec<f32>) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::SnapOffsets(offsets)))
    }

    /// Where a snapped item sits in the viewport (TS: `snapToAlignment`).
    fn snap_alignment(self, alignment: SnapAlignment) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::SnapAlignment(alignment)))
    }

    /// A fling moves one snap point at most (TS: `disableIntervalMomentum`).
    fn disable_interval_momentum(self, disable: bool) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::DisableIntervalMomentum(disable)))
    }

    /// How fast a fling slows down: the velocity kept per millisecond (TS:
    /// `decelerationRate`; 0.99 is `"fast"`, 0.998 `"normal"`).
    fn deceleration(self, rate: f32) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::Deceleration(rate)))
    }

    /// The user can scroll it (default yes; TS: `scrollEnabled`).
    fn scroll_enabled(self, enabled: bool) -> Self {
        self.set_scroll_prop(Prop::Native(crate::NativeProp::ScrollEnabled(enabled)))
    }

    /// Bounces at the ends (default yes).
    fn bounces(self, bounces: bool) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::Bounces(bounces)))
    }

    /// Space around the content that scrolls with it.
    fn content_inset(self, insets: EdgeInsets) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::ContentInset(insets)))
    }

    /// Where it starts (set once).
    fn content_offset(self, x: f32, y: f32) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::ContentOffset(x, y)))
    }

    /// Taps while the keyboard shows: dismiss it, or reach the content (TS:
    /// `keyboardShouldPersistTaps`).
    fn keyboard_taps(self, taps: KeyboardTaps) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::KeyboardTaps(taps)))
    }

    /// Dragging dismisses the keyboard (TS: `keyboardDismissMode`).
    fn keyboard_dismiss_mode(self, mode: KeyboardDismissMode) -> Self {
        self.set_scroll_prop(Prop::KeyboardDismissMode(mode))
    }

    /// At most one `on_scroll` per `ms` milliseconds (default: every frame;
    /// TS: `scrollEventThrottle`).
    fn event_throttle(self, ms: f32) -> Self {
        self.set_scroll_prop(Prop::Scroll(ScrollProp::EventThrottle(ms)))
    }

    /// Shows the pull-to-refresh spinner (start it from
    /// [`Self::on_refresh`], clear it when done).
    fn refreshing(self, refreshing: bool) -> Self {
        self.set_scroll_prop(Prop::Refreshing(refreshing))
    }

    /// Follows the scroll offset (along its axis) with a native value:
    /// headers that shrink or fade, parallax, with no render per frame
    /// (TS: `scrollValue`).
    fn scroll_value(mut self, value: &NativeValue) -> Self {
        host::set(self.node(), Prop::ScrollValue(Some(value.node().id())));
        self.node().attrs.bound.push(value.node().clone());
        self
    }

    /// Pulled down to refresh (enables pull to refresh).
    fn on_refresh(self, listener: Listener<()>) -> Self {
        self.listen_scroll(EventKind::Refresh, listener, events::unit)
    }

    /// It moved (at most once per frame, [`Self::event_throttle`]).
    fn on_scroll(self, listener: Listener<ScrollEvent>) -> Self {
        self.listen_scroll(EventKind::Scroll, listener, events::scroll)
    }

    /// The user started dragging it.
    fn on_scroll_begin_drag(self, listener: Listener<ScrollOffset>) -> Self {
        self.listen_scroll(EventKind::ScrollBeginDrag, listener, events::offset)
    }

    /// The user let go.
    fn on_scroll_end_drag(self, listener: Listener<ScrollOffset>) -> Self {
        self.listen_scroll(EventKind::ScrollEndDrag, listener, events::offset)
    }

    /// It started gliding after a fling.
    fn on_momentum_scroll_begin(self, listener: Listener<ScrollOffset>) -> Self {
        self.listen_scroll(EventKind::MomentumScrollBegin, listener, events::offset)
    }

    /// It stopped gliding.
    fn on_momentum_scroll_end(self, listener: Listener<ScrollOffset>) -> Self {
        self.listen_scroll(EventKind::MomentumScrollEnd, listener, events::offset)
    }

    /// It came to rest.
    fn on_scroll_end(self, listener: Listener<ScrollOffset>) -> Self {
        self.listen_scroll(EventKind::ScrollEnd, listener, events::offset)
    }
}
