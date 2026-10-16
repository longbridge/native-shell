//! `nsh`: the command line with the mobile UI.

fn main() -> std::process::ExitCode {
    nsh_cli::main_with(vec![nsh_ui_mobile::registry()])
}
