use crate::protocol::*;
use crate::utils::DataConverter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Duration;

/// Modbus RTU over TCP客户端 - 支持动态从机ID版本
/// 
/// 这个版本演示了如何在功能码操作时设置从机ID
pub struct ModbusRtuOverTcpClientFlexible {
    stream: TcpStream,
    default_slave_id: u8,
    timeout: Duration,
}

impl ModbusRtuOverTcpClientFlexible {
    /// 创建新的RTU over TCP客户端
    pub async fn new(host: &str, port: u16, default_slave_id: u8) -> Result<Self, ModbusError> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| ModbusError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            stream,
            default_slave_id,
            timeout: Duration::from_millis(5000),
        })
    }
    
    /// 设置超时时间
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    
    /// 读取线圈 - 使用默认从机ID
    pub async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        self.read_coils_with_slave_id(self.default_slave_id, address, count).await
    }
    
    /// 读取线圈 - 指定从机ID
    pub async fn read_coils_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        let request = ModbusRequest {
            slave_id,  // 使用指定的slave_id
            function_code: FunctionCode::ReadCoils,
            address,
            count,
            data: None,
        };
        
        let response = self.send_request(&request).await?;
        
        if response.is_exception {
            return Err(ModbusError::ProtocolError(format!(
                "Exception: {:?}", 
                response.exception_code.unwrap()
            )));
        }
        
        Ok(DataConverter::bytes_to_bool_array(&response.data, count as usize))
    }
    
    /// 读取保持寄存器 - 使用默认从机ID
    pub async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        self.read_holding_registers_with_slave_id(self.default_slave_id, address, count).await
    }
    
    /// 读取保持寄存器 - 指定从机ID
    pub async fn read_holding_registers_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        let request = ModbusRequest {
            slave_id,  // 使用指定的slave_id
            function_code: FunctionCode::ReadHoldingRegisters,
            address,
            count,
            data: None,
        };
        
        let response = self.send_request(&request).await?;
        
        if response.is_exception {
            return Err(ModbusError::ProtocolError(format!(
                "Exception: {:?}", 
                response.exception_code.unwrap()
            )));
        }
        
        DataConverter::bytes_to_u16_array(&response.data, ByteOrder::ABCD)
    }
    
    /// 写入单个寄存器 - 使用默认从机ID
    pub async fn write_single_register(&mut self, address: u16, value: u16) -> Result<(), ModbusError> {
        self.write_single_register_with_slave_id(self.default_slave_id, address, value).await
    }
    
    /// 写入单个寄存器 - 指定从机ID
    pub async fn write_single_register_with_slave_id(&mut self, slave_id: u8, address: u16, value: u16) -> Result<(), ModbusError> {
        let request = ModbusRequest {
            slave_id,  // 使用指定的slave_id
            function_code: FunctionCode::WriteSingleRegister,
            address,
            count: 0,
            data: Some(value.to_be_bytes().to_vec()),
        };
        
        let response = self.send_request(&request).await?;
        
        if response.is_exception {
            return Err(ModbusError::ProtocolError(format!(
                "Exception: {:?}", 
                response.exception_code.unwrap()
            )));
        }
        
        Ok(())
    }
    
    /// 发送请求并接收响应
    async fn send_request(&mut self, request: &ModbusRequest) -> Result<ModbusResponse, ModbusError> {
        // 构建请求帧
        let frame = ModbusRtuOverTcp::build_request(request)?;
        
        // 发送请求
        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;
        
        // 读取响应
        let mut buffer = vec![0u8; 256];
        let bytes_read = tokio::time::timeout(
            self.timeout,
            self.stream.read(&mut buffer)
        ).await
        .map_err(|_| ModbusError::TimeoutError)??;
        
        if bytes_read == 0 {
            return Err(ModbusError::ProtocolError("No response received".to_string()));
        }
        
        // 解析响应
        ModbusRtuOverTcp::parse_response(&buffer[..bytes_read])
    }
}
