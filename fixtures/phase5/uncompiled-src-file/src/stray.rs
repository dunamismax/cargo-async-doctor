use std::thread;

async fn stray_warning() {
    thread::sleep(std::time::Duration::from_millis(1));
}
