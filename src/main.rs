use std::process;

fn main() {
    if let Err(e) = dupfind::run() {
        eprintln!("[ERROR] {}", e);
        process::exit(1);
    }
}
