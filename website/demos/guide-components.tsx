import { Input, type InputProps, type JSX, mount, Scroll, signal, split, Text, View, type YNode } from "nsh";

/** A component: a function of its props, run once. */
function Badge(props: { label: string; color?: string }) {
  return (
    <View style={{ alignSelf: "flex-start", borderRadius: 999, paddingHorizontal: 10, paddingVertical: 4, backgroundColor: props.color ?? "#ddf4ff" }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#1f2328" }}>{props.label}</Text>
    </View>
  );
}

/** Children are whatever the caller put between the tags. */
function Card(props: { title: string; children?: JSX.Element }) {
  return (
    <View style={{ backgroundColor: "#ffffff", borderRadius: 12, padding: 14, gap: 10, borderWidth: 1, borderColor: "#d0d7de" }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: "#6e7781" }}>{props.title}</Text>
      {props.children}
    </View>
  );
}

/** Own props are split off; the rest is spread onto the input and stays reactive. */
function Field(props: InputProps & { label: string }) {
  const [own, input] = split(props, ["label"]);
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 13, color: "#6e7781" }}>{own.label}</Text>
      <Input
        style={{ fontSize: 16, borderWidth: 1, borderColor: "#d0d7de", borderRadius: 8, paddingHorizontal: 10, paddingVertical: 8, backgroundColor: "#ffffff" }}
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
    <Scroll style={{ flex: 1, backgroundColor: "#f6f8fa" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 16 }}>
        <Card title="Props are live">
          <Badge label={`Tapped ${taps()} times`} color={taps() % 2 ? "#fff8c5" : "#ddf4ff"} />
          <View onPress={() => taps.update((n) => n + 1)} activeOpacity={0.6} style={{ backgroundColor: "#0969da", borderRadius: 8, padding: 10 }}>
            <Text style={{ color: "#ffffff", textAlign: "center", fontWeight: "600" }}>Tap</Text>
          </View>
        </Card>

        <Card title="split, spread and ref">
          <Field label="Your name" placeholder="Type here" value={name()} onChangeText={(t) => name.set(t)} ref={(node) => (field = node)} />
          <Text style={{ color: "#1f2328" }}>{name() ? `Hello, ${name()}!` : "Nobody yet."}</Text>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <View onPress={() => field?.focus()} activeOpacity={0.6} style={{ backgroundColor: "#8250df", borderRadius: 8, paddingHorizontal: 12, paddingVertical: 8 }}>
              <Text style={{ color: "#ffffff" }}>Focus</Text>
            </View>
            <View onPress={() => (name.set(""), field?.blur())} activeOpacity={0.6} style={{ backgroundColor: "#57606a", borderRadius: 8, paddingHorizontal: 12, paddingVertical: 8 }}>
              <Text style={{ color: "#ffffff" }}>Clear</Text>
            </View>
          </View>
        </Card>
      </View>
    </Scroll>
  );
}

mount(App);
