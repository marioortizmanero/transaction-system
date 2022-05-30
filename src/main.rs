use std::process;

/// Entry point, which simply handles errors in a pretty & idiomatic way for the
/// implementation.
fn main() {
    if let Err(e) = transaction_system::load() {
        eprintln!("Fatal error: {}", e);
        process::exit(1);
    }
}
