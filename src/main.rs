#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    magia_baiser::run().await
}
