import { mount, Scroll, signal, Text, View } from "nsh";
import { tint, ui } from "./palette";

type Direction = "column" | "row";
type Justify = "flex-start" | "center" | "flex-end" | "space-between" | "space-evenly";
type Align = "stretch" | "flex-start" | "center" | "flex-end";

function Choice<T extends string>(props: { label: string; options: readonly T[]; value: T; onChange: (v: T) => void }) {
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 12, fontWeight: "600", color: ui().muted }}>{props.label}</Text>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 6 }}>
        <Each of={props.options}>
          {(o) => (
            <View
              onPress={() => props.onChange(o())}
              style={{
                paddingHorizontal: 10,
                paddingVertical: 5,
                borderRadius: 8,
                ...(props.value === o() ? ui().accentFill : ui().raised),
              }}
            >
              <Text style={{ fontSize: 12, color: props.value === o() ? ui().onAccent : ui().text }}>{o()}</Text>
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
  const colors = () => [ui().down, ui().info, ui().up];

  return (
    <Scroll style={{ flex: 1, backgroundColor: ui().page }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 14 }}>
        <View
          onPress={() => direction.set(direction() === "row" ? "column" : "row")}
          style={{
            alignSelf: "flex-start",
            paddingHorizontal: 12,
            paddingVertical: 6,
            borderRadius: 8,
            ...ui().primary,
          }}
        >
          <Text style={{ fontSize: 13, fontWeight: "600", color: ui().onPrimary }}>flexDirection: {direction()} (tap to switch)</Text>
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
            ...ui().inset,
            flexDirection: direction(),
            justifyContent: justify(),
            alignItems: align(),
          }}
        >
          <Each of={colors()}>
            {(c, i) => (
              <View style={{ minWidth: 44, minHeight: 44, padding: 6, borderRadius: 8, backgroundColor: c() }}>
                <Text style={{ color: ui().page, fontWeight: "700" }}>{i() + 1}</Text>
              </View>
            )}
          </Each>
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: ui().muted }}>flex: 1 / 2 / fixed, percentages, aspectRatio</Text>
        <View style={{ flexDirection: "row", gap: 8 }}>
          <View style={{ flex: 1, height: 36, borderRadius: 6, backgroundColor: tint(ui().info, 0.3) }} />
          <View style={{ flex: 2, height: 36, borderRadius: 6, backgroundColor: tint(ui().info, 0.6) }} />
          <View style={{ width: 36, height: 36, borderRadius: 6, backgroundColor: ui().info }} />
        </View>
        <View style={{ flexDirection: "row", gap: 8, alignItems: "flex-start" }}>
          <View style={{ width: "30%", aspectRatio: 1, borderRadius: 8, backgroundColor: tint(ui().warn, 0.35) }} />
          <View style={{ width: "40%", aspectRatio: 16 / 9, borderRadius: 8, backgroundColor: ui().warn }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: ui().muted }}>grid</Text>
        <View style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 6 }}>
          <View style={{ gridColumn: "span 2", height: 40, borderRadius: 6, backgroundColor: tint(ui().violet, 0.3) }} />
          <View style={{ height: 40, borderRadius: 6, backgroundColor: tint(ui().violet, 0.6) }} />
          <View style={{ gridRow: "span 2", borderRadius: 6, backgroundColor: ui().violet }} />
          <View style={{ gridColumn: "1 / 4", height: 40, borderRadius: 6, backgroundColor: tint(ui().violet, 0.8) }} />
        </View>

        <Text style={{ fontSize: 12, fontWeight: "600", color: ui().muted }}>position: absolute</Text>
        <View style={{ ...ui().inset, height: 100, borderRadius: 12 }}>
          <View style={{ position: "absolute", top: 8, left: 8, width: 60, height: 40, borderRadius: 6, backgroundColor: ui().up }} />
          <View style={{ position: "absolute", right: 8, bottom: 8, width: "40%", height: 30, borderRadius: 6, backgroundColor: ui().down }} />
          <View style={{ position: "absolute", top: 0, bottom: 0, left: "50%", width: 2, backgroundColor: ui().border }} />
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
