import { type JSX, mount, Scroll, signal, Text, View } from "nsh";
import { createNavigator, Screen, Stack, StatusBar, useSafeAreaInsets } from "nsh/ui/mobile";
import { FlightPlan } from "./cards/FlightPlan";
import { HopLog } from "./cards/HopLog";
import { Mass } from "./cards/Mass";
import { Rcs } from "./cards/Rcs";
import { ThrottleProfile } from "./cards/ThrottleProfile";
import { Tilt } from "./cards/Tilt";
import { Icon, type IconName } from "./icons";
import { color } from "./theme";

type Route = { name: "lab" } | { name: "log" };
const nav = createNavigator<Route>({ name: "lab" });

/** The lab's sections: what flew, what flies next, the vehicle. */
const tabs: { name: string; title: string; icon: IconName }[] = [
  { name: "flights", title: "Flights", icon: "hops" },
  { name: "plan", title: "Plan", icon: "plan" },
  { name: "vehicle", title: "Vehicle", icon: "mass" },
];

/** Screen props of the dark header. */
const header = {
  headerBackgroundColor: color.page,
  headerTintColor: color.accent,
  headerTitleColor: color.text,
} as const;

/** A scrolling column of panels. Hidden sections stay mounted (their
 * sliders and selections keep their state). */
function Section(props: { visible: boolean; children: JSX.Element }) {
  return (
    <Scroll
      style={{ flex: 1, backgroundColor: color.page, display: props.visible ? "flex" : "none" }}
      keyboardDismissMode="on-drag"
      keyboardShouldPersistTaps="handled"
    >
      <View style={{ padding: 16, paddingBottom: 32, gap: 16 }}>{props.children}</View>
    </Scroll>
  );
}

/** The bottom bar switching sections. */
function TabBar(props: { value: string; onChange: (name: string) => void }) {
  const insets = useSafeAreaInsets();
  return (
    <View
      accessibilityRole="tablist"
      style={{
        flexDirection: "row",
        paddingTop: 8,
        paddingBottom: Math.max(10, insets().bottom),
        borderTopWidth: 1,
        borderTopColor: color.line,
        backgroundColor: color.card,
      }}
    >
      <Each of={tabs}>
        {(tab) => {
          const on = () => props.value === tab().name;
          return (
            <View
              onPress={() => props.onChange(tab().name)}
              accessibilityRole="tab"
              accessibilityLabel={tab().title}
              accessibilityState={{ selected: on() }}
              style={{ flex: 1, alignItems: "center", gap: 4 }}
            >
              <View
                style={{
                  width: 52,
                  height: 30,
                  borderRadius: 15,
                  alignItems: "center",
                  justifyContent: "center",
                  backgroundColor: on() ? color.accentDim : "transparent",
                }}
              >
                <Icon name={tab().icon} size={20} tint={on() ? color.accent : color.muted} />
              </View>
              <Text style={{ fontSize: 11, fontWeight: on() ? "600" : "400", color: on() ? color.text : color.muted }}>
                {tab().title}
              </Text>
            </View>
          );
        }}
      </Each>
    </View>
  );
}

/** Hopper Lab: flights (the hop log and the last hop's tilt), the plan for
 * the next hop (flying it adds to the log), and the vehicle. */
function Lab() {
  const tab = signal("flights");
  return (
    <Screen title={tabs.find((t) => t.name === tab())?.title ?? ""} {...header}>
      <View style={{ flex: 1, backgroundColor: color.page }}>
        <Section visible={tab() === "flights"}>
          <HopLog onOpen={() => nav.push({ name: "log" })} />
          <Tilt />
        </Section>
        <Section visible={tab() === "plan"}>
          <FlightPlan />
          <ThrottleProfile />
        </Section>
        <Section visible={tab() === "vehicle"}>
          <Mass />
          <Rcs />
        </Section>
        <TabBar value={tab()} onChange={(name) => tab.set(name)} />
      </View>
    </Screen>
  );
}

/** Every hop, on its own screen. */
function Log() {
  return (
    <Screen title="Hop log" {...header}>
      <Scroll style={{ flex: 1, backgroundColor: color.page }}>
        <View style={{ padding: 16, paddingBottom: 32 }}>
          <HopLog rows={12} />
        </View>
      </Scroll>
    </Screen>
  );
}

function App() {
  return (
    <View style={{ flex: 1, backgroundColor: color.page }}>
      <StatusBar barStyle="light-content" />
      <Stack navigator={nav} style={{ flex: 1 }}>
        {(route) => (route.name === "lab" ? <Lab /> : <Log />)}
      </Stack>
    </View>
  );
}

mount(App);
