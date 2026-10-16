import { Scroll, signal, Text, View, type YNode } from "nsh";
import { Video } from "nsh/video";
import { WebView, type WebViewNavigation } from "nsh/webview";
import { Button, colors, Section } from "../ui";

const page = `<!doctype html><html><head><meta name="viewport" content="width=device-width"></head>
<body style="font-family: -apple-system, sans-serif; margin: 16px">
<h3>An inline page</h3>
<button onclick="window.ReactNativeWebView.postMessage('hello from the page')">Post to the app</button>
<p id="got">Nothing from the app yet.</p>
<script>window.addEventListener('message', (e) => { document.getElementById('got').textContent = 'The app says: ' + e.data; });</script>
</body></html>`;

export function Media() {
  const state = signal<WebViewNavigation | null>(null);
  const controls = signal(false);
  const message = signal("");
  const paused = signal(true);
  const progress = signal("");
  let web!: YNode;
  let site!: YNode;
  let video!: YNode;
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Web view">
          <WebView ref={web} source={{ html: page }} style={{ height: 200, borderRadius: 8, overflow: "hidden" }} onMessage={(e) => message.set(e.data)} />
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
            <Button label="Post to the page" onPress={() => web.postMessage(`hi #${Math.round(performance.now()) % 1000}`)} />
            <Text style={{ flex: 1, color: colors.muted }}>{message()}</Text>
          </View>
        </Section>
        <Section title="A web site">
          <WebView
            ref={site}
            source={{ uri: "https://en.m.wikipedia.org/wiki/React_Native" }}
            style={{ height: 320, borderRadius: 8, overflow: "hidden" }}
            onNavigationStateChange={(v) => state.set(v)}
          />
          <Text numberOfLines={1} style={{ color: colors.muted, fontSize: 13 }}>
            {state() ? `${state()!.loading ? "Loading " : ""}${state()!.title || state()!.url}` : "…"}
          </Text>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <Button label="Back" color={state()?.canGoBack ? colors.accent : "#8c959f"} onPress={() => site.goBack()} />
            <Button label="Forward" color={state()?.canGoForward ? colors.accent : "#8c959f"} onPress={() => site.goForward()} />
            <Button label="Reload" color="#57606a" onPress={() => site.reload()} />
          </View>
        </Section>
        <Section title="Video">
          <Video
            ref={video}
            source="https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4"
            paused={paused()}
            muted
            repeat
            controls={controls()}
            style={{ height: 200, backgroundColor: "#000000", borderRadius: 8, overflow: "hidden" }}
            onProgress={(e) => progress.set(`${e.currentTime.toFixed(1)} / ${e.duration.toFixed(1)} s`)}
          />
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
            <Button label={paused() ? "Play" : "Pause"} onPress={() => paused.set(!paused())} />
            <Button label="Restart" color="#57606a" onPress={() => video.seek(0)} />
            <Button label={controls() ? "Own controls" : "System controls"} color="#8250df" onPress={() => controls.set(!controls())} />
            <Text style={{ color: colors.muted }}>{progress()}</Text>
          </View>
        </Section>
      </View>
    </Scroll>
  );
}
