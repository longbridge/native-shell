---
title: Card
description: 包裹你的内容的圆角卡片，可带页眉和页脚，有带阴影和仅描边两种样式。
order: 3
demo: ui-card
---

# Card

`Card` 是一个圆角表面，用来把相关内容归在一起。内容上方可以有页眉，下方可以有页脚。页眉、内容和页脚都是你的 JSX；卡片负责提供表面、内边距和分隔线。

```tsx
import { Card } from "nsh/ui/mobile";
```

## 用法

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

## 布局

- **页眉**：左右和顶部各有 16 pt 内边距。页眉中的文字为 semibold，字号比主题的 `font.size` 大 1 pt，所以一个普通的 `<Text>` 就能当标题用。页眉可以是任意 JSX，比如一行头像加姓名。
- **内容**：子元素纵向排列，间距 8 pt。有页眉时，内容顶部内边距为 8 pt，左右和底部为 16 pt；没有页眉时，四周都是 16 pt。
- **页脚**：靠末端对齐的一行，间距 8 pt，水平内边距 16 pt，垂直内边距 12 pt，上方有一条分隔线。操作按钮放在这里。

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

页眉和页脚只在你传入时存在：`header={cond() ? <Text>…</Text> : undefined}` 会添加或移除页眉，内容的内边距也会随之调整。

## 变体

| 变体 | 外观 |
|---|---|
| `elevated`（默认） | 1 pt 边框加一层淡阴影。 |
| `outlined` | 1 pt 边框，无阴影。 |

```tsx
<Card>…</Card>
<Card variant="outlined">…</Card>
```

## Props

| Prop | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `variant` | `"elevated" \| "outlined"` | `"elevated"` | 卡片是否带阴影。 |
| `header` | `JSX.Element` | 无 | 内容上方：标题或头像行。其中的文字为 semibold。 |
| `footer` | `JSX.Element` | 无 | 内容下方、分隔线之后：操作按钮，靠末端对齐。 |
| `children` | `JSX.Element` | 无 | 内容。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于卡片的根视图。`class` 和 `style` 会覆盖卡片自身的样式。 |

`Card` 没有事件。把处理函数放在你自己的内容上。

## 主题

| 部位 | 主题键 |
|---|---|
| 表面 | `group_box.background` |
| 文字颜色（内容会继承） | `group_box.foreground` |
| 边框和页脚分隔线 | `border` |
| 圆角半径 | `radius.lg`（默认 16 pt） |
| 页眉字号 | `font.size` + 1 |

参见[主题](./index.md#theming)。

## 无障碍

卡片不添加任何角色或标签。屏幕阅读器按顺序把页眉、内容和页脚作为各自独立的元素朗读。

## 相关内容

- [Group](./group.md)：圆角框内带分隔线的行。
- [Button](./button.md)：页脚中的操作。
