import { mount, Scroll, signal, Text, View, type YNode } from "nsh";

const colors = ["#c8f26d", "#7ea8ff", "#b79cff", "#ff6170", "#ffb547", "#45d49a"];

function App() {
  const page = signal(0);
  const y = signal(0);
  let scroller!: YNode;
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1", paddingHorizontal: 20 }}>scroll</Text>

      <Text style={{ color: "#8d9198", paddingHorizontal: 20 }}>Horizontal, pagingEnabled · page {page() + 1}</Text>
      <Scroll
        horizontal
        pagingEnabled
        showsIndicator={false}
        style={{ height: 140, flexGrow: 0 }}
        onMomentumScrollEnd={(e) => page.set(Math.round(e.x / 390))}
      >
        <Each of={colors}>
          {(color, index) => (
            <View style={{ width: 390, padding: 12 }}>
              <View style={{ flex: 1, borderRadius: 16, backgroundColor: color(), alignItems: "center", justifyContent: "center" }}>
                <Text style={{ color: "#101113", fontSize: 32, fontWeight: "700" }}>{index() + 1}</Text>
              </View>
            </View>
          )}
        </Each>
      </Scroll>

      <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", paddingHorizontal: 20 }}>
        <Text style={{ color: "#8d9198" }}>Vertical · y = {Math.round(y())}</Text>
        <View
          onPress={() => scroller.scrollTo({ y: 0, animated: true })}
          activeOpacity={0.6}
          style={{
            backgroundColor: "#24262a",
            backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
            borderWidth: 1,
            borderColor: "#2c2e33",
            borderRadius: 10,
            paddingHorizontal: 12,
            paddingVertical: 6,
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
          }}
        >
          <Text style={{ color: "#eceef1" }}>scrollTo top</Text>
        </View>
      </View>
      <Scroll ref={scroller} style={{ flex: 1 }} scrollEventThrottle={16} onScroll={(e) => y.set(e.y)}>
        <View style={{ paddingHorizontal: 20, paddingBottom: 20, gap: 8 }}>
          <Each of={Array.from({ length: 40 }, (_, i) => i + 1)}>
            {(n) => (
              <View
                style={{
                  backgroundColor: "#18191c",
                  backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)",
                  borderRadius: 12,
                  padding: 14,
                  borderWidth: 1,
                  borderColor: "#2c2e33",
                  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.05)",
                }}
              >
                <Text style={{ color: "#eceef1" }}>Row {n()}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
    </View>
  );
}

mount(App);
