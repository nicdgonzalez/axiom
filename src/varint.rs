//! Encode and decode Minecraft's VarInt.

/// A mask that represent the value of the byte.
const SEGMENT_BITS: u8 = 0x7F;
/// A mask that indicates if there are more bytes to read.
const CONTINUE_BIT: u8 = 0x80;

/// Describes an error
#[derive(Debug)]
pub enum ReadVarIntError {
    VarIntTooLarge,
    Io(std::io::Error),
}

impl std::fmt::Display for ReadVarIntError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::VarIntTooLarge => write!(f, "VarInt exceeds the size limit (32 bits)"),
            Self::Io(inner) => write!(f, "{inner}"),
        }
    }
}

impl std::error::Error for ReadVarIntError {}

impl From<std::io::Error> for ReadVarIntError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub trait WriteExt: std::io::Write {
    fn write_varint_i32(&mut self, value: i32) -> Result<(), std::io::Error> {
        let mut buffer = Vec::new();
        let mut value = value as u32;

        while value > u32::from(SEGMENT_BITS) {
            buffer.push((value as u8) & SEGMENT_BITS | CONTINUE_BIT);
            value >>= 7;
        }

        debug_assert!((value as u8) & CONTINUE_BIT == 0);
        buffer.push(value as u8);
        debug_assert!(buffer.len() > 0 && buffer.len() <= 5);

        self.write_all(&mut buffer)
    }
}

pub trait ReadExt: std::io::Read {
    fn read_varint_i32(&mut self) -> Result<i32, ReadVarIntError> {
        let mut value = 0u32;
        let mut position = 0;

        loop {
            let byte = {
                let mut buffer = [0u8; 1];
                self.read_exact(&mut buffer)?;
                buffer[0]
            };

            value |= ((byte & SEGMENT_BITS) as u32) << position;
            position += 7;

            if byte & CONTINUE_BIT == 0 {
                break;
            }

            if position >= 32 {
                return Err(ReadVarIntError::VarIntTooLarge);
            }
        }

        Ok(value as i32)
    }
}

impl<W: std::io::Write> WriteExt for W {}
impl<R: std::io::Read> ReadExt for R {}

pub fn encode(value: i32) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.write_varint_i32(value).unwrap();
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write() {
        let input = [
            (0, vec![0x00]),
            (1, vec![0x01]),
            (2, vec![0x02]),
            (127, vec![0x7F]),
            (128, vec![0x80, 0x01]),
            (255, vec![0xFF, 0x01]),
            (25565, vec![0xDD, 0xC7, 0x01]),
            (2097151, vec![0xFF, 0xFF, 0x7F]),
            (2147483647, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07]),
            (-1, vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
            (-2147483648, vec![0x80, 0x80, 0x80, 0x80, 0x08]),
        ];

        for (value, data) in input.into_iter() {
            let mut buffer = Vec::with_capacity(data.len());
            assert!(buffer.write_varint_i32(value).is_ok());
            assert_eq!(buffer, data);
        }
    }

    #[test]
    fn test_read() {
        let input: [(Vec<u8>, i32); 11] = [
            (vec![0x00], 0),
            (vec![0x01], 1),
            (vec![0x02], 2),
            (vec![0x7F], 127),
            (vec![0x80, 0x01], 128),
            (vec![0xFF, 0x01], 255),
            (vec![0xDD, 0xC7, 0x01], 25565),
            (vec![0xFF, 0xFF, 0x7F], 2097151),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x07], 2147483647),
            (vec![0xFF, 0xFF, 0xFF, 0xFF, 0x0F], -1),
            (vec![0x80, 0x80, 0x80, 0x80, 0x08], -2147483648),
        ];

        for (data, value) in input.into_iter() {
            let mut reader = std::io::Cursor::new(&data);
            assert_eq!(reader.read_varint_i32().unwrap(), value);
        }
    }
}
