// TextField: labels, placeholders, controlled and uncontrolled text, secure entry.
import { mount, signal, Text, View } from "nsh";
import { Button, TextField } from "nsh/ui/mobile";
import { ui } from "./palette";

function App() {
  const name = signal("Ada");
  const submitted = signal("");
  const locked = signal(false);
  return (
    <View style={{ flex: 1, backgroundColor: ui().page, padding: 16, paddingTop: 24, gap: 16 }}>
      <Text style={{ fontSize: 22, fontWeight: "700", color: ui().text }}>TextField</Text>
      <TextField
        label="Name (controlled)"
        placeholder="Your name"
        value={name()}
        onChange={(v) => name.set(v)}
        onSubmit={(v) => submitted.set(v)}
        disabled={locked()}
      />
      <Text style={{ color: ui().muted }}>{`Hello, ${name() || "nobody"}`}</Text>
      <TextField label="Password" placeholder="At least 8 characters" secure />
      <TextField label="Notes (no clear button)" defaultValue="Uncontrolled" clearable={false} />
      <Button
        variant="secondary"
        title={locked() ? "Enable the name" : "Disable the name"}
        onPress={() => locked.set(!locked())}
      />
      <Text style={{ color: ui().muted }}>{submitted() ? `Submitted: ${submitted()}` : "Press return to submit."}</Text>
    </View>
  );
}

mount(App);
