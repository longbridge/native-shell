// Navigation: a stack of screens with header options, pushed and popped.
import { mount, signal, Text, View } from "nsh";
import { Button, createNavigator, Screen, Stack } from "nsh/ui/mobile";

type Route = { name: "home" } | { name: "detail"; id: number };

const nav = createNavigator<Route>({ name: "home" });

function Home() {
  const opened = signal(0);
  return (
    <Screen title="Inbox">
      <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, paddingTop: 24, gap: 12 }}>
        <Text style={{ color: "#8d9198" }}>{`Opened ${opened()} messages`}</Text>
        <Button
          title="Open message 1"
          onPress={() => {
            opened.set(opened() + 1);
            nav.push({ name: "detail", id: 1 });
          }}
        />
        <Button title="Open message 2" variant="secondary" onPress={() => nav.push({ name: "detail", id: 2 })} />
      </View>
    </Screen>
  );
}

function Detail(props: { id: number }) {
  const starred = signal(false);
  return (
    <Screen
      title={`Message ${props.id}`}
      headerRight={
        <Text onPress={() => starred.set(!starred())} style={{ fontSize: 17, color: "#c8f26d" }}>
          {starred() ? "★" : "☆"}
        </Text>
      }
    >
      <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, gap: 12 }}>
        <Text style={{ color: "#eceef1" }}>{`The body of message ${props.id}.`}</Text>
        <Button title="Next message" onPress={() => nav.push({ name: "detail", id: props.id + 1 })} />
        <Button title="Back to the inbox" variant="secondary" onPress={() => nav.popToTop()} />
      </View>
    </Screen>
  );
}

function App() {
  return (
    <Stack navigator={nav} style={{ flex: 1 }}>
      {(route) => (route.name === "home" ? <Home /> : <Detail id={route.id} />)}
    </Stack>
  );
}

mount(App);
