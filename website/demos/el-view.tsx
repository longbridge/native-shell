import { mount, signal, Text, View } from "nsh";

function App() {
  const presses = signal(0);
  const size = signal("");
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 20, paddingTop: 24, gap: 16 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1" }}>view</Text>

      <View
        onPress={() => presses.update((n) => n + 1)}
        activeOpacity={0.5}
        style={{
          backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
          borderRadius: 12,
          padding: 16,
          alignItems: "center",
        }}
      >
        <Text style={{ color: "#13160c", fontSize: 16, fontWeight: "600" }}>activeOpacity · pressed {presses()}×</Text>
      </View>

      <View
        onPress={() => presses.update((n) => n + 1)}
        underlayColor="#2c2e33"
        style={{
          backgroundColor: "#18191c",
          borderRadius: 16,
          padding: 16,
          borderWidth: 1,
          borderColor: "#2c2e33",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.38)",
        }}
      >
        <Text style={{ color: "#eceef1", fontSize: 16 }}>underlayColor (tap and hold)</Text>
      </View>

      <View class="flex-row gap-3">
        <View class="flex-1 h-20 rounded-xl bg-[#ff6170] active:bg-[#d94d5b] items-center justify-center" onPress={() => {}}>
          <Text class="text-[#13160c] font-semibold">active:</Text>
        </View>
        <View class="flex-1 h-20 rounded-xl bg-[#45d49a] items-center justify-center shadow-lg">
          <Text class="text-[#0c0d0f] font-semibold">shadow-lg</Text>
        </View>
        <View
          class="flex-1 h-20 rounded-xl items-center justify-center"
          style={{ backgroundImage: "linear-gradient(135deg, #b79cff, #7ea8ff)" }}
        >
          <Text class="text-[#101113] font-semibold">gradient</Text>
        </View>
      </View>

      <View
        onLayout={(e) => size.set(`${Math.round(e.width)} × ${Math.round(e.height)} pt`)}
        style={{ flex: 1, borderRadius: 12, borderWidth: 2, borderStyle: "dashed", borderColor: "#5a5e65", alignItems: "center", justifyContent: "center" }}
      >
        <Text style={{ color: "#8d9198" }}>onLayout: {size()}</Text>
      </View>
    </View>
  );
}

mount(App);
