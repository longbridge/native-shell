// The nsh app examples/embed-gpui runs, several times at once, inside a GPUI
// Kit window. It sets no theme of its own: the components of nsh/ui/mobile
// take the host's (`AppView::set_theme`), in its light or dark mode
// (`AppView::set_appearance`), and change with it.

import { mount, signal, Text, View } from "nsh";
import { colorScheme } from "nsh/device";
import { Button, Card, Group, Stepper, TextField } from "nsh/ui/mobile";

const symbol = signal("AAPL");
const quantity = signal(10);
const orders = signal<string[]>([]);

function place(side: "Buy" | "Sell") {
  orders.update((o) => [`${side} ${quantity()} ${symbol() || "?"}`, ...o].slice(0, 4));
}

function App() {
  const muted = () => (colorScheme() === "dark" ? "#a1a1aa" : "#71717a");
  return (
    <View style={{ flex: 1, padding: 16, gap: 12 }}>
      <Card header={<Text style={{ fontSize: 17, fontWeight: "600" }}>Order ticket</Text>}>
        <TextField label="Symbol" value={symbol()} onChange={(v) => symbol.set(v.toUpperCase())} clearable />
        <TextField label="Note" placeholder="Optional" />
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
          <Text>Quantity</Text>
          <Stepper value={quantity()} min={1} max={1000} onChange={(v) => quantity.set(v)} />
        </View>
        <View style={{ flexDirection: "row", gap: 8 }}>
          <View style={{ flex: 1 }}>
            <Button title="Buy" onPress={() => place("Buy")} />
          </View>
          <View style={{ flex: 1 }}>
            <Button title="Sell" variant="danger" onPress={() => place("Sell")} />
          </View>
        </View>
      </Card>
      <Group header={<Text style={{ color: muted() }}>Recent orders</Text>}>
        {orders().length === 0 ? <Text style={{ color: muted() }}>None yet</Text> : orders().map((o) => <Text>{o}</Text>)}
      </Group>
    </View>
  );
}

mount(App);
