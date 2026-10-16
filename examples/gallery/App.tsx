import { mount, Scroll, Text, View } from "nsh";
import { Screen, Stack } from "nsh/ui/mobile";
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
import { Styles } from "./demos/Styles";
import { System } from "./demos/System";
import { Tailwind } from "./demos/Tailwind";
import { TextDemo } from "./demos/TextDemo";
import { type DemoEntry, nav } from "./nav";
import { colors } from "./ui";

const demos: DemoEntry[] = [
  { name: "Photos", detail: "A long recycled list: images, pull to refresh, paging", screen: Photos },
  { name: "Lists", detail: "Columns, sticky sections, a chat, horizontal lists", screen: Lists },
  { name: "Scrolling", detail: "Paging, snapping, zoom, scroll events, prepending, keyboard taps", screen: Scrolling },
  { name: "Navigation", detail: "Header options, modals, the back button, safe areas", screen: Navigation },
  { name: "Form", detail: "Inputs, keyboard types, the keyboard over the content", screen: Form },
  { name: "Mobile UI", detail: "nsh/ui/mobile in Rust: a ListView and a Group laying out the app's rows", screen: MobileUi },
  { name: "Components", detail: "Press feedback, sheet, layout animation, input accessory, keyboard, modal options, Platform", screen: Components },
  { name: "Gestures", detail: "Pinch, rotate, press states, hit slop, pointer events", screen: Gestures },
  { name: "Physics", detail: "Drags and springs on the UI thread", screen: Physics },
  { name: "Animations", detail: "Easing, sequence / parallel / stagger / loop, springs, interpolation, diffClamp", screen: Animations },
  { name: "Images", detail: "Resize modes, tint, GIF / WebP, placeholders, blur, fade, events", screen: Images },
  { name: "Canvas", detail: "2D context: paths, gradients, text, images, native animation", screen: CanvasDemo },
  { name: "Tailwind", detail: "class=\"…\" resolved natively: variants, grid, shadows, gradients, motion", screen: Tailwind },
  { name: "Styles", detail: "Corners (also continuous), borders, shadows, clipping, 3D transforms", screen: Styles },
  { name: "Text", detail: "Fonts, spans, inline views and images, decorations, truncation, shadows", screen: TextDemo },
  { name: "Web and video", detail: "WKWebView / WebView, AVPlayer / MediaPlayer", screen: Media },
  { name: "System", detail: "Status bar, alerts, action sheets, environment, timers, fetch", screen: System },
];

function Home() {
  return (
    <Screen title="Gallery">
      <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
        <View style={{ margin: 16, borderRadius: 12, backgroundColor: colors.surface, overflow: "hidden" }}>
          <Each of={demos}>
            {(demo, i) => (
              <View
                onPress={() => nav.push({ name: "demo", demo: demo() })}
                activeOpacity={0.5}
                accessibilityRole="button"
                style={{
                  paddingHorizontal: 16,
                  paddingVertical: 12,
                  gap: 2,
                  borderTopWidth: i() === 0 ? 0 : 1,
                  borderTopColor: "#eaeef2",
                }}
              >
                <Text style={{ fontSize: 17, color: colors.text }}>{demo().name}</Text>
                <Text style={{ fontSize: 13, color: colors.muted }}>{demo().detail}</Text>
              </View>
            )}
          </Each>
        </View>
      </Scroll>
    </Screen>
  );
}

function App() {
  return (
    <Stack navigator={nav} style={{ flex: 1 }}>
      {(route) => {
        switch (route.name) {
          case "home":
            return <Home />;
          case "demo": {
            const Demo = route.demo.screen;
            return (
              <Screen title={route.demo.name}>
                <Demo />
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
  );
}

mount(App);
