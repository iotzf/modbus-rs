use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Modbus RTU Server Example");
    
    // 创建Modbus RTU服务器（需要实际的串口设备）
    let mut server = ModbusRtuServer::new("/dev/ttyUSB0", 1, 9600).await?;
    
    // 设置一些示例数据
    server.set_coil(0, true);
    server.set_coil(1, false);
    server.set_discrete_input(0, true);
    server.set_discrete_input(1, true);
    server.set_holding_register(0, 1000);
    server.set_holding_register(1, 2000);
    server.set_input_register(0, 3000);
    server.set_input_register(1, 4000);
    
    println!("Server started on /dev/ttyUSB0");
    println!("Slave ID: 1");
    println!("Baud rate: 9600");
    println!("Example data:");
    println!("  Coils: 0=true, 1=false");
    println!("  Discrete inputs: 0=true, 1=true");
    println!("  Holding registers: 0=1000, 1=2000");
    println!("  Input registers: 0=3000, 1=4000");
    
    // 运行服务器
    server.run().await?;
    
    Ok(())
}
