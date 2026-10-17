# Findings from building Hopper Lab

Hopper Lab was written against nsh as it is, without changing the runtime, to see what an app of this kind needs.
These are the gaps it ran into, each with what the app does instead. Fix them in the runtime and delete the
workaround here in the same change.

| # | Area | Finding | Workaround in the app |
|---|---|---|---|
| 1 | Touch, layout | A child placed outside its parent's bounds (`position: "absolute"`, `overflow` visible) draws but cannot be touched: UIKit only hit-tests inside the parent's bounds. A dropdown under a picker row ignored taps and the tap went to what was behind it. There is no popover / anchored overlay primitive either. | The easing menu lives in a box that covers both the picker and the curve (`cards/ThrottleProfile.tsx`), with a sibling backdrop to close it. |
| 2 | Touch | `locationX` / `locationY` are relative to the view the finger went down on (React Native's semantics), and there is no `measure()` / `measureInWindow()` to turn `pageX` into a position in another view. A slider whose knob is a child got positions relative to the knob. | Every child of the slider has `pointerEvents="none"` (`ui.tsx` `Slider`), so the target is always the slider. |
| 3 | Gestures | A responder that keeps the touch (grant returns `true`) blocks the scroll view around it from the first pixel; there is no direction-locked pan (start only after moving horizontally, let vertical moves scroll), like gesture-handler's `activeOffsetX` / `failOffsetY`. Charts that scrub horizontally make the page hard to scroll on a phone. | None: charts take the touch (`ui.tsx` `track`). |
| 4 | Input | No keyboard for signed numbers: `number-pad` and `numeric` both map to iOS `NumberPad`, which has no minus key (`numbers-and-punctuation` is missing). | A ± button in the field flips the sign (`cards/FlightPlan.tsx`). |
| 5 | Animation | `borderColor` cannot be bound to a native value (`warn js::protocol style borderColor cannot be animated natively`); only opacity, background color and transforms can. A switch's knob ring could not animate. | The knob animates its background color instead (`ui.tsx` `Switch`). |
| 6 | Runtime | QuickJS has no `Intl`: `toLocaleString("en-US")` does not group thousands. | A regex formatter where needed. |
| 7 | Mobile UI | `nsh/ui/mobile` has no switch, slider, segmented control, stepper styled for dark themes, menu or tab bar; each app writes them. | `ui.tsx` and the tab bar in `App.tsx` (about 500 lines). |
| 8 | Canvas | No `isPointInPath`: tapping a shape in a drawing means redoing its geometry for hit testing. | Hand-written hit tests in the drawings' design units (`cards/Mass.tsx`, `cards/Rcs.tsx`). |
| 9 | Tooling | Type checking needs `tsc` from npm (`bunx tsc -p .`), although the toolchain otherwise needs no Node: an `nsh check` would close that. | Checked with `bunx --bun tsc -p examples/hopper`. |
| 10 | Hot reload | A hot update of the entry renders the app again: a pushed screen pops back to the root and component state (a resized landing zone) resets. | None. |
