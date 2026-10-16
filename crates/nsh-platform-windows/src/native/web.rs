//! Web views: WebView2 controllers in child windows.
//!
//! Each web view gets a container window, a child of the surface's window,
//! that its WebView2 controller fills. The container follows its view:
//! after every frame the renderer tells the item where the view was drawn
//! ([`Hosted::placed`]), and the container moves there, clipped to the
//! view's clipping ancestors and rounded corners with a window region, or
//! hides when the view was not drawn. Being a window, it covers whatever
//! nsh draws at its place (a dialog drawn above the web view is under it),
//! and it is absent from offscreen snapshots (`--script` screenshots).
//!
//! WebView2 is asynchronous: the environment (one per thread and profile
//! folder, `%LOCALAPPDATA%\<app id>\WebView2`) and then the controller are
//! made in the background. Props and commands that come before the
//! controller is ready wait for it ([`WebItem::ready`]). The WebView2
//! loader is linked statically (`WebView2LoaderStatic.lib`), so apps do not
//! ship `WebView2Loader.dll`; the WebView2 runtime itself comes with
//! Windows 11 (and Edge's installer on Windows 10). Without it, a web view
//! draws a neutral placeholder, as on Linux.
//!
//! Pages talk to the app as with React Native's web view: the bridge script
//! defines `window.ReactNativeWebView.postMessage(data)` (`onMessage`), and
//! `postMessage()` dispatches a `message` event on the page's `window`.
//! Top-level navigations stay within the app's `webview` capability.
//! Events and their payloads are the iOS ones: `onLoadStart` and
//! `onNavigationStateChange` when a navigation starts, `onLoad` and
//! `onNavigationStateChange` when it completes, `onError` when it fails.
//!
//! Not supported: `source.baseUrl` for HTML (WebView2 loads strings
//! without one), `scrollEnabled` and `allowsBackForwardNavigationGestures`.

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use nsh_core::{Color, Command, EventKind, EventPayload, PayloadValue, WebProps, WebSource};
use webview2_com::{
    AddScriptToExecuteOnDocumentCreatedCompletedHandler,
    CreateCoreWebView2ControllerCompletedHandler, CreateCoreWebView2EnvironmentCompletedHandler,
    Microsoft::Web::WebView2::Win32::{
        COREWEBVIEW2_COLOR, COREWEBVIEW2_WEB_ERROR_STATUS, ICoreWebView2, ICoreWebView2Controller,
        ICoreWebView2Controller2, ICoreWebView2Environment,
        ICoreWebView2ExecuteScriptCompletedHandler, ICoreWebView2Settings2,
    },
    NavigationCompletedEventHandler, NavigationStartingEventHandler,
    WebMessageReceivedEventHandler, take_pwstr,
};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE,
            DirectWrite::{
                DWRITE_FONT_STRETCH_NORMAL, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT_NORMAL,
                DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER,
                DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_WORD_WRAPPING_WRAP, IDWriteTextFormat,
            },
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WINDOW_EX_STYLE,
            WNDCLASSW, WS_CHILD, WS_CLIPCHILDREN, WS_CLIPSIBLINGS,
        },
    },
    core::{BOOL, HRESULT, HSTRING, Interface, PCWSTR, PWSTR, w},
};

use super::{Emitter, Placement, error};
use crate::{
    gfx,
    render::{DrawCx, rect_f},
    surface::Surface,
    view::{Hosted, NshView},
};

/// Lets pages talk to the app, like React Native's web view.
const BRIDGE: &str = "window.ReactNativeWebView = window.ReactNativeWebView || { postMessage: function (data) { window.chrome.webview.postMessage(String(data)); } };";

// The loader, linked statically (webview2-com-sys imports it from
// `WebView2Loader.dll`, which would have to ship next to every app).
#[link(name = "WebView2LoaderStatic", kind = "static")]
unsafe extern "system" {
    #[link_name = "CreateCoreWebView2EnvironmentWithOptions"]
    fn create_environment(
        browser_folder: PCWSTR,
        user_data_folder: PCWSTR,
        options: *mut std::ffi::c_void,
        handler: *mut std::ffi::c_void,
    ) -> HRESULT;
}

/// A web view: its container window and, once made, its controller.
pub(crate) struct WebItem {
    emitter: Emitter,
    surface: Weak<Surface>,
    /// The container window (`None` once closed).
    container: Cell<Option<HWND>>,
    controller: RefCell<Option<ICoreWebView2Controller>>,
    webview: RefCell<Option<ICoreWebView2>>,
    /// No web view can be made (no WebView2 runtime): a placeholder.
    failed: Cell<bool>,
    /// The props asked for (applied once the controller is ready).
    props: RefCell<Option<WebProps>>,
    /// The props applied to the web view.
    applied: RefCell<Option<WebProps>>,
    /// Commands that came before the controller.
    queued: RefCell<Vec<Command>>,
    /// The ids of the app's document scripts (removed when they change).
    scripts: Rc<RefCell<Vec<String>>>,
    /// The engine's own user agent (restored when the app's is removed).
    default_agent: RefCell<Option<String>>,
    /// Where the last frame drew the view (window pixels).
    target: Cell<Option<RECT>>,
    placement: Placement,
    /// The controller's bounds and visibility applied.
    bounds: Cell<Option<(i32, i32)>>,
    /// The view's background, the page's until it paints.
    background: Cell<Color>,
}

impl WebItem {
    pub(crate) fn new(surface: &Rc<Surface>, view: &NshView) -> Rc<Self> {
        let item = Rc::new(WebItem {
            emitter: Emitter::new(surface, view.id()),
            surface: Rc::downgrade(surface),
            container: Cell::new(None),
            controller: RefCell::new(None),
            webview: RefCell::new(None),
            failed: Cell::new(false),
            props: RefCell::new(None),
            applied: RefCell::new(None),
            queued: RefCell::new(Vec::new()),
            scripts: Rc::default(),
            default_agent: RefCell::new(None),
            target: Cell::new(None),
            placement: Placement::default(),
            bounds: Cell::new(None),
            background: Cell::new(view.state().style.background_color),
        });
        match make_container(surface.hwnd()) {
            Some(container) => {
                item.container.set(Some(container));
                let folder = profile_folder(&surface.native.app_id.borrow());
                let weak = Rc::downgrade(&item);
                with_environment(&folder, move |env| {
                    if let Some(item) = weak.upgrade() {
                        item.environment_ready(env);
                    }
                });
            }
            None => item.unavailable("cannot create its window"),
        }
        item
    }

    /// No web view: the placeholder shows.
    fn unavailable(&self, why: &str) {
        self.failed.set(true);
        static WARNED: AtomicBool = AtomicBool::new(false);
        if !WARNED.swap(true, Ordering::Relaxed) {
            log::warn!(
                target: "js",
                "WebView is not available ({why}; is the WebView2 runtime installed?): it shows a placeholder"
            );
        }
        if let Some(surface) = self.surface.upgrade() {
            surface.invalidate();
        }
    }

    fn environment_ready(self: &Rc<Self>, env: Option<ICoreWebView2Environment>) {
        let (Some(env), Some(container)) = (env, self.container.get()) else {
            if self.container.get().is_some() {
                self.unavailable("no WebView2 environment");
            }
            return;
        };
        let weak = Rc::downgrade(self);
        let handler = CreateCoreWebView2ControllerCompletedHandler::create(Box::new(
            move |result, controller| {
                let Some(item) = weak.upgrade() else {
                    // The view went away meanwhile.
                    if let Some(c) = controller {
                        // SAFETY: a call on the controller just made.
                        let _ = unsafe { c.Close() };
                    }
                    return Ok(());
                };
                match (result, controller) {
                    (Ok(()), Some(controller)) => item.ready(controller),
                    (result, _) => {
                        log::warn!("web view: no controller: {result:?}");
                        item.unavailable("cannot create a WebView2 controller");
                    }
                }
                Ok(())
            },
        ));
        // SAFETY: a call on a live environment with a live window of this
        // thread.
        if let Err(err) = unsafe { env.CreateCoreWebView2Controller(container, &handler) } {
            log::warn!("web view: cannot create a controller: {err}");
            self.unavailable("cannot create a WebView2 controller");
        }
    }

    /// The controller is made: sets it up, applies what waited for it.
    fn ready(self: &Rc<Self>, controller: ICoreWebView2Controller) {
        if self.container.get().is_none() {
            // SAFETY: closed before it was ready: the controller goes too.
            let _ = unsafe { controller.Close() };
            return;
        }
        // SAFETY: WebView2 calls on the live controller, on its thread.
        let webview = match unsafe { controller.CoreWebView2() } {
            Ok(webview) => webview,
            Err(err) => {
                log::warn!("web view: {err}");
                return self.unavailable("no WebView2 core");
            }
        };
        // SAFETY: as above.
        unsafe {
            if let Ok(settings) = webview.Settings() {
                let _ = settings.SetIsStatusBarEnabled(false);
                if let Ok(settings) = settings.cast::<ICoreWebView2Settings2>() {
                    let mut agent = PWSTR::null();
                    if settings.UserAgent(&mut agent).is_ok() {
                        *self.default_agent.borrow_mut() = Some(take_pwstr(agent));
                    }
                }
            }
            let _ = webview.AddScriptToExecuteOnDocumentCreated(
                &HSTRING::from(BRIDGE),
                None::<&webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2AddScriptToExecuteOnDocumentCreatedCompletedHandler>,
            );
        }
        self.listen(&webview);
        *self.controller.borrow_mut() = Some(controller);
        *self.webview.borrow_mut() = Some(webview);
        self.paint_background();
        self.bounds.set(None);
        self.place();
        let props = self.props.borrow().clone();
        if let Some(props) = props {
            self.apply(&props);
        }
        let queued = std::mem::take(&mut *self.queued.borrow_mut());
        for command in &queued {
            self.command(command);
        }
    }

    /// Reports the web view's navigations and messages.
    fn listen(self: &Rc<Self>, webview: &ICoreWebView2) {
        let mut token = 0i64;
        let weak = Rc::downgrade(self);
        let starting = NavigationStartingEventHandler::create(Box::new(move |sender, args| {
            let (Some(item), Some(sender), Some(args)) = (weak.upgrade(), sender, args) else {
                return Ok(());
            };
            let mut uri = PWSTR::null();
            // SAFETY: WebView2 calls on the event's live objects.
            let url = unsafe { args.Uri(&mut uri).map(|()| take_pwstr(uri)) }.unwrap_or_default();
            if !item.allows(&url) {
                // SAFETY: as above.
                let _ = unsafe { args.SetCancel(true) };
                log::warn!(target: "js", "web view navigation to {url} is not granted");
                let message = format!("navigation to {url} is not granted");
                item.emitter.emit(EventKind::Error, || {
                    EventPayload::Map(vec![
                        ("message", PayloadValue::Str(message)),
                        ("url", PayloadValue::Str(url.clone())),
                    ])
                });
                return Ok(());
            }
            let mut state = navigation(&sender, true);
            state.url = url;
            item.emitter.emit(EventKind::LoadStart, || state.payload());
            item.emitter
                .emit(EventKind::NavigationStateChange, || state.payload());
            Ok(())
        }));
        let weak = Rc::downgrade(self);
        let completed = NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
            let (Some(item), Some(sender), Some(args)) = (weak.upgrade(), sender, args) else {
                return Ok(());
            };
            let mut success = BOOL(0);
            let mut status = COREWEBVIEW2_WEB_ERROR_STATUS(0);
            // SAFETY: WebView2 calls on the event's live objects.
            unsafe {
                let _ = args.IsSuccess(&mut success);
                let _ = args.WebErrorStatus(&mut status);
            }
            if success.as_bool() {
                let state = navigation(&sender, false);
                item.emitter.emit(EventKind::Load, || state.payload());
                item.emitter
                    .emit(EventKind::NavigationStateChange, || state.payload());
            } else if let Some(message) = web_error(status.0) {
                item.emitter
                    .emit(EventKind::Error, || error(message.to_owned()));
            }
            Ok(())
        }));
        let weak = Rc::downgrade(self);
        let message = WebMessageReceivedEventHandler::create(Box::new(move |_, args| {
            let (Some(item), Some(args)) = (weak.upgrade(), args) else {
                return Ok(());
            };
            let mut text = PWSTR::null();
            // SAFETY: WebView2 calls on the event's live object; a message
            // that is not a string comes as JSON.
            let data = unsafe {
                match args.TryGetWebMessageAsString(&mut text) {
                    Ok(()) => take_pwstr(text),
                    Err(_) => {
                        let mut json = PWSTR::null();
                        args.WebMessageAsJson(&mut json)
                            .map(|()| take_pwstr(json))
                            .unwrap_or_default()
                    }
                }
            };
            item.emitter.emit(EventKind::Message, || {
                EventPayload::Map(vec![("data", PayloadValue::Str(data))])
            });
            Ok(())
        }));
        // SAFETY: WebView2 calls on the live web view; the handlers hold the
        // item weakly, and go with the web view.
        unsafe {
            let _ = webview.add_NavigationStarting(&starting, &mut token);
            let _ = webview.add_NavigationCompleted(&completed, &mut token);
            let _ = webview.add_WebMessageReceived(&message, &mut token);
        }
    }

    /// True when the app may show `url` at the top level.
    fn allows(&self, url: &str) -> bool {
        // Pages the app gave as HTML.
        if url.starts_with("data:") {
            return true;
        }
        let Some(surface) = self.surface.upgrade() else {
            return false;
        };
        let access = surface.native.access.borrow().clone();
        access.is_none_or(|a: Arc<nsh_engine::Access>| a.allows_webview(url))
    }

    pub(crate) fn set(&self, surface: &Rc<Surface>, view: &NshView, p: &WebProps) {
        self.background.set(view.state().style.background_color);
        *self.props.borrow_mut() = Some(p.clone());
        if self.webview.borrow().is_some() {
            self.paint_background();
            self.apply(p);
        }
        surface.invalidate();
    }

    /// Applies `p` to the web view (it is ready).
    fn apply(&self, p: &WebProps) {
        let Some(webview) = self.webview.borrow().clone() else {
            return;
        };
        let old = self.applied.borrow_mut().replace(p.clone());
        let scripts_changed = old.as_ref().is_none_or(|o| {
            o.injected_js != p.injected_js || o.injected_js_before != p.injected_js_before
        });
        // SAFETY: WebView2 calls on the live web view, on its thread.
        unsafe {
            if let Ok(settings) = webview.Settings() {
                let _ = settings.SetIsScriptEnabled(p.javascript_enabled);
                if let Ok(settings) = settings.cast::<ICoreWebView2Settings2>() {
                    let agent = p
                        .user_agent
                        .as_deref()
                        .map(str::to_owned)
                        .or_else(|| self.default_agent.borrow().clone());
                    if let Some(agent) = agent {
                        let _ = settings.SetUserAgent(&HSTRING::from(agent));
                    }
                }
            }
            if scripts_changed {
                for id in self.scripts.borrow_mut().drain(..) {
                    let _ = webview.RemoveScriptToExecuteOnDocumentCreated(&HSTRING::from(id));
                }
                let scripts = [
                    p.injected_js_before.as_deref().map(before_content),
                    p.injected_js.as_deref().map(after_content),
                ];
                for script in scripts.into_iter().flatten() {
                    let ids = Rc::downgrade(&self.scripts);
                    let handler = AddScriptToExecuteOnDocumentCreatedCompletedHandler::create(
                        Box::new(move |result, id| {
                            if let (Ok(()), Some(ids)) = (result, ids.upgrade()) {
                                ids.borrow_mut().push(id);
                            }
                            Ok(())
                        }),
                    );
                    let _ = webview
                        .AddScriptToExecuteOnDocumentCreated(&HSTRING::from(script), &handler);
                }
            }
            let navigate = scripts_changed || old.as_ref().is_none_or(|o| o.source != p.source);
            if navigate {
                let result = match &p.source {
                    WebSource::Uri(uri) => webview.Navigate(&HSTRING::from(&**uri)),
                    WebSource::Html { html, base_url } => {
                        if base_url.is_some() {
                            static WARNED: AtomicBool = AtomicBool::new(false);
                            if !WARNED.swap(true, Ordering::Relaxed) {
                                log::warn!(
                                    target: "js",
                                    "WebView `source.baseUrl` is not supported on Windows: relative links do not resolve"
                                );
                            }
                        }
                        webview.NavigateToString(&HSTRING::from(&**html))
                    }
                    WebSource::None => webview.Navigate(w!("about:blank")),
                };
                if let Err(err) = result {
                    let message = err.message();
                    self.emitter.emit(EventKind::Error, || error(message));
                }
            }
        }
    }

    /// `reload()`, `goBack()`, `goForward()`, `stopLoading()`,
    /// `injectJavaScript()` and `postMessage()`.
    pub(crate) fn command(&self, command: &Command) {
        let Some(webview) = self.webview.borrow().clone() else {
            if !self.failed.get() {
                self.queued.borrow_mut().push(command.clone());
            }
            return;
        };
        let none = None::<&ICoreWebView2ExecuteScriptCompletedHandler>;
        // SAFETY: WebView2 calls on the live web view, on its thread.
        let _ = unsafe {
            match command {
                Command::Reload => webview.Reload(),
                Command::GoBack => webview.GoBack(),
                Command::GoForward => webview.GoForward(),
                Command::StopLoading => webview.Stop(),
                Command::InjectJavaScript(js) => webview.ExecuteScript(&HSTRING::from(js), none),
                Command::PostMessage(data) => {
                    webview.ExecuteScript(&HSTRING::from(post_message(data)), none)
                }
                _ => Ok(()),
            }
        };
    }

    /// The view's background behind the page until it paints (when the
    /// view has an opaque one).
    fn paint_background(&self) {
        let color = self.background.get();
        let Some(controller) = self.controller.borrow().clone() else {
            return;
        };
        let (r, g, b, a) = color.to_f64();
        if a < 1.0 {
            return;
        }
        let byte = |v: f64| (v * 255.0).round() as u8;
        // SAFETY: a WebView2 call on the live controller.
        unsafe {
            if let Ok(c) = controller.cast::<ICoreWebView2Controller2>() {
                let _ = c.SetDefaultBackgroundColor(COREWEBVIEW2_COLOR {
                    A: 255,
                    R: byte(r),
                    G: byte(g),
                    B: byte(b),
                });
            }
        }
    }

    /// Moves the container where the view was drawn, and the controller
    /// with it.
    fn place(&self) {
        let (Some(container), Some(surface)) = (self.container.get(), self.surface.upgrade())
        else {
            return;
        };
        let before = self.placement.last.get();
        let size = self
            .placement
            .apply(&surface, self.emitter.id, container, self.target.get());
        let Some(controller) = self.controller.borrow().clone() else {
            return;
        };
        if self.bounds.replace(size) != size {
            // SAFETY: WebView2 calls on the live controller.
            unsafe {
                if let Some((w, h)) = size {
                    let _ = controller.SetBounds(RECT {
                        left: 0,
                        top: 0,
                        right: w,
                        bottom: h,
                    });
                }
                let _ = controller.SetIsVisible(size.is_some());
            }
        }
        if before != self.placement.last.get() && size.is_some() {
            // Pop-ups (select menus, tooltips) follow the page.
            // SAFETY: as above.
            let _ = unsafe { controller.NotifyParentWindowPositionChanged() };
        }
    }

    /// The view was deleted or the app stopped: the page closes.
    pub(crate) fn close(&self) {
        self.webview.borrow_mut().take();
        self.queued.borrow_mut().clear();
        if let Some(controller) = self.controller.borrow_mut().take() {
            // SAFETY: closes the live controller; it is not used afterwards.
            let _ = unsafe { controller.Close() };
        }
        if let Some(container) = self.container.take() {
            // SAFETY: destroys the window this item made.
            let _ = unsafe { DestroyWindow(container) };
        }
    }
}

impl Hosted for WebItem {
    fn draw(&self, cx: &mut DrawCx<'_>, size: (f32, f32)) {
        if !self.failed.get() {
            return;
        }
        let Some(format) = placeholder_format() else {
            return;
        };
        let text: Vec<u16> = "WebView is not available on this system"
            .encode_utf16()
            .collect();
        cx.push_clip_rect(0.0, 0.0, size.0, size.1);
        // SAFETY: a Direct2D call between BeginDraw and EndDraw.
        unsafe {
            cx.dc.DrawText(
                &text,
                &format,
                &rect_f(8.0, 0.0, (size.0 - 16.0).max(0.0), size.1),
                cx.solid(Color::rgba(140, 140, 140, 255)),
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
        cx.pop();
    }

    fn hit(&self, _x: f32, _y: f32, _size: (f32, f32)) -> bool {
        // The page's window takes its own input.
        false
    }

    fn placed(&self, rect: Option<RECT>) {
        self.target.set(rect);
        self.place();
    }
}

/// The page's navigation state, as `WebViewNavigation`.
#[derive(Debug, Clone, Default, PartialEq)]
struct Navigation {
    url: String,
    title: String,
    loading: bool,
    can_go_back: bool,
    can_go_forward: bool,
}

impl Navigation {
    fn payload(&self) -> EventPayload {
        EventPayload::Map(vec![
            ("url", PayloadValue::Str(self.url.clone())),
            ("title", PayloadValue::Str(self.title.clone())),
            ("loading", PayloadValue::Bool(self.loading)),
            ("canGoBack", PayloadValue::Bool(self.can_go_back)),
            ("canGoForward", PayloadValue::Bool(self.can_go_forward)),
        ])
    }
}

/// The navigation state of `webview`.
fn navigation(webview: &ICoreWebView2, loading: bool) -> Navigation {
    let mut url = PWSTR::null();
    let mut title = PWSTR::null();
    let (mut back, mut forward) = (BOOL(0), BOOL(0));
    // SAFETY: WebView2 getters on the live web view; the strings are freed
    // by `take_pwstr`.
    unsafe {
        let url = webview
            .Source(&mut url)
            .map(|()| take_pwstr(url))
            .unwrap_or_default();
        let title = webview
            .DocumentTitle(&mut title)
            .map(|()| take_pwstr(title))
            .unwrap_or_default();
        let _ = webview.CanGoBack(&mut back);
        let _ = webview.CanGoForward(&mut forward);
        Navigation {
            url,
            title,
            loading,
            can_go_back: back.as_bool(),
            can_go_forward: forward.as_bool(),
        }
    }
}

/// What a failed navigation reports, `None` for one that was replaced or
/// cancelled (not an error).
fn web_error(status: i32) -> Option<&'static str> {
    Some(match status {
        1 => "The certificate's common name does not match the host.",
        2 => "The certificate has expired.",
        3 => "The client certificate contains errors.",
        4 => "The certificate has been revoked.",
        5 => "The certificate is invalid.",
        6 => "The server is unreachable.",
        7 => "The connection timed out.",
        8 => "The server returned an invalid response.",
        9 => "The connection was aborted.",
        10 => "The connection was reset.",
        11 => "The internet connection was lost.",
        12 => "Cannot connect to the server.",
        13 => "The host name could not be resolved.",
        14 => return None,
        15 => "A redirect failed.",
        17 => "Authentication is required.",
        18 => "Proxy authentication is required.",
        _ => "The page could not be loaded.",
    })
}

/// A JS string literal for `s`.
fn js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The script `postMessage(data)` runs in the page.
fn post_message(data: &str) -> String {
    format!(
        "window.dispatchEvent(new MessageEvent('message', {{ data: {} }}));",
        js_string(data)
    )
}

/// `injectedJavaScriptBeforeContentLoaded`: in the main frame, before the
/// page's own scripts (WebView2 runs document scripts in every frame).
fn before_content(js: &str) -> String {
    format!("if (window === window.top) {{\n{js}\n}}")
}

/// `injectedJavaScript`: in the main frame, once the document is parsed.
fn after_content(js: &str) -> String {
    format!(
        "if (window === window.top) {{ (function () {{ var run = function () {{\n{js}\n}}; \
         if (document.readyState === 'loading') {{ document.addEventListener('DOMContentLoaded', run, {{ once: true }}); }} \
         else {{ run(); }} }})(); }}"
    )
}

/// Where web views of app `app_id` keep their profile.
fn profile_folder(app_id: &str) -> String {
    let base = std::env::var("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let app = if app_id.is_empty() { "nsh" } else { app_id };
    // An id is a reverse domain name: keep it a single, valid folder name.
    let app: String = app
        .chars()
        .map(|c| if r#"\/:*?"<>|"#.contains(c) { '_' } else { c })
        .collect();
    base.join(app)
        .join("WebView2")
        .to_string_lossy()
        .into_owned()
}

/// Waiting for an environment.
type EnvCallback = Box<dyn FnOnce(Option<ICoreWebView2Environment>)>;

enum EnvState {
    Pending(Vec<EnvCallback>),
    Ready(ICoreWebView2Environment),
    Failed,
}

thread_local! {
    /// WebView2 environments by profile folder: one browser process
    /// serves every web view of a profile.
    static ENVIRONMENTS: RefCell<HashMap<String, EnvState>> = RefCell::default();
}

/// Calls `f` with the environment of profile `folder` once it is made
/// (`None` when it cannot be).
fn with_environment(folder: &str, f: impl FnOnce(Option<ICoreWebView2Environment>) + 'static) {
    let known = ENVIRONMENTS.with(|envs| match envs.borrow().get(folder) {
        Some(EnvState::Ready(env)) => Some(Some(env.clone())),
        Some(EnvState::Failed) => Some(None),
        _ => None,
    });
    if let Some(env) = known {
        return f(env);
    }
    let start = ENVIRONMENTS.with(|envs| {
        let mut envs = envs.borrow_mut();
        if let Some(EnvState::Pending(waiting)) = envs.get_mut(folder) {
            waiting.push(Box::new(f));
            false
        } else {
            envs.insert(folder.to_owned(), EnvState::Pending(vec![Box::new(f)]));
            true
        }
    });
    // Outside the borrow: a loader that fails at once calls back here.
    if start {
        start_environment(folder);
    }
}

/// Starts making the environment of profile `folder`.
fn start_environment(folder: &str) {
    let key = folder.to_owned();
    let handler =
        CreateCoreWebView2EnvironmentCompletedHandler::create(Box::new(move |result, env| {
            if let Err(err) = &result {
                log::warn!("web view: no WebView2 environment: {err}");
            }
            let env = result.ok().and(env);
            let waiting = ENVIRONMENTS.with(|envs| {
                let mut envs = envs.borrow_mut();
                let state = match &env {
                    Some(env) => EnvState::Ready(env.clone()),
                    None => EnvState::Failed,
                };
                match envs.insert(key.clone(), state) {
                    Some(EnvState::Pending(waiting)) => waiting,
                    _ => Vec::new(),
                }
            });
            for f in waiting {
                f(env.clone());
            }
            Ok(())
        }));
    let folder = HSTRING::from(folder);
    // SAFETY: valid strings; the handler is an owned COM object the loader
    // keeps until it calls it.
    let result = unsafe {
        create_environment(
            PCWSTR::null(),
            PCWSTR(folder.as_ptr()),
            std::ptr::null_mut(),
            windows::core::Interface::as_raw(&handler),
        )
    };
    if let Err(err) = result.ok() {
        log::warn!("web view: cannot start WebView2: {err}");
        let waiting = ENVIRONMENTS.with(|envs| {
            match envs
                .borrow_mut()
                .insert(folder.to_string(), EnvState::Failed)
            {
                Some(EnvState::Pending(waiting)) => waiting,
                _ => Vec::new(),
            }
        });
        for f in waiting {
            f(None);
        }
    }
}

/// A hidden child window of `parent` for a controller to fill.
fn make_container(parent: HWND) -> Option<HWND> {
    if parent.is_invalid() {
        return None;
    }
    static REGISTERED: std::sync::Once = std::sync::Once::new();
    let class = w!("nsh-native-host");
    // SAFETY: plain window calls; the class's procedure is the default
    // one, and the window is destroyed by `WebItem::close`.
    unsafe {
        let instance = GetModuleHandleW(None).ok()?;
        REGISTERED.call_once(|| {
            let wc = WNDCLASSW {
                lpfnWndProc: Some(container_proc),
                hInstance: instance.into(),
                lpszClassName: class,
                ..Default::default()
            };
            RegisterClassW(&wc);
        });
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class,
            w!(""),
            WS_CHILD | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
            0,
            0,
            1,
            1,
            Some(parent),
            None,
            Some(instance.into()),
            None,
        )
        .map_err(|err| log::warn!("web view: cannot create its window: {err}"))
        .ok()
    }
}

unsafe extern "system" fn container_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: default handling: the controller's window paints the page.
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// The placeholder's text format, made once per thread.
fn placeholder_format() -> Option<IDWriteTextFormat> {
    thread_local! {
        static FORMAT: Option<IDWriteTextFormat> = {
            // SAFETY: plain DirectWrite calls with valid strings.
            unsafe {
                gfx::dwrite()
                    .CreateTextFormat(
                        &HSTRING::from("Segoe UI"),
                        None,
                        DWRITE_FONT_WEIGHT_NORMAL,
                        DWRITE_FONT_STYLE_NORMAL,
                        DWRITE_FONT_STRETCH_NORMAL,
                        14.0,
                        &HSTRING::from(""),
                    )
                    .ok()
                    .inspect(|f| {
                        let _ = f.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                        let _ = f.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                        let _ = f.SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP);
                    })
            }
        };
    }
    FORMAT.with(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_js_strings() {
        assert_eq!(js_string("a\"b\\c\nd\u{1}"), r#""a\"b\\c\nd\u0001""#);
        assert_eq!(
            post_message("hi #1"),
            r#"window.dispatchEvent(new MessageEvent('message', { data: "hi #1" }));"#
        );
    }

    #[test]
    fn wraps_injected_scripts() {
        assert!(before_content("x()").starts_with("if (window === window.top) {\nx()"));
        let after = after_content("y()");
        assert!(after.contains("DOMContentLoaded") && after.contains("\ny()\n"));
    }

    #[test]
    fn makes_navigation_payloads() {
        let state = Navigation {
            url: "https://example.com/".into(),
            title: "Example".into(),
            loading: false,
            can_go_back: true,
            can_go_forward: false,
        };
        assert_eq!(
            state.payload(),
            EventPayload::Map(vec![
                ("url", PayloadValue::Str("https://example.com/".into())),
                ("title", PayloadValue::Str("Example".into())),
                ("loading", PayloadValue::Bool(false)),
                ("canGoBack", PayloadValue::Bool(true)),
                ("canGoForward", PayloadValue::Bool(false)),
            ])
        );
    }

    #[test]
    fn words_web_errors() {
        assert_eq!(web_error(13), Some("The host name could not be resolved."));
        assert_eq!(web_error(14), None);
        assert_eq!(web_error(99), Some("The page could not be loaded."));
    }

    #[test]
    fn names_profile_folders() {
        let folder = profile_folder("com.example.app");
        assert!(folder.ends_with(r"com.example.app\WebView2"), "{folder}");
        assert!(profile_folder("a:b").contains("a_b"));
    }
}
