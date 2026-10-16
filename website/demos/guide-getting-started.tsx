import { mount, signal, Text, View } from "nsh";

// The app `nsh init` creates, with a reset button and a short history.
function App() {
  const count = signal(0);
  const history = signal<number[]>([]);

  const change = (delta: number) => {
    const next = count() + delta;
    count.set(next);
    history.update((h) => [next, ...h].slice(0, 4));
  };

  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 20, paddingTop: 24, alignItems: "center", justifyContent: "center", gap: 16 }}>
      <Text style={{ fontSize: 15, color: "#8d9198" }}>You tapped</Text>
      <Text style={{ fontSize: 56, fontWeight: "700", color: count() < 0 ? "#ff6170" : "#eceef1" }}>{count()}</Text>
      <View style={{ flexDirection: "row", gap: 12 }}>
        <View
          onPress={() => change(-1)}
          activeOpacity={0.6}
          accessibilityRole="button"
          style={{
            paddingHorizontal: 20,
            paddingVertical: 12,
            borderRadius: 10,
            backgroundColor: "#24262a",
            backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
            borderWidth: 1,
            borderColor: "#2c2e33",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
          }}
        >
          <Text style={{ color: "#eceef1", fontSize: 17 }}>−1</Text>
        </View>
        <View
          onPress={() => change(1)}
          activeOpacity={0.6}
          accessibilityRole="button"
          style={{
            paddingHorizontal: 20,
            paddingVertical: 12,
            borderRadius: 10,
            backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
          }}
        >
          <Text style={{ color: "#13160c", fontSize: 17, fontWeight: "600" }}>Tap</Text>
        </View>
      </View>
      <If when={history().length > 0}>
        <Text style={{ fontSize: 13, color: "#8d9198" }}>Recent: {history().join(", ")}</Text>
      </If>
      <View onPress={() => (count.set(0), history.set([]))} hitSlop={8}>
        <Text style={{ fontSize: 15, color: "#c8f26d" }}>Reset</Text>
      </View>
    </View>
  );
}

mount(App);
