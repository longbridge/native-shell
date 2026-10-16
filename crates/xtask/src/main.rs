//! Maintainer tasks, run with `cargo xtask <task>`:
//!
//! - `run` (the default, and what `cargo run` runs): an example app in the
//!   Mac Catalyst development host, restarted when Rust code changes.
//! - `android`: the Android runtime (`libnsh.so` per ABI, `classes.dex`)
//!   into `target/android/<profile>`, where `nsh run` / `nsh package` find it.
//! - `dist`: the distribution in `dist/nsh`: the `nsh` CLI plus every
//!   prebuilt runtime, so app developers never need Rust.
//! - `ios-embed`: only the iOS embedding SDK (`NativeShell.xcframework`,
//!   `NativeShellDev.xcframework`) into `dist/nsh/embed/ios`.
//! - `web`: the WebAssembly runtime, its player and the website's demos
//!   (`website/public/play` by default).

mod android;
mod dist;
mod ios;
mod run;
mod term;
mod web;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Maintainer tasks; without one, run the gallery (`cargo run`)",
    args_conflicts_with_subcommands = true
)]
struct Cli {
    /// Show the output of cargo and the other tools as it comes (`run`
    /// passes it on to `nsh`).
    #[arg(
        short,
        long,
        global = true,
        env = "NSH_VERBOSE",
        value_parser = clap::builder::BoolishValueParser::new()
    )]
    verbose: bool,
    #[command(subcommand)]
    task: Option<Task>,
    /// Without a task: `run` with these options.
    #[command(flatten)]
    run: run::RunArgs,
}

#[derive(Subcommand)]
enum Task {
    /// Run an example app (default: the gallery, in Mac Catalyst) with hot
    /// reload, rebuilt and restarted when Rust code changes.
    Run(run::RunArgs),
    /// Build the Android runtime.
    Android {
        /// Optimized build.
        #[arg(long)]
        release: bool,
        /// Include development support (dev server, error overlay).
        #[arg(long)]
        dev: bool,
        /// ABIs to build (default: arm64-v8a and x86_64).
        #[arg(long)]
        abi: Vec<String>,
    },
    /// Run clippy on the Android crate (with the NDK set up).
    ClippyAndroid,
    /// Build the distribution in `dist/nsh`.
    Dist,
    /// Build only the iOS embedding XCFrameworks.
    IosEmbed,
    /// Build the WebAssembly runtime and the website's demos.
    Web {
        /// Output directory.
        #[arg(long, default_value = "website/public/play")]
        out: std::path::PathBuf,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    term::init(cli.verbose);
    match task(cli.task.unwrap_or(Task::Run(cli.run))) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            term::report(&e);
            std::process::ExitCode::FAILURE
        }
    }
}

fn task(task: Task) -> anyhow::Result<()> {
    match task {
        Task::Run(args) => run::run(args),
        Task::Android { release, dev, abi } => {
            let abis = if abi.is_empty() {
                android::ABIS.iter().map(|a| a.to_string()).collect()
            } else {
                abi
            };
            android::build(release, dev, &abis).map(|_| ())
        }
        Task::ClippyAndroid => android::clippy(),
        Task::Dist => dist::dist(),
        Task::IosEmbed => ios::build().map(|out| term::note(out.display())),
        Task::Web { out } => web::build(&out),
    }
}
