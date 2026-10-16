import { batch, cleanup, computed, context, effect, mount, Scroll, signal, Text, untracked, View } from "nsh";

const Accent = context(signal("#c8f26d"));

/** A raised control, as in the gallery: a top highlight over a soft shadow. */
const raised = {
  backgroundColor: "#24262a",
  backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
  borderWidth: 1,
  borderColor: "#2c2e33",
  borderRadius: 10,
  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
} as const;

/** A stopwatch: the effect starts a timer and its cleanup stops it. */
function Stopwatch() {
  const running = signal(false);
  const tenths = signal(0);
  effect(() => {
    if (!running()) return;
    const id = setInterval(() => tenths.update((t) => t + 1), 100);
    cleanup(() => clearInterval(id));
  });
  const accent = Accent();
  return (
    <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
      <Text style={{ flex: 1, fontSize: 28, fontWeight: "600", fontVariant: ["tabular-nums"], color: "#eceef1" }}>
        {(tenths() / 10).toFixed(1)} s
      </Text>
      <View onPress={() => running.set(!running())} activeOpacity={0.6} style={{ backgroundColor: accent(), borderRadius: 8, paddingHorizontal: 14, paddingVertical: 8 }}>
        <Text style={{ color: "#0c0d0f", fontWeight: "600" }}>{running() ? "Stop" : "Start"}</Text>
      </View>
      <View onPress={() => tenths.set(0)} activeOpacity={0.6} style={{ ...raised, paddingHorizontal: 14, paddingVertical: 8 }}>
        <Text style={{ color: "#eceef1" }}>Reset</Text>
      </View>
    </View>
  );
}

function Section(props: { title: string; children: any }) {
  return (
    <View style={{ backgroundColor: "#18191c", backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)", borderWidth: 1, borderColor: "#2c2e33", boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.38)", borderRadius: 16, padding: 14, gap: 10 }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#8d9198" }}>{props.title}</Text>
      {props.children}
    </View>
  );
}

function App() {
  const price = signal(12);
  const quantity = signal(1);
  const total = computed(() => price() * quantity());
  const writes = signal(0);
  const accent = Accent();

  // Runs again whenever `total` changes; reads of `writes` are not tracked.
  const log = signal<string[]>([]);
  effect(() => {
    const t = total();
    const n = untracked(writes);
    log.update((l) => [`total ${t} after ${n} writes`, ...l].slice(0, 3));
  });

  const step = (dp: number, dq: number) => {
    batch(() => {
      price.update((p) => Math.max(0, p + dp));
      quantity.update((q) => Math.max(0, q + dq));
      writes.update((w) => w + 2);
    });
  };

  const colors = ["#c8f26d", "#7ea8ff", "#45d49a", "#b79cff"];

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#101113" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 16 }}>
        <Section title="signal + computed">
          <Text style={{ fontSize: 17, color: "#eceef1" }}>
            {price()} × {quantity()} = <Text style={{ fontWeight: "700", color: accent() }}>{total()}</Text>
          </Text>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <View onPress={() => step(1, 0)} activeOpacity={0.6} style={{ ...raised, padding: 8 }}>
              <Text style={{ color: "#eceef1" }}>price +1</Text>
            </View>
            <View onPress={() => step(0, 1)} activeOpacity={0.6} style={{ ...raised, padding: 8 }}>
              <Text style={{ color: "#eceef1" }}>qty +1</Text>
            </View>
            <View onPress={() => step(1, 1)} activeOpacity={0.6} style={{ backgroundColor: "rgba(255,181,71,0.14)", borderWidth: 1, borderColor: "rgba(255,181,71,0.3)", borderRadius: 10, padding: 8 }}>
              <Text style={{ color: "#ffb547" }}>both (batch)</Text>
            </View>
          </View>
        </Section>

        <Section title="effect + untracked">
          <Each of={log()}>{(line) => <Text style={{ fontSize: 13, color: "#8d9198" }}>{line()}</Text>}</Each>
        </Section>

        <Section title="effect + cleanup">
          <Stopwatch />
        </Section>

        <Section title="context (the value is a signal)">
          <View style={{ flexDirection: "row", gap: 10 }}>
            <Each of={colors}>
              {(c) => (
                <View
                  onPress={() => accent.set(c())}
                  style={{ width: 36, height: 36, borderRadius: 18, backgroundColor: c(), borderWidth: 3, borderColor: accent() === c() ? "#eceef1" : "transparent" }}
                />
              )}
            </Each>
          </View>
        </Section>
      </View>
    </Scroll>
  );
}

mount(() => (
  <Accent.Provider value={signal("#c8f26d")}>
    <App />
  </Accent.Provider>
));
