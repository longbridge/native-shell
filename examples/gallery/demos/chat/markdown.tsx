import { computed, type JSX, Scroll, Text, View } from "nsh";
import { colors, finish, mono, Press, radius } from "../../ui";
import { Glyph } from "./glyphs";

/**
 * A small Markdown subset for replies: headings, paragraphs (bold, italic,
 * inline code), bullet and numbered lists, fenced code, tables and quotes.
 *
 * Blocks are keyed by position and kind, and a block whose source did not
 * change is the same object as before, so while a reply streams only the
 * block being written renders again.
 *
 * Every block (and each list item and table cell) is one selectable text:
 * a long press highlights it and offers Copy.
 */
export type Block =
  | { key: string; src: string; type: "h"; level: number; text: string }
  | { key: string; src: string; type: "p"; text: string }
  | { key: string; src: string; type: "ul" | "ol"; items: string[] }
  | { key: string; src: string; type: "code"; lang: string; text: string; open: boolean }
  | { key: string; src: string; type: "table"; head: string[]; rows: string[][] }
  | { key: string; src: string; type: "quote"; text: string };

const cells = (line: string) =>
  line
    .trim()
    .replace(/^\||\|$/g, "")
    .split("|")
    .map((c) => c.trim());

/**
 * Closes the emphasis a streaming text leaves open (`**bo` renders as bold
 * so far) and drops markers with nothing after them yet.
 */
export function complete(text: string): string {
  if ((text.match(/`/g) ?? []).length % 2) return text.endsWith("`") ? text.slice(0, -1) : `${text}\``;
  const outside = text.replace(/`[^`]*`/g, "");
  let close = "";
  if ((outside.replace(/\*\*/g, "").match(/\*/g) ?? []).length % 2) {
    if (text.endsWith("*") && !text.endsWith("**")) text = text.slice(0, -1);
    else close = "*";
  }
  if ((outside.match(/\*\*/g) ?? []).length % 2) {
    if (text.endsWith("**")) text = text.slice(0, -2);
    else close += "**";
  }
  return text + close;
}

/**
 * Parses `md`; blocks equal to one in `previous` are reused as they are.
 * `streaming` holds back a line that cannot be told apart yet (a half-typed
 * table row, a bare list or heading marker, the start of a fence) and
 * completes the open emphasis of the last block.
 */
export function parse(md: string, previous?: readonly Block[], streaming = false): Block[] {
  const lines = md.split("\n");
  if (streaming && /^(\|.*|#{1,3}|[-*]|\d+\.?|`{1,2})$/.test(lines[lines.length - 1])) lines.pop();
  const out: Block[] = [];
  const reuse = new Map<string, Block>();
  for (const b of previous ?? []) reuse.set(b.key, b);
  const push = (src: string, make: (key: string) => Block) => {
    const key = `${out.length}:${make("").type}`;
    const old = reuse.get(key);
    out.push(old && old.src === src ? old : make(key));
  };
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i++;
      continue;
    }
    if (line.startsWith("```")) {
      const lang = line.slice(3).trim();
      const body: string[] = [];
      i++;
      while (i < lines.length && !lines[i].startsWith("```")) body.push(lines[i++]);
      const open = i >= lines.length;
      i++;
      const src = `${lang}\n${body.join("\n")}${open ? "" : "\n```"}`;
      push(src, (key) => ({ key, src, type: "code", lang, text: body.join("\n"), open }));
      continue;
    }
    const heading = /^(#{1,3}) (.*)$/.exec(line);
    if (heading) {
      i++;
      push(line, (key) => ({ key, src: line, type: "h", level: heading[1].length, text: heading[2] }));
      continue;
    }
    const run = (test: RegExp) => {
      const group: string[] = [];
      while (i < lines.length && test.test(lines[i])) group.push(lines[i++]);
      return group;
    };
    if (line.startsWith(">")) {
      const group = run(/^>/);
      const src = group.join("\n");
      push(src, (key) => ({ key, src, type: "quote", text: group.map((l) => l.replace(/^>\s?/, "")).join(" ") }));
      continue;
    }
    if (line.startsWith("|")) {
      const group = run(/^\|/);
      const src = group.join("\n");
      const rows = group.filter((l) => !/^\|[\s:|-]+\|?$/.test(l)).map(cells);
      push(src, (key) => ({ key, src, type: "table", head: rows[0] ?? [], rows: rows.slice(1) }));
      continue;
    }
    if (/^[-*] /.test(line)) {
      const group = run(/^[-*] /);
      const src = group.join("\n");
      push(src, (key) => ({ key, src, type: "ul", items: group.map((l) => l.slice(2)) }));
      continue;
    }
    if (/^\d+\. /.test(line)) {
      const group = run(/^\d+\. /);
      const src = group.join("\n");
      push(src, (key) => ({ key, src, type: "ol", items: group.map((l) => l.replace(/^\d+\. /, "")) }));
      continue;
    }
    const group = run(/^(?!\s*$|```|#{1,3} |>|\||[-*] |\d+\. )/);
    const src = group.join("\n");
    push(src, (key) => ({ key, src, type: "p", text: group.join(" ") }));
  }
  const last = out[out.length - 1];
  if (streaming && last) {
    // A new object, so the block reused next frame is the raw one.
    if (last.type === "p" || last.type === "h" || last.type === "quote") out[out.length - 1] = { ...last, src: `~${last.src}`, text: complete(last.text) };
    else if (last.type === "ul" || last.type === "ol")
      out[out.length - 1] = { ...last, src: `~${last.src}`, items: [...last.items.slice(0, -1), complete(last.items[last.items.length - 1])] };
  }
  return out;
}

/** Plain text of a reply (for copying). */
export const plain = (md: string) => md.replace(/[*`#>|]/g, "").replace(/\n{3,}/g, "\n\n");

const body = () => ({ fontSize: 15, lineHeight: 22, color: colors.text }) as const;

/** Bold, italic and inline code as spans. */
function inline(text: string): JSX.Element[] {
  return text
    .split(/(\*\*[^*]+\*\*|\*[^*\s][^*]*\*|`[^`]+`)/)
    .filter((s) => s)
    .map((s) => {
      if (s.startsWith("**") && s.endsWith("**") && s.length > 4) return <Text style={{ fontWeight: "600" }}>{s.slice(2, -2)}</Text>;
      if (s.startsWith("`") && s.endsWith("`") && s.length > 2) return <Text style={{ fontFamily: mono, fontSize: 14, color: colors.code }}>{s.slice(1, -1)}</Text>;
      if (s.startsWith("*") && s.endsWith("*") && s.length > 2) return <Text style={{ fontStyle: "italic" }}>{s.slice(1, -1)}</Text>;
      return <Text>{s}</Text>;
    });
}

function Caret(props: { make?: () => JSX.Element }) {
  return props.make ? props.make() : null;
}

/** A fenced code block with its language and a Copy button. */
function CodeBlock(props: { block: () => Extract<Block, { type: "code" }>; copied: boolean; onCopy: () => void; tail: () => JSX.Element }) {
  return (
    <View
      style={{
        backgroundColor: colors.inset,
        ...finish.inset,
        borderRadius: radius.inset,
        borderCurve: "continuous",
        borderWidth: 1,
        borderColor: colors.line,
        overflow: "hidden",
      }}
    >
      <View style={{ flexDirection: "row", alignItems: "center", height: 40, paddingLeft: 12, borderBottomWidth: 1, borderBottomColor: colors.line }}>
        <Text style={{ flex: 1, fontSize: 12, color: colors.muted, fontFamily: mono }}>{props.block().lang || "text"}</Text>
        <Press onPress={props.onCopy} accessibilityLabel="Copy code" hitSlop={4}>
          <View style={{ height: 40, paddingHorizontal: 12, flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Glyph name={props.copied ? "check" : "copy"} size={14} tint={props.copied ? colors.up : colors.muted} />
            <Text style={{ fontSize: 13, color: props.copied ? colors.up : colors.muted }}>{props.copied ? "Copied" : "Copy"}</Text>
          </View>
        </Press>
      </View>
      <Scroll horizontal showsIndicator={false}>
        <View style={{ padding: 12 }}>
          <Text selectable style={{ fontFamily: mono, fontSize: 13, lineHeight: 19, color: colors.code }}>
            {props.block().text}
            {props.tail()}
          </Text>
        </View>
      </Scroll>
    </View>
  );
}

function Table(props: { block: () => Extract<Block, { type: "table" }> }) {
  return (
    <View style={{ borderRadius: radius.control, borderWidth: 1, borderColor: colors.border, overflow: "hidden" }}>
      <View style={{ flexDirection: "row", backgroundColor: colors.raised, ...finish.raised }}>
        <Each of={props.block().head}>
          {(cell) => <Text style={{ flex: 1, paddingHorizontal: 10, paddingVertical: 8, fontSize: 13, fontWeight: "600", color: colors.muted }}>{cell()}</Text>}
        </Each>
      </View>
      <Each of={props.block().rows}>
        {(row) => (
          <View style={{ flexDirection: "row", borderTopWidth: 1, borderTopColor: colors.line }}>
            <Each of={row()}>
              {(cell) => (
                <Text selectable style={{ flex: 1, paddingHorizontal: 10, paddingVertical: 8, fontSize: 14, color: colors.text, fontVariant: ["tabular-nums"] }}>
                  {inline(cell())}
                </Text>
              )}
            </Each>
          </View>
        )}
      </Each>
    </View>
  );
}

/**
 * Renders blocks. `caret` (while streaming) is drawn at the end of the
 * last block, inline with its text; `copied` names the copied code block.
 */
export function Markdown(props: {
  blocks: readonly Block[];
  id: string;
  caret?: () => JSX.Element;
  copied?: string | null;
  onCopy?: (key: string, text: string) => void;
}) {
  return (
    <View style={{ gap: 12 }}>
      <Each of={props.blocks} key={(b) => b.key}>
        {(block, index) => {
          // A memo: the caret is made again only when the last block changes.
          const isLast = computed(() => !!props.caret && index() === props.blocks.length - 1);
          const tail = () => (
            <If when={isLast()}>
              <Caret make={props.caret} />
            </If>
          );
          switch (block().type) {
            case "h": {
              const b = block as () => Extract<Block, { type: "h" }>;
              return (
                <Text selectable style={{ fontSize: b().level === 1 ? 20 : b().level === 2 ? 17 : 15, lineHeight: b().level === 1 ? 26 : 22, fontWeight: "600", color: colors.text, paddingTop: 4 }}>
                  <Text>{inline(b().text)}</Text>
                  {tail()}
                </Text>
              );
            }
            case "p": {
              const b = block as () => Extract<Block, { type: "p" }>;
              return (
                <Text selectable style={body()}>
                  <Text>{inline(b().text)}</Text>
                  {tail()}
                </Text>
              );
            }
            case "ul":
            case "ol": {
              const b = block as () => Extract<Block, { type: "ul" | "ol" }>;
              return (
                <View style={{ gap: 6 }}>
                  {/* By position: an item being written keeps its row. */}
                  <Each of={b().items.map((_, n) => n)}>
                    {(i) => (
                      <View style={{ flexDirection: "row", gap: 10 }}>
                        <If
                          when={b().type === "ol"}
                          else={<View style={{ width: 16, height: 22, alignItems: "center", justifyContent: "center" }}><View style={{ width: 5, height: 5, borderRadius: 2.5, backgroundColor: colors.muted }} /></View>}
                        >
                          <Text style={{ width: 16, fontSize: 15, lineHeight: 22, color: colors.muted, fontVariant: ["tabular-nums"] }}>{i() + 1}.</Text>
                        </If>
                        <Text selectable style={[body(), { flex: 1 }]}>
                          <Text>{inline(b().items[i()] ?? "")}</Text>
                          <If when={i() === b().items.length - 1}>{tail()}</If>
                        </Text>
                      </View>
                    )}
                  </Each>
                </View>
              );
            }
            case "code": {
              const b = block as () => Extract<Block, { type: "code" }>;
              const key = () => `${props.id}:${b().key}`;
              return <CodeBlock block={b} tail={tail} copied={props.copied === key()} onCopy={() => props.onCopy?.(key(), b().text)} />;
            }
            case "table": {
              const b = block as () => Extract<Block, { type: "table" }>;
              return <Table block={b} />;
            }
            case "quote": {
              const b = block as () => Extract<Block, { type: "quote" }>;
              return (
                <View style={{ flexDirection: "row", gap: 12 }}>
                  <View style={{ width: 3, borderRadius: 1.5, backgroundColor: colors.border }} />
                  <Text selectable style={[body(), { flex: 1, color: colors.muted }]}>
                    <Text>{inline(b().text)}</Text>
                    {tail()}
                  </Text>
                </View>
              );
            }
          }
        }}
      </Each>
    </View>
  );
}
