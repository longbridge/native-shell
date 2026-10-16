import { Input, mount, signal, Text, View, type YNode } from "nsh";

function App() {
  const name = signal("");
  const password = signal("");
  const notes = signal("");
  const submitted = signal("");
  let passwordInput!: YNode;
  const field = {
    fontSize: 16,
    color: "#1f2328",
    backgroundColor: "#ffffff",
    borderWidth: 1,
    borderColor: "#d0d7de",
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
  } as const;
  return (
    <View style={{ flex: 1, backgroundColor: "#f6f8fa", padding: 20, paddingTop: 24, gap: 10 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#1f2328" }}>input</Text>

      <Text style={{ color: "#57606a" }}>Name (controlled, maxLength 20)</Text>
      <Input
        style={field}
        value={name()}
        onChangeText={(text) => name.set(text)}
        placeholder="Ada Lovelace"
        placeholderTextColor="#8c959f"
        autoCapitalize="words"
        maxLength={20}
        returnKeyType="next"
        onSubmitEditing={() => passwordInput.focus()}
      />
      <Text style={{ fontSize: 12, color: "#6e7781" }}>{name().length} / 20</Text>

      <Text style={{ color: "#57606a" }}>Password</Text>
      <Input
        ref={passwordInput}
        style={field}
        value={password()}
        onChangeText={(text) => password.set(text)}
        secureTextEntry
        autoComplete="current-password"
        returnKeyType="done"
        onSubmitEditing={() => submitted.set(`Signed in as ${name() || "nobody"}`)}
      />

      <Text style={{ color: "#57606a" }}>Notes (multiline)</Text>
      <Input
        style={{ ...field, height: 100 }}
        multiline
        value={notes()}
        onChangeText={(text) => notes.set(text)}
        placeholder="Anything else?"
        placeholderTextColor="#8c959f"
      />

      <View
        onPress={() => submitted.set(`Signed in as ${name() || "nobody"}`)}
        activeOpacity={0.7}
        disabled={password().length === 0}
        class="disabled:opacity-40"
        style={{ backgroundColor: "#1f883d", borderRadius: 10, padding: 14, alignItems: "center", marginTop: 6 }}
      >
        <Text style={{ color: "#ffffff", fontSize: 16, fontWeight: "600" }}>Sign in</Text>
      </View>
      <Text style={{ color: "#1a7f37" }}>{submitted()}</Text>
    </View>
  );
}

mount(App);
