import type { JSX } from "nsh";
import { Text, View } from "nsh";

export const colors = {
  text: "#1f2328",
  muted: "#6e7781",
  border: "#d0d7de",
  surface: "#ffffff",
  background: "#f4f5f7",
  accent: "#0969da",
} as const;

export function Button(props: { label: string; color?: string; onPress: () => void }) {
  return (
    <View
      onPress={props.onPress}
      activeOpacity={0.6}
      accessibilityRole="button"
      style={{ backgroundColor: props.color ?? colors.accent, borderRadius: 10, paddingVertical: 10, paddingHorizontal: 16 }}
    >
      <Text style={{ color: "#ffffff", fontWeight: "600", textAlign: "center" }}>{props.label}</Text>
    </View>
  );
}

/** A titled group of controls on a white card. */
export function Section(props: { title: string; children: JSX.Element }) {
  return (
    <View style={{ gap: 8 }}>
      <Text style={{ fontSize: 13, fontWeight: "600", color: colors.muted, textTransform: "uppercase" }}>{props.title}</Text>
      <View style={{ backgroundColor: colors.surface, borderRadius: 12, padding: 14, gap: 12 }}>{props.children}</View>
    </View>
  );
}

/** An on / off toggle drawn with views (the same on every platform). */
export function Toggle(props: { value: boolean; onChange: (value: boolean) => void }) {
  return (
    <View
      onPress={() => props.onChange(!props.value)}
      accessibilityRole="switch"
      accessibilityState={{ checked: props.value }}
      hitSlop={8}
      class="h-[31px] w-[51px] rounded-full p-0.5 transition-colors duration-200"
      classList={{ "bg-[#34c759]": props.value, "bg-zinc-300": !props.value }}
    >
      <View
        class="size-[27px] rounded-full bg-white shadow-md transition-transform duration-200"
        classList={{ "translate-x-5": props.value }}
      />
    </View>
  );
}
