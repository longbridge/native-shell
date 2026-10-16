# Text

- **Text**: `<text numberOfLines ellipsizeMode="tail|head|middle|clip">`; styles `fontFamily` (system or imported fonts:
  `import Inter from "./Inter.ttf"` evaluates to the family name and registers the font before the first frame), `fontWeight`,
  `fontStyle`, `letterSpacing`, `textDecorationLine` + `textDecorationColor`, `textTransform`, `lineHeight`, `textAlign`,
  `textAlignVertical` (the core moves the content box down). `adjustsFontSizeToFit` shrinks the fonts (down to
  `minimumFontScale`, at least 0.1) until the text fits its box and lines (`Paragraph::fit`, a binary search in the core).
  `onTextLayout` reports the final lines (`TextEngine::lines`: range, box, ascender, descender, cap / x height).
  `selectable` (or `select-text`): a long press highlights the block and offers Copy (iOS `UIMenuController`, Android a
  floating action mode); whole blocks only, no range selection.
- **Inline attachments**: any other element inside a `<text>` (also inside its spans) is an attachment: laid out as its own
  layout root at its content size (`update_attachments`), it becomes an object replacement character (U+FFFC,
  `AttributedText::attachments`) that the engine sizes to it on the baseline (CTRunDelegate; Android a `ReplacementSpan`), so
  it wraps with the text. After the final layout the core places its native view, a child of the text view (Android's
  `TextBox` is a ViewGroup for this), on the placeholder (`TextEngine::attachments`); attachments cut off by the line limit or
  an ellipsis move out of sight. Their own children are normal layout children; a text inside them built after the block
  resizes them and builds the block again (`check_attachments`). They take presses of their own; copying leaves the
  placeholders out.

- **Rust components** nest texts the same way: `text("a ").child(text("b").underline().on_press(..)).child(view())` is
  one block; a plain string child is a raw text node (as JS's strings), a text's own content its first piece
  (`core/components.rs`, `Part::Raw`).

See also the text pitfalls in [pitfalls.md](pitfalls.md) and the engines in `crates/nsh-platform-ios/src/text.rs` /
`crates/nsh-platform-android/src/text.rs`.
