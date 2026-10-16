//! `cargo run` (the workspace's default task): runs an example app, the
//! gallery by default, in the Mac Catalyst development host (the GTK host
//! on Linux, an Android device or emulator elsewhere) with hot reload
//! (`nsh run`), and rebuilds and restarts it when the runtime's sources
//! under `crates/` change: components and modules are Rust and live in the
//! host, so only JS and TSX reload in place.

use std::{
    path::Path,
    process::{Child, Command},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::Duration,
};

use anyhow::Context as _;
use clap::Args;
use notify::{EventKind, RecursiveMode, Watcher as _};

use crate::{android, term};

/// Options of `cargo run`, passed on to `nsh run`.
#[derive(Args)]
pub(crate) struct RunArgs {
    /// The app: its directory (whose `App.tsx` is the entry) or its entry
    /// module.
    #[arg(default_value = "examples/gallery")]
    entry: String,
    /// Where to run: `catalyst`, `ios` (a simulator), `android` or `linux`;
    /// default: `catalyst` on macOS, `linux` on Linux, else `android`.
    #[arg(long)]
    target: Option<String>,
    /// iOS simulator to use (name or UDID).
    #[arg(long)]
    device: Option<String>,
    /// Log per-commit timings.
    #[arg(long)]
    trace: bool,
}

/// Set by SIGINT / SIGTERM: the app runs in its own process group (so a
/// restart can stop `nsh run` and the host it launched together), which
/// does not see the terminal's Ctrl-C.
static STOP: AtomicBool = AtomicBool::new(false);

extern "C" fn on_signal(_: libc::c_int) {
    STOP.store(true, Ordering::SeqCst);
}

pub(crate) fn run(args: RunArgs) -> anyhow::Result<()> {
    let root = android::root();
    // SAFETY: the handler only stores to an atomic, which is
    // async-signal-safe.
    unsafe {
        libc::signal(libc::SIGINT, on_signal as *const () as libc::sighandler_t);
        libc::signal(libc::SIGTERM, on_signal as *const () as libc::sighandler_t);
    }
    let (tx, changes) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<notify::Event>| {
        if let Ok(event) = event
            && !matches!(event.kind, EventKind::Access(_))
            && event.paths.iter().any(|p| is_source(p))
        {
            let _ = tx.send(());
        }
    })?;
    // Canonical: inotify reports paths under the one watched as given, and
    // `root`'s `..` would make every one look like a dot entry.
    let crates = root.join("crates");
    watcher.watch(
        &crates.canonicalize().unwrap_or(crates),
        RecursiveMode::Recursive,
    )?;
    loop {
        let mut app = match build_cli(&root) {
            Ok(()) => Some(launch(&root, &args)?),
            // Ctrl-C stopped the build too.
            Err(_) if STOP.load(Ordering::SeqCst) => return Ok(()),
            Err(err) => {
                term::report(&err);
                term::note("waiting for changes under crates/");
                None
            }
        };
        // Until a source changes, the app exits or the user stops us.
        loop {
            if STOP.load(Ordering::SeqCst) {
                if let Some(app) = app.as_mut() {
                    stop(app);
                }
                return Ok(());
            }
            if let Some(child) = app.as_mut()
                && let Some(status) = child.try_wait()?
            {
                // The host window was closed.
                std::process::exit(status.code().unwrap_or(1));
            }
            match changes.recv_timeout(Duration::from_millis(100)) {
                Ok(()) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
            }
        }
        // Editors save several files at once: take them as one change.
        while changes.recv_timeout(Duration::from_millis(300)).is_ok() {}
        if let Some(app) = app.as_mut() {
            stop(app);
        }
        term::note("crates/ changed: rebuilding and restarting");
    }
}

/// Whether a changed path is a source of the runtime (not build output,
/// generated types or editor state).
fn is_source(path: &Path) -> bool {
    !path.components().any(|c| {
        let c = c.as_os_str().to_string_lossy();
        c == "target" || c.starts_with('.')
    }) && !path.to_string_lossy().ends_with('~')
}

fn build_cli(root: &Path) -> anyhow::Result<()> {
    term::cargo(
        Command::new("cargo")
            .args(["build", "-p", "nsh-cli"])
            .current_dir(root),
        "nsh",
    )
}

/// Starts `nsh run`, which builds the host it needs, in a process group of
/// its own.
fn launch(root: &Path, args: &RunArgs) -> anyhow::Result<Child> {
    use std::os::unix::process::CommandExt as _;

    let mut cmd = Command::new(root.join("target/debug/nsh"));
    cmd.arg("run").arg(&args.entry).current_dir(root);
    let default = if cfg!(target_os = "macos") {
        "catalyst"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "android"
    };
    cmd.args(["--target", args.target.as_deref().unwrap_or(default)]);
    if let Some(device) = &args.device {
        cmd.args(["--device", device]);
    }
    if args.trace {
        cmd.arg("--trace");
    }
    if term::verbose() {
        cmd.arg("--verbose");
    }
    cmd.process_group(0)
        .spawn()
        .context("launch target/debug/nsh")
}

/// Stops `nsh run` and everything it started (the Catalyst or Linux host).
fn stop(app: &mut Child) {
    let Ok(pgid) = libc::pid_t::try_from(app.id()) else {
        let _ = app.kill();
        return;
    };
    // SAFETY: plain syscalls on the process group `launch` created.
    unsafe { libc::killpg(pgid, libc::SIGTERM) };
    for _ in 0..30 {
        if matches!(app.try_wait(), Ok(Some(_))) {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    // SAFETY: as above.
    unsafe { libc::killpg(pgid, libc::SIGKILL) };
    let _ = app.wait();
}
