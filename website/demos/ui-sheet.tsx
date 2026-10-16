// Sheet: a bottom panel dragged down, tapped outside or closed with a button.
import { mount, signal, Text, View } from "nsh";
import { Button, Sheet } from "nsh/ui/mobile";
import { ui } from "./palette";

function App() {
  const open = signal(false);
  const closes = signal(0);
  const opens = signal(0);
  return (
    <View style={{ flex: 1, backgroundColor: ui().page, padding: 16, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: ui().text }}>Sheet</Text>
      <Text style={{ color: ui().muted }}>{`Opened ${opens()} times, closed by the user ${closes()} times`}</Text>
      <Button title="Open the sheet" onPress={() => {
          open.set(true);
          opens.set(opens() + 1);
        }}
      />
      <Sheet
        open={open()}
        onClose={() => {
          open.set(false);
          closes.set(closes() + 1);
        }}
      >
        <Text style={{ fontSize: 17, fontWeight: "600", color: ui().text }}>Share the photo</Text>
        <Text style={{ color: ui().muted }}>Drag the panel down, tap the dimmed backdrop or press Done.</Text>
        <Button title="Copy link" variant="secondary" onPress={() => open.set(false)} />
        <Button title="Done" onPress={() => open.set(false)} />
      </Sheet>
    </View>
  );
}

mount(App);
