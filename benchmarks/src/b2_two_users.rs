//! B2 — Two users producing conflicts.
//!
//! Simulate two clients. Both start with a synced text object containing 100
//! characters. Both modify the text in a single transaction, then send their
//! changes to the other client. We measure the time to sync concurrent changes
//! into a single client, update sizes, doc sizes, encode/parse time, and memory.

use std::time::Instant;

use mdcs_sdk::document::TextDoc;
use rand::Rng;

use crate::b1_no_conflicts::WORDS;
use crate::harness::{
    allocated_bytes, log_result, measure_all_encodes, BenchmarkMetrics,
};

fn random_word(rng: &mut impl Rng) -> &'static str {
    WORDS[rng.gen_range(0..WORDS.len())]
}

/// Create two TextDocs pre-filled with `init_len` characters, simulating
/// a synced starting state.
fn make_synced_pair(id: &str, init_len: usize) -> (TextDoc, TextDoc) {
    let init_text: String = (0..init_len)
        .map(|i| (b'a' + (i % 26) as u8) as char)
        .collect();
    let mut doc_a = TextDoc::new(id, "Alice");
    doc_a.insert(0, &init_text);

    let mut doc_b = TextDoc::new(id, "Bob");
    doc_b.merge(&doc_a); // sync

    (doc_a, doc_b)
}

// ─── B2.1  Concurrently insert string of length N at index 0 ───────────────

pub fn b2_1_concurrent_insert_string(n: usize) {
    let mem_before = allocated_bytes();

    let (mut doc_a, mut doc_b) = make_synced_pair("b2_1", 100);

    let str_a: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
    let str_b: String = (0..n).map(|i| (b'A' + (i % 26) as u8) as char).collect();

    doc_a.insert(0, &str_a);
    doc_b.insert(0, &str_b);

    // Measure merge time
    let update_size = doc_a.get_text().len() + doc_b.get_text().len();

    let start = Instant::now();
    doc_a.merge(&doc_b);
    let time = start.elapsed();

    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc_a.get_text());

    log_result(
        "B2.1 Concurrently insert string of length N at index 0",
        &BenchmarkMetrics {
            time,
            avg_update_size: update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B2.2  Concurrently insert N characters at random positions ─────────────

pub fn b2_2_concurrent_insert_chars_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();

    let (mut doc_a, mut doc_b) = make_synced_pair("b2_2", 100);

    for i in 0..n {
        let ch = (b'a' + (i % 26) as u8) as char;
        let pos_a = rng.gen_range(0..=doc_a.len());
        doc_a.insert(pos_a, &ch.to_string());

        let ch_b = (b'A' + (i % 26) as u8) as char;
        let pos_b = rng.gen_range(0..=doc_b.len());
        doc_b.insert(pos_b, &ch_b.to_string());
    }

    let update_size = doc_a.get_text().len() + doc_b.get_text().len();

    let start = Instant::now();
    doc_a.merge(&doc_b);
    let time = start.elapsed();

    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc_a.get_text());

    log_result(
        "B2.2 Concurrently insert N characters at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B2.3  Concurrently insert N words at random positions ──────────────────

pub fn b2_3_concurrent_insert_words_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();

    let (mut doc_a, mut doc_b) = make_synced_pair("b2_3", 100);

    for _ in 0..n {
        let word_a = random_word(&mut rng);
        let pos_a = rng.gen_range(0..=doc_a.len());
        doc_a.insert(pos_a, word_a);

        let word_b = random_word(&mut rng);
        let pos_b = rng.gen_range(0..=doc_b.len());
        doc_b.insert(pos_b, word_b);
    }

    let update_size = doc_a.get_text().len() + doc_b.get_text().len();

    let start = Instant::now();
    doc_a.merge(&doc_b);
    let time = start.elapsed();

    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc_a.get_text());

    log_result(
        "B2.3 Concurrently insert N words at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B2.4  Concurrently insert & delete ─────────────────────────────────────

pub fn b2_4_concurrent_insert_delete(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();

    let (mut doc_a, mut doc_b) = make_synced_pair("b2_4", 100);

    for _ in 0..n {
        // Client A
        let len_a = doc_a.len();
        if len_a > 1 && rng.gen_ratio(3, 10) {
            let pos = rng.gen_range(0..len_a);
            let del = std::cmp::min(rng.gen_range(1..=5), len_a - pos);
            doc_a.delete(pos, del);
        } else {
            let word = random_word(&mut rng);
            let pos = if len_a == 0 { 0 } else { rng.gen_range(0..=len_a) };
            doc_a.insert(pos, word);
        }

        // Client B
        let len_b = doc_b.len();
        if len_b > 1 && rng.gen_ratio(3, 10) {
            let pos = rng.gen_range(0..len_b);
            let del = std::cmp::min(rng.gen_range(1..=5), len_b - pos);
            doc_b.delete(pos, del);
        } else {
            let word = random_word(&mut rng);
            let pos = if len_b == 0 { 0 } else { rng.gen_range(0..=len_b) };
            doc_b.insert(pos, word);
        }
    }

    let update_size = doc_a.get_text().len() + doc_b.get_text().len();

    let start = Instant::now();
    doc_a.merge(&doc_b);
    let time = start.elapsed();

    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc_a.get_text());

    log_result(
        "B2.4 Concurrently insert & delete",
        &BenchmarkMetrics {
            time,
            avg_update_size: update_size,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── Run all B2 ─────────────────────────────────────────────────────────────

pub fn run_all(n: usize) {
    println!("═══ B2: Two users producing conflicts (N = {}) ═══\n", n);
    b2_1_concurrent_insert_string(n);
    b2_2_concurrent_insert_chars_random(n);
    b2_3_concurrent_insert_words_random(n);
    b2_4_concurrent_insert_delete(n);
}
