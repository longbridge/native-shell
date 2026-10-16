import { mount, Scroll, signal, Text, View } from "nsh";

const styles = {
  tile: {
    width: 96,
    height: 96,
    borderRadius: 16,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#18191c",
  },
  label: { fontSize: 12, fontWeight: "600", color: "#eceef1", textAlign: "center" },
  selected: { borderWidth: 3, borderColor: "#c8f26d" },
} as const;

function Tile(props: { label: string; style?: object; dark?: boolean }) {
  return (
    <View style={[styles.tile, props.style]}>
      <Text style={[styles.label, props.dark && { color: "#0c0d0f" }]}>{props.label}</Text>
    </View>
  );
}

function App() {
  const on = signal(false);
  const picked = signal(1);

  return (
    <Scroll style={{ flex: 1, backgroundColor: "#101113" }}>
      <View style={{ padding: 20, paddingTop: 24, gap: 20 }}>
        <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 16, justifyContent: "center" }}>
          <Tile
            label="boxShadow"
            style={{ backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)", boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.6)" }}
          />
          <Tile label="dashed border" style={{ borderWidth: 2, borderStyle: "dashed", borderColor: "#b79cff" }} />
          <Tile label="gradient" dark style={{ backgroundImage: "linear-gradient(135deg, #dbf893, #7ea8ff)" }} />
          <Tile label="radial" dark style={{ backgroundImage: "radial-gradient(circle, #ffd99a, #ff8a7a)" }} />
          <Tile label="corners" style={{ borderTopLeftRadius: 40, borderBottomRightRadius: 40, backgroundColor: "#1d4a3a" }} />
          <Tile label="continuous" dark style={{ borderRadius: 28, borderCurve: "continuous", backgroundColor: "#45d49a" }} />
          <Tile label="rotate + scale" style={{ backgroundColor: "#2b3a5c", transform: [{ rotate: "-8deg" }, { scale: 0.9 }] }} />
          <Tile label="outline" style={{ outlineWidth: 2, outlineColor: "#c8f26d", outlineOffset: 3 }} />
          <Tile label="opacity 0.5" dark style={{ backgroundColor: "#eceef1", opacity: 0.5 }} />
        </View>

        <View style={{ gap: 10 }}>
          <Text style={{ fontSize: 13, fontWeight: "600", color: "#8d9198" }}>STYLE ARRAYS: TAP TO SELECT</Text>
          <View style={{ flexDirection: "row", gap: 10 }}>
            <Each of={["Small", "Medium", "Large"]}>
              {(size, i) => (
                <View
                  onPress={() => picked.set(picked() === i() ? -1 : i())}
                  style={[
                    { flex: 1, padding: 12, borderRadius: 10, backgroundColor: "#24262a", alignItems: "center", borderWidth: 3, borderColor: "transparent" },
                    picked() === i() && styles.selected,
                  ]}
                >
                  <Text style={{ color: "#eceef1" }}>{size()}</Text>
                </View>
              )}
            </Each>
          </View>
        </View>

        <View style={{ gap: 10 }}>
          <Text style={{ fontSize: 13, fontWeight: "600", color: "#8d9198" }}>TRANSITIONS: TAP THE BOX</Text>
          <View
            onPress={() => on.set(!on())}
            style={{
              height: 72,
              borderRadius: on() ? 36 : 12,
              backgroundColor: on() ? "#c8f26d" : "#7ea8ff",
              transform: [{ translateX: on() ? 120 : 0 }],
              width: 150,
              alignItems: "center",
              justifyContent: "center",
              transitionProperty: "all",
              transitionDuration: 400,
              transitionTimingFunction: "ease-in-out",
            }}
          >
            <Text style={{ color: "#0c0d0f", fontWeight: "600" }}>{on() ? "On" : "Off"}</Text>
          </View>
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
