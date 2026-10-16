import { mount, Scroll, signal, Text, View, type YNode } from "nsh";

const colors = ["#0969da", "#8250df", "#bf3989", "#cf222e", "#bc4c00", "#1a7f37"];

function App() {
  const page = signal(0);
  const y = signal(0);
  let scroller!: YNode;
  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#1f2328", paddingHorizontal: 20 }}>scroll</Text>

      <Text style={{ color: "#57606a", paddingHorizontal: 20 }}>Horizontal, pagingEnabled · page {page() + 1}</Text>
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
                <Text style={{ color: "#ffffff", fontSize: 32, fontWeight: "700" }}>{index() + 1}</Text>
              </View>
            </View>
          )}
        </Each>
      </Scroll>

      <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", paddingHorizontal: 20 }}>
        <Text style={{ color: "#57606a" }}>Vertical · y = {Math.round(y())}</Text>
        <View
          onPress={() => scroller.scrollTo({ y: 0, animated: true })}
          activeOpacity={0.6}
          style={{ backgroundColor: "#ddf4ff", borderRadius: 8, paddingHorizontal: 12, paddingVertical: 6 }}
        >
          <Text style={{ color: "#0969da" }}>scrollTo top</Text>
        </View>
      </View>
      <Scroll ref={scroller} style={{ flex: 1 }} scrollEventThrottle={16} onScroll={(e) => y.set(e.y)}>
        <View style={{ paddingHorizontal: 20, paddingBottom: 20, gap: 8 }}>
          <Each of={Array.from({ length: 40 }, (_, i) => i + 1)}>
            {(n) => (
              <View style={{ backgroundColor: "#ffffff", borderRadius: 10, padding: 14, borderWidth: 1, borderColor: "#d0d7de" }}>
                <Text style={{ color: "#1f2328" }}>Row {n()}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
    </View>
  );
}

mount(App);
