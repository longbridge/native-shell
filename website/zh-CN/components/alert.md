---
title: Alert
description: 显示平台的警告对话框，并通过 await 得到用户所按按钮的序号。
order: 13
---

# Alert

`Alert.alert` 显示平台自带的警告对话框：iOS 上是 `UIAlertController`，Android 上是 `AlertDialog`。它返回一个 Promise，解析为用户所按按钮的序号。与这个 UI 集中的其他组件不同，警告框使用的是平台自身的外观。

```tsx
import { Alert } from "nsh/ui/mobile";
```

## 用法

```tsx
import { mount, signal, Text, View } from "nsh";
import { Alert, Button } from "nsh/ui/mobile";

function App() {
  const answer = signal("–");
  const confirm = async () => {
    const i = await Alert.alert("Delete the photo?", "This cannot be undone.", [
      { text: "Cancel", style: "cancel" },
      { text: "Delete", style: "destructive" },
    ]);
    answer.set(i === 1 ? "deleted" : "kept");
  };
  return (
    <View style={{ flex: 1, padding: 16, paddingTop: 48, gap: 12 }}>
      <Button title="Delete…" variant="danger" onPress={confirm} />
      <Text>{answer()}</Text>
    </View>
  );
}

mount(App);
```

不传按钮时，警告框只有一个“OK”按钮：

```tsx
await Alert.alert("Saved", "Your changes were saved.");
```

## API

```ts
Alert.alert(
  title: string,
  message?: string | null,
  buttons?: AlertButton[] | null,
  options?: AlertOptions | null,
): Promise<number>
```

Promise 解析为被按按钮在 `buttons` 中的序号；如果对话框未通过按钮而被关闭（Android，且设置了 `cancelable`），则解析为 `-1`。它不会被 reject。

### AlertButton

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `text` | `string \| null` | `"OK"` | 按钮文字。 |
| `style` | `"default" \| "cancel" \| "destructive"` | `"default"` | `cancel` 表示取消按钮；`destructive` 表示破坏性操作（iOS 上显示为红色）。 |

### AlertOptions

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `cancelable` | `boolean` | `false` | Android：点击对话框外部或按返回键会关闭对话框，Promise 解析为 `-1`。 |

## 平台说明

- **iOS**：按钮由 `UIAlertController` 布局。`style: "cancel"` 的按钮会放在 iOS 放置取消按钮的位置，可能与它在数组中的位置不同；但返回的序号始终是它在 `buttons` 中的序号。
- **Android**：`AlertDialog` 最多显示三个按钮，按位置映射：最后一个按钮是 positive 按钮，倒数第二个是 negative 按钮，倒数第三个是 neutral 按钮。数组开头多出的按钮不会显示。按钮样式不影响外观。

> [!TIP]
> 主线程从不等待 JS，因此对话框以原生方式显示和应答，Promise 解析后你的代码才继续执行。你可以在任何事件处理函数中 `await` 一个警告框，例如配合 [`preventBack`](./navigation.md#leaving-a-screen) 在离开页面前请用户确认。

## 相关内容

- [ActionSheet](./action-sheet.md)：一组操作。
- [Sheet](./sheet.md)：放置自定义内容的底部面板。
- [nsh/android](../modules/nsh-android.md)：在 Android 上用 `Toast.show` 显示简短消息。
