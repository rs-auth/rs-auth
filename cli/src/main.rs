#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rs_auth_cli::run().await
}
