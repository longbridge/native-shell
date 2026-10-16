---
title: ActionSheet
description: Show the platform's action sheet with a list of actions and await the index of the action chosen.
order: 14
---

# ActionSheet

`ActionSheet.show` presents a list of actions: an action sheet (`UIAlertController`) on iOS and a list dialog on
Android. It returns a promise that resolves with the index of the chosen action.

```tsx
import { ActionSheet } from "nsh/ui/mobile";
```

## Usage

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

The promise resolves with the index of the chosen action in `options.options`. When the user dismisses the sheet
without choosing (a tap outside, the back button), it resolves with `cancelButtonIndex`, or with `-1` if there is no
cancel action. It does not reject.

### ActionSheetOptions

| Field | Type | Default | Description |
|---|---|---|---|
| `options` | `string[]` | required | The actions' labels, in order. |
| `cancelButtonIndex` | `number \| null` | none | The index of the cancel action. |
| `destructiveButtonIndexes` | `number[] \| null` | none | The indexes of destructive actions (red on iOS). |
| `title` | `string \| null` | none | Shown above the actions. |
| `message` | `string \| null` | none | Shown below the title. |

## Platform notes

- **iOS**: the cancel action is shown separately at the bottom of the sheet. On iPad and Mac Catalyst the sheet is a
  popover, which hides the cancel action: a tap outside the popover dismisses it, and the promise resolves with
  `cancelButtonIndex` (or `-1`).
- **Android**: the actions are a list in a dialog. The cancel action is not listed; a tap outside or the back button
  cancels. A list dialog cannot show a message: without a `title`, the `message` is used as the title, and with a
  `title` the message is not shown. Destructive actions look like the others.

## Related

- [Alert](./alert.md): a dialog with up to three buttons.
- [Sheet](./sheet.md): a bottom sheet with your own content, the same on both platforms.
- [SelectList](./select-list.md): choosing one of many options.
