import { signal, Text, View } from "nsh";
import { createNativeValue } from "nsh/animated";
import { Image } from "nsh/image";
import { List } from "nsh/list";
import logo from "../logo@2x.png";
import { Icon } from "../icons";
import { Badge, colors, finish, Press, radius, Stat, type Tone, toneColor } from "../ui";

type Photo = { id: number; title: string; author: string; tone: Tone; views: number };

const authors = ["Ada", "Linus", "Grace", "Ken", "Barbara"];
const tones: Tone[] = ["accent", "info", "warn", "up", "danger"];
const places = ["Harbor", "Ridge", "Old town", "Dunes", "Canal", "Orchard", "Quarry", "Pier", "Meadow"];
const PAGE = 200;

function makePhotos(from: number, count: number): Photo[] {
  return Array.from({ length: count }, (_, i) => {
    const id = from + i;
    return {
      id,
      title: `${places[id % places.length]} ${String(id).padStart(4, "0")}`,
      author: authors[id % authors.length],
      tone: tones[id % tones.length],
      views: ((id * 7919) % 9800) + 120,
    };
  });
}

/** 1234 → "1,234" (no Intl in the runtime). */
const grouped = (n: number) => String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
const compact = (n: number) => (n >= 1000 ? `${(n / 1000).toFixed(1)}k` : String(n));

function PhotoRow(props: { photo: Photo }) {
  return (
    <Press scale={0.985} accessibilityLabel={props.photo.title}>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingHorizontal: 16, height: 76 }}>
        <Image
          src={`https://picsum.photos/seed/${props.photo.id}/112/112`}
          style={{
            width: 56,
            height: 56,
            borderRadius: radius.inset,
            borderCurve: "continuous",
            borderWidth: 1,
            borderColor: colors.border,
            backgroundColor: colors.raised,
            ...finish.raised,
          }}
        />
        <View style={{ flex: 1, gap: 4, alignSelf: "stretch", justifyContent: "center", borderBottomWidth: 1, borderBottomColor: colors.line }}>
          <Text numberOfLines={1} style={{ fontSize: 15, fontWeight: "600", color: colors.text }}>
            {props.photo.title}
          </Text>
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <View style={{ width: 6, height: 6, borderRadius: 3, backgroundColor: toneColor(props.photo.tone) }} />
            <Text style={{ fontSize: 13, color: colors.muted }}>{props.photo.author}</Text>
            <Text style={{ fontSize: 13, color: colors.faint, fontVariant: ["tabular-nums"] }}>· {compact(props.photo.views)} views</Text>
          </View>
        </View>
        <Text style={{ fontSize: 13, color: colors.faint, fontVariant: ["tabular-nums"] }}>#{props.photo.id}</Text>
      </View>
    </Press>
  );
}

export function Photos() {
  const photos = signal(makePhotos(1, 1000));
  const refreshing = signal(false);
  const refreshes = signal(0);
  // Scroll offset, fed natively: the header's parallax, its fade and the
  // compact bar run without any JS per frame.
  const scrollY = createNativeValue(0);

  const refresh = () => {
    refreshing.set(true);
    setTimeout(() => {
      photos.set(makePhotos(1, 1000));
      refreshes.set(refreshes() + 1);
      refreshing.set(false);
    }, 800);
  };
  const loadMore = () => {
    const current = photos();
    console.log("loading more after", current.length);
    photos.set([...current, ...makePhotos(current.length + 1, PAGE)]);
  };

  const hero = (
    <View
      style={{
        paddingHorizontal: 16,
        paddingTop: 8,
        paddingBottom: 16,
        opacity: scrollY.interpolate({ inputRange: [0, 150], outputRange: [1, 0] }),
        // Moves at half speed: the list slides over it.
        transform: [{ translateY: scrollY.interpolate({ inputRange: [-200, 0, 300], outputRange: [0, 0, 150] }) }],
      }}
    >
      <View
        style={{
          backgroundColor: colors.surface,
          borderRadius: radius.card,
          borderCurve: "continuous",
          borderWidth: 1,
          borderColor: colors.border,
          padding: 16,
          gap: 12,
          ...finish.card,
        }}
      >
        <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
          <View
            style={{
              width: 40,
              height: 40,
              borderRadius: 12,
              borderCurve: "continuous",
              backgroundColor: colors.raised,
              borderWidth: 1,
              borderColor: colors.border,
              alignItems: "center",
              justifyContent: "center",
              ...finish.raised,
            }}
          >
            <Image src={logo} style={{ width: 24, height: 24 }} />
          </View>
          <View style={{ flex: 1, gap: 2 }}>
            <Text style={{ fontSize: 17, fontWeight: "600", color: colors.text }}>Photo stream</Text>
            <Text style={{ fontSize: 13, color: colors.muted }}>A recycled list of remote images</Text>
          </View>
          <Badge label="Remote" tone="info" />
        </View>
        <Text style={{ fontSize: 40, fontWeight: "600", color: colors.text, fontVariant: ["tabular-nums"], letterSpacing: -1 }}>
          {grouped(photos().length)}
          <Text style={{ fontSize: 15, fontWeight: "500", color: colors.accent }}> photos</Text>
        </Text>
        <View style={{ flexDirection: "row", borderTopWidth: 1, borderTopColor: colors.line, paddingTop: 10 }}>
          <Stat width="33%" label="Pages" value={String(Math.ceil(photos().length / PAGE))} />
          <Stat width="33%" label="Per page" value={String(PAGE)} />
          <Stat width="33%" label="Refreshes" value={String(refreshes())} tone={refreshes() ? "accent" : undefined} />
        </View>
      </View>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 8, paddingTop: 12 }}>
        <Icon name="arrow-down" size={14} tint={colors.faint} />
        <Text style={{ flex: 1, fontSize: 13, color: colors.faint }}>Pull to refresh. Scrolling to the end loads {PAGE} more.</Text>
      </View>
    </View>
  );

  return (
    <View style={{ flex: 1, backgroundColor: colors.background }}>
      <List
        each={photos()}
        estimatedItemSize={76}
        style={{ flex: 1 }}
        contentContainerStyle={{ paddingBottom: 24 }}
        scrollValue={scrollY}
        refreshing={refreshing()}
        onRefresh={refresh}
        onEndReached={loadMore}
        ListHeaderComponent={hero}
        ListFooterComponent={
          <Text style={{ textAlign: "center", fontSize: 13, color: colors.faint, padding: 16 }}>Loading {PAGE} more…</Text>
        }
      >
        {(photo) => <PhotoRow photo={photo()} />}
      </List>
      {/* The compact bar fades in once the hero has scrolled away. */}
      <View
        pointerEvents="none"
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          height: 44,
          paddingHorizontal: 16,
          flexDirection: "row",
          alignItems: "center",
          gap: 8,
          backgroundColor: colors.background,
          borderBottomWidth: 1,
          borderBottomColor: colors.border,
          opacity: scrollY.interpolate({ inputRange: [140, 190], outputRange: [0, 1] }),
          transform: [{ translateY: scrollY.interpolate({ inputRange: [140, 190], outputRange: [-8, 0] }) }],
        }}
      >
        <Text style={{ flex: 1, fontSize: 15, fontWeight: "600", color: colors.text }}>Photo stream</Text>
        <Text style={{ fontSize: 13, color: colors.muted, fontVariant: ["tabular-nums"] }}>{grouped(photos().length)} photos</Text>
      </View>
    </View>
  );
}
