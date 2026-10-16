import { Scroll, signal, Text, View } from "nsh";
import { Animated, createNativeValue, Easing, LayoutAnimation } from "nsh/animated";
import { Button, colors, Section } from "../ui";

const TRACK = 220;

/** One dot per easing, all started together natively. */
function Curves() {
  const curves: [string, Easing][] = [
    ["linear", Easing.linear],
    ["ease", Easing.ease],
    ["quad out", Easing.out(Easing.quad)],
    ["cubic inOut", Easing.inOut(Easing.cubic)],
    ["sin in", Easing.sin],
    ["circle out", Easing.out(Easing.circle)],
    ["exp inOut", Easing.inOut(Easing.exp)],
    ["back out", Easing.out(Easing.back())],
    ["elastic out", Easing.out(Easing.elastic(1))],
    ["bounce out", Easing.out(Easing.bounce)],
  ];
  const dots = curves.map(() => createNativeValue(0));
  let there = false;
  const play = () => {
    there = !there;
    Animated.parallel(
      dots.map((x, i) => Animated.timing(x, { toValue: there ? TRACK : 0, duration: 1000, easing: curves[i][1] })),
    ).start();
  };
  return (
    <View style={{ gap: 6 }}>
      <Each of={curves}>
        {(c, i) => (
          <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
            <Text style={{ width: 84, fontSize: 12, color: colors.muted }}>{c()[0]}</Text>
            <View style={{ width: TRACK + 14, height: 14, borderRadius: 7, backgroundColor: "#f6f8fa" }}>
              <View
                style={{
                  width: 14,
                  height: 14,
                  borderRadius: 7,
                  backgroundColor: colors.accent,
                  transform: [{ translateX: dots[i()] }],
                }}
              />
            </View>
          </View>
        )}
      </Each>
      <Button label="Play" onPress={play} />
    </View>
  );
}

const palette = ["#fd8c73", "#f2cc60", "#4ac26b", "#54aeff", "#8250df"];

/** sequence, parallel, stagger and loop, with stop and reset. */
function Composition() {
  const ys = palette.map(() => createNativeValue(0));
  const spin = createNativeValue(0);
  const fade = createNativeValue(1);
  const state = signal("idle");
  const report = (name: string) => (finished: boolean) => state.set(`${name}: ${finished ? "finished" : "stopped"}`);

  const stagger = Animated.stagger(
    80,
    ys.map((y) =>
      Animated.sequence([
        Animated.timing(y, { toValue: -36, duration: 220, easing: Easing.out(Easing.quad) }),
        Animated.spring(y, { toValue: 0, stiffness: 300, damping: 12 }),
      ]),
    ),
  );
  const sequence = Animated.sequence([
    Animated.timing(spin, { toValue: 1, duration: 500, easing: Easing.inOut(Easing.cubic) }),
    Animated.delay(200),
    Animated.parallel([
      Animated.timing(fade, { toValue: 0.2, duration: 300 }),
      Animated.timing(spin, { toValue: 0, duration: 600, easing: Easing.out(Easing.back()) }),
    ]),
    Animated.timing(fade, { toValue: 1, duration: 300 }),
  ]);
  const pulse = Animated.loop(
    Animated.sequence([
      Animated.timing(fade, { toValue: 0.3, duration: 400, easing: Easing.sin }),
      Animated.timing(fade, { toValue: 1, duration: 400, easing: Easing.sin }),
    ]),
  );
  const run = (name: string, a: typeof stagger) => () => {
    state.set(`${name}…`);
    a.start().then(report(name));
  };
  return (
    <View style={{ gap: 12 }}>
      <View style={{ flexDirection: "row", justifyContent: "center", alignItems: "flex-end", gap: 10, height: 80 }}>
        <Each of={palette}>
          {(color, i) => (
            <View
              style={{
                width: 34,
                height: 34,
                borderRadius: 8,
                backgroundColor: color(),
                opacity: fade,
                transform: [
                  { translateY: ys[i()] },
                  { rotate: spin.interpolate({ inputRange: [0, 1], outputRange: ["0deg", `${(i() + 1) * 36}deg`] }) },
                ],
              }}
            />
          )}
        </Each>
      </View>
      <Text style={{ fontSize: 13, color: colors.muted, textAlign: "center" }}>{state()}</Text>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
        <Chip label="Stagger" onPress={run("stagger", stagger)} />
        <Chip label="Sequence" onPress={run("sequence", sequence)} />
        <Chip label="Loop" onPress={run("loop", pulse)} />
        <Chip label="Stop" onPress={() => [stagger, sequence, pulse].forEach((a) => a.stop())} />
        <Chip label="Reset" onPress={() => [stagger, sequence, pulse].forEach((a) => a.reset())} />
      </View>
    </View>
  );
}

function Chip(props: { label: string; onPress: () => void }) {
  return (
    <Text
      onPress={props.onPress}
      style={{
        paddingHorizontal: 12,
        paddingVertical: 6,
        borderRadius: 14,
        overflow: "hidden",
        backgroundColor: "#ddf4ff",
        color: colors.accent,
        fontWeight: "600",
      }}
    >
      {props.label}
    </Text>
  );
}

/** Springs with different settings, released together. */
function Springs() {
  const settings = [
    { label: "gentle", stiffness: 120, damping: 14 },
    { label: "wobbly", stiffness: 180, damping: 8 },
    { label: "stiff", stiffness: 400, damping: 30 },
  ];
  const xs = settings.map(() => createNativeValue(0));
  let there = false;
  const play = () => {
    there = !there;
    Animated.parallel(xs.map((x, i) => Animated.spring(x, { toValue: there ? TRACK : 0, ...settings[i] }))).start();
  };
  return (
    <View style={{ gap: 6 }}>
      <Each of={settings}>
        {(s, i) => (
          <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
            <Text style={{ width: 84, fontSize: 12, color: colors.muted }}>{s().label}</Text>
            <View style={{ width: TRACK + 20, height: 20 }}>
              <View
                style={{ width: 20, height: 20, borderRadius: 4, backgroundColor: "#8250df", transform: [{ translateX: xs[i()] }] }}
              />
            </View>
          </View>
        )}
      </Each>
      <Button label="Release" color="#8250df" onPress={play} />
    </View>
  );
}

/** A header that hides while the list scrolls up and comes back as it
 * scrolls down: `diffClamp` of the scroll offset, natively. */
function HidingHeader() {
  const scroll = createNativeValue(0);
  const header = scroll.diffClamp(0, 44);
  return (
    <View style={{ height: 260, borderRadius: 10, overflow: "hidden", backgroundColor: "#f6f8fa" }}>
      <Scroll style={{ flex: 1 }} scrollValue={scroll}>
        <View style={{ paddingTop: 52, paddingHorizontal: 12, paddingBottom: 12, gap: 8 }}>
          <Each of={Array.from({ length: 30 }, (_, i) => i + 1)}>
            {(n) => (
              <View style={{ padding: 10, borderRadius: 8, backgroundColor: colors.surface }}>
                <Text>Row {n()}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
      <View
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          top: 0,
          height: 44,
          justifyContent: "center",
          paddingHorizontal: 12,
          backgroundColor: colors.accent,
          transform: [{ translateY: header.mul(-1) }],
          opacity: header.interpolate({ inputRange: [0, 44], outputRange: [1, 0], easing: Easing.in(Easing.quad) }),
        }}
      >
        <Text style={{ color: "#ffffff", fontWeight: "700" }}>Scroll up to hide me</Text>
      </View>
    </View>
  );
}

/** Colors and angles from one value; outside its range it extends on the
 * right only. */
function Interpolations() {
  const t = createNativeValue(0);
  const at = signal(0);
  const go = (to: number) => {
    at.set(to);
    t.timing(to, { duration: 700, easing: Easing.inOut(Easing.cubic) });
  };
  return (
    <View style={{ gap: 10 }}>
      <View style={{ flexDirection: "row", justifyContent: "center", gap: 24, height: 90, alignItems: "center" }}>
        <View
          style={{
            width: 60,
            height: 60,
            borderRadius: 12,
            backgroundColor: t.interpolate({ inputRange: [0, 1, 2], outputRange: ["#54aeff", "#8250df", "#cf222e"] }),
            transform: [{ rotate: t.interpolate({ inputRange: [0, 1], outputRange: ["0deg", "180deg"], extrapolateRight: "extend" }) }],
          }}
        />
        <View
          style={{
            width: 60,
            height: 60,
            borderRadius: 30,
            backgroundColor: "#4ac26b",
            transform: [{ scale: t.interpolate({ inputRange: [0, 1], outputRange: [0.6, 1.2], extrapolate: "clamp" }) }],
          }}
        />
      </View>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Each of={[0, 1, 2]}>
          {(n) => <Chip label={`t = ${n()}${at() === n() ? " ●" : ""}`} onPress={() => go(n())} />}
        </Each>
      </View>
      <Text style={{ fontSize: 12, color: colors.muted }}>
        Color through three stops; rotation extends past 1 (to 360° at 2); scale clamps at 1.2.
      </Text>
    </View>
  );
}

/** Layout changes animated natively by the next commit. */
function Layout() {
  const open = signal(false);
  return (
    <View style={{ gap: 10 }}>
      <View style={{ flexDirection: open() ? "column" : "row", gap: 8 }}>
        <Each of={palette}>
          {(color) => <View style={{ height: open() ? 24 : 44, flex: open() ? undefined : 1, borderRadius: 8, backgroundColor: color() }} />}
        </Each>
      </View>
      <Button
        label={open() ? "Row" : "Column"}
        color="#57606a"
        onPress={() => {
          LayoutAnimation.configureNext(LayoutAnimation.Presets.spring);
          open.set(!open());
        }}
      />
    </View>
  );
}

export function Animations() {
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Easing curves">
          <Curves />
        </Section>
        <Section title="Sequence, parallel, stagger, loop">
          <Composition />
        </Section>
        <Section title="Springs">
          <Springs />
        </Section>
        <Section title="Interpolation">
          <Interpolations />
        </Section>
        <Section title="diffClamp: a header that hides on scroll">
          <HidingHeader />
        </Section>
        <Section title="Layout animation">
          <Layout />
        </Section>
      </View>
    </Scroll>
  );
}
