// `nsh/ui/mobile`'s Rust components laying out the app's content: a
// `ListView` recycles rows the app renders, a `Group` wraps each child in a
// row. The app writes the content; the components only place it.
import { signal, Text, View } from "nsh";
import { Button, Group, ListView, SelectList, Sheet, Stepper, type StepperRef, TextField } from "nsh/ui/mobile";

const inset = signal(false);

const cities = Array.from({ length: 500 }, (_, i) => `City ${i + 1}`);

const stocks = Array.from({ length: 300 }, (_, i) => ({
  id: i,
  symbol: `SYM${String(i).padStart(3, "0")}`,
  price: (50 + ((i * 37) % 400) + (i % 7) * 0.13).toFixed(2),
  change: (((i * 13) % 21) - 10) / 4,
}));

function Header() {
  const taps = signal(0);
  const quantity = signal(2);
  const asked = signal("");
  let own!: StepperRef;
  const name = signal("Ada");
  const sheet = signal(false);
  const city = signal<number | null>(null);
  return (
    <View style={{ padding: 16, gap: 16 }}>
      <TextField label="Name (controlled; × clears and focuses)" placeholder="Your name" value={name()} onChange={(v) => name.set(v)} />
      <TextField label="Password" placeholder="At least 8 characters" secure />
      <Text style={{ fontSize: 13, color: "#57606a" }}>{`Hello, ${name() || "nobody"}`}</Text>
      <Button title="Open the sheet (drag it down)" onPress={() => sheet.set(true)} />
      <Sheet open={sheet()} onClose={() => sheet.set(false)}>
        <Text style={{ fontSize: 17, fontWeight: "600" }}>A sheet in Rust</Text>
        <Text style={{ color: "#57606a" }}>It follows the finger natively and springs back; drag it down, tap outside or press back to close.</Text>
        <Button title="Done" onPress={() => sheet.set(false)} />
      </Sheet>
      <Text style={{ fontSize: 13, color: "#57606a" }}>{`SelectList: 500 rows of Rust data, chosen: ${city() === null ? "none" : cities[city()!]}`}</Text>
      <View style={{ height: 240, borderRadius: 12, overflow: "hidden" }}>
        <SelectList options={cities} value={city()} onSelect={(v) => city.set(v)} />
      </View>
      <Group header={<Text>Group: each child is a row</Text>} footer={<Text>The rows are JSX; the group adds the box and the dividers.</Text>}>
        <Text>Notifications</Text>
        <Text>Privacy</Text>
        <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
          <Text>Taps</Text>
          <Text>{taps()}</Text>
        </View>
      </Group>
      <Group header={<Text>Stepper: state and timers in Rust (hold a button)</Text>}>
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
          <Text>Own value</Text>
          <Stepper ref={own} defaultValue={5} min={0} max={50} />
        </View>
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
          <Text>{`Controlled: ${quantity()}`}</Text>
          <Stepper value={quantity()} min={0} max={10} step={0.5} onChange={(v) => quantity.set(v)} />
        </View>
      </Group>
      <Button
        title={asked() || "Ask the stepper, then reset it"}
        variant="secondary"
        onPress={async () => {
          // Methods of a Rust component, called from JS.
          asked.set(`It was ${await own.value()}; reset to 5`);
          own.setValue(5);
        }}
      />
      <Button title="Tap" variant="secondary" onPress={() => taps.set(taps() + 1)} />
      <Button title={inset() ? "Plain rows" : "Inset rows"} variant="plain" onPress={() => inset.set(!inset())} />
      <Text style={{ fontSize: 13, color: "#57606a" }}>ListView below: 300 rows the app renders, recycled natively.</Text>
    </View>
  );
}

export function MobileUi() {
  return (
    <ListView
      variant={inset() ? "inset" : "plain"}
      each={stocks}
      keyExtractor={(s) => String(s.id)}
      ListHeaderComponent={Header}
    >
      {(stock) => (
        <View style={{ flexDirection: "row", alignItems: "center" }}>
          <Text style={{ flex: 1, fontWeight: "600" }}>{stock().symbol}</Text>
          <Text style={{ width: 80, textAlign: "right" }}>{stock().price}</Text>
          <Text style={{ width: 64, textAlign: "right", color: stock().change >= 0 ? "#1a7f37" : "#cf222e" }}>
            {(stock().change >= 0 ? "+" : "") + stock().change.toFixed(2)}%
          </Text>
        </View>
      )}
    </ListView>
  );
}
