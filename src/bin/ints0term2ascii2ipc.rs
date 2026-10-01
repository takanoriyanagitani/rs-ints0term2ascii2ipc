use std::io;
use std::process::ExitCode;

use rs_ints0term2ascii2ipc::Config;

fn io_config() -> impl Fn() -> Config {
    move || Config::default()
}

fn io_main() -> impl Fn() -> Result<(), io::Error> {
    || {
        let cfg: Config = io_config()();
        cfg.stdin2ints2ascii0term2bat2stdout()
    }
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");
        ExitCode::FAILURE
    })
}
