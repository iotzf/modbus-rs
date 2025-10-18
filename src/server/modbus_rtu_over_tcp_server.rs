use crate::protocol::*;
use crate::utils::DataConverter;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Modbus RTU over TCP服务器
/// 
/// RTU over TCP服务器通过TCP连接接收RTU格式的数据帧，
/// 但不需要CRC校验，因为TCP已经提供了可靠性保证。
pub struct ModbusRtuOverTcpServer {
    listener: TcpListener,
    slave_id: u8,
    coils: Arc<Mutex<HashMap<u16, bool>>>,
    discrete_inputs: Arc<Mutex<HashMap<u16, bool>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
}

impl ModbusRtuOverTcpServer {
    /// 创建新的RTU over TCP服务器
    pub async fn new(addr: &str, slave_id: u8) -> Result<Self, ModbusError> {
        let listener = TcpListener::bind(addr).await
            .map_err(|e| ModbusError::NetworkError(e.to_string()))?;
        
        Ok(Self {
            listener,
            slave_id,
            coils: Arc::new(Mutex::new(HashMap::new())),
            discrete_inputs: Arc::new(Mutex::new(HashMap::new())),
            holding_registers: Arc::new(Mutex::new(HashMap::new())),
            input_registers: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    
    /// 设置线圈值
    pub fn set_coil(&self, address: u16, value: bool) {
        self.coils.lock().unwrap().insert(address, value);
    }
    
    /// 设置离散输入值
    pub fn set_discrete_input(&self, address: u16, value: bool) {
        self.discrete_inputs.lock().unwrap().insert(address, value);
    }
    
    /// 设置保持寄存器值
    pub fn set_holding_register(&self, address: u16, value: u16) {
        self.holding_registers.lock().unwrap().insert(address, value);
    }
    
    /// 设置输入寄存器值
    pub fn set_input_register(&self, address: u16, value: u16) {
        self.input_registers.lock().unwrap().insert(address, value);
    }
    
    /// 运行服务器
    pub async fn run(&self) -> Result<(), ModbusError> {
        loop {
            match self.listener.accept().await {
                Ok((stream, addr)) => {
                    log::info!("New RTU over TCP connection from: {}", addr);
                    
                    let server_data = ServerData {
                        coils: Arc::clone(&self.coils),
                        discrete_inputs: Arc::clone(&self.discrete_inputs),
                        holding_registers: Arc::clone(&self.holding_registers),
                        input_registers: Arc::clone(&self.input_registers),
                        slave_id: self.slave_id,
                    };
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, server_data).await {
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
    async fn handle_client(mut stream: TcpStream, server_data: ServerData) -> Result<(), ModbusError> {
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
                            if request.slave_id == server_data.slave_id {
                                // 处理请求
                                let response = Self::handle_request(&request, &server_data).await;
                                
                                // 发送响应
                                if let Ok(response_frame) = ModbusRtuOverTcp::build_response(&response) {
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
    async fn handle_request(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        match request.function_code {
            FunctionCode::ReadCoils => Self::handle_read_coils(request, server_data),
            FunctionCode::ReadDiscreteInputs => Self::handle_read_discrete_inputs(request, server_data),
            FunctionCode::ReadHoldingRegisters => Self::handle_read_holding_registers(request, server_data),
            FunctionCode::ReadInputRegisters => Self::handle_read_input_registers(request, server_data),
            FunctionCode::WriteSingleCoil => Self::handle_write_single_coil(request, server_data),
            FunctionCode::WriteSingleRegister => Self::handle_write_single_register(request, server_data),
            FunctionCode::WriteMultipleCoils => Self::handle_write_multiple_coils(request, server_data),
            FunctionCode::WriteMultipleRegisters => Self::handle_write_multiple_registers(request, server_data),
        }
    }
    
    /// 处理读取线圈请求
    fn handle_read_coils(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let coils = server_data.coils.lock().unwrap();
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
            slave_id: server_data.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取离散输入请求
    fn handle_read_discrete_inputs(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let discrete_inputs = server_data.discrete_inputs.lock().unwrap();
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
            slave_id: server_data.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取保持寄存器请求
    fn handle_read_holding_registers(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let holding_registers = server_data.holding_registers.lock().unwrap();
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
            slave_id: server_data.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理读取输入寄存器请求
    fn handle_read_input_registers(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let input_registers = server_data.input_registers.lock().unwrap();
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
            slave_id: server_data.slave_id,
            function_code: request.function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        }
    }
    
    /// 处理写入单个线圈请求
    fn handle_write_single_coil(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let mut coils = server_data.coils.lock().unwrap();
        coils.insert(request.address, request.count > 0);
        
        ModbusResponse {
            slave_id: server_data.slave_id,
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
    fn handle_write_single_register(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let mut holding_registers = server_data.holding_registers.lock().unwrap();
        let value = u16::from_be_bytes([
            request.data.as_ref().unwrap()[0],
            request.data.as_ref().unwrap()[1],
        ]);
        holding_registers.insert(request.address, value);
        
        ModbusResponse {
            slave_id: server_data.slave_id,
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
    fn handle_write_multiple_coils(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let mut coils = server_data.coils.lock().unwrap();
        let bools = DataConverter::bytes_to_bool_array(request.data.as_ref().unwrap(), request.count as usize);
        
        for (i, value) in bools.iter().enumerate() {
            coils.insert(request.address + i as u16, *value);
        }
        
        ModbusResponse {
            slave_id: server_data.slave_id,
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
    fn handle_write_multiple_registers(request: &ModbusRequest, server_data: &ServerData) -> ModbusResponse {
        let mut holding_registers = server_data.holding_registers.lock().unwrap();
        let values = DataConverter::bytes_to_u16_array(request.data.as_ref().unwrap(), ByteOrder::ABCD).unwrap();
        
        for (i, value) in values.iter().enumerate() {
            holding_registers.insert(request.address + i as u16, *value);
        }
        
        ModbusResponse {
            slave_id: server_data.slave_id,
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

/// 服务器数据共享结构
struct ServerData {
    coils: Arc<Mutex<HashMap<u16, bool>>>,
    discrete_inputs: Arc<Mutex<HashMap<u16, bool>>>,
    holding_registers: Arc<Mutex<HashMap<u16, u16>>>,
    input_registers: Arc<Mutex<HashMap<u16, u16>>>,
    slave_id: u8,
}
