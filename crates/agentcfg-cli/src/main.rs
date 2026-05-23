const HELP: &str = "\
agentcfg

Usage: agentcfg [OPTIONS]

Options:
  -h, --help    Print help
";

fn main() {
    let show_help = std::env::args_os()
        .nth(1)
        .is_none_or(|arg| arg == "--help" || arg == "-h");

    if show_help {
        print!("{HELP}");
    }
}
