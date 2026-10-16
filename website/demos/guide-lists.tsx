import { effect, mount, signal, Text, View, type YNode } from "nsh";
import { List } from "nsh/list";

// Graphite rows, tinted at about 14% with the status and info colours.
const tints = ["#2a2025", "#1c2a25", "#1f2536", "#2c261c", "#24262a", "#232a33"];
const muted = "#8d9198";
const text = "#eceef1";

type Post = { id: number; title: string };

/** Bumped when the selected tab is tapped again: its list scrolls to the top. */
const toTop = signal(0);

/** An endless feed: loads more near the end, pull to refresh, scroll to an item. */
function Feed() {
  let next = 0;
  const make = (n: number): Post[] => Array.from({ length: n }, () => ({ id: next, title: `Post ${++next}` }));
  const posts = signal(make(40));
  const refreshing = signal(false);
  const visible = signal("");
  let list!: YNode;
  effect(() => {
    if (toTop() > 0) list.scrollToOffset({ offset: 0 });
  });
  return (
    <List
      ref={list}
      each={posts()}
      keyExtractor={(p) => String(p.id)}
      estimatedItemSize={56}
      style={{ flex: 1 }}
      contentContainerStyle={{ padding: 12, gap: 8 }}
      onEndReached={() => posts.update((p) => [...p, ...make(20)])}
      onEndReachedThreshold={0.5}
      refreshing={refreshing()}
      onRefresh={() => {
        refreshing.set(true);
        setTimeout(() => {
          next = 0;
          posts.set(make(40));
          refreshing.set(false);
        }, 800);
      }}
      viewabilityConfig={{ itemVisiblePercentThreshold: 50 }}
      onViewableItemsChanged={({ viewableItems }) =>
        visible.set(viewableItems.length ? `${viewableItems[0].index}–${viewableItems[viewableItems.length - 1].index}` : "")
      }
      ListHeaderComponent={
        <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingBottom: 4 }}>
          <Text style={{ flex: 1, color: muted }}>
            {posts().length} posts, visible {visible()}
          </Text>
          <Text onPress={() => list.scrollToIndex({ index: 30, viewPosition: 0.5 })} style={{ color: "#c8f26d" }}>
            #30
          </Text>
          <Text onPress={() => list.scrollToOffset({ offset: 0 })} style={{ color: "#c8f26d" }}>
            Top
          </Text>
        </View>
      }
    >
      {(post) => (
        <View style={{ padding: 16, borderRadius: 12, backgroundColor: "#18191c", backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)", borderWidth: 1, borderColor: "#2c2e33", boxShadow: "inset 0 1px 0 rgba(255,255,255,0.05)" }}>
          <Text style={{ fontSize: 16, color: text }}>{post().title}</Text>
        </View>
      )}
    </List>
  );
}

/** Three columns of different heights. */
function Grid() {
  const items = Array.from({ length: 300 }, (_, i) => ({ id: i, height: 60 + (i % 4) * 18 }));
  return (
    <List each={items} numColumns={3} estimatedItemSize={80} style={{ flex: 1 }} contentContainerStyle={{ padding: 12, gap: 8 }}>
      {(item) => (
        <View style={{ height: item().height, borderRadius: 10, backgroundColor: tints[item().id % tints.length], alignItems: "center", justifyContent: "center" }}>
          <Text style={{ fontWeight: "600", color: text }}>{item().id}</Text>
        </View>
      )}
    </List>
  );
}

/** Sticky letter headers: one list of header and name rows. */
function Contacts() {
  const names = ["Ada", "Alan", "Anita", "Barbara", "Bjarne", "Brian", "Claude", "Dennis", "Donald", "Edsger", "Frances", "Grace", "Guido", "Hedy", "Ken", "Linus", "Margaret", "Niklaus", "Radia", "Tim"];
  type Row = { header: string } | { name: string };
  const rows: Row[] = [];
  const headers: number[] = [];
  for (const letter of [...new Set(names.map((n) => n[0]))]) {
    headers.push(rows.length);
    rows.push({ header: letter });
    for (const name of names.filter((n) => n[0] === letter)) rows.push({ name });
  }
  return (
    <List each={rows} stickyHeaderIndices={headers} estimatedItemSize={44} style={{ flex: 1, backgroundColor: "#18191c" }}>
      {(row) => (
        <If
          when={"header" in row()}
          else={
            <View style={{ paddingHorizontal: 16, paddingVertical: 12, borderBottomWidth: 1, borderBottomColor: "#1f2124" }}>
              <Text style={{ fontSize: 16, color: text }}>{(row() as { name: string }).name}</Text>
            </View>
          }
        >
          <View style={{ backgroundColor: "#0c0d0f", paddingHorizontal: 16, paddingVertical: 6 }}>
            <Text style={{ fontWeight: "700", color: muted }}>{(row() as { header: string }).header}</Text>
          </View>
        </If>
      )}
    </List>
  );
}

type Tab = "feed" | "grid" | "contacts";

function App() {
  const tab = signal<Tab>("feed");
  const button = (id: Tab, label: string) => (
    <Text
      onPress={() => (tab() === id ? toTop.update((n) => n + 1) : tab.set(id))}
      style={{
        flex: 1,
        textAlign: "center",
        paddingVertical: 6,
        borderRadius: 8,
        overflow: "hidden",
        fontWeight: "600",
        color: tab() === id ? text : muted,
        backgroundColor: tab() === id ? "#24262a" : "transparent",
        backgroundImage: tab() === id ? "linear-gradient(180deg, #2d2f34, #222428)" : "none",
        boxShadow: tab() === id ? "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)" : "none",
      }}
    >
      {label}
    </Text>
  );
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", paddingTop: 24 }}>
      <View style={{ flexDirection: "row", marginHorizontal: 12, padding: 3, borderRadius: 10, backgroundColor: "#0c0d0f", borderWidth: 1, borderColor: "#1f2124", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" }}>
        {button("feed", "Feed")}
        {button("grid", "Grid")}
        {button("contacts", "Sticky")}
      </View>
      {() => {
        switch (tab()) {
          case "feed":
            return <Feed />;
          case "grid":
            return <Grid />;
          case "contacts":
            return <Contacts />;
        }
      }}
    </View>
  );
}

mount(App);
