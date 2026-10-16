import { Input, mount, signal, Text, View, type YNode } from "nsh";

function App() {
  const name = signal("");
  const password = signal("");
  const notes = signal("");
  const submitted = signal("");
  let passwordInput!: YNode;
  const field = {
    fontSize: 16,
    color: "#eceef1",
    backgroundColor: "#0c0d0f",
    borderWidth: 1,
    borderColor: "#1f2124",
    borderRadius: 12,
    boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)",
    paddingHorizontal: 12,
    paddingVertical: 10,
  } as const;
  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 20, paddingTop: 24, gap: 10 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: "#eceef1" }}>input</Text>

      <Text style={{ color: "#8d9198" }}>Name (controlled, maxLength 20)</Text>
      <Input
        style={field}
        value={name()}
        onChangeText={(text) => name.set(text)}
        placeholder="Ada Lovelace"
        placeholderTextColor="#5a5e65"
        autoCapitalize="words"
        maxLength={20}
        returnKeyType="next"
        onSubmitEditing={() => passwordInput.focus()}
      />
      <Text style={{ fontSize: 12, color: "#8d9198" }}>{name().length} / 20</Text>

      <Text style={{ color: "#8d9198" }}>Password</Text>
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

      <Text style={{ color: "#8d9198" }}>Notes (multiline)</Text>
      <Input
        style={{ ...field, height: 100 }}
        multiline
        value={notes()}
        onChangeText={(text) => notes.set(text)}
        placeholder="Anything else?"
        placeholderTextColor="#5a5e65"
      />

      <View
        onPress={() => submitted.set(`Signed in as ${name() || "nobody"}`)}
        activeOpacity={0.7}
        disabled={password().length === 0}
        class="disabled:opacity-40"
        style={{
          backgroundImage: "linear-gradient(180deg, #fbfbfc, #d4d6d9)",
          boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
          borderRadius: 10,
          padding: 14,
          alignItems: "center",
          marginTop: 6,
        }}
      >
        <Text style={{ color: "#111214", fontSize: 16, fontWeight: "600" }}>Sign in</Text>
      </View>
      <Text style={{ color: "#45d49a" }}>{submitted()}</Text>
    </View>
  );
}

mount(App);
