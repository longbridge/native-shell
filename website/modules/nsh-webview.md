---
title: nsh/webview
description: WebView, a native web page view (WKWebView, android.webkit.WebView), with its props, events and ref methods.
order: 13
---

# nsh/webview

`nsh/webview` exports `WebView`: a web page shown by the platform's own browser engine (`WKWebView` on iOS and Mac Catalyst, `android.webkit.WebView` on Android). It is a native view laid out like any other, so it scrolls, clips and overlaps other views correctly.

```ts
import { WebView, type WebViewNavigation, type WebViewProps } from "nsh/webview";
```

See [Web view and video](../docs/web-view-and-video.md) for the guide and [`<web-view>`](../elements/web-view.md) for the element underneath.

## WebView

```ts
const WebView: (props: WebViewProps) => YNode;

interface WebViewProps extends Omit<ViewProps, "children" | "ios">, WebViewHostProps {
  source: string | { uri: string } | { html: string; baseUrl?: string };
  ios?: IOSWebViewHostProps;
}
```

```tsx
import { mount, signal, type YNode } from "nsh";
import { WebView, type WebViewNavigation } from "nsh/webview";

function Browser() {
  const nav = signal<WebViewNavigation | null>(null);
  let web!: YNode;
  return (
    <view style={{ flex: 1 }}>
      <view style={{ flexDirection: "row", gap: 16, padding: 12 }}>
        <text onPress={() => web.goBack()} style={{ opacity: nav()?.canGoBack ? 1 : 0.4 }}>Back</text>
        <text onPress={() => web.reload()}>Reload</text>
        <text numberOfLines={1} style={{ flex: 1 }}>{nav()?.title ?? ""}</text>
      </view>
      <WebView ref={web} source="https://example.com" style={{ flex: 1 }} onNavigationStateChange={(n) => nav.set(n)} />
    </view>
  );
}

mount(Browser);
```

## Props

`WebView` takes the [view props](../elements/view.md) (style, classes, layout, accessibility, `ref`; no children), plus:

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `string \| { uri: string } \| { html: string; baseUrl?: string }` | (required) | A URL, `{ uri }`, or inline HTML with an optional base URL for its relative links. |
| `injectedJavaScript` | `string` | | Runs after each page loads. |
| `injectedJavaScriptBeforeContentLoaded` | `string` | | Runs before each page's own scripts. |
| `userAgent` | `string` | platform's | The user agent string. |
| `scrollEnabled` | `boolean` | `true` | The user can scroll the page. |
| `javaScriptEnabled` | `boolean` | `true` | Pages run scripts. |
| `ios` | `{ allowsBackForwardNavigationGestures?: boolean }` | | iOS only: swipe from the edges to go back and forward. |

## Events

| Event | Payload | When |
|---|---|---|
| `onLoadStart` | `WebViewNavigation` | A page starts loading. |
| `onLoad` | `WebViewNavigation` | A page finished loading. |
| `onError` | `{ message: string }` | A page failed to load. |
| `onNavigationStateChange` | `WebViewNavigation` | The URL, title, loading state or history changed. |
| `onMessage` | `{ data: string }` | The page called `window.ReactNativeWebView.postMessage(data)`. |

```ts
interface WebViewNavigation {
  url: string;
  title: string;
  loading: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
}
```

## Ref methods

| Method | Description |
|---|---|
| `reload()` | Reloads the page. |
| `goBack()` | Goes back in its history. |
| `goForward()` | Goes forward in its history. |
| `stopLoading()` | Stops loading. |
| `injectJavaScript(script)` | Runs `script` in the page. |
| `postMessage(data)` | Sends `data` (a string) to the page: a `message` event on `window`, with `data` as `event.data`. |

## Talking to the page

The page sends strings to the app with `window.ReactNativeWebView.postMessage(…)`; the app sends strings to the page with `ref.postMessage(…)`. Send JSON for structured data.

```tsx
const page = `<!doctype html><html><head><meta name="viewport" content="width=device-width"></head>
<body>
  <button onclick="window.ReactNativeWebView.postMessage('hello from the page')">Post to the app</button>
  <p id="got"></p>
  <script>
    window.addEventListener('message', (e) => { document.getElementById('got').textContent = e.data; });
  </script>
</body></html>`;

let web!: YNode;
const message = signal("");

<WebView ref={web} source={{ html: page }} style={{ height: 200 }} onMessage={(e) => message.set(e.data)} />
<text onPress={() => web.postMessage("hi from the app")}>Post to the page</text>
```

## Capabilities

When your app declares `capabilities` in `nsh.json`, a web view may only show and navigate to the hosts listed under `webview`; other top-level navigations are refused natively. An app without a `capabilities` key may show anything. See [Sandbox and capabilities](../docs/sandbox-and-capabilities.md).

```json
{
  "capabilities": {
    "webview": { "hosts": ["example.com", "*.example.com"] }
  }
}
```
