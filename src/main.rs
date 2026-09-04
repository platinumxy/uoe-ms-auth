#[tokio::main]
async fn main() {
    if let Some(cookies) = uoe_ms_auth::run().await {
        println!("Authenticated cookies: {}", cookies);
    } else {
        eprintln!("Failed to authenticate");
    }
}
