use std::fs;

async fn other_warning() {
    let _ = fs::read_to_string("Cargo.toml");
}
