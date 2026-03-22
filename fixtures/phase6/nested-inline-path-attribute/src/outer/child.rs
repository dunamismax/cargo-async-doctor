use std::thread;

async fn active_warning() {
    thread::sleep(std::time::Duration::from_millis(1));
}
