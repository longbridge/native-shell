import { mount, Scroll, Text, View } from "nsh";
import { Screen, Stack, StatusBar } from "nsh/ui/mobile";
import { Animations } from "./demos/Animations";
import { CanvasDemo } from "./demos/CanvasDemo";
import { Components } from "./demos/Components";
import { Form } from "./demos/Form";
import { Gestures } from "./demos/Gestures";
import { Images } from "./demos/Images";
import { Lists } from "./demos/Lists";
import { Media } from "./demos/Media";
import { MobileUi } from "./demos/MobileUi";
import { Navigation } from "./demos/Navigation";
import { Photos } from "./demos/Photos";
import { Physics } from "./demos/Physics";
import { Scrolling } from "./demos/Scrolling";
import { Stocks } from "./demos/Stocks";
import { Chat } from "./demos/Chat";
import { Styles } from "./demos/Styles";
import { System } from "./demos/System";
import { Tailwind } from "./demos/Tailwind";
import { TextDemo } from "./demos/TextDemo";
import { Icon } from "./icons";
import { type DemoEntry, nav } from "./nav";
import { Appear, colors, header, IconTile, isDark, mono, Press, radius } from "./ui";

/** The demos, by what they show. */
const groups: { title: string; demos: DemoEntry[] }[] = [
  {
    title: "Apps",
    demos: [
      { name: "Stocks", detail: "A live watchlist and a stock: candles, depth, the tape, a tick every 50 ms", icon: "stocks", screen: Stocks },
      { name: "AI Chat", detail: "Streaming replies, tools, reasoning, attachments, errors", icon: "chat", screen: Chat },
      { name: "Photos", detail: "A long recycled list: images, pull to refresh, paging", icon: "photos", screen: Photos },
    ],
  },
  {
    title: "Layout and lists",
    demos: [
      { name: "Lists", detail: "Columns, sticky sections, a chat, horizontal lists", icon: "lists", screen: Lists },
      { name: "Scrolling", detail: "Paging, snapping, zoom, scroll events, prepending, keyboard taps", icon: "scroll", screen: Scrolling },
      { name: "Navigation", detail: "Header options, modals, the back button, safe areas", icon: "nav", screen: Navigation },
      { name: "Form", detail: "Inputs, keyboard types, the keyboard over the content", icon: "form", screen: Form },
    ],
  },
  {
    title: "Touch and motion",
    demos: [
      { name: "Gestures", detail: "Pinch, rotate, press states, hit slop, pointer events", icon: "gestures", screen: Gestures },
      { name: "Physics", detail: "Drags and springs on the UI thread", icon: "physics", screen: Physics },
      { name: "Animations", detail: "Easing, sequence / parallel / stagger / loop, springs, interpolation", icon: "animation", screen: Animations },
    ],
  },
  {
    title: "Drawing and type",
    demos: [
      { name: "Canvas", detail: "2D context: paths, gradients, text, images, native animation", icon: "canvas", screen: CanvasDemo },
      { name: "Styles", detail: "Corners, borders, shadows, clipping, 3D transforms", icon: "styles", screen: Styles },
      { name: "Tailwind", detail: "class=\"…\" resolved natively: variants, grid, shadows, motion", icon: "tailwind", screen: Tailwind },
      { name: "Text", detail: "Fonts, spans, inline views and images, decorations, truncation", icon: "text", screen: TextDemo },
      { name: "Images", detail: "Resize modes, tint, GIF / WebP, placeholders, blur, fade", icon: "image", screen: Images },
    ],
  },
  {
    title: "Platform",
    demos: [
      { name: "Mobile UI", detail: "nsh/ui/mobile in Rust: a ListView and a Group of the app's rows", icon: "mobile", screen: MobileUi },
      { name: "Components", detail: "Press feedback, sheet, layout animation, input accessory, modal options", icon: "components", screen: Components },
      { name: "Web and video", detail: "WKWebView / WebView, AVPlayer / MediaPlayer", icon: "media", screen: Media },
      { name: "System", detail: "Status bar, alerts, action sheets, environment, timers, fetch", icon: "system", screen: System },
    ],
  },
];

function DemoRow(props: { demo: DemoEntry; first: boolean }) {
  return (
    <Press onPress={() => nav.push({ name: "demo", demo: props.demo })} scale={0.98} accessibilityLabel={props.demo.name}>
      <View
        style={{
          flexDirection: "row",
          alignItems: "center",
          gap: 14,
          paddingVertical: 12,
          paddingHorizontal: 12,
          borderTopWidth: props.first ? 0 : 1,
          borderTopColor: colors.line,
        }}
      >
        <IconTile name={props.demo.icon} size={38} />
        <View style={{ flex: 1, gap: 2 }}>
          <Text style={{ fontSize: 16, fontWeight: "500", color: colors.text }}>{props.demo.name}</Text>
          <Text numberOfLines={2} style={{ fontSize: 13, lineHeight: 18, color: colors.muted }}>
            {props.demo.detail}
          </Text>
        </View>
        <Icon name="chevron" size={16} tint={colors.faint} />
      </View>
    </Press>
  );
}

function Home() {
  return (
    <Screen title="Gallery" {...header}>
      <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
        <View style={{ padding: 16, paddingBottom: 40, gap: 20 }}>
          <Each of={groups}>
            {(group, g) => (
              <Appear index={g() + 1}>
                <View style={{ gap: 8 }}>
                  <View style={{ flexDirection: "row", alignItems: "baseline", paddingHorizontal: 4 }}>
                    <Text style={{ flex: 1, fontSize: 13, fontWeight: "600", color: colors.muted, letterSpacing: 0.4 }}>
                      {group().title.toUpperCase()}
                    </Text>
                    <Text style={{ fontSize: 12, color: colors.faint, fontFamily: mono }}>{group().demos.length}</Text>
                  </View>
                  <View
                    style={{
                      backgroundColor: colors.surface,
                      borderRadius: radius.card,
                      borderCurve: "continuous",
                      borderWidth: 1,
                      borderColor: colors.border,
                      overflow: "hidden",
                    }}
                  >
                    <Each of={group().demos}>{(demo, i) => <DemoRow demo={demo()} first={i() === 0} />}</Each>
                  </View>
                </View>
              </Appear>
            )}
          </Each>
        </View>
      </Scroll>
    </Screen>
  );
}

function App() {
  return (
    <View style={{ flex: 1, backgroundColor: colors.background }}>
      <StatusBar barStyle={isDark() ? "light-content" : "dark-content"} />
      <Stack navigator={nav} style={{ flex: 1 }}>
        {(route) => {
          switch (route.name) {
            case "home":
              return <Home />;
            case "demo": {
              const Demo = route.demo.screen;
              return (
                <Screen title={route.demo.name} {...header}>
                  <View style={{ flex: 1, backgroundColor: colors.background }}>
                    <Demo />
                  </View>
                </Screen>
              );
            }
            case "page": {
              const Page = route.page;
              return <Page />;
            }
          }
        }}
      </Stack>
    </View>
  );
}

mount(App);
