//! DirectWrite text engine (engine thread) and text drawing (main thread).
//!
//! Text is typeset once, on the engine thread, with DirectWrite's shared
//! factory (free-threaded). The final layout ([`WinLayout`]) is an
//! `IDWriteTextLayout` handed over to the main thread, which only draws it
//! and asks it where characters are; the engine thread never touches it
//! again.

use std::sync::Arc;

use nsh_core::{AttributedText, Rect, ResolvedTextStyle, Size, TextAlign, TextEngine, TextLayout};
use windows::{
    Win32::Graphics::{
        Direct2D::D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
        DirectWrite::{
            DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_ITALIC, DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_WEIGHT, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
            DWRITE_TEXT_ALIGNMENT_LEADING, DWRITE_TEXT_ALIGNMENT_TRAILING, DWRITE_TEXT_METRICS,
            DWRITE_TEXT_RANGE, DWRITE_WORD_WRAPPING_NO_WRAP, DWRITE_WORD_WRAPPING_WRAP,
            IDWriteFactory2, IDWriteTextLayout,
        },
    },
    core::{HSTRING, PCWSTR},
};

use crate::{
    render::DrawCx,
    view::{NshView, ViewState},
};

/// An `IDWriteTextLayout` moved between threads.
pub(crate) struct SendLayout(pub(crate) IDWriteTextLayout);

// SAFETY: DirectWrite objects of the shared factory are free-threaded; a
// layout is made and measured on the engine thread, then used only by the
// main thread (never by both at once).
unsafe impl Send for SendLayout {}
// SAFETY: as above: never used from two threads at once.
unsafe impl Sync for SendLayout {}

/// The final layout of a text, for drawing.
pub(crate) struct WinLayout {
    /// The DirectWrite layout, sized to the text's box.
    pub(crate) layout: SendLayout,
}

/// Typesetting state for one content.
pub(crate) struct Prepared {
    text: Vec<u16>,
    attributed: AttributedText,
}

/// The text engine of a surface (one per engine thread).
pub(crate) struct DWriteTextEngine {
    factory: IDWriteFactory2,
    ui_family: Arc<str>,
}

/// The family of the system's UI font (the message font: Segoe UI).
pub(crate) fn ui_family() -> Arc<str> {
    use windows::Win32::UI::WindowsAndMessaging::{
        NONCLIENTMETRICSW, SPI_GETNONCLIENTMETRICS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
        SystemParametersInfoW,
    };
    let mut metrics = NONCLIENTMETRICSW {
        cbSize: std::mem::size_of::<NONCLIENTMETRICSW>() as u32,
        ..Default::default()
    };
    // SAFETY: the structure's size is set as the call requires.
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETNONCLIENTMETRICS,
            metrics.cbSize,
            Some(&mut metrics as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    }
    .is_ok();
    let face = &metrics.lfMessageFont.lfFaceName;
    let len = face.iter().position(|c| *c == 0).unwrap_or(face.len());
    if ok && len > 0 {
        String::from_utf16_lossy(&face[..len]).into()
    } else {
        "Segoe UI".into()
    }
}

impl DWriteTextEngine {
    /// An engine whose default family is `ui_family`.
    pub(crate) fn new(ui_family: Arc<str>) -> Self {
        DWriteTextEngine {
            factory: crate::gfx::dwrite(),
            ui_family,
        }
    }

    fn family(&self, style: &ResolvedTextStyle) -> Arc<str> {
        match &style.font_family {
            Some(f) => f.name(),
            None => self.ui_family.clone(),
        }
    }

    /// A layout of `prepared` at most `max_width` wide.
    fn make(&self, prepared: &Prepared, max_width: f32, max_height: f32) -> Option<IDWriteTextLayout> {
        let t = &prepared.attributed;
        let first = t.spans.first().map(|s| s.style).unwrap_or_default();
        let family = HSTRING::from(&*self.family(&first));
        // SAFETY: COM calls with valid strings and ranges.
        unsafe {
            let format = self
                .factory
                .CreateTextFormat(
                    &family,
                    None,
                    DWRITE_FONT_WEIGHT(first.font_weight as i32),
                    if first.italic {
                        DWRITE_FONT_STYLE_ITALIC
                    } else {
                        DWRITE_FONT_STYLE_NORMAL
                    },
                    DWRITE_FONT_STRETCH_NORMAL,
                    first.font_size.max(0.1),
                    &HSTRING::from(""),
                )
                .ok()?;
            let align = match t.align {
                TextAlign::Center => DWRITE_TEXT_ALIGNMENT_CENTER,
                TextAlign::Right => DWRITE_TEXT_ALIGNMENT_TRAILING,
                TextAlign::Justify => DWRITE_TEXT_ALIGNMENT_JUSTIFIED,
                _ => DWRITE_TEXT_ALIGNMENT_LEADING,
            };
            let _ = format.SetTextAlignment(align);
            let _ = format.SetWordWrapping(if t.no_wrap {
                DWRITE_WORD_WRAPPING_NO_WRAP
            } else {
                DWRITE_WORD_WRAPPING_WRAP
            });
            let layout = self
                .factory
                .CreateTextLayout(&prepared.text, &format, max_width, max_height)
                .ok()?;
            for span in &t.spans {
                let range = DWRITE_TEXT_RANGE {
                    startPosition: span.start,
                    length: span.end - span.start,
                };
                let s = &span.style;
                let name: Vec<u16> = self.family(s).encode_utf16().chain(Some(0)).collect();
                let _ = layout.SetFontFamilyName(PCWSTR(name.as_ptr()), range);
                let _ = layout.SetFontSize(s.font_size.max(0.1), range);
                let _ = layout.SetFontWeight(DWRITE_FONT_WEIGHT(s.font_weight as i32), range);
                let _ = layout.SetFontStyle(
                    if s.italic {
                        DWRITE_FONT_STYLE_ITALIC
                    } else {
                        DWRITE_FONT_STYLE_NORMAL
                    },
                    range,
                );
            }
            Some(layout)
        }
    }
}

fn metrics(layout: &IDWriteTextLayout) -> DWRITE_TEXT_METRICS {
    let mut m = DWRITE_TEXT_METRICS::default();
    // SAFETY: a local out value.
    let _ = unsafe { layout.GetMetrics(&mut m) };
    m
}

impl TextEngine for DWriteTextEngine {
    type Prepared = Prepared;

    fn prepare(&self, text: &AttributedText) -> Prepared {
        Prepared {
            text: text.text.encode_utf16().collect(),
            attributed: text.clone(),
        }
    }

    fn measure(&self, prepared: &Prepared, max_width: Option<f32>) -> Size {
        let Some(layout) = self.make(prepared, max_width.unwrap_or(f32::MAX / 4.0), f32::MAX / 4.0)
        else {
            return Size::new(0.0, 0.0);
        };
        let m = metrics(&layout);
        Size::new(m.widthIncludingTrailingWhitespace.ceil(), m.height.ceil())
    }

    fn layout(&self, prepared: &Prepared, size: Size) -> TextLayout {
        let layout = self
            .make(prepared, size.width.max(0.0), size.height.max(0.0))
            .map(|l| WinLayout {
                layout: SendLayout(l),
            });
        TextLayout(Arc::new(layout))
    }
}

/// A text view's content.
pub(crate) struct DrawnText {
    /// What it says (spans, colors).
    pub(crate) text: Arc<AttributedText>,
    /// The final layout.
    pub(crate) layout: TextLayout,
    /// Where the layout goes inside the view.
    pub(crate) content: Rect,
}

/// `Text`: a text view's content changed.
pub(crate) fn set(view: &NshView, text: &Arc<AttributedText>, layout: &TextLayout, content: Rect) {
    view.state_mut().text = Some(DrawnText {
        text: text.clone(),
        layout: layout.clone(),
        content,
    });
    view.changed();
}

/// Draws a text view's text.
pub(crate) fn draw(cx: &mut DrawCx<'_>, state: &ViewState) {
    let Some(text) = &state.text else { return };
    let Some(Some(layout)) = text.layout.downcast::<Option<WinLayout>>() else {
        return;
    };
    let color = state.anim.color.unwrap_or_else(|| {
        text.text
            .spans
            .first()
            .map_or(nsh_core::Color::BLACK, |s| s.style.color)
    });
    let origin = windows_numerics::Vector2 {
        X: text.content.x,
        Y: text.content.y,
    };
    // SAFETY: a COM call between BeginDraw and EndDraw.
    unsafe {
        cx.dc.DrawTextLayout(
            origin,
            &layout.layout.0,
            cx.solid(color),
            D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
        );
    }
}
