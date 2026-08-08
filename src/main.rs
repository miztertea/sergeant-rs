//! Entry point for the `sgt` binary: a thin shell over `sergeant_rs::cli`.

fn main() -> std::process::ExitCode {
    sergeant_rs::cli::main()
}
