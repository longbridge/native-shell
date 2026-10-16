import { Input, type InputProps, Scroll, signal, split, Text, View, type YNode } from "nsh";
import { Keyboard, keyboard, KeyboardAvoidingView, Modal, SafeAreaView, Screen, useSafeAreaInsets } from "nsh/ui/mobile";
import { nav } from "../nav";
import { Button, colors, Section } from "../ui";

const field = {
  fontSize: 16,
  color: colors.text,
  backgroundColor: colors.surface,
  borderWidth: 1,
  borderColor: colors.border,
  borderRadius: 10,
  paddingHorizontal: 12,
  paddingVertical: 10,
} as const;

/** A labelled input; `next` focuses another input on return. */
function Field(props: InputProps & { label: string; next?: () => YNode | undefined }) {
  const [own, input] = split(props, ["label", "next"]);
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 13, color: colors.muted }}>{own.label}</Text>
      <Input
        style={field}
        returnKeyType={own.next ? "next" : "done"}
        onSubmitEditing={() => own.next?.()?.focus()}
        {...input}
      />
    </View>
  );
}

type Mode = "none" | "on-drag" | "interactive";

/**
 * Many inputs in a scroll view: the focused one stays above the keyboard
 * (natively), return moves to the next one, dragging the form can hide the
 * keyboard.
 */
export function Form() {
  const name = signal("");
  const email = signal("");
  const shout = signal("");
  const notes = signal("");
  const mode = signal<Mode>("on-drag");
  const submitted = signal("");
  const sheet = signal(false);
  // Inputs in return-key order.
  const refs: YNode[] = [];
  const next = (i: number) => () => refs[i + 1];
  const ref = (i: number) => (node: YNode) => (refs[i] = node);

  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }} keyboardDismissMode={mode()}>
      <View style={{ padding: 16, gap: 20 }}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
          <Text style={{ flex: 1, color: colors.muted }}>Keyboard {Math.round(keyboard().height)} pt</Text>
          <Each of={["none", "on-drag", "interactive"] as Mode[]}>
            {(m) => (
              <Text
                onPress={() => mode.set(m())}
                style={{
                  fontSize: 13,
                  paddingHorizontal: 8,
                  paddingVertical: 4,
                  borderRadius: 6,
                  overflow: "hidden",
                  color: mode() === m() ? "#ffffff" : colors.text,
                  backgroundColor: mode() === m() ? colors.accent : "#e1e4e8",
                }}
              >
                {m()}
              </Text>
            )}
          </Each>
        </View>

        <Section title="Account">
          <Field ref={ref(0)} next={next(0)} label="Name" value={name()} onChangeText={(v) => name.set(v)} autoCapitalize="words" autoComplete="name" placeholder="Ada Lovelace" />
          <Field
            ref={ref(1)}
            next={next(1)}
            label="E-mail"
            value={email()}
            onChangeText={(v) => email.set(v)}
            keyboardType="email-address"
            autoCapitalize="none"
            autoCorrect={false}
            autoComplete="email"
            placeholder="ada@example.com"
          />
          <Field ref={ref(2)} next={next(2)} label="Password" secureTextEntry autoComplete="new-password" placeholder="At least 8 characters" />
          <Field ref={ref(3)} next={next(3)} label="Phone" keyboardType="phone-pad" autoComplete="tel" placeholder="+1 555 0100" />
          <Field ref={ref(4)} next={next(4)} label="Website" keyboardType="url" autoCapitalize="none" autoCorrect={false} placeholder="https://" />
        </Section>

        <Section title="Numbers">
          <Field ref={ref(5)} next={next(5)} label="Age (number pad, 3 digits)" keyboardType="number-pad" maxLength={3} />
          <Field ref={ref(6)} next={next(6)} label="Amount (decimal pad)" keyboardType="decimal-pad" placeholder="0.00" />
          <Field
            ref={ref(7)}
            next={next(7)}
            label="Code (6 digits)"
            keyboardType="numeric"
            maxLength={6}
            style={{ ...field, letterSpacing: 8, fontSize: 22, textAlign: "center" }}
          />
        </Section>

        <Section title="Address (far down: the form scrolls to keep the input visible)">
          <Field ref={ref(8)} next={next(8)} label="Street" placeholder="12 Analytical Way" />
          <Field ref={ref(9)} next={next(9)} label="City" placeholder="London" />
          <Field ref={ref(10)} next={next(10)} label="Postal code" autoCapitalize="characters" placeholder="SW1A 1AA" />
          <Field ref={ref(11)} next={next(11)} label="Country" placeholder="United Kingdom" />
        </Section>

        <Section title="Selection and keys">
          <SelectionDemo />
        </Section>

        <Section title="Text">
          <Field
            ref={ref(12)}
            next={next(12)}
            label="Controlled: uppercased as you type"
            value={shout()}
            onChangeText={(t) => shout.set(t.toUpperCase())}
          />
          <Field label="Read only" value="Not editable" editable={false} style={{ ...field, color: colors.muted, backgroundColor: "#f6f8fa" }} />
          <Field
            ref={ref(13)}
            next={next(13)}
            label="Custom colors"
            placeholder="Placeholder in purple"
            placeholderTextColor="#8250df"
            selectionColor="#bf3989"
          />
          <View style={{ gap: 4 }}>
            <Text style={{ fontSize: 13, color: colors.muted }}>Notes (multiline, grows, 280 max)</Text>
            <Input
              ref={ref(14)}
              style={{ ...field, minHeight: 100 }}
              multiline
              maxLength={280}
              value={notes()}
              onChangeText={(v) => notes.set(v)}
              placeholder="Return adds a line"
            />
            <Text style={{ fontSize: 12, color: colors.muted, textAlign: "right" }}>{`${notes().length}/280`}</Text>
          </View>
          <AutoGrow />
          <View style={{ gap: 4 }}>
            <Text style={{ fontSize: 13, color: colors.muted }}>caretHidden</Text>
            <Input style={field} caretHidden defaultValue="No caret here" />
          </View>
        </Section>

        <View style={{ gap: 10 }}>
          <Button label="Submit" onPress={() => submitted.set(`${name() || "Nobody"} <${email() || "no e-mail"}>`)} />
          <Text style={{ color: "#1a7f37" }}>{submitted()}</Text>
          <Button label="Hide the keyboard" color="#57606a" onPress={() => Keyboard.dismiss()} />
          <Button label="Inputs in a sheet" color="#8250df" onPress={() => sheet.set(true)} />
          <Button
            label="A message bar at the bottom"
            color="#1a7f37"
            onPress={() => nav.push({ name: "page", page: MessageBar })}
          />
        </View>
      </View>

      <Modal visible={sheet()} ios={{ presentationStyle: "pageSheet" }} onRequestClose={() => sheet.set(false)}>
        <SafeAreaView style={{ flex: 1, backgroundColor: colors.background }}>
          <Scroll style={{ flex: 1 }} keyboardDismissMode="interactive">
            <View style={{ padding: 16, gap: 12 }}>
              <Text style={{ fontSize: 22, fontWeight: "700" }}>A sheet</Text>
              <Button label="Close" onPress={() => sheet.set(false)} />
              <Each of={["First", "Second", "Third", "Fourth", "Fifth", "Sixth", "Seventh"]}>
                {(label) => <Field label={label()} placeholder={`${label()} field`} />}
              </Each>
            </View>
          </Scroll>
        </SafeAreaView>
      </Modal>
    </Scroll>
  );
}

/** An input pinned to the bottom (outside any scroll view): it rides on the keyboard. */
/** A multiline input that grows with its text (onContentSizeChange), up to 5 lines. */
function AutoGrow() {
  const height = signal(0);
  const size = signal("");
  return (
    <View style={{ gap: 4 }}>
      <Text style={{ fontSize: 13, color: colors.muted }}>Grows with its text (onContentSizeChange): {size()}</Text>
      <Input
        multiline
        placeholder="Type several lines"
        onContentSizeChange={(e) => {
          size.set(`${Math.round(e.width)}×${Math.round(e.height)}`);
          height.set(Math.min(e.height, 110));
        }}
        style={{ ...field, height: Math.max(height(), 20) + 20 }}
      />
    </View>
  );
}

function MessageBar() {
  const insets = useSafeAreaInsets();
  const messages = signal<string[]>([]);
  const draft = signal("");
  const send = () => {
    if (!draft().trim()) return;
    messages.update((m) => [...m, draft()]);
    draft.set("");
  };
  return (
    <Screen title="Message bar">
    <KeyboardAvoidingView style={{ flex: 1, backgroundColor: colors.background }}>
      <Scroll style={{ flex: 1 }} keyboardDismissMode="interactive">
        <View style={{ padding: 16, gap: 8 }}>
          <Text style={{ color: colors.muted }}>
            The bar below sits outside the scroll view; KeyboardAvoidingView lifts it with the keyboard.
          </Text>
          <Each of={messages()}>
            {(m) => (
              <Text
                style={{
                  alignSelf: "flex-end",
                  color: "#ffffff",
                  backgroundColor: colors.accent,
                  borderRadius: 14,
                  overflow: "hidden",
                  paddingHorizontal: 12,
                  paddingVertical: 8,
                }}
              >
                {m()}
              </Text>
            )}
          </Each>
        </View>
      </Scroll>
      <View
        style={{
          flexDirection: "row",
          gap: 8,
          padding: 8,
          // Above the home indicator, unless the keyboard covers it.
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
          style={{ ...field, flex: 1, borderRadius: 18, paddingVertical: 8 }}
        />
        <Text onPress={send} style={{ alignSelf: "center", color: colors.accent, fontSize: 16, fontWeight: "600" }}>
          Send
        </Text>
      </View>
    </KeyboardAvoidingView>
    </Screen>
  );
}

/** `onSelectionChange`, `onKeyPress`, `selection` and `(v) => selection.set(v)`. */
function SelectionDemo() {
  const text = signal("Select part of this text");
  const range = signal({ start: 0, end: 0 });
  const keys = signal<string[]>([]);
  const selection = signal<{ start: number; end?: number } | undefined>(undefined);
  let input: YNode | undefined;
  return (
    <View style={{ gap: 8 }}>
      <Input
        ref={(node) => (input = node)}
        value={text()}
        onChangeText={(v) => text.set(v)}
        selection={selection()}
        onSelectionChange={(v) => range.set(v)}
        onKeyPress={(e) => keys.update((k) => [...k, e.key].slice(-8))}
        autoComplete="off"
        style={field}
      />
      <Text style={{ fontSize: 13, color: colors.muted }}>
        {`Selection ${range().start}–${range().end}: "${text().slice(range().start, range().end)}"`}
      </Text>
      <Text style={{ fontSize: 13, color: colors.muted }}>{`Keys: ${keys().join(" ") || "–"}`}</Text>
      <View style={{ flexDirection: "row", gap: 8 }}>
        <View style={{ flex: 1 }}>
          <Button label="Select all" onPress={() => selection.set({ start: 0, end: text().length })} />
        </View>
        <View style={{ flex: 1 }}>
          <Button label="Caret to start" color="#57606a" onPress={() => input?.setSelection(0)} />
        </View>
      </View>
      <Field label="One-time code (autofill from messages)" keyboardType="number-pad" autoComplete="one-time-code" maxLength={6} />
    </View>
  );
}
