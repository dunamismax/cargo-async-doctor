fn sync_only() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

async fn async_safe() {
    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
}

fn main() {}
