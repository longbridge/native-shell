---
title: Button
description: A press target with a label in four variants, with native press feedback and a disabled state.
order: 2
demo: ui-button
---

# Button

`Button` is a press target with a text label. It is filled with a theme color, darkens while pressed and reports
presses with `onPress`. It looks the same on iOS and Android.

```tsx
import { Button } from "nsh/ui/mobile";
```

## Usage

```tsx
import { mount, signal, Text, View } from "nsh";
import { Button } from "nsh/ui/mobile";

function App() {
  const count = signal(0);
  return (
    <View style={{ flex: 1, padding: 16, gap: 12, justifyContent: "center" }}>
      <Text>{`Pressed ${count()} times`}</Text>
      <Button title="Press me" onPress={() => count.set(count() + 1)} />
    </View>
  );
}

mount(App);
```

The button lays its label out in a row, centered, with 16 pt of horizontal and 12 pt of vertical padding. In a column
it stretches to the column's width, like any view. Pass `class` or `style` to size or round it differently:

```tsx
<View style={{ flexDirection: "row", gap: 8 }}>
  <Button class="flex-1" title="Cancel" variant="secondary" onPress={cancel} />
  <Button class="flex-1" title="Save" onPress={save} />
</View>
```

## Variants

```tsx
<Button title="Save" />                         {/* primary (default) */}
<Button title="Not now" variant="secondary" />
<Button title="Delete" variant="danger" />
<Button title="Learn more" variant="plain" />
```

| Variant | Background | Pressed | Label |
|---|---|---|---|
| `primary` (default) | `primary.background` | `primary.active.background` | `primary.foreground` |
| `secondary` | `secondary.background` | `secondary.active.background` | `secondary.foreground` |
| `danger` | `danger.background` | `danger.active.background` | `danger.foreground` |
| `plain` | transparent | `accent.background` | `foreground` |

Use `danger` for destructive actions and `plain` for low-emphasis actions such as a card footer's secondary action.

## Disabled

```tsx
<Button title="Submit" disabled={!valid()} onPress={submit} />
```

A disabled button is shown at 50% opacity, does not respond to presses and shows no press feedback.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `title` | `string` | required | The label. |
| `variant` | `"primary" \| "secondary" \| "danger" \| "plain"` | `"primary"` | The look (see [Variants](#variants)). |
| `disabled` | `boolean` | `false` | No presses, no feedback, 50% opacity. |
| `accessibilityLabel` | `string \| null` | `title` | What screen readers read, when `title` is not enough. |
| `class`, `classList`, `style`, `testID`, `onLayout`, `ref` | | | Applied to the button's root view. `class` and `style` override the button's own styles. |

## Events

| Event | Payload | When |
|---|---|---|
| `onPress` | none | The button was pressed (released inside it). Not sent while `disabled`. |

## Theming

The label uses the theme's `font.size` (16 pt by default) with a semibold weight. The corners use `radius` (12 pt by
default). The colors come from the theme keys in the [variants table](#variants). See
[Theming](./index.md#theming).

## Accessibility

The button has the `button` role. Screen readers read `accessibilityLabel` if you set it, otherwise `title`. While
`disabled`, the button is reported as disabled.

## Related

- [View](../elements/view.md): `onPress`, `activeOpacity` and `underlayColor` for custom press targets.
- [Touch and gestures](../docs/touch-and-gestures.md).
