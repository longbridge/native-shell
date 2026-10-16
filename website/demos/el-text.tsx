import { mount, signal, Text, View } from "nsh";

const LONG =
  "Text is typeset natively (CoreText on iOS, StaticLayout on Android) and measured on the engine thread, so the main thread only draws it.";

function App() {
  const tapped = signal("nothing yet");
  const lines = signal(2);
  const link = { color: "#0969da", textDecorationLine: "underline" } as const;
  return (
    <View style={{ flex: 1, backgroundColor: "#ffffff", padding: 20, paddingTop: 24, gap: 14 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#1f2328" }}>text</Text>

      <Text style={{ fontSize: 16, lineHeight: 24, color: "#1f2328" }}>
        {"Nested text elements are spans: "}
        <Text style={{ fontWeight: "700" }}>bold</Text>
        {", "}
        <Text style={{ fontStyle: "italic", color: "#8250df" }}>italic</Text>
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
      <Text style={{ fontSize: 13, color: "#6e7781" }}>Pressed: {tapped()}</Text>

      <View style={{ height: 1, backgroundColor: "#d0d7de" }} />

      <Text numberOfLines={lines()} style={{ fontSize: 15, color: "#1f2328" }}>
        {LONG}
      </Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View
          onPress={() => lines.set(lines() === 2 ? 0 : 2)}
          activeOpacity={0.6}
          style={{ backgroundColor: "#ddf4ff", borderRadius: 8, paddingHorizontal: 12, paddingVertical: 8 }}
        >
          <Text style={{ color: "#0969da" }}>{lines() === 2 ? "Show all" : "numberOfLines 2"}</Text>
        </View>
      </View>

      <Text numberOfLines={1} ellipsizeMode="middle" style={{ fontSize: 15, color: "#57606a" }}>
        {LONG}
      </Text>

      <Text style={{ fontSize: 17, color: "#1f2328" }}>
        <Text style={{ textDecorationLine: "line-through", textDecorationColor: "#cf222e" }}>struck</Text>{" "}
        <Text style={{ textTransform: "uppercase", letterSpacing: 2 }}>spaced</Text>{" "}
        <Text style={{ fontVariant: ["tabular-nums"] }}>0123456789</Text>
      </Text>

      <Text
        style={{
          fontSize: 28,
          fontWeight: "800",
          color: "#ffffff",
          backgroundColor: "#54aeff",
          paddingHorizontal: 10,
          borderRadius: 8,
          textShadowColor: "rgba(0,0,0,0.5)",
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
