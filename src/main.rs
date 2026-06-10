use std::process;

fn main() {
    if let Err(e) = dupfind::run() {
        eprintln!("[错误] {}", e);
        process::exit(1);
    }
}
