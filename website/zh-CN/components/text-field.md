---
title: TextField
description: 带标签的单行文本输入框，支持清除按钮、聚焦边框和密码输入。
order: 11
demo: ui-text-field
---

# TextField

`TextField` 是一个单行文本输入框：上方可以有标签，聚焦时边框变色，并带有清除按钮。它基于 [`<Input>`](../elements/input.md) 元素构建。

```tsx
import { TextField } from "nsh/ui/mobile";
```

## 用法

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

## 受控与非受控

- **受控**：传入 `value` 并在 `onChange` 中更新它，输入框显示 `value`。
- **非受控**：不传 `value`。输入框自己保存文本，从 `defaultValue` 开始，并通过 `onChange` 报告变化。

```tsx
<TextField label="Notes" defaultValue="Draft" onChange={(text) => save(text)} />
```

## 变体

```tsx
<TextField label="Password" placeholder="At least 8 characters" secure />
<TextField label="Code" clearable={false} />
<TextField label="Account" value="ada@example.com" disabled />
```

- `secure` 隐藏输入的内容，用于密码。
- `clearable`（默认开启）在有文本时于末尾显示一个 × 按钮。按下后清空文本，通过 `onChange` 报告 `""`，并保持输入框聚焦，方便用户重新输入。
- `disabled` 让输入框只读，以 50% 不透明度显示，并隐藏清除按钮。

## 提交

用户按下键盘的回车键时，会以当前文本调用 `onSubmit`：

```tsx
<TextField
  placeholder="Search"
  value={query()}
  onChange={(v) => query.set(v)}
  onSubmit={(text) => search(text)}
/>
```

> [!NOTE]
> TextField 覆盖的是常见场景。如果需要键盘类型、回车键文字、自动填充、多行文本，或带 `focus()` 的 ref，请直接使用 [`<Input>`](../elements/input.md) 元素。另请参见[表单与键盘](../docs/forms-and-keyboard.md)。

## 属性

| 属性 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `label` | `string \| null` | 无 | 显示在输入框上方。 |
| `placeholder` | `string \| null` | 无 | 输入框为空时显示。 |
| `value` | `string \| null` | 无（非受控） | 由应用控制时的文本。 |
| `defaultValue` | `string` | `""` | 不由应用控制时的初始文本。 |
| `secure` | `boolean` | `false` | 隐藏输入的内容（密码）。 |
| `clearable` | `boolean` | `true` | 有文本时显示清除按钮。 |
| `disabled` | `boolean` | `false` | 不可编辑，50% 不透明度。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于输入框的根视图（包含标签和输入框的那一列）。 |

## 事件

| 事件 | 参数 | 触发时机 |
|---|---|---|
| `onChange` | `string`：新文本 | 用户输入、删除或按下清除按钮。 |
| `onSubmit` | `string`：文本 | 用户按下回车键。 |

## 主题

| 部位 | 主题键 |
|---|---|
| 标签 | `muted.foreground`，小号文字 |
| 输入框 | `background`，圆角为 `radius`，至少 44 pt 高 |
| 边框 | `input.border`；聚焦时为 `ring` |
| 文本 | `foreground` |
| 占位文字 | `muted.foreground` |
| 选中文本 | `primary.background` |
| 清除按钮 | `muted.background`；按下时为 `secondary.active.background` |

参见[主题](./index.md#theming)。

## 无障碍

输入框是原生文本框，屏幕阅读器会把它播报为可编辑文本。清除按钮的角色是 `button`，标签为 "Clear"，其触摸区域在 24 pt 的圆形之外再向外扩展 8 pt。`label` 是输入框上方的一段独立文本，不会作为输入框的无障碍标签。

## 相关

- [Input](../elements/input.md)：TextField 所基于的元素。
- [表单与键盘](../docs/forms-and-keyboard.md)、[Keyboard](./keyboard.md)、[KeyboardAvoidingView](./keyboard-avoiding-view.md)。
