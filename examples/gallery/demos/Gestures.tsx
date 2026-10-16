import { type GestureResponderEvent, Scroll, signal, Text, View } from "nsh";
import { createNativeValue } from "nsh/animated";
import { PanResponder } from "nsh/gesture";
import { Image } from "nsh/image";
import { Button, colors, Section } from "../ui";

/** Drag, pinch and rotate one card at the same time, all on the UI thread. */
function Card() {
  const x = createNativeValue(0);
  const y = createNativeValue(0);
  const scale = createNativeValue(1);
  const angle = createNativeValue(0);
  const reset = () => {
    x.spring(0);
    y.spring(0);
    scale.spring(1);
    angle.spring(0);
  };
  return (
    <View style={{ gap: 12, alignItems: "center" }}>
      <View style={{ height: 220, alignSelf: "stretch", alignItems: "center", justifyContent: "center", overflow: "hidden" }}>
        <View
          pan={{ x, y }}
          pinch={{ scale }}
          rotation={{ angle }}
          style={{
            width: 200,
            height: 150,
            borderRadius: 14,
            overflow: "hidden",
            transform: [{ translateX: x }, { translateY: y }, { scale }, { rotate: angle }],
          }}
        >
          <Image src="https://picsum.photos/seed/nsh-card/400/300" style={{ flex: 1 }} />
        </View>
      </View>
      <Text style={{ color: colors.muted, fontSize: 13 }}>Drag with one finger; pinch and rotate with two (Option-drag in the simulator).</Text>
      <Button label="Reset" color="#57606a" onPress={reset} />
    </View>
  );
}

/** A ball dragged with a PanResponder (positions go through JS). Its
 * responder blocks the page from scrolling while it is dragged. */
function Ball() {
  const pos = signal({ x: 0, y: 0 });
  const state = signal("idle");
  let start = { x: 0, y: 0 };
  const pan = PanResponder.create({
    onStartShouldSetPanResponder: () => true,
    onPanResponderGrant: () => {
      start = pos();
      state.set("dragging");
    },
    onPanResponderMove: (_, g) => pos.set({ x: start.x + g.dx, y: start.y + g.dy }),
    onPanResponderRelease: (_, g) => state.set(`released at ${g.vx.toFixed(2)}, ${g.vy.toFixed(2)} pt/ms`),
    onPanResponderTerminate: () => state.set("taken over"),
  });
  return (
    <View style={{ gap: 8 }}>
      <View style={{ height: 180, borderRadius: 10, backgroundColor: "#f6f8fa", overflow: "hidden" }}>
        <View
          {...pan.panHandlers}
          style={{
            position: "absolute",
            left: 130,
            top: 60,
            width: 60,
            height: 60,
            borderRadius: 30,
            backgroundColor: colors.accent,
            transform: [{ translateX: pos().x }, { translateY: pos().y }],
          }}
        />
      </View>
      <Text style={{ color: colors.muted, fontSize: 13 }}>{state()}</Text>
      <Button label="Reset" color="#57606a" onPress={() => pos.set({ x: 0, y: 0 })} />
    </View>
  );
}

/** Raw touch events, with every finger. */
function TouchLog() {
  const log = signal<string[]>([]);
  const note = (name: string) => (e: GestureResponderEvent) =>
    log.update((l) =>
      [
        `${name} at ${Math.round(e.locationX)},${Math.round(e.locationY)} (page ${Math.round(e.pageX)},${Math.round(e.pageY)}), ${e.touches.length} down`,
        ...l,
      ].slice(0, 4),
    );
  return (
    <View style={{ gap: 8 }}>
      <View
        onTouchStart={note("start")}
        onTouchMove={note("move")}
        onTouchEnd={note("end")}
        onTouchCancel={note("cancel")}
        style={{ height: 100, borderRadius: 10, backgroundColor: "#ddf4ff", alignItems: "center", justifyContent: "center" }}
      >
        <Text style={{ color: colors.accent }}>Touch here (try two fingers)</Text>
      </View>
      <Text style={{ color: colors.muted, fontSize: 13 }}>{log().join("\n") || "No touches yet"}</Text>
    </View>
  );
}

export function Gestures() {
  const log = signal<string[]>([]);
  const add = (e: string) => log.update((l) => [e, ...l].slice(0, 6));
  const slop = signal(0);
  const through = signal(0);
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Pan, pinch and rotation">
          <Card />
        </Section>
        <Section title="PanResponder (drag the ball; the page does not scroll)">
          <Ball />
        </Section>
        <Section title="Touch events">
          <TouchLog />
        </Section>
        <Section title="Press events">
          <View
            onPressIn={() => add("pressIn")}
            onPressOut={() => add("pressOut")}
            onPress={() => add("press")}
            onLongPress={() => add("longPress")}
            activeOpacity={0.5}
            style={{ padding: 18, borderRadius: 10, backgroundColor: "#ddf4ff", alignItems: "center" }}
          >
            <Text style={{ fontWeight: "600" }}>Tap, hold, or slide off</Text>
          </View>
          <Text style={{ color: colors.muted }}>{log().join(" · ") || "No events yet"}</Text>
          <View style={{ flexDirection: "row", gap: 10 }}>
            <View
              onPress={() => add("ripple")}
              onLongPress={() => add("fast long press")}
              delayLongPress={250}
              android={{ ripple: { color: "rgba(9,105,218,0.3)" } }}
              style={{ flex: 1, padding: 14, borderRadius: 10, backgroundColor: "#f6f8fa", alignItems: "center" }}
            >
              <Text>Ripple, long press 250 ms</Text>
            </View>
            <View
              onPress={() => add("borderless")}
              android={{ ripple: { borderless: true, radius: 40 } }}
              style={{ width: 56, height: 56, alignItems: "center", justifyContent: "center" }}
            >
              <Text style={{ fontSize: 22 }}>★</Text>
            </View>
          </View>
          <Text onPress={() => add("text")} activeOpacity={0.3} style={{ color: colors.accent }}>
            Text with activeOpacity
          </Text>
        </Section>
        <Section title="Hit slop">
          <View style={{ flexDirection: "row", alignItems: "center", gap: 16 }}>
            <View
              hitSlop={30}
              onPress={() => slop.set(slop() + 1)}
              style={{ width: 12, height: 12, borderRadius: 6, backgroundColor: "#cf222e", margin: 20 }}
            />
            <Text style={{ flex: 1 }}>The dot takes taps 30 pt around it: {slop()}</Text>
          </View>
        </Section>
        <Section title="Pointer events">
          <View style={{ height: 70 }}>
            <View
              onPress={() => through.set(through() + 1)}
              style={{ flex: 1, borderRadius: 10, backgroundColor: "#1a7f37", alignItems: "center", justifyContent: "center" }}
            >
              <Text style={{ color: "#ffffff" }}>Button under an overlay: {through()}</Text>
            </View>
            <View
              pointerEvents="none"
              style={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, borderRadius: 10, borderWidth: 3, borderColor: "#bf8700", borderStyle: "dashed" }}
            />
          </View>
          <Text style={{ color: colors.muted, fontSize: 13 }}>The dashed overlay has pointerEvents="none": taps reach the button.</Text>
        </Section>
        <Section title="Accessibility">
          <Each of={[{ label: "Play", hint: "Starts the song" }, { label: "Like", hint: "Adds to favorites" }]}>
            {(item) => (
              <View
                accessible
                accessibilityRole="button"
                accessibilityLabel={item().label}
                accessibilityHint={item().hint}
                onPress={() => add(item().label)}
                style={{ padding: 12, borderRadius: 10, backgroundColor: "#eaeef2" }}
              >
                <Text>{`${item().label} (labelled for VoiceOver / TalkBack)`}</Text>
              </View>
            )}
          </Each>
        </Section>
      </View>
    </Scroll>
  );
}
