import { Input, type JSX, Scroll, signal, Text, View } from "nsh";
import { Image } from "nsh/image";
import logo from "../logo@2x.png";
import { Icon, type IconName } from "../icons";
import { Appear, colors } from "../ui";

/**
 * The gallery's dark design built from class strings alone: the graphite
 * surfaces and the citrus accent are `nsh.json` theme colors
 * (`graphite-*`, `citrus`), the light and depth (`finish` in ui.tsx) are
 * gradients and arbitrary shadows, everything else is stock Tailwind.
 */
const photo = "https://picsum.photos/seed/nsh-tailwind/600/400";

/** The finishes of ui.tsx as classes: a top-lit gradient, a highlight along the top edge, a soft shadow. */
const finish = {
  card: "bg-linear-to-b from-[#1d1e22] to-[#17181b] inset-shadow-[0_1px_0_rgba(255,255,255,0.06)] shadow-[0_10px_28px_rgba(0,0,0,0.38)]",
  inset: "inset-shadow-[0_1px_3px_rgba(0,0,0,0.55)]",
  raised: "bg-linear-to-b from-[#2d2f34] to-[#222428] inset-shadow-[0_1px_0_rgba(255,255,255,0.08)] shadow-[0_1px_2px_rgba(0,0,0,0.45)]",
  light: "bg-linear-to-b from-[#fbfbfc] to-[#d4d6d9] inset-shadow-[0_1px_0_rgba(255,255,255,0.9)] shadow-[0_3px_10px_rgba(0,0,0,0.4)]",
  accent: "bg-linear-to-b from-[#dbf893] to-[#b3dd4e] inset-shadow-[0_1px_0_rgba(255,255,255,0.45)] shadow-[0_3px_12px_rgba(200,242,109,0.22)]",
};

/** A card: an icon tile, a title and the classes it shows, over a darker inset. */
function Card(props: { icon: IconName; title: string; subtitle: string; children: JSX.Element }) {
  return (
    <View class={`rounded-2xl border border-graphite-border bg-graphite-card p-[5px] ${finish.card}`}>
      <View class="flex-row items-center gap-3 px-[11px] pb-3 pt-[11px]">
        <View class={`size-10 items-center justify-center rounded-xl border border-graphite-border bg-graphite-raised ${finish.raised}`}>
          <Icon name={props.icon} size={20} tint={colors.text} />
        </View>
        <View class="flex-1 gap-0.5">
          <Text class="text-[17px] font-semibold text-zinc-100">{props.title}</Text>
          <Text numberOfLines={2} class="font-mono text-xs leading-4 text-zinc-400">
            {props.subtitle}
          </Text>
        </View>
      </View>
      <View class={`gap-3 rounded-xl border border-graphite-line bg-graphite-inset p-3.5 ${finish.inset}`}>{props.children}</View>
    </View>
  );
}

function Label(props: { children: string }) {
  return <Text class="text-center text-xs text-zinc-400 tabular-nums">{props.children}</Text>;
}

/** A gradient hero lit by a citrus glow, with a big tabular figure and two badges. */
function Hero() {
  return (
    <View
      class={`overflow-hidden rounded-2xl border border-graphite-border bg-linear-to-br from-[#26282d] via-graphite-card to-graphite-inset p-5 inset-shadow-[0_1px_0_rgba(255,255,255,0.08)] shadow-[0_10px_28px_rgba(0,0,0,0.38)]`}
    >
      <View class="absolute -right-20 -top-24 size-72 rounded-full bg-radial from-citrus/20 to-transparent to-70%" />
      <View class="flex-row flex-wrap items-center gap-2">
        <View class="h-[26px] flex-row items-center gap-1.5 rounded-full bg-white/10 px-2.5 ring-1 ring-inset ring-white/10">
          <View class="size-2 items-center justify-center">
            <View class="absolute size-2 rounded-full bg-citrus animate-ping" />
            <View class="size-1.5 rounded-full bg-citrus" />
          </View>
          <Text class="text-xs font-medium text-zinc-100">Resolved natively</Text>
        </View>
        <View class="h-[26px] justify-center rounded-full bg-zinc-200 px-2.5 dark:bg-white/10">
          <Text class="font-mono text-xs text-zinc-900 dark:text-zinc-100">dark:</Text>
        </View>
      </View>
      <Text class="mt-6 text-[13px] text-zinc-400">This screen, styled with classes</Text>
      <Text class="text-5xl font-semibold tracking-tight text-zinc-100 tabular-nums">
        {String(sections.length)}
        <Text class="text-[17px] font-normal tracking-normal text-zinc-400"> cards</Text>
      </Text>
      <Text class="mt-2 font-mono text-xs text-zinc-500">bg-linear-to-br · bg-radial · inset-shadow-[…]</Text>
    </View>
  );
}

/** A grid of figures, each with a tinted note. */
function Stats() {
  const stats = [
    { label: "Swatches", value: String(palette.length * shades.length), note: `${palette.length} hues × ${shades.length} shades`, tone: "text-citrus" },
    { label: "Gradients", value: "4", note: "Linear, radial, conic", tone: "text-emerald-400" },
    { label: "Animations", value: "4", note: "Spin, ping, pulse, bounce", tone: "text-sky-400" },
    { label: "Unknown classes", value: "0", note: "Checked at build", tone: "text-amber-400" },
  ];
  return (
    <Card icon="stocks" title="Stat grid" subtitle="grid grid-cols-2 gap-2">
      <View class="grid grid-cols-2 gap-2">
        <Each of={stats}>
          {(s) => (
            <View class="gap-1 rounded-[10px] border border-graphite-line bg-linear-to-b from-[#1d1e22] to-[#17181b] p-3 inset-shadow-[0_1px_0_rgba(255,255,255,0.05)]">
              <Text class="text-xs text-zinc-500">{s().label}</Text>
              <Text class="text-[26px] font-semibold tracking-tight text-zinc-100 tabular-nums">{s().value}</Text>
              <Text numberOfLines={1} class={`text-xs tabular-nums ${s().tone}`}>
                {s().note}
              </Text>
            </View>
          )}
        </Each>
      </View>
      <View class="flex-row flex-wrap gap-x-4 gap-y-2">
        <Each of={["bg-citrus", "bg-emerald-400", "bg-sky-400", "bg-amber-400"]}>
          {(c, i) => (
            <View class="flex-row items-center gap-1.5">
              <View class={`size-2.5 rounded-[3px] ${c()}`} />
              <Text class="text-[13px] text-zinc-400">{["Palette", "Gradients", "Motion", "Checks"][i()]}</Text>
            </View>
          )}
        </Each>
      </View>
    </Card>
  );
}

/** Buttons: press states without JS, transitions, `disabled:`. */
function Buttons() {
  const count = signal(0);
  const busy = signal(false);
  return (
    <Card icon="components" title="Buttons" subtitle="active: hover: transition disabled: bg-linear-to-b">
      <View
        onPress={() => count.set(count() + 1)}
        class={`h-11 flex-row items-center justify-center rounded-[10px] bg-zinc-100 ${finish.light} transition duration-150 active:scale-[0.97] active:opacity-90`}
      >
        <Text class="text-[15px] font-semibold text-zinc-900">Pressed {count()}×</Text>
      </View>
      <View class="flex-row gap-2">
        <View
          onPress={() => count.set(0)}
          class={`h-11 flex-1 items-center justify-center rounded-[10px] border border-graphite-border bg-graphite-raised ${finish.raised} transition active:scale-95 active:opacity-80`}
        >
          <Text class="text-[15px] font-semibold text-zinc-100">Reset</Text>
        </View>
        <View
          onPress={() => busy.set(!busy())}
          disabled={busy()}
          class={`h-11 flex-1 items-center justify-center rounded-[10px] bg-citrus ${finish.accent} transition active:scale-95 hover:opacity-90 disabled:opacity-40`}
        >
          <Text class="text-[15px] font-semibold text-citrus-ink">{busy() ? "Disabled" : "Disable me"}</Text>
        </View>
      </View>
      <View onPress={() => busy.set(false)} class="min-h-11 justify-center self-start rounded-lg px-1 active:bg-white/5">
        <Text class="text-sm text-citrus underline decoration-dotted underline-offset-4">Enable again</Text>
      </View>
    </Card>
  );
}

/** Cards: a group press row, tinted badges, an inner ring. */
function Cards() {
  const badges = [
    "bg-emerald-400/15 text-emerald-400",
    "bg-sky-400/15 text-sky-400",
    "bg-amber-400/15 text-amber-400",
    "border border-rose-400 text-rose-400",
  ];
  return (
    <Card icon="styles" title="Cards and badges" subtitle="group-active: ring-inset shadow-*">
      <View class="group flex-row items-center gap-3 rounded-xl bg-linear-to-b from-[#2d2f34] to-[#222428] p-3 shadow-md shadow-black/40 ring-1 ring-inset ring-white/10 transition-transform duration-200 active:scale-[0.98]">
        <Image src={logo} class="size-12 rounded-[10px]" />
        <View class="flex-1 gap-0.5">
          <Text class="text-base font-semibold text-zinc-100">Group press</Text>
          <Text class="text-[13px] text-zinc-400 group-active:text-citrus">Press the card: this line changes color</Text>
        </View>
        <Text class="text-2xl text-zinc-600 transition-transform group-active:translate-x-1">›</Text>
      </View>
      <View class="flex-row flex-wrap gap-2">
        <Each of={badges}>
          {(b, i) => (
            <View class={`h-[26px] justify-center rounded-full px-2.5 ${b()}`}>
              <Text class="text-xs font-medium">{["Filled", "Live", "Pending", "Outline"][i()]}</Text>
            </View>
          )}
        </Each>
      </View>
    </Card>
  );
}

function Layout() {
  return (
    <Card icon="lists" title="Flex, grid, breakpoints" subtitle="flex-[2] col-span-2 row-span-2 sm:">
      <View class="flex-row gap-2">
        <View class="h-10 flex-1 rounded-lg bg-sky-400/20" />
        <View class="h-10 flex-[2] rounded-lg bg-sky-400/50" />
        <View class="size-10 rounded-lg bg-sky-400" />
      </View>
      <View class="grid grid-cols-3 gap-2">
        <View class="col-span-2 h-12 items-center justify-center rounded-lg border border-violet-400/40 bg-violet-500/15">
          <Text class="font-mono text-xs text-violet-300">col-span-2</Text>
        </View>
        <View class="h-12 rounded-lg bg-violet-500/25" />
        <View class="h-12 rounded-lg bg-violet-500/35" />
        <View class="row-span-2 items-center justify-center rounded-lg bg-violet-500">
          <Text class="font-mono text-[11px] text-white">row-span-2</Text>
        </View>
        <View class="h-12 rounded-lg bg-violet-500/45" />
        <View class="h-12 rounded-lg bg-violet-500/55" />
        <View class="h-12 rounded-lg bg-violet-500/70" />
      </View>
      <View class="flex-col gap-2 sm:flex-row">
        <View class="h-8 flex-1 rounded-lg bg-amber-400/30" />
        <View class="h-8 flex-1 rounded-lg bg-amber-400/60" />
      </View>
      <Label>The last row is a column on phones and a row from 640 pt up (sm:).</Label>
    </Card>
  );
}

const palette = ["red", "orange", "amber", "emerald", "sky", "indigo", "pink", "zinc"];
const shades = ["100", "300", "500", "700", "900"];

function Colors() {
  return (
    <Card icon="image" title="Palette" subtitle="bg-{color}-{shade}">
      <View class="gap-1.5">
        <Each of={palette}>
          {(name) => (
            <View class="flex-row items-center gap-1.5">
              <Text class="w-16 text-xs text-zinc-400">{name()}</Text>
              <Each of={shades}>{(shade) => <View class={`h-6 flex-1 rounded-md bg-${name()}-${shade()}`} />}</Each>
            </View>
          )}
        </Each>
      </View>
    </Card>
  );
}

function Shadows() {
  const sizes = ["shadow-sm", "shadow-md", "shadow-lg", "shadow-xl", "shadow-2xl"];
  return (
    <Card icon="spark" title="Shadows, rings, outlines" subtitle="shadow-* ring-* outline-* inset-shadow-*">
      <View class="flex-row justify-between px-1 py-2">
        <Each of={sizes}>
          {(s) => (
            <View class="items-center gap-2">
              <View class={`size-12 rounded-xl bg-graphite-raised shadow-black ${s()}`} />
              <Label>{s().slice(7)}</Label>
            </View>
          )}
        </Each>
      </View>
      <View class="flex-row justify-between px-1 pb-1">
        <View class="items-center gap-2">
          <View class="size-12 rounded-xl bg-black inset-shadow-sm inset-shadow-white/10 ring-1 ring-inset ring-white/10" />
          <Label>Inset</Label>
        </View>
        <View class="items-center gap-2">
          <View class="size-12 rounded-xl bg-graphite-raised ring-2 ring-sky-400 ring-offset-2 ring-offset-graphite-inset" />
          <Label>Ring</Label>
        </View>
        <View class="items-center gap-2">
          <View class="size-12 rounded-full bg-graphite-raised outline-2 outline-offset-2 outline-dashed outline-pink-400" />
          <Label>Outline</Label>
        </View>
        <View class="items-center gap-2">
          <View class="size-12 rounded-xl bg-emerald-400 shadow-lg shadow-emerald-400/50" />
          <Label>Glow</Label>
        </View>
      </View>
    </Card>
  );
}

function Gradients() {
  const kinds = [
    ["bg-linear-to-r from-sky-400 to-blue-600", "Linear"],
    ["bg-linear-45 from-rose-400 via-fuchsia-500 to-indigo-500", "45°"],
    ["bg-radial from-yellow-200 to-orange-500", "Radial"],
    ["bg-conic from-red-500 via-green-500 to-red-500", "Conic"],
  ];
  return (
    <Card icon="canvas" title="Gradients" subtitle="bg-linear-* bg-radial bg-conic">
      <View class="flex-row justify-between">
        <Each of={kinds}>
          {(k) => (
            <View class="items-center gap-2">
              <View class={`size-16 rounded-2xl ${k()[0]}`} />
              <Label>{k()[1]}</Label>
            </View>
          )}
        </Each>
      </View>
    </Card>
  );
}

function Typography() {
  return (
    <Card icon="text" title="Typography" subtitle="font-* tracking-* decoration-* line-clamp-*">
      <Text class="text-3xl font-black tracking-tight text-zinc-100">Display</Text>
      <Text class="text-lg font-light text-zinc-300">Light, large and relaxed</Text>
      <Text class="font-serif text-lg italic text-zinc-300">Serif italic</Text>
      <Text class="font-mono text-sm text-zinc-300 tabular-nums">font-mono 0123456789</Text>
      <View class="flex-row flex-wrap gap-x-4 gap-y-1 text-[15px] text-zinc-200">
        <Text class="underline decoration-wavy decoration-rose-400">wavy</Text>
        <Text class="underline decoration-double decoration-sky-400">double</Text>
        <Text class="underline decoration-dashed decoration-2">dashed</Text>
        <Text class="line-through decoration-zinc-500">struck</Text>
        <Text class="text-xs font-semibold uppercase tracking-widest text-citrus">caps</Text>
      </View>
      <Text class="truncate text-[13px] text-zinc-500">truncate: a single line that is far too long to fit in the card cut with an ellipsis</Text>
      <Text class="line-clamp-2 text-[13px] leading-relaxed text-zinc-500">
        line-clamp-2: Text is laid out once on the engine thread and drawn on the main thread; classes reach it through the same style path as everything
        else, so the third line of this paragraph never shows.
      </Text>
      <Text class="self-start rounded-md bg-sky-500 px-2 text-xl font-bold text-white text-shadow-md">text-shadow-md</Text>
    </Card>
  );
}

function Effects() {
  return (
    <Card icon="photos" title="Filters and blur" subtitle="grayscale sepia blur backdrop-blur mix-blend">
      <View class="flex-row justify-between">
        <Each of={["", "grayscale", "sepia", "blur-sm", "brightness-50"]}>
          {(f) => (
            <View class="items-center gap-1.5">
              <Image src={photo} class={`size-14 rounded-xl ${f()}`} />
              <Label>{f() || "None"}</Label>
            </View>
          )}
        </Each>
      </View>
      <View class="h-32 overflow-hidden rounded-xl">
        <Image src={photo} class="absolute inset-0 size-full" />
        <View class="absolute inset-x-3 bottom-3 rounded-lg border border-white/10 bg-black/40 px-3 py-2 backdrop-blur-md">
          <Text class="text-[13px] font-semibold text-white">backdrop-blur-md bg-black/40</Text>
        </View>
        <View class="absolute right-3 top-3 size-10 rounded-full bg-yellow-300 mix-blend-multiply" />
      </View>
    </Card>
  );
}

function Motion() {
  return (
    <Card icon="animation" title="Animations and transforms" subtitle="animate-* rotate-* skew-* scale-*">
      <View class="flex-row items-center justify-between px-1 py-2">
        <View class="items-center gap-2.5">
          <View class="size-8 rounded-lg bg-violet-500 animate-spin" />
          <Label>Spin</Label>
        </View>
        <View class="items-center gap-2.5">
          <View class="size-8 items-center justify-center">
            <View class="absolute size-4 rounded-full bg-sky-400 animate-ping" />
            <View class="size-4 rounded-full bg-sky-400" />
          </View>
          <Label>Ping</Label>
        </View>
        <View class="items-center gap-2.5">
          <View class="h-8 w-14 rounded-lg bg-graphite-raised animate-pulse" />
          <Label>Pulse</Label>
        </View>
        <View class="items-center gap-2.5">
          <View class="size-8 rounded-full bg-orange-400 animate-bounce" />
          <Label>Bounce</Label>
        </View>
        <View class="items-center gap-2.5">
          <View class="size-8 rotate-12 skew-x-6 scale-90 rounded-lg bg-emerald-400" />
          <Label>Transform</Label>
        </View>
      </View>
    </Card>
  );
}

function Lists() {
  const items = ["Inbox", "Drafts", "Sent", "Archive", "Trash"];
  return (
    <Card icon="lists" title="Structural variants" subtitle="first: even: last: active:">
      <View class="overflow-hidden rounded-xl border border-graphite-line">
        <Each of={items}>
          {(item) => (
            <View class="min-h-12 flex-row items-center justify-between border-t border-graphite-line px-3.5 first:border-t-0 even:bg-white/5 last:bg-rose-500/10 active:bg-citrus/10">
              <Text class="text-[15px] text-zinc-200 last:text-rose-300">{item()}</Text>
              <Text class="text-base text-zinc-600">›</Text>
            </View>
          )}
        </Each>
      </View>
    </Card>
  );
}

function Inputs() {
  return (
    <Card icon="form" title="Inputs" subtitle="focus: placeholder: caret-*">
      <Input
        placeholder="Focus me"
        class="h-11 rounded-[10px] border border-graphite-border bg-graphite-card px-3.5 inset-shadow-[0_1px_3px_rgba(0,0,0,0.55)] text-[15px] text-zinc-100 placeholder:text-zinc-600 caret-citrus focus:border-citrus focus:ring-2 focus:ring-citrus/25"
      />
      <Label>Focus adds a border and a ring; the placeholder and caret colors are classes too.</Label>
    </Card>
  );
}

const sections: (() => JSX.Element)[] = [Hero, Stats, Buttons, Cards, Layout, Shadows, Gradients, Typography, Effects, Motion, Lists, Inputs, Colors];

export function Tailwind() {
  return (
    <Scroll class="flex-1 bg-graphite-page">
      <View class="gap-4 p-4 pb-10">
        <Each of={sections}>
          {(section, i) => {
            const Body = section();
            return (
              <Appear index={i()}>
                <Body />
              </Appear>
            );
          }}
        </Each>
      </View>
    </Scroll>
  );
}
