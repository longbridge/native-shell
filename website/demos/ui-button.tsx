// Button: the four variants, a disabled button and press counting.
import { mount, signal, Text, View } from "nsh";
import { Button } from "nsh/ui/mobile";

function App() {
  const presses = signal(0);
  const enabled = signal(true);
  const press = () => presses.set(presses() + 1);
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1" }}>Button</Text>
      <Text style={{ color: "#8d9198" }}>{`Pressed ${presses()} times`}</Text>
      <Button title="Primary" onPress={press} />
      <Button title="Secondary" variant="secondary" onPress={press} />
      <Button title="Danger" variant="danger" onPress={press} />
      <Button title="Plain" variant="plain" onPress={press} />
      <Button title={enabled() ? "Enabled" : "Disabled"} disabled={!enabled()} onPress={press} />
      <View style={{ flexDirection: "row", gap: 12 }}>
        <Button class="flex-1" title="Reset" variant="secondary" onPress={() => presses.set(0)} />
        <Button
          class="flex-1"
          title={enabled() ? "Disable" : "Enable"}
          variant="plain"
          onPress={() => enabled.set(!enabled())}
        />
      </View>
      <Button title="Custom corners" class="rounded-full" accessibilityLabel="A button with round ends" onPress={press} />
    </View>
  );
}

mount(App);
