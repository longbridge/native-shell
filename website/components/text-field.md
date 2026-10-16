---
title: TextField
description: A labeled single-line text input with a clear button, focus ring and secure entry.
order: 11
demo: ui-text-field
---

# TextField

`TextField` is a single-line text input with an optional label above it, a border that changes color while focused,
and a clear button. It is built on the [`<Input>`](../elements/input.md) element.

```tsx
import { TextField } from "nsh/ui/mobile";
```

## Usage

```tsx
import { mount, signal, Text, View } from "nsh";
import { TextField } from "nsh/ui/mobile";

function App() {
  const name = signal("");
  return (
    <View style={{ flex: 1, padding: 16, paddingTop: 48, gap: 12 }}>
      <TextField label="Name" placeholder="Your name" value={name()} onChange={(v) => name.set(v)} />
      <Text>{`Hello, ${name() || "nobody"}`}</Text>
    </View>
  );
}

mount(App);
```

## Controlled and uncontrolled

- **Controlled**: pass `value` and update it in `onChange`. The field shows `value`.
- **Uncontrolled**: leave out `value`. The field keeps its own text, starting at `defaultValue`, and reports changes
  with `onChange`.

```tsx
<TextField label="Notes" defaultValue="Draft" onChange={(text) => save(text)} />
```

## Variants

```tsx
<TextField label="Password" placeholder="At least 8 characters" secure />
<TextField label="Code" clearable={false} />
<TextField label="Account" value="ada@example.com" disabled />
```

- `secure` hides what is typed, for passwords.
- `clearable` (on by default) shows a × button at the end while there is text. Pressing it clears the text, reports
  `""` with `onChange` and keeps the input focused, so the user can type again.
- `disabled` makes the field read-only, dims it to 50% opacity and hides the clear button.

## Submitting

`onSubmit` is called with the text when the user presses the keyboard's return key:

```tsx
<TextField
  placeholder="Search"
  value={query()}
  onChange={(v) => query.set(v)}
  onSubmit={(text) => search(text)}
/>
```

> [!NOTE]
> TextField covers the common case. For keyboard types, return key labels, autofill, multiline text or a ref with
> `focus()`, use the [`<Input>`](../elements/input.md) element directly. See also
> [Forms and keyboard](../docs/forms-and-keyboard.md).

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `label` | `string \| null` | none | Shown above the input. |
| `placeholder` | `string \| null` | none | Shown while the input is empty. |
| `value` | `string \| null` | none (uncontrolled) | The text, when the app controls it. |
| `defaultValue` | `string` | `""` | The first text, when the app does not control it. |
| `secure` | `boolean` | `false` | Hides what is typed (passwords). |
| `clearable` | `boolean` | `true` | Shows a clear button while there is text. |
| `disabled` | `boolean` | `false` | Not editable, 50% opacity. |
| `class`, `classList`, `style`, `testID`, `onLayout`, `ref` | | | Applied to the field's root view (the column holding the label and the input box). |

## Events

| Event | Payload | When |
|---|---|---|
| `onChange` | `string`: the new text | The user typed, deleted or pressed the clear button. |
| `onSubmit` | `string`: the text | The user pressed the return key. |

## Theming

| Part | Theme key |
|---|---|
| Label | `muted.foreground`, small text |
| Input box | `background`, corner radius `radius`, at least 44 pt tall |
| Border | `input.border`; `ring` while focused |
| Text | `foreground` |
| Placeholder | `muted.foreground` |
| Selection | `primary.background` |
| Clear button | `muted.background`; `secondary.active.background` while pressed |

See [Theming](./index.md#theming).

## Accessibility

The input is a native text field, so screen readers announce it as editable text. The clear button has the `button`
role and the label "Clear", and its touch area extends 8 pt beyond its 24 pt circle. The `label` is a separate text
above the input; it is not attached to the input as its accessibility label.

## Related

- [Input](../elements/input.md): the element TextField is built on.
- [Forms and keyboard](../docs/forms-and-keyboard.md), [Keyboard](./keyboard.md),
  [KeyboardAvoidingView](./keyboard-avoiding-view.md).
