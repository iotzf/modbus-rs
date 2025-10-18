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
