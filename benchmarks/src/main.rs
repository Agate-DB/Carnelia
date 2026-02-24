//! MDCS CRDT Benchmarks
//!
//! Port of [dmonad/crdt-benchmarks](https://github.com/dmonad/crdt-benchmarks)
//! targeting the MDCS stack. Runs B1–B3 benchmark suites and logs per-test
//! metrics matching the crdt-benchmarks output format.
//!
//! Usage:
//!   cargo run -p mdcs-benchmarks                     # all benchmarks, N=6000
//!   cargo run -p mdcs-benchmarks -- --n 1000         # smaller N
//!   cargo run -p mdcs-benchmarks -- --only b1        # only B1 group
//!   cargo run -p mdcs-benchmarks -- --only b1.1      # single sub-benchmark

mod b1_no_conflicts;
mod b2_two_users;
mod b3_many_conflicts;
#[allow(dead_code)]
mod encoding;
mod harness;

use harness::TrackingAllocator;

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Parse --n <value>
    let n: usize = args
        .iter()
        .position(|a| a == "--n")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(6000);

    // Parse --only <filter>
    let only: Option<&str> = args
        .iter()
        .position(|a| a == "--only")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.as_str());

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║             MDCS CRDT Benchmarks                       ║");
    println!("║  Port of dmonad/crdt-benchmarks                        ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  N = {:<6}                                             ║", n);
    if let Some(f) = only {
        println!("║  Filter: {:<48}║", f);
    }
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let start = std::time::Instant::now();

    match only {
        Some(f) => run_filtered(f, n),
        None => run_all(n),
    }

    let total = start.elapsed();
    println!("════════════════════════════════════════════════════════════");
    println!(
        "Total benchmark time: {}.{:03}s",
        total.as_secs(),
        total.subsec_millis()
    );
}

fn run_all(n: usize) {
    b1_no_conflicts::run_all(n);
    println!();
    b2_two_users::run_all(n);
    println!();
    b3_many_conflicts::run_all(n);
}

fn run_filtered(filter: &str, n: usize) {
    let f = filter.to_lowercase();

    match f.as_str() {
        // Entire groups
        "b1" => b1_no_conflicts::run_all(n),
        "b2" => b2_two_users::run_all(n),
        "b3" => b3_many_conflicts::run_all(n),

        // B1 individual
        "b1.1" => b1_no_conflicts::b1_1_append_n_characters(n),
        "b1.2" => b1_no_conflicts::b1_2_insert_string_of_length_n(n),
        "b1.3" => b1_no_conflicts::b1_3_prepend_n_characters(n),
        "b1.4" => b1_no_conflicts::b1_4_insert_n_chars_random(n),
        "b1.5" => b1_no_conflicts::b1_5_insert_n_words_random(n),
        "b1.6" => b1_no_conflicts::b1_6_insert_then_delete(n),
        "b1.7" => b1_no_conflicts::b1_7_insert_delete_random(n),
        "b1.8" => b1_no_conflicts::b1_8_append_n_numbers(n),
        "b1.9" => b1_no_conflicts::b1_9_insert_array_of_n_numbers(n),
        "b1.10" => b1_no_conflicts::b1_10_prepend_n_numbers(n),
        "b1.11" => b1_no_conflicts::b1_11_insert_n_numbers_random(n),

        // B2 individual
        "b2.1" => b2_two_users::b2_1_concurrent_insert_string(n),
        "b2.2" => b2_two_users::b2_2_concurrent_insert_chars_random(n),
        "b2.3" => b2_two_users::b2_3_concurrent_insert_words_random(n),
        "b2.4" => b2_two_users::b2_4_concurrent_insert_delete(n),

        // B3 individual
        "b3.1" => b3_many_conflicts::b3_1_set_number_in_map(n),
        "b3.2" => b3_many_conflicts::b3_2_set_object_in_map(n),
        "b3.3" => b3_many_conflicts::b3_3_set_string_in_map(n),
        "b3.4" => b3_many_conflicts::b3_4_insert_text_in_array(n),

        _ => {
            eprintln!("Unknown filter: '{}'. Valid: b1, b1.1..b1.11, b2, b2.1..b2.4, b3, b3.1..b3.4", filter);
            std::process::exit(1);
        }
    }
}
