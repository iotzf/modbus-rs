use crate::protocol::*;
use crate::utils::DataConverter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Duration;
use std::sync::atomic::{AtomicU16, Ordering};

/// Modbus TCP客户端
pub struct ModbusTcpClient {
    stream: TcpStream,
    slave_id: u8,
    timeout: Duration,
    transaction_id: AtomicU16,
}

impl ModbusTcpClient {
    /// 创建新的TCP客户端
    pub async fn new(host: &str, port: u16, slave_id: u8) -> Result<Self, ModbusError> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await
            .map_err(|e| ModbusError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            stream,
            slave_id,
            timeout: Duration::from_millis(5000),
            transaction_id: AtomicU16::new(1),
        })
    }
    
    /// 设置超时时间
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }
    
    /// 读取线圈
    pub async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        self.read_coils_with_slave_id(self.slave_id, address, count).await
    }

    /// 按指定从机地址读取线圈
    pub async fn read_coils_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        let request = ModbusRequest {
            slave_id,
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
    
    /// 读取离散输入
    pub async fn read_discrete_inputs(&mut self, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        self.read_discrete_inputs_with_slave_id(self.slave_id, address, count).await
    }

    /// 按指定从机地址读取离散输入
    pub async fn read_discrete_inputs_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<bool>, ModbusError> {
        let request = ModbusRequest {
            slave_id,
            function_code: FunctionCode::ReadDiscreteInputs,
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
    
    /// 读取保持寄存器
    pub async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        self.read_holding_registers_with_slave_id(self.slave_id, address, count).await
    }

    /// 按指定从机地址读取保持寄存器
    pub async fn read_holding_registers_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        let request = ModbusRequest {
            slave_id,
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
    
    /// 读取输入寄存器
    pub async fn read_input_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        self.read_input_registers_with_slave_id(self.slave_id, address, count).await
    }

    /// 按指定从机地址读取输入寄存器
    pub async fn read_input_registers_with_slave_id(&mut self, slave_id: u8, address: u16, count: u16) -> Result<Vec<u16>, ModbusError> {
        let request = ModbusRequest {
            slave_id,
            function_code: FunctionCode::ReadInputRegisters,
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
    
    /// 写入单个线圈
    pub async fn write_single_coil(&mut self, address: u16, value: bool) -> Result<(), ModbusError> {
        self.write_single_coil_with_slave_id(self.slave_id, address, value).await
    }

    /// 按指定从机地址写入单个线圈
    pub async fn write_single_coil_with_slave_id(&mut self, slave_id: u8, address: u16, value: bool) -> Result<(), ModbusError> {
        let request = ModbusRequest {
            slave_id,
            function_code: FunctionCode::WriteSingleCoil,
            address,
            count: if value { 1 } else { 0 },
            data: None,
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
    
    /// 写入单个寄存器
    pub async fn write_single_register(&mut self, address: u16, value: u16) -> Result<(), ModbusError> {
        self.write_single_register_with_slave_id(self.slave_id, address, value).await
    }

    /// 按指定从机地址写入单个寄存器
    pub async fn write_single_register_with_slave_id(&mut self, slave_id: u8, address: u16, value: u16) -> Result<(), ModbusError> {
        let request = ModbusRequest {
            slave_id,
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
    
    /// 写入多个线圈
    pub async fn write_multiple_coils(&mut self, address: u16, values: &[bool]) -> Result<(), ModbusError> {
        self.write_multiple_coils_with_slave_id(self.slave_id, address, values).await
    }

    /// 按指定从机地址写入多个线圈
    pub async fn write_multiple_coils_with_slave_id(&mut self, slave_id: u8, address: u16, values: &[bool]) -> Result<(), ModbusError> {
        let request = ModbusRequest {
            slave_id,
            function_code: FunctionCode::WriteMultipleCoils,
            address,
            count: values.len() as u16,
            data: Some(DataConverter::bool_array_to_bytes(values)),
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
    
    /// 写入多个寄存器
    pub async fn write_multiple_registers(&mut self, address: u16, values: &[u16]) -> Result<(), ModbusError> {
        self.write_multiple_registers_with_slave_id(self.slave_id, address, values).await
    }

    /// 按指定从机地址写入多个寄存器
    pub async fn write_multiple_registers_with_slave_id(&mut self, slave_id: u8, address: u16, values: &[u16]) -> Result<(), ModbusError> {
        let request = ModbusRequest {
            slave_id,
            function_code: FunctionCode::WriteMultipleRegisters,
            address,
            count: values.len() as u16,
            data: Some(DataConverter::u16_array_to_bytes(values, ByteOrder::ABCD)),
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
        // 获取事务ID
        let transaction_id = self.transaction_id.fetch_add(1, Ordering::SeqCst);
        
        // 构建请求帧
        let frame = ModbusTcp::build_request(request, transaction_id)?;
        
        // 发送请求
        self.stream.write_all(&frame).await?;
        self.stream.flush().await?;
        
        // 读取MBAP头部
        let mut mbap_header = [0u8; 6];
        tokio::time::timeout(
            self.timeout,
            self.stream.read_exact(&mut mbap_header)
        ).await
        .map_err(|_| ModbusError::TimeoutError)??;
        
        // 解析长度
        let length = u16::from_be_bytes([mbap_header[4], mbap_header[5]]) as usize;
        
        // 读取剩余数据
        let mut buffer = vec![0u8; length];
        tokio::time::timeout(
            self.timeout,
            self.stream.read_exact(&mut buffer)
        ).await
        .map_err(|_| ModbusError::TimeoutError)??;
        
        // 组合完整响应
        let mut full_response = Vec::new();
        full_response.extend_from_slice(&mbap_header);
        full_response.extend_from_slice(&buffer);
        
        // 解析响应
        let (_, response) = ModbusTcp::parse_response(&full_response)?;
        
        Ok(response)
    }
}
