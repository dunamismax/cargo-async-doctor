use std::thread;

async fn outside_src_warning() {
    thread::sleep(std::time::Duration::from_millis(1));
}
