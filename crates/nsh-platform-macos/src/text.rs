//! CoreText text engine: typesets on the engine thread, draws on the main
//! thread from finished `CTLine`s, so every text is laid out once.

use std::{cell::RefCell, ffi::c_void, ptr::NonNull};

use nsh_core::{
    AttributedText, Color, EllipsizeMode, FontFamily, ResolvedTextStyle, Size, TextAlign,
    TextEngine, TextLayout,
};
use objc2::rc::Retained;
use objc2_app_kit::{
    NSFont, NSFontWeight, NSFontWeightBlack, NSFontWeightBold, NSFontWeightHeavy,
    NSFontWeightLight, NSFontWeightMedium, NSFontWeightRegular, NSFontWeightSemibold,
    NSFontWeightThin, NSFontWeightUltraLight,
};
use objc2_core_foundation::{
    CFArray, CFAttributedString, CFDictionary, CFIndex, CFMutableAttributedString, CFNumber,
    CFRange, CFRetained, CFString, CFType, CFURL, CGFloat, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{CGColor, CGContext, CGPath};
use objc2_core_text::{
    CTFont, CTFontDescriptor, CTFontManagerRegisterFontsForURL, CTFontManagerScope,
    CTFontSymbolicTraits, CTFrame, CTFramesetter, CTLine, CTLineTruncationType, CTParagraphStyle,
    CTParagraphStyleSetting, CTParagraphStyleSpecifier, CTRun, CTRunDelegate,
    CTRunDelegateCallbacks, CTTextAlignment, CTTypesetter, CTWritingDirection,
    kCTFontAttributeName, kCTFontFamilyNameAttribute, kCTFontFeatureSettingsAttribute,
    kCTFontOpenTypeFeatureTag, kCTFontOpenTypeFeatureValue, kCTFontTraitsAttribute,
    kCTFontWeightTrait, kCTForegroundColorAttributeName, kCTKernAttributeName,
    kCTParagraphStyleAttributeName, kCTRunDelegateAttributeName, kCTRunDelegateCurrentVersion,
    kCTUnderlineColorAttributeName, kCTUnderlineStyleAttributeName,
};
use rustc_hash::FxHashMap;

/// Attribute carrying the strikethrough color: CoreText draws underlines
/// but not strikethroughs, so [`crate::text_view::TextView`] draws those.
pub(crate) fn strikethrough_key() -> &'static CFString {
    static KEY: std::sync::OnceLock<CFStringKey> = std::sync::OnceLock::new();
    &KEY.get_or_init(|| CFStringKey(CFString::from_static_str("NshStrikethroughColor")))
        .0
}

/// Attribute carrying the color of an underline CoreText cannot draw
/// (styled, thick or offset): [`crate::text_view::TextView`] draws it.
pub(crate) fn underline_key() -> &'static CFString {
    static KEY: std::sync::OnceLock<CFStringKey> = std::sync::OnceLock::new();
    &KEY.get_or_init(|| CFStringKey(CFString::from_static_str("NshUnderlineColor")))
        .0
}

/// Attribute carrying how drawn decoration lines look: a `CFArray` of
/// [style, thickness (0: the font's), offset (NaN: the font's)].
pub(crate) fn decoration_key() -> &'static CFString {
    static KEY: std::sync::OnceLock<CFStringKey> = std::sync::OnceLock::new();
    &KEY.get_or_init(|| CFStringKey(CFString::from_static_str("NshDecorationStyle")))
        .0
}

struct CFStringKey(CFRetained<CFString>);
// SAFETY: CFStrings are immutable and thread-safe.
unsafe impl Send for CFStringKey {}
// SAFETY: as above.
unsafe impl Sync for CFStringKey {}

/// Identifies a font: size bits, weight, family, italic, OpenType features.
type FontKey = (u32, u16, Option<FontFamily>, bool, u8);

/// Caches fonts. Bounded: an app animating `fontSize` would otherwise grow
/// it without limit.
#[derive(Default)]
struct FontCache {
    fonts: RefCell<FxHashMap<FontKey, CFRetained<CTFont>>>,
}

impl FontCache {
    const MAX: usize = 64;

    fn get(&self, style: &ResolvedTextStyle) -> CFRetained<CTFont> {
        let key = (
            style.font_size.to_bits(),
            style.font_weight,
            style.font_family,
            style.italic,
            style.font_variant.0,
        );
        let mut fonts = self.fonts.borrow_mut();
        if let Some(font) = fonts.get(&key) {
            return font.clone();
        }
        if fonts.len() >= Self::MAX {
            fonts.clear();
        }
        let font = make_font(style);
        fonts.insert(key, font.clone());
        font
    }

    fn clear(&self) {
        self.fonts.borrow_mut().clear();
    }
}

pub(crate) fn make_font(style: &ResolvedTextStyle) -> CFRetained<CTFont> {
    let size = style.font_size as CGFloat;
    let font = style
        .font_family
        .and_then(|family| {
            let name = family.name();
            generic_font(&name, size, style.font_weight)
                .unwrap_or_else(|| family_font(&name, size, style.font_weight))
        })
        .unwrap_or_else(|| system_font(size, style.font_weight));
    let font = if style.italic {
        let italic = CTFontSymbolicTraits::TraitItalic;
        // SAFETY: a valid font; size 0 keeps the size, null matrix keeps it.
        unsafe { font.copy_with_symbolic_traits(0.0, std::ptr::null(), italic, italic) }
            // No italic face: keep the upright one, as web browsers do (they
            // synthesize an oblique; CoreText cannot).
            .unwrap_or(font)
    } else {
        font
    };
    with_features(font, style.font_variant)
}

/// `font` with the OpenType features of `variant` (`fontVariant`).
fn with_features(font: CFRetained<CTFont>, variant: nsh_core::FontVariant) -> CFRetained<CTFont> {
    if variant.0 == 0 {
        return font;
    }
    let one = CFNumber::new_i32(1);
    let features: Vec<CFRetained<CFDictionary<CFString, CFType>>> = variant
        .tags()
        .map(|tag| {
            let tag = CFString::from_str(tag);
            // SAFETY: immutable CoreText constants.
            let keys = unsafe { [kCTFontOpenTypeFeatureTag, kCTFontOpenTypeFeatureValue] };
            CFDictionary::from_slices(&keys, &[tag.as_ref(), one.as_ref()])
        })
        .collect();
    let features = CFArray::from_retained_objects(&features);
    // SAFETY: an immutable CoreText constant.
    let key = unsafe { kCTFontFeatureSettingsAttribute };
    let attributes = CFDictionary::<CFString, CFType>::from_slices(&[key], &[features.as_ref()]);
    // SAFETY: attribute values have the types CoreText documents; size 0
    // and a null matrix keep the font's.
    unsafe {
        let descriptor = CTFontDescriptor::with_attributes(attributes.as_opaque());
        font.copy_with_attributes(0.0, std::ptr::null(), Some(&descriptor))
    }
}

/// The font of `style` as a `NSFont` (toll-free bridged with `CTFont`), for
/// AppKit text views such as inputs.
pub(crate) fn ns_font(style: &ResolvedTextStyle) -> Retained<NSFont> {
    let font = make_font(style);
    let ptr = CFRetained::into_raw(font).as_ptr().cast::<NSFont>();
    // SAFETY: toll-free bridging; ownership of the +1 reference moves over.
    unsafe { Retained::from_raw(ptr).expect("non-null font") }
}

/// CSS generic families (`font-mono`, `font-serif`): the system font's
/// designs. `Some(None)` is the plain system font; `None` is not generic.
fn generic_font(family: &str, size: CGFloat, weight: u16) -> Option<Option<CFRetained<CTFont>>> {
    use objc2_app_kit::{
        NSFontDescriptorSystemDesign, NSFontDescriptorSystemDesignRounded,
        NSFontDescriptorSystemDesignSerif,
    };
    let design: &NSFontDescriptorSystemDesign = match family {
        "system-ui" | "sans-serif" | "ui-sans-serif" | "-apple-system" => return Some(None),
        "monospace" | "ui-monospace" => {
            let font = NSFont::monospacedSystemFontOfSize_weight(size, font_weight(weight));
            return Some(Some(bridge(font)));
        }
        // SAFETY: immutable AppKit constants.
        "serif" | "ui-serif" => unsafe { NSFontDescriptorSystemDesignSerif },
        "ui-rounded" => unsafe { NSFontDescriptorSystemDesignRounded },
        _ => return None,
    };
    let base = NSFont::systemFontOfSize_weight(size, font_weight(weight));
    let descriptor = base.fontDescriptor();
    let font = descriptor
        .fontDescriptorWithDesign(design)
        .and_then(|d| NSFont::fontWithDescriptor_size(&d, size))
        .unwrap_or(base);
    Some(Some(bridge(font)))
}

/// A `NSFont` as the `CTFont` it is (toll-free bridged).
fn bridge(font: Retained<NSFont>) -> CFRetained<CTFont> {
    let ptr = Retained::into_raw(font).cast::<CTFont>();
    // SAFETY: toll-free bridging; ownership of the +1 reference moves over.
    unsafe { CFRetained::from_raw(NonNull::new(ptr).expect("non-null font")) }
}

/// A font of an installed or registered family, or `None` (logged) if the
/// family does not exist.
fn family_font(family: &str, size: CGFloat, weight: u16) -> Option<CFRetained<CTFont>> {
    let name = CFString::from_str(family);
    let weight = CFNumber::new_cgfloat(font_weight(weight));
    let traits = CFDictionary::<CFString, CFType>::from_slices(
        // SAFETY: immutable CoreText constants.
        &[unsafe { kCTFontWeightTrait }],
        &[weight.as_ref()],
    );
    // SAFETY: immutable CoreText constants.
    let keys = unsafe { [kCTFontFamilyNameAttribute, kCTFontTraitsAttribute] };
    let attributes =
        CFDictionary::<CFString, CFType>::from_slices(&keys, &[name.as_ref(), traits.as_ref()]);
    // SAFETY: attribute values have the types CoreText documents.
    let font = unsafe {
        let descriptor = CTFontDescriptor::with_attributes(attributes.as_opaque());
        CTFont::with_font_descriptor(&descriptor, size, std::ptr::null())
    };
    // An unknown family silently resolves to a default font: detect it.
    // SAFETY: a valid font.
    let resolved = unsafe { font.family_name() }.to_string();
    if resolved.eq_ignore_ascii_case(family) {
        Some(font)
    } else {
        log::warn!("font family `{family}` is not available; using the system font");
        None
    }
}

/// The system font as a `CTFont`. `NSFont` is toll-free bridged with
/// `CTFont`, and the AppKit constructor is the supported way to get the
/// system font at an arbitrary weight.
fn system_font(size: CGFloat, weight: u16) -> CFRetained<CTFont> {
    let font: Retained<NSFont> = NSFont::systemFontOfSize_weight(size, font_weight(weight));
    let ptr = Retained::into_raw(font).cast::<CTFont>();
    // SAFETY: toll-free bridging; ownership of the +1 reference moves over.
    unsafe { CFRetained::from_raw(NonNull::new(ptr).expect("non-null font")) }
}

fn font_weight(weight: u16) -> NSFontWeight {
    // SAFETY: AppKit's font weight constants are immutable statics.
    unsafe {
        match weight {
            0..=149 => NSFontWeightUltraLight,
            150..=249 => NSFontWeightThin,
            250..=349 => NSFontWeightLight,
            350..=449 => NSFontWeightRegular,
            450..=549 => NSFontWeightMedium,
            550..=649 => NSFontWeightSemibold,
            650..=749 => NSFontWeightBold,
            750..=849 => NSFontWeightHeavy,
            _ => NSFontWeightBlack,
        }
    }
}

pub(crate) fn cg_color(color: Color) -> CFRetained<CGColor> {
    let (r, g, b, a) = color.to_f64();
    CGColor::new_srgb(r, g, b, a)
}

/// The font of the first run of `line`.
fn line_font(line: &CTLine) -> Option<CFRetained<CTFont>> {
    // SAFETY: the line's own run array (of CTRuns) and attribute
    // dictionary, whose font value is a CTFont.
    unsafe {
        let runs = line.glyph_runs();
        if runs.count() == 0 {
            return None;
        }
        let run = &*runs.value_at_index(0).cast::<objc2_core_text::CTRun>();
        let attributes = run.attributes();
        let attributes: &objc2_core_foundation::CFDictionary<CFString, CFType> =
            attributes.cast_unchecked();
        attributes
            .get(kCTFontAttributeName)
            .and_then(|f| f.downcast::<CTFont>().ok())
    }
}

/// Typesetting state of one text content.
pub struct Prepared {
    framesetter: CFRetained<CTFramesetter>,
    string: CFRetained<CFMutableAttributedString>,
    /// String length in UTF-16 units.
    length: CFIndex,
    max_lines: u32,
    ellipsize: EllipsizeMode,
    align: TextAlign,
    /// Attachments: placeholder offset (UTF-16) and height.
    attachments: Vec<(CFIndex, CGFloat)>,
}

/// A finished line and its origin (CoreText's y-up space, from the bottom of
/// the box).
pub(crate) struct LaidLine {
    pub(crate) line: CFRetained<CTLine>,
    pub(crate) origin: CGPoint,
    /// Its characters in the text (before any ellipsis).
    pub(crate) range: CFRange,
}

/// Finished lines, drawn by [`crate::text_view::TextView`].
pub(crate) struct CoreTextLayout {
    pub(crate) lines: Vec<LaidLine>,
    /// The box's height (lines are placed y up from its bottom).
    pub(crate) height: CGFloat,
    /// Some run carries a strikethrough.
    pub(crate) strikethrough: bool,
    /// Top left of each attachment's placeholder (y down), if shown.
    attachments: Vec<Option<(f32, f32)>>,
}

// SAFETY: lines are immutable once created, and CoreText objects are
// thread-safe; they are created on the engine thread and only read (drawn)
// on the main thread.
unsafe impl Send for CoreTextLayout {}
// SAFETY: see `Send`.
unsafe impl Sync for CoreTextLayout {}

/// [`TextEngine`] backed by CoreText.
#[derive(Default)]
pub struct CoreTextEngine {
    fonts: FontCache,
}

const WHOLE: CFRange = CFRange {
    location: 0,
    length: 0,
};

impl CoreTextEngine {
    fn attributed_string(&self, text: &AttributedText) -> CFRetained<CFMutableAttributedString> {
        let string =
            CFMutableAttributedString::new(None, 0).expect("CFAttributedStringCreateMutable");
        let s = Some(&*string);
        // SAFETY: valid mutable string and range.
        unsafe {
            CFMutableAttributedString::replace_string(
                s,
                WHOLE,
                Some(&CFString::from_str(&text.text)),
            )
        };
        let length = text.spans.last().map_or(0, |span| span.end as CFIndex);
        CFMutableAttributedString::begin_editing(s);
        let set = |range: CFRange, key: &CFString, value: &CFType| {
            // SAFETY: values have the types CoreText requires for their keys
            // (checked at each call), and every range lies within the string
            // because span offsets are UTF-16 units, like CFString.
            unsafe { CFMutableAttributedString::set_attribute(s, range, Some(key), Some(value)) }
        };
        let paragraph = paragraph_style(text);
        // SAFETY: immutable CoreText constants (here and below).
        set(
            CFRange {
                location: 0,
                length,
            },
            unsafe { kCTParagraphStyleAttributeName },
            paragraph.as_ref(),
        );
        for span in &text.spans {
            let range = CFRange {
                location: span.start as CFIndex,
                length: (span.end - span.start) as CFIndex,
            };
            let style = &span.style;
            set(
                range,
                unsafe { kCTFontAttributeName },
                self.fonts.get(style).as_ref(),
            );
            set(
                range,
                unsafe { kCTForegroundColorAttributeName },
                cg_color(style.color).as_ref(),
            );
            if style.letter_spacing != 0.0 {
                let kern = CFNumber::new_cgfloat(style.letter_spacing as CGFloat);
                set(range, unsafe { kCTKernAttributeName }, kern.as_ref());
            }
            let decoration = cg_color(style.decoration_color.unwrap_or(style.color));
            let custom = style.decoration_style != nsh_core::DecorationStyle::Solid
                || style.decoration_thickness.is_some()
                || style.underline_offset.is_some();
            if custom && !style.decoration.is_none() {
                let code = match style.decoration_style {
                    nsh_core::DecorationStyle::Solid => 0,
                    nsh_core::DecorationStyle::Double => 1,
                    nsh_core::DecorationStyle::Dotted => 2,
                    nsh_core::DecorationStyle::Dashed => 3,
                    nsh_core::DecorationStyle::Wavy => 4,
                };
                let numbers = [
                    CFNumber::new_i32(code),
                    CFNumber::new_cgfloat(style.decoration_thickness.unwrap_or(0.0) as CGFloat),
                    CFNumber::new_cgfloat(
                        style
                            .underline_offset
                            .map_or(CGFloat::NAN, |o| o as CGFloat),
                    ),
                ];
                let array = CFArray::from_retained_objects(&numbers);
                set(range, decoration_key(), array.as_ref());
            }
            if style.decoration.underline && custom {
                set(range, underline_key(), decoration.as_ref());
            } else if style.decoration.underline {
                // kCTUnderlineStyleSingle.
                let single = CFNumber::new_i32(1);
                set(
                    range,
                    unsafe { kCTUnderlineStyleAttributeName },
                    single.as_ref(),
                );
                set(
                    range,
                    unsafe { kCTUnderlineColorAttributeName },
                    decoration.as_ref(),
                );
            }
            if style.decoration.line_through {
                set(range, strikethrough_key(), decoration.as_ref());
            }
        }
        // Attachments: their placeholders take the attachment's size (on the
        // baseline) and draw nothing.
        let clear = cg_color(Color::TRANSPARENT);
        for a in &text.attachments {
            let range = CFRange {
                location: a.index as CFIndex,
                length: 1,
            };
            if let Some(delegate) = run_delegate(a.width as CGFloat, a.height as CGFloat) {
                set(
                    range,
                    unsafe { kCTRunDelegateAttributeName },
                    delegate.as_ref(),
                );
            }
            set(
                range,
                unsafe { kCTForegroundColorAttributeName },
                clear.as_ref(),
            );
        }
        CFMutableAttributedString::end_editing(s);
        string
    }
}

/// A run delegate reserving `width` x `height` above the baseline.
fn run_delegate(width: CGFloat, height: CGFloat) -> Option<CFRetained<CTRunDelegate>> {
    type Size = [CGFloat; 2];
    unsafe extern "C-unwind" fn dealloc(refcon: NonNull<c_void>) {
        // SAFETY: the box created below, freed once by CoreText.
        drop(unsafe { Box::from_raw(refcon.as_ptr().cast::<Size>()) });
    }
    unsafe extern "C-unwind" fn get_ascent(refcon: NonNull<c_void>) -> CGFloat {
        // SAFETY: the live box created below.
        unsafe { (*refcon.as_ptr().cast::<Size>())[1] }
    }
    unsafe extern "C-unwind" fn get_descent(_: NonNull<c_void>) -> CGFloat {
        0.0
    }
    unsafe extern "C-unwind" fn get_width(refcon: NonNull<c_void>) -> CGFloat {
        // SAFETY: as above.
        unsafe { (*refcon.as_ptr().cast::<Size>())[0] }
    }
    static CALLBACKS: CTRunDelegateCallbacks = CTRunDelegateCallbacks {
        version: kCTRunDelegateCurrentVersion as CFIndex,
        dealloc: Some(dealloc),
        getAscent: Some(get_ascent),
        getDescent: Some(get_descent),
        getWidth: Some(get_width),
    };
    let refcon = Box::into_raw(Box::new([width, height]));
    // SAFETY: static callbacks; the delegate owns `refcon` (freed by
    // `dealloc`).
    let delegate = unsafe { CTRunDelegate::new(NonNull::from(&CALLBACKS), refcon.cast()) };
    if delegate.is_none() {
        // SAFETY: not handed over.
        drop(unsafe { Box::from_raw(refcon) });
    }
    delegate
}

/// Where the lines put the placeholders at `attachments` (offset, height):
/// the top left of each (y down in a box `height` tall), if shown.
fn attachment_points(
    lines: &[LaidLine],
    attachments: &[(CFIndex, CGFloat)],
    height: CGFloat,
) -> Vec<Option<(f32, f32)>> {
    let mut points = vec![None; attachments.len()];
    for l in lines {
        // SAFETY: the line's own run array (of CTRuns).
        let runs = unsafe { l.line.glyph_runs() };
        for r in 0..runs.count() {
            // SAFETY: an index within the array.
            let run = unsafe { &*runs.value_at_index(r).cast::<CTRun>() };
            // SAFETY: a valid run.
            let range = unsafe { run.string_range() };
            // Only placeholder runs: an ellipsis token claims the range it
            // cut.
            // SAFETY: a valid run; an immutable CoreText key.
            let placeholder = unsafe {
                let key: *const CFString = kCTRunDelegateAttributeName;
                run.attributes().contains_ptr_key(key.cast())
            };
            if !placeholder {
                continue;
            }
            for (i, &(at, h)) in attachments.iter().enumerate() {
                if points[i].is_some() || at < range.location || at >= range.location + range.length
                {
                    continue;
                }
                // A delegate run holds its one placeholder glyph.
                let mut p = CGPoint::new(0.0, 0.0);
                // SAFETY: one position into a one-point buffer.
                unsafe {
                    run.positions(
                        CFRange {
                            location: 0,
                            length: 1,
                        },
                        NonNull::from(&mut p),
                    )
                };
                points[i] = Some(((l.origin.x + p.x) as f32, (height - l.origin.y - h) as f32));
            }
        }
    }
    points
}

fn paragraph_style(text: &AttributedText) -> CFRetained<CTParagraphStyle> {
    let alignment = match text.align {
        // React Native: the right side in right-to-left layouts.
        TextAlign::Auto if text.rtl => CTTextAlignment::Right,
        TextAlign::Auto => CTTextAlignment::Natural,
        TextAlign::Left => CTTextAlignment::Left,
        TextAlign::Center => CTTextAlignment::Center,
        TextAlign::Right => CTTextAlignment::Right,
        TextAlign::Justify => CTTextAlignment::Justified,
    };
    let line_height = text.spans.first().map_or(0.0, |s| s.style.line_height) as CGFloat;
    let setting = |spec, value: *const c_void, size| CTParagraphStyleSetting {
        spec,
        valueSize: size,
        value: NonNull::new(value.cast_mut()).expect("non-null setting"),
    };
    // `writingDirection`; natural (from the content) by default.
    let direction = match text.writing_direction {
        None => CTWritingDirection::Natural,
        Some(false) => CTWritingDirection::LeftToRight,
        Some(true) => CTWritingDirection::RightToLeft,
    };
    let mut settings = vec![
        setting(
            CTParagraphStyleSpecifier::Alignment,
            (&raw const alignment).cast(),
            size_of::<CTTextAlignment>(),
        ),
        setting(
            CTParagraphStyleSpecifier::BaseWritingDirection,
            (&raw const direction).cast(),
            size_of::<CTWritingDirection>(),
        ),
    ];
    if line_height > 0.0 {
        for spec in [
            CTParagraphStyleSpecifier::MinimumLineHeight,
            CTParagraphStyleSpecifier::MaximumLineHeight,
        ] {
            settings.push(setting(
                spec,
                (&raw const line_height).cast(),
                size_of::<CGFloat>(),
            ));
        }
    }
    // SAFETY: every setting points at a live value of the declared size.
    unsafe { CTParagraphStyle::new(settings.as_ptr(), settings.len()) }
}

impl Prepared {
    fn typesetter(&self) -> CFRetained<CTTypesetter> {
        // SAFETY: a valid framesetter.
        unsafe { self.framesetter.typesetter() }
    }

    /// Length (UTF-16 units) of the first `lines` lines at `width`.
    fn lines_length(&self, lines: u32, width: f64) -> CFIndex {
        let typesetter = self.typesetter();
        let mut end = 0;
        for _ in 0..lines {
            if end >= self.length {
                break;
            }
            // SAFETY: `end` lies within the string.
            let count = unsafe { typesetter.suggest_line_break(end, width) };
            if count <= 0 {
                break;
            }
            end += count;
        }
        end
    }

    /// `line` (starting at `start`) cut to `width` with an ellipsis, as
    /// `ellipsizeMode` says, standing for the rest of the text.
    fn truncate(&self, start: CFIndex, width: f64) -> Option<CFRetained<CTLine>> {
        let kind = match self.ellipsize {
            EllipsizeMode::Tail => CTLineTruncationType::End,
            EllipsizeMode::Head => CTLineTruncationType::Start,
            EllipsizeMode::Middle => CTLineTruncationType::Middle,
            EllipsizeMode::Clip => return None,
        };
        // The token copies the attributes of the last character that is
        // not an attachment (a placeholder's would make it invisible).
        let mut last = (self.length - 1).max(start);
        while last > start && self.attachments.iter().any(|&(at, _)| at == last) {
            last -= 1;
        }
        // SAFETY: ranges lie within the string.
        unsafe {
            let rest = self.typesetter().line(CFRange {
                location: start,
                length: self.length - start,
            });
            let token_string =
                CFMutableAttributedString::new(None, 0).expect("CFAttributedStringCreateMutable");
            CFMutableAttributedString::replace_string(
                Some(&token_string),
                WHOLE,
                Some(&CFString::from_static_str("\u{2026}")),
            );
            let source: &CFAttributedString = &self.string;
            let attributes = CFAttributedString::attributes(source, last, std::ptr::null_mut());
            if let Some(attributes) = attributes {
                CFMutableAttributedString::set_attributes(
                    Some(&token_string),
                    CFRange {
                        location: 0,
                        length: 1,
                    },
                    Some(&attributes),
                    false,
                );
            }
            let token = CTLine::with_attributed_string(&token_string);
            rest.truncated_line(width, kind, Some(&token))
        }
    }

    fn flush_factor(&self) -> CGFloat {
        match self.align {
            TextAlign::Center => 0.5,
            TextAlign::Right => 1.0,
            TextAlign::Auto | TextAlign::Left | TextAlign::Justify => 0.0,
        }
    }
}

impl TextEngine for CoreTextEngine {
    type Prepared = Prepared;

    fn prepare(&self, text: &AttributedText) -> Prepared {
        let string = self.attributed_string(text);
        let attributed: &CFAttributedString = &string;
        // SAFETY: a valid attributed string.
        let framesetter = unsafe { CTFramesetter::with_attributed_string(attributed) };
        Prepared {
            framesetter,
            length: text.text.encode_utf16().count() as CFIndex,
            string,
            max_lines: text.max_lines,
            ellipsize: text.ellipsize,
            align: text.align,
            attachments: text
                .attachments
                .iter()
                .map(|a| (a.index as CFIndex, a.height as CGFloat))
                .collect(),
        }
    }

    fn measure(&self, prepared: &Prepared, max_width: Option<f32>) -> Size {
        let width = max_width.map_or(CGFloat::MAX, |w| w as CGFloat);
        let range = if prepared.max_lines > 0 {
            CFRange {
                location: 0,
                length: prepared.lines_length(prepared.max_lines, width),
            }
        } else {
            WHOLE
        };
        // SAFETY: a range within the string, no frame attributes, no fit
        // range out-param.
        let size = unsafe {
            prepared.framesetter.suggest_frame_size_with_constraints(
                range,
                None,
                CGSize::new(width, CGFloat::MAX),
                std::ptr::null_mut(),
            )
        };
        Size::new(size.width as f32, size.height as f32)
    }

    fn layout(&self, prepared: &Prepared, size: Size) -> TextLayout {
        let (width, height) = (size.width as CGFloat, size.height as CGFloat);
        let rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(width, height));
        // SAFETY: a valid rectangle path and whole-string range.
        let frame: CFRetained<CTFrame> = unsafe {
            let path = CGPath::with_rect(rect, std::ptr::null());
            prepared.framesetter.frame(WHOLE, &path, None)
        };
        // SAFETY: the frame's own line array (of CTLines) and origins.
        let mut lines = unsafe {
            let array = frame.lines();
            let count = array.count();
            let mut origins = vec![CGPoint::new(0.0, 0.0); count as usize];
            if let Some(ptr) = NonNull::new(origins.as_mut_ptr()) {
                frame.line_origins(WHOLE, ptr);
            }
            (0..count)
                .map(|i| {
                    let line = array.value_at_index(i).cast::<CTLine>();
                    let line = CFRetained::retain(NonNull::new(line.cast_mut()).expect("line"));
                    let range = line.string_range();
                    LaidLine {
                        line,
                        origin: origins[i as usize],
                        range,
                    }
                })
                .collect::<Vec<_>>()
        };
        if prepared.max_lines > 0 {
            lines.truncate(prepared.max_lines as usize);
        }
        // Text left over (cut by `numberOfLines` or the box): ellipsize the
        // last line.
        if let Some(last) = lines.last_mut() {
            // SAFETY: a valid line.
            let range = unsafe { last.line.string_range() };
            if range.location + range.length < prepared.length
                && let Some(truncated) = prepared.truncate(range.location, width)
            {
                // SAFETY: a valid line.
                let x = unsafe { truncated.pen_offset_for_flush(prepared.flush_factor(), width) };
                last.origin.x = x;
                last.line = truncated;
            }
        }
        let attachments = if prepared.attachments.is_empty() {
            Vec::new()
        } else {
            attachment_points(&lines, &prepared.attachments, height)
        };
        TextLayout(std::sync::Arc::new(CoreTextLayout {
            strikethrough: has_strikethrough(prepared),
            lines,
            height,
            attachments,
        }))
    }

    fn attachments(&self, layout: &TextLayout) -> Vec<Option<(f32, f32)>> {
        layout
            .downcast::<CoreTextLayout>()
            .map_or_else(Vec::new, |l| l.attachments.clone())
    }

    fn lines(&self, layout: &TextLayout) -> Vec<nsh_core::TextLine> {
        let Some(layout) = layout.downcast::<CoreTextLayout>() else {
            return Vec::new();
        };
        layout
            .lines
            .iter()
            .map(|l| {
                let (mut ascent, mut descent, mut leading) = (0.0, 0.0, 0.0);
                // SAFETY: a valid line and out pointers.
                let width = unsafe {
                    l.line
                        .typographic_bounds(&mut ascent, &mut descent, &mut leading)
                };
                let (cap, x_height) =
                    line_font(&l.line).map_or((ascent * 0.7, ascent * 0.5), |f| {
                        // SAFETY: a valid font.
                        unsafe { (f.cap_height(), f.x_height()) }
                    });
                nsh_core::TextLine {
                    text: String::new(),
                    start: l.range.location as u32,
                    end: (l.range.location + l.range.length) as u32,
                    x: l.origin.x as f32,
                    y: (layout.height - l.origin.y - ascent) as f32,
                    width: width as f32,
                    height: (ascent + descent + leading) as f32,
                    ascender: ascent as f32,
                    descender: descent as f32,
                    cap_height: cap as f32,
                    x_height: x_height as f32,
                }
            })
            .collect()
    }

    fn register_font(&self, data: &[u8]) -> Result<(), String> {
        // CoreText registers fonts synchronously only from files: write the
        // data to a content-addressed file first.
        let path = font_file(data).map_err(|e| format!("cannot store font: {e}"))?;
        let url = CFURL::from_file_path(&path).ok_or("bad font path")?;
        let mut error = std::ptr::null_mut();
        // SAFETY: a valid file URL and error out-param.
        let ok = unsafe {
            CTFontManagerRegisterFontsForURL(&url, CTFontManagerScope::Process, &mut error)
        };
        // Fonts resolve differently now (a family may have been missing).
        self.fonts.clear();
        if ok {
            return Ok(());
        }
        // SAFETY: on failure the out-param holds a +1 CFError.
        let error = unsafe { CFRetained::from_raw(NonNull::new(error).ok_or("unknown error")?) };
        // kCTFontManagerErrorAlreadyRegistered: e.g. after a hot reload.
        if error.code() == 105 {
            return Ok(());
        }
        Err(error
            .description()
            .map_or_else(|| "font rejected".to_owned(), |d| d.to_string()))
    }
}

/// Writes font data to `<temp>/nsh-fonts/<hash>.font` (once).
fn font_file(data: &[u8]) -> std::io::Result<std::path::PathBuf> {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::hash::DefaultHasher::new();
    data.hash(&mut hasher);
    let dir = std::env::temp_dir().join("nsh-fonts");
    let path = dir.join(format!("{:016x}.font", hasher.finish()));
    if !path.exists() {
        std::fs::create_dir_all(&dir)?;
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, data)?;
        std::fs::rename(&tmp, &path)?;
    }
    Ok(path)
}

fn has_strikethrough(prepared: &Prepared) -> bool {
    let mut index = 0;
    let source: &CFAttributedString = &prepared.string;
    while index < prepared.length {
        let mut range = CFRange {
            location: 0,
            length: 0,
        };
        // SAFETY: `index` lies within the string; `range` is a valid out-param.
        let value = unsafe {
            CFAttributedString::attribute(source, index, Some(strikethrough_key()), &mut range)
        };
        if value.is_some() {
            return true;
        }
        let mut other = CFRange {
            location: 0,
            length: 0,
        };
        // SAFETY: as above.
        let underline = unsafe {
            CFAttributedString::attribute(source, index, Some(underline_key()), &mut other)
        };
        if underline.is_some() {
            return true;
        }
        if other.location + other.length < range.location + range.length {
            range = other;
        }
        index = (range.location + range.length).max(index + 1);
    }
    false
}

/// Warms up font and CoreText caches (process-wide) on a background thread
/// so the first real layout does not pay for them.
pub(crate) fn prewarm() {
    let engine = CoreTextEngine::default();
    for (size, weight) in [(14.0, 400), (17.0, 400), (17.0, 700)] {
        let mut text = AttributedText::default();
        let style = ResolvedTextStyle {
            font_size: size,
            font_weight: weight,
            ..Default::default()
        };
        text.push("Hg 0123 ab", style);
        let prepared = engine.prepare(&text);
        let measured = engine.measure(&prepared, Some(100.0));
        let _ = engine.layout(&prepared, measured);
    }
}

// ---- Drawing (main thread) ----

/// What a text view draws: the text, its final layout and content box.
pub(crate) struct DrawnText {
    /// The text (spans, links, selection options).
    pub(crate) text: std::sync::Arc<AttributedText>,
    /// Its final layout ([`CoreTextLayout`]).
    pub(crate) layout: TextLayout,
    /// The content box in the view (inside padding and borders).
    pub(crate) content: nsh_core::Rect,
}

/// A text view got new text or a new layout.
pub(crate) fn set(
    view: &crate::view::NshView,
    text: &std::sync::Arc<AttributedText>,
    layout: &TextLayout,
    content: nsh_core::Rect,
) {
    view.state_mut().text = Some(DrawnText {
        text: text.clone(),
        layout: layout.clone(),
        content,
    });
    view.changed();
}

/// Draws a text view's text (in `drawRect:`; the context is y down).
pub(crate) fn draw(ctx: &CGContext, state: &crate::view::ViewState) {
    use objc2_core_foundation::CGAffineTransform;
    let Some(drawn) = &state.text else { return };
    let Some(text) = drawn.layout.downcast::<CoreTextLayout>() else {
        return;
    };
    let content = drawn.content;
    let c = Some(ctx);
    CGContext::save_g_state(c);
    // CoreText draws in a y-up space with the origin at the bottom of the
    // box.
    CGContext::set_text_matrix(
        c,
        CGAffineTransform {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        },
    );
    CGContext::translate_ctm(
        c,
        content.x as CGFloat,
        (content.y + content.height) as CGFloat,
    );
    CGContext::scale_ctm(c, 1.0, -1.0);
    if let Some(shadow) = drawn.text.shadow {
        // The context is flipped: a positive offset must still go down.
        CGContext::set_shadow_with_color(
            c,
            CGSize::new(shadow.offset_x as CGFloat, -shadow.offset_y as CGFloat),
            shadow.radius as CGFloat,
            Some(&cg_color(shadow.color)),
        );
    }
    // An animated color: the text (decorations too) drawn as is into a
    // layer, then painted over in the color where it drew, so its
    // antialiasing stays; the layer casts the shadow.
    let color = state.anim.color;
    if color.is_some() {
        // SAFETY: no auxiliary info.
        unsafe { CGContext::begin_transparency_layer(c, None) };
    }
    for line in &text.lines {
        CGContext::set_text_position(c, line.origin.x, line.origin.y);
        // SAFETY: a valid line and context.
        unsafe { line.line.draw(ctx) };
        if text.strikethrough {
            draw_decorations(ctx, line);
        }
    }
    if let Some(color) = color {
        CGContext::set_blend_mode(c, objc2_core_graphics::CGBlendMode::SourceIn);
        CGContext::set_fill_color_with_color(c, Some(&cg_color(color)));
        // Wherever the text drew (glyphs may reach past the box); the clip
        // bounds the rest.
        CGContext::fill_rect(c, CGContext::clip_bounding_box(c));
        CGContext::end_transparency_layer(c);
    }
    CGContext::restore_g_state(c);
}

/// Draws the decoration lines CoreText does not: strikethroughs, and
/// underlines with a style, thickness or offset (y-up text space).
fn draw_decorations(ctx: &CGContext, line: &LaidLine) {
    // SAFETY: the line's own run array (of CTRuns) and their attribute
    // dictionaries, whose values have the types set by the text engine.
    unsafe {
        let runs = line.line.glyph_runs();
        for i in 0..runs.count() {
            let run = &*runs.value_at_index(i).cast::<CTRun>();
            let attributes = run.attributes();
            let attributes: &CFDictionary<CFString, CFType> = attributes.cast_unchecked();
            let strike = attributes
                .get(strikethrough_key())
                .and_then(|c| c.downcast::<CGColor>().ok());
            let under = attributes
                .get(underline_key())
                .and_then(|c| c.downcast::<CGColor>().ok());
            if strike.is_none() && under.is_none() {
                continue;
            }
            let Some(font) = attributes
                .get(kCTFontAttributeName)
                .and_then(|f| f.downcast::<CTFont>().ok())
            else {
                continue;
            };
            let (mut style, mut thickness, mut offset) = (0, 0.0, CGFloat::NAN);
            if let Some(d) = attributes
                .get(decoration_key())
                .and_then(|d| d.downcast::<CFArray>().ok())
            {
                let n = |i: CFIndex| {
                    (*d.value_at_index(i).cast::<CFNumber>())
                        .as_cgfloat()
                        .unwrap_or(0.0)
                };
                style = n(0) as i32;
                thickness = n(1);
                offset = n(2);
            }
            if thickness <= 0.0 {
                thickness = font.underline_thickness().max(1.0);
            }
            let range = run.string_range();
            let x = line.origin.x
                + line
                    .line
                    .offset_for_string_index(range.location, std::ptr::null_mut());
            let width = run.typographic_bounds(
                CFRange {
                    location: 0,
                    length: 0,
                },
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            if let Some(color) = strike {
                let y = line.origin.y + font.x_height() / 2.0;
                stroke_decoration(ctx, &color, style, x, y, width, thickness);
            }
            if let Some(color) = under {
                // Below the baseline: the font's position, or the offset.
                let below = if offset.is_nan() {
                    -font.underline_position()
                } else {
                    offset + thickness / 2.0
                };
                stroke_decoration(
                    ctx,
                    &color,
                    style,
                    x,
                    line.origin.y - below,
                    width,
                    thickness,
                );
            }
        }
    }
}

/// One decoration line centered on `y` (y-up), `style` as encoded by the
/// text engine: solid, double, dotted, dashed, wavy.
fn stroke_decoration(
    ctx: &CGContext,
    color: &CGColor,
    style: i32,
    x: CGFloat,
    y: CGFloat,
    width: CGFloat,
    thickness: CGFloat,
) {
    let c = Some(ctx);
    CGContext::save_g_state(c);
    CGContext::set_stroke_color_with_color(c, Some(color));
    CGContext::set_line_width(c, thickness);
    let line = |y: CGFloat| {
        CGContext::move_to_point(c, x, y);
        CGContext::add_line_to_point(c, x + width, y);
    };
    match style {
        1 => {
            // Two lines a thickness apart.
            line(y + thickness);
            line(y - thickness);
        }
        2 => {
            let pattern = [0.0, thickness * 2.0];
            // SAFETY: a two-element pattern.
            unsafe { CGContext::set_line_dash(c, 0.0, pattern.as_ptr(), 2) };
            CGContext::set_line_cap(c, objc2_core_graphics::CGLineCap::Round);
            line(y);
        }
        3 => {
            let pattern = [thickness * 3.0, thickness * 2.0];
            // SAFETY: a two-element pattern.
            unsafe { CGContext::set_line_dash(c, 0.0, pattern.as_ptr(), 2) };
            line(y);
        }
        4 => {
            // A sine wave, one period per four thicknesses.
            let amplitude = thickness * 1.5;
            let period = (thickness * 4.0).max(4.0);
            let steps = ((width / period) * 8.0).ceil().max(2.0) as usize;
            CGContext::move_to_point(c, x, y);
            for i in 1..=steps {
                let t = width * i as CGFloat / steps as CGFloat;
                let phase = t / period * std::f64::consts::TAU;
                CGContext::add_line_to_point(c, x + t, y + amplitude * phase.sin());
            }
        }
        _ => line(y),
    }
    CGContext::stroke_path(c);
    CGContext::restore_g_state(c);
}
