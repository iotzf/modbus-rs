#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Modbus Rust Library");
    println!("Available examples:");
    println!("  cargo run --example tcp_client");
    println!("  cargo run --example rtu_client");
    println!("  cargo run --example tcp_server");
    println!("  cargo run --example rtu_server");
    
    Ok(())
}
