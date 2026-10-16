//! `AppView` (one surface in a window, for embedding) and the standalone
//! app ([`run_app`]: one `AppView` in a top-level window and the message
//! loop).

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use nsh_core::platform::Appearance;
use nsh_engine::{App, Bundle, ColorScheme, EngineConfig, Environment, Idiom, Limits, Os, Root};
use windows::{
    Win32::{
        Foundation::HWND,
        System::Ole::OleInitialize,
        UI::{
            HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext},
            WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW},
        },
    },
    core::HSTRING,
};

use crate::{
    image::{AssetBase, AssetReader},
    surface::Surface,
};

static UI_SETS: RwLock<Option<Vec<nsh_core::ui::Registry>>> = RwLock::new(None);

/// Sets the UI sets every engine serves (default: `nsh/ui/mobile`); an app
/// with Rust components of its own calls this before [`run_app`].
pub fn set_ui_sets(sets: Vec<nsh_core::ui::Registry>) {
    *UI_SETS.write().unwrap_or_else(|e| e.into_inner()) = Some(sets);
}

/// The UI sets engines serve.
pub(crate) fn ui_sets() -> Vec<nsh_core::ui::Registry> {
    UI_SETS
        .read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_else(|| vec![nsh_ui_mobile::registry()])
}

/// What [`run_app`] runs.
#[derive(Debug)]
pub enum AppSource {
    /// A Rust app: its root component.
    Rust(Root),
    /// A JS bundle shipped with (or passed to) the app.
    Bundle(Bundle),
    /// A dev server (`ws://host:port/ws`); bundles are pushed on every
    /// rebuild.
    #[cfg(feature = "dev")]
    DevServer(String),
}

/// Options of an [`AppView`] and [`run_app`].
#[derive(Debug, Clone)]
pub struct AppOptions {
    /// Log per-commit timings.
    pub trace: bool,
    /// Where bundled assets are (for [`AppSource::Bundle`]); dev servers
    /// serve them themselves.
    pub asset_base: Option<AssetBase>,
    /// The application id (`com.example.app`): the taskbar groups the app's
    /// windows by it (`AppUserModelID`).
    pub app_id: String,
    /// The window title.
    pub title: String,
    /// The initial window size in points.
    pub size: (i32, i32),
    /// Steps run once the window shows (`wait 500; shot a.png; tap 10 20;
    /// quit`): for checking a backend change without touching the screen.
    /// See [`crate::script`].
    pub script: Option<String>,
    /// The native components apps can use (`nsh/native`); shared by the
    /// views made with these options. See [`AppView::register_component`].
    pub components: crate::native::Components,
    /// The app's appearance until it sets one (`nsh.json` `"appearance"`,
    /// which the bundle sets again when it starts): known before the
    /// first frame, the window opens in it rather than flashing the
    /// system's.
    pub appearance: Appearance,
}

impl Default for AppOptions {
    fn default() -> Self {
        AppOptions {
            trace: false,
            asset_base: None,
            app_id: "com.longbridge.nsh.host".to_owned(),
            title: "nsh".to_owned(),
            size: (1200, 800),
            script: None,
            components: Default::default(),
            appearance: Appearance::System,
        }
    }
}

/// One nsh surface in a window: an engine running one app.
///
/// Embedded in another app's window, it is a child window the host
/// positions (`MoveWindow` on its `hwnd()`); the host's message loop runs
/// it (COM must be initialized on the thread, `OleInitialize` for the
/// clipboard).
///
/// ```no_run
/// use nsh_platform_windows::{AppOptions, AppView, AssetBase, Bundle, windows::Win32::Foundation::HWND};
///
/// # fn host(parent: HWND) -> windows::core::Result<()> {
/// let view = AppView::new(
///     Some(parent),
///     AppOptions {
///         asset_base: Some(AssetBase::Directory("app/assets".into())),
///         ..AppOptions::default()
///     },
/// )?;
/// let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
/// view.load(Bundle::source(code).into());
/// // On WM_SIZE of `parent`: MoveWindow(view.hwnd(), 0, 0, width, height, true).
/// // Destroying the window (or `view.stop()`) stops the app.
/// # Ok(())
/// # }
/// ```
pub struct AppView {
    surface: Rc<Surface>,
    hwnd: HWND,
    options: RefCell<AppOptions>,
    #[cfg(feature = "dev")]
    pub(crate) dev: RefCell<Option<crate::dev::DevSession>>,
}

impl AppView {
    /// A view with no app yet: a child window of `parent`, or a top-level
    /// window (hidden until [`AppView::show`]) when `None`.
    pub fn new(parent: Option<HWND>, options: AppOptions) -> windows::core::Result<Rc<AppView>> {
        let surface = Surface::new(initial_environment());
        surface.appearance.set(options.appearance);
        surface.native.set_components(options.components.clone());
        surface.native.set_app_id(&options.app_id);
        let hwnd = crate::window::create(&surface, &options.title, options.size, parent)?;
        let view = Rc::new(AppView {
            surface,
            hwnd,
            options: RefCell::new(options),
            #[cfg(feature = "dev")]
            dev: RefCell::new(None),
        });
        crate::system::watch_environment(&view.surface);
        Ok(view)
    }

    /// The view's window.
    pub fn hwnd(&self) -> HWND {
        self.hwnd
    }

    /// Shows a top-level view's window.
    pub fn show(&self) {
        crate::window::show(self.hwnd);
    }

    /// Runs `app`, replacing what ran.
    pub fn load(&self, app: App) {
        let access = match &app {
            App::Js(bundle) => bundle.policy.access(),
            // Native code reaches what the platform allows.
            App::Rust(_) => nsh_engine::Policy::default().access(),
        };
        self.surface.native.set_access(Arc::new(access));
        let options = self.options.borrow();
        let config = EngineConfig {
            app,
            viewport: nsh_core::Size::new(0.0, 0.0),
            trace: options.trace,
            network: Some(crate::net::network()),
            assets: Some(Arc::new(AssetReader(options.asset_base.clone()))),
            environment: self.surface.environment.get(),
            header_heights: crate::navigation::HEADER_HEIGHTS,
            limits: Limits::default(),
            ui_sets: ui_sets(),
            runtime: None,
        };
        crate::image::set_asset_base(&self.surface, options.asset_base.clone());
        self.surface
            .native
            .set_asset_base(options.asset_base.clone());
        self.surface.start(config);
    }

    /// Where `asset:/` URIs resolve from now on.
    pub fn set_asset_base(&self, base: Option<AssetBase>) {
        self.options.borrow_mut().asset_base = base;
    }

    /// Registers native component `name` (`nativeComponent(name, ...)` in
    /// JS): `factory` makes one for every element. Applies to elements
    /// created from now on.
    pub fn register_component(&self, name: &str, factory: crate::native::ComponentFactory) {
        self.surface.native.components().register(name, factory);
    }

    /// Stops the app.
    pub fn stop(&self) {
        self.surface.stop();
    }

    pub(crate) fn surface(&self) -> &Rc<Surface> {
        &self.surface
    }
}

fn initial_environment() -> Environment {
    Environment {
        window: (0.0, 0.0),
        color_scheme: ColorScheme::Light,
        os: Os::Windows,
        idiom: Idiom::Desktop,
        ..Environment::default()
    }
}

/// Runs `source` in a window until it is closed.
pub fn run_app(source: AppSource, options: AppOptions) -> std::process::ExitCode {
    // SAFETY: process setup before any window exists: per-monitor DPI
    // (points stay sharp on every monitor) and an OLE apartment for the
    // clipboard, drag and drop and WIC.
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let _ = OleInitialize(None);
    }
    crate::system::set_app_id(&options.app_id);
    let view = match AppView::new(None, options.clone()) {
        Ok(view) => view,
        Err(err) => {
            fatal_error(&format!("Cannot open a window: {err}"));
            return std::process::ExitCode::FAILURE;
        }
    };
    match source {
        AppSource::Rust(root) => view.load(App::Rust(root)),
        AppSource::Bundle(bundle) => view.load(App::Js(bundle)),
        #[cfg(feature = "dev")]
        AppSource::DevServer(url) => crate::dev::start(&view, url),
    }
    crate::system::environment_applied(view.surface());
    view.show();
    if let Some(script) = &options.script {
        crate::script::run(view.surface().clone(), script);
    }
    let code = crate::window::run_loop();
    view.stop();
    drop(view);
    if code == 0 {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}

/// Writes `text` to the debugger's output (`OutputDebugString`): where a
/// windowed program's logs go.
pub fn debug_output(text: &str) {
    use windows::Win32::System::Diagnostics::Debug::OutputDebugStringW;
    // SAFETY: a valid, NUL-terminated string.
    unsafe { OutputDebugStringW(&HSTRING::from(text)) };
}

/// Shows `message` in an error box (a windowed program has no console).
pub fn fatal_error(message: &str) {
    log::error!("{message}");
    // SAFETY: valid strings; no owner window.
    unsafe {
        MessageBoxW(
            None,
            &HSTRING::from(message),
            &HSTRING::from("nsh"),
            MB_OK | MB_ICONERROR,
        );
    }
}
