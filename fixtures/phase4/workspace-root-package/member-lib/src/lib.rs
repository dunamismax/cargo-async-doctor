use tokio::runtime::Handle;

async fn lib_warning() {
    Handle::current().block_on(async {});
}
