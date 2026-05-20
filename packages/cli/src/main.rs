/// Specter CLI — UCI-compatible chess engine binary.
/// Build: cargo build --release -p specter-cli
use specter::uci::handler::run_uci_loop;

fn main() {
    // Print engine info to stderr (doesn't interfere with UCI protocol on stdout)
    eprintln!("Specter Chess Engine v0.1.0");
    eprintln!("Type 'uci' to begin UCI mode");
    run_uci_loop();
}
