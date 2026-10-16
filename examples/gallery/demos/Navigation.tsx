import { cleanup, Input, type JSX, Scroll, signal, Text, View } from "nsh";
import { BackHandler } from "nsh/android";
import { Alert, Modal, SafeAreaView, Screen, useSafeAreaInsets } from "nsh/ui/mobile";
import { nav } from "../nav";
import { Button, colors, Section } from "../ui";

/** The safe-area insets where it renders. */
function Insets() {
  const insets = useSafeAreaInsets();
  return (
    <Text style={{ color: colors.muted, fontSize: 13 }}>
      {`insets: top ${Math.round(insets().top)}, bottom ${Math.round(insets().bottom)}, left ${Math.round(insets().left)}, right ${Math.round(insets().right)}`}
    </Text>
  );
}

function Filler(props: { count: number }) {
  return (
    <Each of={Array.from({ length: props.count }, (_, i) => i + 1)}>
      {(i) => (
        <View style={{ padding: 14, backgroundColor: colors.surface, borderRadius: 10 }}>
          <Text>Row {i()}</Text>
        </View>
      )}
    </Each>
  );
}

function Page(props: { children: JSX.Element }) {
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 10 }}>
        <Insets />
        {props.children}
      </View>
    </Scroll>
  );
}

function LargeTitle() {
  return (
    <Screen title="Large title" ios={{ headerLargeTitle: true }}>
      <Page>
        <Text style={{ color: colors.muted }}>iOS: the title shrinks into the bar as the content scrolls.</Text>
        <Filler count={20} />
      </Page>
    </Screen>
  );
}

function Transparent() {
  return (
    <Screen title="Transparent" headerTransparent headerTintColor="#ffffff" headerTitleColor="#ffffff">
      <View style={{ flex: 1, backgroundColor: "#8250df", padding: 24, justifyContent: "center", gap: 12 }}>
        <Text style={{ color: "#ffffff", fontSize: 22, fontWeight: "700" }}>Content under the header</Text>
        <Insets />
      </View>
    </Screen>
  );
}

function CustomHeader() {
  const likes = signal(0);
  return (
    <Screen
      title="Custom"
      headerTintColor="#ffffff"
      headerBackgroundColor="#1a7f37"
      headerTitleColor="#ffffff"
      headerBackVisible={false}
      headerLeft={
        <Text onPress={() => nav.pop()} style={{ color: "#ffffff", fontSize: 17 }}>
          Close
        </Text>
      }
      headerTitle={
        <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
          <View style={{ width: 12, height: 12, borderRadius: 6, backgroundColor: "#ffffff" }} />
          <Text style={{ color: "#ffffff", fontSize: 17, fontWeight: "700" }}>Custom title</Text>
        </View>
      }
      headerRight={
        <Text onPress={() => likes.set(likes() + 1)} style={{ color: "#ffffff", fontSize: 17 }}>
          ♥ {likes()}
        </Text>
      }
    >
      <Page>
        <Text>Header items are views: the ♥ button updates the header ({likes()}).</Text>
      </Page>
    </Screen>
  );
}

function Locked() {
  return (
    <Screen title="No swipe back" gestureEnabled={false}>
      <Page>
        <Text>The back gesture is off on this screen (no swipe on iOS, no predictive animation on Android); the back buttons still work.</Text>
      </Page>
    </Screen>
  );
}

/** `preventBack`: leaving with unsaved text asks first. */
function Unsaved() {
  const text = signal("");
  const confirm = async () => {
    const i = await Alert.alert("Discard your changes?", undefined, [
      { text: "Keep editing", style: "cancel" },
      { text: "Discard", style: "destructive" },
    ]);
    if (i === 1) nav.pop();
  };
  return (
    <Screen title="Unsaved changes" preventBack={text().length > 0} onBackRequest={confirm}>
      <Page>
        <Text>Type something, then go back (header button, swipe, Android back): the screen asks first.</Text>
        <Input
          value={text()}
          onChangeText={(v) => text.set(v)}
          placeholder="A draft"
          style={{ borderWidth: 1, borderColor: "#d0d7de", borderRadius: 8, padding: 10, fontSize: 16 }}
        />
        <Text style={{ color: "#57606a" }}>{text().length > 0 ? "preventBack is on" : "Nothing to lose: back pops"}</Text>
      </Page>
    </Screen>
  );
}

function Faded() {
  return (
    <Screen title="Fade" animation="fade">
      <Page>
        <Text>This screen faded in, and fades out.</Text>
      </Page>
    </Screen>
  );
}

function Instant() {
  return (
    <Screen title="No animation" animation="none">
      <Page>
        <Text>Shown without a transition.</Text>
      </Page>
    </Screen>
  );
}

function Headerless() {
  return (
    <Screen headerShown={false}>
      <SafeAreaView style={{ flex: 1, backgroundColor: "#fff8c5", padding: 16, gap: 12 }}>
        <Text style={{ fontSize: 24, fontWeight: "700" }}>No header</Text>
        <Insets />
        <Button label="Back" onPress={() => nav.pop()} />
      </SafeAreaView>
    </Screen>
  );
}

function Deep(props: { depth: number }) {
  return (
    <Screen title={`Level ${props.depth}`}>
      <Page>
        <Button label="Deeper" onPress={() => nav.push({ name: "page", page: () => <Deep depth={props.depth + 1} /> })} />
        <Button label="Back to the index" color="#57606a" onPress={() => nav.popToTop()} />
      </Page>
    </Screen>
  );
}

type Sheet = "fullScreen" | "pageSheet" | "formSheet" | "fade" | "dialog" | "dismissible" | null;

export function Navigation() {
  const sheet = signal<Sheet>(null);
  const intercept = signal(false);
  const backs = signal(0);
  const close = () => sheet.set(null);
  const back = BackHandler.addEventListener("hardwareBackPress", () => {
    if (!intercept()) return false;
    backs.set(backs() + 1);
    return true;
  });
  cleanup(() => back.remove());
  const push = (page: () => JSX.Element) => () => nav.push({ name: "page", page });

  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <Section title="Screens">
          <Insets />
          <Button label="Large title" onPress={push(LargeTitle)} />
          <Button label="Transparent header" onPress={push(Transparent)} />
          <Button label="Custom header items and colors" onPress={push(CustomHeader)} />
          <Button label="No header" onPress={push(Headerless)} />
          <Button label="Fade in" onPress={push(Faded)} />
          <Button label="No animation" onPress={push(Instant)} />
          <Button label="No swipe back" onPress={push(Locked)} />
          <Button label="Unsaved changes (preventBack)" onPress={push(Unsaved)} />
          <Button label="A deep stack" onPress={push(() => <Deep depth={1} />)} />
        </Section>
        <Section title="Modals">
          <Button label="Full screen (slide)" color="#8250df" onPress={() => sheet.set("fullScreen")} />
          <Button label="Page sheet (iOS)" color="#8250df" onPress={() => sheet.set("pageSheet")} />
          <Button label="Form sheet (iOS)" color="#8250df" onPress={() => sheet.set("formSheet")} />
          <Button label="Fade" color="#8250df" onPress={() => sheet.set("fade")} />
          <Button label="Transparent dialog" color="#8250df" onPress={() => sheet.set("dialog")} />
          <Button label="Dismissible sheet (swipe / back closes)" color="#8250df" onPress={() => sheet.set("dismissible")} />
        </Section>
        <Section title="Back button (Android)">
          <View style={{ flexDirection: "row", alignItems: "center", gap: 12 }}>
            <Text style={{ flex: 1 }}>{intercept() ? `Intercepted ${backs()} times` : "Back pops this screen"}</Text>
            <Button label={intercept() ? "Release" : "Intercept"} color="#57606a" onPress={() => intercept.set(!intercept())} />
          </View>
        </Section>
      </View>

      <Modal
        visible={sheet() === "fullScreen" || sheet() === "pageSheet" || sheet() === "formSheet" || sheet() === "fade"}
        ios={{ presentationStyle: sheet() === "pageSheet" ? "pageSheet" : sheet() === "formSheet" ? "formSheet" : "fullScreen" }}
        animationType={sheet() === "fade" ? "fade" : "slide"}
        onRequestClose={close}
      >
        <SafeAreaView style={{ flex: 1, backgroundColor: colors.surface, padding: 20, gap: 12 }}>
          <Text style={{ fontSize: 24, fontWeight: "700" }}>{sheet()}</Text>
          <Insets />
          <Text style={{ color: colors.muted }}>Close with the button, the back button (Android) or a swipe down (iOS sheets).</Text>
          <Button label="Close" onPress={close} />
        </SafeAreaView>
      </Modal>
      <Modal visible={sheet() === "dismissible"} dismissible ios={{ presentationStyle: "pageSheet" }} onRequestClose={close}>
        <SafeAreaView style={{ flex: 1, backgroundColor: colors.surface, padding: 20, gap: 12 }}>
          <Text style={{ fontSize: 24, fontWeight: "700" }}>Dismissible</Text>
          <Text style={{ color: colors.muted }}>Swipe it down (iOS) or press back (Android): it closes natively, then tells the app.</Text>
          <Button label="Close" onPress={close} />
        </SafeAreaView>
      </Modal>
      <Modal visible={sheet() === "dialog"} transparent animationType="fade" onRequestClose={close}>
        <View style={{ flex: 1, backgroundColor: "rgba(0,0,0,0.4)", alignItems: "center", justifyContent: "center", padding: 32 }}>
          <View style={{ alignSelf: "stretch", backgroundColor: colors.surface, borderRadius: 14, padding: 20, gap: 12 }}>
            <Text style={{ fontSize: 20, fontWeight: "700" }}>A dialog</Text>
            <Text style={{ color: colors.muted }}>A transparent modal shows what is behind it.</Text>
            <Button label="OK" onPress={close} />
          </View>
        </View>
      </Modal>
    </Scroll>
  );
}
