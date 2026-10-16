import { Input, Scroll, signal, Text, View } from "nsh";
import { AccessibilityInfo, reduceMotion, screenReaderEnabled } from "nsh/accessibility";
import { LayoutAnimation } from "nsh/animated";
import { PixelRatio, Platform } from "nsh/device";
import { InputAccessoryView } from "nsh/ios";
import { Button, Card, KeyboardAvoidingView, Modal, SafeAreaView, Screen } from "nsh/ui/mobile";
import { nav } from "../nav";
import { Button as UiButton, colors, Section } from "../ui";

const hairline = 1 / PixelRatio.get();

const styles = {
  row: { flexDirection: "row", alignItems: "center", gap: 12 },
  card: { backgroundColor: "#ffffff", borderRadius: 10, padding: 12, borderWidth: hairline, borderColor: colors.border },
  label: { fontSize: 15, color: colors.text },
  muted: { fontSize: 13, color: colors.muted },
} as const;

function LayoutAnimations() {
  const items = signal([1, 2, 3]);
  let next = 4;
  const animate = (change: () => void) => {
    LayoutAnimation.configureNext(LayoutAnimation.Presets.spring);
    change();
  };
  return (
    <Section title="Layout animation">
      <View style={styles.row}>
        <UiButton label="Add" onPress={() => animate(() => items.set([next++, ...items()]))} />
        <UiButton label="Shuffle" color="#8250df" onPress={() => animate(() => items.set([...items()].sort(() => Math.random() - 0.5)))} />
        <UiButton label="Remove" color="#cf222e" onPress={() => animate(() => items.set(items().slice(1)))} />
      </View>
      <View style={{ flexDirection: "row", flexWrap: "wrap", gap: 8 }}>
        <Each of={items()}>
          {(n) => (
            <View style={{ width: 56, height: 56, borderRadius: 12, backgroundColor: "#ddf4ff", alignItems: "center", justifyContent: "center" }}>
              <Text style={{ fontWeight: "600" }}>{n()}</Text>
            </View>
          )}
        </Each>
      </View>
    </Section>
  );
}

function PressableSheet() {
  const sheet = signal(false);
  return (
    <Section title="Sheet">
      <View
        onPress={() => sheet.set(true)}
        class="rounded-[10px] bg-[#8250df] p-3.5 transition active:scale-[0.97] active:bg-[#bf3989]"
      >
        <Text style={{ color: "#ffffff", fontWeight: "600" }}>Open a sheet</Text>
      </View>
      <Modal visible={sheet()} ios={{ presentationStyle: "pageSheet" }} animationType="slide" onRequestClose={() => sheet.set(false)}>
        <SafeAreaView style={{ flex: 1, backgroundColor: colors.surface, padding: 20, gap: 16 }}>
          <Text style={{ fontSize: 24, fontWeight: "700" }}>A sheet</Text>
          <Text style={{ color: colors.muted }}>Swipe down (iOS) or press back (Android) to close.</Text>
          <UiButton label="Close" onPress={() => sheet.set(false)} />
        </SafeAreaView>
      </Modal>
    </Section>
  );
}

/** Press feedback is native: opacity, an underlay, classes; `disabled` takes no presses. */
function PressFeedback() {
  const last = signal("nothing yet");
  const disabled = signal(false);
  return (
    <Section title="Press feedback">
      <View style={styles.card} activeOpacity={0.4} onPress={() => last.set("activeOpacity")}>
        <Text style={styles.label}>activeOpacity: fades while pressed</Text>
      </View>
      <View style={styles.card} underlayColor="#dbeafe" onPress={() => last.set("underlayColor")}>
        <Text style={styles.label}>underlayColor: the background while pressed</Text>
      </View>
      <View
        style={styles.card}
        disabled={disabled()}
        underlayColor="#fee2e2"
        class="disabled:opacity-40"
        onPress={() => last.set("disabled view")}
      >
        <Text style={styles.label}>{disabled() ? "disabled: takes no presses" : "enabled"}</Text>
      </View>
      <View style={styles.row}>
        <UiButton label={disabled() ? "Enable" : "Disable"} color="#57606a" onPress={() => disabled.set(!disabled())} />
        <Text style={styles.muted}>Last pressed: {last()}</Text>
      </View>
    </Section>
  );
}

function Accessory() {
  const text = signal("");
  let input: any;
  return (
    <Section title="InputAccessoryView">
      <Input
        ref={input}
        value={text()}
        onChangeText={(v) => text.set(v)}
        placeholder="Focus me (iOS shows a bar over the keyboard)"
        ios={{ inputAccessoryViewID: "counter" }}
        style={{ borderWidth: 1, borderColor: colors.border, borderRadius: 8, paddingHorizontal: 10, paddingVertical: 8, fontSize: 16 }}
      />
      <If when={Platform.OS === "android"}>
        <Text style={styles.muted}>Android has no input accessories.</Text>
      </If>
      <InputAccessoryView nativeID="counter" style={{ backgroundColor: "#f6f8fa" }}>
        <View style={{ flexDirection: "row", alignItems: "center", paddingHorizontal: 12, paddingVertical: 8, borderTopWidth: hairline, borderTopColor: colors.border }}>
          <Text style={[styles.muted, { flex: 1 }]}>{text().length} characters</Text>
          <UiButton label="Clear" color="#57606a" onPress={() => text.set("")} />
          <View style={{ width: 8 }} />
          <UiButton label="Done" onPress={() => input?.blur()} />
        </View>
      </InputAccessoryView>
    </Section>
  );
}

function AvoidingPage() {
  const behavior = signal<"padding" | "height" | "position">("padding");
  return (
    <Screen title="KeyboardAvoidingView">
      <KeyboardAvoidingView behavior={behavior()} style={{ flex: 1, backgroundColor: "#ddf4ff" }} contentContainerStyle={{ flex: 1 }}>
        <View style={{ flex: 1, padding: 16, gap: 12 }}>
          <Text style={styles.label}>behavior="{behavior()}"</Text>
          <View style={styles.row}>
            <Each of={["padding", "height", "position"] as const}>
              {(b) => <UiButton label={b()} color={behavior() === b() ? colors.accent : "#57606a"} onPress={() => behavior.set(b())} />}
            </Each>
          </View>
          <Text style={styles.muted}>
            padding: the bottom pads (the blue background stays under the keyboard); height: the view shrinks; position: the content moves up.
          </Text>
          <View style={{ flex: 1 }} />
          <Input
            placeholder="Type here"
            style={{ backgroundColor: "#ffffff", borderRadius: 8, paddingHorizontal: 10, paddingVertical: 10, fontSize: 16 }}
          />
        </View>
      </KeyboardAvoidingView>
    </Screen>
  );
}

function ModalOptions() {
  const visible = signal(false);
  const translucent = signal(false);
  return (
    <Section title="Modal options">
      <View style={styles.row}>
        <UiButton label="Open" onPress={() => visible.set(true)} />
        <UiButton
          label={`statusBarTranslucent: ${translucent()}`}
          color="#57606a"
          onPress={() => translucent.set(!translucent())}
        />
      </View>
      <Text style={styles.muted}>
        Android: translucent content reaches under the status bar. iOS: this modal allows portrait and landscape.
      </Text>
      <Modal
        visible={visible()}
        animationType="slide"
        android={{ statusBarTranslucent: translucent() }}
        ios={{ supportedOrientations: ["portrait", "landscape"] }}
        onRequestClose={() => visible.set(false)}
      >
        <View style={{ flex: 1, backgroundColor: "#fff8c5" }}>
          <SafeAreaView style={{ flex: 1, padding: 16, gap: 12 }}>
            <Text style={{ fontSize: 22, fontWeight: "700" }}>A modal</Text>
            <Text style={styles.muted}>The yellow background starts {translucent() ? "under" : "below"} the status bar on Android.</Text>
            <UiButton label="Close" onPress={() => visible.set(false)} />
          </SafeAreaView>
        </View>
      </Modal>
    </Section>
  );
}

function PlatformInfo() {
  const rows = (): [string, string][] => [
    ["Platform.OS", Platform.OS],
    ["Platform.Version", String(Platform.Version)],
    ["isPad / isTV / isMacCatalyst", `${Platform.isPad} / ${Platform.isTV} / ${Platform.isMacCatalyst}`],
    ["Platform.select", Platform.select({ ios: "ios branch", android: "android branch" }) ?? ""],
    ["PixelRatio.get()", String(PixelRatio.get())],
    ["roundToNearestPixel(10.3)", String(PixelRatio.roundToNearestPixel(10.3))],
    ["screenReaderEnabled()", String(screenReaderEnabled())],
    ["reduceMotion()", String(reduceMotion())],
  ];
  const announced = signal(0);
  let target: any;
  return (
    <Section title="Platform, PixelRatio, AccessibilityInfo">
      <Each of={rows()}>
        {(row) => (
          <View style={[styles.row, { justifyContent: "space-between" }]}>
            <Text style={styles.muted}>{row()[0]}</Text>
            <Text style={styles.label}>{row()[1]}</Text>
          </View>
        )}
      </Each>
      <View style={{ height: hairline, backgroundColor: colors.text }} />
      <Text style={styles.muted}>The line above is one device pixel tall.</Text>
      <View style={styles.row}>
        <UiButton
          label="Announce"
          onPress={() => {
            announced.set(announced() + 1);
            AccessibilityInfo.announceForAccessibility(`Announcement ${announced()}`);
          }}
        />
        <UiButton label="Focus the note" color="#57606a" onPress={() => AccessibilityInfo.setAccessibilityFocus(target)} />
      </View>
      <Text ref={target} style={styles.muted}>
        With VoiceOver or TalkBack on, the buttons above speak and move the reader here.
      </Text>
    </Section>
  );
}

/** The components of nsh/ui/mobile (Rust, `nsh-ui-mobile`). */
function MobileComponents() {
  const presses = signal(0);
  return (
    <Section title="nsh/ui/mobile">
      <Button title={`Primary (${presses()})`} onPress={() => presses.set(presses() + 1)} />
      <Button title="Secondary" variant="secondary" onPress={() => presses.set(presses() + 1)} />
      <Button title="Danger" variant="danger" onPress={() => presses.set(presses() + 1)} />
      <Button title="Plain" variant="plain" onPress={() => presses.set(presses() + 1)} />
      <Button title="Disabled" disabled onPress={() => presses.set(presses() + 1)} />
      <Card
        header={<Text>Card with slots</Text>}
        footer={<Button title="Reset" variant="plain" onPress={() => presses.set(0)} />}
      >
        <Text>The header, this body and the footer are JSX in a Rust component's slots and children.</Text>
      </Card>
      <Card variant="outlined">
        <Text>An outlined card, children only.</Text>
      </Card>
    </Section>
  );
}

export function Components() {
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }} keyboardDismissMode="interactive">
      <View style={{ padding: 16, gap: 20 }}>
        <MobileComponents />
        <PressFeedback />
        <PressableSheet />
        <LayoutAnimations />
        <Accessory />
        <Section title="KeyboardAvoidingView">
          <UiButton label="padding / height / position" onPress={() => nav.push({ name: "page", page: AvoidingPage })} />
        </Section>
        <ModalOptions />
        <PlatformInfo />
      </View>
    </Scroll>
  );
}
