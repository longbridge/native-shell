import { type GestureResponderEvent, Scroll, signal, Text, View } from "nsh";
import { createNativeValue, Easing } from "nsh/animated";
import { Canvas, Path2D } from "nsh/canvas";
import logo from "../logo@2x.png";
import { Button, colors, Section } from "../ui";

const TAU = Math.PI * 2;

/** A ring whose sweep is a native value: animating it redraws natively. */
function ProgressRing() {
  const progress = createNativeValue(0.3);
  const target = signal(0.3);
  const go = (to: number) => {
    target.set(to);
    progress.timing(to, { duration: 900, easing: Easing.inOut(Easing.cubic) });
  };
  return (
    <Section title="Native animation (progress ring)">
      <View style={{ flexDirection: "row", alignItems: "center", gap: 16 }}>
        <Canvas
          style={{ width: 120, height: 120 }}
          draw={(ctx, { width, height }) => {
            const cx = width / 2, cy = height / 2, r = width / 2 - 10;
            ctx.lineWidth = 12;
            ctx.lineCap = "round";
            ctx.strokeStyle = "#e5e7eb";
            ctx.beginPath();
            ctx.arc(cx, cy, r, 0, TAU);
            ctx.stroke();
            const g = ctx.createConicGradient(-Math.PI / 2, cx, cy);
            g.addColorStop(0, "#22d3ee");
            g.addColorStop(1, "#6366f1");
            ctx.strokeStyle = g;
            ctx.beginPath();
            // The end angle reads the native value on every frame.
            ctx.arc(cx, cy, r, -Math.PI / 2, progress.mul(TAU).sub(Math.PI / 2));
            ctx.stroke();
            ctx.fillStyle = colors.text;
            ctx.font = "600 20px system-ui";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText(`${Math.round(target() * 100)}%`, cx, cy);
          }}
        />
        <View style={{ gap: 8 }}>
          <Button label="Random" onPress={() => go(Math.round(Math.random() * 100) / 100)} />
          <Button label="Full" color="#1a7f37" onPress={() => go(1)} />
        </View>
      </View>
    </Section>
  );
}

/** A chart: gradient area, a line, labels measured with measureText. */
function Chart() {
  const data = signal([12, 30, 22, 48, 36, 60, 52, 75, 64, 88]);
  return (
    <Section title="Chart">
      <Canvas
        style={{ height: 180 }}
        draw={(ctx, { width, height }) => {
          const values = data();
          const pad = { left: 32, right: 12, top: 12, bottom: 24 };
          const w = width - pad.left - pad.right, h = height - pad.top - pad.bottom;
          const max = 100;
          const x = (i: number) => pad.left + (i / (values.length - 1)) * w;
          const y = (v: number) => pad.top + h - (v / max) * h;
          ctx.font = "11px system-ui";
          ctx.fillStyle = colors.muted;
          ctx.strokeStyle = "#e5e7eb";
          ctx.lineWidth = 1;
          for (const v of [0, 25, 50, 75, 100]) {
            ctx.beginPath();
            ctx.moveTo(pad.left, y(v));
            ctx.lineTo(width - pad.right, y(v));
            ctx.stroke();
            const label = String(v);
            ctx.fillText(label, pad.left - 6 - ctx.measureText(label).width, y(v) + 4);
          }
          const area = ctx.createLinearGradient(0, pad.top, 0, pad.top + h);
          area.addColorStop(0, "rgba(59,130,246,0.35)");
          area.addColorStop(1, "rgba(59,130,246,0)");
          ctx.beginPath();
          ctx.moveTo(x(0), y(values[0]));
          values.forEach((v, i) => i > 0 && ctx.lineTo(x(i), y(v)));
          ctx.lineTo(x(values.length - 1), pad.top + h);
          ctx.lineTo(x(0), pad.top + h);
          ctx.closePath();
          ctx.fillStyle = area;
          ctx.fill();
          ctx.beginPath();
          values.forEach((v, i) => (i === 0 ? ctx.moveTo(x(i), y(v)) : ctx.lineTo(x(i), y(v))));
          ctx.strokeStyle = "#3b82f6";
          ctx.lineWidth = 2.5;
          ctx.lineJoin = "round";
          ctx.stroke();
          ctx.fillStyle = "#ffffff";
          ctx.strokeStyle = "#3b82f6";
          ctx.lineWidth = 2;
          values.forEach((v, i) => {
            ctx.beginPath();
            ctx.arc(x(i), y(v), 3.5, 0, TAU);
            ctx.fill();
            ctx.stroke();
          });
        }}
      />
      <Button label="New data" onPress={() => data.set(data().map(() => Math.round(10 + Math.random() * 85)))} />
    </Section>
  );
}

/** A clock: transforms, save / restore, a spinning second hand natively. */
function Clock() {
  const seconds = createNativeValue(0);
  // One turn a minute, natively (no JS per frame).
  const spin = async () => {
    for (;;) {
      seconds.set(0);
      await seconds.timing(60, { duration: 60_000, easing: Easing.linear });
    }
  };
  spin();
  return (
    <Section title="Transforms (a clock)">
      <Canvas
        style={{ width: 160, height: 160, alignSelf: "center" }}
        draw={(ctx, { width }) => {
          const r = width / 2;
          ctx.translate(r, r);
          ctx.fillStyle = "#f8fafc";
          ctx.strokeStyle = "#334155";
          ctx.lineWidth = 3;
          ctx.beginPath();
          ctx.arc(0, 0, r - 4, 0, TAU);
          ctx.fill();
          ctx.stroke();
          for (let i = 0; i < 60; i++) {
            ctx.save();
            ctx.rotate((i / 60) * TAU);
            ctx.lineWidth = i % 5 === 0 ? 3 : 1;
            ctx.beginPath();
            ctx.moveTo(r - 10, 0);
            ctx.lineTo(r - (i % 5 === 0 ? 20 : 14), 0);
            ctx.stroke();
            ctx.restore();
          }
          const hand = (angle: number, length: number, width: number, color: string) => {
            ctx.save();
            ctx.rotate(angle - Math.PI / 2);
            ctx.strokeStyle = color;
            ctx.lineWidth = width;
            ctx.lineCap = "round";
            ctx.beginPath();
            ctx.moveTo(-8, 0);
            ctx.lineTo(length, 0);
            ctx.stroke();
            ctx.restore();
          };
          hand((10 / 12) * TAU, r * 0.5, 5, "#0f172a");
          hand((10 / 60) * TAU, r * 0.72, 3.5, "#0f172a");
          ctx.save();
          // Degrees of a turn per second, as a native rotation.
          ctx.rotate(seconds.mul(TAU / 60));
          ctx.rotate(-Math.PI / 2);
          ctx.strokeStyle = "#ef4444";
          ctx.lineWidth = 1.5;
          ctx.beginPath();
          ctx.moveTo(-14, 0);
          ctx.lineTo(r * 0.8, 0);
          ctx.stroke();
          ctx.restore();
          ctx.fillStyle = "#ef4444";
          ctx.beginPath();
          ctx.arc(0, 0, 4, 0, TAU);
          ctx.fill();
        }}
      />
    </Section>
  );
}

const icons: [string, string][] = [
  ["heart", "M12 21s-7-4.35-9.5-8.5C.5 8.5 3 4 7 4c2 0 3.5 1 5 3 1.5-2 3-3 5-3 4 0 6.5 4.5 4.5 8.5C19 16.65 12 21 12 21z"],
  ["star", "M12 2l3.09 6.26L22 9.27l-5 4.87L18.18 21 12 17.27 5.82 21 7 14.14l-5-4.87 6.91-1.01z"],
  ["bolt", "M13 2L3 14h7l-1 8 10-12h-7z"],
  ["check", "M20 6L9 17l-5-5"],
];

/** SVG path data through Path2D: icons that take the text color. */
function Icons() {
  return (
    <Section title="Path2D (SVG path data)">
      <View style={{ flexDirection: "row", gap: 16 }}>
        <Each of={icons}>
          {(icon, i) => {
            const [name, d] = icon();
            const path = new Path2D(d);
            return (
              <View style={{ alignItems: "center", gap: 4 }}>
                <Canvas
                  style={{ width: 40, height: 40 }}
                  draw={(ctx) => {
                    ctx.scale(40 / 24, 40 / 24);
                    const color = ["#ef4444", "#f59e0b", "#8b5cf6", "#10b981"][i()];
                    if (name === "check") {
                      ctx.strokeStyle = color;
                      ctx.lineWidth = 2.5;
                      ctx.lineCap = "round";
                      ctx.lineJoin = "round";
                      ctx.stroke(path);
                    } else {
                      ctx.fillStyle = color;
                      ctx.fill(path);
                    }
                  }}
                />
                <Text style={{ fontSize: 12, color: colors.muted }}>{name}</Text>
              </View>
            );
          }}
        </Each>
      </View>
    </Section>
  );
}

/** Images, compositing, shadows, dashes. */
function Compositing() {
  return (
    <Section title="Images, compositing, shadows">
      <Canvas
        style={{ height: 130 }}
        draw={(ctx, { width }) => {
          // An image clipped to a circle.
          ctx.save();
          ctx.beginPath();
          ctx.arc(60, 60, 48, 0, TAU);
          ctx.clip();
          ctx.drawImage(logo, 12, 12, 96, 96);
          ctx.restore();
          // A shadowed card with a cut-out hole (destination-out).
          ctx.save();
          ctx.shadowColor = "rgba(0,0,0,0.25)";
          ctx.shadowBlur = 12;
          ctx.shadowOffsetY = 4;
          ctx.fillStyle = "#6366f1";
          ctx.beginPath();
          ctx.roundRect(130, 16, 110, 90, 14);
          ctx.fill();
          ctx.restore();
          ctx.globalCompositeOperation = "destination-out";
          ctx.beginPath();
          ctx.arc(185, 61, 22, 0, TAU);
          ctx.fill();
          ctx.globalCompositeOperation = "source-over";
          // Dashes and a radial gradient.
          const g = ctx.createRadialGradient(width - 50, 50, 4, width - 50, 50, 44);
          g.addColorStop(0, "#fde68a");
          g.addColorStop(1, "#f97316");
          ctx.fillStyle = g;
          ctx.beginPath();
          ctx.arc(width - 50, 60, 40, 0, TAU);
          ctx.fill();
          ctx.setLineDash([6, 5]);
          ctx.strokeStyle = "#9a3412";
          ctx.lineWidth = 2;
          ctx.stroke();
        }}
      />
    </Section>
  );
}

/** Touch drawing: each move draws again from JS. */
function Sketch() {
  const strokes = signal<[number, number][][]>([]);
  const point = (e: GestureResponderEvent): [number, number] => [e.locationX, e.locationY];
  return (
    <Section title="Sketch (touch → draw)">
      <Canvas
        style={{ height: 180, backgroundColor: "#f8fafc", borderRadius: 10 }}
        onStartShouldSetResponder={() => true}
        onResponderGrant={(e) => {
          strokes.set([...strokes(), [point(e)]]);
          // Keep the touches: the scroll view around must not take them.
          return true;
        }}
        onResponderTerminationRequest={() => false}
        onResponderMove={(e) => {
          const all = strokes();
          const last = all[all.length - 1];
          strokes.set([...all.slice(0, -1), [...last, point(e)]]);
        }}
        draw={(ctx) => {
          ctx.strokeStyle = "#0f172a";
          ctx.lineWidth = 3;
          ctx.lineCap = "round";
          ctx.lineJoin = "round";
          for (const s of strokes()) {
            ctx.beginPath();
            s.forEach(([x, y], i) => (i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y)));
            ctx.stroke();
          }
        }}
      />
      <Button label="Clear" color="#57606a" onPress={() => strokes.set([])} />
    </Section>
  );
}

export function CanvasDemo() {
  return (
    <Scroll style={{ flex: 1, backgroundColor: colors.background }}>
      <View style={{ padding: 16, gap: 20 }}>
        <ProgressRing />
        <Chart />
        <Clock />
        <Icons />
        <Compositing />
        <Sketch />
      </View>
    </Scroll>
  );
}
