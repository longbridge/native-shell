import { signal } from "nsh";
import { Canvas } from "nsh/canvas";
import { nitrogenTank, quads } from "../store";
import { color, mono } from "../theme";
import { Gauge, Inset, Line, Panel, Pill, track } from "../ui";

const SIZE = 240;
const C = SIZE / 2;
const HULL = 46;
const ARM = 78;

const at = (angle: number, r: number) => {
  const a = (angle * Math.PI) / 180;
  return [C + Math.cos(a) * r, C + Math.sin(a) * r] as const;
};

/** The lander from above: hull, legs and the four RCS quads with their
 * nozzles. A degraded quad is rose; tap a quad to select it. */
function Plan(props: { selected: string | null; onPick: (id: string | null) => void }) {
  const size = signal(0);
  return (
    <Canvas
      style={{ width: SIZE, height: SIZE, alignSelf: "center" }}
      onLayout={(e) => size.set(e.width)}
      {...track((px, py, first) => {
        if (!first || size() <= 0) return;
        const k = SIZE / size();
        const hit = quads.find((q) => {
          const [x, y] = at(q.angle, ARM);
          return Math.hypot(px * k - x, py * k - y) < 26;
        });
        props.onPick(hit?.id ?? null);
      })}
      accessibilityLabel="Top view: four RCS quads around the hull, Q3 degraded"
      draw={(ctx, { width }) => {
        ctx.scale(width / SIZE, width / SIZE);
        // Range rings.
        ctx.strokeStyle = color.grid;
        ctx.lineWidth = 1;
        for (const r of [40, 80, 112]) {
          ctx.beginPath();
          ctx.arc(C, C, r, 0, Math.PI * 2);
          ctx.stroke();
        }
        // Legs between the quads.
        ctx.strokeStyle = color.lineStrong;
        ctx.lineWidth = 3;
        ctx.lineCap = "round";
        for (const a of [0, 90, 180, 270]) {
          const [x0, y0] = at(a, HULL);
          const [x1, y1] = at(a, 108);
          ctx.beginPath();
          ctx.moveTo(x0, y0);
          ctx.lineTo(x1, y1);
          ctx.stroke();
          ctx.beginPath();
          ctx.arc(x1, y1, 5, 0, Math.PI * 2);
          ctx.stroke();
        }
        // Hull and engine.
        ctx.strokeStyle = "#c8d1da";
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        ctx.arc(C, C, HULL, 0, Math.PI * 2);
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(C, C, 16, 0, Math.PI * 2);
        ctx.stroke();
        ctx.fillStyle = color.faint;
        ctx.font = `10px ${mono}`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("FWD", C, C - HULL - 10);

        for (const q of quads) {
          const [x, y] = at(q.angle, ARM);
          const weak = q.delta < 0;
          const on = props.selected === q.id;
          const tint = weak ? color.alert : on ? color.accent : color.muted;
          // Strut from the hull.
          const [sx, sy] = at(q.angle, HULL);
          ctx.strokeStyle = color.lineStrong;
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.moveTo(sx, sy);
          ctx.lineTo(x, y);
          ctx.stroke();
          // Three nozzles fanning out, longer while selected.
          ctx.strokeStyle = tint;
          ctx.lineWidth = on ? 2.5 : 1.5;
          ctx.setLineDash(weak ? [3, 3] : []);
          for (const spread of [-40, 0, 40]) {
            const [ex, ey] = at(q.angle + spread * 0.35, ARM + (on ? 30 : 20));
            ctx.beginPath();
            ctx.moveTo(x, y);
            ctx.lineTo(ex + (x - C) * 0.08, ey + (y - C) * 0.08);
            ctx.stroke();
          }
          ctx.setLineDash([]);
          ctx.fillStyle = on ? color.accentDim : color.card;
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.roundRect(x - 13, y - 13, 26, 26, 6);
          ctx.fill();
          ctx.stroke();
          ctx.fillStyle = tint;
          ctx.font = `10px ${mono}`;
          ctx.fillText(q.id, x, y);
        }
      }}
    />
  );
}

export function Rcs() {
  const selected = signal<string | null>(null);
  const percent = Math.round((nitrogenTank.pressure / nitrogenTank.capacity) * 100);
  return (
    <Panel icon="rcs" title="RCS" status="Cold nitrogen · 4 quads · hop 40">
      <Plan selected={selected()} onPick={(id) => selected.set(id)} />
      <view>
        <each of={quads}>
          {(q: () => (typeof quads)[number], i: () => number) => (
            <view
              onPress={() => selected.set(selected() === q().id ? null : q().id)}
              accessibilityRole="button"
              accessibilityState={{ selected: selected() === q().id }}
            >
              <Line first={i() === 0} selected={selected() === q().id}>
                <Pill label={q().id} tone={q().delta < 0 ? "alert" : selected() === q().id ? "accent" : "neutral"} width={44} />
                <text style={{ flex: 1, fontSize: 15, color: color.text }}>{q().name}</text>
                <if when={q().delta < 0}>
                  <text style={{ fontFamily: mono, fontSize: 13, color: color.alert }}>{q().delta}%</text>
                </if>
                <text style={{ fontFamily: mono, fontSize: 13, color: color.text }}>{q().used.toFixed(1)} N·s</text>
              </Line>
            </view>
          )}
        </each>
      </view>
      <Inset>
        <view style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "baseline" }}>
          <text style={{ fontSize: 14, color: color.text }}>Nitrogen bottle</text>
          <text style={{ fontFamily: mono, fontSize: 13, color: color.muted }}>{percent}%</text>
        </view>
        <Gauge value={nitrogenTank.pressure / nitrogenTank.capacity} />
        <text style={{ fontSize: 13, color: color.muted }}>
          <text style={{ fontFamily: mono, color: color.text }}>{nitrogenTank.pressure}</text> of {nitrogenTank.capacity} bar · about{" "}
          {nitrogenTank.hopsLeft} hops left
        </text>
      </Inset>
    </Panel>
  );
}
