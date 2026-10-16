---
title: Group
description: 设置页风格的行，放在圆角框内并以细线分隔，可带标题说明和脚注。
order: 4
demo: ui-group
---

# Group

`Group` 把若干行放进一个圆角框里，行与行之间以细线分隔，就像设置页中的一个分区。每个 JSX 子元素成为一行。可以在框上方加一句简短的标题说明，在下方加一段解释文字。

```tsx
import { Group } from "nsh/ui/mobile";
```

## 用法

```tsx
import { mount, signal, Text, View } from "nsh";
import { Group } from "nsh/ui/mobile";

function App() {
  const sounds = signal(true);
  return (
    <View style={{ flex: 1, backgroundColor: "#f5f5f5", padding: 16, paddingTop: 48 }}>
      <Group
        header={<Text>NOTIFICATIONS</Text>}
        footer={<Text>Sounds play for new messages only.</Text>}
      >
        <Text>Show previews</Text>
        <View
          onPress={() => sounds.set(!sounds())}
          style={{ flexDirection: "row", justifyContent: "space-between" }}
        >
          <Text>Sounds</Text>
          <Text>{sounds() ? "On" : "Off"}</Text>
        </View>
      </Group>
    </View>
  );
}

mount(App);
```

## 行

Group 把每个子元素包进一行：水平内边距 16 pt，垂直内边距 12 pt，最小高度 44 pt（舒适触控目标的最小尺寸）。内容垂直居中。除第一行外，每一行顶部都有一条 1 pt 的分隔线。

行的内容就是你传入的内容：要在一行里放标签和值，就放一个 `flexDirection: "row"` 的 `<View>`；要让这一行可按，就在该视图上加 `onPress`。子元素增加或删除时（比如通过 [`<If>` 或 `<Each>`](../elements/control-flow.md)），分组会相应更新行和分隔线。

```tsx
<Group header={<Text>DEVICES</Text>}>
  <Each of={devices()}>
    {(device) => (
      <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
        <Text>{device().name}</Text>
        <Text style={{ color: "#737373" }}>{device().status}</Text>
      </View>
    )}
  </Each>
</Group>
```

> [!TIP]
> Group 会渲染所有行。对于很长或长度不定的列表，请使用 `variant="inset"` 的 [ListView](./list-view.md)：外观同样是圆角框，但只创建屏幕附近的行。

## 标题说明与脚注

标题说明距左侧 16 pt，使用小号（`text-sm`）的弱化文字；脚注使用更小号（`text-xs`）的弱化文字。标题说明、圆角框和脚注之间间距 8 pt。两者都是可选的。

## Props

| Prop | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `header` | `JSX.Element` | 无 | 框的上方：一句简短的标题说明。 |
| `footer` | `JSX.Element` | 无 | 框的下方：解释文字。 |
| `children` | `JSX.Element` | 无 | 各行，每个子元素一行。 |
| `class`、`classList`、`style`、`testID`、`onLayout`、`ref` | | | 作用于分组的根视图（容纳标题说明、圆角框和脚注的那一列）。 |

`Group` 没有事件。

## 主题

| 部位 | 主题键 |
|---|---|
| 圆角框 | `group_box.background`，圆角半径 `radius.lg` |
| 行文字颜色 | `group_box.foreground` |
| 分隔线 | `border` |
| 标题说明与脚注文字 | `muted.foreground` |

参见[主题](./index.md#theming)。

## 无障碍

Group 不添加任何角色或标签，每一行的内容按你写的样子被朗读。如果某一行可按，请给它的视图设置 `accessibilityRole`（例如 `"button"`），必要时再加上 `accessibilityLabel`。参见[无障碍](../docs/accessibility.md)。

## 相关内容

- [Card](./card.md)：带页眉和页脚的单个表面。
- [ListView](./list-view.md)：同样 inset 外观的虚拟化行。
- [Stepper](./stepper.md)：行内常用的控件。
