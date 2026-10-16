import { mount, Scroll, signal, Text, View } from "nsh";

function App() {
  const count = signal(0);
  const busy = signal(false);
  const tab = signal("Day");
  const tabs = ["Day", "Week", "Month"];

  return (
    <Scroll class="flex-1 bg-zinc-100 dark:bg-[#101113]">
      <View class="gap-5 p-5 pt-6">
        <Text class="text-2xl font-bold tracking-tight text-zinc-900 dark:text-[#eceef1]">Tailwind classes</Text>

        <View class="flex-row flex-wrap gap-2">
          <View
            onPress={() => count.update((n) => n + 1)}
            class="rounded-lg bg-lime-300 px-4 py-2 transition-colors duration-150 active:bg-lime-400"
          >
            <Text class="font-medium text-lime-950">Pressed {count()}×</Text>
          </View>
          <View
            onPress={() => busy.set(!busy())}
            class="rounded-lg border border-zinc-300 px-4 py-2 transition active:scale-95 active:bg-zinc-200 dark:border-[#2c2e33] dark:bg-[#24262a] dark:active:bg-[#2d2f34]"
          >
            <Text class="font-medium text-zinc-900 dark:text-[#eceef1]">{busy() ? "Enable" : "Disable"}</Text>
          </View>
          <View disabled={busy()} onPress={() => count.set(0)} class="rounded-lg bg-emerald-400 px-4 py-2 active:bg-emerald-500 disabled:opacity-40">
            <Text class="font-medium text-emerald-950">Reset</Text>
          </View>
        </View>

        <View class="flex-row rounded-xl bg-zinc-200 p-1 dark:border dark:border-[#1f2124] dark:bg-[#0c0d0f]">
          <Each of={tabs}>
            {(t) => (
              <View
                onPress={() => tab.set(t())}
                class="flex-1 items-center rounded-lg py-2 transition-colors"
                classList={{ "bg-white shadow-sm dark:bg-[#2a2c31]": tab() === t() }}
              >
                <Text class="text-sm font-medium" classList={{ "text-zinc-900 dark:text-[#eceef1]": tab() === t(), "text-zinc-500 dark:text-[#8d9198]": tab() !== t() }}>
                  {t()}
                </Text>
              </View>
            )}
          </Each>
        </View>

        <View class="overflow-hidden rounded-2xl bg-linear-to-br from-lime-300 via-emerald-300 to-sky-400 p-5 shadow-xl shadow-lime-300/20">
          <Text class="text-xl font-bold text-zinc-950">{tab()} view</Text>
          <Text class="mt-1 text-sm text-zinc-950/70">bg-linear-to-br from-lime-300 via-emerald-300 to-sky-400</Text>
        </View>

        <View class="grid grid-cols-3 gap-2">
          <View class="col-span-2 h-14 items-center justify-center rounded-lg bg-sky-950">
            <Text class="text-xs text-sky-200">col-span-2</Text>
          </View>
          <View class="h-14 rounded-lg bg-sky-900" />
          <View class="h-14 rounded-lg bg-sky-800" />
          <View class="row-span-2 rounded-lg bg-sky-400" />
          <View class="h-14 rounded-lg bg-sky-700" />
          <View class="h-14 rounded-lg bg-sky-600" />
          <View class="h-14 rounded-lg bg-sky-500" />
        </View>

        <View class="overflow-hidden rounded-xl border border-zinc-200 bg-white dark:border-[#2c2e33] dark:bg-[#18191c]">
          <Each of={["Inbox", "Drafts", "Sent", "Archive"]}>
            {(item) => (
              <View class="flex-row items-center justify-between border-t border-zinc-200 px-4 py-3 first:border-t-0 even:bg-zinc-50 active:bg-indigo-50 dark:border-[#1f2124] dark:even:bg-[#141518] dark:active:bg-[#24262a]">
                <Text class="text-zinc-800 dark:text-[#eceef1]">{item()}</Text>
                <Text class="text-zinc-400 dark:text-[#5a5e65]">›</Text>
              </View>
            )}
          </Each>
        </View>

        <View class="flex-row items-center justify-around py-2">
          <View class="size-8 animate-spin rounded-md bg-lime-300" />
          <View class="size-8 animate-bounce rounded-full bg-sky-400" />
          <View class="h-8 w-14 animate-pulse rounded-md bg-zinc-300 dark:bg-[#2c2e33]" />
        </View>
      </View>
    </Scroll>
  );
}

mount(App);
