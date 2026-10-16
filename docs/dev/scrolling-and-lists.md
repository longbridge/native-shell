# Scrolling and lists

- **Scrolling**: `pagingEnabled`, `snapToInterval` / `snapToOffsets` / `snapToAlignment`, `disableIntervalMomentum`,
  `decelerationRate`, `scrollEnabled`, `bounces`, `contentInset`, `contentOffset`, `keyboardShouldPersistTaps`
  (`never` / `handled` / `always`), `onScroll` (+ `scrollEventThrottle`; at most once per engine turn), `onScrollBeginDrag` /
  `EndDrag`, `onMomentumScrollBegin` / `End` (a glide caught by a new drag ends with `onMomentumScrollEnd`). Snapping is decided
  by `ScrollBehavior::snap_target` (core) on iOS and its Java port on Android, from the release offset, the projected glide and
  the velocity. Lists keep the visible rows in place when items are inserted above them (`splice`).
- **Zooming** (`<scroll>`, not lists): `maximumZoomScale` above `minimumZoomScale` makes the content zoomable; `zoomScale`
  zooms when it changes, `bouncesZoom`, `pinchGestureEnabled`, `centerContent` (content smaller than the viewport is centered);
  ref `zoomTo({ scale, x, y, animated })` (content point `x`, `y` in the middle); `onScroll` reports `zoomScale` and the zoomed
  content size. iOS: the children live in a `NshZoomView` (`zoom.rs`) that UIKit scales (`viewForZoomingInScrollView:`), its
  bounds the unzoomed content size, so the core's frames stay valid; texts and canvases are drawn again at the zoomed
  resolution when a zoom ends. Android has no zooming scroll view: `ScrollBox` switches to its `ZScroll`, which scales the
  content around its top left and scrolls both ways itself (ScaleGestureDetector around the focal point, panning with any
  number of fingers, OverScroller flings, rubber bands past the edges and the zoom limits). Pinches start only after
  Android's minimum span (about 27 mm between the fingers). Zoom reports go to the core (`EngineControl::zoom`).
- **List** props: `each`, `estimatedItemSize`, `numColumns`, `inverted` (chat: item 0 at the bottom, starts at the end and keeps its
  distance from the end as content changes), `stickyHeaderIndices` (one column, not inverted), `ListHeaderComponent` /
  `ListFooterComponent` / `ListEmptyComponent` / `ItemSeparatorComponent` (a component or an element), `contentContainerStyle`
  (padding and gaps, numbers), `initialScrollIndex`, `onEndReached` + `onEndReachedThreshold`, `onViewableItemsChanged` +
  `viewabilityConfig` (`itemVisiblePercentThreshold` / `viewAreaCoveragePercentThreshold` / `minimumViewTime`), `keyExtractor`,
  `refreshing` / `onRefresh`, `scrollValue`. Positions, columns, sticky travel and viewability are all computed in the core; sticky cells
  are pinned natively on every scroll (`MountItem::StickyCells`: translate by `clamp(offset - start, 0, end - start)`).
- **List internals** (the window, binding cells, recycling): `crates/nsh-core/src/core/list.rs`.
- **Rust components**: the same props are `Scrollable`'s methods (`scroll()`, `virtual_list(..)`) and
  `VirtualList`'s (header, footer, empty view, separators, sticky rows, viewability); `docs/dev/components.md`.
