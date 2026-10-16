---
title: nsh/desktop
description: "Desktop-only APIs: the window (title, title bar, maximize, full screen), a title bar of the app's own with draggable regions, the application menu, context menus and keyboard shortcuts."
order: 12.5
---

# nsh/desktop

`nsh/desktop` holds what only desktops have: the window and its title bar, the application menu and context menus. It works on macOS, Windows and Linux. The module can be imported on every platform, so one bundle runs everywhere: on iOS and Android its functions do nothing, and `isAvailable` tells you whether you are on a desktop.

```ts
import { isAvailable, Menu, titleBar, Window } from "nsh/desktop";
```

## isAvailable

```ts
const isAvailable: boolean;
```

`true` on macOS, Windows and Linux; `false` elsewhere.

## Window

```ts
const Window: {
  setTitle(title: string): void;
  setTitleBar(style: "default" | "hidden"): void;
  minimize(): void;
  toggleMaximize(): void;
  toggleFullScreen(): void;
  close(): void;
};
```

| Member | Description |
|---|---|
| `setTitle(title)` | The window's title: the title bar, the taskbar or Dock, the window switcher. |
| `setTitleBar(style)` | `"hidden"` removes the system's title bar: the app fills the window up to its top edge and draws a title bar of its own. The system's window controls stay (macOS's traffic lights, Windows' minimize / maximize / close buttons, GTK's window buttons). `"default"` brings the system's title bar back. |
| `minimize()` | Minimizes the window. |
| `toggleMaximize()` | Maximizes the window, or restores it when it is maximized. |
| `toggleFullScreen()` | Enters or leaves full screen. |
| `close()` | Closes the window; the app ends with it. |

## A title bar of your own

With the title bar hidden, the app draws its own bar and says which parts of it move the window with `desktop={{ windowDrag: true }}`:

```tsx
import { Input, Text, View } from "nsh";
import { titleBar, Window } from "nsh/desktop";

Window.setTitleBar("hidden");

function TitleBar() {
  return (
    <View
      desktop={{ windowDrag: true }}
      style={{
        height: Math.max(titleBar().height, 40),
        paddingLeft: titleBar().left + 8, // macOS's traffic lights
        paddingRight: titleBar().right + 8, // the caption buttons on Windows, GTK's buttons
        flexDirection: "row",
        alignItems: "center",
      }}
    >
      <Text>My App</Text>
      <View style={{ flex: 1 }} />
      <Input placeholder="Search" style={{ width: 180 }} />
    </View>
  );
}
```

Pressing a view with `windowDrag` drags the window as the system's title bar would: a double click maximizes it (or does what the user set: on macOS, "Double-click a window's title bar to"), and on Windows and Linux a right click opens the window menu.

- The nearest view that sets `windowDrag` decides: `desktop={{ windowDrag: false }}` keeps a part of a draggable view clickable.
- Text fields and views with press or touch handlers (`onPress`, `onTouch*`, responders) inside a draggable view stay clickable without it: the search field and any buttons in the example above work as usual.
- `windowDrag` also works with the system's title bar: a sidebar or toolbar can move the window too.

### titleBar

```ts
const titleBar: Accessor<{ height: number; left: number; right: number }>;
```

The hidden title bar's area, in points (reactive): its `height`, and the widths the system's window controls take at its `left` and `right` ends. Lay your bar out with it so that nothing goes under the controls. All three are 0 while the system shows its title bar (and in full screen, and off desktops).

| | `height` | `left` | `right` |
|---|---|---|---|
| macOS | The title bar's (28) | The traffic lights' width | 0 |
| Windows | 32 | 0 | The three caption buttons (138) |
| Linux | The header bar's | The window buttons at the start (the desktop's button layout) | The window buttons at the end |

On Windows the caption buttons are drawn by nsh in Windows 11's style and follow the app's appearance; hovering the maximize button shows Windows 11's snap layouts. The window keeps its resize borders, shadow and rounded corners on every platform.

## Menu

```ts
const Menu: {
  setApplicationMenu(items: MenuItem[] | null): void;
  popup(items: MenuItem[], at?: { x: number; y: number }): void;
};
```

| Member | Description |
|---|---|
| `setApplicationMenu(items)` | Sets the application menu: one submenu per top-level item (`File`, `Edit`, ...). On macOS it is the menu bar, whose first menu is the app menu (AppKit titles it with the app's name). On Windows and Linux it is the window's menu bar, which is not shown while the title bar is hidden; its shortcuts still work, and a title bar of the app's own can open its menus with `Menu.popup`. `null` restores the platform's default menu. |
| `popup(items, at?)` | Shows a context menu with its top left corner at `at` (the window's coordinates, in points), or at the pointer. |

Call `setApplicationMenu` again to change the menu. Inside an `effect` it follows the signals it reads: a check mark or an enabled state that depends on app state stays current.

### Menu items

```ts
type MenuItem =
  | { type: "separator" }
  | {
      label?: string;
      role?: MenuRole;
      shortcut?: string;
      enabled?: boolean;
      checked?: boolean;
      onSelect?: () => void;
      submenu?: MenuItem[];
    };
```

| Field | Description |
|---|---|
| `label` | What the item shows. A role's item has a label of its own without it. |
| `onSelect` | Called when the item is chosen, by a click or its shortcut. |
| `shortcut` | Its keyboard shortcut: modifiers and a key joined by `+`, such as `"Mod+N"`, `"Mod+Shift+Z"`, `"Alt+F4"` or `"F11"`. `Mod` is Command on macOS and Control elsewhere; the other modifiers are `Cmd`, `Ctrl`, `Alt` (Option) and `Shift`. Keys are letters, digits, punctuation, `Enter`, `Escape`, `Tab`, `Backspace`, `Delete`, `Space`, the arrows (`Up`, `Down`, `Left`, `Right`), `Home`, `End`, `PageUp`, `PageDown`, `F1`–`F24` and `Plus`. |
| `enabled` | `false` grays the item out (default `true`). |
| `checked` | Set (to `true` or `false`) for an item with a check mark. |
| `submenu` | Makes the item a submenu (it needs a `label`). |
| `role` | A standard item the platform carries out (below). |

`null`, `undefined` and `false` items are left out, so `cond && { … }` works in a list.

Roles:

| Role | Does | Platforms |
|---|---|---|
| `about`, `hide`, `hideOthers`, `showAll` | The app menu's standard items | macOS (left out elsewhere) |
| `quit` | Quits the app | All |
| `undo`, `redo`, `cut`, `copy`, `paste`, `selectAll` | Edit the focused text field (its own shortcuts stay its own) | All |
| `minimize`, `zoom`, `toggleFullScreen`, `close` | The window | All |

A role's item has the platform's label and shortcut unless it sets its own (macOS adds the app's name to `about`, `hide` and `quit`).

```tsx
import { effect, signal } from "nsh";
import { Platform } from "nsh/device";
import { Menu } from "nsh/desktop";

const mac = Platform.OS === "macos";
const wrap = signal(true);

effect(() => {
  Menu.setApplicationMenu([
    mac && {
      label: "My App",
      submenu: [{ role: "about" }, { type: "separator" }, { role: "hide" }, { role: "hideOthers" }, { type: "separator" }, { role: "quit" }],
    },
    {
      label: "File",
      submenu: [
        { label: "New", shortcut: "Mod+N", onSelect: () => createDocument() },
        { label: "Open…", shortcut: "Mod+O", onSelect: () => openDocument() },
        !mac && { type: "separator" },
        !mac && { role: "quit", label: "Exit" },
      ],
    },
    {
      label: "Edit",
      submenu: [{ role: "undo" }, { role: "redo" }, { type: "separator" }, { role: "cut" }, { role: "copy" }, { role: "paste" }],
    },
    {
      label: "View",
      submenu: [
        { label: "Word Wrap", checked: wrap(), onSelect: () => wrap.update((w) => !w) },
        { role: "toggleFullScreen" },
      ],
    },
  ]);
});
```

A context menu, from a button's press:

```tsx
<View
  onPress={() =>
    Menu.popup([
      { label: "Rename", onSelect: rename },
      { label: "Duplicate", onSelect: duplicate },
      { type: "separator" },
      { label: "Delete", onSelect: remove },
    ])
  }
>
  <Text>More</Text>
</View>
```

> [!NOTE]
> Menus are the system's (NSMenu, Win32 menus, GTK popover menus): they look and behave like every other app's, follow the system's (or the app's) appearance and are keyboard accessible. `examples/desktop` is an app with a title bar of its own, an application menu and context menus.
