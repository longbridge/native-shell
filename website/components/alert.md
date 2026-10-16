---
title: Alert
description: Show the platform's alert dialog and await the index of the button the user pressed.
order: 13
---

# Alert

`Alert.alert` shows the platform's alert dialog: `UIAlertController` on iOS and `AlertDialog` on Android. It returns a
promise that resolves with the index of the button the user pressed. Unlike the other components in this set, an alert
has the platform's own look.

```tsx
import { Alert } from "nsh/ui/mobile";
```

## Usage

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

With no buttons, the alert has one "OK" button:

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

The promise resolves with the index of the pressed button in `buttons`, or `-1` when the alert was dismissed without
a button (Android, with `cancelable`). It does not reject.

### AlertButton

| Field | Type | Default | Description |
|---|---|---|---|
| `text` | `string \| null` | `"OK"` | The button's label. |
| `style` | `"default" \| "cancel" \| "destructive"` | `"default"` | `cancel` marks the cancel button; `destructive` marks a destructive action (red on iOS). |

### AlertOptions

| Field | Type | Default | Description |
|---|---|---|---|
| `cancelable` | `boolean` | `false` | Android: a tap outside the dialog or the back button dismisses it, and the promise resolves with `-1`. |

## Platform notes

- **iOS**: the buttons are laid out by `UIAlertController`. A button with `style: "cancel"` is placed where iOS puts
  cancel buttons, which may differ from its position in your array; the index you get back is always its index in
  `buttons`.
- **Android**: an `AlertDialog` shows at most three buttons. They are mapped by position: the last button is the
  positive button, the one before it the negative button, and the one before that the neutral button. Further buttons
  at the start of the array are not shown. Button styles do not change the look.

> [!TIP]
> Because the main thread never waits for JS, the dialog is shown and answered natively and your code continues when
> the promise resolves. You can `await` an alert inside any event handler, for example to confirm leaving a screen
> with [`preventBack`](./navigation.md#leaving-a-screen).

## Related

- [ActionSheet](./action-sheet.md): a list of actions.
- [Sheet](./sheet.md): a bottom sheet with your own content.
- [nsh/android](../modules/nsh-android.md): `Toast.show` for short messages on Android.
