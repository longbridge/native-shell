import { Input, Scroll, type ScrollEvent, signal, Text, View, type YNode } from "nsh";
import { windowSize } from "nsh/device";
import { List } from "nsh/list";
import { Button, colors, Section, Toggle as Switch } from "../ui";

const palette = ["#0969da", "#8250df", "#bf3989", "#cf222e", "#bc4c00", "#1a7f37", "#0a3069"];

function Toggle(props: { label: string; value: boolean; onChange: (v: boolean) => void }) {
  return (
    <View style={{ flexDirection: "row", alignItems: "center" }}>
      <Text style={{ flex: 1 }}>{props.label}</Text>
      <Switch value={props.value} onChange={props.onChange} />
    </View>
  );
}

function Choice<T extends string>(props: { options: readonly T[]; value: T; onChange: (v: T) => void }) {
  return (
    <View style={{ flexDirection: "row", gap: 6, flexWrap: "wrap" }}>
      <Each of={props.options}>
        {(option) => {
          const o = option();
          return (
          <Text
            onPress={() => props.onChange(o)}
            style={{
              fontSize: 13,
              paddingHorizontal: 10,
              paddingVertical: 5,
              borderRadius: 6,
              overflow: "hidden",
              color: props.value === o ? "#ffffff" : colors.text,
              backgroundColor: props.value === o ? colors.accent : "#e1e4e8",
            }}
          >
            {o}
          </Text>
          );
        }}
      </Each>
    </View>
  );
}

/** Full-width pages: one per fling, with a page indicator. */
function Pager() {
  // Pages as wide as the scroll view.
  const width = signal(windowSize().width - 60);
  const page = signal(0);
  return (
    <View style={{ gap: 8 }}>
      <Scroll
        horizontal
        pagingEnabled
        showsIndicator={false}
        style={{ height: 140, borderRadius: 12 }}
        onLayout={(e) => width.set(e.width)}
        onMomentumScrollEnd={(e) => page.set(Math.round(e.x / width()))}
        onScrollEnd={(e) => page.set(Math.round(e.x / width()))}
      >
        <View style={{ flexDirection: "row" }}>
          <Each of={palette.slice(0, 5)}>
            {(color, i) => (
              <View style={{ width: width(), height: 140, backgroundColor: color(), alignItems: "center", justifyContent: "center" }}>
                <Text style={{ color: "#ffffff", fontSize: 28, fontWeight: "700" }}>Page {i() + 1}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
      <View style={{ flexDirection: "row", justifyContent: "center", gap: 6 }}>
        <Each of={palette.slice(0, 5)}>
          {(_, i) => (
            <View style={{ width: 8, height: 8, borderRadius: 4, backgroundColor: page() === i() ? colors.accent : colors.border }} />
          )}
        </Each>
      </View>
    </View>
  );
}

/** Cards that snap to the center; a fling can pass several unless limited to one. */
function Cards() {
  const card = 220;
  const gap = 12;
  const oneStep = signal(false);
  const align = signal<"start" | "center" | "end">("center");
  return (
    <View style={{ gap: 10 }}>
      <List
        each={Array.from({ length: 20 }, (_, i) => i)}
        horizontal
        estimatedItemSize={card + gap}
        snapToInterval={card + gap}
        snapToAlignment={align()}
        decelerationRate="fast"
        disableIntervalMomentum={oneStep()}
        showsIndicator={false}
        style={{ height: 120 }}
        contentContainerStyle={{ gap }}
      >
        {(i) => (
          <View
            style={{
              width: card,
              height: 120,
              borderRadius: 14,
              backgroundColor: palette[i() % palette.length],
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <Text style={{ color: "#ffffff", fontSize: 22, fontWeight: "700" }}>Card {i() + 1}</Text>
          </View>
        )}
      </List>
      <Choice options={["start", "center", "end"] as const} value={align()} onChange={(v) => align.set(v)} />
      <Toggle label="One card per fling" value={oneStep()} onChange={(v) => oneStep.set(v)} />
    </View>
  );
}

/** onScroll (throttled), drag and momentum events, scrollEnabled, bounces, contentInset. */
function Events() {
  const offset = signal<ScrollEvent | null>(null);
  const log = signal<string[]>([]);
  const enabled = signal(true);
  const bounces = signal(true);
  const inset = signal(false);
  const throttle = signal<"0" | "100" | "500">("100");
  const note = (name: string) => (e: { y: number }) => log.update((l) => [`${name} @ ${Math.round(e.y)}`, ...l].slice(0, 5));
  return (
    <View style={{ gap: 10 }}>
      <Scroll
        style={{ height: 180, borderRadius: 10, backgroundColor: "#f6f8fa" }}
        scrollEnabled={enabled()}
        bounces={bounces()}
        contentInset={inset() ? { top: 40, bottom: 40 } : undefined}
        scrollEventThrottle={Number(throttle())}
        onScroll={(v) => offset.set(v)}
        onScrollBeginDrag={note("begin drag")}
        onScrollEndDrag={note("end drag")}
        onMomentumScrollBegin={note("momentum begin")}
        onMomentumScrollEnd={note("momentum end")}
      >
        <View style={{ padding: 12, gap: 6 }}>
          <Each of={Array.from({ length: 30 }, (_, i) => i + 1)}>
            {(i) => <Text style={{ color: colors.muted }}>Line {i()}</Text>}
          </Each>
        </View>
      </Scroll>
      <Text style={{ fontVariant: ["tabular-nums"] }}>
        {offset()
          ? `y ${Math.round(offset()!.y)} of ${Math.round(offset()!.contentHeight)} (viewport ${Math.round(offset()!.height)})`
          : "Scroll the box"}
      </Text>
      <Text style={{ color: colors.muted, fontSize: 13 }}>{log().join("\n") || "No drags yet"}</Text>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
        <Text style={{ flex: 1 }}>onScroll every</Text>
        <Choice options={["0", "100", "500"] as const} value={throttle()} onChange={(v) => throttle.set(v)} />
        <Text>ms</Text>
      </View>
      <Toggle label="Scroll enabled" value={enabled()} onChange={(v) => enabled.set(v)} />
      <Toggle label="Bounces" value={bounces()} onChange={(v) => bounces.set(v)} />
      <Toggle label="Content inset 40 (top and bottom)" value={inset()} onChange={(v) => inset.set(v)} />
    </View>
  );
}

/** Pinch to zoom (natively); tap a tile to zoom in on it. */
function Zoom() {
  let scroll: YNode | undefined;
  const scale = signal(1);
  const bounces = signal(true);
  const tiles = Array.from({ length: 24 }, (_, i) => i);
  return (
    <View style={{ gap: 10 }}>
      <Scroll
        ref={scroll}
        style={{ height: 240, borderRadius: 10, backgroundColor: "#0d1117" }}
        minimumZoomScale={0.5}
        maximumZoomScale={5}
        bouncesZoom={bounces()}
        centerContent
        scrollEventThrottle={50}
        onScroll={(e) => scale.set(e.zoomScale)}
      >
        <View style={{ flexDirection: "row", flexWrap: "wrap", padding: 6, gap: 6 }}>
          <Each of={tiles}>
            {(tile) => {
              const i = tile();
              return (
              <View
                style={{
                  width: 74,
                  height: 50,
                  borderRadius: 8,
                  borderCurve: "continuous",
                  backgroundColor: palette[i % palette.length],
                  alignItems: "center",
                  justifyContent: "center",
                }}
                onPress={() => {
                  // The tile's middle, in the content's (unzoomed) points.
                  const x = 6 + (i % 4) * 80 + 37;
                  const y = 6 + Math.floor(i / 4) * 56 + 25;
                  scroll?.zoomTo({ scale: scale() < 2.5 ? 3 : 1, x, y });
                }}
              >
                <Text style={{ color: "#ffffff", fontSize: 11 }}>Tile {i + 1}</Text>
              </View>
              );
            }}
          </Each>
        </View>
      </Scroll>
      <Text style={{ fontVariant: ["tabular-nums"] }}>Zoom {scale().toFixed(2)}×</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Button label="1×" onPress={() => scroll?.zoomTo({ scale: 1 })} />
        <Button label="2×" onPress={() => scroll?.zoomTo({ scale: 2 })} />
        <Button label="0.5×" onPress={() => scroll?.zoomTo({ scale: 0.5 })} />
      </View>
      <Toggle label="Bounces past the limits" value={bounces()} onChange={(v) => bounces.set(v)} />
    </View>
  );
}

type Post = { id: number; label: string };

/** New items arrive at the top; the rows on screen stay where they are. */
function Feed() {
  let next = 0;
  const post = (label: string): Post => ({ id: next++, label });
  const posts = signal(Array.from({ length: 30 }, (_, i) => post(`Post ${i + 1}`)));
  let fresh = 0;
  const addOnTop = () => posts.update((p) => [...Array.from({ length: 3 }, () => post(`New ${++fresh}`)), ...p]);
  return (
    <View style={{ gap: 10 }}>
      <List each={posts()} estimatedItemSize={44} style={{ height: 240, borderRadius: 10, backgroundColor: "#f6f8fa" }}>
        {(p) => (
          <View style={{ paddingHorizontal: 12, paddingVertical: 12, borderBottomWidth: 1, borderBottomColor: "#eaeef2" }}>
            <Text style={{ color: p().label.startsWith("New") ? colors.accent : colors.text }}>{p().label}</Text>
          </View>
        )}
      </List>
      <Button label="Add 3 posts on top" onPress={addOnTop} />
    </View>
  );
}

type Taps = "never" | "handled" | "always";

export function Scrolling() {
  const taps = signal<Taps>("never");
  const presses = signal(0);
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }} keyboardShouldPersistTaps={taps()}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Paging">
          <Pager />
        </Section>
        <Section title="Snapping (snapToInterval)">
          <Cards />
        </Section>
        <Section title="Zoom (pinch, tap a tile)">
          <Zoom />
        </Section>
        <Section title="Events and behavior">
          <Events />
        </Section>
        <Section title="Prepending keeps the position">
          <Feed />
        </Section>
        <Section title="Taps with the keyboard up (keyboardShouldPersistTaps)">
          <Choice options={["never", "handled", "always"] as const} value={taps()} onChange={(v) => taps.set(v)} />
          <Text style={{ color: colors.muted, fontSize: 13 }}>
            {taps() === "never"
              ? "A tap outside the input only hides the keyboard; the button does not get it."
              : taps() === "handled"
                ? "The button gets the tap and the keyboard stays; a tap on empty space hides it."
                : "Taps never hide the keyboard."}
          </Text>
          <Input
            placeholder="Type here, then tap the button"
            style={{ fontSize: 16, borderWidth: 1, borderColor: colors.border, borderRadius: 10, padding: 10 }}
          />
          <Button label={`Pressed ${presses()} times`} onPress={() => presses.set(presses() + 1)} />
          <View style={{ height: 300 }} />
        </Section>
      </View>
    </Scroll>
  );
}
