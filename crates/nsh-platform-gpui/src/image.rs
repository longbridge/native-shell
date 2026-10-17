//! Images. (Being ported: see `nsh-platform-windows/src/image.rs`.)

use std::{path::PathBuf, rc::Rc, sync::Arc};

use nsh_core::{Command, ImageOptions, ImageSource, NodeId, ResizeMode};

use crate::{
    render::DrawCx,
    surface::Surface,
    view::{NshView, ViewState},
};

/// Where bundled (`asset:/…`) files are.
#[derive(Debug, Clone)]
pub enum AssetBase {
    /// Served over HTTP (the dev server): `http://host:port/assets/`.
    Http(String),
    /// A directory on disk.
    Directory(PathBuf),
}

/// Reads `asset:/…` files for the engine (fonts), on the engine thread.
pub(crate) struct AssetReader(pub(crate) Option<AssetBase>);

impl nsh_engine::Assets for AssetReader {
    fn read(&self, uri: &str) -> Result<Vec<u8>, String> {
        let Some(path) = uri.strip_prefix("asset:/") else {
            return Err(format!("not an asset URI: {uri}"));
        };
        let path = path.trim_start_matches('/');
        match &self.0 {
            Some(AssetBase::Directory(dir)) => {
                std::fs::read(dir.join(path)).map_err(|e| e.to_string())
            }
            Some(AssetBase::Http(base)) => {
                let url = format!("{}/{}", base.trim_end_matches('/'), path);
                let mut response = ureq::get(&url).call().map_err(|e| e.to_string())?;
                response.body_mut().read_to_vec().map_err(|e| e.to_string())
            }
            None => Err(format!("{uri}: no asset location configured")),
        }
    }
}

/// Where this surface's images resolve `asset:/` URIs.
pub(crate) fn set_asset_base(surface: &Rc<Surface>, base: Option<AssetBase>) {
    *surface.asset_base.borrow_mut() = base;
}

/// An image view's content.
pub(crate) struct ImageState;

/// An image view was made.
pub(crate) fn create(_view: &NshView) {}

/// `Image`: the source or options changed.
pub(crate) fn set(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _source: Option<&ImageSource>,
    _mode: ResizeMode,
    _options: &ImageOptions,
) {
}

/// A decoded image (stub until images are ported: what canvases draw).
pub(crate) struct Texture(Arc<gpui::RenderImage>);

impl Texture {
    /// The GPUI image (BGRA).
    pub(crate) fn image(&self) -> &Arc<gpui::RenderImage> {
        &self.0
    }
}

/// Loads `uri` (stub until images are ported: `asset:/` files in an asset
/// directory and file paths, read on the spot) and hands it to `done`
/// with its size in pixels.
pub(crate) fn load(
    surface: &Rc<Surface>,
    uri: &str,
    done: impl FnOnce(Result<(Rc<Texture>, (u32, u32)), String>) + 'static,
) {
    let path = match (uri.strip_prefix("asset:/"), &*surface.asset_base.borrow()) {
        (Some(p), Some(AssetBase::Directory(dir))) => dir.join(p.trim_start_matches('/')),
        (None, _) => PathBuf::from(uri.strip_prefix("file://").unwrap_or(uri)),
        _ => {
            return done(Err(format!(
                "{uri}: not loaded (images are not ported yet)"
            )));
        }
    };
    let decoded = std::fs::read(&path)
        .map_err(|e| format!("{uri}: {e}"))
        .and_then(|bytes| image::load_from_memory(&bytes).map_err(|e| format!("{uri}: {e}")));
    done(decoded.map(|image| {
        let mut rgba = image.into_rgba8();
        let size = rgba.dimensions();
        for p in rgba.pixels_mut() {
            p.0.swap(0, 2);
        }
        let image = gpui::RenderImage::new(vec![image::Frame::new(rgba)]);
        (Rc::new(Texture(Arc::new(image))), size)
    }))
}

/// Image commands (`ImageInfo`).
pub(crate) fn command(_surface: &Rc<Surface>, _id: NodeId, _command: &Command) {}

/// Draws an image view's image.
pub(crate) fn draw(_cx: &mut DrawCx<'_>, _state: &ViewState) {}
