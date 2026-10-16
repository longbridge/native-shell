import { signal, Text, View } from "nsh";
import { createNativeValue } from "nsh/animated";

export function Physics() {
  const x = createNativeValue(0);
  const y = createNativeValue(0);
  const pressed = createNativeValue(0);
  const releases = signal(0);

  const release = (e: { velocityX: number; velocityY: number }) => {
    releases.set(releases() + 1);
    x.spring(0, { velocity: e.velocityX, stiffness: 180, damping: 14 });
    y.spring(0, { velocity: e.velocityY, stiffness: 180, damping: 14 });
  };
  const pulse = async () => {
    await pressed.timing(1, { duration: 120, easing: "easeOut" });
    await pressed.timing(0, { duration: 260, easing: "easeInOut" });
  };

  return (
    <View style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 24 }}>
      <Text style={{ fontSize: 15, color: "#6e7781" }}>Drag the card, tap it to pulse, long-press to reset</Text>
      <View
        pan={{ x, y }}
        onPanEnd={release}
        onPress={pulse}
        onLongPress={() => releases.set(0)}
        style={{
          width: 160,
          height: 160,
          borderRadius: 28,
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: pressed.interpolate({ inputRange: [0, 1], outputRange: ["#8250df", "#bf3989"] }),
          transform: [
            { translateX: x },
            { translateY: y },
            { scale: pressed.interpolate({ inputRange: [0, 1], outputRange: [1, 1.12] }) },
            { rotate: x.interpolate({ inputRange: [-200, 200], outputRange: [-12, 12] }) },
          ],
        }}
      >
        <Text style={{ color: "#ffffff", fontSize: 20, fontWeight: "bold" }}>Drag me</Text>
      </View>
      <Text style={{ fontSize: 13, color: "#6e7781" }}>Released {releases()} times</Text>
    </View>
  );
}
