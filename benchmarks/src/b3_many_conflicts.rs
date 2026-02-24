//! B3 — Many conflicts.
//!
//! Simulate `C = floor(20 * sqrt(N))` concurrent clients. Each client makes one
//! mutation, then all merge pair-wise into a single sink replica. Measures time,
//! update sizes, doc sizes, encode/parse time, and memory.

use std::time::Instant;

use mdcs_core::lattice::Lattice;
use mdcs_db::json_crdt::{JsonCrdt, JsonPath, JsonValue};
use mdcs_db::rga_list::RGAList;
use rand::Rng;

use crate::harness::{
    allocated_bytes, log_result, measure_all_encodes, BenchmarkMetrics,
};

/// Compute number of concurrent clients: 20 * sqrt(N), floored, min 2.
fn num_clients(n: usize) -> usize {
    let c = (20.0 * (n as f64).sqrt()).floor() as usize;
    c.max(2)
}

// ─── B3.1  C clients concurrently set number in Map ─────────────────────────

pub fn b3_1_set_number_in_map(n: usize) {
    let c = num_clients(n);
    let mem_before = allocated_bytes();
    let start = Instant::now();

    // Each client creates a JsonCrdt and sets a key to a number
    let mut docs: Vec<JsonCrdt> = Vec::with_capacity(c);
    for i in 0..c {
        let replica_id = format!("client_{}", i);
        let mut doc = JsonCrdt::new(&replica_id);
        let path = JsonPath::parse(&format!("key_{}", i));
        let _ = doc.set(&path, JsonValue::Float(i as f64));
        docs.push(doc);
    }

    // Compute total update size
    let total_update_size: usize = docs
        .iter()
        .map(|d| serde_json::to_vec(d).map(|v| v.len()).unwrap_or(0))
        .sum();

    // Merge all into the first doc
    let merge_start = Instant::now();
    let mut sink = docs[0].clone();
    for doc in &docs[1..] {
        sink = sink.join(doc);
    }
    let _merge_time = merge_start.elapsed();

    let total_time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&sink);

    log_result(
        &format!("B3.1 {}*sqrt(N) clients concurrently set number in Map", 20),
        &BenchmarkMetrics {
            time: total_time,
            avg_update_size: total_update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B3.2  C clients concurrently set Object in Map ─────────────────────────

pub fn b3_2_set_object_in_map(n: usize) {
    let c = num_clients(n);
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut docs: Vec<JsonCrdt> = Vec::with_capacity(c);
    for i in 0..c {
        let replica_id = format!("client_{}", i);
        let mut doc = JsonCrdt::new(&replica_id);
        // Set a nested object: { key_i: { name: "client_i", value: i } }
        let base = format!("key_{}", i);
        let _ = doc.set(
            &JsonPath::parse(&format!("{}.name", base)),
            JsonValue::String(format!("client_{}", i)),
        );
        let _ = doc.set(
            &JsonPath::parse(&format!("{}.value", base)),
            JsonValue::Float(i as f64),
        );
        docs.push(doc);
    }

    let total_update_size: usize = docs
        .iter()
        .map(|d| serde_json::to_vec(d).map(|v| v.len()).unwrap_or(0))
        .sum();

    let mut sink = docs[0].clone();
    for doc in &docs[1..] {
        sink = sink.join(doc);
    }

    let total_time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&sink);

    log_result(
        &format!("B3.2 {}*sqrt(N) clients concurrently set Object in Map", 20),
        &BenchmarkMetrics {
            time: total_time,
            avg_update_size: total_update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B3.3  C clients concurrently set String in Map ─────────────────────────

pub fn b3_3_set_string_in_map(n: usize) {
    let c = num_clients(n);
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut docs: Vec<JsonCrdt> = Vec::with_capacity(c);
    for i in 0..c {
        let replica_id = format!("client_{}", i);
        let mut doc = JsonCrdt::new(&replica_id);
        // Set a large random string (1000 chars)
        let big_string: String = (0..1000)
            .map(|_| (b'a' + rng.gen_range(0..26)) as char)
            .collect();
        let _ = doc.set(
            &JsonPath::parse(&format!("key_{}", i)),
            JsonValue::String(big_string),
        );
        docs.push(doc);
    }

    let total_update_size: usize = docs
        .iter()
        .map(|d| serde_json::to_vec(d).map(|v| v.len()).unwrap_or(0))
        .sum();

    let mut sink = docs[0].clone();
    for doc in &docs[1..] {
        sink = sink.join(doc);
    }

    let total_time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&sink);

    log_result(
        &format!("B3.3 {}*sqrt(N) clients concurrently set String in Map", 20),
        &BenchmarkMetrics {
            time: total_time,
            avg_update_size: total_update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B3.4  C clients concurrently insert text in Array ──────────────────────

pub fn b3_4_insert_text_in_array(n: usize) {
    let c = num_clients(n);
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let words = &[
        "the", "quick", "brown", "fox", "jumps", "over", "lazy", "dog",
        "lorem", "ipsum",
    ];

    let mut lists: Vec<RGAList<String>> = Vec::with_capacity(c);
    for i in 0..c {
        let replica_id = format!("client_{}", i);
        let mut list = RGAList::<String>::new(&replica_id);
        // Insert a random 3-word string
        let text = format!(
            "{} {} {}",
            words[rng.gen_range(0..words.len())],
            words[rng.gen_range(0..words.len())],
            words[rng.gen_range(0..words.len())]
        );
        list.push_back(text);
        lists.push(list);
    }

    let total_update_size: usize = lists
        .iter()
        .map(|l| serde_json::to_vec(l).map(|v| v.len()).unwrap_or(0))
        .sum();

    // Merge all into the first list
    let mut sink = lists[0].clone();
    for list in &lists[1..] {
        sink = sink.join(list);
    }

    let total_time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&sink);

    log_result(
        &format!("B3.4 {}*sqrt(N) clients concurrently insert text in Array", 20),
        &BenchmarkMetrics {
            time: total_time,
            avg_update_size: total_update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── Run all B3 ─────────────────────────────────────────────────────────────

pub fn run_all(n: usize) {
    let c = num_clients(n);
    println!(
        "═══ B3: Many conflicts (N = {}, clients = {}) ═══\n",
        n, c
    );
    b3_1_set_number_in_map(n);
    b3_2_set_object_in_map(n);
    b3_3_set_string_in_map(n);
    b3_4_insert_text_in_array(n);
}
