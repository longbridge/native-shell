//! `<Image>`: images loaded and decoded off the main thread (ImageIO),
//! shown in a sublayer of the view.

use std::{path::PathBuf, rc::Rc};

use nsh_core::{Command, ImageOptions, ImageSource, NodeId, ResizeMode};

use crate::{surface::Surface, view::NshView};

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
        match &self.0 {
            Some(AssetBase::Directory(dir)) => {
                std::fs::read(dir.join(path.trim_start_matches('/'))).map_err(|e| e.to_string())
            }
            Some(AssetBase::Http(_)) => Err(format!("{uri}: not loaded yet")),
            None => Err(format!("{uri}: no asset location configured")),
        }
    }
}

/// Sets where `asset:/` URIs of `surface` resolve.
pub(crate) fn set_asset_base(surface: &Rc<Surface>, base: Option<AssetBase>) {
    *surface.asset_base.borrow_mut() = base;
}

/// An image view's image.
#[derive(Default)]
pub(crate) struct ImageState {}

/// Sets up an image view.
pub(crate) fn create(view: &NshView) {
    view.state_mut().image = Some(ImageState::default());
}

/// An image view's source changed.
pub(crate) fn set(
    _surface: &Rc<Surface>,
    _view: &NshView,
    _source: Option<&ImageSource>,
    _mode: ResizeMode,
    _options: &ImageOptions,
) {
}

/// `Image.getSize` / `prefetch` / ...
pub(crate) fn command(_surface: &Rc<Surface>, _id: NodeId, _command: &Command) {}
