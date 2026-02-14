


fn main() {
    if let Err(e) = rusty_judge::run_cli() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

