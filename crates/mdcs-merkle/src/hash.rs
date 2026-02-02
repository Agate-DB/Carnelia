//! Content-addressed hashing for Merkle nodes.
//!
//! Uses SHA-256 to generate Content Identifiers (CIDs) for nodes.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fmt;

/// A 32-byte SHA-256 hash used as a Content Identifier (CID).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct Hash([u8; 32]);

impl Hash {
    /// Create a hash from raw bytes.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash(bytes)
    }

    /// Get the underlying bytes.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create a zero hash (used for genesis nodes).
    pub fn zero() -> Self {
        Hash([0u8; 32])
    }

    /// Check if this is the zero hash.
    pub fn is_zero(&self) -> bool {
        self.0 == [0u8; 32]
    }

    /// Convert to hex string for display.
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Create from hex string.
    pub fn from_hex(s: &str) -> Option<Self> {
        if s.len() != 64 {
            return None;
        }
        let mut bytes = [0u8; 32];
        for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
            let hex_str = std::str::from_utf8(chunk).ok()?;
            bytes[i] = u8::from_str_radix(hex_str, 16).ok()?;
        }
        Some(Hash(bytes))
    }

    /// Truncated display (first 8 chars).
    pub fn short(&self) -> String {
        self.to_hex()[..8].to_string()
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({}...)", &self.to_hex()[..8])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl Default for Hash {
    fn default() -> Self {
        Hash::zero()
    }
}

/// Hasher utility for computing content hashes.
pub struct Hasher {
    inner: Sha256,
}

impl Hasher {
    /// Create a new hasher.
    pub fn new() -> Self {
        Hasher {
            inner: Sha256::new(),
        }
    }

    /// Update the hasher with data.
    pub fn update(&mut self, data: &[u8]) {
        self.inner.update(data);
    }

    /// Finalize and return the hash.
    pub fn finalize(self) -> Hash {
        let result = self.inner.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Hash(bytes)
    }

    /// Hash data directly.
    pub fn hash(data: &[u8]) -> Hash {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize()
    }

    /// Hash multiple pieces of data.
    pub fn hash_all(parts: &[&[u8]]) -> Hash {
        let mut hasher = Self::new();
        for part in parts {
            hasher.update(part);
        }
        hasher.finalize()
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let data = b"hello world";
        let h1 = Hasher::hash(data);
        let h2 = Hasher::hash(data);
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_data() {
        let h1 = Hasher::hash(b"hello");
        let h2 = Hasher::hash(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hex_roundtrip() {
        let h1 = Hasher::hash(b"test data");
        let hex = h1.to_hex();
        let h2 = Hash::from_hex(&hex).unwrap();
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_zero_hash() {
        let zero = Hash::zero();
        assert!(zero.is_zero());
        assert!(!Hasher::hash(b"test").is_zero());
    }

    #[test]
    fn test_hash_all() {
        let h1 = Hasher::hash_all(&[b"hello", b"world"]);
        
        let mut hasher = Hasher::new();
        hasher.update(b"hello");
        hasher.update(b"world");
        let h2 = hasher.finalize();
        
        assert_eq!(h1, h2);
    }
}
