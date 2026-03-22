#[cfg(any())]
mod disabled_module;

#[cfg(feature = "enabled")]
mod enabled_module;

#[cfg(any())]
async fn disabled_function() {
    std::thread::sleep(std::time::Duration::from_millis(1));
}

fn main() {}
