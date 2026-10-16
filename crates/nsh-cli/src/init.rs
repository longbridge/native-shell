//! `nsh init` (a new app) and the TypeScript declarations editors read
//! (`.nsh/types`, refreshed by every command that builds an app).

use std::{fs, path::Path};

use anyhow::{Context as _, bail};

use crate::ui;

const APP: &str = r##"import { mount, signal, Text, View } from "nsh";
import { SafeAreaView } from "nsh/ui/mobile";

function App() {
  const count = signal(0);
  return (
    <SafeAreaView style={{ flex: 1, alignItems: "center", justifyContent: "center", gap: 16 }}>
      <Text style={{ fontSize: 28, fontWeight: "600" }}>{count()}</Text>
      <View
        onPress={() => count.update((c) => c + 1)}
        activeOpacity={0.6}
        accessibilityRole="button"
        style={{ paddingHorizontal: 20, paddingVertical: 12, borderRadius: 10, backgroundColor: "#0969da" }}
      >
        <Text style={{ color: "white", fontSize: 17 }}>Tap</Text>
      </View>
    </SafeAreaView>
  );
}

mount(App);
"##;

const GITIGNORE: &str = ".nsh/\nout/\n";

/// Creates an app in `dir`: `App.tsx` (the entry by convention), `nsh.json`, `tsconfig.json`,
/// `.gitignore` and the type declarations. Existing files are kept.
pub(crate) fn init(dir: &Path, name: Option<String>) -> anyhow::Result<()> {
    fs::create_dir_all(dir).with_context(|| format!("create {}", dir.display()))?;
    let run = if dir == Path::new(".") {
        "nsh run".to_owned()
    } else {
        format!("nsh run {}", dir.display())
    };
    let dir = dir.canonicalize()?;
    let name = name.unwrap_or_else(|| {
        dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_owned()
    });
    let id_part: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();
    let config = serde_json::json!({
        "name": name,
        "id": format!("com.example.{}", if id_part.is_empty() { "app" } else { &id_part }),
        "version": "1.0.0",
        "build": 1,
    });
    let files = [
        ("App.tsx", APP.to_owned()),
        (
            "nsh.json",
            format!("{}\n", serde_json::to_string_pretty(&config)?),
        ),
        ("tsconfig.json", nsh_bundler::TSCONFIG.to_owned()),
        (".gitignore", GITIGNORE.to_owned()),
    ];
    if files.iter().all(|(f, _)| dir.join(f).exists()) {
        bail!("{} already has an app", ui::path(&dir));
    }
    let (mut created, mut kept) = (Vec::new(), Vec::new());
    for (file, contents) in files {
        let path = dir.join(file);
        if path.exists() {
            kept.push(file);
            continue;
        }
        fs::write(&path, contents).with_context(|| format!("write {}", path.display()))?;
        created.push(file);
    }
    write_types(&dir);
    let kept = if kept.is_empty() {
        String::new()
    } else {
        format!(" {}", ui::dim(format!("(kept {})", kept.join(", "))))
    };
    ui::done(format!(
        "created {}: {}{kept}",
        ui::path(&dir),
        created.join(", ")
    ));
    ui::note(format!("run it with `{run}`"));
    Ok(())
}

/// Writes `.nsh/types` in the app directory `dir` (files whose contents did
/// not change are left alone, so editors do not reload them). Files nsh no
/// longer generates (a module that moved) are removed.
pub(crate) fn write_types(dir: &Path) {
    let types = dir.join(".nsh/types");
    let mut definitions = nsh_js::type_definitions(crate::ui_sets());
    definitions.extend(nsh_bundler::type_definitions());
    remove_stale(&types, &types, &definitions);
    for (path, contents) in definitions {
        let path = types.join(path);
        if fs::read(&path).is_ok_and(|old| old == contents.as_bytes()) {
            continue;
        }
        let written = path
            .parent()
            .map_or(Ok(()), fs::create_dir_all)
            .and_then(|()| fs::write(&path, contents));
        if let Err(err) = written {
            ui::warn(format!("cannot write {}: {err}", ui::path(&path)));
            return;
        }
    }
}

/// Removes the files under `dir` (inside `types`) that are not among
/// `definitions`, and directories left empty.
fn remove_stale(types: &Path, dir: &Path, definitions: &[(String, String)]) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            remove_stale(types, &path, definitions);
            let _ = fs::remove_dir(&path);
        } else if let Ok(relative) = path.strip_prefix(types) {
            let relative = relative.to_string_lossy().replace('\\', "/");
            if !definitions.iter().any(|(p, _)| *p == relative) {
                let _ = fs::remove_file(&path);
            }
        }
    }
}

/// [`write_types`] for the app whose entry is `entry`.
pub(crate) fn write_types_for(entry: &Path) {
    if let Some(dir) = entry
        .canonicalize()
        .ok()
        .and_then(|e| e.parent().map(Path::to_path_buf))
    {
        write_types(&dir);
    }
}
