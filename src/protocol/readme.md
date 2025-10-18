# Modbus协议实现文档

本文档描述了Rust语言实现的Modbus协议库的详细说明。

## 支持的协议

### Modbus RTU协议
- 支持串口通信
- 支持CRC16校验
- 支持所有标准功能码

### Modbus TCP协议
- 支持TCP/IP网络通信
- 支持MBAP头部
- 支持所有标准功能码

### Modbus RTU over TCP协议
- 支持TCP/IP网络通信
- 使用RTU格式数据帧
- 无需CRC校验（TCP提供可靠性保证）
- 支持所有标准功能码

## 功能码支持

| 功能码 | 名称 | 描述 |
|--------|------|------|
| 0x01 | Read Coils | 读取线圈状态 |
| 0x02 | Read Discrete Inputs | 读取离散输入状态 |
| 0x03 | Read Holding Registers | 读取保持寄存器 |
| 0x04 | Read Input Registers | 读取输入寄存器 |
| 0x05 | Write Single Coil | 写入单个线圈 |
| 0x06 | Write Single Register | 写入单个寄存器 |
| 0x0F | Write Multiple Coils | 写入多个线圈 |
| 0x10 | Write Multiple Registers | 写入多个寄存器 |

## 异常码支持

| 异常码 | 名称 | 描述 |
|--------|------|------|
| 0x01 | Illegal Function | 非法功能码 |
| 0x02 | Illegal Data Address | 非法数据地址 |
| 0x03 | Illegal Data Value | 非法数据值 |
| 0x04 | Slave Device Failure | 从机设备故障 |
| 0x05 | Acknowledge | 确认 |
| 0x06 | Slave Device Busy | 从机设备忙 |
| 0x08 | Memory Parity Error | 内存奇偶校验错误 |
| 0x0A | Gateway Path Unavailable | 网关路径不可用 |
| 0x0B | Gateway Target Device Failed to Respond | 网关目标设备响应失败 |

## 字节序支持

支持四种字节序格式：

- **ABCD**: 大端序-高字节在前
- **DCBA**: 小端序-低字节在前  
- **BADC**: 大端序-字节交换
- **CDAB**: 小端序-字节交换

## 数据类型转换

支持以下数据类型的转换：

- u16数组 ↔ 字节数组
- u32数组 ↔ 字节数组
- f32数组 ↔ 字节数组（IEEE 754）
- f64数组 ↔ 字节数组（IEEE 754）
- 布尔数组 ↔ 线圈字节

## 使用示例

### TCP客户端
```rust
use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ModbusTcpClient::new("127.0.0.1", 502, 1).await?;
    
    // 读取保持寄存器
    let values = client.read_holding_registers(0, 10).await?;
    println!("Values: {:?}", values);
    
    // 写入单个寄存器
    client.write_single_register(0, 1234).await?;
    
    Ok(())
}
```

### RTU客户端
```rust
use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ModbusRtuClient::new("/dev/ttyUSB0", 1, 9600).await?;
    
    // 读取保持寄存器
    let values = client.read_holding_registers(0, 10).await?;
    println!("Values: {:?}", values);
    
    Ok(())
}
```

### RTU over TCP客户端
```rust
use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ModbusRtuOverTcpClient::new("127.0.0.1", 5020, 1).await?;
    
    // 读取保持寄存器
    let values = client.read_holding_registers(0, 10).await?;
    println!("Values: {:?}", values);
    
    // 写入单个寄存器
    client.write_single_register(0, 1234).await?;
    
    Ok(())
}
```

### RTU over TCP服务器
```rust
use modbus_rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ModbusRtuOverTcpServer::new("127.0.0.1:5020", 1).await?;
    
    // 设置示例数据
    server.set_holding_register(0, 1000);
    server.set_coil(0, true);
    
    // 运行服务器
    server.run().await?;
    
    Ok(())
}
```

## 错误处理

所有函数都返回`Result`类型，包含详细的错误信息：

- `ModbusError::InvalidFunctionCode`: 无效的功能码
- `ModbusError::InvalidExceptionCode`: 无效的异常码
- `ModbusError::InvalidDataLength`: 无效的数据长度
- `ModbusError::CrcCheckFailed`: CRC校验失败
- `ModbusError::IoError`: IO错误
- `ModbusError::SerialError`: 串口错误
- `ModbusError::NetworkError`: 网络错误
- `ModbusError::ProtocolError`: 协议错误

## 性能优化

- 使用异步I/O提高并发性能
- 使用`Arc<Mutex<>>`实现线程安全的数据共享
- 使用`Bytes`和`BytesMut`减少内存分配
- 支持超时设置避免阻塞

## 测试

运行测试：
```bash
cargo test
```

运行示例：
```bash
# TCP客户端示例
cargo run --example tcp_client

# RTU客户端示例  
cargo run --example rtu_client

# RTU over TCP客户端示例
cargo run --example rtu_over_tcp_client

# TCP服务器示例
cargo run --example tcp_server

# RTU服务器示例
cargo run --example rtu_server

# RTU over TCP服务器示例
cargo run --example rtu_over_tcp_server
```
