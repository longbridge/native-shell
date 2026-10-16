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
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", padding: 20, paddingTop: 24, alignItems: "center", justifyContent: "center", gap: 16 }}>
      <Text style={{ fontSize: 15, color: "#6e7781" }}>You tapped</Text>
      <Text style={{ fontSize: 56, fontWeight: "700", color: count() < 0 ? "#cf222e" : "#1f2328" }}>{count()}</Text>
      <View style={{ flexDirection: "row", gap: 12 }}>
        <View
          onPress={() => change(-1)}
          activeOpacity={0.6}
          accessibilityRole="button"
          style={{ paddingHorizontal: 20, paddingVertical: 12, borderRadius: 10, backgroundColor: "#ffffff", borderWidth: 1, borderColor: "#d0d7de" }}
        >
          <Text style={{ color: "#1f2328", fontSize: 17 }}>−1</Text>
        </View>
        <View
          onPress={() => change(1)}
          activeOpacity={0.6}
          accessibilityRole="button"
          style={{ paddingHorizontal: 20, paddingVertical: 12, borderRadius: 10, backgroundColor: "#0969da" }}
        >
          <Text style={{ color: "white", fontSize: 17 }}>Tap</Text>
        </View>
      </View>
      <If when={history().length > 0}>
        <Text style={{ fontSize: 13, color: "#6e7781" }}>Recent: {history().join(", ")}</Text>
      </If>
      <View onPress={() => (count.set(0), history.set([]))} hitSlop={8}>
        <Text style={{ fontSize: 15, color: "#0969da" }}>Reset</Text>
      </View>
    </View>
  );
}

mount(App);
