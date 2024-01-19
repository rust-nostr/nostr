mod quickstart;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    quickstart::quickstart().await.unwrap();
}
