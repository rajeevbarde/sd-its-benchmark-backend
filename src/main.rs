use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    println!("SD-ITS Benchmark Backend Starting...");
    
    // TODO: Initialize database pool
    // TODO: Start web server
}
