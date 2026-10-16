import { Scroll, signal, Text, View } from "nsh";
import { fontScale } from "nsh/device";
import { Image } from "nsh/image";
import pacifico from "../fonts/Pacifico-Regular.ttf";
import logo from "../logo@2x.png";

const LOREM =
  "Native text, typeset once on the engine thread with CoreText and drawn on the main thread without any layout work.";

export function TextDemo() {
  const caption = { fontSize: 12, color: "#6e7781", marginTop: 10 } as const;
  const link = { color: "#0969da", textDecorationLine: "underline" } as const;
  const tapped = signal("Tap a link in the sentence");
  const likes = signal(3);
  return (
    <Scroll style={{ flex: 1 }}>
      <View style={{ padding: 20, gap: 4 }}>
        <Text style={{ fontSize: 16, lineHeight: 24 }} onPress={() => tapped.set("the sentence (not a link)")}>
          {"By continuing you agree to the "}
          <Text style={link} onPress={() => tapped.set("Terms of service")} onLongPress={() => tapped.set("Terms (long press)")}>
            Terms of service
          </Text>
          {" and the "}
          <Text style={link} onPress={() => tapped.set("Privacy policy")}>
            Privacy policy
          </Text>
          {", which mention "}
          <Text style={{ fontWeight: "700" }}>
            nested <Text style={link} onPress={() => tapped.set("a link inside bold text")}>links</Text> too
          </Text>
          .
        </Text>
        <Text style={caption}>{`Pressed: ${tapped()}`}</Text>
        <Text style={caption}>Views and images inside text (they wrap with it)</Text>
        <Text style={{ fontSize: 16, lineHeight: 26 }}>
          {"Made with "}
          <Image src={logo} style={{ width: 18, height: 18, borderRadius: 4 }} />
          {" nsh. Status "}
          <View style={{ backgroundColor: "#1a7f37", borderRadius: 9, paddingHorizontal: 7, height: 18, justifyContent: "center" }}>
            <Text style={{ color: "#ffffff", fontSize: 11, fontWeight: "700" }}>LIVE</Text>
          </View>
          {" and a button inside the sentence "}
          <View
            onPress={() => likes.set(likes() + 1)}
            style={{ backgroundColor: "#ddf4ff", borderRadius: 6, paddingHorizontal: 8, height: 22, justifyContent: "center" }}
          >
            <Text style={{ color: "#0969da", fontSize: 13 }}>♥ {likes()}</Text>
          </View>
          {" that takes presses of its own."}
        </Text>
        <Text style={{ fontFamily: pacifico, fontSize: 30, color: "#8250df" }}>Custom fonts</Text>
        <Text style={caption}>Styles</Text>
        <Text style={{ fontSize: 17 }}>
          <Text style={{ fontWeight: "300" }}>Light </Text>
          <Text style={{ fontWeight: "bold" }}>Bold </Text>
          <Text style={{ fontStyle: "italic" }}>Italic </Text>
          <Text style={{ fontWeight: "bold", fontStyle: "italic" }}>Both </Text>
          <Text style={{ letterSpacing: 4 }}>Spaced</Text>
        </Text>
        <Text style={{ fontSize: 17 }}>
          <Text style={{ textDecorationLine: "underline" }}>Underline</Text>{" "}
          <Text style={{ textDecorationLine: "line-through", textDecorationColor: "#cf222e" }}>Strikethrough</Text>{" "}
          <Text style={{ textDecorationLine: "underline line-through", color: "#0969da" }}>Both</Text>
        </Text>
        <Text style={{ fontSize: 17, textTransform: "uppercase" }}>
          uppercase <Text style={{ textTransform: "capitalize" }}>capitalized words here</Text>
        </Text>
        <Text style={{ fontSize: 26, fontWeight: "800", color: "#ffffff", textShadowColor: "rgba(0,0,0,0.6)", textShadowOffset: { width: 2, height: 2 }, textShadowRadius: 4, backgroundColor: "#54aeff", paddingHorizontal: 8 }}>
          Text shadow
        </Text>
        <Text style={caption}>System text size (fontScale {fontScale().toFixed(2)})</Text>
        <Text style={{ fontSize: 17 }}>Scales with the system setting</Text>
        <Text allowFontScaling={false} style={{ fontSize: 17 }}>
          Fixed size (allowFontScaling false)
        </Text>
        <Text maxFontSizeMultiplier={1.2} style={{ fontSize: 17 }}>
          At most 1.2× (maxFontSizeMultiplier)
        </Text>
        <Text style={caption}>numberOfLines=2 (tail)</Text>
        <Text numberOfLines={2} style={{ fontSize: 15 }}>{LOREM}</Text>
        <Text style={caption}>numberOfLines=1 head / middle / clip</Text>
        <Text numberOfLines={1} ellipsizeMode="head" style={{ fontSize: 15 }}>{LOREM}</Text>
        <Text numberOfLines={1} ellipsizeMode="middle" style={{ fontSize: 15 }}>{LOREM}</Text>
        <Text numberOfLines={1} ellipsizeMode="clip" style={{ fontSize: 15 }}>{LOREM}</Text>
        <Text style={caption}>Centered, 3 lines</Text>
        <Text numberOfLines={3} style={{ fontSize: 15, textAlign: "center", lineHeight: 22 }}>
          {LOREM} {LOREM}
        </Text>
      </View>
    </Scroll>
  );
}
