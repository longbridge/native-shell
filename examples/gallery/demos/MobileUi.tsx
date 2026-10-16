// `nsh/ui/mobile`'s Rust components laying out the app's content: a
// `ListView` recycles rows the app renders, a `Group` wraps each child in a
// row. The app writes the content; the components only place it. Their
// colors come from the app's theme, so the text the app puts inside them
// keeps the color they give it.
import { signal, Text, View } from "nsh";
import { Button, Group, ListView, SelectList, Sheet, Stepper, type StepperRef, TextField } from "nsh/ui/mobile";
import { Appear, Badge, Caption, Card, colors, hint, isDark, mono, Row, Toggle } from "../ui";

const inset = signal(false);

const cities = Array.from({ length: 500 }, (_, i) => `City ${i + 1}`);

const stocks = Array.from({ length: 300 }, (_, i) => ({
  id: i,
  symbol: `SYM${String(i).padStart(3, "0")}`,
  price: (50 + ((i * 37) % 400) + (i % 7) * 0.13).toFixed(2),
  change: (((i * 13) % 21) - 10) / 4,
}));

function Header() {
  const taps = signal(0);
  const quantity = signal(2);
  const asked = signal("");
  let own!: StepperRef;
  const name = signal("Ada");
  const sheet = signal(false);
  const city = signal<number | null>(null);
  return (
    <View style={{ padding: 16, gap: 16, backgroundColor: colors.background }}>
      <Appear>
        <View style={{ gap: 6, paddingHorizontal: 4 }}>
          <View style={{ flexDirection: "row", gap: 8 }}>
            <Badge label="Rust" tone="accent" />
            <Badge label="nsh-ui-mobile" />
          </View>
          <Text style={{ fontSize: 15, lineHeight: 21, color: colors.muted }}>
            Components written in Rust, running in the core: the app hands them JSX and data, they draw the boxes and keep their own state.
          </Text>
        </View>
      </Appear>

      <Appear index={1}>
        <Card icon="form" title="TextField" subtitle="A label, a clear button, controlled or not">
          <TextField label="Name" placeholder="Your name" value={name()} onChange={(v) => name.set(v)} />
          <TextField label="Password" placeholder="At least 8 characters" secure />
          <Caption label={`Hello, ${name() || "nobody"}`} note="× clears and focuses" />
        </Card>
      </Appear>

      <Appear index={2}>
        <Card icon="components" title="Sheet" subtitle={hint("Follows the finger natively, springs back", "Follows the pointer natively, springs back")}>
          <Button title="Open the sheet" onPress={() => sheet.set(true)} />
          <Caption label={hint("Drag, tap outside or press back", "Drag it down or click outside")} note="onClose" />
        </Card>
      </Appear>
      <Sheet open={sheet()} onClose={() => sheet.set(false)}>
        <Text style={{ fontSize: 17, fontWeight: "600" }}>A sheet in Rust</Text>
        <Text style={{ opacity: 0.7 }}>
          {hint(
            "It follows the finger natively and springs back; drag it down, tap outside or press back to close.",
            "It follows the pointer natively and springs back; drag it down or click outside to close.",
          )}
        </Text>
        <Button title="Done" onPress={() => sheet.set(false)} />
      </Sheet>

      <Appear index={3}>
        <Card
          icon="lists"
          title="SelectList"
          subtitle="500 rows of Rust data"
          right={<Badge label={city() === null ? "none" : cities[city()!]} tone={city() === null ? "neutral" : "accent"} />}
          flush
        >
          <View style={{ height: 240, borderRadius: 15, borderCurve: "continuous", overflow: "hidden" }}>
            <SelectList options={cities} value={city()} onSelect={(v) => city.set(v)} />
          </View>
        </Card>
      </Appear>

      <Appear index={4}>
        <Card icon="mobile" title="Group" subtitle="Each JSX child becomes a row" flush>
          <Group header={<Text>Settings</Text>} footer={<Text>The rows are JSX; the group adds the box and the dividers.</Text>}>
            <Text>Notifications</Text>
            <Text>Privacy</Text>
            <View style={{ flexDirection: "row", justifyContent: "space-between" }}>
              <Text>Taps</Text>
              <Text style={{ fontVariant: ["tabular-nums"] }}>{taps()}</Text>
            </View>
          </Group>
        </Card>
      </Appear>

      <Appear index={5}>
        <Card icon="plus" title="Stepper" subtitle="State and timers in Rust: hold a button" flush>
          <Group>
            <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
              <Text>Own value</Text>
              <Stepper ref={own} defaultValue={5} min={0} max={50} />
            </View>
            <View style={{ flexDirection: "row", alignItems: "center", justifyContent: "space-between" }}>
              <Text style={{ fontVariant: ["tabular-nums"] }}>{`Controlled: ${quantity()}`}</Text>
              <Stepper value={quantity()} min={0} max={10} step={0.5} onChange={(v) => quantity.set(v)} />
            </View>
          </Group>
        </Card>
      </Appear>

      <Appear index={6}>
        <Card icon="spark" title="Methods" subtitle="A Rust component's ref, called from JS">
          <Button
            title={asked() || "Ask the stepper, then reset it"}
            variant="secondary"
            onPress={async () => {
              // Methods of a Rust component, called from JS.
              asked.set(`It was ${await own.value()}; reset to 5`);
              own.setValue(5);
            }}
          />
          <Button title={`Tap (${taps()})`} variant="secondary" onPress={() => taps.set(taps() + 1)} />
          <View>
            <Row first>
              <View style={{ flex: 1, gap: 2 }}>
                <Text style={{ fontSize: 15, color: colors.text }}>Inset rows</Text>
                <Text style={{ fontSize: 12, color: colors.faint, fontFamily: mono }}>{`variant="${inset() ? "inset" : "plain"}"`}</Text>
              </View>
              <Toggle value={inset()} onChange={(v) => inset.set(v)} label="Inset rows" />
            </Row>
          </View>
        </Card>
      </Appear>

      <Text style={{ fontSize: 13, color: colors.muted, paddingHorizontal: 4 }}>
        Below: a ListView of 300 rows the app renders, recycled natively
      </Text>
    </View>
  );
}

export function MobileUi() {
  return (
    <ListView variant={inset() ? "inset" : "plain"} each={stocks} keyExtractor={(s) => String(s.id)} ListHeaderComponent={Header}>
      {(stock) => (
        <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
          <View style={{ flex: 1, gap: 1 }}>
            <Text style={{ fontSize: 15, fontWeight: "600" }}>{stock().symbol}</Text>
            <Text style={{ fontSize: 12, opacity: 0.55, fontVariant: ["tabular-nums"] }}>{`No. ${stock().id + 1}`}</Text>
          </View>
          <Text style={{ width: 80, textAlign: "right", fontSize: 15, fontVariant: ["tabular-nums"] }}>{stock().price}</Text>
          <View
            style={{
              width: 72,
              height: 28,
              borderRadius: 8,
              alignItems: "center",
              justifyContent: "center",
              backgroundColor: stock().change >= 0 ? colors.up : colors.down,
            }}
          >
            <Text style={{ fontSize: 13, fontWeight: "600", color: isDark() ? colors.inset : "#ffffff", fontVariant: ["tabular-nums"] }}>
              {(stock().change >= 0 ? "+" : "") + stock().change.toFixed(2)}%
            </Text>
          </View>
        </View>
      )}
    </ListView>
  );
}
