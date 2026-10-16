// SelectList: 500 options, one chosen; controlled and uncontrolled.
import { mount, signal, Text, View } from "nsh";
import { Button, SelectList } from "nsh/ui/mobile";
import { ui } from "./palette";

const cities = Array.from({ length: 500 }, (_, i) => `City ${i + 1}`);
const sizes = ["Small", "Medium", "Large"];

function App() {
  const city = signal<number | null>(2);
  const lastSize = signal("none");
  return (
    <View style={{ flex: 1, backgroundColor: ui().page, padding: 16, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: ui().text }}>SelectList</Text>
      <Text style={{ color: ui().muted }}>{`Controlled: ${city() === null ? "none" : cities[city()!]}`}</Text>
      <Button
        variant="secondary"
        title="Choose the next city"
        onPress={() => city.set(city() === null ? 0 : (city()! + 1) % cities.length)}
      />
      <View style={{ flex: 1, borderRadius: 12, overflow: "hidden" }}>
        <SelectList options={cities} value={city()} onSelect={(i) => city.set(i)} />
      </View>
      <Text style={{ color: ui().muted }}>{`Uncontrolled, last chosen: ${lastSize()}`}</Text>
      <View style={{ height: 144, borderRadius: 12, overflow: "hidden" }}>
        <SelectList options={sizes} onSelect={(i) => lastSize.set(sizes[i])} />
      </View>
    </View>
  );
}

mount(App);
