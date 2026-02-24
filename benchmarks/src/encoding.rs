//! Encoding helpers — thin wrappers around serde_json, bincode, and postcard.

use serde::{de::DeserializeOwned, Serialize};

// ── JSON ───────────────────────────────────────────────────────────────────

pub fn encode_json<T: Serialize>(val: &T) -> Vec<u8> {
    serde_json::to_vec(val).expect("json encode failed")
}

pub fn decode_json<T: DeserializeOwned>(bytes: &[u8]) -> T {
    serde_json::from_slice(bytes).expect("json decode failed")
}

// ── Bincode ────────────────────────────────────────────────────────────────

pub fn encode_bincode<T: Serialize>(val: &T) -> Vec<u8> {
    bincode::serialize(val).expect("bincode encode failed")
}

pub fn decode_bincode<T: DeserializeOwned>(bytes: &[u8]) -> T {
    bincode::deserialize(bytes).expect("bincode decode failed")
}

// ── Postcard ───────────────────────────────────────────────────────────────

pub fn encode_postcard<T: Serialize>(val: &T) -> Vec<u8> {
    postcard::to_allocvec(val).expect("postcard encode failed")
}

pub fn decode_postcard<T: DeserializeOwned>(bytes: &[u8]) -> T {
    postcard::from_bytes(bytes).expect("postcard decode failed")
}
