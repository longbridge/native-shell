//! System UI driven from JS: the status bar and alerts / action sheets.

use std::{cell::Cell, rc::Rc};

use block2::RcBlock;
use nsh_core::{
    Dialog, DialogButtonStyle, EventKind, EventPayload, PayloadValue, StatusBarConfig,
    StatusBarStyle, platform::Appearance,
};
use nsh_engine::EngineControl;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::NSString;
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle, UIDevice,
    UIPopoverArrowDirection, UIStatusBarAnimation, UIStatusBarStyle, UIUserInterfaceIdiom,
    UIUserInterfaceStyle, UIView, UIViewController,
};

thread_local! {
    /// The status bar the app asked for (one per process, like the bar).
    static STATUS_BAR: Cell<StatusBarConfig> = Cell::new(StatusBarConfig::default());
    /// The appearance the app asked for last (`setColorScheme`), for what
    /// is not a subview of a surface: the status bar, alerts, modals.
    static APPEARANCE: Cell<UIUserInterfaceStyle> = const { Cell::new(UIUserInterfaceStyle::Unspecified) };
}

/// The appearance the app asked for (`Unspecified`: the system's).
pub(crate) fn appearance() -> UIUserInterfaceStyle {
    APPEARANCE.get()
}

/// Applies the app's appearance to the surface `root` and to what is
/// shown outside of it (the status bar's default style, alerts and modals
/// shown from now on).
pub(crate) fn set_appearance(appearance: Appearance, root: &UIView) {
    let style = match appearance {
        Appearance::System => UIUserInterfaceStyle::Unspecified,
        Appearance::Light => UIUserInterfaceStyle::Light,
        Appearance::Dark => UIUserInterfaceStyle::Dark,
    };
    APPEARANCE.set(style);
    root.setOverrideUserInterfaceStyle(style);
    if let Some(window) = root.window() {
        let mut controller = window.rootViewController();
        while let Some(c) = controller {
            if c.isViewLoaded()
                && c.view()
                    .is_some_and(|v| !v.isDescendantOfView(root) && !root.isDescendantOfView(&v))
            {
                // Modals the surface presented.
                c.setOverrideUserInterfaceStyle(style);
            }
            c.setNeedsStatusBarAppearanceUpdate();
            controller = c.presentedViewController();
        }
    }
}

/// The status bar style the app asked for, for view controllers'
/// `preferredStatusBarStyle` (embedders forward theirs here too).
pub fn preferred_status_bar_style() -> UIStatusBarStyle {
    match STATUS_BAR.get().style {
        // The default follows the app's appearance when it has one.
        StatusBarStyle::Default => match APPEARANCE.get() {
            UIUserInterfaceStyle::Dark => UIStatusBarStyle::LightContent,
            UIUserInterfaceStyle::Light => UIStatusBarStyle::DarkContent,
            _ => UIStatusBarStyle::Default,
        },
        StatusBarStyle::Light => UIStatusBarStyle::LightContent,
        StatusBarStyle::Dark => UIStatusBarStyle::DarkContent,
    }
}

/// Whether the app hid the status bar, for `prefersStatusBarHidden`.
pub fn prefers_status_bar_hidden() -> bool {
    STATUS_BAR.get().hidden
}

/// How status bar changes animate, for `preferredStatusBarUpdateAnimation`.
pub fn status_bar_animation() -> UIStatusBarAnimation {
    if STATUS_BAR.get().animated {
        UIStatusBarAnimation::Fade
    } else {
        UIStatusBarAnimation::None
    }
}

/// Applies `config`: asks the controllers of `view`'s window to update.
pub(crate) fn set_status_bar(config: StatusBarConfig, view: &UIView) {
    STATUS_BAR.set(config);
    let Some(window) = view.window() else { return };
    let update = move || {
        let mut controller = window.rootViewController();
        while let Some(c) = controller {
            c.setNeedsStatusBarAppearanceUpdate();
            controller = c.presentedViewController();
        }
    };
    if config.animated {
        let block = RcBlock::new(update);
        UIView::animateWithDuration_animations(0.25, &block, view.mtm());
    } else {
        update();
    }
}

/// The controller to present over: the top one of `view`'s window.
fn top_controller(view: &UIView) -> Option<Retained<UIViewController>> {
    let mut top = view.window()?.rootViewController()?;
    while let Some(next) = top.presentedViewController() {
        if next.isBeingDismissed() {
            break;
        }
        top = next;
    }
    Some(top)
}

/// Reports one dialog's answer, once.
struct DialogAnswer {
    id: u32,
    control: EngineControl,
    answered: Cell<bool>,
}

impl DialogAnswer {
    fn answer(&self, index: i32) {
        if self.answered.replace(true) {
            return;
        }
        self.control.dispatch_event(
            nsh_core::ROOT_ID,
            EventKind::Dialog,
            EventPayload::Map(vec![
                ("id", PayloadValue::Number(self.id as f64)),
                ("index", PayloadValue::Number(index as f64)),
            ]),
        );
    }
}

/// Shows `dialog` over `view`'s window (a `UIAlertController`).
pub(crate) fn show_dialog(dialog: &Dialog, view: &UIView, control: &EngineControl) {
    let mtm = view.mtm();
    let answer = Rc::new(DialogAnswer {
        id: dialog.id,
        control: control.clone(),
        answered: Cell::new(false),
    });
    let Some(top) = top_controller(view) else {
        log::warn!("no window to show a dialog in");
        return answer.answer(-1);
    };
    let style = if dialog.sheet {
        UIAlertControllerStyle::ActionSheet
    } else {
        UIAlertControllerStyle::Alert
    };
    let title = dialog.title.as_deref().map(NSString::from_str);
    let message = dialog.message.as_deref().map(NSString::from_str);
    let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
        title.as_deref(),
        message.as_deref(),
        style,
        mtm,
    );
    alert.setOverrideUserInterfaceStyle(appearance());
    for (index, button) in dialog.buttons.iter().enumerate() {
        let style = match button.style {
            DialogButtonStyle::Default => UIAlertActionStyle::Default,
            DialogButtonStyle::Cancel => UIAlertActionStyle::Cancel,
            DialogButtonStyle::Destructive => UIAlertActionStyle::Destructive,
        };
        let answer = answer.clone();
        let handler = RcBlock::new(move |_| answer.answer(index as i32));
        let action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str(&button.text)),
            style,
            Some(&handler),
            mtm,
        );
        alert.addAction(&action);
    }
    let phone = UIDevice::currentDevice(mtm).userInterfaceIdiom() == UIUserInterfaceIdiom::Phone;
    if dialog.sheet
        && !phone
        && !dialog
            .buttons
            .iter()
            .any(|b| b.style == DialogButtonStyle::Cancel)
    {
        // A popover hides its cancel action and runs it on a tap outside:
        // one that answers "dismissed".
        let answer = answer.clone();
        let handler = RcBlock::new(move |_| answer.answer(-1));
        let action = UIAlertAction::actionWithTitle_style_handler(
            Some(&NSString::from_str("Cancel")),
            UIAlertActionStyle::Cancel,
            Some(&handler),
            mtm,
        );
        alert.addAction(&action);
    }
    if let Some(popover) = alert.popoverPresentationController() {
        // iPad and Mac: an action sheet is a popover; anchor it at the
        // bottom center of the surface, without an arrow.
        let bounds = view.bounds();
        popover.setSourceView(Some(view));
        popover.setSourceRect(CGRect::new(
            CGPoint::new(bounds.size.width / 2.0, bounds.size.height),
            CGSize::ZERO,
        ));
        popover.setPermittedArrowDirections(UIPopoverArrowDirection::empty());
    }
    top.presentViewController_animated_completion(&alert, true, None);
}

/// `Clipboard.setString`: puts `text` on the general pasteboard.
pub(crate) fn set_clipboard(text: &str) {
    // SAFETY: a plain setter of the general pasteboard.
    unsafe {
        objc2_ui_kit::UIPasteboard::generalPasteboard().setString(Some(&NSString::from_str(text)))
    };
}

/// `Clipboard.getString` / `hasString`: whether the general pasteboard
/// holds text and, unless `has_only`, the text. `hasStrings` does not read
/// it, so it shows no paste notice.
pub(crate) fn read_clipboard(has_only: bool) -> (bool, Option<String>) {
    let board = objc2_ui_kit::UIPasteboard::generalPasteboard();
    if has_only {
        // SAFETY: a plain getter.
        return (unsafe { board.hasStrings() }, None);
    }
    // SAFETY: a plain getter.
    let text = unsafe { board.string() }.map(|s| s.to_string());
    (text.is_some(), text)
}

/// `AccessibilityInfo.announceForAccessibility`: VoiceOver reads `text`.
pub(crate) fn announce(text: &str) {
    let text = NSString::from_str(text);
    // SAFETY: announcements take the message as an NSString.
    unsafe {
        objc2_ui_kit::UIAccessibilityPostNotification(
            objc2_ui_kit::UIAccessibilityAnnouncementNotification,
            Some(&text),
        )
    };
}

/// `AccessibilityInfo.setAccessibilityFocus`: VoiceOver moves to `view`.
pub(crate) fn accessibility_focus(view: &UIView) {
    // SAFETY: layout changes take the element to focus.
    unsafe {
        objc2_ui_kit::UIAccessibilityPostNotification(
            objc2_ui_kit::UIAccessibilityLayoutChangedNotification,
            Some(view),
        )
    };
}
