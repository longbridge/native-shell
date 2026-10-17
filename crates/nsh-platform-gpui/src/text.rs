//! Text: GPUI's text system on the engine thread, glyphs painted on the
//! main thread.
//!
//! [`GpuiTextEngine`] shapes on the engine thread with a
//! [`gpui::WindowTextSystem`] of its own over the app's
//! [`gpui::TextSystem`] (thread-safe; the fonts are the app's, so the
//! `FontId`s it hands out are the ones the main thread paints with).
//! GPUI shapes one line of one font size at a time, so `prepare` shapes
//! each style run of each paragraph on its own (a run's glyph positions
//! follow on from the previous run's width) into one flat glyph list per
//! paragraph; lines are then broken by nsh ([`Prepared::break_lines`]) at
//! the opportunities [`break_after`] finds: after spaces, around CJK
//! ideographs, after hyphens and slashes, or anywhere in a word too long
//! for its line.
//!
//! The final layout ([`GpuiLayout`]) is plain data: per line its origin,
//! baseline, metrics and glyph range; per glyph its font, id, size and
//! position. Colors and decorations are not in it (the core reuses a
//! layout for text that only changes them): [`draw`] reads them from the
//! spans by each glyph's UTF-16 offset, as the Linux backend does.

use std::{borrow::Cow, cell::RefCell, ops::Range, sync::Arc};

use gpui::{
    Font, FontFeatures, FontId, FontStyle, FontWeight, GlyphId, SharedString, TextRun, TextSystem,
    WindowTextSystem, px,
};
use nsh_core::{
    AttributedText, EllipsizeMode, Rect, ResolvedTextStyle, Size, TextAlign, TextEngine,
    TextLayout, TextLine,
};

use crate::{render::DrawCx, view::ViewState};

/// The family of the platform's UI font (GPUI resolves the name).
pub(crate) fn ui_family() -> Arc<str> {
    Arc::from(".SystemUIFont")
}

/// How many lines the engine's line cache may hold before it is made
/// anew (GPUI's cache keeps every line until its frame is finished, which
/// only a window does).
const CACHE_LINES: u32 = 4096;

/// The text engine of one surface (engine thread).
pub(crate) struct GpuiTextEngine {
    system: Arc<TextSystem>,
    shaper: RefCell<WindowTextSystem>,
    shaped: std::cell::Cell<u32>,
    family: SharedString,
}

impl GpuiTextEngine {
    /// An engine over the app's text system, with `family` as the UI
    /// font.
    pub(crate) fn new(system: Arc<TextSystem>, family: Arc<str>) -> Self {
        GpuiTextEngine {
            shaper: RefCell::new(WindowTextSystem::new(system.clone())),
            system,
            shaped: std::cell::Cell::new(0),
            family: SharedString::from(family.to_string()),
        }
    }

    fn font(&self, style: &ResolvedTextStyle) -> Font {
        let family = match &style.font_family {
            Some(f) => SharedString::from(f.name().to_string()),
            None => self.family.clone(),
        };
        Font {
            family,
            features: FontFeatures::default(),
            fallbacks: None,
            weight: FontWeight(style.font_weight as f32),
            style: if style.italic {
                FontStyle::Italic
            } else {
                FontStyle::Normal
            },
        }
    }

    /// Shapes `text` (no line breaks) in one font and size.
    fn shape(&self, text: &str, font: &Font, size: f32) -> Arc<gpui::LineLayout> {
        let n = self.shaped.get() + 1;
        if n > CACHE_LINES {
            *self.shaper.borrow_mut() = WindowTextSystem::new(self.system.clone());
            self.shaped.set(0);
        } else {
            self.shaped.set(n);
        }
        let run = TextRun {
            len: text.len(),
            font: font.clone(),
            color: gpui::black(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        self.shaper
            .borrow()
            .layout_line(text, px(size), &[run], None)
    }
}

/// The metrics of one style run's font.
#[derive(Debug, Clone, Copy, Default)]
struct Metrics {
    ascent: f32,
    descent: f32,
    /// The line height the style asks for (0: the font's).
    line_height: f32,
    cap_height: f32,
    x_height: f32,
}

/// One shaped glyph of a paragraph.
#[derive(Debug, Clone, Copy)]
struct Shaped {
    /// Left edge along the paragraph (points).
    x: f32,
    /// Advance.
    advance: f32,
    font: FontId,
    id: GlyphId,
    size: f32,
    emoji: bool,
    /// Byte offset of its cluster in the text.
    byte: usize,
    /// UTF-16 offset of its cluster.
    utf16: u32,
    /// Its style run.
    span: u16,
    /// An attachment's placeholder (not drawn).
    attachment: bool,
}

/// A paragraph (text between line breaks).
#[derive(Debug, Clone)]
struct Paragraph {
    /// Its bytes in the text.
    bytes: Range<usize>,
    /// Its glyphs in [`Prepared::glyphs`].
    glyphs: Range<usize>,
    /// The style run at its start (an empty paragraph's metrics).
    span: u16,
}

/// The text shaped for measuring and layout.
pub(crate) struct Prepared {
    text: AttributedText,
    glyphs: Vec<Shaped>,
    paragraphs: Vec<Paragraph>,
    metrics: Vec<Metrics>,
    /// The ellipsis shaped in the last run's font, its advance.
    ellipsis: Option<(Shaped, f32)>,
    /// UTF-16 offset of every byte boundary (`len + 1` entries).
    utf16: Vec<u32>,
}

/// A line broken from a paragraph.
#[derive(Debug, Clone)]
struct Broken {
    glyphs: Range<usize>,
    /// Width without trailing spaces.
    width: f32,
    bytes: Range<usize>,
    paragraph: usize,
}

/// A laid-out line.
#[derive(Debug, Clone)]
pub(crate) struct Line {
    /// Its glyphs in [`GpuiLayout::glyphs`].
    glyphs: Range<usize>,
    /// Left edge in the content box.
    x: f32,
    /// Top edge.
    y: f32,
    /// Baseline.
    baseline: f32,
    width: f32,
    height: f32,
    ascent: f32,
    descent: f32,
    cap_height: f32,
    x_height: f32,
    /// UTF-16 range.
    start: u32,
    end: u32,
}

/// One glyph to paint.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Glyph {
    /// Left edge relative to its line's.
    x: f32,
    advance: f32,
    font: FontId,
    id: GlyphId,
    size: f32,
    emoji: bool,
    /// UTF-16 offset of its cluster (colors, decorations, hit testing).
    utf16: u32,
    attachment: bool,
}

/// The final layout of a text (plain data, sent to the main thread).
#[derive(Debug, Clone, Default)]
pub(crate) struct GpuiLayout {
    lines: Vec<Line>,
    glyphs: Vec<Glyph>,
    /// Attachment top-left corners, in text order.
    attachments: Vec<Option<(f32, f32)>>,
}

impl GpuiLayout {
    /// The UTF-16 offset of the character at `(x, y)` (content box).
    pub(crate) fn char_at(&self, x: f32, y: f32) -> Option<u32> {
        let line = self.lines.iter().find(|l| y >= l.y && y < l.y + l.height)?;
        let x = x - line.x;
        self.glyphs[line.glyphs.clone()]
            .iter()
            .find(|g| x >= g.x && x < g.x + g.advance.max(0.5))
            .map(|g| g.utf16)
    }

    /// The UTF-16 offset nearest to `(x, y)` (content box): the caret
    /// position a click there puts.
    pub(crate) fn offset_at(&self, x: f32, y: f32) -> u32 {
        let Some(line) = self
            .lines
            .iter()
            .find(|l| y < l.y + l.height)
            .or(self.lines.last())
        else {
            return 0;
        };
        let x = x - line.x;
        for g in &self.glyphs[line.glyphs.clone()] {
            if x < g.x + g.advance / 2.0 {
                return g.utf16;
            }
        }
        line.end
    }

    /// The rectangles (content box) covering UTF-16 range `range`.
    pub(crate) fn range_rects(&self, range: Range<u32>) -> Vec<Rect> {
        let mut out = Vec::new();
        for line in &self.lines {
            if line.end <= range.start || line.start >= range.end {
                continue;
            }
            let glyphs = &self.glyphs[line.glyphs.clone()];
            let inside: Vec<&Glyph> = glyphs
                .iter()
                .filter(|g| g.utf16 >= range.start && g.utf16 < range.end)
                .collect();
            let (Some(first), Some(last)) = (inside.first(), inside.last()) else {
                continue;
            };
            out.push(Rect {
                x: line.x + first.x,
                y: line.y,
                width: last.x + last.advance - first.x,
                height: line.height,
            });
        }
        out
    }
}

/// True when a line may break after `a` (followed by `b`).
fn break_after(a: char, b: Option<char>) -> bool {
    let Some(b) = b else { return true };
    if a.is_whitespace() {
        return !b.is_whitespace();
    }
    if is_closing(b) {
        return false;
    }
    if is_cjk(a) || is_cjk(b) {
        return !is_opening(a);
    }
    matches!(a, '-' | '/' | '\u{2010}' | '\u{2013}' | '\u{2014}') && !b.is_whitespace()
}

/// CJK ideographs, kana, hangul and fullwidth forms: lines break between
/// any two of them.
fn is_cjk(c: char) -> bool {
    matches!(c as u32,
        0x1100..=0x11FF | 0x2E80..=0x2FFF | 0x3040..=0x30FF | 0x3100..=0x31FF
        | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF
        | 0xFF00..=0xFFEF | 0x20000..=0x3FFFF)
}

/// Punctuation a line never starts with (CJK closing marks among them).
fn is_closing(c: char) -> bool {
    matches!(
        c,
        '、' | '。'
            | '，'
            | '．'
            | '：'
            | '；'
            | '！'
            | '？'
            | '）'
            | '」'
            | '』'
            | '】'
            | '》'
            | '〉'
            | '’'
            | '”'
            | ','
            | '.'
            | ':'
            | ';'
            | '!'
            | '?'
            | ')'
            | ']'
            | '}'
    )
}

/// Punctuation a line never ends with.
fn is_opening(c: char) -> bool {
    matches!(
        c,
        '（' | '「' | '『' | '【' | '《' | '〈' | '‘' | '“' | '(' | '[' | '{'
    )
}

impl Prepared {
    /// The style run at UTF-16 offset `at`.
    fn span_at(&self, at: u32) -> u16 {
        self.text
            .spans
            .iter()
            .position(|s| at >= s.start && at < s.end)
            .unwrap_or(self.text.spans.len().saturating_sub(1)) as u16
    }

    /// Breaks the paragraphs into lines at most `max` wide.
    fn break_lines(&self, max: Option<f32>) -> Vec<Broken> {
        let mut lines = Vec::new();
        let text = &self.text.text;
        for (pi, p) in self.paragraphs.iter().enumerate() {
            let glyphs = &self.glyphs[p.glyphs.clone()];
            if glyphs.is_empty() {
                lines.push(Broken {
                    glyphs: p.glyphs.clone(),
                    width: 0.0,
                    bytes: p.bytes.clone(),
                    paragraph: pi,
                });
                continue;
            }
            let max = match max {
                Some(m) if !self.text.no_wrap => m,
                _ => f32::INFINITY,
            };
            // Clusters: glyph index ranges sharing a byte offset.
            let mut start = 0;
            let mut last_break: Option<usize> = None;
            let mut i = 0;
            while i < glyphs.len() {
                let line_x = glyphs[start].x;
                let end_x = glyphs[i].x + glyphs[i].advance - line_x;
                let ch = text[glyphs[i].byte..].chars().next().unwrap_or(' ');
                if end_x > max + 0.01 && i > start && !ch.is_whitespace() {
                    let cut = last_break.filter(|b| *b > start).unwrap_or(i);
                    lines.push(self.broken(pi, p, start, cut));
                    start = cut;
                    last_break = None;
                    // Spaces at the start of the next line are dropped.
                    while start < glyphs.len()
                        && text[glyphs[start].byte..]
                            .chars()
                            .next()
                            .is_some_and(char::is_whitespace)
                    {
                        start += 1;
                    }
                    i = start;
                    continue;
                }
                let next = glyphs
                    .get(i + 1)
                    .and_then(|g| text[g.byte..].chars().next());
                let same_cluster = glyphs.get(i + 1).is_some_and(|g| g.byte == glyphs[i].byte);
                if !same_cluster && break_after(ch, next) {
                    last_break = Some(i + 1);
                }
                i += 1;
            }
            if start < glyphs.len() || lines.last().is_none_or(|l| l.paragraph != pi) {
                lines.push(self.broken(pi, p, start, glyphs.len()));
            }
        }
        lines
    }

    fn broken(&self, pi: usize, p: &Paragraph, start: usize, end: usize) -> Broken {
        let glyphs = &self.glyphs[p.glyphs.clone()];
        let text = &self.text.text;
        let mut visible = end;
        while visible > start
            && text[glyphs[visible - 1].byte..]
                .chars()
                .next()
                .is_some_and(char::is_whitespace)
        {
            visible -= 1;
        }
        let width = if visible > start {
            glyphs[visible - 1].x + glyphs[visible - 1].advance - glyphs[start].x
        } else {
            0.0
        };
        let b0 = glyphs.get(start).map_or(p.bytes.end, |g| g.byte);
        let b1 = glyphs.get(end).map_or(p.bytes.end, |g| g.byte);
        Broken {
            glyphs: p.glyphs.start + start..p.glyphs.start + end,
            width,
            bytes: b0..b1,
            paragraph: pi,
        }
    }

    /// The metrics of a broken line: the largest of its runs'.
    fn line_metrics(&self, line: &Broken) -> (Metrics, f32) {
        let mut m = Metrics::default();
        let mut height: f32 = 0.0;
        let mut spans: Vec<u16> = self.glyphs[line.glyphs.clone()]
            .iter()
            .map(|g| g.span)
            .collect();
        if spans.is_empty() {
            spans.push(self.paragraphs[line.paragraph].span);
        }
        spans.dedup();
        for s in spans {
            let sm = self.metrics.get(s as usize).copied().unwrap_or_default();
            m.ascent = m.ascent.max(sm.ascent);
            m.descent = m.descent.max(sm.descent);
            m.cap_height = m.cap_height.max(sm.cap_height);
            m.x_height = m.x_height.max(sm.x_height);
            height = height.max(if sm.line_height > 0.0 {
                sm.line_height
            } else {
                sm.ascent + sm.descent
            });
        }
        // Attachments sit on the baseline: tall ones raise the ascent.
        for g in &self.glyphs[line.glyphs.clone()] {
            if g.attachment
                && let Some(a) = self.text.attachments.iter().find(|a| a.index == g.utf16)
            {
                m.ascent = m.ascent.max(a.height);
                height = height.max(m.ascent + m.descent);
            }
        }
        (m, height.max(m.ascent + m.descent * 0.0))
    }
}

impl TextEngine for GpuiTextEngine {
    type Prepared = Prepared;

    fn prepare(&self, text: &AttributedText) -> Prepared {
        let s = &text.text;
        // The UTF-16 offset of every byte: a character's bytes all have
        // its start's, so an offset maps to the boundary it starts at.
        let mut utf16 = Vec::with_capacity(s.len() + 1);
        let mut u = 0u32;
        for c in s.chars() {
            utf16.extend(std::iter::repeat_n(u, c.len_utf8()));
            u += c.len_utf16() as u32;
        }
        utf16.push(u);
        let byte_of =
            |offset: u32| -> usize { utf16.partition_point(|x| *x < offset).min(s.len()) };
        let styles: Vec<&ResolvedTextStyle> = text.spans.iter().map(|sp| &sp.style).collect();
        let default_style = ResolvedTextStyle::default();
        let mut metrics = Vec::with_capacity(styles.len().max(1));
        let mut fonts = Vec::with_capacity(styles.len().max(1));
        for style in styles
            .iter()
            .copied()
            .chain(styles.is_empty().then_some(&default_style))
        {
            let font = self.font(style);
            let id = self.system.resolve_font(&font);
            let size = px(style.font_size);
            let line_height = if style.line_height > 0.0 {
                style.line_height
            } else if style.line_height_multiple > 0.0 {
                style.line_height_multiple * style.font_size
            } else {
                0.0
            };
            metrics.push(Metrics {
                ascent: f32::from(self.system.ascent(id, size)),
                descent: f32::from(self.system.descent(id, size)).abs(),
                line_height,
                cap_height: f32::from(self.system.cap_height(id, size)),
                x_height: f32::from(self.system.x_height(id, size)),
            });
            fonts.push(font);
        }
        let mut glyphs = Vec::new();
        let mut paragraphs = Vec::new();
        let mut para_start = 0usize;
        let spans: Vec<(usize, usize, u16)> = if text.spans.is_empty() {
            vec![(0, s.len(), 0)]
        } else {
            text.spans
                .iter()
                .enumerate()
                .map(|(i, sp)| (byte_of(sp.start), byte_of(sp.end), i as u16))
                .collect()
        };
        let mut push_paragraph = |range: Range<usize>, glyphs: &mut Vec<Shaped>| {
            let first = glyphs.len();
            let mut x = 0.0f32;
            let mut span0 = None;
            for &(a, b, si) in &spans {
                let (a, b) = (a.max(range.start), b.min(range.end));
                if span0.is_none() && a <= range.start.max(a) && b >= a {
                    span0 = Some(si);
                }
                if a >= b {
                    continue;
                }
                let style = styles.get(si as usize).copied().unwrap_or(&default_style);
                let font = &fonts[si as usize];
                let size = style.font_size;
                let piece = &s[a..b];
                let line = self.shape(piece, font, size);
                let spacing = style.letter_spacing;
                // Glyphs in order of their positions, each with its advance.
                let mut flat: Vec<(f32, FontId, GlyphId, bool, usize)> = Vec::new();
                for run in &line.runs {
                    for g in &run.glyphs {
                        flat.push((
                            f32::from(g.position.x),
                            run.font_id,
                            g.id,
                            g.is_emoji,
                            g.index,
                        ));
                    }
                }
                flat.sort_by(|p, q| p.0.total_cmp(&q.0));
                let width = f32::from(line.width);
                let mut chars_before = 0usize;
                let mut last_index = usize::MAX;
                for k in 0..flat.len() {
                    let (gx, font_id, id, emoji, index) = flat[k];
                    let next_x = flat.get(k + 1).map_or(width, |n| n.0);
                    if index != last_index {
                        chars_before += 1;
                        last_index = index;
                    }
                    let byte = a + index;
                    let attachment = s[byte..].starts_with(nsh_core::ATTACHMENT_CHAR);
                    let mut advance = next_x - gx + spacing;
                    let u = utf16[byte];
                    if attachment && let Some(at) = text.attachments.iter().find(|at| at.index == u)
                    {
                        advance = at.width;
                    }
                    glyphs.push(Shaped {
                        x: x + gx + spacing * (chars_before.saturating_sub(1)) as f32,
                        advance,
                        font: font_id,
                        id,
                        size,
                        emoji,
                        byte,
                        utf16: u,
                        span: si,
                        attachment,
                    });
                }
                // Attachments replace their placeholder's advance: the
                // glyphs after them move.
                let mut shift = 0.0;
                let mut prev_end = x;
                for g in &mut glyphs[first..] {
                    if g.x + shift < prev_end && g.byte >= a {
                        shift = prev_end - g.x;
                    }
                    if g.byte >= a {
                        g.x += shift;
                    }
                    prev_end = g.x + g.advance;
                }
                x = glyphs.last().map_or(x, |g| g.x + g.advance).max(x);
                let _ = width;
            }
            paragraphs.push(Paragraph {
                bytes: range,
                glyphs: first..glyphs.len(),
                span: span0.unwrap_or(0),
            });
        };
        for (i, c) in s.char_indices() {
            if c == '\n' {
                push_paragraph(para_start..i, &mut glyphs);
                para_start = i + 1;
            }
        }
        push_paragraph(para_start..s.len(), &mut glyphs);
        let ellipsis = if text.max_lines > 0 && text.ellipsize != EllipsizeMode::Clip {
            let si = (fonts.len() - 1) as u16;
            let style = styles.get(si as usize).copied().unwrap_or(&default_style);
            let line = self.shape("…", &fonts[si as usize], style.font_size);
            line.runs
                .first()
                .and_then(|r| r.glyphs.first().map(|g| (r.font_id, g)))
                .map(|(font, g)| {
                    let advance = f32::from(line.width);
                    (
                        Shaped {
                            x: 0.0,
                            advance,
                            font,
                            id: g.id,
                            size: style.font_size,
                            emoji: g.is_emoji,
                            byte: s.len(),
                            utf16: u,
                            span: si,
                            attachment: false,
                        },
                        advance,
                    )
                })
        } else {
            None
        };
        Prepared {
            text: text.clone(),
            glyphs,
            paragraphs,
            metrics,
            ellipsis,
            utf16,
        }
    }

    fn measure(&self, prepared: &Prepared, max_width: Option<f32>) -> Size {
        let mut lines = prepared.break_lines(max_width);
        if prepared.text.max_lines > 0 {
            lines.truncate(prepared.text.max_lines as usize);
        }
        let mut width: f32 = 0.0;
        let mut height = 0.0;
        for line in &lines {
            let (_, h) = prepared.line_metrics(line);
            width = width.max(line.width);
            height += h;
        }
        if let Some(max) = max_width
            && !prepared.text.no_wrap
        {
            width = width.min(max);
        }
        Size::new(width.ceil(), height.ceil())
    }

    fn layout(&self, prepared: &Prepared, size: Size) -> TextLayout {
        let text = &prepared.text;
        let mut broken = prepared.break_lines(Some(size.width));
        let mut truncated = false;
        if text.max_lines > 0 && broken.len() > text.max_lines as usize {
            broken.truncate(text.max_lines as usize);
            truncated = true;
        }
        let rtl = text.writing_direction.unwrap_or(text.rtl);
        let mut out = GpuiLayout::default();
        let mut y = 0.0;
        let count = broken.len();
        for (li, line) in broken.iter().enumerate() {
            let (m, height) = prepared.line_metrics(line);
            let leading = (height - m.ascent - m.descent).max(0.0) / 2.0;
            let baseline = y + leading + m.ascent;
            let first = out.glyphs.len();
            let origin = prepared.glyphs.get(line.glyphs.start).map_or(0.0, |g| g.x);
            let mut width = line.width;
            let mut glyphs: Vec<Glyph> = prepared.glyphs[line.glyphs.clone()]
                .iter()
                .map(|g| Glyph {
                    x: g.x - origin,
                    advance: g.advance,
                    font: g.font,
                    id: g.id,
                    size: g.size,
                    emoji: g.emoji,
                    utf16: g.utf16,
                    attachment: g.attachment,
                })
                .collect();
            let last = li + 1 == count;
            let cut = last
                && (truncated || (text.max_lines > 0 && width > size.width + 0.5))
                && text.ellipsize != EllipsizeMode::Clip;
            if cut && let Some((e, advance)) = prepared.ellipsis {
                while glyphs
                    .last()
                    .is_some_and(|g| g.x + g.advance + advance > size.width)
                {
                    glyphs.pop();
                }
                let x = glyphs.last().map_or(0.0, |g| g.x + g.advance);
                glyphs.push(Glyph {
                    x,
                    advance,
                    font: e.font,
                    id: e.id,
                    size: e.size,
                    emoji: e.emoji,
                    utf16: glyphs.last().map_or(0, |g| g.utf16),
                    attachment: false,
                });
                width = x + advance;
            }
            let free = (size.width - width).max(0.0);
            let x = match text.align {
                TextAlign::Center => free / 2.0,
                TextAlign::Right => free,
                TextAlign::Auto | TextAlign::Justify if rtl => free,
                _ => 0.0,
            };
            if text.align == TextAlign::Justify && !last && free > 0.0 {
                let spaces: Vec<usize> = glyphs
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| {
                        let b = prepared.utf16.partition_point(|u| *u < g.utf16);
                        text.text[b.min(text.text.len())..].starts_with(' ')
                    })
                    .map(|(i, _)| i)
                    .collect();
                if !spaces.is_empty() {
                    let extra = free / spaces.len() as f32;
                    let mut add = 0.0;
                    for (i, g) in glyphs.iter_mut().enumerate() {
                        g.x += add;
                        if spaces.contains(&i) {
                            add += extra;
                        }
                    }
                }
            }
            for g in &glyphs {
                if g.attachment {
                    let a_height = text
                        .attachments
                        .iter()
                        .find(|a| a.index == g.utf16)
                        .map_or(0.0, |a| a.height);
                    out.attachments.push(Some((x + g.x, baseline - a_height)));
                }
            }
            out.glyphs.extend(glyphs);
            out.lines.push(Line {
                glyphs: first..out.glyphs.len(),
                x,
                y,
                baseline,
                width,
                height,
                ascent: m.ascent,
                descent: m.descent,
                cap_height: m.cap_height,
                x_height: m.x_height,
                start: prepared.utf16[line.bytes.start],
                end: prepared.utf16[line.bytes.end],
            });
            y += height;
        }
        // Attachments cut off by the line limit.
        while out.attachments.len() < text.attachments.len() {
            out.attachments.push(None);
        }
        TextLayout(Arc::new(out))
    }

    fn lines(&self, layout: &TextLayout) -> Vec<TextLine> {
        let Some(layout) = layout.downcast::<GpuiLayout>() else {
            return Vec::new();
        };
        layout
            .lines
            .iter()
            .map(|l| TextLine {
                text: String::new(),
                start: l.start,
                end: l.end,
                x: l.x,
                y: l.y,
                width: l.width,
                height: l.height,
                ascender: l.ascent,
                descender: l.descent,
                cap_height: l.cap_height,
                x_height: l.x_height,
            })
            .collect()
    }

    fn attachments(&self, layout: &TextLayout) -> Vec<Option<(f32, f32)>> {
        layout
            .downcast::<GpuiLayout>()
            .map(|l| l.attachments.clone())
            .unwrap_or_default()
    }

    fn register_font(&self, data: &[u8]) -> Result<(), String> {
        self.system
            .add_fonts(vec![Cow::Owned(data.to_vec())])
            .map_err(|e| e.to_string())
    }
}

/// What a text view draws.
pub(crate) struct DrawnText {
    /// What it says (spans, colors).
    pub(crate) text: Arc<AttributedText>,
    /// The engine's layout of it.
    pub(crate) layout: TextLayout,
    /// Where the layout goes inside the view.
    pub(crate) content: Rect,
}

/// `Text`: a text view's content changed.
pub(crate) fn set(
    view: &crate::view::NshView,
    text: &Arc<AttributedText>,
    layout: &TextLayout,
    content: Rect,
) {
    view.state_mut().text = Some(DrawnText {
        text: text.clone(),
        layout: layout.clone(),
        content,
    });
    view.changed();
}

/// The style run of `text` at UTF-16 offset `at`.
fn style_at(text: &AttributedText, at: u32) -> Option<&ResolvedTextStyle> {
    text.spans
        .iter()
        .find(|s| at >= s.start && at < s.end)
        .or(text.spans.last())
        .map(|s| &s.style)
}

/// Draws a text view's glyphs (view coordinates set on `cx`).
pub(crate) fn draw(cx: &mut DrawCx<'_>, state: &ViewState) {
    let Some(drawn) = &state.text else { return };
    let Some(layout) = drawn.layout.downcast::<GpuiLayout>() else {
        return;
    };
    if cx.culled() {
        return;
    }
    let text = &drawn.text;
    let animated = state.anim.color;
    let k = cx.transform_scale();
    let c = drawn.content;
    let mut glyphs = Vec::with_capacity(layout.glyphs.len());
    let mut lines = Vec::new();
    let shadow = text.shadow;
    for line in &layout.lines {
        let mut deco: Option<(f32, f32, nsh_core::Color, f32, bool)> = None;
        for g in &layout.glyphs[line.glyphs.clone()] {
            let style = style_at(text, g.utf16);
            let color = animated
                .or(style.map(|s| s.color))
                .unwrap_or(nsh_core::Color::BLACK);
            let x = c.x + line.x + g.x;
            let y = c.y + line.baseline;
            if !g.attachment {
                glyphs.push((cx.map(x, y), *g, color));
            }
            // Underlines and strikethroughs, merged along a line.
            if let Some(s) = style {
                for (on, strike) in [
                    (s.decoration.underline, false),
                    (s.decoration.line_through, true),
                ] {
                    if !on {
                        continue;
                    }
                    let thickness = s
                        .decoration_thickness
                        .unwrap_or((s.font_size / 14.0).max(1.0));
                    let dy = if strike {
                        -line.x_height.max(s.font_size * 0.25) / 2.0
                    } else {
                        s.underline_offset.unwrap_or(line.descent * 0.45)
                    };
                    let dc = s.decoration_color.unwrap_or(color);
                    lines.push((x, y + dy, g.advance, thickness, dc));
                }
            }
            let _ = &mut deco;
        }
    }
    let opacity = cx.opacity;
    if let Some(sh) = shadow {
        let (sx, sy) = (sh.offset_x * k, sh.offset_y * k);
        let mut color = crate::render::hsla(sh.color);
        color.a *= opacity;
        let list: Vec<_> = glyphs.iter().map(|(p, g, _)| (*p, *g)).collect();
        cx.paint(|window, _| {
            for (p, g) in list {
                let origin = gpui::point(p.x + px(sx), p.y + px(sy));
                let _ = window.paint_glyph(origin, g.font, g.id, px(g.size * k), color);
            }
        });
    }
    cx.paint(|window, _| {
        for (p, g, color) in glyphs {
            let result = if g.emoji {
                window.paint_emoji(p, g.font, g.id, px(g.size * k))
            } else {
                let mut c = crate::render::hsla(color);
                c.a *= opacity;
                window.paint_glyph(p, g.font, g.id, px(g.size * k), c)
            };
            if let Err(err) = result {
                log::debug!("text: cannot paint a glyph: {err}");
            }
        }
    });
    for (x, y, w, t, color) in lines {
        cx.fill_rect(crate::geom::rect(x, y - t / 2.0, w, t), [0.0; 4], color);
    }
}

/// The UTF-16 offset of the character at `(x, y)` (view coordinates), if
/// any (presses on nested text links).
pub(crate) fn char_at(state: &ViewState, x: f32, y: f32) -> Option<u32> {
    let drawn = state.text.as_ref()?;
    let layout = drawn.layout.downcast::<GpuiLayout>()?;
    layout.char_at(x - drawn.content.x, y - drawn.content.y)
}

/// True when the text can be selected with the pointer.
pub(crate) fn selectable(state: &ViewState) -> bool {
    state.text.as_ref().is_some_and(|t| t.text.selectable)
}

/// A pointer event on selectable text. True when handled.
pub(crate) fn pointer(
    _surface: &std::rc::Rc<crate::surface::Surface>,
    _view: &crate::view::NshView,
    _event: &crate::view::ItemPointer,
) -> bool {
    false
}

/// A key while selectable text has a selection. True when handled.
pub(crate) fn key(
    _surface: &std::rc::Rc<crate::surface::Surface>,
    _event: &crate::input::KeyEvent,
) -> bool {
    false
}

/// The window was pressed at `(x, y)` (root coordinates): selections
/// elsewhere clear.
pub(crate) fn window_pressed(_surface: &crate::surface::Surface, _x: f32, _y: f32) {}

/// The context menu of selectable text.
pub(crate) fn context_menu(
    _surface: &std::rc::Rc<crate::surface::Surface>,
    _view: &crate::view::NshView,
) {
}
