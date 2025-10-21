use modbus_rs::client::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    // 创建 TCP 客户端
    let mut client = ModbusTcpClient::new("127.0.0.1", 502, 1).await?;
    
    println!("测试多从机服务器...");
    
    // 测试从机 1
    println!("\n=== 测试从机 1 ===");
    let coils_1 = client.read_coils_with_slave_id(1, 0, 2).await?;
    println!("从机 1 线圈 0-1: {:?}", coils_1);
    
    let registers_1 = client.read_holding_registers_with_slave_id(1, 0, 2).await?;
    println!("从机 1 寄存器 0-1: {:?}", registers_1);
    
    // 测试从机 2
    println!("\n=== 测试从机 2 ===");
    let coils_2 = client.read_coils_with_slave_id(2, 0, 2).await?;
    println!("从机 2 线圈 0-1: {:?}", coils_2);
    
    let registers_2 = client.read_holding_registers_with_slave_id(2, 0, 2).await?;
    println!("从机 2 寄存器 0-1: {:?}", registers_2);
    
    // 测试从机 3
    println!("\n=== 测试从机 3 ===");
    let coils_3 = client.read_coils_with_slave_id(3, 0, 2).await?;
    println!("从机 3 线圈 0-1: {:?}", coils_3);
    
    let registers_3 = client.read_holding_registers_with_slave_id(3, 0, 2).await?;
    println!("从机 3 寄存器 0-1: {:?}", registers_3);
    
    // 测试不存在的从机
    println!("\n=== 测试不存在的从机 99 ===");
    match client.read_coils_with_slave_id(99, 0, 1).await {
        Ok(_) => println!("意外成功"),
        Err(e) => println!("预期的错误: {}", e),
    }
    
    // 写入测试
    println!("\n=== 写入测试 ===");
    client.write_single_coil_with_slave_id(1, 10, true).await?;
    client.write_single_register_with_slave_id(1, 10, 9999).await?;
    
    let new_coil = client.read_coils_with_slave_id(1, 10, 1).await?;
    let new_register = client.read_holding_registers_with_slave_id(1, 10, 1).await?;
    println!("从机 1 新写入的线圈 10: {:?}", new_coil);
    println!("从机 1 新写入的寄存器 10: {:?}", new_register);
    
    println!("\n多从机测试完成！");
    
    Ok(())
}
