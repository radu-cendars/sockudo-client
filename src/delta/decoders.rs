//! Delta compression decoders.

use crate::error::{Result, SockudoError};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

/// Trait for delta decoders
pub trait DeltaDecoder: Send + Sync {
    /// Decode a delta against a base message
    fn decode(&self, base: &[u8], delta: &[u8]) -> Result<Vec<u8>>;

    /// Get the algorithm name
    fn algorithm(&self) -> &'static str;

    /// Check if the decoder is available
    fn is_available(&self) -> bool {
        true
    }
}

/// Fossil Delta decoder
///
/// Uses the fossil-delta crate for efficient binary delta decoding.
#[derive(Debug, Clone, Default)]
pub struct FossilDeltaDecoder {
    _private: (),
}

impl FossilDeltaDecoder {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl DeltaDecoder for FossilDeltaDecoder {
    fn decode(&self, base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
        // fossil-delta has inverted semantics: deltainv(base, delta) applies the delta forward
        Ok(fossil_delta::deltainv(base, delta))
    }

    fn algorithm(&self) -> &'static str {
        "fossil"
    }
}

/// Xdelta3/VCDIFF decoder
#[derive(Debug, Clone, Default)]
pub struct Xdelta3Decoder {
    _private: (),
}

impl Xdelta3Decoder {
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl DeltaDecoder for Xdelta3Decoder {
    fn decode(&self, base: &[u8], delta: &[u8]) -> Result<Vec<u8>> {
        // Use vcdiff-decoder for VCDIFF/xdelta3 decoding (works with WASM)
        vcdiff_decoder::decode(delta, base)
            .map_err(|e| SockudoError::delta(format!("VCDIFF decode failed: {}", e)))
    }

    fn algorithm(&self) -> &'static str {
        "xdelta3"
    }

    fn is_available(&self) -> bool {
        true
    }
}

/// Utility functions for base64 encoding/decoding
pub fn decode_base64(input: &str) -> Result<Vec<u8>> {
    BASE64
        .decode(input)
        .map_err(|e| SockudoError::delta(format!("Base64 decode error: {}", e)))
}

pub fn encode_base64(input: &[u8]) -> String {
    BASE64.encode(input)
}

/// Get a decoder for the specified algorithm
pub fn get_decoder(algorithm: &str) -> Option<Box<dyn DeltaDecoder>> {
    match algorithm.to_lowercase().as_str() {
        "fossil" => Some(Box::new(FossilDeltaDecoder::new())),
        "xdelta3" | "vcdiff" => {
            let decoder = Xdelta3Decoder::new();
            if decoder.is_available() {
                Some(Box::new(decoder))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fossil_delta_roundtrip() {
        let base = b"Hello, World!";
        let target = b"Hello, Rust World!";

        // Note: fossil-delta has inverted semantics
        // delta(target, base) creates a delta that when applied to base gives target
        // So we need to create the delta from target to base
        let delta = fossil_delta::delta(target, base);

        // Decode the delta - this should transform base -> target
        let decoder = FossilDeltaDecoder::new();
        let result = decoder.decode(base, &delta).unwrap();

        assert_eq!(result, target);
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = b"Test data for encoding";
        let encoded = encode_base64(original);
        let decoded = decode_base64(&encoded).unwrap();

        assert_eq!(decoded, original);
    }

    // xdelta3 encoder is only available for non-wasm targets (dev-dependency)
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_xdelta3_decoder() {
        let base = b"Hello, World!";
        let target = b"Hello, Rust World!";

        // Create a delta from base to target
        // xdelta3::encode(input, src) where input=target (new data), src=base (original)
        let delta = xdelta3::encode(target, base).unwrap();

        // Decode the delta - this should transform base -> target
        let decoder = Xdelta3Decoder::new();
        let result = decoder.decode(base, &delta).unwrap();

        assert_eq!(result, target);
    }

    #[test]
    fn test_xdelta3_decoder_availability() {
        let decoder = Xdelta3Decoder::new();
        assert!(decoder.is_available());
    }
}
