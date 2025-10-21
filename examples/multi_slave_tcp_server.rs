use modbus_rs::server::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 创建多从机 TCP 服务器
    let server = ModbusMultiSlaveTcpServer::new("127.0.0.1:502").await?;
    
    // 添加多个从机
    server.add_slave(1);  // 从机 1
    server.add_slave(2);  // 从机 2
    server.add_slave(3);  // 从机 3
    
    // 为不同从机设置不同的数据
    server.set_coil(1, 0, true)?;
    server.set_coil(1, 1, false)?;
    server.set_holding_register(1, 0, 1000)?;
    server.set_holding_register(1, 1, 2000)?;
    
    server.set_coil(2, 0, false)?;
    server.set_coil(2, 1, true)?;
    server.set_holding_register(2, 0, 3000)?;
    server.set_holding_register(2, 1, 4000)?;
    
    server.set_coil(3, 0, true)?;
    server.set_coil(3, 1, true)?;
    server.set_holding_register(3, 0, 5000)?;
    server.set_holding_register(3, 1, 6000)?;
    
    println!("多从机 TCP 服务器启动，支持从机 ID: {:?}", server.get_slave_ids());
    println!("从机 1: 线圈 0=true, 1=false, 寄存器 0=1000, 1=2000");
    println!("从机 2: 线圈 0=false, 1=true, 寄存器 0=3000, 1=4000");
    println!("从机 3: 线圈 0=true, 1=true, 寄存器 0=5000, 1=6000");
    println!("服务器监听地址: 127.0.0.1:502");
    
    // 运行服务器
    server.run().await?;
    
    Ok(())
}
