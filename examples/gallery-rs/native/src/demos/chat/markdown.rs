//! A small Markdown subset for replies (as the TSX gallery's
//! `demos/chat/markdown.tsx`): headings, paragraphs (bold, italic, inline
//! code), bullet and numbered lists, fenced code, tables and quotes.
//!
//! Each block is a [`MarkdownBlock`] given its block as props, keyed by
//! position and kind: while a reply streams, only the block being written
//! changes, so only it renders again. Every block (and each list item and
//! table cell) is one selectable text: a long press highlights it and
//! offers Copy.

use nsh_core::ui::{AnyElement, JsObject, PropEnum, prelude::*};

use super::{
    glyphs::{GlyphName, glyph},
    parts::caret,
};
use crate::ui::{Finish, Finished, MONO, PressProps, colors, press_with, radius};

/// What a block is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PropEnum)]
pub(crate) enum Kind {
    H,
    #[default]
    P,
    Ul,
    Ol,
    Code,
    Table,
    Quote,
}

/// A block of a reply.
#[derive(Debug, Clone, Default, PartialEq, JsObject)]
pub(crate) struct Block {
    /// Its place and kind (`2:p`).
    pub(crate) key: SharedString,
    pub(crate) kind: Kind,
    /// A heading's level (1 to 3).
    pub(crate) level: u32,
    /// A heading's, paragraph's, quote's or code block's text.
    pub(crate) text: SharedString,
    /// A list's items.
    pub(crate) items: Vec<SharedString>,
    /// A code block's language.
    pub(crate) lang: SharedString,
    /// A table's header and rows.
    pub(crate) head: Vec<SharedString>,
    pub(crate) rows: Vec<Vec<SharedString>>,
}

fn cells(line: &str) -> Vec<SharedString> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    line.split('|')
        .map(|c| c.trim().to_owned().into())
        .collect()
}

/// A heading's level and text (`## Title`).
fn heading(line: &str) -> Option<(u32, &str)> {
    let hashes = line.bytes().take_while(|&b| b == b'#').count();
    if (1..=3).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
        Some((hashes as u32, &line[hashes + 1..]))
    } else {
        None
    }
}

fn bullet(line: &str) -> bool {
    line.starts_with("- ") || line.starts_with("* ")
}

/// The text of a numbered item (`1. text`).
fn numbered(line: &str) -> Option<&str> {
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    (digits > 0 && line[digits..].starts_with(". ")).then(|| &line[digits + 2..])
}

/// A line of a paragraph: none of the other blocks starts with it.
fn prose(line: &str) -> bool {
    !(line.trim().is_empty()
        || line.starts_with("```")
        || heading(line).is_some()
        || line.starts_with('>')
        || line.starts_with('|')
        || bullet(line)
        || numbered(line).is_some())
}

/// A line that cannot be told apart yet while it streams: a half-typed
/// table row, a bare list or heading marker, the start of a fence.
fn undecided(line: &str) -> bool {
    let hashes = line.bytes().all(|b| b == b'#') && (1..=3).contains(&line.len());
    let digits = line.trim_end_matches('.');
    let number = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && line.len() - digits.len() <= 1;
    line.starts_with('|')
        || hashes
        || line == "-"
        || line == "*"
        || number
        || line == "`"
        || line == "``"
}

/// Closes the emphasis a streaming text leaves open (`**bo` renders as
/// bold so far) and drops markers with nothing after them yet.
pub(crate) fn complete(text: &str) -> String {
    if text.matches('`').count() % 2 == 1 {
        return match text.strip_suffix('`') {
            Some(t) => t.to_owned(),
            None => format!("{text}`"),
        };
    }
    // Outside inline code.
    let mut outside = String::new();
    let mut inside = false;
    let mut pending = String::new();
    for c in text.chars() {
        if c == '`' {
            if inside {
                pending.clear();
            } else {
                outside.push_str(&pending);
                pending.clear();
            }
            inside = !inside;
        } else if inside {
            pending.push(c);
        } else {
            outside.push(c);
        }
    }
    let mut text = text.to_owned();
    let mut close = String::new();
    if outside.replace("**", "").matches('*').count() % 2 == 1 {
        if text.ends_with('*') && !text.ends_with("**") {
            text.pop();
        } else {
            close.push('*');
        }
    }
    if outside.matches("**").count() % 2 == 1 {
        if text.ends_with("**") {
            text.truncate(text.len() - 2);
        } else {
            close.push_str("**");
        }
    }
    text + &close
}

/// Parses `md`. `streaming` holds back a line that cannot be told apart
/// yet and completes the open emphasis of the last block.
pub(crate) fn parse(md: &str, streaming: bool) -> Vec<Block> {
    let mut lines: Vec<&str> = md.split('\n').collect();
    if streaming && lines.last().is_some_and(|l| undecided(l)) {
        lines.pop();
    }
    let mut out: Vec<Block> = Vec::new();
    let push = |out: &mut Vec<Block>, mut block: Block| {
        block.key = format!("{}:{}", out.len(), kind_name(block.kind)).into();
        out.push(block);
    };
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.trim().is_empty() {
            i += 1;
            continue;
        }
        if let Some(lang) = line.strip_prefix("```") {
            let lang = lang.trim();
            let mut body = Vec::new();
            i += 1;
            while i < lines.len() && !lines[i].starts_with("```") {
                body.push(lines[i]);
                i += 1;
            }
            i += 1;
            push(
                &mut out,
                Block {
                    kind: Kind::Code,
                    lang: lang.to_owned().into(),
                    text: body.join("\n").into(),
                    ..Block::default()
                },
            );
            continue;
        }
        if let Some((level, text)) = heading(line) {
            i += 1;
            push(
                &mut out,
                Block {
                    kind: Kind::H,
                    level,
                    text: text.to_owned().into(),
                    ..Block::default()
                },
            );
            continue;
        }
        let mut run = |test: &dyn Fn(&str) -> bool| {
            let mut group = Vec::new();
            while i < lines.len() && test(lines[i]) {
                group.push(lines[i]);
                i += 1;
            }
            group
        };
        let block = if line.starts_with('>') {
            let group = run(&|l| l.starts_with('>'));
            let text: Vec<&str> = group
                .iter()
                .map(|l| {
                    let l = &l[1..];
                    l.strip_prefix(char::is_whitespace).unwrap_or(l)
                })
                .collect();
            Block {
                kind: Kind::Quote,
                text: text.join(" ").into(),
                ..Block::default()
            }
        } else if line.starts_with('|') {
            let group = run(&|l| l.starts_with('|'));
            let rule = |l: &str| l.len() > 1 && l[1..].chars().all(|c| " \t:|-".contains(c));
            let mut rows: Vec<Vec<SharedString>> = group
                .iter()
                .filter(|l| !rule(l))
                .map(|l| cells(l))
                .collect();
            let head = if rows.is_empty() {
                Vec::new()
            } else {
                rows.remove(0)
            };
            Block {
                kind: Kind::Table,
                head,
                rows,
                ..Block::default()
            }
        } else if bullet(line) {
            let group = run(&bullet);
            Block {
                kind: Kind::Ul,
                items: group.iter().map(|l| l[2..].to_owned().into()).collect(),
                ..Block::default()
            }
        } else if numbered(line).is_some() {
            let group = run(&|l| numbered(l).is_some());
            Block {
                kind: Kind::Ol,
                items: group
                    .iter()
                    .filter_map(|l| numbered(l))
                    .map(|t| t.to_owned().into())
                    .collect(),
                ..Block::default()
            }
        } else {
            let group = run(&prose);
            Block {
                kind: Kind::P,
                text: group.join(" ").into(),
                ..Block::default()
            }
        };
        push(&mut out, block);
    }
    if streaming && let Some(last) = out.last_mut() {
        match last.kind {
            Kind::P | Kind::H | Kind::Quote => last.text = complete(&last.text).into(),
            Kind::Ul | Kind::Ol => {
                if let Some(item) = last.items.last_mut() {
                    *item = complete(item).into();
                }
            }
            _ => {}
        }
    }
    out
}

fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::H => "h",
        Kind::P => "p",
        Kind::Ul => "ul",
        Kind::Ol => "ol",
        Kind::Code => "code",
        Kind::Table => "table",
        Kind::Quote => "quote",
    }
}

/// How a piece of inline text looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Span {
    Plain,
    Bold,
    Italic,
    Code,
}

/// The length of a `**bold**`, `*italic*` or `` `code` `` at the start of
/// `s`, in bytes.
fn emphasis(s: &str) -> Option<(usize, Span)> {
    if let Some(rest) = s.strip_prefix("**") {
        let n = rest.find('*').unwrap_or(0);
        if n > 0 && rest[n..].starts_with("**") {
            return Some((n + 4, Span::Bold));
        }
    }
    if let Some(rest) = s.strip_prefix('*') {
        let first = rest.chars().next();
        if first.is_some_and(|c| c != '*' && !c.is_whitespace())
            && let Some(n) = rest.find('*')
        {
            return Some((n + 2, Span::Italic));
        }
    }
    if let Some(rest) = s.strip_prefix('`')
        && let Some(n) = rest.find('`')
        && n > 0
    {
        return Some((n + 2, Span::Code));
    }
    None
}

/// Bold, italic and inline code as pieces of `text`.
fn pieces(text: &str) -> Vec<(Span, &str)> {
    let mut out = Vec::new();
    let mut plain = 0;
    let mut at = 0;
    while at < text.len() {
        if let Some((len, span)) = emphasis(&text[at..]) {
            if plain < at {
                out.push((Span::Plain, &text[plain..at]));
            }
            let inner = match span {
                Span::Bold => &text[at + 2..at + len - 2],
                _ => &text[at + 1..at + len - 1],
            };
            out.push((span, inner));
            at += len;
            plain = at;
        } else {
            at += text[at..].chars().next().map_or(1, char::len_utf8);
        }
    }
    if plain < text.len() {
        out.push((Span::Plain, &text[plain..]));
    }
    out
}

/// `text` with its emphasis as spans.
pub(crate) fn inline(text: &str) -> Text {
    pieces(text).into_iter().fold(text_root(), |t, (span, s)| {
        let piece = nsh_core::ui::text(s.to_owned());
        t.child(match span {
            Span::Plain => piece,
            Span::Bold => piece.font_weight(FontWeight::SEMIBOLD),
            Span::Italic => piece.italic(),
            Span::Code => piece
                .font_family(MONO)
                .text_size(px(14.0))
                .text_color(colors::CODE),
        })
    })
}

fn text_root() -> Text {
    nsh_core::ui::text("")
}

/// A paragraph's look.
fn body(t: Text) -> Text {
    t.text_size(px(15.0))
        .line_height(px(22.0))
        .text_color(colors::TEXT)
}

/// One block of a reply.
#[derive(Debug, Default)]
pub(crate) struct MarkdownBlock;

/// The props of [`MarkdownBlock`].
#[derive(Debug, Clone, PartialEq, Props)]
pub(crate) struct MarkdownBlockProps {
    pub(crate) block: Block,
    /// The reply it is in (code blocks are copied by `reply:block`).
    pub(crate) id: SharedString,
    /// The streaming caret ends it.
    #[prop(default)]
    pub(crate) caret: bool,
    /// Its code was just copied.
    #[prop(default)]
    pub(crate) copied: bool,
}

/// What a [`MarkdownBlock`] reports.
#[derive(Debug, Clone, PartialEq, Event)]
pub(crate) enum MarkdownEvent {
    /// Its code's Copy button was pressed.
    Copy {
        key: SharedString,
        text: SharedString,
    },
}

impl Component for MarkdownBlock {
    type Props = MarkdownBlockProps;
    type Event = MarkdownEvent;

    fn render(&self, props: &MarkdownBlockProps, cx: &mut RenderCx<'_, Self>) -> impl IntoElement {
        let b = &props.block;
        let tail = |t: Text, on: bool| if on { t.child(caret()) } else { t };
        let paragraph = |t: Text| tail(text_root().selectable(true).child(t), props.caret);
        match b.kind {
            Kind::H => {
                let (size, line) = match b.level {
                    1 => (20.0, 26.0),
                    2 => (17.0, 22.0),
                    _ => (15.0, 22.0),
                };
                view().child(
                    paragraph(inline(&b.text))
                        .text_size(px(size))
                        .line_height(px(line))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(colors::TEXT)
                        .pt_1(),
                )
            }
            Kind::P => view().child(body(paragraph(inline(&b.text)))),
            Kind::Ul | Kind::Ol => {
                let count = b.items.len();
                view()
                    .gap(px(6.0))
                    .children(b.items.iter().enumerate().map(|(i, item)| {
                        let marker: AnyElement = if b.kind == Kind::Ol {
                            nsh_core::ui::text(format!("{}.", i + 1))
                                .w(px(16.0))
                                .text_size(px(15.0))
                                .line_height(px(22.0))
                                .text_color(colors::MUTED)
                                .tabular_nums()
                                .into_element()
                        } else {
                            view()
                                .w(px(16.0))
                                .h(px(22.0))
                                .items_center()
                                .justify_center()
                                .child(view().size(px(5.0)).rounded(px(2.5)).bg(colors::MUTED))
                                .into_element()
                        };
                        view().flex_row().gap(px(10.0)).child(marker).child(
                            body(tail(
                                text_root().selectable(true).child(inline(item)),
                                props.caret && i + 1 == count,
                            ))
                            .flex_1(),
                        )
                    }))
            }
            Kind::Code => {
                let copied = props.copied;
                let key: SharedString = format!("{}:{}", props.id, b.key).into();
                let copy = cx.listener(move |_: &mut Self, _: &(), cx| {
                    let text = cx.props().block.text.clone();
                    cx.emit(MarkdownEvent::Copy {
                        key: key.clone(),
                        text,
                    });
                });
                let tone = if copied { colors::UP } else { colors::MUTED };
                view()
                    .bg(colors::INSET)
                    .finish(Finish::Inset)
                    .rounded(px(radius::INSET))
                    .border_curve(BorderCurve::Continuous)
                    .border_1()
                    .border_color(colors::LINE)
                    .overflow_hidden()
                    .child(
                        view()
                            .flex_row()
                            .items_center()
                            .h(px(40.0))
                            .pl_3()
                            .border_b_1()
                            .border_b_color(colors::LINE)
                            .child(
                                nsh_core::ui::text(if b.lang.is_empty() {
                                    SharedString::from("text")
                                } else {
                                    b.lang.clone()
                                })
                                .flex_1()
                                .text_size(px(12.0))
                                .text_color(colors::MUTED)
                                .font_family(MONO),
                            )
                            .child(
                                press_with(
                                    PressProps {
                                        label: Some("Copy code".into()).into(),
                                        hit_slop: Some(4.0),
                                        ..PressProps::default()
                                    },
                                    copy,
                                )
                                .child(
                                    view()
                                        .h(px(40.0))
                                        .px_3()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(6.0))
                                        .child(glyph(
                                            if copied {
                                                GlyphName::Check
                                            } else {
                                                GlyphName::Copy
                                            },
                                            14.0,
                                            tone,
                                        ))
                                        .child(
                                            nsh_core::ui::text(if copied {
                                                "Copied"
                                            } else {
                                                "Copy"
                                            })
                                            .text_size(px(13.0))
                                            .text_color(tone),
                                        ),
                                ),
                            ),
                    )
                    .child(
                        scroll().horizontal().shows_indicator(false).child(
                            view().p_3().child(tail(
                                nsh_core::ui::text(b.text.clone())
                                    .selectable(true)
                                    .font_family(MONO)
                                    .text_size(px(13.0))
                                    .line_height(px(19.0))
                                    .text_color(colors::CODE),
                                props.caret,
                            )),
                        ),
                    )
            }
            Kind::Table => {
                let cell = |t: Text| t.flex_1().px(px(10.0)).py_2();
                view()
                    .rounded(px(radius::CONTROL))
                    .border_1()
                    .border_color(colors::BORDER)
                    .overflow_hidden()
                    .child(
                        view()
                            .flex_row()
                            .bg(colors::RAISED)
                            .finish(Finish::Raised)
                            .children(b.head.iter().map(|h| {
                                cell(nsh_core::ui::text(h.clone()))
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(colors::MUTED)
                            })),
                    )
                    .children(b.rows.iter().map(|row| {
                        view()
                            .flex_row()
                            .border_t_1()
                            .border_t_color(colors::LINE)
                            .children(row.iter().map(|c| {
                                cell(inline(c))
                                    .selectable(true)
                                    .text_size(px(14.0))
                                    .text_color(colors::TEXT)
                                    .tabular_nums()
                            }))
                    }))
            }
            Kind::Quote => view()
                .flex_row()
                .gap_3()
                .child(view().w(px(3.0)).rounded(px(1.5)).bg(colors::BORDER))
                .child(
                    body(paragraph(inline(&b.text)))
                        .flex_1()
                        .text_color(colors::MUTED),
                ),
        }
    }
}

/// The blocks of a reply as components; the caret (while streaming) ends
/// the last one, `copied` names the copied code block.
pub(crate) fn markdown(
    blocks: &[Block],
    id: &str,
    caret: bool,
    copied: Option<&str>,
    on_copy: Listener<MarkdownEvent>,
) -> View {
    let count = blocks.len();
    view()
        .gap_3()
        .children(blocks.iter().enumerate().map(|(i, block)| {
            let key = format!("{id}:{}", block.key);
            let copied = copied == Some(key.as_str());
            MarkdownBlock::element(MarkdownBlockProps {
                block: block.clone(),
                id: id.to_owned().into(),
                caret: caret && i + 1 == count,
                copied,
            })
            .key(block.key.clone())
            .on_event(on_copy.clone())
        }))
}
