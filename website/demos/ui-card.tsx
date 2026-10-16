// Card: elevated and outlined, with a header and a footer slot.
import { mount, Scroll, signal, Text, View } from "nsh";
import { Button, Card } from "nsh/ui/mobile";

function App() {
  const likes = signal(0);
  return (
    <Scroll style={{ flex: 1, backgroundColor: "#ffffff" }}>
      <View style={{ padding: 16, paddingTop: 24, gap: 16 }}>
        <Text style={{ fontSize: 22, fontWeight: "700" }}>Card</Text>
        <Card
          header={<Text>Weekly summary</Text>}
          footer={
            <View style={{ flexDirection: "row", gap: 8 }}>
              <Button title="Reset" variant="plain" onPress={() => likes.set(0)} />
              <Button title={`Like (${likes()})`} onPress={() => likes.set(likes() + 1)} />
            </View>
          }
        >
          <Text>You walked 42 km this week, 8 km more than last week.</Text>
          <Text style={{ color: "#737373" }}>The header, this body and the footer are the app's JSX.</Text>
        </Card>
        <Card variant="outlined">
          <Text style={{ fontWeight: "600" }}>Outlined</Text>
          <Text>A border, no shadow, and no header or footer: only children.</Text>
        </Card>
        <Card
          variant="outlined"
          header={
            <View style={{ flexDirection: "row", alignItems: "center", gap: 10 }}>
              <View style={{ width: 32, height: 32, borderRadius: 16, backgroundColor: "#a78bfa" }} />
              <Text>Ada Lovelace</Text>
            </View>
          }
        >
          <Text>A header can be any row, such as an avatar and a name.</Text>
        </Card>
      </View>
    </Scroll>
  );
}

mount(App);
