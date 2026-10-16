---
title: ListView
description: 渲染你的行的虚拟化、可回收列表，自带内边距和分隔线，有 plain 和 inset 两种样式。
order: 5
demo: ui-list-view
---

# ListView

`ListView` 是一个由你渲染各行的滚动列表。和 [List](../elements/list.md) 一样，它只创建屏幕附近的行，并在滚动时复用单元格，因此一个上千项的列表，开销与可见行数大致相当。ListView 在此之上提供外观：每行的内边距、行间分隔线，以及可选的 inset 样式。

```tsx
import { ListView } from "nsh/ui/mobile";
```

## 用法

和 `List` 一样，用 `each` 传入数据，用子函数渲染每一行：

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

行函数接收的是 accessor：`person()` 和 `index()`。它对每个单元格只运行一次；单元格被复用来显示另一项时，accessor 的值随之变化。每项的状态请保存在数据里，不要放在行内创建的 signal 中。回收机制的细节见[列表](../docs/lists.md)。

ListView 会填满父视图（`flex: 1`），请把它放在有高度的父视图中，比如根视图或 [Screen](./navigation.md)。

## 行

ListView 把每一行包进一个视图：水平内边距 16 pt，垂直内边距 12 pt，最小高度 48 pt，内容垂直居中。开启 `dividers`（默认开启）时，除第一行外每一行顶部都有一条 1 pt 分隔线。

列表末尾会额外加上与安全区域底部 inset 等高的内边距，这样最后一行可以滚动到 Home 指示条上方。

## 变体

| 变体 | 外观 |
|---|---|
| `plain`（默认） | 通栏行，铺在 `background` 颜色上。 |
| `inset` | 行放在带 16 pt 外边距的圆角框里，页面背景为 `muted.background`，类似分组的设置页。第一行顶部圆角，最后一行底部圆角。 |

```tsx
<ListView variant="inset" each={settings} dividers={false}>
  {(item) => <Text>{item().label}</Text>}
</ListView>
```

## 页眉、页脚与列表行为

ListView 接受 [List](../elements/list.md) 的数据类 props。布局（预估行高、内边距）由 ListView 自己决定，所以 `List` 的布局类 props，如 `numColumns`、`horizontal`、`estimatedItemSize` 和 `contentContainerStyle`，在这里不可用。

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

| Prop | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `each` | `readonly T[]` | 必填 | 列表项。 |
| `children` | `(item: () => T, index: () => number) => JSX.Element` | 必填 | 渲染一行。每个回收单元格调用一次。 |
| `variant` | `"plain" \| "inset"` | `"plain"` | 通栏行，或带外边距的圆角框内的行。 |
| `dividers` | `boolean` | `true` | 行间细分隔线。 |
| `keyExtractor` | `(item: T, index: number) => string` | `item.key`、`item.id` 或索引 | 列表项的 key。 |
| `ListHeaderComponent` | `ListSlot` | 无 | 显示在第一行之前（组件或元素）。 |
| `ListFooterComponent` | `ListSlot` | 无 | 显示在最后一行之后。 |
| `ListEmptyComponent` | `ListSlot` | 无 | `each` 为空时代替各行显示。 |
| `stickyHeaderIndices` | `number[]` | 无 | 滚动经过时吸附在顶部的项。 |
| `initialScrollIndex` | `number` | 无 | 在第一帧之前滚动到该项。 |
| `refreshing` | `boolean` | `false` | 显示下拉刷新指示器。 |
| `onRefresh` | `() => void` | 无 | 用户下拉刷新。加载期间请设置 `refreshing`。 |
| `onEndReached` | `() => void` | 无 | 接近列表末尾（用于加载更多）。 |
| `onEndReachedThreshold` | `number` | `1` | 距末尾多近（以视口高度计）时触发 `onEndReached`。 |
| `onViewableItemsChanged` | `(info) => void` | 无 | 有项变为可见或不再可见。 |
| `viewabilityConfig` | `ViewabilityConfig` | 无 | 何时算作可见。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于列表的根视图。 |

`ListView` 没有自己的事件，上面的列表 props 就是它的回调。

## 主题

| 部位 | 主题键 |
|---|---|
| 页面背景 | `background`（plain）、`muted.background`（inset） |
| 行背景（inset） | `background` |
| 文字颜色 | `foreground` |
| 分隔线 | `border` |
| inset 圆角框的圆角 | `radius.lg` |

参见[主题](./index.md#theming)。

## 无障碍

ListView 不给行添加任何角色或标签。如果某一行可按，请给它的视图设置 `accessibilityRole`，必要时再加上 `accessibilityLabel`。

## 相关内容

- [List](../elements/list.md)：ListView 所基于的虚拟化列表，可完全控制布局。
- [列表指南](../docs/lists.md)：回收、key、分页。
- [SelectList](./select-list.md)：带勾号的选项列表。
- [Group](./group.md)：圆角框内的少量行，不做虚拟化。
