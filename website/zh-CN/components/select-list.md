---
title: SelectList
description: 带勾选标记的可滚动字符串选项列表，支持受控与非受控两种用法。
order: 7
demo: ui-select-list
---

# SelectList

`SelectList` 显示一个可滚动的文本选项列表，并用勾选标记标出当前选中的一项。行由核心根据你传入的字符串数组渲染，因此即使有上千个选项，开销也与可见的几行相当，滚动时不会运行任何 JS。

```tsx
import { SelectList } from "nsh/ui/mobile";
```

## 用法

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

SelectList 会填满父视图（`flex: 1`）。如果要把它放在滚动视图里或与其他内容并排，请给它一个固定高度的父视图：

```tsx
<View style={{ height: 240, borderRadius: 12, overflow: "hidden" }}>
  <SelectList options={cities} onSelect={(i) => console.log(cities[i])} />
</View>
```

## 受控与非受控

- **受控**：传入 `value`（选中项的下标，`null` 表示未选中），并在 `onSelect` 中更新它。勾选标记跟随 `value`；如果你不更新 `value`，勾选标记就不会移动。
- **非受控**：不传 `value`。列表自己记录选择，每次按下都会移动勾选标记，`onSelect` 照样会报告每次选择。

每次按下都会调用 `onSelect`，包括用户按下已经选中的那一项。

## 行

每行至少 48 pt 高，左右内边距 16 pt，显示选项文本，选中行末尾有一个勾选标记（✓）。行与行之间有 1 pt 分隔线，按下时会高亮。

> [!NOTE]
> 数组变化时，选项按文本匹配，因此插入或删除选项时其余行会保持原位。请确保选项字符串互不重复。

## 属性

| 属性 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `options` | `string[]` | 必填 | 选项，按顺序排列。 |
| `value` | `number \| null` | 无（非受控） | 由应用控制时，选中项的下标。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于列表的根视图。 |

## 事件

| 事件 | 参数 | 触发时机 |
|---|---|---|
| `onSelect` | `number`：选项的下标 | 用户按下了某个选项。 |

## 主题

| 部位 | 主题键 |
|---|---|
| 行背景 | `background` |
| 按下时的行 | `list.active.background` |
| 文本 | `foreground` |
| 勾选标记 | `primary.background` |
| 分隔线 | `border` |

参见[主题](./index.md#theming)。

## 无障碍

每一行的角色都是 `button`，选中的行会报告为 **selected**（已选中）。屏幕阅读器朗读选项的文本。

## 相关

- [ListView](./list-view.md)：由你自己渲染行。
- [Sheet](./sheet.md)：把 SelectList 放进底部面板，当作选择器使用。
- [ActionSheet](./action-sheet.md)：在系统对话框中显示少量操作。
