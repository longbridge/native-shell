import { Scroll, signal, Text, View } from "nsh";
import { createNativeValue } from "nsh/animated";
import { I18nManager } from "nsh/device";
import { Image } from "nsh/image";
import { List } from "nsh/list";

const card = {
  width: 150,
  height: 110,
  alignItems: "center",
  justifyContent: "center",
  backgroundColor: "#ffffff",
} as const;

function Label(props: { children: string }) {
  return <Text style={{ fontSize: 13, fontWeight: "600", color: "#1f2328" }}>{props.children}</Text>;
}

/** Two faces back to back: each hides while turned away. */
function FlipCard() {
  const angle = createNativeValue(0);
  let flipped = false;
  const flip = () => {
    flipped = !flipped;
    angle.timing(flipped ? 180 : 0, { duration: 500, easing: "easeInOut" });
  };
  const face = { ...card, position: "absolute", left: 0, top: 0, borderRadius: 14, backfaceVisibility: "hidden" } as const;
  return (
    <View onPress={flip} style={{ width: 150, height: 110 }}>
      <View style={{ ...face, backgroundColor: "#ddf4ff", transform: [{ perspective: 600 }, { rotateY: angle }] }}>
        <Label>Front (tap)</Label>
      </View>
      <View
        style={{
          ...face,
          backgroundColor: "#fff8c5",
          transform: [{ perspective: 600 }, { rotateY: angle.interpolate({ inputRange: [0, 180], outputRange: [180, 360] }) }],
        }}
      >
        <Label>Back: backfaceVisibility</Label>
      </View>
    </View>
  );
}

/** Rows, `marginStart` and text under a right-to-left layout. */
function RightToLeft() {
  return (
    <View style={{ width: "100%", gap: 10, padding: 14, backgroundColor: "#ffffff", borderRadius: 12 }}>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
        <Text style={{ flex: 1, fontWeight: "600" }}>{I18nManager.isRTL ? "Right to left" : "Left to right"}</Text>
        <Text
          onPress={() => I18nManager.forceRTL(!I18nManager.isRTL)}
          style={{ color: "#ffffff", backgroundColor: "#0969da", paddingHorizontal: 12, paddingVertical: 6, borderRadius: 8, overflow: "hidden" }}
        >
          Flip
        </Text>
      </View>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Each of={["1", "2", "3"]}>
          {(n, i) => (
            <View
              style={{
                width: 44,
                height: 44,
                borderRadius: 8,
                alignItems: "center",
                justifyContent: "center",
                backgroundColor: ["#fd8c73", "#54aeff", "#4ac26b"][i()],
                marginStart: i() === 0 ? 16 : 0,
                borderTopStartRadius: 22,
              }}
            >
              <Text style={{ color: "#ffffff", fontWeight: "700" }}>{n()}</Text>
            </View>
          )}
        </Each>
      </View>
      <Text>Text aligns to the start side; the first box has marginStart 16 and a round top-start corner.</Text>
      <Text style={{ fontSize: 13, color: "#6e7781" }}>A horizontal scroll view and a horizontal list start at the start side:</Text>
      <Scroll horizontal showsIndicator={false} style={{ height: 40 }}>
        <View style={{ flexDirection: "row", gap: 6, paddingHorizontal: 4 }}>
          <Each of={Array.from({ length: 12 }, (_, i) => i + 1)}>
            {(n) => (
              <View style={{ width: 40, height: 40, borderRadius: 6, backgroundColor: "#ddf4ff", alignItems: "center", justifyContent: "center" }}>
                <Text>{String(n())}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
      <List each={Array.from({ length: 30 }, (_, i) => i + 1)} horizontal estimatedItemSize={46} showsIndicator={false} style={{ height: 40 }}>
        {(n) => (
          <View style={{ width: 40, height: 40, marginEnd: 6, borderRadius: 6, backgroundColor: "#fff8c5", alignItems: "center", justifyContent: "center" }}>
            <Text>{String(n())}</Text>
          </View>
        )}
      </List>
      <Text style={{ textAlign: "left", color: "#6e7781" }}>textAlign left (the start side right to left)</Text>
      <Text style={{ writingDirection: "rtl", color: "#6e7781" }}>مرحبا بالعالم — writingDirection rtl</Text>
    </View>
  );
}

export function Styles() {
  const clip = signal(true);
  const front = signal(0);
  return (
    <Scroll style={{ flex: 1 }}>
    <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 24, padding: 24, justifyContent: "center" }}>
      <View style={{ ...card, borderRadius: 16, shadowOpacity: 0.18, shadowRadius: 12, shadowOffset: { width: 0, height: 6 } }}>
        <Label>Shadow</Label>
      </View>
      <View
        style={{
          ...card,
          borderTopLeftRadius: 28,
          borderTopRightRadius: 28,
          borderWidth: 2,
          borderColor: "#8250df",
        }}
      >
        <Label>Top corners</Label>
      </View>
      <View
        style={{
          ...card,
          backgroundColor: "#ddf4ff",
          borderTopLeftRadius: 44,
          borderTopRightRadius: 6,
          borderBottomRightRadius: 28,
          borderWidth: 3,
          borderColor: "#0969da",
          shadowColor: "#0969da",
          shadowOpacity: 0.35,
          shadowRadius: 10,
          shadowOffset: { width: 4, height: 8 },
        }}
      >
        <Label>Mixed radii</Label>
      </View>
      <View
        onPress={() => clip.set(!clip())}
        style={{
          ...card,
          backgroundColor: "#fff8c5",
          borderRadius: 24,
          borderWidth: 2,
          borderColor: "#9a6700",
          overflow: clip() ? "hidden" : "visible",
          shadowOpacity: 0.3,
          shadowRadius: 8,
          shadowOffset: { width: 0, height: 4 },
        }}
      >
        <View style={{ position: "absolute", left: -30, top: -30, width: 90, height: 90, borderRadius: 45, backgroundColor: "#bf3989" }} />
        <Label>{clip() ? "Clipped (tap)" : "Visible (tap)"}</Label>
      </View>
      <Image
        src="https://picsum.photos/seed/nsh/300/220"
        style={{
          width: 150,
          height: 110,
          borderTopLeftRadius: 36,
          borderBottomRightRadius: 36,
          backgroundColor: "#e1e4e8",
          shadowOpacity: 0.3,
          shadowRadius: 10,
          shadowOffset: { width: 0, height: 6 },
        }}
      />
      <View style={{ ...card, backgroundColor: "transparent", shadowOpacity: 0.5, shadowRadius: 3, shadowOffset: { width: 2, height: 2 } }}>
        <Text style={{ fontSize: 22, fontWeight: "bold", color: "#1f2328" }}>Text shadow</Text>
      </View>
      <View style={{ width: 150, alignItems: "center" }}>
        <Text
          style={{
            fontSize: 15,
            color: "#ffffff",
            backgroundColor: "#1a7f37",
            borderTopLeftRadius: 14,
            borderBottomRightRadius: 14,
            paddingHorizontal: 12,
            paddingVertical: 6,
          }}
        >
          Text corners
        </Text>
      </View>
      <View
        style={{
          ...card,
          borderTopWidth: 6,
          borderTopColor: "#cf222e",
          borderRightWidth: 2,
          borderRightColor: "#1a7f37",
          borderBottomWidth: 4,
          borderBottomColor: "#0969da",
          borderLeftWidth: 1,
          borderLeftColor: "#8250df",
        }}
      >
        <Label>Per-side borders</Label>
      </View>
      <View style={{ ...card, gap: 8 }}>
        <View style={{ width: 120, height: 30, borderRadius: 8, borderWidth: 2, borderColor: "#0969da", borderStyle: "dashed" }} />
        <View style={{ width: 120, height: 30, borderRadius: 8, borderWidth: 3, borderColor: "#bf3989", borderStyle: "dotted" }} />
        <Label>Dashed / dotted</Label>
      </View>
      <View style={{ ...card, width: 324, gap: 8 }}>
        <View style={{ flexDirection: "row", gap: 12 }}>
          <View style={{ width: 64, height: 64, borderRadius: 20, backgroundColor: "#8250df" }} />
          <View style={{ width: 64, height: 64, borderRadius: 20, borderCurve: "continuous", backgroundColor: "#8250df" }} />
          <View
            style={{ width: 64, height: 64, borderRadius: 20, borderCurve: "continuous", borderWidth: 3, borderColor: "#0969da", borderTopLeftRadius: 4 }}
          />
          <View style={{ width: 64, height: 30, borderRadius: 15, borderCurve: "continuous", backgroundColor: "#1a7f37" }} />
        </View>
        <Label>Circular / continuous corners</Label>
      </View>
      <View onPress={() => front.set((front() + 1) % 3)} style={{ ...card, backgroundColor: "transparent" }}>
        <Each of={["#fd8c73", "#54aeff", "#4ac26b"]}>
          {(color, i) => (
            <View
              style={{
                position: "absolute",
                left: 20 + i() * 30,
                top: 10 + i() * 22,
                width: 70,
                height: 56,
                borderRadius: 10,
                backgroundColor: color(),
                zIndex: front() === i() ? 10 : 0,
                shadowOpacity: 0.2,
                shadowRadius: 4,
                shadowOffset: { width: 0, height: 2 },
              }}
            />
          )}
        </Each>
        <View style={{ position: "absolute", bottom: -18 }}>
          <Label>zIndex (tap)</Label>
        </View>
      </View>
      <View style={{ ...card, backgroundColor: "#ddf4ff", transform: [{ perspective: 400 }, { rotateY: "35deg" }] }}>
        <Label>rotateY + perspective</Label>
      </View>
      <View style={{ ...card, backgroundColor: "#fbefff", transform: [{ perspective: 400 }, { rotateX: "40deg" }] }}>
        <Label>rotateX</Label>
      </View>
      <View style={{ ...card, backgroundColor: "#fff1e5", transform: [{ skewX: "-12deg" }] }}>
        <Label>skewX (iOS)</Label>
      </View>
      <View style={{ ...card, backgroundColor: "#1f2328", opacity: 0.45 }}>
        <Text style={{ color: "#ffffff", fontWeight: "600" }}>opacity 0.45</Text>
      </View>
      <View android={{ elevation: 10 }} style={{ ...card, borderRadius: 16 }}>
        <Label>elevation 10 (Android)</Label>
      </View>
      <View style={{ ...card, backgroundColor: "transparent" }}>
        <View style={{ width: 60, height: 60, borderWidth: 1, borderColor: "#d0d7de", borderStyle: "dashed", position: "absolute", left: 45, top: 15 }} />
        <View
          style={{
            width: 60,
            height: 60,
            position: "absolute",
            left: 45,
            top: 15,
            backgroundColor: "rgba(130,80,223,0.6)",
            transformOrigin: "top left",
            transform: [{ rotate: "30deg" }],
          }}
        />
        <View style={{ position: "absolute", bottom: -18 }}>
          <Label>transformOrigin top left</Label>
        </View>
      </View>
      <FlipCard />
      <View style={{ ...card, alignItems: "flex-start", paddingHorizontal: 16, gap: 2 }}>
        <Text style={{ fontSize: 13, color: "#6e7781" }}>Proportional / tabular</Text>
        <Text style={{ fontSize: 18 }}>1,111.11</Text>
        <Text style={{ fontSize: 18 }}>8,888.88</Text>
        <Text style={{ fontSize: 18, fontVariant: ["tabular-nums"] }}>1,111.11</Text>
        <Text style={{ fontSize: 18, fontVariant: ["tabular-nums"] }}>8,888.88</Text>
      </View>
      <View style={{ ...card }}>
        <Text style={{ fontSize: 18, fontVariant: ["small-caps"] }}>Small Caps</Text>
        <Text style={{ fontSize: 18, fontVariant: ["oldstyle-nums"] }}>Oldstyle 1234567890</Text>
      </View>
      <RightToLeft />
    </View>
    </Scroll>
  );
}
