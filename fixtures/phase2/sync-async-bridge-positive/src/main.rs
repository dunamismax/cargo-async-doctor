use tokio::runtime::Handle;

async fn bridge_hazard() {
    Handle::current().block_on(async {});
}

fn main() {}
