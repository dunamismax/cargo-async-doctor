struct Handle;

impl Handle {
    fn current() -> Self {
        Self
    }

    fn block_on<F>(&self, _: F) {}
}

fn sync_bridge() {
    Handle::current().block_on(async {});
}

async fn async_lookalike() {
    Handle::current().block_on(async {});
}

fn main() {}
