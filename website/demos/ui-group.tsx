// Group: settings-style rows; each child becomes a row with a divider.
import { mount, signal, Text, View } from "nsh";
import { Group, Stepper } from "nsh/ui/mobile";

function Row(props: { label: string; value: string; onPress?: () => void }) {
  return (
    <View onPress={props.onPress} style={{ flexDirection: "row", justifyContent: "space-between" }}>
      <Text style={{ color: "#eceef1" }}>{props.label}</Text>
      <Text style={{ color: "#8d9198" }}>{props.value}</Text>
    </View>
  );
}

function App() {
  const notifications = signal(true);
  const size = signal(16);
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, paddingTop: 24, gap: 24 }}>
      <Group header={<Text style={{ color: "#8d9198" }}>ACCOUNT</Text>}>
        <Row label="Name" value="Ada Lovelace" />
        <Row label="E-mail" value="ada@example.com" />
      </Group>
      <Group
        header={<Text style={{ color: "#8d9198" }}>PREFERENCES</Text>}
        footer={<Text style={{ color: "#8d9198" }}>Tap the first row to toggle it. Rows are the app's JSX; the group adds the box and the dividers.</Text>}
      >
        <Row
          label="Notifications"
          value={notifications() ? "On" : "Off"}
          onPress={() => notifications.set(!notifications())}
        />
        <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
          <Text style={{ color: "#eceef1" }}>{`Text size ${size()}`}</Text>
          <Stepper value={size()} min={12} max={24} onChange={(v) => size.set(v)} />
        </View>
      </Group>
    </View>
  );
}

mount(App);
