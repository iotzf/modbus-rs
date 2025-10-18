pub mod modbus_rtu;
pub mod modbus_tcp;
pub mod modbus_rtu_over_tcp;

pub use modbus_rtu::*;
pub use modbus_tcp::*;
pub use modbus_rtu_over_tcp::*;

use thiserror::Error;

/// Modbus功能码
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FunctionCode {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

impl FunctionCode {
    pub fn from_u8(code: u8) -> Result<Self, ModbusError> {
        match code {
            0x01 => Ok(FunctionCode::ReadCoils),
            0x02 => Ok(FunctionCode::ReadDiscreteInputs),
            0x03 => Ok(FunctionCode::ReadHoldingRegisters),
            0x04 => Ok(FunctionCode::ReadInputRegisters),
            0x05 => Ok(FunctionCode::WriteSingleCoil),
            0x06 => Ok(FunctionCode::WriteSingleRegister),
            0x0F => Ok(FunctionCode::WriteMultipleCoils),
            0x10 => Ok(FunctionCode::WriteMultipleRegisters),
            _ => Err(ModbusError::InvalidFunctionCode(code)),
        }
    }
}

/// Modbus异常码
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExceptionCode {
    IllegalFunction = 0x01,
    IllegalDataAddress = 0x02,
    IllegalDataValue = 0x03,
    SlaveDeviceFailure = 0x04,
    Acknowledge = 0x05,
    SlaveDeviceBusy = 0x06,
    MemoryParityError = 0x08,
    GatewayPathUnavailable = 0x0A,
    GatewayTargetDeviceFailedToRespond = 0x0B,
}

/// Modbus错误类型
#[derive(Error, Debug)]
pub enum ModbusError {
    #[error("Invalid function code: {0}")]
    InvalidFunctionCode(u8),
    
    #[error("Invalid exception code: {0}")]
    InvalidExceptionCode(u8),
    
    #[error("Invalid data length")]
    InvalidDataLength,
    
    #[error("CRC check failed")]
    CrcCheckFailed,
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serial port error: {0}")]
    SerialError(#[from] tokio_serial::Error),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Protocol error: {0}")]
    ProtocolError(String),
    
    #[error("Timeout error")]
    TimeoutError,
}

/// Modbus请求结构
#[derive(Debug, Clone)]
pub struct ModbusRequest {
    pub slave_id: u8,
    pub function_code: FunctionCode,
    pub address: u16,
    pub count: u16,
    pub data: Option<Vec<u8>>,
}

/// Modbus响应结构
#[derive(Debug, Clone)]
pub struct ModbusResponse {
    pub slave_id: u8,
    pub function_code: FunctionCode,
    pub data: Vec<u8>,
    pub is_exception: bool,
    pub exception_code: Option<ExceptionCode>,
}

/// 字节序类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ByteOrder {
    /// ABCD：大端序-高字节在前
    ABCD,
    /// DCBA：小端序-低字节在前
    DCBA,
    /// BADC：大端序-字节交换
    BADC,
    /// CDAB：小端序-字节交换
    CDAB,
}

impl ByteOrder {
    /// 将字节数组转换为u16值
    pub fn bytes_to_u16(&self, bytes: &[u8]) -> Result<u16, ModbusError> {
        if bytes.len() < 2 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        match self {
            ByteOrder::ABCD => Ok(u16::from_be_bytes([bytes[0], bytes[1]])),
            ByteOrder::DCBA => Ok(u16::from_le_bytes([bytes[0], bytes[1]])),
            ByteOrder::BADC => Ok(u16::from_be_bytes([bytes[1], bytes[0]])),
            ByteOrder::CDAB => Ok(u16::from_le_bytes([bytes[1], bytes[0]])),
        }
    }
    
    /// 将u16值转换为字节数组
    pub fn u16_to_bytes(&self, value: u16) -> [u8; 2] {
        match self {
            ByteOrder::ABCD => value.to_be_bytes(),
            ByteOrder::DCBA => value.to_le_bytes(),
            ByteOrder::BADC => {
                let bytes = value.to_be_bytes();
                [bytes[1], bytes[0]]
            },
            ByteOrder::CDAB => {
                let bytes = value.to_le_bytes();
                [bytes[1], bytes[0]]
            },
        }
    }
    
    /// 将字节数组转换为u32值
    pub fn bytes_to_u32(&self, bytes: &[u8]) -> Result<u32, ModbusError> {
        if bytes.len() < 4 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        match self {
            ByteOrder::ABCD => Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            ByteOrder::DCBA => Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])),
            ByteOrder::BADC => Ok(u32::from_be_bytes([bytes[1], bytes[0], bytes[3], bytes[2]])),
            ByteOrder::CDAB => Ok(u32::from_le_bytes([bytes[1], bytes[0], bytes[3], bytes[2]])),
        }
    }
    
    /// 将u32值转换为字节数组
    pub fn u32_to_bytes(&self, value: u32) -> [u8; 4] {
        match self {
            ByteOrder::ABCD => value.to_be_bytes(),
            ByteOrder::DCBA => value.to_le_bytes(),
            ByteOrder::BADC => {
                let bytes = value.to_be_bytes();
                [bytes[1], bytes[0], bytes[3], bytes[2]]
            },
            ByteOrder::CDAB => {
                let bytes = value.to_le_bytes();
                [bytes[1], bytes[0], bytes[3], bytes[2]]
            },
        }
    }
}

/// CRC16计算
pub fn calculate_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    
    for &byte in data {
        crc ^= byte as u16;
        
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    
    crc
}

/// 验证CRC16
pub fn verify_crc16(data: &[u8], expected_crc: u16) -> bool {
    calculate_crc16(data) == expected_crc
}
