// The module the home page's host app shows: what an assistant could
// generate for "How are my tech stocks doing today?" — a compact watchlist
// in the gallery's look (examples/gallery, Stocks): the composite, then a
// row per stock with today's sparkline, the price and its change, ticking
// live. Fictional tickers; sized for a region about 300 points wide. It
// follows the page's appearance.
import { cleanup, computed, mount, signal, Text, View } from "nsh";
import { Canvas } from "nsh/canvas";
import { colorScheme } from "nsh/device";

const PALETTES = {
  dark: {
    background: "#18191c",
    text: "#eceef1",
    muted: "#8d9198",
    faint: "#5a5e65",
    line: "#1f2124",
    up: "#45d49a",
    down: "#ff6170",
  },
  light: {
    background: "#ffffff",
    text: "#16171a",
    muted: "#5f636a",
    faint: "#9a9ea4",
    line: "#ececea",
    up: "#16a06a",
    down: "#e5484d",
  },
};
const palette = () => PALETTES[colorScheme() === "light" ? "light" : "dark"];

interface Stock {
  code: string;
  name: string;
  prevClose: number;
  points: () => number[];
  set: (points: number[]) => void;
}

/** Points a sparkline draws. */
const POINTS = 24;

/** A seeded random walk, so the first frame is the same on every load. */
function walk(seed: number, start: number, volatility: number, drift: number): number[] {
  let x = seed;
  const rand = () => ((x = (x * 16807) % 2147483647) / 2147483647) - 0.5;
  const out = [start];
  for (let i = 1; i < POINTS; i++) out.push(out[i - 1] * (1 + rand() * volatility + drift));
  return out;
}

function stock(code: string, name: string, prevClose: number, seed: number, drift: number): Stock {
  const points = signal(walk(seed, prevClose, 0.01, drift));
  return { code, name, prevClose, points, set: (p) => points.set(p) };
}

const STOCKS = [
  stock("QNTM", "Quantum Silicon", 182.4, 11, 0.0016),
  stock("ORBT", "Orbital Cloud", 64.18, 23, -0.0006),
  stock("NVLX", "Novalux Devices", 241.9, 37, 0.0012),
  stock("HLXS", "Helix Systems", 38.72, 51, 0.0018),
];

const last = (s: Stock) => s.points()[POINTS - 1];
const change = (s: Stock) => ((last(s) - s.prevClose) / s.prevClose) * 100;
const signed = (v: number) => `${v > 0 ? "+" : ""}${v.toFixed(2)}`;
const tone = (v: number) => (v > 0 ? palette().up : v < 0 ? palette().down : palette().muted);

/** Today's line through the points, a soft fill under it and a dot at the
 * live price; the previous close dotted. */
function Sparkline(props: { stock: Stock }) {
  return (
    <Canvas
      pointerEvents="none"
      style={{ width: 64, height: 28 }}
      draw={(ctx, { width, height }) => {
        const ys = props.stock.points();
        const color = tone(change(props.stock));
        let lo = props.stock.prevClose;
        let hi = lo;
        for (const v of ys) {
          lo = Math.min(lo, v);
          hi = Math.max(hi, v);
        }
        const pad = 4;
        const X = (k: number) => pad + (k / (POINTS - 1)) * (width - pad * 2);
        const Y = (v: number) => pad + ((hi - v) / (hi - lo || 1)) * (height - pad * 2);
        const curve = () => {
          ctx.moveTo(X(0), Y(ys[0]));
          for (let k = 1; k < POINTS - 1; k++) {
            ctx.quadraticCurveTo(X(k), Y(ys[k]), (X(k) + X(k + 1)) / 2, (Y(ys[k]) + Y(ys[k + 1])) / 2);
          }
          ctx.lineTo(X(POINTS - 1), Y(ys[POINTS - 1]));
        };
        const fill = ctx.createLinearGradient(0, 0, 0, height);
        fill.addColorStop(0, color === palette().up ? "rgba(69,212,154,0.22)" : "rgba(255,97,112,0.22)");
        fill.addColorStop(1, "rgba(0,0,0,0)");
        ctx.beginPath();
        curve();
        ctx.lineTo(X(POINTS - 1), height);
        ctx.lineTo(X(0), height);
        ctx.closePath();
        ctx.fillStyle = fill;
        ctx.fill();

        ctx.strokeStyle = palette().faint;
        ctx.lineWidth = 1;
        ctx.setLineDash([1, 3]);
        ctx.beginPath();
        ctx.moveTo(pad, Y(props.stock.prevClose));
        ctx.lineTo(width - pad, Y(props.stock.prevClose));
        ctx.stroke();
        ctx.setLineDash([]);

        ctx.beginPath();
        curve();
        ctx.strokeStyle = color;
        ctx.lineWidth = 1.5;
        ctx.lineJoin = "round";
        ctx.lineCap = "round";
        ctx.stroke();

        ctx.fillStyle = color;
        ctx.globalAlpha = 0.25;
        ctx.beginPath();
        ctx.arc(X(POINTS - 1), Y(ys[POINTS - 1]), 5, 0, Math.PI * 2);
        ctx.fill();
        ctx.globalAlpha = 1;
        ctx.beginPath();
        ctx.arc(X(POINTS - 1), Y(ys[POINTS - 1]), 2.5, 0, Math.PI * 2);
        ctx.fill();
      }}
    />
  );
}

function Row(props: { stock: Stock; first: boolean }) {
  return (
    <View style={{ height: 54, flexDirection: "row", alignItems: "center", gap: 10, paddingHorizontal: 14 }}>
      <If when={!props.first}>
        <View style={{ position: "absolute", top: 0, left: 14, right: 0, height: 1, backgroundColor: palette().line }} />
      </If>
      <View style={{ flex: 1, gap: 2 }}>
        <Text numberOfLines={1} style={{ fontSize: 14, fontWeight: "600", color: palette().text }}>
          {props.stock.code}
        </Text>
        <Text numberOfLines={1} style={{ fontSize: 11.5, color: palette().muted }}>
          {props.stock.name}
        </Text>
      </View>
      <Sparkline stock={props.stock} />
      <View style={{ width: 70, alignItems: "flex-end", gap: 2 }}>
        <Text style={{ fontSize: 14, fontWeight: "600", color: palette().text, fontVariant: ["tabular-nums"] }}>
          {last(props.stock).toFixed(2)}
        </Text>
        <Text style={{ fontSize: 11.5, fontWeight: "500", color: tone(change(props.stock)), fontVariant: ["tabular-nums"] }}>
          {signed(change(props.stock))}%
        </Text>
      </View>
    </View>
  );
}

function App() {
  // A tick: one stock moves; every few ticks the line advances a point.
  let tick = 0;
  const id = setInterval(() => {
    tick++;
    const s = STOCKS[tick % STOCKS.length];
    const p = s.points();
    const next = p[POINTS - 1] * (1 + (Math.random() - 0.47) * 0.006);
    s.set(tick % 8 === 0 ? [...p.slice(1), next] : [...p.slice(0, POINTS - 1), next]);
  }, 450);
  cleanup(() => clearInterval(id));

  const composite = computed(() => STOCKS.reduce((sum, s) => sum + change(s), 0) / STOCKS.length);

  return (
    <View style={{ flex: 1, backgroundColor: palette().background, paddingTop: 12 }}>
      <View style={{ flexDirection: "row", alignItems: "baseline", gap: 8, paddingHorizontal: 14, paddingBottom: 8 }}>
        <Text style={{ flex: 1, fontSize: 16, fontWeight: "700", color: palette().text }}>Watchlist</Text>
        <Text style={{ fontSize: 12, color: palette().muted }}>Tech</Text>
        <Text style={{ fontSize: 13, fontWeight: "600", color: tone(composite()), fontVariant: ["tabular-nums"] }}>
          {signed(composite())}%
        </Text>
      </View>
      <Each of={STOCKS}>{(s, i) => <Row stock={s()} first={i() === 0} />}</Each>
    </View>
  );
}

mount(App);
