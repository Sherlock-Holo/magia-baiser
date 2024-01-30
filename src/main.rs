use std::io;

#[tokio::main(flavor = "current_thread")]
async fn main() -> io::Result<()> {
    magia_baiser::run().await
}
