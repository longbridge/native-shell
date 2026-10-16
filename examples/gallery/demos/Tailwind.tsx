import { Input, Scroll, signal, Text, View } from "nsh";
import { Image } from "nsh/image";
import logo from "../logo@2x.png";

const photo = "https://picsum.photos/seed/nsh-tailwind/600/400";

function Section(props: { title: string; children: any }) {
  return (
    <View class="gap-2">
      <Text class="text-xs font-semibold uppercase tracking-wide text-zinc-500 dark:text-zinc-400">{props.title}</Text>
      <View class="gap-3 rounded-card bg-white p-4 dark:bg-zinc-800">{props.children}</View>
    </View>
  );
}

function Label(props: { children: string }) {
  return <Text class="text-center text-[11px] text-zinc-500 dark:text-zinc-400">{props.children}</Text>;
}

/** Buttons: press states without JS, transitions, `disabled:`. */
function Buttons() {
  const count = signal(0);
  const busy = signal(false);
  return (
    <View class="gap-3">
      <View class="flex-row flex-wrap gap-2">
        <View
          onPress={() => count.set(count() + 1)}
          class="rounded-lg bg-brand-600 px-4 py-2 transition-colors duration-150 active:bg-brand-700 hover:bg-brand-500"
        >
          <Text class="font-medium text-white">Pressed {count()}×</Text>
        </View>
        <View
          onPress={() => count.set(0)}
          class="rounded-lg border border-zinc-300 px-4 py-2 transition active:scale-95 active:bg-zinc-100 dark:border-zinc-600 dark:active:bg-zinc-700"
        >
          <Text class="font-medium text-zinc-900 dark:text-white">Reset</Text>
        </View>
        <View
          onPress={() => busy.set(!busy())}
          disabled={busy()}
          class="rounded-full bg-emerald-500 px-4 py-2 disabled:opacity-40 active:bg-emerald-600"
        >
          <Text class="font-medium text-white">{busy() ? "Disabled" : "Enabled"}</Text>
        </View>
      </View>
      <View onPress={() => busy.set(false)} class="self-start rounded-md px-2 py-1 active:bg-zinc-100 dark:active:bg-zinc-700">
        <Text class="text-sm text-brand-600 underline decoration-dotted underline-offset-4">Enable again</Text>
      </View>
    </View>
  );
}

function Cards() {
  return (
    <View class="gap-3">
      <View class="group flex-row items-center gap-3 rounded-2xl bg-white p-3 shadow-lg ring-1 ring-black/5 transition-transform duration-200 active:scale-[0.98] dark:bg-zinc-700">
        <Image src={logo} class="size-12 rounded-xl" />
        <View class="flex-1 gap-0.5">
          <Text class="text-base font-semibold text-zinc-900 dark:text-white">Group press</Text>
          <Text class="text-sm text-zinc-500 group-active:text-brand-600 dark:text-zinc-300">Press the card: this line changes color</Text>
        </View>
        <Text class="text-2xl text-zinc-300 transition-transform group-active:translate-x-1">›</Text>
      </View>
      <View class="overflow-hidden rounded-2xl bg-linear-to-br from-brand-500 via-fuchsia-500 to-orange-400 p-5 shadow-xl shadow-brand-500/40">
        <Text class="text-xl font-bold text-white">Gradient card</Text>
        <Text class="mt-1 text-sm text-white/80">bg-linear-to-br from-brand-500 via-fuchsia-500 to-orange-400</Text>
      </View>
    </View>
  );
}

function Layout() {
  return (
    <View class="gap-3">
      <View class="flex-row gap-2">
        <View class="h-10 flex-1 rounded-md bg-sky-200" />
        <View class="h-10 flex-[2] rounded-md bg-sky-400" />
        <View class="h-10 w-10 rounded-md bg-sky-600" />
      </View>
      <View class="grid grid-cols-3 gap-2">
        <View class="col-span-2 h-12 items-center justify-center rounded-md bg-violet-200">
          <Text class="text-xs text-violet-900">col-span-2</Text>
        </View>
        <View class="h-12 rounded-md bg-violet-300" />
        <View class="h-12 rounded-md bg-violet-400" />
        <View class="row-span-2 rounded-md bg-violet-500" />
        <View class="h-12 rounded-md bg-violet-600" />
        <View class="h-12 rounded-md bg-violet-700" />
        <View class="h-12 rounded-md bg-violet-800" />
      </View>
      <View class="flex-col gap-2 sm:flex-row">
        <View class="h-8 flex-1 rounded-md bg-amber-300" />
        <View class="h-8 flex-1 rounded-md bg-amber-500" />
      </View>
      <Label>The last row is a column on phones and a row from `sm:` (640pt) up.</Label>
    </View>
  );
}

const palette = ["red", "orange", "amber", "emerald", "sky", "indigo", "pink", "zinc"];
const shades = ["100", "300", "500", "700", "900"];

function Colors() {
  return (
    <View class="gap-1">
      <Each of={palette}>
        {(name) => (
          <View class="flex-row gap-1">
            <Text class="w-16 text-xs text-zinc-500">{name()}</Text>
            <Each of={shades}>{(shade) => <View class={`h-6 flex-1 rounded bg-${name()}-${shade()}`} />}</Each>
          </View>
        )}
      </Each>
    </View>
  );
}

function Shadows() {
  const sizes = ["shadow-xs", "shadow-sm", "shadow-md", "shadow-lg", "shadow-xl", "shadow-2xl"];
  return (
    <View class="gap-4">
      <View class="flex-row flex-wrap justify-between gap-y-5 px-1 py-2">
        <Each of={sizes}>
          {(s) => (
            <View class="gap-1">
              <View class={`size-14 rounded-xl bg-white dark:bg-zinc-600 ${s()}`} />
              <Label>{s().slice(7)}</Label>
            </View>
          )}
        </Each>
      </View>
      <View class="flex-row justify-between px-1">
        <View class="gap-1">
          <View class="size-14 rounded-xl bg-zinc-100 inset-shadow-sm" />
          <Label>inset</Label>
        </View>
        <View class="gap-1">
          <View class="size-14 rounded-xl bg-white ring-2 ring-sky-500 ring-offset-2" />
          <Label>ring</Label>
        </View>
        <View class="gap-1">
          <View class="size-14 rounded-full bg-white outline-2 outline-offset-2 outline-dashed outline-pink-500" />
          <Label>outline</Label>
        </View>
        <View class="gap-1">
          <View class="size-14 rounded-xl bg-emerald-500 shadow-lg shadow-emerald-500/50" />
          <Label>colored</Label>
        </View>
      </View>
    </View>
  );
}

function Gradients() {
  const kinds = [
    "bg-linear-to-r from-sky-400 to-blue-600",
    "bg-linear-45 from-rose-400 via-fuchsia-500 to-indigo-500",
    "bg-radial from-yellow-200 to-orange-500",
    "bg-conic from-red-500 via-green-500 to-red-500",
  ];
  return (
    <View class="flex-row justify-between">
      <Each of={kinds}>{(k) => <View class={`size-16 rounded-xl ${k()}`} />}</Each>
    </View>
  );
}

function Typography() {
  return (
    <View class="gap-2">
      <Text class="text-3xl font-black tracking-tight text-zinc-900 dark:text-white">Display</Text>
      <Text class="text-lg font-light text-zinc-700 dark:text-zinc-200">Light, large and relaxed</Text>
      <Text class="font-serif text-lg italic text-zinc-700 dark:text-zinc-200">Serif italic</Text>
      <Text class="font-mono text-sm text-zinc-700 tabular-nums dark:text-zinc-200">font-mono 0123456789</Text>
      <View class="flex-row flex-wrap gap-x-4 gap-y-1 text-zinc-800 dark:text-zinc-100">
        <Text class="underline decoration-wavy decoration-red-500">wavy</Text>
        <Text class="underline decoration-double decoration-sky-500">double</Text>
        <Text class="underline decoration-dashed decoration-2">dashed</Text>
        <Text class="line-through decoration-zinc-400">struck</Text>
        <Text class="uppercase tracking-widest text-xs font-semibold text-brand-600">caps</Text>
      </View>
      <Text class="truncate text-sm text-zinc-500">truncate: a single line that is far too long to fit in the card cut with an ellipsis</Text>
      <Text class="line-clamp-2 text-sm leading-relaxed text-zinc-500">
        line-clamp-2: Text is laid out once on the engine thread and drawn on the main thread; classes reach it through the same style path as everything
        else, so the third line of this paragraph never shows.
      </Text>
      <Text class="text-shadow-md text-xl font-bold text-white bg-sky-500 rounded-md px-2 self-start">text-shadow-md</Text>
    </View>
  );
}

function Effects() {
  return (
    <View class="gap-3">
      <View class="flex-row justify-between">
        <Each of={["", "grayscale", "sepia", "blur-sm", "brightness-50"]}>
          {(f) => (
            <View class="gap-1">
              <Image src={photo} class={`size-14 rounded-lg ${f()}`} />
              <Label>{f() || "none"}</Label>
            </View>
          )}
        </Each>
      </View>
      <View class="h-28 overflow-hidden rounded-xl">
        <Image src={photo} class="absolute inset-0 size-full" />
        <View class="absolute inset-x-3 bottom-3 rounded-lg bg-white/40 px-3 py-2 backdrop-blur-md">
          <Text class="text-sm font-semibold text-zinc-900">backdrop-blur-md bg-white/40</Text>
        </View>
        <View class="absolute right-3 top-3 size-10 rounded-full bg-yellow-300 mix-blend-multiply" />
      </View>
    </View>
  );
}

function Motion() {
  return (
    <View class="flex-row items-center justify-between px-2 py-3">
      <View class="gap-2 items-center">
        <View class="size-8 rounded-md bg-brand-500 animate-spin" />
        <Label>spin</Label>
      </View>
      <View class="gap-2 items-center">
        <View class="size-8 items-center justify-center">
          <View class="absolute size-4 rounded-full bg-sky-400 animate-ping" />
          <View class="size-4 rounded-full bg-sky-500" />
        </View>
        <Label>ping</Label>
      </View>
      <View class="gap-2 items-center">
        <View class="h-8 w-14 rounded-md bg-zinc-300 animate-pulse" />
        <Label>pulse</Label>
      </View>
      <View class="gap-2 items-center">
        <View class="size-8 rounded-full bg-orange-400 animate-bounce" />
        <Label>bounce</Label>
      </View>
      <View class="gap-2 items-center">
        <View class="size-8 rotate-12 skew-x-6 scale-90 rounded-md bg-emerald-500" />
        <Label>transform</Label>
      </View>
    </View>
  );
}

function Lists() {
  const items = ["Inbox", "Drafts", "Sent", "Archive", "Trash"];
  return (
    <View class="overflow-hidden rounded-xl border border-zinc-200 dark:border-zinc-700">
      <Each of={items}>
        {(item) => (
          <View class="flex-row items-center justify-between border-t border-zinc-200 px-3 py-2.5 first:border-t-0 even:bg-zinc-50 last:bg-rose-50 active:bg-brand-50 dark:border-zinc-700 dark:even:bg-zinc-900 dark:last:bg-rose-950">
            <Text class="text-zinc-800 dark:text-zinc-100">{item()}</Text>
            <Text class="text-xs text-zinc-400">›</Text>
          </View>
        )}
      </Each>
    </View>
  );
}

function Inputs() {
  return (
    <View class="gap-2">
      <Input
        placeholder="Focus me"
        class="rounded-lg border border-zinc-300 px-3 py-2 text-base text-zinc-900 placeholder:text-zinc-400 caret-brand-600 focus:border-brand-500 focus:ring-2 focus:ring-brand-500/30 dark:border-zinc-600 dark:text-white"
      />
      <Label>focus: border and ring, placeholder: color, caret color</Label>
    </View>
  );
}

const sections: [string, () => any][] = [
  ["Buttons: active, hover, transition, disabled", Buttons],
  ["Cards: shadow, ring, group, gradient", Cards],
  ["Flex, grid, breakpoints", Layout],
  ["Palette", Colors],
  ["Shadows, rings, outlines", Shadows],
  ["Gradients", Gradients],
  ["Typography", Typography],
  ["Filters, backdrop blur, blend modes", Effects],
  ["Animations and transforms", Motion],
  ["first:, even:, last:", Lists],
  ["Inputs", Inputs],
];

export function Tailwind() {
  return (
    <Scroll class="flex-1 bg-zinc-100 dark:bg-zinc-900">
      <View class="gap-5 p-4">
        <Each of={sections}>
          {(section) => {
            const [title, Body] = section();
            return (
              <Section title={title}>
                <Body />
              </Section>
            );
          }}
        </Each>
      </View>
    </Scroll>
  );
}
