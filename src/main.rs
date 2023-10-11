mod codec;
mod server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    server::listen(1234).await
}
