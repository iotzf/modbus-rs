use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("Modbus RTU over TCP Client Example");
    
    // 连接到Modbus RTU over TCP服务器
    let mut client = ModbusRtuOverTcpClient::new("127.0.0.1", 5020, 1).await?;
    
    // 读取保持寄存器
    match client.read_holding_registers(0, 10).await {
        Ok(values) => {
            println!("Read holding registers: {:?}", values);
        },
        Err(e) => {
            println!("Failed to read holding registers: {}", e);
        }
    }
    
    // 写入单个寄存器
    match client.write_single_register(0, 1234).await {
        Ok(_) => {
            println!("Successfully wrote single register");
        },
        Err(e) => {
            println!("Failed to write single register: {}", e);
        }
    }
    
    // 读取线圈
    match client.read_coils(0, 8).await {
        Ok(values) => {
            println!("Read coils: {:?}", values);
        },
        Err(e) => {
            println!("Failed to read coils: {}", e);
        }
    }
    
    // 写入单个线圈
    match client.write_single_coil(0, true).await {
        Ok(_) => {
            println!("Successfully wrote single coil");
        },
        Err(e) => {
            println!("Failed to write single coil: {}", e);
        }
    }
    
    // 写入多个寄存器
    let values = vec![1000, 2000, 3000, 4000];
    match client.write_multiple_registers(10, &values).await {
        Ok(_) => {
            println!("Successfully wrote multiple registers");
        },
        Err(e) => {
            println!("Failed to write multiple registers: {}", e);
        }
    }
    
    // 读取输入寄存器
    match client.read_input_registers(0, 5).await {
        Ok(values) => {
            println!("Read input registers: {:?}", values);
        },
        Err(e) => {
            println!("Failed to read input registers: {}", e);
        }
    }
    
    Ok(())
}
