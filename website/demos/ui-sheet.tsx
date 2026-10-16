// Sheet: a bottom panel dragged down, tapped outside or closed with a button.
import { mount, signal, Text, View } from "nsh";
import { Button, Sheet } from "nsh/ui/mobile";

function App() {
  const open = signal(false);
  const closes = signal(0);
  const opens = signal(0);
  return (
    <View style={{ flex: 1, backgroundColor: "#ffffff", padding: 16, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700" }}>Sheet</Text>
      <Text style={{ color: "#737373" }}>{`Opened ${opens()} times, closed by the user ${closes()} times`}</Text>
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
        <Text style={{ fontSize: 17, fontWeight: "600" }}>Share the photo</Text>
        <Text style={{ color: "#737373" }}>Drag the panel down, tap the dimmed backdrop or press Done.</Text>
        <Button title="Copy link" variant="secondary" onPress={() => open.set(false)} />
        <Button title="Done" onPress={() => open.set(false)} />
      </Sheet>
    </View>
  );
}

mount(App);
