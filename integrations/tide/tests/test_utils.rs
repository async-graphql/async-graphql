use reqwest::Client;
use std::time::Duration;

pub fn find_listen_addr() -> &'static str {
    Box::leak(
        format!(
            "http://{}",
            std::net::TcpListener::bind("localhost:0")
                .unwrap()
                .local_addr()
                .unwrap()
        )
        .into_boxed_str(),
    )
}

pub fn client() -> Client {
    Client::builder().no_proxy().build().unwrap()
}

pub async fn wait_server_ready() {
    async_std::task::sleep(Duration::from_secs(1)).await;
}
