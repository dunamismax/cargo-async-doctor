use std::fs;

async fn blocking_read() {
    let _ = fs::read_to_string("Cargo.toml");
}

fn main() {}
