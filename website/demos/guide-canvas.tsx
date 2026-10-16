import { type GestureResponderEvent, mount, signal, Text, View } from "nsh";
import { createNativeValue, Easing } from "nsh/animated";
import { Canvas, Path2D } from "nsh/canvas";

const TAU = Math.PI * 2;
const heart = new Path2D("M12 21s-7-4.35-9.5-8.5C.5 8.5 3 4 7 4c2 0 3.5 1 5 3 1.5-2 3-3 5-3 4 0 6.5 4.5 4.5 8.5C19 16.65 12 21 12 21z");

function Button(props: { label: string; onPress: () => void }) {
  return (
    <View
      onPress={props.onPress}
      activeOpacity={0.6}
      style={{
        paddingHorizontal: 14,
        paddingVertical: 9,
        borderRadius: 10,
        borderWidth: 1,
        borderColor: "#2c2e33",
        backgroundColor: "#24262a",
        backgroundImage: "linear-gradient(180deg, #2d2f34, #222428)",
        boxShadow: "inset 0 1px 0 rgba(255,255,255,0.08), 0 1px 2px rgba(0,0,0,0.45)",
      }}
    >
      <Text style={{ color: "#eceef1", fontWeight: "600" }}>{props.label}</Text>
    </View>
  );
}

function App() {
  // The ring's sweep is a native value: animating it redraws without running `draw`.
  const progress = createNativeValue(0.35);
  const target = signal(0.35);
  const go = () => {
    const to = Math.round(Math.random() * 100) / 100;
    target.set(to);
    progress.timing(to, { duration: 800, easing: Easing.inOut(Easing.cubic) });
  };

  // The chart reads a signal: changing it runs `draw` again.
  const data = signal([12, 30, 22, 48, 36, 60, 52, 75]);

  // Finger drawing through the responder system.
  const strokes = signal<[number, number][][]>([]);
  const point = (e: GestureResponderEvent): [number, number] => [e.locationX, e.locationY];

  return (
    <View style={{ flex: 1, backgroundColor: "#101113", padding: 16, paddingTop: 24, gap: 14 }}>
      <View style={{ flexDirection: "row", alignItems: "center", gap: 16 }}>
        <Canvas
          style={{ width: 110, height: 110 }}
          draw={(ctx, { width, height }) => {
            const cx = width / 2;
            const cy = height / 2;
            const r = width / 2 - 8;
            ctx.lineWidth = 10;
            ctx.lineCap = "round";
            ctx.strokeStyle = "#24262a";
            ctx.beginPath();
            ctx.arc(cx, cy, r, 0, TAU);
            ctx.stroke();
            const g = ctx.createConicGradient(-Math.PI / 2, cx, cy);
            g.addColorStop(0, "#45d49a");
            g.addColorStop(1, "#c8f26d");
            ctx.strokeStyle = g;
            ctx.beginPath();
            ctx.arc(cx, cy, r, -Math.PI / 2, progress.mul(TAU).sub(Math.PI / 2));
            ctx.stroke();
            ctx.fillStyle = "#eceef1";
            ctx.font = "600 18px system-ui";
            ctx.textAlign = "center";
            ctx.textBaseline = "middle";
            ctx.fillText(`${Math.round(target() * 100)}%`, cx, cy);
          }}
        />
        <View style={{ gap: 10 }}>
          <Button label="Animate the ring" onPress={go} />
          <Canvas
            style={{ width: 48, height: 48 }}
            draw={(ctx) => {
              ctx.scale(2, 2);
              ctx.shadowColor = "rgba(255,97,112,0.45)";
              ctx.shadowBlur = 4;
              ctx.fillStyle = "#ff6170";
              ctx.fill(heart);
            }}
          />
        </View>
      </View>

      <Canvas
        style={{ height: 150, borderRadius: 12, borderWidth: 1, borderColor: "#1f2124", backgroundColor: "#0c0d0f", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" }}
        draw={(ctx, { width, height }) => {
          const values = data();
          const pad = 16;
          const x = (i: number) => pad + (i / (values.length - 1)) * (width - 2 * pad);
          const y = (v: number) => height - pad - (v / 100) * (height - 2 * pad);
          const area = ctx.createLinearGradient(0, pad, 0, height - pad);
          area.addColorStop(0, "rgba(200,242,109,0.28)");
          area.addColorStop(1, "rgba(200,242,109,0)");
          ctx.beginPath();
          values.forEach((v, i) => (i === 0 ? ctx.moveTo(x(i), y(v)) : ctx.lineTo(x(i), y(v))));
          ctx.lineTo(x(values.length - 1), height - pad);
          ctx.lineTo(x(0), height - pad);
          ctx.closePath();
          ctx.fillStyle = area;
          ctx.fill();
          ctx.beginPath();
          values.forEach((v, i) => (i === 0 ? ctx.moveTo(x(i), y(v)) : ctx.lineTo(x(i), y(v))));
          ctx.strokeStyle = "#c8f26d";
          ctx.lineWidth = 2.5;
          ctx.lineJoin = "round";
          ctx.stroke();
        }}
      />
      <Button label="New data" onPress={() => data.set(data().map(() => Math.round(10 + Math.random() * 85)))} />

      <Canvas
        style={{ flex: 1, borderRadius: 12, borderWidth: 1, borderColor: "#1f2124", backgroundColor: "#0c0d0f", boxShadow: "inset 0 1px 3px rgba(0,0,0,0.55)" }}
        onStartShouldSetResponder={() => true}
        onResponderGrant={(e) => {
          strokes.set([...strokes(), [point(e)]]);
          return true;
        }}
        onResponderMove={(e) => {
          const all = strokes();
          strokes.set([...all.slice(0, -1), [...all[all.length - 1], point(e)]]);
        }}
        draw={(ctx, { width }) => {
          ctx.fillStyle = "#5a5e65";
          ctx.font = "13px system-ui";
          ctx.textAlign = "center";
          ctx.fillText("Draw here", width / 2, 20);
          ctx.strokeStyle = "#eceef1";
          ctx.lineWidth = 3;
          ctx.lineCap = "round";
          ctx.lineJoin = "round";
          for (const s of strokes()) {
            ctx.beginPath();
            s.forEach(([px, py], i) => (i === 0 ? ctx.moveTo(px, py) : ctx.lineTo(px, py)));
            ctx.stroke();
          }
        }}
      />
    </View>
  );
}

mount(App);
