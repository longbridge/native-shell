import { Input, type InputProps, type JSX, mount, Scroll, signal, split, Text, View, type YNode } from "nsh";

/** A component: a function of its props, run once. */
function Badge(props: { label: string; color?: string }) {
  return (
    <View style={{ alignSelf: "flex-start", borderRadius: 999, paddingHorizontal: 10, paddingVertical: 4, backgroundColor: props.color ?? "rgba(200,242,109,0.14)" }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#eceef1" }}>{props.label}</Text>
    </View>
  );
}

/** Children are whatever the caller put between the tags. */
function Card(props: { title: string; children?: JSX.Element }) {
  return (
    <View
      style={{
        backgroundColor: "#18191c",
        backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)",
        borderRadius: 16,
        padding: 14,
        gap: 10,
        borderWidth: 1,
        borderColor: "#2c2e33",
        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.38)",
      }}
    >
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#8d9198" }}>{props.title}</Text>
      {props.children}
    </View>
  );
}

/** Own props are split off; the rest is spread onto the input and stays reactive. */
function Field(props: InputProps & { label: string }) {
  const [own, input] = split(props, ["label"]);
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 13, color: "#8d9198" }}>{own.label}</Text>
      <Input
        style={{
          fontSize: 16,
          color: "#eceef1",
          borderWidth: 1,
          borderColor: "#1f2124",
          borderRadius: 12,
          paddingHorizontal: 10,
          paddingVertical: 8,
          backgroundColor: "#0c0d0f",
          boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)",
        }}
        placeholderTextColor="#5a5e65"
        {...input}
      />
    </View>
  );
}

function App() {
  const name = signal("");
  const taps = signal(0);
  let field: YNode | undefined;

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#101113" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 16 }}>
        <Card title="Props are live">
          <Badge label={`Tapped ${taps()} times`} color={taps() % 2 ? "rgba(255,181,71,0.16)" : "rgba(200,242,109,0.14)"} />
          <View onPress={() => taps.update((n) => n + 1)} activeOpacity={0.6} style={{
              backgroundImage: "linear-gradient(180deg, #fbfbfc, #d4d6d9)",
              boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
              borderRadius: 10,
              padding: 10,
            }}
          >
            <Text style={{ color: "#111214", textAlign: "center", fontWeight: "600" }}>Tap</Text>
          </View>
        </Card>

        <Card title="split, spread and ref">
          <Field label="Your name" placeholder="Type here" value={name()} onChangeText={(t) => name.set(t)} ref={(node) => (field = node)} />
          <Text style={{ color: "#eceef1" }}>{name() ? `Hello, ${name()}!` : "Nobody yet."}</Text>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <View onPress={() => field?.focus()} activeOpacity={0.6} style={{
                backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
                boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
                borderRadius: 10,
                paddingHorizontal: 12,
                paddingVertical: 8,
              }}
            >
              <Text style={{ color: "#13160c", fontWeight: "600" }}>Focus</Text>
            </View>
            <View onPress={() => (name.set(""), field?.blur())} activeOpacity={0.6} style={{
                backgroundColor: "#24262a",
                backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
                borderWidth: 1,
                borderColor: "#2c2e33",
                boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
                borderRadius: 10,
                paddingHorizontal: 12,
                paddingVertical: 8,
              }}
            >
              <Text style={{ color: "#eceef1" }}>Clear</Text>
            </View>
          </View>
        </Card>
      </View>
    </Scroll>
  );
}

mount(App);
