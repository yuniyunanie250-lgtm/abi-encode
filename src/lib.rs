//! Static ABI encoding: turn typed values into 32-byte words.
//!
//! Every static ABI argument occupies exactly 32 bytes:
//!   uint<M>/int<M>  right-aligned, ints two's complement
//!   address         right-aligned, low 20 bytes
//!   bool            0 or 1, right-aligned
//!   bytes<M>        left-aligned, right-padded with zeros
//!
//! Dynamic types are rejected rather than mis-encoded: their layout depends on
//! every other argument in the call, so encoding one in isolation produces a
//! payload that is silently wrong for the real function.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    UnsupportedType(String),
    WrongWidth(u32),
    BadHex(String),
    AddressTooLong(usize),
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncodeError::UnsupportedType(t) => write!(f, "unsupported type: {}", t),
            EncodeError::WrongWidth(w) => write!(f, "bad width: {}", w),
            EncodeError::BadHex(s) => write!(f, "bad hex: {}", s),
            EncodeError::AddressTooLong(n) => write!(f, "address is {} bytes, max 20", n),
        }
    }
}

impl std::error::Error for EncodeError {}

fn hex_to_bytes(s: &str) -> Result<Vec<u8>, EncodeError> {
    let t = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    if t.len() % 2 != 0 {
        return Err(EncodeError::BadHex(format!("odd length: {}", t.len())));
    }
    (0..t.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&t[i..i + 2], 16).map_err(|_| EncodeError::BadHex(t.to_string()))
        })
        .collect()
}

/// Encode a static `uint<M>` from a decimal string.
pub fn uint(ty: &str, value: &str) -> Result<[u8; 32], EncodeError> {
    let width = parse_width(ty, "uint")?;
    let mut limbs = [0u64; 4]; // little-endian 256-bit accumulator
    for ch in value.trim().chars() {
        let d = ch
            .to_digit(10)
            .ok_or_else(|| EncodeError::BadHex(value.to_string()))? as u64;
        let mut carry = d as u128;
        for limb in limbs.iter_mut() {
            let cur = (*limb as u128) * 10 + carry;
            *limb = cur as u64;
            carry = cur >> 64;
        }
        if carry != 0 {
            return Err(EncodeError::BadHex(format!("{} overflows 256 bits", value)));
        }
    }
    if let Some(bits) = width {
        if bits < 256 {
            let limit = 1u128 << bits;
            let upper_used = limbs[1] | limbs[2] | limbs[3];
            if upper_used != 0 || limbs[0] as u128 >= limit {
                return Err(EncodeError::WrongWidth(bits));
            }
        }
    }
    let mut out = [0u8; 32];
    for i in 0..4 {
        // limbs are little-endian, so limb i occupies the i-th 8 bytes from the
        // right: limb 0 is the least significant and goes in the LAST slot
        out[24 - i * 8..32 - i * 8].copy_from_slice(&limbs[i].to_be_bytes());
    }
    Ok(out)
}

fn parse_width(ty: &str, prefix: &str) -> Result<Option<u32>, EncodeError> {
    let rest = ty
        .strip_prefix(prefix)
        .ok_or_else(|| EncodeError::UnsupportedType(ty.into()))?;
    if rest.is_empty() {
        return Ok(None);
    }
    let bits: u32 = rest
        .parse()
        .map_err(|_| EncodeError::UnsupportedType(ty.into()))?;
    if bits % 8 != 0 || bits == 0 || bits > 256 {
        return Err(EncodeError::WrongWidth(bits));
    }
    Ok(Some(bits))
}

/// Encode an address (20 bytes max) into the low 20 bytes of a word.
pub fn address(addr: &str) -> Result<[u8; 32], EncodeError> {
    let bytes = hex_to_bytes(addr)?;
    if bytes.len() > 20 {
        return Err(EncodeError::AddressTooLong(bytes.len()));
    }
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    Ok(out)
}

pub fn boolean(v: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = v as u8;
    out
}

/// Encode `bytes<M>`: left-aligned, right-padded. In Solidity `bytesM` means M
/// **bytes**, not M bits, so `bytes4` is 4 bytes wide.
pub fn fixed_bytes(ty: &str, data: &str) -> Result<[u8; 32], EncodeError> {
    let size: u32 = ty
        .strip_prefix("bytes")
        .and_then(|r| r.parse().ok())
        .ok_or_else(|| EncodeError::UnsupportedType(ty.into()))?;
    if size == 0 || size > 32 {
        return Err(EncodeError::WrongWidth(size));
    }
    let expected = size as usize;
    let bytes = hex_to_bytes(data)?;
    if bytes.len() != expected {
        return Err(EncodeError::WrongWidth(size));
    }
    let mut out = [0u8; 32];
    out[..bytes.len()].copy_from_slice(&bytes);
    Ok(out)
}

/// Encode `(type, value)` pairs. Rejects dynamic types explicitly.
pub fn encode(pairs: &[(&str, &str)]) -> Result<Vec<u8>, EncodeError> {
    let mut out = Vec::with_capacity(pairs.len() * 32);
    for (ty, value) in pairs {
        let word = match *ty {
            "address" => address(value)?,
            "bool" => boolean(matches!(value.trim(), "true" | "1")),
            t if t.starts_with("uint") => uint(t, value)?,
            t if t.starts_with("bytes") && t != "bytes" => fixed_bytes(t, value)?,
            t => return Err(EncodeError::UnsupportedType(t.to_string())),
        };
        out.extend_from_slice(&word);
    }
    Ok(out)
}

pub fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::from("0x");
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
