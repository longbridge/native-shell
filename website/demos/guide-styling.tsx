import { mount, Scroll, signal, Text, View } from "nsh";

const styles = {
  tile: {
    width: 96,
    height: 96,
    borderRadius: 16,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#ffffff",
  },
  label: { fontSize: 12, fontWeight: "600", color: "#1f2328", textAlign: "center" },
  selected: { borderWidth: 3, borderColor: "#0969da" },
} as const;

function Tile(props: { label: string; style?: object; dark?: boolean }) {
  return (
    <View style={[styles.tile, props.style]}>
      <Text style={[styles.label, props.dark && { color: "#ffffff" }]}>{props.label}</Text>
    </View>
  );
}

function App() {
  const on = signal(false);
  const picked = signal(1);

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#f6f8fa" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 20 }}>
        <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 16, justifyContent: "center" }}>
          <Tile label="boxShadow" style={{ boxShadow: "0 6px 16px rgba(0,0,0,0.18)" }} />
          <Tile label="dashed border" style={{ borderWidth: 2, borderStyle: "dashed", borderColor: "#8250df" }} />
          <Tile label="gradient" dark style={{ backgroundImage: "linear-gradient(135deg, #54aeff, #8250df)" }} />
          <Tile label="radial" style={{ backgroundImage: "radial-gradient(circle, #fff8c5, #fd8c73)" }} />
          <Tile label="corners" style={{ borderTopLeftRadius: 40, borderBottomRightRadius: 40, backgroundColor: "#dafbe1" }} />
          <Tile label="continuous" dark style={{ borderRadius: 28, borderCurve: "continuous", backgroundColor: "#1a7f37" }} />
          <Tile label="rotate + scale" style={{ backgroundColor: "#ddf4ff", transform: [{ rotate: "-8deg" }, { scale: 0.9 }] }} />
          <Tile label="outline" style={{ outlineWidth: 2, outlineColor: "#bf3989", outlineOffset: 3 }} />
          <Tile label="opacity 0.5" dark style={{ backgroundColor: "#1f2328", opacity: 0.5 }} />
        </View>

        <View style={{ gap: 10 }}>
          <Text style={{ fontSize: 13, fontWeight: "600", color: "#6e7781" }}>STYLE ARRAYS: TAP TO SELECT</Text>
          <View style={{ flexDirection: "row", gap: 10 }}>
            <Each of={["Small", "Medium", "Large"]}>
              {(size, i) => (
                <View
                  onPress={() => picked.set(picked() === i() ? -1 : i())}
                  style={[
                    { flex: 1, padding: 12, borderRadius: 10, backgroundColor: "#ffffff", alignItems: "center", borderWidth: 3, borderColor: "transparent" },
                    picked() === i() && styles.selected,
                  ]}
                >
                  <Text style={{ color: "#1f2328" }}>{size()}</Text>
                </View>
              )}
            </Each>
          </View>
        </View>

        <View style={{ gap: 10 }}>
          <Text style={{ fontSize: 13, fontWeight: "600", color: "#6e7781" }}>TRANSITIONS: TAP THE BOX</Text>
          <View
            onPress={() => on.set(!on())}
            style={{
              height: 72,
              borderRadius: on() ? 36 : 12,
              backgroundColor: on() ? "#8250df" : "#54aeff",
              transform: [{ translateX: on() ? 120 : 0 }],
              width: 150,
              alignItems: "center",
              justifyContent: "center",
              transitionProperty: "all",
              transitionDuration: 400,
              transitionTimingFunction: "ease-in-out",
            }}
          >
            <Text style={{ color: "#ffffff", fontWeight: "600" }}>{on() ? "On" : "Off"}</Text>
          </View>
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
