use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::Context as _;
use nsh_bundler::{BundleError, BundleOptions};

use crate::ui;

/// A failed build, formatted for terminals and for device error screens.
#[derive(Debug)]
pub(crate) struct BuildError {
    pub(crate) colored: String,
    pub(crate) plain: String,
}

impl From<BundleError> for BuildError {
    fn from(err: BundleError) -> Self {
        Self {
            colored: err.colored,
            plain: err.plain,
        }
    }
}

impl From<nsh_js::JsError> for BuildError {
    fn from(err: nsh_js::JsError) -> Self {
        Self {
            colored: err.message.clone(),
            plain: err.message,
        }
    }
}

/// A compiled app, in memory.
pub(crate) struct Built {
    /// The bundle as JS source.
    pub(crate) code: String,
    /// The bundle as QuickJS bytecode.
    pub(crate) bytecode: Vec<u8>,
    /// Source map of `code`.
    pub(crate) source_map: Option<String>,
    /// The files the app ships: those it imports and those its UI sets'
    /// crates ship ([`crate_assets`]).
    pub(crate) assets: Vec<Asset>,
    /// What the app declares it may reach (`nsh.json` `capabilities`).
    pub(crate) capabilities: Option<nsh_core::access::Capabilities>,
    /// The UI sets it imports, by specifier ([`imported_sets`]).
    pub(crate) ui_sets: Vec<String>,
    /// How long bundling and compiling took.
    pub(crate) elapsed: Duration,
}

/// A file an app ships.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Asset {
    /// Its path among the app's assets (`asset:/<path>`), `/`-separated.
    pub(crate) path: String,
    /// The file.
    pub(crate) source: PathBuf,
}

/// The files the UI sets' crates ship (`Registry::assets`), published
/// under `@<package>/`.
pub(crate) fn crate_assets() -> impl Iterator<Item = Asset> {
    crate::ui_sets()
        .iter()
        .flat_map(nsh_core::ui::Registry::asset_files)
        .map(|file| Asset {
            path: file.path().to_owned(),
            source: PathBuf::from(file.source),
        })
}

/// The native modules of `natives` that are UI sets (`nsh/ui/mobile`, an
/// app's own): what a host must serve besides nsh's own modules, which
/// every host has.
fn imported_sets(natives: &[String]) -> Vec<String> {
    natives
        .iter()
        .filter(|spec| {
            crate::ui_sets()
                .iter()
                .any(|set| set.specifier() == Some(spec.as_str()))
        })
        .cloned()
        .collect()
}

/// The app's assets: `imported` (relative to the entry's directory `root`)
/// and the UI sets' ([`crate_assets`]).
fn assets(root: Option<&Path>, imported: &[PathBuf]) -> Vec<Asset> {
    let root = root.unwrap_or(Path::new("."));
    let mut assets: Vec<Asset> = imported
        .iter()
        .map(|rel| Asset {
            path: rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/"),
            source: root.join(rel),
        })
        .chain(crate_assets())
        .collect();
    assets.sort_by(|a, b| a.path.cmp(&b.path));
    assets.dedup_by(|a, b| a.path == b.path);
    assets
}

impl Built {
    /// `capabilities.json` (written next to the bundle), when the app
    /// declares capabilities.
    pub(crate) fn capabilities_json(&self) -> Option<String> {
        self.capabilities.as_ref().map(|c| c.to_json().to_string())
    }

    /// `(3.2 KB js, 9.8 KB bytecode<more>, 21ms)`, dimmed.
    pub(crate) fn summary(&self, more: &str) -> String {
        ui::dim(format!(
            "({} js, {} bytecode{more}, {})",
            ui::bytes(self.code.len()),
            ui::bytes(self.bytecode.len()),
            ui::duration(self.elapsed)
        ))
    }
}

impl BuildError {
    /// The error for a command that ends with it.
    pub(crate) fn into_error(self, entry: &Path) -> anyhow::Error {
        ui::failure(format!("building {} failed", ui::path(entry)), self.colored)
    }
}

/// Bundles `entry` and compiles it to bytecode.
pub(crate) fn compile(entry: &Path, release: bool) -> Result<Built, BuildError> {
    let start = Instant::now();
    let bundle = nsh_bundler::bundle(&BundleOptions {
        entry: entry.to_path_buf(),
        minify: release,
        ui_sets: crate::ui_sets().to_vec(),
    })?;
    let dir = dunce::canonicalize(entry)
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf));
    let bytecode = nsh_js::compile_bytecode_with(&bundle.code, release, crate::ui_sets())?;
    Ok(Built {
        code: bundle.code,
        bytecode,
        source_map: bundle.source_map,
        assets: assets(dir.as_deref(), &bundle.assets),
        capabilities: bundle.capabilities,
        ui_sets: imported_sets(&bundle.natives),
        elapsed: start.elapsed(),
    })
}

/// Bundles `entry` for development with hot module replacement: the whole
/// app (source and bytecode) plus the modules, to compute later updates.
pub(crate) fn compile_hot(entry: &Path) -> Result<(Built, nsh_bundler::HotBundle), BuildError> {
    let start = Instant::now();
    let hot = nsh_bundler::bundle_hot(entry, crate::ui_sets())?;
    let dir = dunce::canonicalize(entry)
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf));
    let (code, source_map) = hot.script();
    let bytecode = nsh_js::compile_bytecode_with(&code, false, crate::ui_sets())?;
    let built = Built {
        code,
        bytecode,
        source_map,
        assets: assets(dir.as_deref(), &hot.assets),
        capabilities: hot.capabilities.clone(),
        ui_sets: imported_sets(hot.natives()),
        elapsed: start.elapsed(),
    };
    Ok((built, hot))
}

/// Bundles `entry` into `out/bundle.js`, `out/bundle.js.map` and
/// `out/bundle.nshc`.
pub(crate) fn build(entry: &Path, out: &Path, release: bool) -> anyhow::Result<()> {
    let built = compile(entry, release).map_err(|e| e.into_error(entry))?;
    fs::create_dir_all(out).with_context(|| format!("create {}", out.display()))?;
    let map = built.source_map.as_deref().unwrap_or_default();
    for (name, bytes) in [
        ("bundle.js", built.code.as_bytes()),
        ("bundle.js.map", map.as_bytes()),
        ("bundle.nshc", &built.bytecode),
    ] {
        let path = out.join(name);
        fs::write(&path, bytes).with_context(|| format!("write {}", path.display()))?;
    }
    let declared = out.join("capabilities.json");
    match built.capabilities_json() {
        Some(json) => {
            fs::write(&declared, json).with_context(|| format!("write {}", declared.display()))?
        }
        None => {
            let _ = fs::remove_file(&declared);
        }
    }
    // Assets keep their paths: `asset:/img/a.png` -> `out/assets/img/a.png`.
    for asset in &built.assets {
        let dest = out.join("assets").join(&asset.path);
        if let Some(dir) = dest.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::copy(&asset.source, &dest).with_context(|| format!("copy {}", asset.path))?;
    }
    let assets = match built.assets.len() {
        0 => String::new(),
        1 => ", 1 asset".to_owned(),
        n => format!(", {n} assets"),
    };
    ui::done(format!(
        "built {} into {} {}",
        ui::path(entry),
        ui::path(out),
        built.summary(&assets)
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    #[test]
    fn hosts_must_serve_the_ui_sets_an_app_imports() {
        let natives = ["nsh", "nsh/list", "nsh/ui/mobile"].map(String::from);
        assert_eq!(super::imported_sets(&natives), ["nsh/ui/mobile"]);
    }

    #[test]
    fn imported_assets_keep_their_paths_under_the_entry() {
        let assets = super::assets(
            Some(Path::new("/app")),
            &[PathBuf::from("img/a.png"), PathBuf::from("b.ttf")],
        );
        let paths: Vec<_> = assets
            .iter()
            .map(|a| (a.path.as_str(), a.source.clone()))
            .collect();
        assert_eq!(
            paths,
            [
                ("b.ttf", PathBuf::from("/app/b.ttf")),
                ("img/a.png", PathBuf::from("/app/img/a.png")),
            ]
        );
    }
}
