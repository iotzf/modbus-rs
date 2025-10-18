use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Modbus RTU over TCP Server Example");
    
    // 创建Modbus RTU over TCP服务器
    let server = ModbusRtuOverTcpServer::new("127.0.0.1:5020", 1).await?;
    
    // 设置一些示例数据
    server.set_coil(0, true);
    server.set_coil(1, false);
    server.set_coil(2, true);
    server.set_discrete_input(0, true);
    server.set_discrete_input(1, true);
    server.set_discrete_input(2, false);
    server.set_holding_register(0, 1000);
    server.set_holding_register(1, 2000);
    server.set_holding_register(2, 3000);
    server.set_input_register(0, 4000);
    server.set_input_register(1, 5000);
    server.set_input_register(2, 6000);
    
    println!("RTU over TCP Server started on 127.0.0.1:5020");
    println!("Slave ID: 1");
    println!("Example data:");
    println!("  Coils: 0=true, 1=false, 2=true");
    println!("  Discrete inputs: 0=true, 1=true, 2=false");
    println!("  Holding registers: 0=1000, 1=2000, 2=3000");
    println!("  Input registers: 0=4000, 1=5000, 2=6000");
    println!();
    println!("Protocol: RTU over TCP (no CRC, TCP provides reliability)");
    println!("Frame format: [Slave ID][Function Code][Data...]");
    
    // 运行服务器
    server.run().await?;
    
    Ok(())
}
