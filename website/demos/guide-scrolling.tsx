import { mount, Scroll, signal, Text, View, type YNode } from "nsh";
import { createNativeValue } from "nsh/animated";

const palette = ["#0969da", "#8250df", "#bf3989", "#cf222e", "#bc4c00", "#1a7f37"];
const muted = "#57606a";

/** Pages as wide as the scroll view, with a page indicator. */
function Pager() {
  const width = signal(340);
  const page = signal(0);
  return (
    <View style={{ gap: 8 }}>
      <Scroll
        horizontal
        pagingEnabled
        showsIndicator={false}
        style={{ height: 110, borderRadius: 12 }}
        onLayout={(e) => width.set(e.width)}
        onMomentumScrollEnd={(e) => page.set(Math.round(e.x / width()))}
        onScrollEnd={(e) => page.set(Math.round(e.x / width()))}
      >
        <View style={{ flexDirection: "row" }}>
          <Each of={palette.slice(0, 4)}>
            {(color, i) => (
              <View style={{ width: width(), height: 110, backgroundColor: color(), alignItems: "center", justifyContent: "center" }}>
                <Text style={{ color: "#ffffff", fontSize: 24, fontWeight: "700" }}>Page {i() + 1}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
      <View style={{ flexDirection: "row", justifyContent: "center", gap: 6 }}>
        <Each of={[0, 1, 2, 3]}>
          {(i) => <View style={{ width: 8, height: 8, borderRadius: 4, backgroundColor: page() === i() ? "#0969da" : "#d0d7de" }} />}
        </Each>
      </View>
    </View>
  );
}

/** Cards that snap: one stop every card plus gap. */
function Cards() {
  return (
    <Scroll horizontal showsIndicator={false} snapToInterval={152} decelerationRate="fast" style={{ height: 90 }}>
      <View style={{ flexDirection: "row", gap: 12, paddingHorizontal: 4 }}>
        <Each of={Array.from({ length: 12 }, (_, i) => i)}>
          {(i) => (
            <View style={{ width: 140, height: 90, borderRadius: 14, backgroundColor: palette[i() % palette.length], alignItems: "center", justifyContent: "center" }}>
              <Text style={{ color: "#ffffff", fontWeight: "700" }}>Card {i() + 1}</Text>
            </View>
          )}
        </Each>
      </View>
    </Scroll>
  );
}

function App() {
  // The vertical offset, written natively in the frame it changes.
  const offset = createNativeValue(0);
  const y = signal(0);
  const refreshing = signal(false);
  const refreshed = signal(0);
  let scroll!: YNode;
  const refresh = () => {
    refreshing.set(true);
    setTimeout(() => {
      refreshed.update((n) => n + 1);
      refreshing.set(false);
    }, 1200);
  };
  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", paddingTop: 24 }}>
      {/* A bar that fades in and shrinks its title as the page scrolls: no JS per frame. */}
      <View
        style={{
          height: 48,
          paddingHorizontal: 16,
          justifyContent: "center",
          backgroundColor: offset.interpolate({ inputRange: [0, 80], outputRange: ["#f6f8fa", "#ffffff"] }),
          borderBottomWidth: 1,
          borderBottomColor: "#d0d7de",
        }}
      >
        <Text
          style={{
            fontSize: 22,
            fontWeight: "700",
            transformOrigin: "left center",
            transform: [{ scale: offset.interpolate({ inputRange: [0, 80], outputRange: [1, 0.8] }) }],
          }}
        >
          Scrolling
        </Text>
      </View>
      <Scroll
        ref={scroll}
        style={{ flex: 1 }}
        scrollValue={offset}
        scrollEventThrottle={100}
        onScroll={(e) => y.set(Math.round(e.y))}
        refreshing={refreshing()}
        onRefresh={refresh}
        keyboardDismissMode="on-drag"
      >
        <View style={{ padding: 16, gap: 18 }}>
          <Text style={{ color: muted }}>
            Pull down to refresh ({refreshed()}×). onScroll: y = {y()}
          </Text>
          <Text style={{ fontWeight: "600" }}>pagingEnabled</Text>
          <Pager />
          <Text style={{ fontWeight: "600" }}>snapToInterval</Text>
          <Cards />
          <Each of={Array.from({ length: 24 }, (_, i) => i + 1)}>
            {(n) => (
              <View style={{ padding: 14, borderRadius: 10, backgroundColor: "#ffffff" }}>
                <Text>Row {n()}</Text>
              </View>
            )}
          </Each>
          <View
            onPress={() => scroll.scrollTo({ y: 0 })}
            activeOpacity={0.6}
            style={{ padding: 14, borderRadius: 10, backgroundColor: "#0969da", alignItems: "center" }}
          >
            <Text style={{ color: "#ffffff", fontWeight: "600" }}>Back to the top (ref.scrollTo)</Text>
          </View>
        </View>
      </Scroll>
    </View>
  );
}

mount(App);
