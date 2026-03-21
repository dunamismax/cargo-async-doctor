use std::fs;

async fn member_warning() {
    let _ = fs::read_to_string("Cargo.toml");
}

fn main() {}
