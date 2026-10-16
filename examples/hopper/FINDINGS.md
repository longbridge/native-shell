# Findings from building Hopper Lab

Hopper Lab was written against nsh as it is, without changing the runtime, to see what an app of this kind needs.
These are the gaps it ran into, each with what the app does instead. Fix them in the runtime and delete the
workaround here in the same change.

## Open

| # | Area | Finding | Workaround in the app |
|---|---|---|---|
| 1 | Mobile UI | `nsh/ui/mobile` has no switch, slider, segmented control, stepper styled for dark themes, menu or tab bar; each app writes them. | `ui.tsx` and the tab bar in `App.tsx` (about 500 lines). |
| 2 | Touch, layout | There is no popover / anchored overlay primitive (children outside their parent can be touched now, see below). | The easing menu lives in a box that covers both the picker and the curve (`cards/ThrottleProfile.tsx`). |

## Fixed in the runtime, workaround still in the app

| Finding | Now | Workaround to remove |
|---|---|---|
| A child outside its parent's bounds draws but cannot be touched. | Hit testing reaches children that overflow a parent with visible overflow (`MountItem::Overflow`). | The easing menu's covering box (`cards/ThrottleProfile.tsx`). |
| `locationX` is relative to the touched view, and there is no `measure()`. | `ref.measure()` gives the view's `pageX` / `pageY`. | `pointerEvents="none"` on the slider's children (`ui.tsx` `Slider`). |
| No keyboard for signed numbers. | `keyboardType="numbers-and-punctuation"`. | The ± button (`cards/FlightPlan.tsx`). |
| `borderColor` cannot be animated natively. | `borderColor`, `color` and shadows bind to native values. | The knob animates its background instead (`ui.tsx` `Switch`). |
| QuickJS has no `Intl`. | `Intl.NumberFormat` / `DateTimeFormat` and `toLocaleString`, formatted in Rust. | The regex formatter. |
| No `isPointInPath`. | `ctx.isPointInPath` / `isPointInStroke`, answered in Rust. | Hand-written hit tests (`cards/Mass.tsx`, `cards/Rcs.tsx`). |

## Fixed

- No direction-locked pan: the `panActivation` prop decides natively which of a pan and its scroll view takes the
  touch, and the negotiation callbacks get a `gestureState`.
- Mac Catalyst header band and colors: the content sits below the measured header, and the bar stays in the window.
- Type checking needed `tsc` from npm: `nsh check`.
- A hot update of the entry popped pushed screens and reset state: top-level signals and navigators keep their state,
  and an unchanged `mount(App)` is kept (editing `App` itself still resets the state inside it).
