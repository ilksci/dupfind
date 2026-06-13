use std::process;

fn main() {
    if let Err(e) = dupfind_cli::run() {
        eprintln!("[错误] {}", e);
        process::exit(1);
    }
}
