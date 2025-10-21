# 纯Rust语言实现Modbus协议

为了能够在Rust语言中实现Modbus协议，并完成设备的接入。支持Modbus TCP协议，Modbus RTU协议，以及Modbus RTU over TCP协议。每个协议的详细说明可以参考protocol实现的readme.md文件。

## 项目结构

```
modbus-rs
├── README.md
├── Cargo.toml
├── src
│   ├── lib.rs
│   ├── main.rs
│   ├── protocol
│   │   ├── mod.rs
│   │   ├── modbus_rtu.rs
│   │   ├── modbus_tcp.rs
│   │   ├── modbus_rtu_over_tcp.rs
│   │   └── readme.md
│   ├── utils
│   │   ├── mod.rs
│   │   └── data.rs
│   ├── client
│   │   ├── mod.rs
│   │   ├── modbus_rtu_client.rs
│   │   ├── modbus_tcp_client.rs
│   │   └── modbus_rtu_over_tcp_client.rs
│   └── server
│       ├── mod.rs
│       ├── modbus_rtu_server.rs
│       ├── modbus_tcp_server.rs
│       └── modbus_rtu_over_tcp_server.rs
```

## 技术路线

1. 实现Modbus RTU协议  
   * 实现Modbus RTU协议的功能码  
   * 实现Modbus RTU Over TCP协议  
   后续  
   * 串口协议
2. 实现Modbus TCP协议  
   * 实现Modbus TCP协议的功能码  
   * 实现Modbus TCP协议
3. 实现Modbus RTU over TCP协议
   * 实现RTU格式数据帧在TCP上的传输
   * 无需CRC校验，TCP提供可靠性保证
4. Modbus协议数据转换成需要的数据类型

## 技术栈

### 实现Modbus协议

* Rust语言：纯Rust语言实现
* tokio：异步运行时
* tokio-serial：串口通信（仅在RTU协议中使用）
* tokio-util：工具库
* bytes：字节操作
* serde：序列化/反序列化

## 核心流程

* Modbus协议请求和响应数据帧的生成和解析
* Modbus协议数据**解析规则**，数据类型、字节序描述  
   * ABCD：大端序-高字节在前  
   * DCBA：小端序-低字节在前  
   * BADC：大端序-字节交换  
   * CDAB：小端序-字节交换

我继续创建多从机 RTU 服务器和 RTU over TCP 服务器：


[2 tools called]

现在我需要更新 `mod.rs` 文件来导出这些新的多从机服务器：


[2 tools called]

现在让我创建一个示例来演示如何使用多从机服务器：


[2 tools called]

现在让我检查是否有任何编译错误：


[2 tools called]

完成！我已经成功实现了支持多个 slave ID 的 Modbus 服务器。

## 多从机服务器功能

### 新增的多从机服务器：

1. **ModbusMultiSlaveTcpServer** - 多从机 TCP 服务器
2. **ModbusMultiSlaveRtuServer** - 多从机 RTU 服务器  
3. **ModbusMultiSlaveRtuOverTcpServer** - 多从机 RTU over TCP 服务器

### 主要特性：

**服务器端功能：**
- `add_slave(slave_id)` - 添加新的从机
- `remove_slave(slave_id)` - 移除从机
- `set_coil(slave_id, address, value)` - 为指定从机设置线圈
- `set_discrete_input(slave_id, address, value)` - 为指定从机设置离散输入
- `set_holding_register(slave_id, address, value)` - 为指定从机设置保持寄存器
- `set_input_register(slave_id, address, value)` - 为指定从机设置输入寄存器
- `get_slave_ids()` - 获取所有已注册的从机 ID

**客户端功能（之前已实现）：**
- `read_coils_with_slave_id(slave_id, address, count)` - 按从机 ID 读取线圈
- `read_holding_registers_with_slave_id(slave_id, address, count)` - 按从机 ID 读取保持寄存器
- 等等...

### 使用示例：

**服务器端：**
```rust
let server = ModbusMultiSlaveTcpServer::new("127.0.0.1:502").await?;
server.add_slave(1);
server.add_slave(2);
server.set_coil(1, 0, true)?;
server.set_holding_register(2, 0, 1000)?;
server.run().await?;
```

**客户端：**
```rust
let mut client = ModbusTcpClient::new("127.0.0.1", 502, 1).await?;
let coils = client.read_coils_with_slave_id(1, 0, 2).await?;
let registers = client.read_holding_registers_with_slave_id(2, 0, 2).await?;
```

### 错误处理：
- 当客户端请求不存在的从机 ID 时，服务器会返回异常响应
- 所有操作都有适当的错误处理

现在您可以在同一个 Modbus 服务器上支持多个不同的从机设备，每个从机都有独立的数据存储空间！

## 工具

* Modbus协议分析工具

## 运行示例

```bash
# 运行TCP客户端示例
cargo run --example tcp_client

# 运行RTU客户端示例
cargo run --example rtu_client

# 运行RTU over TCP客户端示例
cargo run --example rtu_over_tcp_client

# 运行TCP服务器示例
cargo run --example tcp_server

# 运行RTU服务器示例
cargo run --example rtu_server

# 运行RTU over TCP服务器示例
cargo run --example rtu_over_tcp_server
```
