//! DXIL -> SPIR-V for the Vulkan backend.
//!
//! The translator emits a DXIL container; Vulkan needs SPIR-V. This module
//! wraps the `dxil-spirv` bindings so the rest of the workspace does not
//! depend on its API.

/// Converts a DXIL/DXBC shader blob to SPIR-V words.
///
/// `blob` may be a full DXBC container (SM4/SM5/SM6) or a raw DXIL bitcode
/// slice.
pub fn convert(blob: &[u8]) -> Result<Vec<u32>, String> {
    dxil_spirv::convert_to_spirv(blob).map_err(|e| e.to_string())
}

/// Converts a DXIL/DXBC shader blob to little-endian SPIR-V bytes.
pub fn convert_to_bytes(blob: &[u8]) -> Result<Vec<u8>, String> {
    Ok(words_to_bytes(&convert(blob)?))
}

/// Little-endian bytes of SPIR-V words.
pub fn words_to_bytes(words: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * 4);
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_is_an_error() {
        assert!(convert(&[]).is_err());
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(convert(b"not a shader").is_err());
    }

    #[test]
    fn words_round_trip_as_little_endian() {
        assert_eq!(words_to_bytes(&[0x0723_0203, 0x0001_0000]), vec![
            0x03, 0x02, 0x23, 0x07, 0x00, 0x00, 0x01, 0x00,
        ]);
    }
}
