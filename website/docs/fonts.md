---
title: Fonts
description: The system font and generic families, importing .ttf and .otf files, families with several weights and italics, and using fonts from Tailwind classes.
order: 37
group: ui
---

# Fonts

Text uses the system font (San Francisco on iOS, Roboto on Android) unless a `fontFamily` says otherwise. A family can be a generic family, a font installed on the device, or a font file you ship with the app by importing it.

## System and generic families

| `fontFamily` | Font |
|---|---|
| (none), `"system-ui"`, `"sans-serif"`, `"ui-sans-serif"`, `"-apple-system"` | The system font |
| `"serif"`, `"ui-serif"` | The system serif (New York on iOS, Noto Serif on Android) |
| `"monospace"`, `"ui-monospace"` | The system monospace font (SF Mono on iOS) |
| `"ui-rounded"` | The rounded system font (SF Pro Rounded), iOS only |

The system font honors every `fontWeight` from 100 to 900. Tailwind's `font-sans`, `font-serif` and `font-mono` map to the first three rows.

A family installed on the device can be named directly (`"Georgia"`, `"Avenir Next"` on iOS). Installed fonts differ between platforms and versions, so ship the fonts your design depends on. On iOS, a family that does not exist falls back to the system font with a warning in the log (`font family 'X' is not available`).

## Shipping a font

Import a `.ttf` or `.otf` file. The import evaluates to the font's family name, and the font is registered before the first frame, so it is ready when text first renders:

```tsx
import { Text } from "nsh";
import Pacifico from "./fonts/Pacifico-Regular.ttf";

<Text style={{ fontFamily: Pacifico, fontSize: 30 }}>Custom fonts</Text>
```

`Pacifico` is the string `"Pacifico"`: the family name stored in the font file (its typographic family name, or else its family name), not the file name. Using the imported value instead of writing the name by hand keeps the two in sync and makes sure the file is bundled.

The bundler copies the file into the app like any other [asset](./images-and-assets.md). Fonts are registered for the app's process only; they are not installed on the device.

> [!WARNING]
> An import whose value is never used is removed by TypeScript's import elision, and with it the font. If you only refer to a family by its name (for example in the Tailwind theme), import the file for its side effect: `import "./fonts/Inter-Bold.ttf";`.

## Weights and italics

A family usually comes as several files, one per weight and style. Import each one; they register under the same family name, and `fontWeight` and `fontStyle` choose the face:

```tsx
import { Text } from "nsh";
import Inter from "./fonts/Inter-Regular.ttf";
import "./fonts/Inter-Medium.ttf";
import "./fonts/Inter-Bold.ttf";
import "./fonts/Inter-Italic.ttf";

const body = { fontFamily: Inter, fontSize: 16 } as const;

<Text style={body}>
  Regular, <Text style={{ fontWeight: "500" }}>medium</Text>, <Text style={{ fontWeight: "bold" }}>bold</Text> and{" "}
  <Text style={{ fontStyle: "italic" }}>italic</Text>.
</Text>
```

When no registered face has the exact weight asked for, the nearest one is used. The platforms differ when a face is missing:

| | iOS | Android |
|---|---|---|
| Weight without a face | The nearest registered face | The nearest face, with Android's synthetic bold where it applies |
| Italic without an italic face | The upright face (no slanting) | A slanted version of the upright face |

Ship the faces you use to get the same result on both.

## Fonts in Tailwind classes

Name your families in the Tailwind theme in `nsh.json` to use them as `font-*` classes:

```json
{
  "tailwind": {
    "fontFamily": { "display": "Pacifico", "body": "Inter" }
  }
}
```

```tsx
import { Text } from "nsh";
import "./fonts/Pacifico-Regular.ttf";
import "./fonts/Inter-Regular.ttf";
import "./fonts/Inter-Bold.ttf";

<Text class="font-display text-3xl text-violet-600">Custom fonts</Text>
<Text class="font-body font-bold">Bold Inter</Text>
```

An arbitrary family works without a theme entry: `font-[Pacifico]`. Weight classes (`font-light`, `font-semibold`, `font-bold`, ...) select faces as `fontWeight` does. Text classes on a view pass down to the texts inside it, so `<View class="font-body">` sets the family for a whole screen.

## A default font for the app

There is no global default font setting. Set the family on a container with a class, which the texts inside inherit, or keep a shared style:

```tsx
mount(() => (
  <View class="flex-1 font-body">
    <App />
  </View>
));
```

## Font features

`fontVariant` turns on OpenType features of the font: `"tabular-nums"`, `"proportional-nums"`, `"lining-nums"`, `"oldstyle-nums"` and `"small-caps"` ([Text](./text.md#text-styles)). The font has to contain them; system fonts do.

## Related

- [Text](./text.md): text styles, sizes and the system text size setting.
- [Tailwind](./tailwind.md#theme): the theme keys.
- [Images and assets](./images-and-assets.md): how imported files are shipped.
