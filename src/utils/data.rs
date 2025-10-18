use crate::protocol::{ByteOrder, ModbusError};

/// 数据类型转换工具
pub struct DataConverter;

impl DataConverter {
    /// 将字节数组转换为u16数组
    pub fn bytes_to_u16_array(bytes: &[u8], byte_order: ByteOrder) -> Result<Vec<u16>, ModbusError> {
        if bytes.len() % 2 != 0 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut result = Vec::new();
        for chunk in bytes.chunks(2) {
            result.push(byte_order.bytes_to_u16(chunk)?);
        }
        
        Ok(result)
    }
    
    /// 将u16数组转换为字节数组
    pub fn u16_array_to_bytes(values: &[u16], byte_order: ByteOrder) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            let bytes = byte_order.u16_to_bytes(value);
            result.extend_from_slice(&bytes);
        }
        result
    }
    
    /// 将字节数组转换为u32数组
    pub fn bytes_to_u32_array(bytes: &[u8], byte_order: ByteOrder) -> Result<Vec<u32>, ModbusError> {
        if bytes.len() % 4 != 0 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut result = Vec::new();
        for chunk in bytes.chunks(4) {
            result.push(byte_order.bytes_to_u32(chunk)?);
        }
        
        Ok(result)
    }
    
    /// 将u32数组转换为字节数组
    pub fn u32_array_to_bytes(values: &[u32], byte_order: ByteOrder) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            let bytes = byte_order.u32_to_bytes(value);
            result.extend_from_slice(&bytes);
        }
        result
    }
    
    /// 将字节数组转换为f32数组（IEEE 754）
    pub fn bytes_to_f32_array(bytes: &[u8], byte_order: ByteOrder) -> Result<Vec<f32>, ModbusError> {
        if bytes.len() % 4 != 0 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut result = Vec::new();
        for chunk in bytes.chunks(4) {
            let u32_value = byte_order.bytes_to_u32(chunk)?;
            result.push(f32::from_bits(u32_value));
        }
        
        Ok(result)
    }
    
    /// 将f32数组转换为字节数组（IEEE 754）
    pub fn f32_array_to_bytes(values: &[f32], byte_order: ByteOrder) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            let u32_value = value.to_bits();
            let bytes = byte_order.u32_to_bytes(u32_value);
            result.extend_from_slice(&bytes);
        }
        result
    }
    
    /// 将字节数组转换为f64数组（IEEE 754）
    pub fn bytes_to_f64_array(bytes: &[u8], byte_order: ByteOrder) -> Result<Vec<f64>, ModbusError> {
        if bytes.len() % 8 != 0 {
            return Err(ModbusError::InvalidDataLength);
        }
        
        let mut result = Vec::new();
        for chunk in bytes.chunks(8) {
            let u64_value = match byte_order {
                ByteOrder::ABCD => u64::from_be_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3],
                    chunk[4], chunk[5], chunk[6], chunk[7]
                ]),
                ByteOrder::DCBA => u64::from_le_bytes([
                    chunk[0], chunk[1], chunk[2], chunk[3],
                    chunk[4], chunk[5], chunk[6], chunk[7]
                ]),
                ByteOrder::BADC => u64::from_be_bytes([
                    chunk[1], chunk[0], chunk[3], chunk[2],
                    chunk[5], chunk[4], chunk[7], chunk[6]
                ]),
                ByteOrder::CDAB => u64::from_le_bytes([
                    chunk[1], chunk[0], chunk[3], chunk[2],
                    chunk[5], chunk[4], chunk[7], chunk[6]
                ]),
            };
            result.push(f64::from_bits(u64_value));
        }
        
        Ok(result)
    }
    
    /// 将f64数组转换为字节数组（IEEE 754）
    pub fn f64_array_to_bytes(values: &[f64], byte_order: ByteOrder) -> Vec<u8> {
        let mut result = Vec::new();
        for &value in values {
            let u64_value = value.to_bits();
            let bytes = match byte_order {
                ByteOrder::ABCD => u64_value.to_be_bytes(),
                ByteOrder::DCBA => u64_value.to_le_bytes(),
                ByteOrder::BADC => {
                    let bytes = u64_value.to_be_bytes();
                    [
                        bytes[1], bytes[0], bytes[3], bytes[2],
                        bytes[5], bytes[4], bytes[7], bytes[6]
                    ]
                },
                ByteOrder::CDAB => {
                    let bytes = u64_value.to_le_bytes();
                    [
                        bytes[1], bytes[0], bytes[3], bytes[2],
                        bytes[5], bytes[4], bytes[7], bytes[6]
                    ]
                },
            };
            result.extend_from_slice(&bytes);
        }
        result
    }
    
    /// 将线圈字节转换为布尔数组
    pub fn bytes_to_bool_array(bytes: &[u8], bit_count: usize) -> Vec<bool> {
        let mut result = Vec::new();
        
        for (byte_idx, &byte) in bytes.iter().enumerate() {
            for bit_idx in 0..8 {
                let global_bit_idx = byte_idx * 8 + bit_idx;
                if global_bit_idx >= bit_count {
                    break;
                }
                
                result.push((byte & (1 << bit_idx)) != 0);
            }
        }
        
        result
    }
    
    /// 将布尔数组转换为线圈字节
    pub fn bool_array_to_bytes(bools: &[bool]) -> Vec<u8> {
        let mut result = Vec::new();
        
        for chunk in bools.chunks(8) {
            let mut byte = 0u8;
            for (bit_idx, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << bit_idx;
                }
            }
            result.push(byte);
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::ByteOrder;
    
    #[test]
    fn test_bytes_to_u16_array() {
        let bytes = [0x12, 0x34, 0x56, 0x78];
        
        // ABCD (大端序)
        let result = DataConverter::bytes_to_u16_array(&bytes, ByteOrder::ABCD).unwrap();
        assert_eq!(result, vec![0x1234, 0x5678]);
        
        // DCBA (小端序)
        let result = DataConverter::bytes_to_u16_array(&bytes, ByteOrder::DCBA).unwrap();
        assert_eq!(result, vec![0x3412, 0x7856]);
    }
    
    #[test]
    fn test_u16_array_to_bytes() {
        let values = vec![0x1234, 0x5678];
        
        // ABCD (大端序)
        let result = DataConverter::u16_array_to_bytes(&values, ByteOrder::ABCD);
        assert_eq!(result, vec![0x12, 0x34, 0x56, 0x78]);
        
        // DCBA (小端序)
        let result = DataConverter::u16_array_to_bytes(&values, ByteOrder::DCBA);
        assert_eq!(result, vec![0x34, 0x12, 0x78, 0x56]);
    }
    
    #[test]
    fn test_bytes_to_f32_array() {
        let bytes = [0x40, 0x49, 0x0F, 0xDB]; // 3.14159 in IEEE 754
        
        let result = DataConverter::bytes_to_f32_array(&bytes, ByteOrder::ABCD).unwrap();
        assert!((result[0] - 3.14159).abs() < 0.00001);
    }
    
    #[test]
    fn test_bool_array_to_bytes() {
        let bools = vec![true, false, true, false, true, false, true, false];
        let result = DataConverter::bool_array_to_bytes(&bools);
        assert_eq!(result, vec![0b01010101]); // 从低位到高位：true, false, true, false, true, false, true, false
        
        let bools = vec![true, false, true, false, true, false, true, false, true];
        let result = DataConverter::bool_array_to_bytes(&bools);
        assert_eq!(result, vec![0b01010101, 0b00000001]);
    }
}
