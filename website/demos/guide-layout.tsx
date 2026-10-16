import { mount, Scroll, signal, Text, View } from "nsh";

type Direction = "column" | "row";
type Justify = "flex-start" | "center" | "flex-end" | "space-between" | "space-evenly";
type Align = "stretch" | "flex-start" | "center" | "flex-end";

function Choice<T extends string>(props: { label: string; options: readonly T[]; value: T; onChange: (v: T) => void }) {
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 12, fontWeight: "600", color: "#8d9198" }}>{props.label}</Text>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 6 }}>
        <Each of={props.options}>
          {(o) => (
            <View
              onPress={() => props.onChange(o())}
              style={{
                paddingHorizontal: 10,
                paddingVertical: 5,
                borderRadius: 8,
                borderWidth: 1,
                borderColor: props.value === o() ? "#c8f26d" : "#2c2e33",
                backgroundColor: props.value === o() ? "#c8f26d" : "#24262a",
              }}
            >
              <Text style={{ fontSize: 12, color: props.value === o() ? "#13160c" : "#eceef1" }}>{o()}</Text>
            </View>
          )}
        </Each>
      </View>
    </View>
  );
}

function App() {
  const direction = signal<Direction>("row");
  const justify = signal<Justify>("flex-start");
  const align = signal<Align>("stretch");
  const colors = ["#ff8a7a", "#7ea8ff", "#45d49a"];

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#101113" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 14 }}>
        <View
          onPress={() => direction.set(direction() === "row" ? "column" : "row")}
          style={{
            alignSelf: "flex-start",
            paddingHorizontal: 12,
            paddingVertical: 6,
            borderRadius: 8,
            backgroundImage: "linear-gradient(180deg, #fbfbfc, #d4d6d9)",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
          }}
        >
          <Text style={{ fontSize: 13, fontWeight: "600", color: "#111214" }}>flexDirection: {direction()} (tap to switch)</Text>
        </View>
        <Choice
          label="justifyContent"
          options={["flex-start", "center", "flex-end", "space-between", "space-evenly"] as const}
          value={justify()}
          onChange={(v) => justify.set(v)}
        />
        <Choice label="alignItems" options={["stretch", "flex-start", "center", "flex-end"] as const} value={align()} onChange={(v) => align.set(v)} />

        <View
          style={{
            height: 220,
            padding: 10,
            gap: 8,
            borderRadius: 12,
            backgroundColor: "#0c0d0f",
            borderWidth: 1,
            borderColor: "#1f2124",
            boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)",
            flexDirection: direction(),
            justifyContent: justify(),
            alignItems: align(),
          }}
        >
          <Each of={colors}>
            {(c, i) => (
              <View style={{ minWidth: 44, minHeight: 44, padding: 6, borderRadius: 8, backgroundColor: c() }}>
                <Text style={{ color: "#0c0d0f", fontWeight: "700" }}>{i() + 1}</Text>
              </View>
            )}
          </Each>
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#8d9198" }}>flex: 1 / 2 / fixed, percentages, aspectRatio</Text>
        <View style={{ flexDirection: "row", gap: 8 }}>
          <View style={{ flex: 1, height: 36, borderRadius: 6, backgroundColor: "#2b3a5c" }} />
          <View style={{ flex: 2, height: 36, borderRadius: 6, backgroundColor: "#4f6aa3" }} />
          <View style={{ width: 36, height: 36, borderRadius: 6, backgroundColor: "#7ea8ff" }} />
        </View>
        <View style={{ flexDirection: "row", gap: 8, alignItems: "flex-start" }}>
          <View style={{ width: "30%", aspectRatio: 1, borderRadius: 8, backgroundColor: "#5c4626" }} />
          <View style={{ width: "40%", aspectRatio: 16 / 9, borderRadius: 8, backgroundColor: "#ffb547" }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#8d9198" }}>grid</Text>
        <View style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 6 }}>
          <View style={{ gridColumn: "span 2", height: 40, borderRadius: 6, backgroundColor: "#3d3260" }} />
          <View style={{ height: 40, borderRadius: 6, backgroundColor: "#6a58a8" }} />
          <View style={{ gridRow: "span 2", borderRadius: 6, backgroundColor: "#b79cff" }} />
          <View style={{ gridColumn: "1 / 4", height: 40, borderRadius: 6, backgroundColor: "#8a72d6" }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#8d9198" }}>position: absolute</Text>
        <View style={{ height: 100, borderRadius: 12, backgroundColor: "#0c0d0f", borderWidth: 1, borderColor: "#1f2124", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" }}>
          <View style={{ position: "absolute", top: 8, left: 8, width: 60, height: 40, borderRadius: 6, backgroundColor: "#45d49a" }} />
          <View style={{ position: "absolute", right: 8, bottom: 8, width: "40%", height: 30, borderRadius: 6, backgroundColor: "#ff8a7a" }} />
          <View style={{ position: "absolute", top: 0, bottom: 0, left: "50%", width: 2, backgroundColor: "#2c2e33" }} />
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
