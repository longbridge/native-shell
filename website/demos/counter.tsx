import { mount, signal, Text, View } from "nsh";

// Raised: a dark control lit from above; accent: the citrus one.
const FINISH = {
  raised: {
    backgroundColor: "#24262a",
    backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
    borderWidth: 1,
    borderColor: "#2c2e33",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
  },
  accent: {
    backgroundColor: "#c8f26d",
    backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
  },
} as const;

function Button(props: { label: string; accent?: boolean; onPress: () => void }) {
  return (
    <View
      onPress={props.onPress}
      activeOpacity={0.75}
      style={{
        ...(props.accent ? FINISH.accent : FINISH.raised),
        borderRadius: 12,
        paddingVertical: 12,
        paddingHorizontal: 28,
        alignItems: "center",
      }}
    >
      <Text style={{ color: props.accent ? "#13160c" : "#eceef1", fontSize: 18, fontWeight: "bold" }}>{props.label}</Text>
    </View>
  );
}

function App() {
  const count = signal(0);
  const history = signal<number[]>([]);

  const change = (delta: number) => {
    const next = count() + delta;
    count.set(next);
    history.update((h) => [next, ...h].slice(0, 5));
  };

  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 24, paddingTop: 64, gap: 24 }}>
      <Text style={{ fontSize: 28, fontWeight: "bold", color: "#eceef1" }}>nsh counter</Text>
      <View
        style={{
          backgroundColor: "#18191c",
          backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)",
          borderWidth: 1,
          borderColor: "#2c2e33",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.38)",
          borderRadius: 16,
          padding: 24,
          alignItems: "center",
          gap: 8,
        }}
      >
        <Text style={{ fontSize: 14, color: "#8d9198" }}>Current value</Text>
        <Text style={{ fontSize: 64, fontWeight: "bold", fontVariant: ["tabular-nums"], color: count() < 0 ? "#ff6170" : "#eceef1" }}>
          {count()}
        </Text>
      </View>
      <View style={{ flexDirection: "row", gap: 12, justifyContent: "center" }}>
        <Button label="−1" onPress={() => change(-1)} />
        <Button label="+1" accent onPress={() => change(1)} />
      </View>
      <View style={{ gap: 6 }}>
        <Text style={{ fontSize: 14, color: "#8d9198" }}>
          Recent values: <Text style={{ fontWeight: "bold", color: "#eceef1" }}>{history().length}</Text>
        </Text>
        <Each of={history()}>{(value) => <Text style={{ fontSize: 16, color: "#eceef1" }}>• {value()}</Text>}</Each>
      </View>
    </View>
  );
}

mount(App);
