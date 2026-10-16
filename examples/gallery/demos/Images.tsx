import { Scroll, signal, Text, View } from "nsh";
import { Image } from "nsh/image";
import bounce from "../bounce.gif";
import logo from "../logo@2x.png";
import spinner from "../spinner.webp";
import { Button, colors, Section } from "../ui";

const wide = "https://picsum.photos/seed/nsh-wide/480/240";

function Caption(props: { children: string }) {
  return <Text style={{ fontSize: 12, color: colors.muted, textAlign: "center" }}>{props.children}</Text>;
}

/** Network images behind a placeholder, fading in; "Reload" asks for
 * images that are not cached yet. */
function Placeholders() {
  const seed = signal(1);
  const fades = [0, 300, 1000];
  return (
    <View style={{ gap: 10 }}>
      <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
        <Each of={fades}>
          {(fade) => (
            <View style={{ gap: 4, alignItems: "center" }}>
              <Image
                src={`https://picsum.photos/seed/nsh-fade-${seed()}-${fade()}/300/300`}
                defaultSource={logo}
                fadeDuration={fade()}
                resizeMode="cover"
                style={{ width: 100, height: 100, borderRadius: 12, backgroundColor: "#eaeef2" }}
              />
              <Caption>{`fade ${fade()}ms`}</Caption>
            </View>
          )}
        </Each>
      </View>
      <Button label="Reload" onPress={() => seed.set(seed() + 1)} />
    </View>
  );
}

function Blur() {
  const radii = [0, 2, 6, 16];
  return (
    <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
      <Each of={radii}>
        {(radius) => (
          <View style={{ gap: 4, alignItems: "center" }}>
            <Image src={wide} blurRadius={radius()} style={{ width: 76, height: 76, borderRadius: 8 }} />
            <Caption>{`blur ${radius()}`}</Caption>
          </View>
        )}
      </Each>
    </View>
  );
}

function Info() {
  const big = "https://picsum.photos/seed/nsh-progress/2400/1600";
  const size = signal("");
  const prefetched = signal("");
  const progress = signal("");
  const show = signal(false);
  return (
    <View style={{ gap: 10 }}>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <Button
          label="getSize"
          onPress={() =>
            Image.getSize(big).then(
              (s) => size.set(`${s.width}×${s.height} px`),
              (e) => size.set(String(e)),
            )
          }
        />
        <Button label="prefetch" color="#8250df" onPress={() => Image.prefetch(big).then((ok) => prefetched.set(ok ? "prefetched" : "failed"))} />
        <Button label={show() ? "Hide" : "Show"} color="#1a7f37" onPress={() => show.set(!show())} />
      </View>
      <Text style={{ color: colors.muted }}>
        {[size(), prefetched(), progress()].filter(Boolean).join(" · ") || "Prefetch, then show: no download, no progress"}
      </Text>
      {show() && (
        <Image
          src={big}
          onProgress={(e) => progress.set(`${Math.round((e.loaded / Math.max(1, e.total)) * 100)}%`)}
          onLoad={() => progress.update((p) => (p ? `${p} loaded` : "loaded (no download)"))}
          style={{ height: 160, borderRadius: 10, backgroundColor: "#eaeef2" }}
        />
      )}
    </View>
  );
}

export function Images() {
  const events = signal<string[]>([]);
  const log = (line: string) => events.set([...events(), line]);
  const loaded = signal("loading…");
  const failed = signal("loading…");
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Resize modes (a 2:1 image in squares)">
          <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 12, justifyContent: "space-between" }}>
            <Each of={["cover", "contain", "stretch", "center"] as const}>
              {(mode) => (
                <View style={{ gap: 4 }}>
                  <Image src={wide} resizeMode={mode()} style={{ width: 140, height: 140, backgroundColor: "#eaeef2" }} />
                  <Caption>{mode()}</Caption>
                </View>
              )}
            </Each>
          </View>
        </Section>
        <Section title="Corners and borders">
          <View style={{ flexDirection: "row", gap: 12, justifyContent: "center" }}>
            <Image src={wide} style={{ width: 90, height: 90, borderRadius: 45 }} />
            <Image src={wide} style={{ width: 90, height: 90, borderRadius: 16, borderWidth: 3, borderColor: "#8250df" }} />
            <Image src={wide} style={{ width: 90, height: 90, borderTopLeftRadius: 40, borderBottomRightRadius: 40 }} />
          </View>
        </Section>
        <Section title="Assets and tint">
          <View style={{ flexDirection: "row", gap: 16, alignItems: "center", justifyContent: "center" }}>
            <View style={{ gap: 4, alignItems: "center" }}>
              <Image src={logo} />
              <Caption>logo@2x.png</Caption>
            </View>
            <Each of={["#cf222e", "#1a7f37", "#0969da"]}>
              {(tint) => (
                <View style={{ gap: 4, alignItems: "center" }}>
                  <Image src={logo} style={{ tintColor: tint() }} />
                  <Caption>{tint()}</Caption>
                </View>
              )}
            </Each>
          </View>
        </Section>
        <Section title="Animated GIF and WebP">
          <View style={{ flexDirection: "row", gap: 24, justifyContent: "center" }}>
            <View style={{ gap: 4, alignItems: "center" }}>
              <Image src={bounce} />
              <Caption>bounce.gif</Caption>
            </View>
            <View style={{ gap: 4, alignItems: "center" }}>
              <Image src={spinner} />
              <Caption>spinner.webp</Caption>
            </View>
            <View style={{ gap: 4, alignItems: "center" }}>
              <Image src={bounce} style={{ width: 48, height: 48, borderRadius: 24 }} />
              <Caption>scaled, round</Caption>
            </View>
          </View>
        </Section>
        <Section title="Placeholder (defaultSource) and fade in">
          <Placeholders />
        </Section>
        <Section title="Blur radius">
          <Blur />
        </Section>
        <Section title="Image.getSize, Image.prefetch, onProgress">
          <Info />
        </Section>
        <Section title="Events">
          <View style={{ flexDirection: "row", gap: 12, alignItems: "center" }}>
            <Image
              src="https://picsum.photos/seed/nsh-load/300/200"
              onLoadStart={() => log("onLoadStart")}
              onLoad={(e) => {
                log("onLoad");
                loaded.set(`onLoad: ${e.width}×${e.height}`);
              }}
              onError={(e) => loaded.set(`onError: ${e.message}`)}
              onLoadEnd={() => log("onLoadEnd")}
              style={{ width: 90, height: 60, borderRadius: 8, backgroundColor: "#eaeef2" }}
            />
            <Text style={{ flex: 1 }}>{loaded()}</Text>
          </View>
          <View style={{ flexDirection: "row", gap: 12, alignItems: "center" }}>
            <Image
              src="https://example.invalid/missing.png"
              onError={(e) => failed.set(`onError: ${e.message}`)}
              style={{ width: 90, height: 60, borderRadius: 8, backgroundColor: "#eaeef2" }}
            />
            <Text numberOfLines={3} style={{ flex: 1 }}>{failed()}</Text>
          </View>
          <Caption>{events().join(" → ") || "no load events yet"}</Caption>
        </Section>
      </View>
    </Scroll>
  );
}
