# Elements and interaction

| JSX | Native (Apple) | Notes |
|---|---|---|
| `<view>` | `NshView` (UIView) | `onPress` / `onLongPress` / `onPressIn` / `onPressOut`, `activeOpacity`, `pan={{x, y}}` + `onPanStart` / `onPanEnd`, `pinch` / `rotation` |
| `<text>` | `TextView` (CTFrame) | nested `<text>` is a span, other elements are inline attachments ([text.md](text.md)); `numberOfLines`, `ellipsizeMode`, `allowFontScaling`, `maxFontSizeMultiplier`, `adjustsFontSizeToFit` + `minimumFontScale`, `selectable`, `onTextLayout` |
| `<image src>` | UIView + image CALayer | `src` is a URL or `import img from "./a.png"` (an asset descriptor with its size); `resizeMode`; `defaultSource` (placeholder), `blurRadius`, `fadeDuration`; `onLoadStart` / `onProgress` (`loaded` / `total` bytes) / `onLoad` (the source's pixel size) / `onError` / `onLoadEnd`; `tintColor`; animated GIF / WebP / APNG; `Image.getSize(uri)` / `Image.prefetch(uri)` |
| `<scroll>` | UIScrollView | `horizontal`, `refreshing` + `onRefresh`, `scrollValue`, `onScrollEnd`, `keyboardDismissMode`, zooming, see [scrolling-and-lists.md](scrolling-and-lists.md) |
| `Canvas` (runtime) | `<canvas>`: `CanvasView` (drawRect replay) | see [canvas.md](canvas.md) |
| `List` (runtime) | UIScrollView + recycled cells | see [scrolling-and-lists.md](scrolling-and-lists.md) |
| `<input>` | UITextField / UITextView in a container | `value` / `defaultValue` / `onChangeText` / `onSubmitEditing` / `onFocus` / `onBlur`, `placeholder`, `secureTextEntry`, `keyboardType`, `returnKeyType`, `autoCapitalize`, `autoCorrect`, `multiline`, `maxLength`, `editable`, `autoFocus`, `selectionColor`, `blurOnSubmit`, `selection` + `onSelectionChange` (UTF-16 `start` / `end`), `onKeyPress` (`key`: the text, `Backspace`, `Enter`), `autoComplete` (autofill; `ios={{ textContentType }}` for iOS's names), `caretHidden`, `onContentSizeChange` (the text's size, for growing inputs); ref `setSelection(start, end)` |
| `Stack` / `Screen` (runtime) | UINavigationController | `createNavigator(route)`: `push` / `pop` / `popToTop` / `replace` / `reset`; screens keep their state while stacked; native header (`title`, `headerLeft` / `headerRight` / `headerTitle` elements, `headerTransparent`, colors; `ios={{ headerLargeTitle, headerBackTitle }}`), swipe back (`gestureEnabled`: the interactive gesture only); `preventBack` + `onBackRequest` keep the user on a screen (no swipe; the header / Android back buttons send the event instead of popping) |
| `Modal` (runtime) | presented UIViewController | `visible`, `animationType`, `transparent`, `onRequestClose` (controlled), `onShow`, `onDismiss`; `dismissible` (iOS sheets swipe away, Android back closes natively, then `onRequestClose` + `onDismiss`); `ios={{ presentationStyle, supportedOrientations }}` (`pageSheet` / `formSheet`), `android={{ statusBarTranslucent }}` (otherwise the content starts below the status bar) |
| `InputAccessoryView` (`nsh/ios`) | `<inputaccessory>`: a NshView set as `inputAccessoryView` | `nativeID`; inputs name it with `ios={{ inputAccessoryViewID }}`. A layout root as wide as the window and as tall as its content; the core resolves the id (`InputConfig::accessory`). Renders nothing on Android |
| `WebView` | WKWebView (WebKit loaded on first use) | `source` (`uri` / `html` + `baseUrl`), `injectedJavaScript(BeforeContentLoaded)`, `onMessage` (pages call `window.ReactNativeWebView.postMessage`), `onLoad*`, `onNavigationStateChange`; ref: `reload()`, `goBack()`, `goForward()`, `postMessage(data)`, `injectJavaScript(js)` |
| `nativeComponent` (`nsh/native`) | `<native component>`: a `NshView` box filled by the registered component's view | see [embedding.md](embedding.md) |
| `Video` | AVPlayerLayer / AVPlayerViewController (AVFoundation loaded on first use) | `source`, `paused`, `muted`, `volume`, `rate`, `repeat`, `resizeMode`, `controls`, `onLoad` / `onProgress` / `onEnd` / `onBuffer` / `onError`; ref: `seek(seconds)` |

On Android the same elements map to `Box` (ViewGroup), `TextBox`, `ImageBox`, `ScrollBox` (ScrollView), `InputBox` (EditText),
`StackBox` (screens with a Toolbar, predictive back), `ModalHost` (a Dialog), `WebBox` (android.webkit.WebView) and `VideoBox`
(MediaPlayer on a TextureView, MediaController); both are `WidgetBox`es. Every host element and runtime component is typed in
`.nsh/types/nsh/types.d.ts` and the declarations of the runtime's modules (made from the Rust tables: `tooling.md`).

**Host props are one definition.** A host prop is a variant of `nsh_core::Prop` (or of the enum it nests: `ScrollProp`,
`NativeProp`, `AccessibilityProp`, ...) with `#[prop(js = "name", ...)]`: `#[derive(HostProps)]` (`nsh_core::js::host`)
derives from it how `nsh-js` decodes the JS value (straight from QuickJS values, no JSON), the default clearing the
prop sets (`HostProps::reset`, also used when a Rust component's render stops setting it) and its declaration: one
interface per element group and platform (`InputHostProps`, `IOSInputHostProps`, ...) that the element types of
`types.d.ts` and the runtime's components extend. The payload's type decodes itself (`HostValue`;
`#[derive(PropEnum)]` for string unions); `default`, `min` / `max`, `on` (the element groups), `ios` / `android`
(names inside `ios={{…}}` / `android={{…}}`), `with` (a decoder of the whole variant, `js/host_values.rs`), `once`
(not reset) and `internal` (set by the runtime, untyped) cover the rest. **Events** are a table next to it
(`host::EVENTS`): the event, the element groups whose props declare it and its handler's type, one row per payload
(`onLoad` gives an image its size and a video its duration); the handlers join the same interfaces. An event no
element declares (the root's, the runtime's own) is listed in `INTERNAL_EVENTS`; a test checks every `EventKind` is
in one of them.

## Primitives only

nsh's runtime provides the basic capabilities. Components are the UI sets' (Rust, `crates/nsh-ui-mobile`,
`docs/dev/components.md`), drawn with views, classes, canvases and native animations so they look the same on iOS and Android. Do not
add platform-styled widgets (UISwitch / SeekBar wrappers and the like), and do not add compositions of existing
primitives to the runtime (section lists, touchables, drawers, ...): they belong to a UI set. React Native is a
reference for which capabilities exist, not an API to be compatible with. Components that need core or native support
are primitives of their own (`List`, `Canvas`, `WebView`, `Video`; in the mobile set `Stack`, `Modal`, `SafeAreaView`,
`KeyboardAvoidingView`; `InputAccessoryView` in `nsh/ios`). Lowercase JSX names are built-ins (host elements, `Tag::from_name`;
`<list>`, `<if>`, `<each>`), capitalized ones are components: a component is imported (`import { Button } from
"nsh/ui/mobile"`) and may be written in kebab case (`<button>`); an imported or local binding of a built-in's name
replaces it (`Modal` from `nsh/ui/mobile` makes `<modal>` that component), except objects of functions of nsh's modules
(`Image` from `nsh/image` leaves `<image>` the element; `native-pipeline.md`).

## The main thread never waits for JS

Some callbacks are questions the system wants answered on the spot (may this
text change, may the back button pop, may the sheet close, may this gesture begin, may the web view navigate): they are
answered natively from props mounted beforehand, never by asking JS (blocking the main thread drops frames, can deadlock
and reorders fast input). Events tell JS afterwards. Existing answers: input `maxLength` (`shouldChangeCharactersIn`,
`InputFilter`), `blurOnSubmit` (`textFieldShouldReturn`), `gestureEnabled` / `preventBack` (`gestureRecognizerShouldBegin`,
`navigationBar:shouldPopItem:`, Android `wantsBack()` / `OnBackInvokedCallback` registration), `dismissible`
(`presentationControllerShouldDismiss`, the dialog's back), `pan` / `pinch` / `hitSlop` / `pointerEvents` /
`keyboardShouldPersistTaps` (recognizers), capabilities (`decidePolicyForNavigationAction`, `shouldOverrideUrlLoading`),
`cancelable`. A new capability with such a callback gets a prop (a menu's items, a rule, a flag); what a prop cannot
express is let through and corrected in the event afterwards (a controlled `value` in `onChangeText`). `BackHandler` is
of that kind: with a listener the press is taken, and JS exits the app when no listener handles it.

## Interaction and the environment

- **Press feedback primitives** (native, no JS round trip): `activeOpacity`; `underlayColor` + `underlayOpacity` (default 0.85)
  on any view — the core shows the underlay as the background and fades the children while the platform reports the view
  pressed; `active:` classes. `KeyboardAvoidingView` (Rust; `behavior`: `padding` → inset padding, `height` → inset margin,
  `position` → an inner view moved up; `keyboardVerticalOffset`).
- **Platform modules**: `Platform` (`OS`: `ios` / `android` / `macos` / `windows` / `linux`, from the platform's
  `Environment::os` (`nsh_engine::Os`; the default is the build's mobile OS, so `nsh trace` is iOS); `isDesktop`,
  `Version`, `isPad`, `isTV`, `isMacCatalyst`, `select` (the OS, then `mobile` / `desktop`, `native`, `default`)), `PixelRatio`, `AccessibilityInfo` (`isScreenReaderEnabled`,
  `isReduceMotionEnabled`, `addEventListener("screenReaderChanged" | "reduceMotionChanged")`, `announceForAccessibility`,
  `setAccessibilityFocus(ref)`: Rust, the node passed as `{ $node: id }`), reactive `screenReaderEnabled()` / `reduceMotion()`. The environment carries `pixelRatio`,
  `screenReader`, `osVersion` and `idiom` (VoiceOver / TalkBack changes are observed natively).
- **`disabled`** on any element applies `disabled:` classes and removes its press listeners from what is mounted (no press,
  no feedback).
- **`onPanEnd` / `onPanStart`** carry `valueX` / `valueY`: the bound pan values at that moment (a drag may grab a value mid
  animation), for snapping.
- **Imperative calls**: nodes have `focus()`, `blur()` and `scrollTo({ x, y, animated })` (use `ref`); lists also
  `scrollToIndex({ index, viewPosition, viewOffset, animated })`, `scrollToEnd()`, `scrollToOffset()`; `Keyboard.dismiss()`.
  Commands run after the commit's other mount items; list scrolls are resolved by the core with the current item sizes.
- **Touch** (React Native semantics): `pointerEvents`, `hitSlop`, `activeOpacity`, `delayLongPress`, `android={{ ripple }}`
  (`{ color, borderless, radius, foreground }`, Android only); press-in waits ~130ms inside scroll views; only the innermost
  pressable view responds. Nested `<text>` spans with press handlers
  are links: the core lists them in `AttributedText::links` (UTF-16 ranges) and the block's press recognizer routes a press to
  the span under the finger (CoreText `CTLineGetStringIndexForPosition` / `Layout.getOffsetForHorizontal`).
- **Raw touches and the responder system**: `onTouchStart/Move/End/Cancel` (+ `Capture`), `onStartShouldSetResponder`,
  `onMoveShouldSetResponder` (+ `Capture`), `onResponderGrant/Reject/Start/Move/End/Release/Terminate/TerminationRequest`, and
  `PanResponder.create` on top. A node with any of them listens to `onTouch` natively; one tracker per root view (surface,
  modals: `touch.rs`, `TouchTracker.java`) observes without recognizing and reports touches whose target or an ancestor
  listens, with the path of listening nodes (`EventPayload::Touch`, sent to the root; moves are merged per engine turn). The
  negotiation runs as in React Native, natively (`nsh-js` `touch.rs`), asking the app's handlers. A grant returning true
  (PanResponder's default) sends `SetJsResponder`: scroll
  views around stop taking the touches (UIKit: their pan recognizer is disabled; Android: `ScrollBox.jsBlocked`); a native
  scroll view that starts dragging first cancels the JS touches. Rust components' nodes on the path hear the same
  event first and negotiate their own responder in the core (`claim_touches`, `docs/dev/components.md`); the core
  sends both sides' blocks.
- **Accessibility**: `accessible`, `accessibilityLabel` / `Hint` / `Role` (`role`) / `State` / `Value` / `Actions` +
  `onAccessibilityAction`, `accessibilityElementsHidden`, `importantForAccessibility`, `accessibilityLiveRegion`, `testID`
  (Android: resource id). Views without a label read their texts.
- **Frames**: `requestAnimationFrame` / `cancelAnimationFrame` follow the display (CADisplayLink / Choreographer).
- **Back button** (`nsh/android`): `BackHandler.addEventListener("hardwareBackPress", ...)`, after modals and stacks; the app exits
  when nothing handles it. Rust components listen with `cx.on_back_press(..)`, before JS's listeners; the root wants
  the button while either side listens (`docs/dev/components.md`).
- **System UI**: `StatusBar` (component and static functions; the last mounted wins, prop by prop, over what the
  functions set: the core layers them, `docs/dev/components.md`), `Alert.alert` and `ActionSheet.show` (promises of
  the index chosen, -1 when dismissed; a list dialog on Android), `Toast` (`nsh/android`). Answers come back as the
  root's `Dialog` event, which the core resolves. Embedding apps forward their controller's status bar to `nsh_status_bar_style()` / `_hidden()` / `_animation()`.
- **Right to left**: `I18nManager` (`isRTL`, reactive; `allowRTL`, `forceRTL`, applied at once) sets the root's direction;
  `direction` (`inherit` / `ltr` / `rtl`) overrides it per view. The core resolves each node's direction (inherited, also for
  subtrees built detached), sets taffy's `direction` (rows run from the right) and, right to left, swaps the left and right
  properties (margins, paddings, insets, borders, corners — React Native's default `doLeftAndRightSwapInRTL`) and places the
  start / end ones (`marginStart`, `start`, `borderTopEndRadius`, ...). A direction change swaps the current taffy style in place
  (an involution), so authored styles need not be kept. Text: `auto` alignment is the right side right to left, left / right
  swap, `writingDirection` sets the base direction (else the content's). Native widgets mirror (`MountItem::LayoutDirection`:
  `semanticContentAttribute`, `View.setLayoutDirection`; Android apps need `android:supportsRtl`). Horizontal scroll views
  are laid out left to right and their children mirrored in the content by the core (a right-to-left row would overflow to
  negative offsets), starting at the right end and keeping their distance from it; horizontal lists are flipped natively
  (`ScrollConfig::mirrored`: the scroll view and each cell get scaleX −1, composed with their transforms), so the list's
  offset logic stays left to right.
- **Environment** (reactive accessors; `nsh/device`, and `nsh/ui/mobile` for the mobile ones): `windowSize()`, `safeArea()`, `keyboard()` (`{ height, duration }`: how
  far the keyboard reaches into the surface, the bottom safe area under it included, on every platform),
  `colorScheme()`, `fontScale()`. `KeyboardAvoidingView` pads its bottom by `keyboard().height`, animated with the keyboard,
  for content outside scroll views (message bars). Inputs inside a scroll view are kept clear of the keyboard natively (bottom inset + scroll to the
  focused input), with no JS involved.
- **Controlled inputs**: each native edit carries an edit count; a `value` from JS is applied only if it is based on the latest
  edit, so a slow JS round trip can never overwrite newer typing (React Native's eventCount).

Text, styles, scrolling and lists, animation, canvases and images have their own pages.
