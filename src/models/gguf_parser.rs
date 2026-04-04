use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

const GGUF_MAGIC: u32 = 0x46554747; // "GGUF" in little-endian

#[derive(Debug)]
pub enum GgufValue {
    U8(u8),
    I8(i8),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    F32(f32),
    Bool(bool),
    Str(String),
    U64(u64),
    I64(i64),
    F64(f64),
}

#[derive(Debug)]
pub struct GgufMetadata {
    pub entries: HashMap<String, GgufValue>,
    pub tensor_count: u64,
}

impl GgufMetadata {
    pub fn get_str(&self, key: &str) -> Option<&str> {
        match self.entries.get(key) {
            Some(GgufValue::Str(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn get_u32(&self, key: &str) -> Option<u32> {
        match self.entries.get(key) {
            Some(GgufValue::U32(v)) => Some(*v),
            Some(GgufValue::I32(v)) => Some(*v as u32),
            Some(GgufValue::U64(v)) => Some(*v as u32),
            Some(GgufValue::I64(v)) => Some(*v as u32),
            _ => None,
        }
    }

    pub fn get_u64(&self, key: &str) -> Option<u64> {
        match self.entries.get(key) {
            Some(GgufValue::U64(v)) => Some(*v),
            Some(GgufValue::I64(v)) => Some(*v as u64),
            Some(GgufValue::U32(v)) => Some(*v as u64),
            Some(GgufValue::I32(v)) => Some(*v as u64),
            _ => None,
        }
    }

    pub fn get_f32(&self, key: &str) -> Option<f32> {
        match self.entries.get(key) {
            Some(GgufValue::F32(v)) => Some(*v),
            _ => None,
        }
    }
}

/// Parse GGUF metadata from the header of a file. Only reads the metadata
/// section (first few KB), never touches multi-GB weight data.
pub fn parse_gguf_metadata(path: &Path) -> Result<GgufMetadata> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open GGUF file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    // Validate magic bytes
    let magic = read_u32(&mut reader)?;
    if magic != GGUF_MAGIC {
        bail!(
            "Not a GGUF file (magic: 0x{:08X}, expected 0x{:08X})",
            magic,
            GGUF_MAGIC
        );
    }

    // Read version
    let version = read_u32(&mut reader)?;
    if version < 2 || version > 3 {
        bail!("Unsupported GGUF version: {}", version);
    }

    // Read counts
    let tensor_count = read_u64(&mut reader)?;
    let metadata_kv_count = read_u64(&mut reader)?;

    // Sanity check — metadata shouldn't have millions of keys
    if metadata_kv_count > 10_000 {
        bail!("Suspicious metadata count: {}", metadata_kv_count);
    }

    let mut entries = HashMap::with_capacity(metadata_kv_count as usize);

    for _ in 0..metadata_kv_count {
        let key = read_gguf_string(&mut reader)?;
        let value_type = read_u32(&mut reader)?;

        if value_type == 9 {
            // Array type — skip efficiently (tokenizer data, etc.)
            skip_array(&mut reader)?;
            continue;
        }

        match read_value(&mut reader, value_type) {
            Ok(value) => {
                entries.insert(key, value);
            }
            Err(_) => {
                // If we can't parse a value, stop — we've likely hit corruption
                // or an unsupported format. Return what we have so far.
                break;
            }
        }
    }

    Ok(GgufMetadata {
        entries,
        tensor_count,
    })
}

fn read_value<R: Read + Seek>(reader: &mut R, value_type: u32) -> Result<GgufValue> {
    match value_type {
        0 => Ok(GgufValue::U8(read_u8(reader)?)),
        1 => Ok(GgufValue::I8(read_u8(reader)? as i8)),
        2 => Ok(GgufValue::U16(read_u16(reader)?)),
        3 => Ok(GgufValue::I16(read_u16(reader)? as i16)),
        4 => Ok(GgufValue::U32(read_u32(reader)?)),
        5 => Ok(GgufValue::I32(read_u32(reader)? as i32)),
        6 => Ok(GgufValue::F32(read_f32(reader)?)),
        7 => Ok(GgufValue::Bool(read_u8(reader)? != 0)),
        8 => Ok(GgufValue::Str(read_gguf_string(reader)?)),
        // 9 = array, handled separately
        10 => Ok(GgufValue::U64(read_u64(reader)?)),
        11 => Ok(GgufValue::I64(read_u64(reader)? as i64)),
        12 => Ok(GgufValue::F64(read_f64(reader)?)),
        _ => bail!("Unknown GGUF value type: {}", value_type),
    }
}

/// Skip an array without reading all elements. Arrays can have 100K+ entries
/// (tokenizer data), so we seek past them efficiently.
fn skip_array<R: Read + Seek>(reader: &mut R) -> Result<()> {
    let element_type = read_u32(reader)?;
    let count = read_u64(reader)?;

    // Calculate bytes to skip based on element type
    let element_size: Option<u64> = match element_type {
        0 | 1 | 7 => Some(1),   // u8, i8, bool
        2 | 3 => Some(2),       // u16, i16
        4 | 5 | 6 => Some(4),   // u32, i32, f32
        10 | 11 | 12 => Some(8), // u64, i64, f64
        _ => None,               // strings/arrays — variable size
    };

    if let Some(size) = element_size {
        reader.seek(SeekFrom::Current((count * size) as i64))?;
    } else {
        // Variable-size elements (strings, nested arrays) — read through them
        for _ in 0..count {
            if element_type == 8 {
                read_gguf_string(reader)?;
            } else if element_type == 9 {
                skip_array(reader)?;
            } else {
                bail!("Cannot skip array of type {}", element_type);
            }
        }
    }

    Ok(())
}

// --- Low-level readers ---

fn read_u8<R: Read>(reader: &mut R) -> Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16<R: Read>(reader: &mut R) -> Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64<R: Read>(reader: &mut R) -> Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_f32<R: Read>(reader: &mut R) -> Result<f32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f64<R: Read>(reader: &mut R) -> Result<f64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

fn read_gguf_string<R: Read>(reader: &mut R) -> Result<String> {
    let len = read_u64(reader)? as usize;
    if len > 1_000_000 {
        bail!("GGUF string too long: {} bytes", len);
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}
