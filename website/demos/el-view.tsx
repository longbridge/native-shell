import { mount, signal, Text, View } from "nsh";

function App() {
  const presses = signal(0);
  const size = signal("");
  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", padding: 20, paddingTop: 24, gap: 16 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#1f2328" }}>view</Text>

      <View
        onPress={() => presses.update((n) => n + 1)}
        activeOpacity={0.5}
        style={{ backgroundColor: "#0969da", borderRadius: 12, padding: 16, alignItems: "center" }}
      >
        <Text style={{ color: "#ffffff", fontSize: 16, fontWeight: "600" }}>activeOpacity · pressed {presses()}×</Text>
      </View>

      <View
        onPress={() => presses.update((n) => n + 1)}
        underlayColor="#d0d7de"
        style={{ backgroundColor: "#ffffff", borderRadius: 12, padding: 16, borderWidth: 1, borderColor: "#d0d7de" }}
      >
        <Text style={{ color: "#1f2328", fontSize: 16 }}>underlayColor (tap and hold)</Text>
      </View>

      <View class="flex-row gap-3">
        <View class="flex-1 h-20 rounded-xl bg-rose-500 active:bg-rose-700 items-center justify-center" onPress={() => {}}>
          <Text class="text-white font-semibold">active:</Text>
        </View>
        <View class="flex-1 h-20 rounded-xl bg-emerald-500 items-center justify-center shadow-lg">
          <Text class="text-white font-semibold">shadow-lg</Text>
        </View>
        <View
          class="flex-1 h-20 rounded-xl items-center justify-center"
          style={{ backgroundImage: "linear-gradient(135deg, #8250df, #0969da)" }}
        >
          <Text class="text-white font-semibold">gradient</Text>
        </View>
      </View>

      <View
        onLayout={(e) => size.set(`${Math.round(e.width)} × ${Math.round(e.height)} pt`)}
        style={{ flex: 1, borderRadius: 12, borderWidth: 2, borderStyle: "dashed", borderColor: "#8c959f", alignItems: "center", justifyContent: "center" }}
      >
        <Text style={{ color: "#57606a" }}>onLayout: {size()}</Text>
      </View>
    </View>
  );
}

mount(App);
