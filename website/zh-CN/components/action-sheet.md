---
title: ActionSheet
description: 显示平台的操作表，列出一组操作，并通过 await 得到所选操作的序号。
order: 14
---

# ActionSheet

`ActionSheet.show` 展示一组操作：iOS 上是操作表（`UIAlertController`），Android 上是列表对话框。它返回一个 Promise，解析为所选操作的序号。

```tsx
import { ActionSheet } from "nsh/ui/mobile";
```

## 用法

```tsx
import { mount, signal, Text, View } from "nsh";
import { ActionSheet, Button } from "nsh/ui/mobile";

function App() {
  const chosen = signal("–");
  const share = async () => {
    const options = ["Cancel", "Copy link", "Save", "Delete"];
    const i = await ActionSheet.show({
      title: "Share the photo",
      options,
      cancelButtonIndex: 0,
      destructiveButtonIndexes: [3],
    });
    chosen.set(options[i] ?? "dismissed");
  };
  return (
    <View style={{ flex: 1, padding: 16, paddingTop: 48, gap: 12 }}>
      <Button title="Share…" onPress={share} />
      <Text>{chosen()}</Text>
    </View>
  );
}

mount(App);
```

## API

```ts
ActionSheet.show(options: ActionSheetOptions): Promise<number>
```

Promise 解析为所选操作在 `options.options` 中的序号。如果用户没有选择就关闭了操作表（点击外部、按返回键），则解析为 `cancelButtonIndex`；没有取消操作时解析为 `-1`。它不会被 reject。

### ActionSheetOptions

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `options` | `string[]` | 必填 | 各操作的文字，按顺序排列。 |
| `cancelButtonIndex` | `number \| null` | 无 | 取消操作的序号。 |
| `destructiveButtonIndexes` | `number[] \| null` | 无 | 破坏性操作的序号（iOS 上显示为红色）。 |
| `title` | `string \| null` | 无 | 显示在操作上方。 |
| `message` | `string \| null` | 无 | 显示在标题下方。 |

## 平台说明

- **iOS**：取消操作单独显示在操作表底部。在 iPad 和 Mac Catalyst 上操作表以 popover 形式出现，取消操作会被隐藏：点击 popover 外部即可关闭，Promise 解析为 `cancelButtonIndex`（或 `-1`）。
- **Android**：操作以列表形式显示在对话框中。取消操作不出现在列表里，点击外部或按返回键即为取消。列表对话框无法显示 message：没有 `title` 时，`message` 会作为标题显示；有 `title` 时 message 不显示。破坏性操作与其他操作外观相同。

## 相关内容

- [Alert](./alert.md)：最多三个按钮的对话框。
- [Sheet](./sheet.md)：放置自定义内容的底部面板，两个平台外观一致。
- [SelectList](./select-list.md)：从众多选项中选择一个。
