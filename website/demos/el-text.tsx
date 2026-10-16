import { mount, signal, Text, View } from "nsh";

const LONG =
  "Text is typeset natively (CoreText on iOS, StaticLayout on Android) and measured on the engine thread, so the main thread only draws it.";

function App() {
  const tapped = signal("nothing yet");
  const lines = signal(2);
  const link = { color: "#c8f26d", textDecorationLine: "underline" } as const;
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 20, paddingTop: 24, gap: 14 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1" }}>text</Text>

      <Text style={{ fontSize: 16, lineHeight: 24, color: "#eceef1" }}>
        {"Nested text elements are spans: "}
        <Text style={{ fontWeight: "700" }}>bold</Text>
        {", "}
        <Text style={{ fontStyle: "italic", color: "#b79cff" }}>italic</Text>
        {", and "}
        <Text style={link} onPress={() => tapped.set("the first link")}>
          links
        </Text>
        {" that take presses of "}
        <Text style={link} onPress={() => tapped.set("the second link")}>
          their own
        </Text>
        .
      </Text>
      <Text style={{ fontSize: 13, color: "#8d9198" }}>Pressed: {tapped()}</Text>

      <View style={{ height: 1, backgroundColor: "#1f2124" }} />

      <Text numberOfLines={lines()} style={{ fontSize: 15, color: "#eceef1" }}>
        {LONG}
      </Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View
          onPress={() => lines.set(lines() === 2 ? 0 : 2)}
          activeOpacity={0.6}
          style={{
            backgroundColor: "#24262a",
            backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
            borderWidth: 1,
            borderColor: "#2c2e33",
            borderRadius: 10,
            paddingHorizontal: 12,
            paddingVertical: 8,
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
          }}
        >
          <Text style={{ color: "#eceef1" }}>{lines() === 2 ? "Show all" : "numberOfLines 2"}</Text>
        </View>
      </View>

      <Text numberOfLines={1} ellipsizeMode="middle" style={{ fontSize: 15, color: "#8d9198" }}>
        {LONG}
      </Text>

      <Text style={{ fontSize: 17, color: "#eceef1" }}>
        <Text style={{ textDecorationLine: "line-through", textDecorationColor: "#ff6170" }}>struck</Text>{" "}
        <Text style={{ textTransform: "uppercase", letterSpacing: 2 }}>spaced</Text>{" "}
        <Text style={{ fontVariant: ["tabular-nums"] }}>0123456789</Text>
      </Text>

      <Text
        style={{
          fontSize: 28,
          fontWeight: "800",
          color: "#13160c",
          backgroundColor: "#c8f26d",
          paddingHorizontal: 10,
          borderRadius: 8,
          textShadowColor: "rgba(255,255,255,0.55)",
          textShadowOffset: { width: 2, height: 2 },
          textShadowRadius: 4,
        }}
      >
        Text shadow
      </Text>
    </View>
  );
}

mount(App);
