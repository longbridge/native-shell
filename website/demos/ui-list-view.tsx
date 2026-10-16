// ListView: 1,000 recycled rows the app renders; plain or inset, with or without dividers.
import { mount, signal, Text, View } from "nsh";
import { Button, ListView } from "nsh/ui/mobile";

const people = Array.from({ length: 1000 }, (_, i) => ({
  id: i,
  name: `Person ${i + 1}`,
  role: ["Design", "Engineering", "Sales", "Support"][i % 4],
}));

function Header(props: { inset: () => boolean; dividers: () => boolean; toggleInset: () => void; toggleDividers: () => void }) {
  return (
    <View style={{ padding: 16, paddingTop: 24, gap: 8 }}>
      <Text style={{ fontSize: 22, fontWeight: "700" }}>ListView</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Button class="flex-1" variant="secondary" title={props.inset() ? "Plain" : "Inset"} onPress={props.toggleInset} />
        <Button
          class="flex-1"
          variant="secondary"
          title={props.dividers() ? "No dividers" : "Dividers"}
          onPress={props.toggleDividers}
        />
      </View>
    </View>
  );
}

function App() {
  const inset = signal(false);
  const dividers = signal(true);
  return (
    <ListView
      variant={inset() ? "inset" : "plain"}
      dividers={dividers()}
      each={people}
      keyExtractor={(p) => String(p.id)}
      ListHeaderComponent={
        <Header
          inset={inset}
          dividers={dividers}
          toggleInset={() => inset.set(!inset())}
          toggleDividers={() => dividers.set(!dividers())}
        />
      }
    >
      {(person) => (
        <View style={{ flexDirection: "row", alignItems: "center" }}>
          <Text style={{ flex: 1, fontWeight: "600" }}>{person().name}</Text>
          <Text style={{ color: "#737373" }}>{person().role}</Text>
        </View>
      )}
    </ListView>
  );
}

mount(App);
