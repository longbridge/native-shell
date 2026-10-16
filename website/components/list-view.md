---
title: ListView
description: A virtualized, recycling list of your rows, padded and divided, in a plain or inset style.
order: 5
demo: ui-list-view
---

# ListView

`ListView` is a scrolling list of rows that you render. Like [List](../elements/list.md), it only creates the rows
near the screen and reuses their cells as you scroll, so a list of thousands of items costs about as much as the
visible rows. ListView adds the look: padding around each row, dividers between rows and an optional inset style.

```tsx
import { ListView } from "nsh/ui/mobile";
```

## Usage

Pass the data as `each` and a row renderer as the child function, the same way as with `List`:

```tsx
import { mount, Text, View } from "nsh";
import { ListView } from "nsh/ui/mobile";

const people = Array.from({ length: 1000 }, (_, i) => ({ id: i, name: `Person ${i + 1}` }));

function App() {
  return (
    <ListView each={people} keyExtractor={(p) => String(p.id)}>
      {(person, index) => (
        <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
          <Text style={{ fontWeight: "600" }}>{person().name}</Text>
          <Text style={{ color: "#737373" }}>{`#${index() + 1}`}</Text>
        </View>
      )}
    </ListView>
  );
}

mount(App);
```

The row function receives accessors: `person()` and `index()`. It runs once per cell, and the accessors change when
the cell is reused for another item. Keep per-item state in your data, not in signals created inside the row. See
[Lists](../docs/lists.md) for details on recycling.

ListView fills its parent (`flex: 1`). Put it in a parent with a height, such as the root view or a
[Screen](./navigation.md).

## Rows

ListView wraps each row in a view with 16 pt horizontal and 12 pt vertical padding and a minimum height of 48 pt. The
content is centered vertically. With `dividers` (the default), every row except the first has a 1 pt divider at its
top.

The end of the list has extra bottom padding equal to the safe area's bottom inset. This lets the last row scroll
above the home indicator.

## Variants

| Variant | Look |
|---|---|
| `plain` (default) | Full-width rows on the `background` color. |
| `inset` | Rows in a rounded box with a 16 pt margin, on a `muted.background` page, like grouped settings. The first row has rounded top corners and the last row has rounded bottom corners. |

```tsx
<ListView variant="inset" each={settings} dividers={false}>
  {(item) => <Text>{item().label}</Text>}
</ListView>
```

## Headers, footers and list behavior

ListView accepts the data props of [List](../elements/list.md). ListView sets the layout itself (estimated row size,
padding), so `List`'s layout props such as `numColumns`, `horizontal`, `estimatedItemSize` and
`contentContainerStyle` are not available.

```tsx
<ListView
  each={messages()}
  keyExtractor={(m) => m.id}
  ListHeaderComponent={<Text style={{ padding: 16, fontSize: 22, fontWeight: "700" }}>Inbox</Text>}
  ListEmptyComponent={<Text style={{ padding: 16 }}>No messages</Text>}
  refreshing={loading()}
  onRefresh={reload}
  onEndReached={loadMore}
>
  {(m) => <Text numberOfLines={1}>{m().subject}</Text>}
</ListView>
```

## Props

| Prop | Type | Default | Description |
|---|---|---|---|
| `each` | `readonly T[]` | required | The items. |
| `children` | `(item: () => T, index: () => number) => JSX.Element` | required | Renders one row. Called once per recycled cell. |
| `variant` | `"plain" \| "inset"` | `"plain"` | Full-width rows, or rows in a rounded box with a margin. |
| `dividers` | `boolean` | `true` | Hairlines between rows. |
| `keyExtractor` | `(item: T, index: number) => string` | `item.key`, `item.id` or the index | The key of an item. |
| `ListHeaderComponent` | `ListSlot` | none | Shown before the first row (a component or an element). |
| `ListFooterComponent` | `ListSlot` | none | Shown after the last row. |
| `ListEmptyComponent` | `ListSlot` | none | Shown instead of the rows while `each` is empty. |
| `stickyHeaderIndices` | `number[]` | none | Items that stick to the top while scrolled past. |
| `initialScrollIndex` | `number` | none | Scrolls to this item before the first frame. |
| `refreshing` | `boolean` | `false` | The pull-to-refresh spinner shows. |
| `onRefresh` | `() => void` | none | The user pulled to refresh. Set `refreshing` while you load. |
| `onEndReached` | `() => void` | none | The end of the list came near (to load more). |
| `onEndReachedThreshold` | `number` | `1` | How close to the end, in viewports, `onEndReached` fires. |
| `onViewableItemsChanged` | `(info) => void` | none | Items became viewable or stopped being viewable. |
| `viewabilityConfig` | `ViewabilityConfig` | none | When items count as viewable. |
| `class`, `classList`, `style`, `testID`, `onLayout`, `ref` | | | Applied to the list's root view. |

`ListView` has no events of its own; the list props above are its callbacks.

## Theming

| Part | Theme key |
|---|---|
| Page background | `background` (plain), `muted.background` (inset) |
| Row background (inset) | `background` |
| Text color | `foreground` |
| Dividers | `border` |
| Inset box corners | `radius.lg` |

See [Theming](./index.md#theming).

## Accessibility

ListView adds no roles or labels to rows. If a row is pressable, give its view an `accessibilityRole` and, if needed,
an `accessibilityLabel`.

## Related

- [List](../elements/list.md): the virtualized list ListView is built on, with full layout control.
- [Lists guide](../docs/lists.md): recycling, keys, paging.
- [SelectList](./select-list.md): a list of options with a check mark.
- [Group](./group.md): a few rows in a rounded box, without virtualization.
