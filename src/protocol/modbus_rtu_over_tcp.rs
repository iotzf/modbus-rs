use super::*;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Modbus RTU over TCP协议实现
/// 
/// RTU over TCP是一种混合协议，它在TCP连接上传输RTU格式的数据帧，
/// 但不需要CRC校验，因为TCP已经提供了可靠性保证。
pub struct ModbusRtuOverTcp;

impl ModbusRtuOverTcp {
    /// 构建RTU over TCP请求帧
    /// 
    /// RTU over TCP帧格式：
    /// - 从机地址 (1字节)
    /// - 功能码 (1字节)  
    /// - 数据 (N字节)
    /// - 无CRC校验
    pub fn build_request(request: &ModbusRequest) -> Result<Bytes, ModbusError> {
        let mut frame = BytesMut::new();
        
        // 从机地址
        frame.put_u8(request.slave_id);
        
        // 功能码
        frame.put_u8(request.function_code as u8);
        
        // 地址（大端序）
        frame.put_u16(request.address);
        
        match request.function_code {
            FunctionCode::ReadCoils | 
            FunctionCode::ReadDiscreteInputs | 
            FunctionCode::ReadHoldingRegisters | 
            FunctionCode::ReadInputRegisters => {
                // 读取数量
                frame.put_u16(request.count);
            },
            FunctionCode::WriteSingleCoil => {
                // 写入值
                let value = if request.count > 0 { 0xFF00 } else { 0x0000 };
                frame.put_u16(value);
            },
            FunctionCode::WriteSingleRegister => {
                // 写入值
                if let Some(data) = &request.data {
                    if data.len() >= 2 {
                        frame.put_u16(u16::from_be_bytes([data[0], data[1]]));
                    } else {
                        return Err(ModbusError::InvalidDataLength);
                    }
                } else {
                    return Err(ModbusError::InvalidDataLength);
                }
            },
            FunctionCode::WriteMultipleCoils => {
                // 线圈数量
                frame.put_u16(request.count);
                // 字节数
                let byte_count = ((request.count + 7) / 8) as u8;
                frame.put_u8(byte_count);
                // 线圈数据
                if let Some(data) = &request.data {
                    frame.extend_from_slice(data);
                } else {
                    return Err(ModbusError::InvalidDataLength);
                }
            },
            FunctionCode::WriteMultipleRegisters => {
                // 寄存器数量
                frame.put_u16(request.count);
                // 字节数
                let byte_count = (request.count * 2) as u8;
                frame.put_u8(byte_count);
                // 寄存器数据
                if let Some(data) = &request.data {
                    frame.extend_from_slice(data);
                } else {
                    return Err(ModbusError::InvalidDataLength);
                }
            },
        }
        
        Ok(frame.freeze())
    }
    
    /// 解析RTU over TCP响应帧
    pub fn parse_response(data: &[u8]) -> Result<ModbusResponse, ModbusError> {
        if data.len() < 2 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut buf = Bytes::copy_from_slice(data);
        let slave_id = buf.get_u8();
        let function_code_byte = buf.get_u8();
        
        // 检查是否为异常响应
        if function_code_byte & 0x80 != 0 {
            let function_code = FunctionCode::from_u8(function_code_byte & 0x7F)?;
            let exception_code_byte = buf.get_u8();
            
            let exception_code = match exception_code_byte {
                0x01 => ExceptionCode::IllegalFunction,
                0x02 => ExceptionCode::IllegalDataAddress,
                0x03 => ExceptionCode::IllegalDataValue,
                0x04 => ExceptionCode::SlaveDeviceFailure,
                0x05 => ExceptionCode::Acknowledge,
                0x06 => ExceptionCode::SlaveDeviceBusy,
                0x08 => ExceptionCode::MemoryParityError,
                0x0A => ExceptionCode::GatewayPathUnavailable,
                0x0B => ExceptionCode::GatewayTargetDeviceFailedToRespond,
                _ => return Err(ModbusError::InvalidExceptionCode(exception_code_byte)),
            };
            
            return Ok(ModbusResponse {
                slave_id,
                function_code,
                data: Vec::new(),
                is_exception: true,
                exception_code: Some(exception_code),
            });
        }
        
        let function_code = FunctionCode::from_u8(function_code_byte)?;
        
        // 解析数据部分
        let mut response_data = Vec::new();
        
        match function_code {
            FunctionCode::ReadCoils | 
            FunctionCode::ReadDiscreteInputs => {
                let byte_count = buf.get_u8();
                for _ in 0..byte_count {
                    response_data.push(buf.get_u8());
                }
            },
            FunctionCode::ReadHoldingRegisters | 
            FunctionCode::ReadInputRegisters => {
                let byte_count = buf.get_u8();
                for _ in 0..byte_count {
                    response_data.push(buf.get_u8());
                }
            },
            FunctionCode::WriteSingleCoil | 
            FunctionCode::WriteSingleRegister => {
                // 回显地址和值
                response_data.push(buf.get_u8()); // 地址高字节
                response_data.push(buf.get_u8()); // 地址低字节
                response_data.push(buf.get_u8()); // 值高字节
                response_data.push(buf.get_u8()); // 值低字节
            },
            FunctionCode::WriteMultipleCoils | 
            FunctionCode::WriteMultipleRegisters => {
                // 回显地址和数量
                response_data.push(buf.get_u8()); // 地址高字节
                response_data.push(buf.get_u8()); // 地址低字节
                response_data.push(buf.get_u8()); // 数量高字节
                response_data.push(buf.get_u8()); // 数量低字节
            },
        }
        
        Ok(ModbusResponse {
            slave_id,
            function_code,
            data: response_data,
            is_exception: false,
            exception_code: None,
        })
    }
    
    /// 构建RTU over TCP响应帧
    pub fn build_response(response: &ModbusResponse) -> Result<Bytes, ModbusError> {
        let mut frame = BytesMut::new();
        
        // 从机地址
        frame.put_u8(response.slave_id);
        
        if response.is_exception {
            // 异常响应
            frame.put_u8((response.function_code as u8) | 0x80);
            frame.put_u8(response.exception_code.unwrap() as u8);
        } else {
            // 正常响应
            frame.put_u8(response.function_code as u8);
            frame.extend_from_slice(&response.data);
        }
        
        Ok(frame.freeze())
    }
    
    /// 解析RTU over TCP请求帧
    pub fn parse_request(data: &[u8]) -> Result<ModbusRequest, ModbusError> {
        if data.len() < 4 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut buf = Bytes::copy_from_slice(data);
        let slave_id = buf.get_u8();
        let function_code = FunctionCode::from_u8(buf.get_u8())?;
        let address = buf.get_u16();
        
        let mut request = ModbusRequest {
            slave_id,
            function_code,
            address,
            count: 0,
            data: None,
        };
        
        match function_code {
            FunctionCode::ReadCoils | 
            FunctionCode::ReadDiscreteInputs | 
            FunctionCode::ReadHoldingRegisters | 
            FunctionCode::ReadInputRegisters => {
                request.count = buf.get_u16();
            },
            FunctionCode::WriteSingleCoil => {
                request.count = buf.get_u16();
            },
            FunctionCode::WriteSingleRegister => {
                let value = buf.get_u16();
                request.data = Some(value.to_be_bytes().to_vec());
            },
            FunctionCode::WriteMultipleCoils => {
                request.count = buf.get_u16();
                let byte_count = buf.get_u8();
                let mut data = Vec::new();
                for _ in 0..byte_count {
                    data.push(buf.get_u8());
                }
                request.data = Some(data);
            },
            FunctionCode::WriteMultipleRegisters => {
                request.count = buf.get_u16();
                let byte_count = buf.get_u16();
                let mut data = Vec::new();
                for _ in 0..byte_count {
                    data.push(buf.get_u8());
                }
                request.data = Some(data);
            },
        }
        
        Ok(request)
    }
}
