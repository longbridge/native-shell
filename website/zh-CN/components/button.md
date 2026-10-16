---
title: Button
description: 带文字标签的按压目标，有四种变体，提供原生按压反馈和禁用状态。
order: 2
demo: ui-button
---

# Button

`Button` 是带文字标签的按压目标。它以主题色填充，按下时颜色加深，并通过 `onPress` 报告按压。在 iOS 和 Android 上外观一致。

```tsx
import { Button } from "nsh/ui/mobile";
```

## 用法

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

按钮把标签居中排成一行，水平内边距 16 pt，垂直内边距 12 pt。和其他视图一样，在纵向排列的容器中它会拉伸到容器宽度。传入 `class` 或 `style` 可以改变尺寸或圆角：

```tsx
<View style={{ flexDirection: "row", gap: 8 }}>
  <Button class="flex-1" title="Cancel" variant="secondary" onPress={cancel} />
  <Button class="flex-1" title="Save" onPress={save} />
</View>
```

<a id="variants"></a>

## 变体

```tsx
<Button title="Save" />                         {/* primary (default) */}
<Button title="Not now" variant="secondary" />
<Button title="Delete" variant="danger" />
<Button title="Learn more" variant="plain" />
```

| 变体 | 背景 | 按下时 | 标签 |
|---|---|---|---|
| `primary`（默认） | `primary.background` | `primary.active.background` | `primary.foreground` |
| `secondary` | `secondary.background` | `secondary.active.background` | `secondary.foreground` |
| `danger` | `danger.background` | `danger.active.background` | `danger.foreground` |
| `plain` | 透明 | `accent.background` | `foreground` |

`danger` 用于破坏性操作；`plain` 用于不需要强调的操作，比如卡片页脚里的次要操作。

## 禁用

```tsx
<Button title="Submit" disabled={!valid()} onPress={submit} />
```

禁用的按钮以 50% 不透明度显示，不响应按压，也没有按压反馈。

## Props

| Prop | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `title` | `string` | 必填 | 标签文字。 |
| `variant` | `"primary" \| "secondary" \| "danger" \| "plain"` | `"primary"` | 外观（见[变体](#variants)）。 |
| `disabled` | `boolean` | `false` | 不响应按压、无反馈、50% 不透明度。 |
| `accessibilityLabel` | `string \| null` | `title` | 当 `title` 不足以说明时，屏幕阅读器朗读的内容。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于按钮的根视图。`class` 和 `style` 会覆盖按钮自身的样式。 |

## 事件

| 事件 | 参数 | 触发时机 |
|---|---|---|
| `onPress` | 无 | 按钮被按下（并在按钮内松开）。`disabled` 时不触发。 |

## 主题

标签使用主题的 `font.size`（默认 16 pt），字重为 semibold。圆角使用 `radius`（默认 12 pt）。颜色取自[变体表](#variants)中的主题键。参见[主题](./index.md#theming)。

## 无障碍

按钮的角色为 `button`。设置了 `accessibilityLabel` 时屏幕阅读器朗读它，否则朗读 `title`。`disabled` 时按钮会被报告为已禁用。

## 相关内容

- [View](../elements/view.md)：用 `onPress`、`activeOpacity` 和 `underlayColor` 制作自定义按压目标。
- [触摸与手势](../docs/touch-and-gestures.md)。
