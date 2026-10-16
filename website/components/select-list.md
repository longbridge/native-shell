---
title: SelectList
description: A scrolling list of string options with a check mark on the chosen one, controlled or uncontrolled.
order: 7
demo: ui-select-list
---

# SelectList

`SelectList` shows a scrolling list of text options and marks the chosen one with a check mark. The core renders the
rows from your array of strings, so a list of thousands of options costs about as much as the visible rows, and no
JS runs while scrolling.

```tsx
import { SelectList } from "nsh/ui/mobile";
```

## Usage

```tsx
import { mount, signal, Text, View } from "nsh";
import { SelectList } from "nsh/ui/mobile";

const languages = ["English", "Deutsch", "Español", "Français", "日本語", "中文"];

function App() {
  const chosen = signal<number | null>(0);
  return (
    <View style={{ flex: 1, paddingTop: 48 }}>
      <Text style={{ padding: 16 }}>{`Language: ${chosen() === null ? "none" : languages[chosen()!]}`}</Text>
      <SelectList options={languages} value={chosen()} onSelect={(i) => chosen.set(i)} />
    </View>
  );
}

mount(App);
```

SelectList fills its parent (`flex: 1`). To show it inside a scroll view or next to other content, give it a parent
with a fixed height:

```tsx
<View style={{ height: 240, borderRadius: 12, overflow: "hidden" }}>
  <SelectList options={cities} onSelect={(i) => console.log(cities[i])} />
</View>
```

## Controlled and uncontrolled

- **Controlled**: pass `value` (the chosen index, or `null` for none) and update it in `onSelect`. The check mark
  follows `value`. If you do not update `value`, the check mark does not move.
- **Uncontrolled**: leave out `value`. The list keeps the choice itself and moves the check mark on each press.
  `onSelect` still reports each choice.

`onSelect` is called on every press, also when the user presses the option that is already chosen.

## Rows

Each row is at least 48 pt tall, with 16 pt horizontal padding, the option's text, and a check mark (✓) at the end of
the chosen row. Rows are separated by 1 pt dividers and highlighted while pressed.

> [!NOTE]
> Options are matched by their text when the array changes, so rows keep their place when you insert or remove
> options. Use unique option strings.

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `options` | `string[]` | required | The options, in order. |
| `value` | `number \| null` | none (uncontrolled) | The index of the chosen option, when the app controls it. |
| `class`, `classList`, `style`, `testID`, `onLayout`, `ref` | | | Applied to the list's root view. |

## Events

| Event | Payload | When |
|---|---|---|
| `onSelect` | `number`: the option's index | The user pressed an option. |

## Theming

| Part | Theme key |
|---|---|
| Row background | `background` |
| Row while pressed | `list.active.background` |
| Text | `foreground` |
| Check mark | `primary.background` |
| Dividers | `border` |

See [Theming](./index.md#theming).

## Accessibility

Each row has the `button` role, and the chosen row is reported as **selected**. Screen readers read the option's text.

## Related

- [ListView](./list-view.md): rows you render yourself.
- [Sheet](./sheet.md): show a SelectList in a bottom sheet as a picker.
- [ActionSheet](./action-sheet.md): a short list of actions in a system dialog.
