import { Input, Scroll, signal, Text, View, type YNode } from "nsh";
import { List } from "nsh/list";
import { keyboard, KeyboardAvoidingView, useSafeAreaInsets } from "nsh/ui/mobile";
import { colors } from "../ui";

const tints = ["#fde2e4", "#e2ece9", "#dfe7fd", "#fff1e6", "#e8e8e4", "#cddafd"];

/** Three columns, a header and a footer, padding and gaps, viewability. */
function Grid() {
  const items = Array.from({ length: 90 }, (_, i) => ({ id: i, height: 70 + (i % 4) * 20 }));
  const visible = signal("");
  let list!: YNode;
  return (
    <List
      ref={list}
      each={items}
      numColumns={3}
      estimatedItemSize={90}
      style={{ flex: 1 }}
      contentContainerStyle={{ padding: 12, gap: 8 }}
      viewabilityConfig={{ itemVisiblePercentThreshold: 50 }}
      onViewableItemsChanged={({ viewableItems }) =>
        visible.set(viewableItems.length ? `${viewableItems[0].index}–${viewableItems[viewableItems.length - 1].index}` : "")
      }
      ListHeaderComponent={
        <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingBottom: 4 }}>
          <Text style={{ flex: 1, color: colors.muted }}>Visible {visible()}</Text>
          <Text onPress={() => list.scrollToIndex({ index: 60, viewPosition: 0.5 })} style={{ color: colors.accent }}>
            Item 60
          </Text>
          <Text onPress={() => list.scrollToEnd()} style={{ color: colors.accent }}>
            End
          </Text>
        </View>
      }
      ListFooterComponent={<Text style={{ textAlign: "center", color: colors.muted, padding: 12 }}>— the end —</Text>}
    >
      {(item) => (
        <View
          style={{
            height: item().height,
            borderRadius: 10,
            backgroundColor: tints[item().id % tints.length],
            alignItems: "center",
            justifyContent: "center",
          }}
        >
          <Text style={{ fontSize: 17, fontWeight: "600" }}>{item().id}</Text>
        </View>
      )}
    </List>
  );
}

/** Section headers stick to the top until the next one pushes them out. */
function Contacts() {
  const names = ["Ada", "Alan", "Barbara", "Brian", "Claude", "Dennis", "Donald", "Edsger", "Frances", "Grace", "Guido", "Ken", "Linus", "Margaret", "Niklaus"];
  // One list of header and name rows; the headers' indices stick.
  type Row = { header: string } | { name: string; last: boolean };
  const rows: Row[] = [];
  const headers: number[] = [];
  for (const letter of [...new Set(names.map((n) => n[0]))]) {
    headers.push(rows.length);
    rows.push({ header: letter });
    const group = names.filter((n) => n[0] === letter);
    group.forEach((name, i) => rows.push({ name, last: i === group.length - 1 }));
  }
  return (
    <List each={rows} stickyHeaderIndices={headers} estimatedItemSize={44} style={{ flex: 1, backgroundColor: colors.surface }}>
      {(row) => {
        const header = () => row() as { header: string };
        const contact = () => row() as { name: string; last: boolean };
        return (
          <If
            when={"header" in row()}
            else={
              <View
                style={{
                  paddingHorizontal: 16,
                  paddingVertical: 12,
                  borderBottomWidth: contact().last ? 0 : 1,
                  borderBottomColor: "#eaeef2",
                }}
              >
                <Text style={{ fontSize: 17 }}>{contact().name}</Text>
              </View>
            }
          >
            <View style={{ backgroundColor: "#eaeef2", paddingHorizontal: 16, paddingVertical: 6 }}>
              <Text style={{ fontWeight: "700", color: colors.muted }}>{header().header}</Text>
            </View>
          </If>
        );
      }}
    </List>
  );
}

type Message = { id: number; text: string; mine: boolean };

/** An inverted list: starts at the newest message, keeps its place as older ones load. */
function Chat() {
  const insets = useSafeAreaInsets();
  let next = 0;
  const message = (text: string, mine: boolean): Message => ({ id: next++, text, mine });
  const messages = signal(
    Array.from({ length: 30 }, (_, i) => message(`Message ${30 - i}`, i % 3 === 0)),
  );
  const draft = signal("");
  const send = () => {
    if (!draft().trim()) return;
    messages.update((m) => [message(draft(), true), ...m]);
    draft.set("");
  };
  const loadOlder = () => {
    const count = messages().length;
    messages.update((m) => [...m, ...Array.from({ length: 20 }, (_, i) => message(`Older ${count + i + 1}`, i % 4 === 0))]);
  };
  return (
    <KeyboardAvoidingView style={{ flex: 1 }}>
      <List
        each={messages()}
        inverted
        estimatedItemSize={44}
        style={{ flex: 1 }}
        contentContainerStyle={{ padding: 12, gap: 6 }}
        keyboardDismissMode="interactive"
        onEndReached={loadOlder}
        onEndReachedThreshold={0.5}
      >
        {(m) => (
          <View
            style={{
              alignSelf: m().mine ? "flex-end" : "flex-start",
              maxWidth: "75%",
              paddingHorizontal: 12,
              paddingVertical: 8,
              borderRadius: 14,
              backgroundColor: m().mine ? colors.accent : colors.surface,
            }}
          >
            <Text style={{ color: m().mine ? "#ffffff" : colors.text, fontSize: 16 }}>{m().text}</Text>
          </View>
        )}
      </List>
      <View
        style={{
          flexDirection: "row",
          gap: 8,
          padding: 8,
          paddingBottom: 8 + (keyboard().height > 0 ? 0 : insets().bottom),
          backgroundColor: colors.surface,
        }}
      >
        <Input
          value={draft()}
          onChangeText={(v) => draft.set(v)}
          onSubmitEditing={send}
          blurOnSubmit={false}
          placeholder="Message"
          returnKeyType="send"
          style={{ flex: 1, fontSize: 16, borderWidth: 1, borderColor: colors.border, borderRadius: 18, paddingHorizontal: 12, paddingVertical: 8 }}
        />
        <Text onPress={send} style={{ alignSelf: "center", color: colors.accent, fontSize: 16, fontWeight: "600" }}>
          Send
        </Text>
      </View>
    </KeyboardAvoidingView>
  );
}

/** Horizontal scrolling, pull to refresh on a plain scroll view, and an empty list. */
function More() {
  const refreshing = signal(false);
  const refreshed = signal(0);
  const items = signal<number[]>([]);
  const refresh = () => {
    refreshing.set(true);
    setTimeout(() => {
      refreshed.set(refreshed() + 1);
      refreshing.set(false);
    }, 1000);
  };
  const cards = Array.from({ length: 1000 }, (_, i) => i);
  return (
    <Scroll style={{ flex: 1 }} refreshing={refreshing()} onRefresh={refresh}>
      <View style={{ paddingVertical: 12, gap: 16 }}>
        <Text style={{ marginHorizontal: 12, color: colors.muted }}>Pull down to refresh (refreshed {refreshed()} times)</Text>
        <Text style={{ marginHorizontal: 12, fontWeight: "600" }}>A horizontal scroll view</Text>
        <Scroll horizontal showsIndicator={false}>
          <View style={{ flexDirection: "row", gap: 8, paddingHorizontal: 12 }}>
            <Each of={["Today", "Yesterday", "This week", "This month", "This year", "All time", "Custom"]}>
              {(label) => (
                <Text style={{ paddingHorizontal: 14, paddingVertical: 8, borderRadius: 16, overflow: "hidden", backgroundColor: colors.surface }}>
                  {label()}
                </Text>
              )}
            </Each>
          </View>
        </Scroll>
        <Text style={{ marginHorizontal: 12, fontWeight: "600" }}>A horizontal list of 1000 recycled cards</Text>
        <List each={cards} horizontal estimatedItemSize={130} style={{ height: 120 }} contentContainerStyle={{ paddingHorizontal: 12, gap: 10 }}>
          {(i) => (
            <View style={{ width: 120, height: 120, borderRadius: 12, backgroundColor: tints[i() % tints.length], alignItems: "center", justifyContent: "center" }}>
              <Text style={{ fontSize: 20, fontWeight: "700" }}>{i()}</Text>
            </View>
          )}
        </List>
        <View style={{ flexDirection: "row", alignItems: "center", marginHorizontal: 12 }}>
          <Text style={{ flex: 1, fontWeight: "600" }}>An empty list ({items().length} items)</Text>
          <Text onPress={() => items.set([...items(), items().length + 1])} style={{ color: colors.accent, marginRight: 16 }}>
            Add
          </Text>
          <Text onPress={() => items.set([])} style={{ color: colors.accent }}>
            Clear
          </Text>
        </View>
        <List
          each={items()}
          estimatedItemSize={44}
          style={{ height: 180, marginHorizontal: 12, borderRadius: 10, backgroundColor: colors.surface }}
          ListEmptyComponent={<Text style={{ padding: 24, textAlign: "center", color: colors.muted }}>Nothing here yet: tap Add</Text>}
          ItemSeparatorComponent={() => <View style={{ height: 1, backgroundColor: "#eaeef2" }} />}
        >
          {(n) => <Text style={{ padding: 12 }}>Item {n()}</Text>}
        </List>
      </View>
    </Scroll>
  );
}

type Tab = "grid" | "sections" | "chat" | "more";

export function Lists() {
  const tab = signal<Tab>("grid");
  const tabButton = (id: Tab, label: string) => (
    <Text
      onPress={() => tab.set(id)}
      style={{
        flex: 1,
        textAlign: "center",
        paddingVertical: 6,
        borderRadius: 8,
        overflow: "hidden",
        backgroundColor: tab() === id ? colors.surface : "transparent",
        fontWeight: "600",
      }}
    >
      {label}
    </Text>
  );
  return (
    <View style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ flexDirection: "row", margin: 12, padding: 3, borderRadius: 10, backgroundColor: "#e1e4e8" }}>
        {tabButton("grid", "Grid")}
        {tabButton("sections", "Sections")}
        {tabButton("chat", "Chat")}
        {tabButton("more", "More")}
      </View>
      {(() => {
        switch (tab()) {
          case "grid":
            return <Grid />;
          case "sections":
            return <Contacts />;
          case "chat":
            return <Chat />;
          case "more":
            return <More />;
        }
      })()}
    </View>
  );
}
