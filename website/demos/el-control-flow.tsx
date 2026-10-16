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
  const button = { borderRadius: 8, paddingHorizontal: 12, paddingVertical: 8 } as const;

  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", padding: 20, paddingTop: 24, gap: 12 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#1f2328" }}>if / each</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View onPress={add} activeOpacity={0.6} style={{ ...button, backgroundColor: "#1f883d" }}>
          <Text style={{ color: "#ffffff" }}>Add</Text>
        </View>
        <View onPress={() => showDone.set(!showDone())} activeOpacity={0.6} style={{ ...button, backgroundColor: "#ddf4ff" }}>
          <Text style={{ color: "#0969da" }}>{showDone() ? "Hide done" : "Show done"}</Text>
        </View>
      </View>

      <If
        when={visible().length > 0}
        else={<Text style={{ color: "#6e7781", textAlign: "center", marginTop: 40 }}>Nothing to show.</Text>}
      >
        <View style={{ gap: 8 }}>
          <Each of={visible()} key={(t: Todo) => t.id}>
            {(todo, index) => (
              <View
                style={{
                  flexDirection: "row",
                  alignItems: "center",
                  gap: 12,
                  backgroundColor: "#ffffff",
                  borderRadius: 10,
                  padding: 12,
                  borderWidth: 1,
                  borderColor: "#d0d7de",
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
                    borderColor: "#1f883d",
                    backgroundColor: todo().done ? "#1f883d" : "transparent",
                  }}
                />
                <Text
                  style={{
                    flex: 1,
                    color: todo().done ? "#8c959f" : "#1f2328",
                    textDecorationLine: todo().done ? "line-through" : "none",
                  }}
                >
                  {index() + 1}. {todo().title}
                </Text>
                <If when={todo().done}>
                  <View onPress={() => remove(todo().id)} hitSlop={8}>
                    <Text style={{ color: "#cf222e" }}>Remove</Text>
                  </View>
                </If>
              </View>
            )}
          </Each>
        </View>
      </If>
      <Text style={{ fontSize: 12, color: "#6e7781" }}>
        {todos().filter((t) => t.done).length} of {todos().length} done
      </Text>
    </View>
  );
}

mount(App);
