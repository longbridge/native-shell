// The module the home page's host app shows: what an assistant could
// generate for "How did AAPL and NVDA do?" — a card with a range picker,
// a chart and an action. Sized for a region about 300 points wide.
import { computed, mount, signal, Text, View } from "nsh";
import { Canvas } from "nsh/canvas";

type Range = "1D" | "1W" | "1M";

// Normalized closes (start = 100) per range, fixed so the demo needs no network.
const SERIES: Record<Range, { a: number[]; b: number[] }> = {
  "1D": {
    a: [100, 100.4, 100.1, 100.9, 101.3, 101.0, 101.6, 101.9, 101.4, 102.1],
    b: [100, 99.2, 99.8, 100.6, 101.9, 102.4, 101.7, 103.0, 103.8, 104.2],
  },
  "1W": {
    a: [100, 101.2, 100.6, 102.0, 101.4, 102.8, 103.1, 102.6, 103.9, 104.4],
    b: [100, 98.6, 101.1, 103.4, 102.2, 105.0, 106.3, 105.1, 107.8, 108.6],
  },
  "1M": {
    a: [100, 97.8, 99.4, 101.9, 100.8, 103.6, 105.2, 104.1, 106.9, 108.3],
    b: [100, 103.2, 101.0, 106.4, 109.8, 107.2, 112.5, 115.1, 113.0, 118.4],
  },
};

// The site's graphite palette: the citrus accent for the lead series.
const CITRUS = "#c8f26d";
const SLATE = "#8d9198";

function change(values: number[]): string {
  const delta = values[values.length - 1] - values[0];
  return `${delta >= 0 ? "+" : ""}${delta.toFixed(1)}%`;
}

function Chart(props: { range: () => Range }) {
  return (
    <Canvas
      style={{ height: 112, alignSelf: "stretch" }}
      draw={(ctx, size) => {
        const { a, b } = SERIES[props.range()];
        const all = [...a, ...b];
        const min = Math.min(...all) - 0.6;
        const max = Math.max(...all) + 0.6;
        const x = (i: number) => (i / (a.length - 1)) * size.width;
        const y = (v: number) => size.height - ((v - min) / (max - min)) * (size.height - 8) - 4;
        // Baseline at the start value.
        ctx.strokeStyle = "#2c2e33";
        ctx.lineWidth = 1;
        ctx.setLineDash([3, 3]);
        ctx.beginPath();
        ctx.moveTo(0, y(100));
        ctx.lineTo(size.width, y(100));
        ctx.stroke();
        ctx.setLineDash([]);
        const line = (values: number[], color: string, fill: boolean) => {
          ctx.beginPath();
          values.forEach((v, i) => (i === 0 ? ctx.moveTo(x(i), y(v)) : ctx.lineTo(x(i), y(v))));
          ctx.strokeStyle = color;
          ctx.lineWidth = 2;
          ctx.lineJoin = "round";
          ctx.stroke();
          if (fill) {
            ctx.lineTo(size.width, size.height);
            ctx.lineTo(0, size.height);
            ctx.closePath();
            const gradient = ctx.createLinearGradient(0, 0, 0, size.height);
            gradient.addColorStop(0, "rgba(200, 242, 109, 0.18)");
            gradient.addColorStop(1, "rgba(200, 242, 109, 0)");
            ctx.fillStyle = gradient;
            ctx.fill();
          }
          const last = values.length - 1;
          ctx.beginPath();
          ctx.arc(x(last) - 2, y(values[last]), 3, 0, Math.PI * 2);
          ctx.fillStyle = color;
          ctx.fill();
        };
        line(a, SLATE, false);
        line(b, CITRUS, true);
      }}
    />
  );
}

function Legend(props: { color: string; symbol: string; value: () => string; strong?: boolean }) {
  return (
    <View class="flex-1 gap-0.5">
      <View class="flex-row items-center gap-1.5">
        <View style={{ width: 8, height: 8, borderRadius: 4, backgroundColor: props.color }} />
        <Text class="text-xs text-[#8d9198]">{props.symbol}</Text>
      </View>
      <Text class={props.strong ? "text-lg font-semibold text-[#eceef1]" : "text-lg font-semibold text-[#8d9198]"}>
        {props.value()}
      </Text>
    </View>
  );
}

function App() {
  const range = signal<Range>("1W");
  const alert = signal(false);
  const data = computed(() => SERIES[range()]);
  const ranges: Range[] = ["1D", "1W", "1M"];

  return (
    <View
      class="flex-1 px-4 pt-4 pb-4 gap-3.5"
      style={{ backgroundColor: "#18191c", backgroundImage: "linear-gradient(180deg, #1d1e22, #17181b)" }}
    >
      <View class="flex-row items-center justify-between">
        <Text class="text-[15px] font-semibold text-[#eceef1]">NVDA vs AAPL</Text>
        <View
          class="flex-row rounded-[10px] border border-[#1f2124] bg-[#0c0d0f] p-0.5"
          style={{ boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" }}
        >
          <Each of={ranges}>
            {(r) => (
              <View
                onPress={() => range.set(r())}
                class="rounded-[7px] px-2 py-1 transition-colors"
                style={
                  range() === r()
                    ? { backgroundColor: "#24262a", boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)" }
                    : {}
                }
              >
                <Text class={range() === r() ? "text-xs font-semibold text-[#eceef1]" : "text-xs text-[#8d9198]"}>{r()}</Text>
              </View>
            )}
          </Each>
        </View>
      </View>

      <View class="flex-row">
        <Legend color={CITRUS} symbol="NVDA" value={() => change(data().b)} strong />
        <Legend color={SLATE} symbol="AAPL" value={() => change(data().a)} />
      </View>

      <Chart range={range} />

      <View
        onPress={() => alert.update((v) => !v)}
        activeOpacity={0.7}
        class="h-10 flex-row items-center justify-center gap-1.5 rounded-[10px] transition-colors"
        style={
          alert()
            ? { backgroundColor: "rgba(69,212,154,0.14)", backgroundImage: "none", boxShadow: "none" }
            : {
                backgroundColor: "#eceef1",
                backgroundImage: "linear-gradient(180deg, #fbfbfc, #d4d6d9)",
                boxShadow: "inset 0 1px 0 rgba(255,255,255,0.9), 0 3px 10px rgba(0,0,0,0.4)",
              }
        }
      >
        <Text class={alert() ? "text-sm font-semibold text-[#45d49a]" : "text-sm font-semibold text-[#111214]"}>
          {alert() ? "✓  Alert set at +10%" : "Alert me at +10%"}
        </Text>
      </View>
    </View>
  );
}

mount(App);
