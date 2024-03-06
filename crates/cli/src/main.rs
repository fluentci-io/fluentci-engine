use fluentci_server::start;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    start().await
}
