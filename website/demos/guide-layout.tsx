import { mount, Scroll, signal, Text, View } from "nsh";

type Direction = "column" | "row";
type Justify = "flex-start" | "center" | "flex-end" | "space-between" | "space-evenly";
type Align = "stretch" | "flex-start" | "center" | "flex-end";

function Choice<T extends string>(props: { label: string; options: readonly T[]; value: T; onChange: (v: T) => void }) {
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 12, fontWeight: "600", color: "#6e7781" }}>{props.label}</Text>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 6 }}>
        <Each of={props.options}>
          {(o) => (
            <View
              onPress={() => props.onChange(o())}
              style={{ paddingHorizontal: 10, paddingVertical: 5, borderRadius: 6, backgroundColor: props.value === o() ? "#0969da" : "#ffffff" }}
            >
              <Text style={{ fontSize: 12, color: props.value === o() ? "#ffffff" : "#1f2328" }}>{o()}</Text>
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
  const colors = ["#fd8c73", "#54aeff", "#4ac26b"];

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#f6f8fa" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 14 }}>
        <View
          onPress={() => direction.set(direction() === "row" ? "column" : "row")}
          style={{ alignSelf: "flex-start", paddingHorizontal: 12, paddingVertical: 6, borderRadius: 6, backgroundColor: "#0969da" }}
        >
          <Text style={{ fontSize: 13, color: "#ffffff" }}>flexDirection: {direction()} (tap to switch)</Text>
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
            backgroundColor: "#ffffff",
            flexDirection: direction(),
            justifyContent: justify(),
            alignItems: align(),
          }}
        >
          <Each of={colors}>
            {(c, i) => (
              <View style={{ minWidth: 44, minHeight: 44, padding: 6, borderRadius: 8, backgroundColor: c() }}>
                <Text style={{ color: "#ffffff", fontWeight: "700" }}>{i() + 1}</Text>
              </View>
            )}
          </Each>
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#6e7781" }}>flex: 1 / 2 / fixed, percentages, aspectRatio</Text>
        <View style={{ flexDirection: "row", gap: 8 }}>
          <View style={{ flex: 1, height: 36, borderRadius: 6, backgroundColor: "#c8e1ff" }} />
          <View style={{ flex: 2, height: 36, borderRadius: 6, backgroundColor: "#79b8ff" }} />
          <View style={{ width: 36, height: 36, borderRadius: 6, backgroundColor: "#0366d6" }} />
        </View>
        <View style={{ flexDirection: "row", gap: 8, alignItems: "flex-start" }}>
          <View style={{ width: "30%", aspectRatio: 1, borderRadius: 8, backgroundColor: "#ffdfb6" }} />
          <View style={{ width: "40%", aspectRatio: 16 / 9, borderRadius: 8, backgroundColor: "#ffab70" }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#6e7781" }}>grid</Text>
        <View style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 6 }}>
          <View style={{ gridColumn: "span 2", height: 40, borderRadius: 6, backgroundColor: "#d8b9ff" }} />
          <View style={{ height: 40, borderRadius: 6, backgroundColor: "#b083f0" }} />
          <View style={{ gridRow: "span 2", borderRadius: 6, backgroundColor: "#8250df" }} />
          <View style={{ gridColumn: "1 / 4", height: 40, borderRadius: 6, backgroundColor: "#6639ba" }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: "#6e7781" }}>position: absolute</Text>
        <View style={{ height: 100, borderRadius: 12, backgroundColor: "#ffffff" }}>
          <View style={{ position: "absolute", top: 8, left: 8, width: 60, height: 40, borderRadius: 6, backgroundColor: "#4ac26b" }} />
          <View style={{ position: "absolute", right: 8, bottom: 8, width: "40%", height: 30, borderRadius: 6, backgroundColor: "#fd8c73" }} />
          <View style={{ position: "absolute", top: 0, bottom: 0, left: "50%", width: 2, backgroundColor: "#d0d7de" }} />
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
