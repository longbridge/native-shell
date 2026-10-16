---
title: Card
description: A rounded surface with an optional header and footer around your content, elevated or outlined.
order: 3
demo: ui-card
---

# Card

`Card` is a rounded surface that groups related content. It can have a header above the content and a footer below
it. The header, the content and the footer are your JSX; the card adds the surface, the padding and the divider.

```tsx
import { Card } from "nsh/ui/mobile";
```

## Usage

```tsx
import { mount, Text, View } from "nsh";
import { Button, Card } from "nsh/ui/mobile";

function App() {
  return (
    <View style={{ flex: 1, padding: 16, paddingTop: 48 }}>
      <Card
        header={<Text>Weekly summary</Text>}
        footer={<Button title="Details" variant="plain" onPress={() => console.log("details")} />}
      >
        <Text>You walked 42 km this week.</Text>
        <Text>That is 8 km more than last week.</Text>
      </Card>
    </View>
  );
}

mount(App);
```

## Layout

- **Header**: 16 pt padding at the sides and the top. Text in the header is semibold and 1 pt larger than the theme's
  `font.size`, so a plain `<Text>` works as a title. A header can be any JSX, for example an avatar and a name in a
  row.
- **Content**: the children are stacked in a column with 8 pt gaps. With a header, the content has 8 pt padding at the
  top and 16 pt at the sides and the bottom. Without a header, it has 16 pt padding on all sides.
- **Footer**: a row aligned to the end, with 8 pt gaps, 16 pt horizontal and 12 pt vertical padding, and a divider
  above it. Put actions here.

```tsx
<Card
  variant="outlined"
  header={
    <View style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
      <View style={{ width: 32, height: 32, borderRadius: 16, backgroundColor: "#a78bfa" }} />
      <Text>Ada Lovelace</Text>
    </View>
  }
  footer={
    <View style={{ flexDirection: "row", gap: 8 }}>
      <Button title="Ignore" variant="plain" onPress={ignore} />
      <Button title="Reply" onPress={reply} />
    </View>
  }
>
  <Text>Can we move the meeting to Thursday?</Text>
</Card>
```

The header and footer only exist while you pass them: `header={cond() ? <Text>…</Text> : undefined}` adds and
removes the header, and the content's padding changes to match.

## Variants

| Variant | Look |
|---|---|
| `elevated` (default) | A 1 pt border and a small shadow. |
| `outlined` | A 1 pt border, no shadow. |

```tsx
<Card>…</Card>
<Card variant="outlined">…</Card>
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `variant` | `"elevated" \| "outlined"` | `"elevated"` | Whether the card has a shadow. |
| `header` | `JSX.Element` | none | Above the content: a title or an avatar row. Its text is semibold. |
| `footer` | `JSX.Element` | none | Below the content, after a divider: actions, aligned to the end. |
| `children` | `JSX.Element` | none | The content. |
| `class`, `classList`, `style`, `testID`, `onLayout`, `ref` | | | Applied to the card's root view. `class` and `style` override the card's own styles. |

`Card` has no events. Put handlers on your own content.

## Theming

| Part | Theme key |
|---|---|
| Surface | `group_box.background` |
| Text color (inherited by the content) | `group_box.foreground` |
| Border and footer divider | `border` |
| Corner radius | `radius.lg` (16 pt by default) |
| Header text size | `font.size` + 1 |

See [Theming](./index.md#theming).

## Accessibility

The card adds no role or label. Screen readers read the header, content and footer in order, as separate elements.

## Related

- [Group](./group.md): rows in a rounded box with dividers.
- [Button](./button.md): actions in a footer.
