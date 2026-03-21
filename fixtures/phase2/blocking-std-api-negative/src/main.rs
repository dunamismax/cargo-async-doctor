mod fs {
    pub fn read_to_string(_: &str) {}
}

fn sync_read() {
    let _ = std::fs::read_to_string("Cargo.toml");
}

async fn lookalike_async_read() {
    fs::read_to_string("Cargo.toml");
}

fn main() {}
