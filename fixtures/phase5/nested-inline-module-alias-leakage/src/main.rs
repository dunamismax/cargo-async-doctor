use std::{fs, thread};
use tokio::runtime::Handle;

mod nested {
    mod thread {
        pub fn sleep(_: std::time::Duration) {}
    }

    mod fs {
        pub fn read_to_string(_: &str) {}
    }

    struct Handle;

    impl Handle {
        fn current() -> Self {
            Self
        }

        fn block_on<F>(&self, _: F) {}
    }

    async fn should_stay_quiet() {
        thread::sleep(std::time::Duration::from_millis(1));
        fs::read_to_string("Cargo.toml");
        Handle::current().block_on(async {});
    }
}

fn main() {}
