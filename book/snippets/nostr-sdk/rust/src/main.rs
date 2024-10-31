mod quickstart;
mod nip17;

#[tokio::main]
async fn main() {
    quickstart::quickstart().await.unwrap();
    nip17::run().await.unwrap();
}
