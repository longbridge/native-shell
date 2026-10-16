---
title: Elements
description: The host elements every nsh app is built from, and the props, events and ref methods they all share.
order: 1
---

# Elements

Host elements are the lowercase JSX tags: `<view>`, `<text>`, `<image>` and the rest. Each one is a native view (a UIView
subclass on iOS and Mac Catalyst, an Android `View`), created and laid out by the nsh runtime. Everything an app shows is
made of them: the components of [`nsh/ui/mobile`](../components/index.md) are drawn with the same elements.

```tsx
import { mount, signal } from "nsh";

function App() {
  const count = signal(0);
  return (
    <view style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 12 }}>
      <text style={{ fontSize: 24 }}>Tapped {count()} times</text>
      <view onPress={() => count.update((n) => n + 1)} activeOpacity={0.6} class="rounded-lg bg-blue-600 px-4 py-2">
        <text class="text-white">Tap</text>
      </view>
    </view>
  );
}

mount(App);
```

Built-in tags need no import. Capitalized tags are components; a component in scope may also be written in kebab case
(`Button` as `<button>`), and an app's binding wins over a built-in of the same name: importing `Modal` from
`nsh/ui/mobile` makes `<modal>` that component. See [JSX and components](../docs/jsx-and-components.md).

## All elements

| Tag | Native view (iOS / Android) | Typed component | Page |
|---|---|---|---|
| `<view>` | UIView / ViewGroup | | [view](./view.md) |
| `<text>` | CoreText view / StaticLayout view | | [text](./text.md) |
| `<image>` | a view with an image layer / an image view | | [image](./image.md) |
| `<scroll>` | UIScrollView / ScrollView | | [scroll](./scroll.md) |
| `<input>` | UITextField or UITextView / EditText | | [input](./input.md) |
| `<list>` | UIScrollView / ScrollView with recycled cells | `List` (`nsh/list`) | [list](./list.md) |
| `<canvas>` | a view replaying a 2D drawing | `Canvas` (`nsh/canvas`) | [canvas](./canvas.md) |
| `<web-view>` | WKWebView / android.webkit.WebView | `WebView` (`nsh/webview`) | [web-view](./web-view.md) |
| `<video>` | AVPlayerLayer / MediaPlayer on a TextureView | `Video` (`nsh/video`) | [video](./video.md) |
| `<modal>` | presented view controller / Dialog | `Modal` (`nsh/ui/mobile`) | [modal](./modal.md) |
| `<input-accessory>` | a bar over the keyboard (iOS only) | `InputAccessoryView` (`nsh/ios`) | [nsh/ios](../modules/nsh-ios.md) |
| `<if>`, `<each>` | none: control flow | | [control flow](./control-flow.md) |

The tags without a typed form of their own (`<canvas>`, `<web-view>`, `<video>`, `<modal>`, `<input-accessory>`) accept
any prop in TypeScript. Prefer their components: those are typed and add what the bare tag lacks (a `draw` function, a
`visible` prop, safe-area insets).

> [!NOTE]
> A string is only drawn inside a `<text>`. A string child of a `<view>` creates no native view and takes no part in
> layout.

## Common props

Every host element accepts the props below (the `CommonProps` type). Element pages list only their own props.

### Styling

| Prop | Type | Description |
|---|---|---|
| `style` | `StyleProp<ViewStyle>` (`TextStyle` for `<text>` and `<input>`, `ImageStyle` for `<image>`) | Inline style, or an array of styles merged left to right (falsy entries skipped). Wins over `class`. See the [style reference](../modules/style-reference.md). |
| `class` | `string` | Tailwind classes, resolved natively (`"flex-row gap-2 p-4 dark:bg-zinc-900"`). Later classes win over earlier ones. See [Tailwind](../docs/tailwind.md). |
| `classList` | `Record<string, boolean \| null \| undefined>` | Classes applied while their value is truthy (`{ "opacity-50": busy() }`). |
| `disabled` | `boolean` | Ignores presses (no press feedback either), applies `disabled:` classes and disables controls such as inputs. |

### Presses and feedback

Press handling runs natively: the platform recognizes the press and shows the feedback, and JS only hears about it
afterwards. Only the innermost pressable view responds; inside scroll views a press-in waits about 130 ms so a scroll can
start first.

| Prop | Type | Default | Description |
|---|---|---|---|
| `onPress` | `() => void` | | A tap ended inside the view. |
| `onLongPress` | `() => void` | | The finger stayed down for `delayLongPress`. |
| `onPressIn` | `() => void` | | A finger went down on the view. |
| `onPressOut` | `() => void` | | The finger lifted or left. |
| `delayLongPress` | `number` | `500` | Milliseconds before a press becomes a long press. |
| `activeOpacity` | `number` | | The view's opacity while pressed. |
| `underlayColor` | `ColorValue` | | The background while pressed; the children fade to `underlayOpacity`. |
| `underlayOpacity` | `number` | `0.85` | The children's opacity while `underlayColor` shows. |
| `hitSlop` | `number \| { top?, right?, bottom?, left? }` | | Extends the touch area beyond the bounds, in points. |
| `pointerEvents` | `"auto" \| "none" \| "box-none" \| "box-only"` | `"auto"` | Which touches the view and its children receive: `"none"` neither, `"box-none"` only the children, `"box-only"` only the view itself. |

`active:` Tailwind classes also apply while a view is pressed, with no JS round trip.

### Layout and identity

| Prop | Type | Description |
|---|---|---|
| `onLayout` | `(e: LayoutEvent) => void` | The view's frame relative to its parent, `{ x, y, width, height }`, after each layout that changes it. |
| `ref` | `YNode \| (node: YNode) => void` | The native node (see [ref methods](#ref-methods)). `ref={el}` assigns a variable, `ref={fn}` calls a function. |
| `nativeID` | `string` | Names the view for others, e.g. the `InputAccessoryView` an input refers to. |
| `testID` | `string` | An identifier for UI tests (on Android, the view's resource id). |

### Gestures driving native values

These props bind a gesture to [native animated values](../modules/nsh-animated.md) that move on the UI thread, with no
JS per frame. See [touch and gestures](../docs/touch-and-gestures.md).

| Prop | Type | Description |
|---|---|---|
| `pan` | `{ x?: NativeValue; y?: NativeValue }` | Drags drive these values (points). |
| `onPanStart` | `(e: PanEvent) => void` | A drag started. |
| `onPanEnd` | `(e: PanEvent) => void` | A drag ended, with the release velocity (e.g. to start a `decay`). |
| `pinch` | `{ scale?: NativeValue }` | Two-finger pinches drive `scale`. |
| `onPinchStart` / `onPinchEnd` | `(e: { scale: number; velocity: number }) => void` | A pinch started / ended. |
| `rotation` | `{ angle?: NativeValue }` | Two-finger rotations drive `angle` (degrees). |
| `onRotationStart` / `onRotationEnd` | `(e: { rotation: number; velocity: number }) => void` | A rotation started / ended. |

`PanEvent` is `{ translationX, translationY, velocityX, velocityY, valueX?, valueY? }`: the translation since the start,
the velocity in points per second, and the bound values at that moment (a drag can grab a value mid-animation).

### Raw touches and the responder system

For gestures the props above do not cover, every element takes the touch and responder props:
`onTouchStart`, `onTouchMove`, `onTouchEnd`, `onTouchCancel` (and their `…Capture` forms),
`onStartShouldSetResponder`, `onMoveShouldSetResponder` (and `…Capture`), `onResponderGrant`, `onResponderReject`,
`onResponderStart`, `onResponderMove`, `onResponderEnd`, `onResponderRelease`, `onResponderTerminate` and
`onResponderTerminationRequest`. Each handler gets a `GestureResponderEvent`
(`{ identifier, locationX, locationY, pageX, pageY, target, timestamp, touches, changedTouches, nativeEvent,
stopPropagation() }`). Returning `true` from `onResponderGrant` keeps native scroll views from taking the touches.
`PanResponder` in [`nsh/gesture`](../modules/nsh-gesture.md) builds on them.

### Accessibility

| Prop | Type | Description |
|---|---|---|
| `accessible` | `boolean` | The view is one element for assistive technologies; its children are not exposed separately. |
| `accessibilityLabel` | `string` | What screen readers read. Views without a label read their texts. |
| `accessibilityHint` | `string` | What activating the element does. |
| `accessibilityRole` / `role` | `AccessibilityRole` | What the element is (`"button"`, `"header"`, `"link"`, `"switch"`, …). |
| `accessibilityState` | `{ disabled?, selected?, checked?, busy?, expanded? }` | The state assistive technologies report (`checked` may be `"mixed"`). |
| `accessibilityValue` | `{ min?, max?, now?, text? }` | The value of a range or textual element. |
| `accessibilityElementsHidden` | `boolean` | Hides the view and its subtree from assistive technologies. |
| `importantForAccessibility` | `"auto" \| "yes" \| "no" \| "no-hide-descendants"` | Whether the view is exposed. |
| `accessibilityActions` | `{ name: string; label?: string }[]` | Custom actions offered to assistive technologies. |
| `onAccessibilityAction` | `(e: { actionName: string }) => void` | The user invoked one of `accessibilityActions`. |

See [accessibility](../docs/accessibility.md).

### Platform-only props

Props that exist on one platform go in an object named after it, so the shared props stay the same everywhere. Only the
current platform's object is sent; the others are ignored.

| Prop | Description |
|---|---|
| `ios={{…}}` | iOS and Mac Catalyst. `<input>`, `Modal` and `WebView` add their own keys. |
| `android={{…}}` | Android. Every element takes `ripple`, `elevation` and `accessibilityLiveRegion` (below). |
| `desktop={{…}}` | Desktop platforms; ignored on mobile. |

| `android` key | Type | Description |
|---|---|---|
| `ripple` | `{ color?, borderless?, radius?, foreground? }` | The material ripple while pressed. |
| `elevation` | `number` | The material shadow (points), from the view's shape. |
| `accessibilityLiveRegion` | `"none" \| "polite" \| "assertive"` | How changes of the view are announced to TalkBack. |

```tsx
<view onPress={save} android={{ ripple: { color: "#00000022" }, elevation: 2 }} class="rounded-lg bg-white p-4">
  <text>Save</text>
</view>
```

See [platform-specific code](../docs/platform-specific.md).

## Ref methods

`ref` receives a `YNode`. Its methods send commands to the native view, in order with the changes made before them in the
same turn. Each element page says which apply to it.

| Method | Applies to | Description |
|---|---|---|
| `focus()` / `blur()` | `<input>` | Focuses (shows the keyboard) / unfocuses. |
| `setSelection(start, end?)` | `<input>` | Selects `start..end` in UTF-16 units; a caret without `end`. |
| `scrollTo({ x?, y?, animated? })` | `<scroll>`, `List` | Scrolls to an offset. |
| `zoomTo({ scale, x?, y?, animated? })` | zoomable `<scroll>` | Zooms to `scale` around a content point. |
| `scrollToOffset({ offset, animated? })` | `List` | Scrolls to an offset along the list's axis. |
| `scrollToIndex({ index, animated?, viewPosition?, viewOffset? })` | `List` | Scrolls an item into view. |
| `scrollToEnd({ animated? })` | `List` | Scrolls to the end of the content. |
| `reload()`, `goBack()`, `goForward()`, `stopLoading()`, `injectJavaScript(script)`, `postMessage(data)` | `WebView` | Controls the page. |
| `seek(seconds)` | `Video` | Jumps to a time. |
| `command(name, args?)` | native components | Calls a command of a registered native component ([`nsh/native`](../modules/nsh-native.md)). |

```tsx
import { type YNode } from "nsh";

let input!: YNode;
<view onPress={() => input.focus()}>
  <input ref={input} placeholder="Tap the row to type" />
</view>;
```

## Why handlers never block the UI

The main thread never waits for JS. When the system needs an answer on the spot (may this text change, may the back
button pop, may this sheet close), nsh answers it natively from props set beforehand (`maxLength`, `preventBack`,
`dismissible`, `pointerEvents`, …), and the event reaches JS afterwards. This is why such behavior is a prop, not a
return value of a handler.
