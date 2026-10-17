//! `AppView` (one surface as a GPUI view, for embedding in any GPUI or
//! GPUI Kit app) and the standalone app ([`run_app`]: one `AppView`
//! filling a window).

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, RwLock},
};

use gpui::{
    App as GpuiApp, AppContext, Context, Entity, FocusHandle, Focusable, IntoElement, Render,
    Window,
};
use nsh_core::platform::Appearance;
use nsh_engine::{App, Bundle, ColorScheme, EngineConfig, Environment, Idiom, Limits, Os, Root};

use crate::{
    image::{AssetBase, AssetReader},
    render::SurfaceElement,
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
    /// The application id (`com.example.app`).
    pub app_id: String,
    /// The window title.
    pub title: String,
    /// The initial window size in points.
    pub size: (i32, i32),
    /// Steps run once the window shows (`wait 500; shot a.png; tap 10 20;
    /// quit`): for checking a backend change without touching the screen.
    pub script: Option<String>,
    /// The native components apps can use (`nsh/native`); shared by the
    /// views made with these options. See [`AppView::register_component`].
    pub components: crate::native::Components,
    /// The app's appearance until it sets one (`nsh.json` `"appearance"`).
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

/// One nsh surface as a GPUI view: an engine running one app, drawn by
/// GPUI wherever the embedding app puts the view (it fills the space its
/// parent element gives it).
///
/// ```no_run
/// use gpui::{AppContext, Application, WindowOptions};
/// use nsh_platform_gpui::{App, AppOptions, AppView, Bundle};
///
/// gpui_platform::application().run(|cx| {
///     cx.open_window(WindowOptions::default(), |window, cx| {
///         cx.new(|cx| {
///             let view = AppView::new(AppOptions::default(), window, cx);
///             let code = std::fs::read_to_string("app/bundle.js").expect("bundle");
///             view.load(App::Js(Bundle::source(code)));
///             view
///         })
///     })
///     .unwrap();
/// });
/// ```
pub struct AppView {
    surface: Rc<Surface>,
    focus: FocusHandle,
    options: RefCell<AppOptions>,
    #[cfg(feature = "dev")]
    pub(crate) dev: RefCell<Option<crate::dev::DevSession>>,
}

impl AppView {
    /// A view with no app yet (in `cx.new(|cx| AppView::new(...))`).
    pub fn new(options: AppOptions, window: &mut Window, cx: &mut Context<Self>) -> AppView {
        crate::rt::init(cx);
        // The focused field's caret follows the window's activation
        // (`text_input::window_focus`, checked as the view is drawn).
        cx.observe_window_activation(window, |_, _, cx| cx.notify())
            .detach();
        let surface = Surface::new(initial_environment());
        surface.appearance.set(options.appearance);
        surface.entity.set(Some(cx.entity_id()));
        *surface.text_system.borrow_mut() = Some(cx.text_system().clone());
        surface.native.set_components(options.components.clone());
        AppView {
            surface,
            focus: cx.focus_handle(),
            options: RefCell::new(options),
            #[cfg(feature = "dev")]
            dev: RefCell::new(None),
        }
    }

    /// Runs `app`, replacing what ran.
    pub fn load(&self, app: App) {
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

impl Drop for AppView {
    fn drop(&mut self) {
        self.surface.stop();
    }
}

impl Focusable for AppView {
    fn focus_handle(&self, _cx: &GpuiApp) -> FocusHandle {
        self.focus.clone()
    }
}

/// The window's input handler while a field of the surface has the focus
/// (registered by the surface's element): text and input methods go to
/// that field ([`crate::text_input::ime`]).
impl gpui::EntityInputHandler for AppView {
    fn text_for_range(
        &mut self,
        range: std::ops::Range<usize>,
        adjusted: &mut Option<std::ops::Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        crate::text_input::ime::text_for_range(&self.surface, range, adjusted)
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::UTF16Selection> {
        crate::text_input::ime::selected_text_range(&self.surface)
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<std::ops::Range<usize>> {
        crate::text_input::ime::marked_text_range(&self.surface)
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        crate::text_input::ime::unmark_text(&self.surface);
        crate::rt::flush(cx);
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<std::ops::Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        crate::text_input::ime::replace_text_in_range(&self.surface, range, text);
        crate::rt::flush(cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<std::ops::Range<usize>>,
        text: &str,
        selected: Option<std::ops::Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        crate::text_input::ime::replace_and_mark_text_in_range(
            &self.surface,
            range,
            text,
            selected,
        );
        crate::rt::flush(cx);
    }

    fn bounds_for_range(
        &mut self,
        range: std::ops::Range<usize>,
        _element_bounds: gpui::Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<gpui::Bounds<gpui::Pixels>> {
        crate::text_input::ime::bounds_for_range(&self.surface, range)
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        crate::text_input::ime::character_index_for_point(&self.surface, point)
    }

    fn set_selected_text_range(
        &mut self,
        range: std::ops::Range<usize>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        crate::text_input::ime::set_selected_text_range(&self.surface, range);
        crate::rt::flush(cx);
    }

    fn text_length_utf16(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        crate::text_input::ime::text_length_utf16(&self.surface)
    }

    fn text_input_configuration(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> gpui::TextInputConfiguration {
        crate::text_input::ime::configuration(&self.surface)
    }
}

impl Render for AppView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        SurfaceElement {
            surface: self.surface.clone(),
            focus: self.focus.clone(),
            view: cx.entity().downgrade(),
        }
    }
}

fn initial_environment() -> Environment {
    Environment {
        window: (0.0, 0.0),
        color_scheme: ColorScheme::Light,
        os: if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(windows) {
            Os::Windows
        } else {
            Os::Linux
        },
        idiom: Idiom::Desktop,
        ..Environment::default()
    }
}

/// Opens a window showing `source` in an [`AppView`] (in a running GPUI
/// app).
pub fn open_app_window(
    source: AppSource,
    options: AppOptions,
    cx: &mut GpuiApp,
) -> anyhow::Result<gpui::WindowHandle<AppView>> {
    let size = gpui::size(
        gpui::px(options.size.0 as f32),
        gpui::px(options.size.1 as f32),
    );
    let bounds = gpui::Bounds::centered(None, size, cx);
    let window_options = gpui::WindowOptions {
        window_bounds: Some(gpui::WindowBounds::Windowed(bounds)),
        titlebar: Some(gpui::TitlebarOptions {
            title: Some(options.title.clone().into()),
            ..Default::default()
        }),
        app_id: Some(options.app_id.clone()),
        ..Default::default()
    };
    let script = options.script.clone();
    let handle = cx.open_window(window_options, move |window, cx| {
        let view: Entity<AppView> = cx.new(|cx| AppView::new(options, window, cx));
        match source {
            AppSource::Rust(root) => view.read(cx).load(App::Rust(root)),
            AppSource::Bundle(bundle) => view.read(cx).load(App::Js(bundle)),
            #[cfg(feature = "dev")]
            AppSource::DevServer(url) => crate::dev::start(&view, url, cx),
        }
        let focus = view.read(cx).focus.clone();
        window.focus(&focus, cx);
        view
    })?;
    if let Some(script) = script {
        crate::script::run(handle, &script, cx);
    }
    Ok(handle)
}

/// Runs `source` in a window until it is closed.
pub fn run_app(source: AppSource, options: AppOptions) -> std::process::ExitCode {
    let ok = Rc::new(std::cell::Cell::new(true));
    let result = ok.clone();
    gpui_platform::application().run(move |cx| {
        cx.on_window_closed(|cx, _| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        if let Err(err) = open_app_window(source, options, cx) {
            log::error!("cannot open a window: {err}");
            ok.set(false);
            cx.quit();
            return;
        }
        cx.activate(true);
    });
    if result.get() {
        std::process::ExitCode::SUCCESS
    } else {
        std::process::ExitCode::FAILURE
    }
}
