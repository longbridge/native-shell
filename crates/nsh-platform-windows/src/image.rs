//! Images: loading (files, `asset:/`, HTTP, `data:`), decoding with WIC on
//! worker threads, Direct2D bitmaps in a cache, drawing with the resize
//! modes, animated GIF / WebP.

use std::{path::PathBuf, rc::Rc};

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
                let file = dir.join(path);
                std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))
            }
            Some(AssetBase::Http(base)) => {
                let sep = if base.ends_with('/') { "" } else { "/" };
                let url = format!("{base}{sep}{path}");
                let mut response = ureq::get(&url)
                    .call()
                    .map_err(|e| format!("{url}: {e}"))?;
                response
                    .body_mut()
                    .read_to_vec()
                    .map_err(|e| format!("{url}: {e}"))
            }
            None => Err(format!("{uri}: no asset location configured")),
        }
    }
}

/// Where this surface's images resolve `asset:/` URIs.
pub(crate) fn set_asset_base(surface: &Rc<Surface>, base: Option<AssetBase>) {
    *surface.asset_base.borrow_mut() = base;
}

/// An image view's state.
#[derive(Default)]
pub(crate) struct ImageState {}

/// Sets up an image view.
pub(crate) fn create(view: &NshView) {
    view.state_mut().image = Some(ImageState::default());
}

/// An image view's source changed.
pub(crate) fn set(
    surface: &Rc<Surface>,
    view: &NshView,
    source: Option<&ImageSource>,
    mode: ResizeMode,
    options: &ImageOptions,
) {
    let _ = (surface, view, source, mode, options);
}

/// `ImageInfo` and other image commands.
pub(crate) fn command(surface: &Rc<Surface>, id: NodeId, command: &Command) {
    let _ = (surface, id, command);
}

/// Draws the image.
pub(crate) fn draw(cx: &mut DrawCx<'_>, state: &ViewState) {
    let _ = (cx, state);
}
