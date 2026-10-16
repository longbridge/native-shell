//! `nsh/clipboard`: the system clipboard's text (UIPasteboard,
//! ClipboardManager, `navigator.clipboard`). Needs the `clipboard`
//! capability when the app declares capabilities.

use crate::ui::{AsyncCx, ModuleCx, module};

/// The system clipboard.
pub(crate) struct Clipboard;

/// The system clipboard's text.
#[module]
impl Clipboard {
    /// Puts `text` on the clipboard.
    pub(crate) fn set_string(text: String, cx: &mut ModuleCx) {
        cx.set_clipboard(text);
    }

    /// The clipboard's text, `""` when it holds none. On iOS, reading it
    /// may show the system's paste notice.
    pub(crate) async fn get_string(cx: AsyncCx) -> String {
        cx.clipboard_string().await
    }

    /// Whether the clipboard holds text, without reading it (no paste
    /// notice).
    pub(crate) async fn has_string(cx: AsyncCx) -> bool {
        cx.clipboard_has_string().await
    }
}
