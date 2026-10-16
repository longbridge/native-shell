import { cleanup, Scroll, signal, Text, View } from "nsh";
import { Toast } from "nsh/android";
import { colorScheme, fontScale, windowSize } from "nsh/device";
import { ActionSheet, Alert, keyboard, safeArea, StatusBar } from "nsh/ui/mobile";
import { Button, colors, Section } from "../ui";

function Row(props: { label: string; value: string }) {
  return (
    <View style={{ flexDirection: "row" }}>
      <Text style={{ flex: 1, color: colors.muted }}>{props.label}</Text>
      <Text>{props.value}</Text>
    </View>
  );
}

type Post = { id: number; title: string };

export function System() {
  const fps = signal(0);
  const running = signal(false);
  const ticks = signal(0);
  const posts = signal<Post[]>([]);
  const status = signal("");
  const answer = signal("");
  const barStyle = signal<"default" | "light-content" | "dark-content">("default");
  const barHidden = signal(false);

  // Frames per second, counted with requestAnimationFrame.
  let frame = 0;
  let frames = 0;
  let since = 0;
  const loop = (time: number) => {
    frames++;
    if (time - since >= 1000) {
      fps.set(Math.round((frames * 1000) / (time - since)));
      frames = 0;
      since = time;
    }
    frame = requestAnimationFrame(loop);
  };
  const toggle = () => {
    if (running()) cancelAnimationFrame(frame);
    else {
      frames = 0;
      since = performance.now();
      frame = requestAnimationFrame(loop);
    }
    running.set(!running());
  };
  const timer = setInterval(() => ticks.set(ticks() + 1), 1000);
  cleanup(() => {
    clearInterval(timer);
    cancelAnimationFrame(frame);
  });

  const load = async () => {
    status.set("loading…");
    const started = performance.now();
    try {
      const response = await fetch("https://jsonplaceholder.typicode.com/posts?_limit=3");
      posts.set(await response.json());
      status.set(`HTTP ${response.status} in ${Math.round(performance.now() - started)} ms`);
    } catch (e) {
      status.set(String(e));
    }
  };

  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Environment (reactive)">
          <Row label="Window" value={`${Math.round(windowSize().width)} × ${Math.round(windowSize().height)}`} />
          <Row label="Safe area" value={`${Math.round(safeArea().top)} / ${Math.round(safeArea().right)} / ${Math.round(safeArea().bottom)} / ${Math.round(safeArea().left)}`} />
          <Row label="Keyboard" value={`${Math.round(keyboard().height)} pt`} />
          <Row label="Color scheme" value={colorScheme()} />
          <Row label="Font scale" value={fontScale().toFixed(2)} />
        </Section>
        <Section title="Status bar">
          <StatusBar barStyle={barStyle()} hidden={barHidden()} animated />
          <Row label="Style" value={barStyle()} />
          <View style={{ flexDirection: "row", gap: 8 }}>
            <View style={{ flex: 1 }}>
              <Button label="Next style" onPress={() => barStyle.set(barStyle() === "default" ? "light-content" : barStyle() === "light-content" ? "dark-content" : "default")} />
            </View>
            <View style={{ flex: 1 }}>
              <Button label={barHidden() ? "Show" : "Hide"} color="#57606a" onPress={() => barHidden.set(!barHidden())} />
            </View>
          </View>
        </Section>
        <Section title="Alerts">
          <Button
            label="Alert with one button"
            onPress={async () => {
              await Alert.alert("Saved", "Your changes were saved.");
              answer.set("OK");
            }}
          />
          <Button
            label="Confirm (cancel / destructive)"
            color="#cf222e"
            onPress={async () => {
              const i = await Alert.alert(
                "Delete the photo?",
                "This cannot be undone.",
                [
                  { text: "Cancel", style: "cancel" },
                  { text: "Delete", style: "destructive" },
                ],
                { cancelable: true },
              );
              answer.set(["Cancel", "Delete"][i] ?? "dismissed");
            }}
          />
          <Button
            label="Three buttons"
            color="#8250df"
            onPress={async () => {
              const buttons = ["Later", "Never", "Install"];
              const i = await Alert.alert("Update available", "Install it now?", [
                { text: "Later" },
                { text: "Never", style: "cancel" },
                { text: "Install" },
              ]);
              answer.set(buttons[i] ?? "dismissed");
            }}
          />
          <Button
            label="Action sheet"
            color="#1a7f37"
            onPress={async () => {
              const options = ["Cancel", "Copy link", "Save", "Delete"];
              const i = await ActionSheet.show({ title: "Share the photo", options, cancelButtonIndex: 0, destructiveButtonIndexes: [3] });
              answer.set(options[i] ?? "dismissed");
            }}
          />
          <Button label="Toast (Android)" color="#57606a" onPress={() => Toast.show("Hello from a toast", "long")} />
          <Row label="Answer" value={answer() || "–"} />
        </Section>
        <Section title="Frames and timers">
          <Row label="requestAnimationFrame" value={running() ? `${fps()} fps` : "stopped"} />
          <Row label="setInterval (1 s)" value={String(ticks())} />
          <Button label={running() ? "Stop counting frames" : "Count frames"} onPress={toggle} />
        </Section>
        <Section title="Network and logs">
          <Button label="fetch() three posts" onPress={load} />
          <Text style={{ color: colors.muted }}>{status()}</Text>
          <Each of={posts()}>
            {(post) => (
              <Text numberOfLines={1}>
                <Text style={{ fontWeight: "bold" }}>{`#${post().id} `}</Text>
                {post().title}
              </Text>
            )}
          </Each>
          <Button label="console.log (shown by nsh run)" color="#57606a" onPress={() => console.log("hello from the gallery", { at: new Date().toISOString() })} />
        </Section>
      </View>
    </Scroll>
  );
}
