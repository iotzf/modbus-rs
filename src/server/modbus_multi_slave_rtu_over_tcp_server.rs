use crate::protocol::*;
use crate::utils::DataConverter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 多从机 Modbus RTU over TCP 服务器
/// 
/// 支持多个 slave ID 的 RTU over TCP 服务器，每个 slave ID 都有独立的数据存储
pub struct ModbusMultiSlaveRtuOverTcpServer {
    listener: TcpListener,
    slaves: Arc<Mutex<HashMap<u8, SlaveData>>>,
}

/// 单个从机的数据存储
#[derive(Clone)]
struct SlaveData {
    coils: Arc<Mutex<HashMap<u16, bool>>>,
    discrete_inputs: Arc<Mutex<HashMap<u16, bool>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl ModbusMultiSlaveRtuOverTcpServer {
    /// 创建新的多从机 RTU over TCP 服务器
    pub async fn new(addr: &str) -> Result<Self, ModbusError> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| ModbusError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            listener,
            slaves: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// 添加从机
    pub fn add_slave(&self, slave_id: u8) {
        let mut slaves = self.slaves.lock().unwrap();
        slaves.insert(slave_id, SlaveData {
            coils: Arc::new(Mutex::new(HashMap::new())),
            discrete_inputs: Arc::new(Mutex::new(HashMap::new())),
            holding_registers: Arc::new(Mutex::new(HashMap::new())),
            input_registers: Arc::new(Mutex::new(HashMap::new())),
        });
    }
    
    /// 移除从机
    pub fn remove_slave(&self, slave_id: u8) {
        let mut slaves = self.slaves.lock().unwrap();
        slaves.remove(&slave_id);
    }
    
    /// 设置指定从机的线圈值
    pub fn set_coil(&self, slave_id: u8, address: u16, value: bool) -> Result<(), ModbusError> {
        let slaves = self.slaves.lock().unwrap();
        if let Some(slave_data) = slaves.get(&slave_id) {
            slave_data.coils.lock().unwrap().insert(address, value);
            Ok(())
        } else {
            Err(ModbusError::ProtocolError(format!("Slave {} not found", slave_id)))
        }
    }
    
    /// 设置指定从机的离散输入值
    pub fn set_discrete_input(&self, slave_id: u8, address: u16, value: bool) -> Result<(), ModbusError> {
        let slaves = self.slaves.lock().unwrap();
        if let Some(slave_data) = slaves.get(&slave_id) {
            slave_data.discrete_inputs.lock().unwrap().insert(address, value);
            Ok(())
        } else {
            Err(ModbusError::ProtocolError(format!("Slave {} not found", slave_id)))
        }
    }
    
    /// 设置指定从机的保持寄存器值
    pub fn set_holding_register(&self, slave_id: u8, address: u16, value: u16) -> Result<(), ModbusError> {
        let slaves = self.slaves.lock().unwrap();
        if let Some(slave_data) = slaves.get(&slave_id) {
            slave_data.holding_registers.lock().unwrap().insert(address, value);
            Ok(())
        } else {
            Err(ModbusError::ProtocolError(format!("Slave {} not found", slave_id)))
        }
    }
    
    /// 设置指定从机的输入寄存器值
    pub fn set_input_register(&self, slave_id: u8, address: u16, value: u16) -> Result<(), ModbusError> {
        let slaves = self.slaves.lock().unwrap();
        if let Some(slave_data) = slaves.get(&slave_id) {
            slave_data.input_registers.lock().unwrap().insert(address, value);
            Ok(())
        } else {
            Err(ModbusError::ProtocolError(format!("Slave {} not found", slave_id)))
        }
    }
    
    /// 获取所有已注册的从机 ID
    pub fn get_slave_ids(&self) -> Vec<u8> {
        let slaves = self.slaves.lock().unwrap();
        slaves.keys().copied().collect()
    }
    
    /// 运行服务器
    pub async fn run(&self) -> Result<(), ModbusError> {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("New RTU over TCP connection from: {}", addr);
                    
                    let slaves = Arc::clone(&self.slaves);
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, slaves).await {
                            log::error!("Error handling RTU over TCP client: {}", e);
                        }
                    });
                },
                Err(e) => {
                    log::error!("Failed to accept RTU over TCP connection: {}", e);
                }
            }
        }
    }
    
    /// 处理客户端连接
    async fn handle_client(mut stream: TcpStream, slaves: Arc<Mutex<HashMap<u8, SlaveData>>>) -> Result<(), ModbusError> {
        let mut buffer = vec![0u8; 1024];
        
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    log::info!("RTU over TCP client disconnected");
                    break;
                },
                Ok(bytes_read) => {
                    let request_data = &buffer[..bytes_read];
                    
                    // 解析请求
                    match ModbusRtuOverTcp::parse_request(request_data) {
                        Ok(request) => {
                            // 检查从机是否存在并克隆数据
                            let slave_data = {
                                let slaves_guard = slaves.lock().unwrap();
                                slaves_guard.get(&request.slave_id).cloned()
                            };
                            
                            if let Some(slave_data) = slave_data {
                                // 处理请求
                                let response = Self::handle_request(&request, &slave_data).await;
                                
                                // 发送响应
                                if let Ok(response_frame) = ModbusRtuOverTcp::build_response(&response) {
                                    stream.write_all(&response_frame).await?;
                                    stream.flush().await?;
                                }
                            } else {
                                // 从机不存在，返回异常响应
                                let exception_response = ModbusResponse {
                                    slave_id: request.slave_id,
                                    function_code: request.function_code,
                                    data: vec![],
                                    is_exception: true,
                                    exception_code: Some(ExceptionCode::IllegalDataAddress),
                                };
                                
                                if let Ok(response_frame) = ModbusRtuOverTcp::build_response(&exception_response) {
                                    stream.write_all(&response_frame).await?;
                                    stream.flush().await?;
                                }
                            }
                        },
                        Err(e) => {
                            log::warn!("Failed to parse RTU over TCP request: {}", e);
                        }
                    }
                },
                Err(e) => {
                    log::error!("RTU over TCP read error: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// 处理请求
    async fn handle_request(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        match request.function_code {
            FunctionCode::ReadCoils => Self::handle_read_coils(request, slave_data),
            FunctionCode::ReadDiscreteInputs => Self::handle_read_discrete_inputs(request, slave_data),
            FunctionCode::ReadHoldingRegisters => Self::handle_read_holding_registers(request, slave_data),
            FunctionCode::ReadInputRegisters => Self::handle_read_input_registers(request, slave_data),
            FunctionCode::WriteSingleCoil => Self::handle_write_single_coil(request, slave_data),
            FunctionCode::WriteSingleRegister => Self::handle_write_single_register(request, slave_data),
            FunctionCode::WriteMultipleCoils => Self::handle_write_multiple_coils(request, slave_data),
            FunctionCode::WriteMultipleRegisters => Self::handle_write_multiple_registers(request, slave_data),
        }
    }
    
    /// 处理读取线圈请求
    fn handle_read_coils(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let coils = slave_data.coils.lock().unwrap();
        let mut data = Vec::new();
        let mut byte_count = 0;
        let mut current_byte = 0u8;
        let mut bit_count = 0;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = coils.get(&address).copied().unwrap_or(false);
            
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
            slave_id: request.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取离散输入请求
    fn handle_read_discrete_inputs(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let discrete_inputs = slave_data.discrete_inputs.lock().unwrap();
        let mut data = Vec::new();
        let mut byte_count = 0;
        let mut current_byte = 0u8;
        let mut bit_count = 0;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = discrete_inputs.get(&address).copied().unwrap_or(false);
            
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
            slave_id: request.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取保持寄存器请求
    fn handle_read_holding_registers(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let holding_registers = slave_data.holding_registers.lock().unwrap();
        let mut data = Vec::new();
        let byte_count = (request.count * 2) as u8;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = holding_registers.get(&address).copied().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: request.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取输入寄存器请求
    fn handle_read_input_registers(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let input_registers = slave_data.input_registers.lock().unwrap();
        let mut data = Vec::new();
        let byte_count = (request.count * 2) as u8;
        
        for i in 0..request.count {
            let address = request.address + i;
            let value = input_registers.get(&address).copied().unwrap_or(0);
            data.extend_from_slice(&value.to_be_bytes());
        }
        
        let mut response_data = vec![byte_count];
        response_data.extend_from_slice(&data);
        
        ModbusResponse {
            slave_id: request.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入单个线圈请求
    fn handle_write_single_coil(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let mut coils = slave_data.coils.lock().unwrap();
        coils.insert(request.address, request.count > 0);
        
        ModbusResponse {
            slave_id: request.slave_id,
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
    fn handle_write_single_register(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let mut holding_registers = slave_data.holding_registers.lock().unwrap();
        let value = u16::from_be_bytes([
            request.data.as_ref().unwrap()[0],
            request.data.as_ref().unwrap()[1],
        ]);
        holding_registers.insert(request.address, value);
        
        ModbusResponse {
            slave_id: request.slave_id,
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
    fn handle_write_multiple_coils(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let mut coils = slave_data.coils.lock().unwrap();
        let bools = DataConverter::bytes_to_bool_array(request.data.as_ref().unwrap(), request.count as usize);
        
        for (i, value) in bools.iter().enumerate() {
            coils.insert(request.address + i as u16, *value);
        }
        
        ModbusResponse {
            slave_id: request.slave_id,
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
    fn handle_write_multiple_registers(request: &ModbusRequest, slave_data: &SlaveData) -> ModbusResponse {
        let mut holding_registers = slave_data.holding_registers.lock().unwrap();
        let values = DataConverter::bytes_to_u16_array(request.data.as_ref().unwrap(), ByteOrder::ABCD).unwrap();
        
        for (i, value) in values.iter().enumerate() {
            holding_registers.insert(request.address + i as u16, *value);
        }
        
        ModbusResponse {
            slave_id: request.slave_id,
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
