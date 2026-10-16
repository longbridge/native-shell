import { batch, cleanup, computed, context, effect, mount, Scroll, signal, Text, untracked, View } from "nsh";

const Accent = context(signal("#0969da"));

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
      <Text style={{ flex: 1, fontSize: 28, fontWeight: "600", fontVariant: ["tabular-nums"], color: "#1f2328" }}>
        {(tenths() / 10).toFixed(1)} s
      </Text>
      <View onPress={() => running.set(!running())} activeOpacity={0.6} style={{ backgroundColor: accent(), borderRadius: 8, paddingHorizontal: 14, paddingVertical: 8 }}>
        <Text style={{ color: "#ffffff", fontWeight: "600" }}>{running() ? "Stop" : "Start"}</Text>
      </View>
      <View onPress={() => tenths.set(0)} activeOpacity={0.6} style={{ backgroundColor: "#57606a", borderRadius: 8, paddingHorizontal: 14, paddingVertical: 8 }}>
        <Text style={{ color: "#ffffff" }}>Reset</Text>
      </View>
    </View>
  );
}

function Section(props: { title: string; children: any }) {
  return (
    <View style={{ backgroundColor: "#ffffff", borderRadius: 12, padding: 14, gap: 10 }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#6e7781" }}>{props.title}</Text>
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

  const colors = ["#0969da", "#8250df", "#1a7f37", "#bf3989"];

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#f6f8fa" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 16 }}>
        <Section title="signal + computed">
          <Text style={{ fontSize: 17, color: "#1f2328" }}>
            {price()} × {quantity()} = <Text style={{ fontWeight: "700", color: accent() }}>{total()}</Text>
          </Text>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <View onPress={() => step(1, 0)} activeOpacity={0.6} style={{ backgroundColor: "#ddf4ff", borderRadius: 8, padding: 8 }}>
              <Text>price +1</Text>
            </View>
            <View onPress={() => step(0, 1)} activeOpacity={0.6} style={{ backgroundColor: "#ddf4ff", borderRadius: 8, padding: 8 }}>
              <Text>qty +1</Text>
            </View>
            <View onPress={() => step(1, 1)} activeOpacity={0.6} style={{ backgroundColor: "#fff8c5", borderRadius: 8, padding: 8 }}>
              <Text>both (batch)</Text>
            </View>
          </View>
        </Section>

        <Section title="effect + untracked">
          <Each of={log()}>{(line) => <Text style={{ fontSize: 13, color: "#57606a" }}>{line()}</Text>}</Each>
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
                  style={{ width: 36, height: 36, borderRadius: 18, backgroundColor: c(), borderWidth: 3, borderColor: accent() === c() ? "#1f2328" : "transparent" }}
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
  <Accent.Provider value={signal("#0969da")}>
    <App />
  </Accent.Provider>
));
