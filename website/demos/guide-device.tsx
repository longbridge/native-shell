import { mount, Text, View } from "nsh";
import { colorScheme, fontScale, I18nManager, PixelRatio, Platform, windowSize } from "nsh/device";

function Row(props: { label: string; value: string }) {
  return (
    <View class="flex-row py-2 border-b border-zinc-200 dark:border-zinc-700">
      <Text class="flex-1 text-zinc-500 dark:text-zinc-400">{props.label}</Text>
      <Text class="text-zinc-900 dark:text-zinc-100">{props.value}</Text>
    </View>
  );
}

function App() {
  const hairline = 1 / PixelRatio.get();
  return (
    <View class="flex-1 bg-zinc-100 dark:bg-zinc-900 px-4 pt-6 gap-4">
      <Text class="text-2xl font-bold text-zinc-900 dark:text-white">Environment</Text>
      <View class="rounded-xl bg-white dark:bg-zinc-800 px-4 py-1">
        <Row label="windowSize()" value={`${Math.round(windowSize().width)} × ${Math.round(windowSize().height)}`} />
        <Row label="colorScheme()" value={colorScheme()} />
        <Row label="fontScale()" value={fontScale().toFixed(2)} />
        <Row label="Platform.OS" value={Platform.OS} />
        <Row label="Platform.Version" value={String(Platform.Version)} />
        <Row label="interfaceIdiom" value={Platform.constants.interfaceIdiom} />
        <Row label="PixelRatio.get()" value={String(PixelRatio.get())} />
        <Row label="I18nManager.isRTL" value={String(I18nManager.isRTL)} />
      </View>

      <Text class="text-zinc-600 dark:text-zinc-300">
        {Platform.select({ ios: "Running on iOS.", android: "Running on Android.", default: "Running elsewhere." })}
      </Text>

      <View class="gap-1">
        <View style={{ height: hairline, backgroundColor: "#8c959f" }} />
        <Text class="text-xs text-zinc-500">The line above is one device pixel tall.</Text>
      </View>

      <View class={windowSize().width >= 600 ? "flex-row gap-3" : "gap-3"}>
        <View class="flex-1 rounded-xl bg-sky-500 dark:bg-sky-700 p-4">
          <Text class="text-white font-semibold">dark: classes follow the system</Text>
        </View>
        <View class="flex-1 rounded-xl bg-violet-500 dark:bg-violet-700 p-4">
          <Text class="text-white font-semibold">A row only when the window is 600 pt or wider</Text>
        </View>
      </View>

      <View
        onPress={() => I18nManager.forceRTL(!I18nManager.isRTL)}
        activeOpacity={0.6}
        class="flex-row items-center gap-3 rounded-xl bg-white dark:bg-zinc-800 p-4"
      >
        <Text class="text-lg">⇄</Text>
        <Text class="flex-1 text-zinc-900 dark:text-zinc-100">Lay out right to left: {I18nManager.isRTL ? "on" : "off"}</Text>
      </View>
    </View>
  );
}

mount(App);
