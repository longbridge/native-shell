import { mount, signal, Text, View } from "nsh";
import { createNativeValue } from "nsh/animated";
import { PanResponder } from "nsh/gesture";

const muted = "#8d9198";
const text = "#eceef1";
const raised = {
  backgroundColor: "#24262a",
  backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
  borderWidth: 1,
  borderColor: "#2c2e33",
  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
} as const;

/** A card dragged on the UI thread: `pan` drives two native values. */
function DragCard() {
  const x = createNativeValue(0);
  const y = createNativeValue(0);
  const flings = signal(0);
  return (
    <View style={{ height: 150, borderRadius: 12, backgroundColor: "#0c0d0f", borderWidth: 1, borderColor: "#1f2124", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)", alignItems: "center", justifyContent: "center", overflow: "hidden" }}>
      <View
        pan={{ x, y }}
        onPanEnd={(e) => {
          flings.update((n) => n + 1);
          // Spring home, starting with the release velocity.
          x.spring(0, { velocity: e.velocityX, stiffness: 200, damping: 16 });
          y.spring(0, { velocity: e.velocityY, stiffness: 200, damping: 16 });
        }}
        style={{
          width: 120,
          height: 80,
          borderRadius: 16,
          backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
          alignItems: "center",
          justifyContent: "center",
          transform: [{ translateX: x }, { translateY: y }],
        }}
      >
        <Text style={{ color: "#13160c", fontWeight: "700" }}>Drag me</Text>
        <Text style={{ color: "#13160c", fontSize: 12 }}>released {flings()}×</Text>
      </View>
    </View>
  );
}

/** A dot moved with PanResponder: every move goes through JS. */
function ResponderDot() {
  const pos = signal({ x: 0, y: 0 });
  const state = signal("idle");
  let start = { x: 0, y: 0 };
  const responder = PanResponder.create({
    onStartShouldSetPanResponder: () => true,
    onPanResponderGrant: () => {
      start = pos();
      state.set("dragging");
    },
    onPanResponderMove: (_, g) => pos.set({ x: start.x + g.dx, y: start.y + g.dy }),
    onPanResponderRelease: (_, g) => state.set(`released, vx ${g.vx.toFixed(2)} pt/ms`),
    onPanResponderTerminate: () => state.set("taken over"),
  });
  return (
    <View style={{ gap: 6 }}>
      <View style={{ height: 110, borderRadius: 12, backgroundColor: "#0c0d0f", borderWidth: 1, borderColor: "#1f2124", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)", overflow: "hidden" }}>
        <View
          {...responder.panHandlers}
          style={{
            position: "absolute",
            left: 150,
            top: 30,
            width: 50,
            height: 50,
            borderRadius: 25,
            backgroundColor: "#7ea8ff",
            transform: [{ translateX: pos().x }, { translateY: pos().y }],
          }}
        />
      </View>
      <Text style={{ color: muted, fontSize: 13 }}>PanResponder: {state()}</Text>
    </View>
  );
}

function App() {
  const log = signal<string[]>([]);
  const add = (event: string) => log.update((l) => [event, ...l].slice(0, 4));
  const dot = signal(0);
  const under = signal(0);
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, paddingTop: 24, gap: 14 }}>
      <View style={{ flexDirection: "row", gap: 10 }}>
        <View
          onPressIn={() => add("pressIn")}
          onPressOut={() => add("pressOut")}
          onPress={() => add("press")}
          onLongPress={() => add("longPress")}
          activeOpacity={0.5}
          style={{
            flex: 1,
            padding: 14,
            borderRadius: 12,
            backgroundImage: "linear-gradient(180deg, #fbfbfc, #d4d6d9)",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
            alignItems: "center",
          }}
        >
          <Text style={{ color: "#111214", fontWeight: "600" }}>activeOpacity</Text>
        </View>
        <View
          onPress={() => add("underlay")}
          underlayColor="#3a3d43"
          style={{ ...raised, flex: 1, padding: 14, borderRadius: 12, alignItems: "center" }}
        >
          <Text style={{ color: text, fontWeight: "600" }}>underlayColor</Text>
        </View>
        <View
          onPress={() => add("active:")}
          class="flex-1 items-center rounded-xl bg-[#45d49a] p-3.5 transition active:scale-95 active:bg-[#2fae7b]"
        >
          <Text class="font-semibold text-[#0c0d0f]">active:</Text>
        </View>
      </View>
      <Text style={{ color: muted, fontSize: 13 }}>{log().join(" · ") || "Tap, hold, or slide off a button"}</Text>

      <View style={{ flexDirection: "row", alignItems: "center", gap: 16 }}>
        <View
          hitSlop={24}
          onPress={() => dot.update((n) => n + 1)}
          style={{ width: 12, height: 12, borderRadius: 6, backgroundColor: "#ff6170", margin: 12 }}
        />
        <Text style={{ flex: 1, color: text }}>hitSlop 24: the dot takes taps around it ({dot()})</Text>
      </View>

      <View style={{ height: 52 }}>
        <View
          onPress={() => under.update((n) => n + 1)}
          style={{ flex: 1, borderRadius: 12, backgroundColor: "rgba(69,212,154,0.14)", alignItems: "center", justifyContent: "center" }}
        >
          <Text style={{ color: "#45d49a" }}>Under an overlay: {under()}</Text>
        </View>
        <View
          pointerEvents="none"
          style={{ position: "absolute", top: 0, left: 0, right: 0, bottom: 0, borderRadius: 12, borderWidth: 3, borderColor: "#ffb547", borderStyle: "dashed" }}
        />
      </View>

      <DragCard />
      <ResponderDot />
    </View>
  );
}

mount(App);
