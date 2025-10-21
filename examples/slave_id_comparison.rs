use modbus_rs::*;

/// 演示两种不同的从机ID设置方式
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Modbus客户端从机ID设置方式对比演示");
    println!("=====================================");
    
    // 方式1：初始化时设置从机ID（当前实现）
    println!("\n方式1：初始化时设置从机ID");
    println!("------------------------");
    
    // 为不同设备创建不同的客户端
    let mut device1_client = ModbusRtuOverTcpClient::new("127.0.0.1", 5020, 1).await?;
    let mut device2_client = ModbusRtuOverTcpClient::new("127.0.0.1", 5020, 2).await?;
    
    println!("设备1客户端 (slave_id=1):");
    match device1_client.read_holding_registers(0, 5).await {
        Ok(values) => println!("  读取成功: {:?}", values),
        Err(e) => println!("  读取失败: {}", e),
    }
    
    println!("设备2客户端 (slave_id=2):");
    match device2_client.read_holding_registers(0, 5).await {
        Ok(values) => println!("  读取成功: {:?}", values),
        Err(e) => println!("  读取失败: {}", e),
    }
    
    // 方式2：功能码操作时设置从机ID（灵活版本）
    println!("\n方式2：功能码操作时设置从机ID");
    println!("----------------------------");
    
    let mut flexible_client = ModbusTcpClient::new("127.0.0.1", 5020, 1).await?;
    
    println!("使用默认从机ID (slave_id=1):");
    match flexible_client.read_holding_registers(0, 5).await {
        Ok(values) => println!("  读取成功: {:?}", values),
        Err(e) => println!("  读取失败: {}", e),
    }
    
    println!("指定从机ID (slave_id=2):");
    match flexible_client.read_holding_registers_with_slave_id(2, 0, 5).await {
        Ok(values) => println!("  读取成功: {:?}", values),
        Err(e) => println!("  读取失败: {}", e),
    }
    
    println!("指定从机ID (slave_id=3):");
    match flexible_client.read_holding_registers_with_slave_id(3, 0, 5).await {
        Ok(values) => println!("  读取成功: {:?}", values),
        Err(e) => println!("  读取失败: {}", e),
    }
    
    // 对比分析
    println!("\n对比分析");
    println!("--------");
    println!("方式1 (初始化时设置):");
    println!("  优点: 简单、类型安全、性能好");
    println!("  缺点: 需要为每个设备创建客户端");
    println!("  适用: 大多数应用场景");
    
    println!("\n方式2 (操作时设置):");
    println!("  优点: 灵活、支持多设备");
    println!("  缺点: 复杂、容易出错");
    println!("  适用: 需要动态切换设备的场景");
    
    Ok(())
}
