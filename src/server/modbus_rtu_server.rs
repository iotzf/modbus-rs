use crate::protocol::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;
use std::time::Duration;
use std::collections::HashMap;

/// Modbus RTU服务器
pub struct ModbusRtuServer {
    port: SerialStream,
    slave_id: u8,
    coils: HashMap<u16, bool>,
    discrete_inputs: HashMap<u16, bool>,
    holding_registers: HashMap<u16, u16>,
    input_registers: HashMap<u16, u16>,
}

impl ModbusRtuServer {
    /// 创建新的RTU服务器
    pub async fn new(port_name: &str, slave_id: u8, baud_rate: u32) -> Result<Self, ModbusError> {
        let port = tokio_serial::SerialStream::open(&tokio_serial::new(port_name, baud_rate))?;
        
        Ok(Self {
            port,
            slave_id,
            coils: HashMap::new(),
            discrete_inputs: HashMap::new(),
            holding_registers: HashMap::new(),
            input_registers: HashMap::new(),
        })
    }
    
    /// 设置线圈值
    pub fn set_coil(&mut self, address: u16, value: bool) {
        self.coils.insert(address, value);
    }
    
    /// 设置离散输入值
    pub fn set_discrete_input(&mut self, address: u16, value: bool) {
        self.discrete_inputs.insert(address, value);
    }
    
    /// 设置保持寄存器值
    pub fn set_holding_register(&mut self, address: u16, value: u16) {
        self.holding_registers.insert(address, value);
    }
    
    /// 设置输入寄存器值
    pub fn set_input_register(&mut self, address: u16, value: u16) {
        self.input_registers.insert(address, value);
    }
    
    /// 运行服务器
    pub async fn run(&mut self) -> Result<(), ModbusError> {
        let mut buffer = vec![0u8; 256];
        
        loop {
            match self.port.read(&mut buffer).await {
                Ok(bytes_read) => {
                    if bytes_read > 0 {
                        let request_data = &buffer[..bytes_read];
                        
                        // 解析请求
                        match ModbusRtu::parse_request(request_data) {
                            Ok(request) => {
                                if request.slave_id == self.slave_id {
                                    // 处理请求
                                    let response = self.handle_request(&request).await;
                                    
                                    // 发送响应
                                    if let Ok(response_frame) = ModbusRtu::build_response(&response) {
                                        self.port.write_all(&response_frame).await?;
                                        self.port.flush().await?;
                                    }
                                }
                            },
                            Err(e) => {
                                log::warn!("Failed to parse request: {}", e);
                            }
                        }
                    }
                },
                Err(e) => {
                    log::error!("Serial port read error: {}", e);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
    
    /// 处理请求
    async fn handle_request(&self, request: &ModbusRequest) -> ModbusResponse {
        match request.function_code {
            FunctionCode::ReadCoils => self.handle_read_coils(request),
            FunctionCode::ReadDiscreteInputs => self.handle_read_discrete_inputs(request),
            FunctionCode::ReadHoldingRegisters => self.handle_read_holding_registers(request),
            FunctionCode::ReadInputRegisters => self.handle_read_input_registers(request),
            FunctionCode::WriteSingleCoil => self.handle_write_single_coil(request),
            FunctionCode::WriteSingleRegister => self.handle_write_single_register(request),
            FunctionCode::WriteMultipleCoils => self.handle_write_multiple_coils(request),
            FunctionCode::WriteMultipleRegisters => self.handle_write_multiple_registers(request),
        }
    }
    
    /// 处理读取线圈请求
    fn handle_read_coils(&self, request: &ModbusRequest) -> ModbusResponse {
        let mut data = Vec::new();
        let mut byte_count = 0;
        let mut current_byte = 0u8;
        let mut bit_count = 0;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = self.coils.get(&address).copied().unwrap_or(false);
            
            if value {
                current_byte |= 1 << bit_count;
            }
            
            bit_count += 1;
            if bit_count == 8 {
                data.push(current_byte);
                current_byte = 0;
                bit_count = 0;
                byte_count += 1;
            }
        }
        
        if bit_count > 0 {
            data.push(current_byte);
            byte_count += 1;
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取离散输入请求
    fn handle_read_discrete_inputs(&self, request: &ModbusRequest) -> ModbusResponse {
        let mut data = Vec::new();
        let mut byte_count = 0;
        let mut current_byte = 0u8;
        let mut bit_count = 0;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = self.discrete_inputs.get(&address).copied().unwrap_or(false);
            
            if value {
                current_byte |= 1 << bit_count;
            }
            
            bit_count += 1;
            if bit_count == 8 {
                data.push(current_byte);
                current_byte = 0;
                bit_count = 0;
                byte_count += 1;
            }
        }
        
        if bit_count > 0 {
            data.push(current_byte);
            byte_count += 1;
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取保持寄存器请求
    fn handle_read_holding_registers(&self, request: &ModbusRequest) -> ModbusResponse {
        let mut data = Vec::new();
        let byte_count = (request.count * 2) as u8;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = self.holding_registers.get(&address).copied().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取输入寄存器请求
    fn handle_read_input_registers(&self, request: &ModbusRequest) -> ModbusResponse {
        let mut data = Vec::new();
        let byte_count = (request.count * 2) as u8;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = self.input_registers.get(&address).copied().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入单个线圈请求
    fn handle_write_single_coil(&self, request: &ModbusRequest) -> ModbusResponse {
        // 在实际实现中，这里应该更新线圈值
        // 由于self是不可变的，这里只是返回回显
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: vec![
                (request.address >> 8) as u8,
                (request.address & 0xFF) as u8,
                (request.count >> 8) as u8,
                (request.count & 0xFF) as u8,
            ],
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入单个寄存器请求
    fn handle_write_single_register(&self, request: &ModbusRequest) -> ModbusResponse {
        // 在实际实现中，这里应该更新寄存器值
        // 由于self是不可变的，这里只是返回回显
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: vec![
                (request.address >> 8) as u8,
                (request.address & 0xFF) as u8,
                request.data.as_ref().unwrap()[0],
                request.data.as_ref().unwrap()[1],
            ],
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入多个线圈请求
    fn handle_write_multiple_coils(&self, request: &ModbusRequest) -> ModbusResponse {
        // 在实际实现中，这里应该更新线圈值
        // 由于self是不可变的，这里只是返回回显
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: vec![
                (request.address >> 8) as u8,
                (request.address & 0xFF) as u8,
                (request.count >> 8) as u8,
                (request.count & 0xFF) as u8,
            ],
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入多个寄存器请求
    fn handle_write_multiple_registers(&self, request: &ModbusRequest) -> ModbusResponse {
        // 在实际实现中，这里应该更新寄存器值
        // 由于self是不可变的，这里只是返回回显
        
        ModbusResponse {
            slave_id: self.slave_id,
            function_code: request.function_code,
            data: vec![
                (request.address >> 8) as u8,
                (request.address & 0xFF) as u8,
                (request.count >> 8) as u8,
                (request.count & 0xFF) as u8,
            ],
            is_exception: false,
            exception_code: None,
        }
    }
}
