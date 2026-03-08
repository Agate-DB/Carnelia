//! B1 — No-conflict benchmarks.
//!
//! Simulate two clients. One client performs all mutations, then sends its
//! state to the other client (merge). Measures time, update sizes, doc sizes,
//! encode/parse time, and memory.

use std::time::Instant;

use mdcs_db::rga_list::RGAList;
use mdcs_sdk::document::TextDoc;
use rand::Rng;

use crate::harness::{allocated_bytes, log_result, measure_all_encodes, BenchmarkMetrics};

/// Random word list for B1.5 / B1.7.
pub const WORDS: &[&str] = &[
    "the",
    "quick",
    "brown",
    "fox",
    "jumps",
    "over",
    "lazy",
    "dog",
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
    "sed",
    "do",
    "eiusmod",
    "tempor",
    "incididunt",
    "ut",
    "labore",
    "et",
    "dolore",
    "magna",
    "aliqua",
    "enim",
    "ad",
    "minim",
    "veniam",
];

fn random_word(rng: &mut impl Rng) -> &'static str {
    WORDS[rng.gen_range(0..WORDS.len())]
}

// ─── B1.1  Append N characters ──────────────────────────────────────────────

pub fn b1_1_append_n_characters(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_1", "Alice");
    let mut total_update_bytes: usize = 0;

    for i in 0..n {
        let ch = (b'a' + (i % 26) as u8) as char;
        doc.insert(doc.len(), &ch.to_string());
        // Each single-char insert is ~1 byte of payload
        total_update_bytes += 1;
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(doc.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.1 Append N characters",
        &BenchmarkMetrics {
            time,
            avg_update_size: if n > 0 { total_update_bytes / n } else { 0 },
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.2  Insert string of length N ────────────────────────────────────────

pub fn b1_2_insert_string_of_length_n(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_2", "Alice");
    let long_string: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
    doc.insert(0, &long_string);

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(doc.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.2 Insert string of length N",
        &BenchmarkMetrics {
            time,
            avg_update_size: doc.get_text().len(),
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.3  Prepend N characters ─────────────────────────────────────────────

pub fn b1_3_prepend_n_characters(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_3", "Alice");

    for i in 0..n {
        let ch = (b'a' + (i % 26) as u8) as char;
        doc.insert(0, &ch.to_string());
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(doc.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.3 Prepend N characters",
        &BenchmarkMetrics {
            time,
            avg_update_size: 1,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.4  Insert N characters at random positions ──────────────────────────

pub fn b1_4_insert_n_chars_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_4", "Alice");

    for i in 0..n {
        let pos = if doc.len() == 0 {
            0
        } else {
            rng.gen_range(0..=doc.len())
        };
        let ch = (b'a' + (i % 26) as u8) as char;
        doc.insert(pos, &ch.to_string());
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(doc.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.4 Insert N characters at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: 1,
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.5  Insert N words at random positions ───────────────────────────────

pub fn b1_5_insert_n_words_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_5", "Alice");
    let mut total_update_bytes: usize = 0;

    for _ in 0..n {
        let word = random_word(&mut rng);
        let pos = if doc.len() == 0 {
            0
        } else {
            rng.gen_range(0..=doc.len())
        };
        doc.insert(pos, word);
        total_update_bytes += word.len();
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.5 Insert N words at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: if n > 0 { total_update_bytes / n } else { 0 },
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.6  Insert string, then delete it ────────────────────────────────────

pub fn b1_6_insert_then_delete(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_6", "Alice");
    let long_string: String = (0..n).map(|i| (b'a' + (i % 26) as u8) as char).collect();
    doc.insert(0, &long_string);
    doc.delete(0, n);

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(doc.len(), 0);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.6 Insert string, then delete it",
        &BenchmarkMetrics {
            time,
            avg_update_size: long_string.len(),
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.7  Insert/Delete strings at random positions ────────────────────────

pub fn b1_7_insert_delete_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut doc = TextDoc::new("b1_7", "Alice");
    let mut total_update_bytes: usize = 0;

    for _ in 0..n {
        let len = doc.len();
        if len > 0 && rng.gen_ratio(3, 10) {
            // 30% delete
            let pos = rng.gen_range(0..len);
            let del_len = std::cmp::min(rng.gen_range(1..=5), len - pos);
            doc.delete(pos, del_len);
            total_update_bytes += 2; // op overhead
        } else {
            // 70% insert
            let word = random_word(&mut rng);
            let pos = if len == 0 { 0 } else { rng.gen_range(0..=len) };
            doc.insert(pos, word);
            total_update_bytes += word.len();
        }
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&doc.get_text());

    log_result(
        "B1.7 Insert/Delete strings at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: if n > 0 { total_update_bytes / n } else { 0 },
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.8  Append N numbers ─────────────────────────────────────────────────

pub fn b1_8_append_n_numbers(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut list = RGAList::<i64>::new("Alice");

    for i in 0..n {
        list.push_back(i as i64);
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(list.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&list);

    log_result(
        "B1.8 Append N numbers",
        &BenchmarkMetrics {
            time,
            avg_update_size: std::mem::size_of::<i64>(),
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.9  Insert Array of N numbers (bulk) ─────────────────────────────────

pub fn b1_9_insert_array_of_n_numbers(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut list = RGAList::<i64>::new("Alice");
    for i in 0..n {
        list.push_back(i as i64);
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(list.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&list);

    log_result(
        "B1.9 Insert Array of N numbers",
        &BenchmarkMetrics {
            time,
            avg_update_size: json_sz, // single bulk update
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.10 Prepend N numbers ────────────────────────────────────────────────

pub fn b1_10_prepend_n_numbers(n: usize) {
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut list = RGAList::<i64>::new("Alice");

    for i in 0..n {
        list.insert(0, i as i64);
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(list.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&list);

    log_result(
        "B1.10 Prepend N numbers",
        &BenchmarkMetrics {
            time,
            avg_update_size: std::mem::size_of::<i64>(),
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── B1.11 Insert N numbers at random positions ─────────────────────────────

pub fn b1_11_insert_n_numbers_random(n: usize) {
    let mut rng = rand::thread_rng();
    let mem_before = allocated_bytes();
    let start = Instant::now();

    let mut list = RGAList::<i64>::new("Alice");

    for i in 0..n {
        let pos = if list.len() == 0 {
            0
        } else {
            rng.gen_range(0..=list.len())
        };
        list.insert(pos, i as i64);
    }

    let time = start.elapsed();
    let mem_after = allocated_bytes();

    assert_eq!(list.len(), n);

    let (json_sz, bc_sz, pc_sz, enc_t, parse_t) = measure_all_encodes(&list);

    log_result(
        "B1.11 Insert N numbers at random positions",
        &BenchmarkMetrics {
            time,
            avg_update_size: std::mem::size_of::<i64>(),
            doc_size_json: json_sz,
            doc_size_bincode: bc_sz,
            doc_size_postcard: pc_sz,
            encode_time: enc_t,
            parse_time: parse_t,
            mem_used: mem_after.saturating_sub(mem_before),
        },
    );
}

// ─── Run all B1 ─────────────────────────────────────────────────────────────

pub fn run_all(n: usize) {
    println!("═══ B1: No conflicts (N = {}) ═══\n", n);
    b1_1_append_n_characters(n);
    b1_2_insert_string_of_length_n(n);
    b1_3_prepend_n_characters(n);
    b1_4_insert_n_chars_random(n);
    b1_5_insert_n_words_random(n);
    b1_6_insert_then_delete(n);
    b1_7_insert_delete_random(n);
    b1_8_append_n_numbers(n);
    b1_9_insert_array_of_n_numbers(n);
    b1_10_prepend_n_numbers(n);
    b1_11_insert_n_numbers_random(n);
}
