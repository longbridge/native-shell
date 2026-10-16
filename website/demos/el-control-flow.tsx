import { mount, signal, Text, View } from "nsh";

interface Todo {
  id: number;
  title: string;
  done: boolean;
}

let nextId = 4;

function App() {
  const todos = signal<Todo[]>([
    { id: 1, title: "Read the elements page", done: true },
    { id: 2, title: "Try <If> and <Each>", done: false },
    { id: 3, title: "Ship the app", done: false },
  ]);
  const showDone = signal(true);
  const visible = () => (showDone() ? todos() : todos().filter((t) => !t.done));
  const toggle = (id: number) => todos.update((list) => list.map((t) => (t.id === id ? { ...t, done: !t.done } : t)));
  const add = () => todos.update((list) => [...list, { id: nextId++, title: `Task ${nextId - 1}`, done: false }]);
  const remove = (id: number) => todos.update((list) => list.filter((t) => t.id !== id));
  const button = { borderRadius: 10, paddingHorizontal: 12, paddingVertical: 8 } as const;

  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 20, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1" }}>if / each</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View
          onPress={add}
          activeOpacity={0.6}
          style={{
            ...button,
            backgroundImage: "linear-gradient(180deg, #dbf893, #b3dd4e)",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
          }}
        >
          <Text style={{ color: "#13160c", fontWeight: "600" }}>Add</Text>
        </View>
        <View
          onPress={() => showDone.set(!showDone())}
          activeOpacity={0.6}
          style={{
            ...button,
            backgroundColor: "#24262a",
            backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
            borderWidth: 1,
            borderColor: "#2c2e33",
            boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
          }}
        >
          <Text style={{ color: "#eceef1" }}>{showDone() ? "Hide done" : "Show done"}</Text>
        </View>
      </View>

      <If
        when={visible().length > 0}
        else={<Text style={{ color: "#8d9198", textAlign: "center", marginTop: 40 }}>Nothing to show.</Text>}
      >
        <View style={{ gap: 8 }}>
          <Each of={visible()} key={(t: Todo) => t.id}>
            {(todo, index) => (
              <View
                style={{
                  flexDirection: "row",
                  alignItems: "center",
                  gap: 12,
                  backgroundColor: "#18191c",
                  backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)",
                  borderRadius: 12,
                  padding: 12,
                  borderWidth: 1,
                  borderColor: "#2c2e33",
                  boxShadow: "inset 0 1px 0 rgba(255,255,255,0.05)",
                }}
              >
                <View
                  onPress={() => toggle(todo().id)}
                  hitSlop={8}
                  style={{
                    width: 22,
                    height: 22,
                    borderRadius: 11,
                    borderWidth: 2,
                    borderColor: "#c8f26d",
                    backgroundColor: todo().done ? "#c8f26d" : "transparent",
                  }}
                />
                <Text
                  style={{
                    flex: 1,
                    color: todo().done ? "#5a5e65" : "#eceef1",
                    textDecorationLine: todo().done ? "line-through" : "none",
                  }}
                >
                  {index() + 1}. {todo().title}
                </Text>
                <If when={todo().done}>
                  <View onPress={() => remove(todo().id)} hitSlop={8}>
                    <Text style={{ color: "#ff6170" }}>Remove</Text>
                  </View>
                </If>
              </View>
            )}
          </Each>
        </View>
      </If>
      <Text style={{ fontSize: 12, color: "#8d9198", fontVariant: ["tabular-nums"] }}>
        {todos().filter((t) => t.done).length} of {todos().length} done
      </Text>
    </View>
  );
}

mount(App);
