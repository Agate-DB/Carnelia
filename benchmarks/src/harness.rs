//! Benchmark harness — shared measurement infrastructure.
//!
//! Provides a tracking global allocator for `memUsed` measurements,
//! a `BenchmarkMetrics` struct that mirrors the crdt-benchmarks output,
//! and helper functions for timing and reporting.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::time::{Duration, Instant};

use serde::Serialize;

// ── Tracking allocator ──────────────────────────────────────────────────────

/// A thin wrapper around the system allocator that tracks live heap bytes.
pub struct TrackingAllocator;

/// Atomic counter of currently-live heap bytes (alloc increments, dealloc decrements).
static ALLOCATED: AtomicIsize = AtomicIsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size() as isize, Ordering::Relaxed);
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc_zeroed(layout);
        if !ptr.is_null() {
            ALLOCATED.fetch_add(layout.size() as isize, Ordering::Relaxed);
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            let diff = new_size as isize - layout.size() as isize;
            ALLOCATED.fetch_add(diff, Ordering::Relaxed);
        }
        new_ptr
    }
}

/// Return current live-heap bytes according to the tracking allocator.
pub fn allocated_bytes() -> usize {
    let v = ALLOCATED.load(Ordering::Relaxed);
    if v < 0 {
        0
    } else {
        v as usize
    }
}

/// Take a snapshot of allocated bytes, run `f`, and return the delta.
#[allow(dead_code)]
pub fn measure_memory<F: FnOnce()>(f: F) -> usize {
    let before = allocated_bytes();
    f();
    let after = allocated_bytes();
    after.saturating_sub(before)
}

// ── Benchmark metrics ───────────────────────────────────────────────────────

/// All metrics collected for a single benchmark.
#[derive(Debug, Clone)]
pub struct BenchmarkMetrics {
    pub time: Duration,
    pub avg_update_size: usize,
    pub doc_size_json: usize,
    pub doc_size_bincode: usize,
    pub doc_size_postcard: usize,
    pub encode_time: Duration,
    pub parse_time: Duration,
    pub mem_used: usize,
}

impl Default for BenchmarkMetrics {
    fn default() -> Self {
        Self {
            time: Duration::ZERO,
            avg_update_size: 0,
            doc_size_json: 0,
            doc_size_bincode: 0,
            doc_size_postcard: 0,
            encode_time: Duration::ZERO,
            parse_time: Duration::ZERO,
            mem_used: 0,
        }
    }
}

// ── Encoding helpers (measure encode + decode) ──────────────────────────────

#[allow(dead_code)]
pub fn measure_encode_json<T: Serialize>(val: &T) -> (Vec<u8>, Duration) {
    let start = Instant::now();
    let bytes = serde_json::to_vec(val).expect("json encode");
    (bytes, start.elapsed())
}

pub fn measure_encode_bincode<T: Serialize>(val: &T) -> (Vec<u8>, Duration) {
    let start = Instant::now();
    let bytes = bincode::serialize(val).expect("bincode encode");
    (bytes, start.elapsed())
}

pub fn measure_encode_postcard<T: Serialize>(val: &T) -> (Vec<u8>, Duration) {
    let start = Instant::now();
    let bytes = postcard::to_allocvec(val).expect("postcard encode");
    (bytes, start.elapsed())
}

pub fn measure_decode_json<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> (T, Duration) {
    let start = Instant::now();
    let val: T = serde_json::from_slice(bytes).expect("json decode");
    (val, start.elapsed())
}

pub fn measure_decode_bincode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> (T, Duration) {
    let start = Instant::now();
    let val: T = bincode::deserialize(bytes).expect("bincode decode");
    (val, start.elapsed())
}

#[allow(dead_code)]
pub fn measure_decode_postcard<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> (T, Duration) {
    let start = Instant::now();
    let val: T = postcard::from_bytes(bytes).expect("postcard decode");
    (val, start.elapsed())
}

// ── Formatting helpers ──────────────────────────────────────────────────────

pub fn format_bytes(b: usize) -> String {
    if b == 0 {
        return "0 B".to_string();
    }
    const UNITS: &[&str] = &["B", "kB", "MB", "GB"];
    let mut val = b as f64;
    for unit in UNITS {
        if val < 1024.0 {
            if val == val.floor() {
                return format!("{} {}", val as usize, unit);
            }
            return format!("{:.1} {}", val, unit);
        }
        val /= 1024.0;
    }
    format!("{:.1} TB", val)
}

pub fn format_duration(d: Duration) -> String {
    let ms = d.as_millis();
    if ms == 0 {
        let us = d.as_micros();
        if us == 0 {
            return "0 ms".to_string();
        }
        return format!("{} µs", us);
    }
    format!("{} ms", ms)
}

// ── Logging ─────────────────────────────────────────────────────────────────

/// Log a single benchmark result in the same format as crdt-benchmarks.
pub fn log_result(test_name: &str, m: &BenchmarkMetrics) {
    println!(
        "[{}] (time)               {}",
        test_name,
        format_duration(m.time)
    );
    if m.avg_update_size > 0 {
        println!(
            "[{}] (avgUpdateSize)      {} bytes",
            test_name, m.avg_update_size
        );
    }
    println!(
        "[{}] (encodeTime)         {}",
        test_name,
        format_duration(m.encode_time)
    );
    if m.doc_size_json > 0 {
        println!(
            "[{}] (docSize:json)       {} bytes",
            test_name, m.doc_size_json
        );
    } else {
        println!("[{}] (docSize:json)       n/a (non-string keys)", test_name);
    }
    println!(
        "[{}] (docSize:bincode)    {} bytes",
        test_name, m.doc_size_bincode
    );
    println!(
        "[{}] (docSize:postcard)   {} bytes",
        test_name, m.doc_size_postcard
    );
    println!(
        "[{}] (parseTime)          {}",
        test_name,
        format_duration(m.parse_time)
    );
    println!(
        "[{}] (memUsed)            {}",
        test_name,
        format_bytes(m.mem_used)
    );
    println!();
}

/// Quick helper: encode with all three formats, measure sizes and times.
pub fn measure_all_encodes<T: Serialize + serde::de::DeserializeOwned>(
    val: &T,
) -> (usize, usize, usize, Duration, Duration) {
    // JSON may fail for types with non-string map keys (e.g., RGAList internals)
    let (json_bytes, t_json) = {
        let start = Instant::now();
        match serde_json::to_vec(val) {
            Ok(bytes) => (bytes, start.elapsed()),
            Err(_) => (Vec::new(), start.elapsed()),
        }
    };

    let (bc_bytes, t_bc) = measure_encode_bincode(val);
    let (pc_bytes, t_pc) = measure_encode_postcard(val);

    let encode_time = t_json + t_bc + t_pc;

    // Measure parse with bincode (always works) and JSON (if available)
    let (_, parse_bc) = measure_decode_bincode::<T>(&bc_bytes);
    let parse_json_dur = if !json_bytes.is_empty() {
        let (_, d) = measure_decode_json::<T>(&json_bytes);
        d
    } else {
        Duration::ZERO
    };
    let parse_time = parse_bc + parse_json_dur;

    (
        json_bytes.len(),
        bc_bytes.len(),
        pc_bytes.len(),
        encode_time,
        parse_time,
    )
}
