use std::thread;

async fn blocking_sleep() {
    thread::sleep(std::time::Duration::from_millis(1));
}

fn main() {}
