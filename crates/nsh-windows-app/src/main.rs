//! The release runner with the mobile UI (`nsh_platform_windows`'s default UI sets).

// A windowed program: no console opens with it.
#![windows_subsystem = "windows"]

fn main() -> std::process::ExitCode {
    nsh_windows_app::main()
}
