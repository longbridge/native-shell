import { effect, mount, signal, Text, View, type YNode } from "nsh";
import { List } from "nsh/list";

const tints = ["#fde2e4", "#e2ece9", "#dfe7fd", "#fff1e6", "#e8e8e4", "#cddafd"];
const muted = "#57606a";

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
          <Text onPress={() => list.scrollToIndex({ index: 30, viewPosition: 0.5 })} style={{ color: "#0969da" }}>
            #30
          </Text>
          <Text onPress={() => list.scrollToOffset({ offset: 0 })} style={{ color: "#0969da" }}>
            Top
          </Text>
        </View>
      }
    >
      {(post) => (
        <View style={{ padding: 16, borderRadius: 10, backgroundColor: "#ffffff" }}>
          <Text style={{ fontSize: 16 }}>{post().title}</Text>
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
          <Text style={{ fontWeight: "600" }}>{item().id}</Text>
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
    <List each={rows} stickyHeaderIndices={headers} estimatedItemSize={44} style={{ flex: 1, backgroundColor: "#ffffff" }}>
      {(row) => (
        <If
          when={"header" in row()}
          else={
            <View style={{ paddingHorizontal: 16, paddingVertical: 12, borderBottomWidth: 1, borderBottomColor: "#eaeef2" }}>
              <Text style={{ fontSize: 16 }}>{(row() as { name: string }).name}</Text>
            </View>
          }
        >
          <View style={{ backgroundColor: "#eaeef2", paddingHorizontal: 16, paddingVertical: 6 }}>
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
        backgroundColor: tab() === id ? "#ffffff" : "transparent",
      }}
    >
      {label}
    </Text>
  );
  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", paddingTop: 24 }}>
      <View style={{ flexDirection: "row", marginHorizontal: 12, padding: 3, borderRadius: 10, backgroundColor: "#e1e4e8" }}>
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
