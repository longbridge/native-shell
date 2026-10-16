import { type JSX, mount, signal } from "nsh";
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
    <scroll
      style={{ flex: 1, backgroundColor: color.page, display: props.visible ? "flex" : "none" }}
      keyboardDismissMode="on-drag"
      keyboardShouldPersistTaps="handled"
    >
      <view style={{ padding: 16, paddingBottom: 32, gap: 16 }}>{props.children}</view>
    </scroll>
  );
}

/** The bottom bar switching sections. */
function TabBar(props: { value: string; onChange: (name: string) => void }) {
  const insets = useSafeAreaInsets();
  return (
    <view
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
      <each of={tabs}>
        {(tab: () => (typeof tabs)[number]) => {
          const on = () => props.value === tab().name;
          return (
            <view
              onPress={() => props.onChange(tab().name)}
              accessibilityRole="tab"
              accessibilityLabel={tab().title}
              accessibilityState={{ selected: on() }}
              style={{ flex: 1, alignItems: "center", gap: 4 }}
            >
              <view
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
              </view>
              <text style={{ fontSize: 11, fontWeight: on() ? "600" : "400", color: on() ? color.text : color.muted }}>
                {tab().title}
              </text>
            </view>
          );
        }}
      </each>
    </view>
  );
}

/** Hopper Lab: flights (the hop log and the last hop's tilt), the plan for
 * the next hop (flying it adds to the log), and the vehicle. */
function Lab() {
  const tab = signal("flights");
  return (
    <Screen title={tabs.find((t) => t.name === tab())?.title ?? ""} {...header}>
      <view style={{ flex: 1, backgroundColor: color.page }}>
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
      </view>
    </Screen>
  );
}

/** Every hop, on its own screen. */
function Log() {
  return (
    <Screen title="Hop log" {...header}>
      <scroll style={{ flex: 1, backgroundColor: color.page }}>
        <view style={{ padding: 16, paddingBottom: 32 }}>
          <HopLog rows={12} />
        </view>
      </scroll>
    </Screen>
  );
}

function App() {
  return (
    <view style={{ flex: 1, backgroundColor: color.page }}>
      <StatusBar barStyle="light-content" />
      <Stack navigator={nav} style={{ flex: 1 }}>
        {(route) => (route.name === "lab" ? <Lab /> : <Log />)}
      </Stack>
    </view>
  );
}

mount(App);
