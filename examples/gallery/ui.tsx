import type { JSX, ViewStyle } from "nsh";
import { effect, signal, Text, View } from "nsh";
import { createNativeValue, Easing } from "nsh/animated";
import { Icon, type IconName } from "./icons";

/**
 * The gallery's look: a graphite page, cards with a darker inset surface,
 * tabular numbers and one citrus accent. Sizes are for phones: 16 pt
 * gutters, 44 pt touch targets, 15 pt body text.
 */
export const colors = {
  /** The page behind the cards. */
  background: "#101113",
  /** A card. */
  surface: "#18191c",
  /** A surface inside a card. */
  inset: "#0c0d0f",
  /** A control on a card (buttons, fields, tracks). */
  raised: "#24262a",
  border: "#2c2e33",
  /** Hairlines inside insets. */
  line: "#1f2124",
  text: "#eceef1",
  muted: "#8d9198",
  faint: "#5a5e65",
  accent: "#c8f26d",
  /** Text on the accent. */
  onAccent: "#13160c",
  /** A light button (the main action). */
  light: "#eef0f2",
  info: "#7ea8ff",
  warn: "#ffb547",
  danger: "#ff6170",
  up: "#45d49a",
  down: "#ff6170",
} as const;

/** Translucent tints of the tones, for fills behind text. */
export const tints = {
  accent: "rgba(200,242,109,0.14)",
  info: "rgba(126,168,255,0.16)",
  warn: "rgba(255,181,71,0.16)",
  danger: "rgba(255,97,112,0.16)",
  up: "rgba(69,212,154,0.16)",
  down: "rgba(255,97,112,0.16)",
} as const;

/**
 * Light and depth: gentle top-to-bottom gradients, a highlight along top
 * edges and soft shadows, so cards, controls and buttons read as surfaces.
 */
export const finish = {
  card: {
    backgroundImage: "linear-gradient(180deg, #1d1e22 0%, #17181b 100%)",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.06), 0 10px 28px rgba(0,0,0,0.38)",
  },
  inset: { boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" },
  raised: {
    backgroundImage: "linear-gradient(180deg, #2d2f34 0%, #222428 100%)",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
  },
  light: {
    backgroundImage: "linear-gradient(180deg, #fbfbfc 0%, #d4d6d9 100%)",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
  },
  accent: {
    backgroundImage: "linear-gradient(180deg, #dbf893 0%, #b3dd4e 100%)",
    boxShadow: "inset 0 1px 0 rgba(255,255,255,0.45), 0 3px 12px rgba(200,242,109,0.22)",
  },
  track: { boxShadow: "inset 0 1px 3px rgba(0,0,0,0.6)" },
} as const;

export type Tone = "neutral" | "accent" | "info" | "warn" | "danger" | "up" | "down";

/** The color of a tone. */
export const toneColor = (tone: Tone) => (tone === "neutral" ? colors.muted : colors[tone]);
/** The tint of a tone. */
export const toneTint = (tone: Tone) => (tone === "neutral" ? colors.raised : tints[tone]);

/** The monospaced family, for code (API names, identifiers). Figures use
 * the system font with tabular numbers instead. */
export const mono = "monospace";
export const radius = { card: 16, inset: 12, control: 10 } as const;

/** Header props of every screen. */
export const header = {
  headerBackgroundColor: colors.background,
  headerTitleColor: colors.text,
  headerTintColor: colors.accent,
} as const;

/** The motion every control shares: quick, slightly springy. */
export const motion = {
  press: { stiffness: 520, damping: 34 },
  settle: { stiffness: 320, damping: 30 },
  enter: { duration: 420, easing: Easing.out(Easing.cubic) },
} as const;

/**
 * A pressable that sinks a little under the finger and springs back (the
 * spring runs natively; only press in / out reach JS).
 */
export function Press(props: {
  onPress?: () => void;
  onLongPress?: () => void;
  disabled?: boolean;
  /** How far it shrinks (default 0.97). */
  scale?: number;
  style?: ViewStyle;
  accessibilityRole?: "button" | "tab" | "switch" | "link";
  accessibilityLabel?: string;
  accessibilityState?: { selected?: boolean; checked?: boolean; disabled?: boolean };
  hitSlop?: number;
  children: JSX.Element;
}) {
  const s = createNativeValue(1);
  return (
    <View
      onPress={props.onPress}
      onLongPress={props.onLongPress}
      disabled={props.disabled}
      onPressIn={() => s.spring(props.scale ?? 0.97, motion.press)}
      onPressOut={() => s.spring(1, motion.press)}
      accessibilityRole={props.accessibilityRole ?? "button"}
      accessibilityLabel={props.accessibilityLabel}
      accessibilityState={props.accessibilityState}
      hitSlop={props.hitSlop}
      style={[props.style ?? null, { transform: [{ scale: s }], opacity: props.disabled ? 0.4 : 1 }]}
    >
      {props.children}
    </View>
  );
}

/**
 * Fades and lifts its content in once mounted; `index` staggers siblings
 * (40 ms apart). `flex` lets it wrap a page that fills its parent.
 */
export function Appear(props: { index?: number; flex?: boolean; children: JSX.Element }) {
  const t = createNativeValue(0);
  t.timing(1, { ...motion.enter, delay: Math.min(8, props.index ?? 0) * 40 });
  return (
    <View style={{ flex: props.flex ? 1 : undefined, opacity: t, transform: [{ translateY: t.interpolate({ inputRange: [0, 1], outputRange: [14, 0] }) }] }}>
      {props.children}
    </View>
  );
}

/** A rounded icon tile. */
export function IconTile(props: { name: IconName; tint?: string; size?: number }) {
  const size = props.size ?? 40;
  return (
    <View
      style={{
        width: size,
        height: size,
        borderRadius: size * 0.3,
        borderCurve: "continuous",
        borderWidth: 1,
        borderColor: colors.border,
        backgroundColor: colors.raised,
        ...finish.raised,
        alignItems: "center",
        justifyContent: "center",
      }}
    >
      <Icon name={props.name} size={size * 0.5} tint={props.tint ?? colors.text} />
    </View>
  );
}

/**
 * A card: an optional icon, a title and a subtitle, something on the right
 * (a badge, an action), then the content on an inset surface (or bare with
 * `flush`).
 */
export function Card(props: {
  icon?: IconName;
  title?: string;
  subtitle?: JSX.Element;
  right?: JSX.Element;
  flush?: boolean;
  style?: ViewStyle;
  children: JSX.Element;
}) {
  return (
    <View
      style={[
        {
          backgroundColor: colors.surface,
          borderRadius: radius.card,
          borderCurve: "continuous",
          borderWidth: 1,
          borderColor: colors.border,
          padding: 5,
          ...finish.card,
        },
        props.style ?? null,
      ]}
    >
      <If when={props.title}>
        <View style={{ flexDirection: "row", alignItems: "center", gap: 12, paddingHorizontal: 11, paddingTop: 11, paddingBottom: 12 }}>
          <If when={props.icon}>
            <IconTile name={props.icon!} />
          </If>
          <View style={{ flex: 1, gap: 2 }}>
            <Text style={{ fontSize: 17, fontWeight: "600", color: colors.text }}>{props.title}</Text>
            <If when={props.subtitle}>
              <Text numberOfLines={2} style={{ fontSize: 13, color: colors.muted }}>
                {props.subtitle}
              </Text>
            </If>
          </View>
          {props.right}
        </View>
      </If>
      <If when={!props.flush} else={props.children}>
        <Inset>{props.children}</Inset>
      </If>
    </View>
  );
}

/** The darker surface inside a card. */
export function Inset(props: { gap?: number; padding?: number; children: JSX.Element }) {
  return (
    <View
      style={{
        backgroundColor: colors.inset,
        borderRadius: radius.inset,
        borderCurve: "continuous",
        borderWidth: 1,
        borderColor: colors.line,
        ...finish.inset,
        padding: props.padding ?? 14,
        gap: props.gap ?? 12,
      }}
    >
      {props.children}
    </View>
  );
}

/** A caption row: a label on the left, a quieter note on the right. */
export function Caption(props: { label: JSX.Element; note?: JSX.Element }) {
  return (
    <View style={{ flexDirection: "row", alignItems: "baseline", gap: 8 }}>
      <Text style={{ flex: 1, fontSize: 14, color: colors.muted }}>{props.label}</Text>
      <If when={props.note}>
        <Text style={{ fontSize: 13, color: colors.faint, fontVariant: ["tabular-nums"] }}>{props.note}</Text>
      </If>
    </View>
  );
}

/** A large figure with a tinted note after it ("38 / 1k  would pass"). */
export function Figure(props: { value: string; unit?: string; note?: string; tone?: Tone }) {
  return (
    <Text style={{ fontSize: 32, fontWeight: "600", color: colors.text, fontVariant: ["tabular-nums"], letterSpacing: -0.5 }}>
      {props.value}
      <If when={props.unit}>
        <Text style={{ fontSize: 17, fontWeight: "400", color: colors.muted }}> {props.unit}</Text>
      </If>
      <If when={props.note}>
        <Text style={{ fontSize: 15, fontWeight: "500", color: toneColor(props.tone ?? "accent") }}> {props.note}</Text>
      </If>
    </Text>
  );
}

/** A small label in a tinted capsule. */
export function Badge(props: { label: string; tone?: Tone; outline?: boolean }) {
  const tone = () => props.tone ?? "neutral";
  return (
    <View
      style={{
        paddingHorizontal: 10,
        height: 26,
        borderRadius: 13,
        justifyContent: "center",
        backgroundColor: props.outline ? "transparent" : toneTint(tone()),
        borderWidth: props.outline ? 1 : 0,
        borderColor: toneColor(tone()),
      }}
    >
      <Text style={{ fontSize: 12, fontWeight: "500", color: tone() === "neutral" ? colors.text : toneColor(tone()) }}>{props.label}</Text>
    </View>
  );
}

/** A label above a value, for grids of figures. */
export function Stat(props: { label: string; value: string; tone?: Tone; width?: `${number}%` }) {
  return (
    <View style={{ width: props.width, gap: 3, paddingVertical: 4 }}>
      <Text style={{ fontSize: 12, color: colors.muted }}>{props.label}</Text>
      <Text style={{ fontSize: 15, color: props.tone ? toneColor(props.tone) : colors.text, fontVariant: ["tabular-nums"] }}>
        {props.value}
      </Text>
    </View>
  );
}

/** A colored square and a label, for chart legends. */
export function Legend(props: { color: string; label: string; value?: string }) {
  return (
    <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
      <View style={{ width: 9, height: 9, borderRadius: 2.5, backgroundColor: props.color }} />
      <Text style={{ fontSize: 13, color: colors.muted }}>
        {props.label}
        <If when={props.value}>
          <Text style={{ color: colors.text, fontVariant: ["tabular-nums"] }}> {props.value}</Text>
        </If>
      </Text>
    </View>
  );
}

/** A row inside an inset, with a hairline above all but the first. */
export function Row(props: { first?: boolean; onPress?: () => void; children: JSX.Element }) {
  return (
    <View
      onPress={props.onPress}
      underlayColor={props.onPress ? colors.raised : undefined}
      style={{
        minHeight: 48,
        flexDirection: "row",
        alignItems: "center",
        gap: 12,
        borderTopWidth: props.first ? 0 : 1,
        borderTopColor: colors.line,
      }}
    >
      {props.children}
    </View>
  );
}

export type ButtonVariant = "primary" | "secondary" | "accent" | "ghost";

/**
 * A button, 44 pt tall: `primary` is light (the main action), `secondary`
 * dark, `accent` citrus. A `color` fills it with that color instead.
 */
export function Button(props: {
  label: string;
  variant?: ButtonVariant;
  color?: string;
  icon?: IconName;
  disabled?: boolean;
  /** Grows to share a row with its siblings. */
  flex?: number;
  onPress: () => void;
}) {
  const variant = () => props.variant ?? "primary";
  const bg = () =>
    props.color ?? (variant() === "primary" ? colors.light : variant() === "accent" ? colors.accent : variant() === "ghost" ? "transparent" : colors.raised);
  const fg = () => (props.color ? "#ffffff" : variant() === "primary" ? "#111214" : variant() === "accent" ? colors.onAccent : colors.text);
  return (
    <Press onPress={props.onPress} disabled={props.disabled} accessibilityLabel={props.label} style={props.flex ? { flex: props.flex } : undefined}>
      <View
        style={{
          height: 44,
          paddingHorizontal: 18,
          borderRadius: radius.control,
          borderCurve: "continuous",
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "center",
          gap: 8,
          backgroundColor: bg(),
          ...(props.color || variant() === "ghost"
            ? {}
            : variant() === "primary"
              ? finish.light
              : variant() === "accent"
                ? finish.accent
                : finish.raised),
          borderWidth: variant() === "secondary" || variant() === "ghost" ? 1 : 0,
          borderColor: colors.border,
        }}
      >
        <If when={props.icon}>
          <Icon name={props.icon!} size={16} tint={fg()} />
        </If>
        <Text style={{ fontSize: 15, fontWeight: "600", color: fg() }}>{props.label}</Text>
      </View>
    </Press>
  );
}

/** A round icon button (44 pt): light by default, `variant="secondary"` dark. */
export function RoundButton(props: {
  icon: IconName;
  label: string;
  variant?: "primary" | "secondary";
  size?: number;
  disabled?: boolean;
  onPress: () => void;
}) {
  const size = props.size ?? 44;
  const primary = () => (props.variant ?? "primary") === "primary";
  return (
    <Press onPress={props.onPress} disabled={props.disabled} scale={0.92} accessibilityLabel={props.label}>
      <View
        style={{
          width: size,
          height: size,
          borderRadius: size / 2,
          alignItems: "center",
          justifyContent: "center",
          backgroundColor: primary() ? colors.light : colors.raised,
          ...(primary() ? finish.light : finish.raised),
          borderWidth: primary() ? 0 : 1,
          borderColor: colors.border,
        }}
      >
        <Icon name={props.icon} size={size * 0.42} tint={primary() ? "#111214" : colors.text} />
      </View>
    </Press>
  );
}

/** A titled card of controls (the demos' sections). */
export function Section(props: { title: string; note?: string; style?: ViewStyle; children: JSX.Element }) {
  return (
    <View
      style={[
        {
          backgroundColor: colors.surface,
          borderRadius: radius.card,
          borderCurve: "continuous",
          borderWidth: 1,
          borderColor: colors.border,
          padding: 5,
          ...finish.card,
        },
        props.style ?? null,
      ]}
    >
      <View style={{ flexDirection: "row", alignItems: "baseline", paddingHorizontal: 11, paddingTop: 10, paddingBottom: 10, gap: 8 }}>
        <Text style={{ flex: 1, fontSize: 15, fontWeight: "600", color: colors.text }}>{props.title}</Text>
        <If when={props.note}>
          <Text style={{ fontSize: 13, color: colors.faint, fontVariant: ["tabular-nums"] }}>{props.note}</Text>
        </If>
      </View>
      <Inset>{props.children}</Inset>
    </View>
  );
}

/** An on / off switch: the knob springs across, the track changes color natively. */
export function Toggle(props: { value: boolean; onChange: (value: boolean) => void; label?: string }) {
  const t = createNativeValue(props.value ? 1 : 0);
  let last = props.value;
  return (
    <View
      onPress={() => {
        last = !props.value;
        t.spring(last ? 1 : 0, motion.settle);
        props.onChange(last);
      }}
      accessibilityRole="switch"
      accessibilityLabel={props.label}
      accessibilityState={{ checked: props.value }}
      hitSlop={8}
      style={{
        width: 50,
        height: 30,
        borderRadius: 15,
        padding: 3,
        backgroundColor: t.interpolate({ inputRange: [0, 1], outputRange: [colors.raised, colors.accent] }),
        borderWidth: 1,
        borderColor: colors.border,
        ...finish.track,
      }}
    >
      <View
        style={{
          width: 22,
          height: 22,
          borderRadius: 11,
          backgroundColor: props.value ? colors.onAccent : colors.muted,
          transform: [{ translateX: t.interpolate({ inputRange: [0, 1], outputRange: [0, 20] }) }],
        }}
      />
    </View>
  );
}

/**
 * Equal segments with an indicator that springs to the selected one
 * (`count` shows a quiet number after a label).
 */
export function Segmented<T extends string>(props: {
  options: readonly { value: T; label: string; count?: number }[];
  value: T;
  onChange: (value: T) => void;
  size?: "small" | "regular";
}) {
  const width = signal(0);
  const x = createNativeValue(0);
  const index = () => Math.max(0, props.options.findIndex((o) => o.value === props.value));
  const segment = () => (width() - 6) / Math.max(1, props.options.length);
  const height = props.size === "small" ? 32 : 40;
  // The first placement jumps; later ones spring.
  let placed = false;
  effect(() => {
    const to = index() * segment();
    if (width() <= 0) return;
    if (placed) x.spring(to, motion.settle);
    else x.set(to);
    placed = true;
  });
  return (
    <View
      onLayout={(e) => width.set(e.width)}
      accessibilityRole="tablist"
      style={{
        height,
        flexDirection: "row",
        padding: 3,
        borderRadius: height / 2,
        backgroundColor: colors.inset,
        ...finish.track,
        borderWidth: 1,
        borderColor: colors.border,
      }}
    >
      <View
        pointerEvents="none"
        style={{
          position: "absolute",
          top: 3,
          left: 3,
          bottom: 3,
          width: segment(),
          borderRadius: (height - 6) / 2,
          backgroundColor: colors.raised,
          ...finish.raised,
          borderWidth: 1,
          borderColor: colors.border,
          transform: [{ translateX: x }],
        }}
      />
      <Each of={props.options}>
        {(option) => (
          <View
            onPress={() => props.onChange(option().value)}
            accessibilityRole="tab"
            accessibilityState={{ selected: props.value === option().value }}
            style={{ flex: 1, flexDirection: "row", alignItems: "center", justifyContent: "center", gap: 5 }}
          >
            <Text
              numberOfLines={1}
              style={{
                fontSize: props.size === "small" ? 13 : 14,
                fontWeight: "500",
                color: props.value === option().value ? colors.text : colors.muted,
              }}
            >
              {option().label}
            </Text>
            <If when={option().count !== undefined}>
              <Text style={{ fontSize: 12, color: colors.faint, fontVariant: ["tabular-nums"] }}>{option().count}</Text>
            </If>
          </View>
        )}
      </Each>
    </View>
  );
}

/** A horizontal meter with an optional marker (a limit). */
export function Meter(props: { value: number; tone?: Tone; marker?: number }) {
  return (
    <View style={{ height: 6, borderRadius: 3, backgroundColor: colors.raised }}>
      <View
        style={{
          width: `${Math.max(0, Math.min(1, props.value)) * 100}%`,
          height: 6,
          borderRadius: 3,
          backgroundColor: toneColor(props.tone ?? "accent"),
        }}
      />
      <If when={props.marker !== undefined}>
        <View style={{ position: "absolute", left: `${(props.marker ?? 0) * 100}%`, top: -3, width: 2, height: 12, borderRadius: 1, backgroundColor: colors.text }} />
      </If>
    </View>
  );
}

