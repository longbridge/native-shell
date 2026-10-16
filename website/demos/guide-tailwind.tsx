import { mount, Scroll, signal, Text, View } from "nsh";

function App() {
  const count = signal(0);
  const busy = signal(false);
  const tab = signal("Day");
  const tabs = ["Day", "Week", "Month"];

  return (
    <Scroll class="flex-1 bg-zinc-100 dark:bg-zinc-900">
      <View class="gap-5 p-5 pt-6">
        <Text class="text-2xl font-bold tracking-tight text-zinc-900 dark:text-white">Tailwind classes</Text>

        <View class="flex-row flex-wrap gap-2">
          <View
            onPress={() => count.update((n) => n + 1)}
            class="rounded-lg bg-indigo-600 px-4 py-2 transition-colors duration-150 active:bg-indigo-800"
          >
            <Text class="font-medium text-white">Pressed {count()}×</Text>
          </View>
          <View
            onPress={() => busy.set(!busy())}
            class="rounded-lg border border-zinc-300 px-4 py-2 transition active:scale-95 active:bg-zinc-200 dark:border-zinc-600"
          >
            <Text class="font-medium text-zinc-900 dark:text-white">{busy() ? "Enable" : "Disable"}</Text>
          </View>
          <View disabled={busy()} onPress={() => count.set(0)} class="rounded-lg bg-emerald-500 px-4 py-2 active:bg-emerald-700 disabled:opacity-40">
            <Text class="font-medium text-white">Reset</Text>
          </View>
        </View>

        <View class="flex-row rounded-xl bg-zinc-200 p-1 dark:bg-zinc-800">
          <Each of={tabs}>
            {(t) => (
              <View
                onPress={() => tab.set(t())}
                class="flex-1 items-center rounded-lg py-2 transition-colors"
                classList={{ "bg-white shadow-sm dark:bg-zinc-600": tab() === t() }}
              >
                <Text class="text-sm font-medium" classList={{ "text-zinc-900 dark:text-white": tab() === t(), "text-zinc-500": tab() !== t() }}>
                  {t()}
                </Text>
              </View>
            )}
          </Each>
        </View>

        <View class="overflow-hidden rounded-2xl bg-linear-to-br from-indigo-500 via-fuchsia-500 to-orange-400 p-5 shadow-xl shadow-indigo-500/40">
          <Text class="text-xl font-bold text-white">{tab()} view</Text>
          <Text class="mt-1 text-sm text-white/80">bg-linear-to-br from-indigo-500 via-fuchsia-500 to-orange-400</Text>
        </View>

        <View class="grid grid-cols-3 gap-2">
          <View class="col-span-2 h-14 items-center justify-center rounded-lg bg-sky-200">
            <Text class="text-xs text-sky-900">col-span-2</Text>
          </View>
          <View class="h-14 rounded-lg bg-sky-300" />
          <View class="h-14 rounded-lg bg-sky-400" />
          <View class="row-span-2 rounded-lg bg-sky-500" />
          <View class="h-14 rounded-lg bg-sky-600" />
          <View class="h-14 rounded-lg bg-sky-700" />
          <View class="h-14 rounded-lg bg-sky-800" />
        </View>

        <View class="overflow-hidden rounded-xl border border-zinc-200 bg-white dark:border-zinc-700 dark:bg-zinc-800">
          <Each of={["Inbox", "Drafts", "Sent", "Archive"]}>
            {(item) => (
              <View class="flex-row items-center justify-between border-t border-zinc-200 px-4 py-3 first:border-t-0 even:bg-zinc-50 active:bg-indigo-50 dark:border-zinc-700 dark:even:bg-zinc-900">
                <Text class="text-zinc-800 dark:text-zinc-100">{item()}</Text>
                <Text class="text-zinc-400">›</Text>
              </View>
            )}
          </Each>
        </View>

        <View class="flex-row items-center justify-around py-2">
          <View class="size-8 animate-spin rounded-md bg-indigo-500" />
          <View class="size-8 animate-bounce rounded-full bg-orange-400" />
          <View class="h-8 w-14 animate-pulse rounded-md bg-zinc-300" />
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
