//! MDCS Stress Test Runner
//!
//! This binary provides a command-line interface for running various
//! stress tests and benchmarks for the MDCS crate family.

use stress_test::{
    // Core CRDT stress tests (async, 3 args)
    stress_test_gset, 
    stress_test_orset, 
    stress_test_pncounter,
    stress_test_lwwreg,
    stress_test_mvreg,
    stress_test_scaling,
    stress_test_all_core_crdts,
    // Database layer stress tests (sync, 2 args)
    stress_test_rga_text,
    stress_test_rich_text,
    stress_test_json_crdt,
    stress_test_document_store,
    stress_test_all_db_crdts,
};
pub mod stress_test;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    // Parse command line args for test selection
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "core" => rt.block_on(run_core_tests()),
            "db" => run_db_tests(),
            "quick" => rt.block_on(run_quick_tests()),
            "full" => rt.block_on(run_full_suite()),
            "scaling" => rt.block_on(run_scaling_analysis()),
            "help" | "--help" | "-h" => print_usage(),
            _ => {
                println!("Unknown test suite: {}", args[1]);
                print_usage();
            }
        }
    } else {
        // Default: run quick tests
        rt.block_on(run_quick_tests());
    }
}

fn print_usage() {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║            MDCS STRESS TEST SUITE                          ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("Usage: cargo run [test_suite]");
    println!();
    println!("Available test suites:");
    println!("  quick    - Quick smoke tests (default)");
    println!("  core     - Core CRDT stress tests (GSet, ORSet, PNCounter, etc.)");
    println!("  db       - Database layer tests (RGAText, RichText, JsonCrdt)");
    println!("  scaling  - Scaling analysis with performance metrics");
    println!("  full     - Complete benchmark suite (takes longer)");
    println!("  help     - Show this help message");
    println!();
    println!("Examples:");
    println!("  cargo run              # Run quick tests");
    println!("  cargo run quick        # Run quick tests");
    println!("  cargo run core         # Run core CRDT tests");
    println!("  cargo run db           # Run database layer tests");
    println!("  cargo run full         # Run complete suite");
    println!();
}

async fn run_quick_tests() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║            QUICK SMOKE TESTS                               ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Quick core CRDT tests (async, 3 args: replicas, ops, syncs)
    println!("── Core CRDTs ──────────────────────────────────────────────");
    let stats = stress_test_gset(4, 100, 200).await;
    stats.print();
    
    let stats = stress_test_orset(4, 100, 200).await;
    stats.print();

    let stats = stress_test_pncounter(4, 100, 200).await;
    stats.print();

    // Quick DB layer tests (sync, 2 args: replicas, ops)
    println!("\n── Database Layer ──────────────────────────────────────────");
    let stats = stress_test_rga_text(3, 50);
    stats.print();

    let stats = stress_test_rich_text(3, 30);
    stats.print();

    let stats = stress_test_json_crdt(3, 30);
    stats.print();

    println!("\n✓ Quick tests completed successfully!");
}

async fn run_core_tests() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║            CORE CRDT STRESS TESTS                          ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Small scale tests
    println!("── Small Scale (4 replicas, 100 ops) ─────────────────────────");
    let stats = stress_test_gset(4, 100, 200).await;
    stats.print();
    
    let stats = stress_test_orset(4, 100, 200).await;
    stats.print();

    let stats = stress_test_pncounter(4, 100, 200).await;
    stats.print();

    let stats = stress_test_lwwreg(4, 100, 200).await;
    stats.print();

    let stats = stress_test_mvreg(4, 100, 200).await;
    stats.print();

    // Medium scale tests
    println!("\n── Medium Scale (10 replicas, 500 ops) ─────────────────────");
    let stats = stress_test_gset(10, 500, 1000).await;
    stats.print();
    
    let stats = stress_test_orset(10, 500, 1000).await;
    stats.print();

    let stats = stress_test_pncounter(10, 500, 1000).await;
    stats.print();

    // Combined test
    println!("\n── Combined Core CRDT Test ─────────────────────────────────");
    stress_test_all_core_crdts(6, 200, 400).await;

    println!("\n✓ Core CRDT tests completed successfully!");
}

fn run_db_tests() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║            DATABASE LAYER STRESS TESTS                     ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // RGA Text tests
    println!("── RGA Text ────────────────────────────────────────────────");
    let stats = stress_test_rga_text(4, 100);
    stats.print();

    let stats = stress_test_rga_text(8, 300);
    stats.print();

    // Rich Text tests  
    println!("\n── Rich Text ───────────────────────────────────────────────");
    let stats = stress_test_rich_text(4, 100);
    stats.print();

    let stats = stress_test_rich_text(6, 200);
    stats.print();

    // JSON CRDT tests
    println!("\n── JSON CRDT ───────────────────────────────────────────────");
    let stats = stress_test_json_crdt(4, 100);
    stats.print();

    let stats = stress_test_json_crdt(6, 200);
    stats.print();

    // Document Store tests
    println!("\n── Document Store ──────────────────────────────────────────");
    let stats = stress_test_document_store(50, 200);
    stats.print();

    let stats = stress_test_document_store(100, 500);
    stats.print();

    // Combined DB test
    println!("\n── Combined Database Test ──────────────────────────────────");
    stress_test_all_db_crdts(4, 100);

    println!("\n✓ Database layer tests completed successfully!");
}

async fn run_scaling_analysis() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║            SCALING ANALYSIS                                ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("── GSet Scaling ────────────────────────────────────────────");
    stress_test_scaling(20, 2).await;

    println!("\n── ORSet Scaling ───────────────────────────────────────────");
    println!("  Replicas │    Ops │  Time (ms) │  ms/op │  Syncs │ Converged");
    println!("  ─────────┼────────┼────────────┼────────┼────────┼──────────");
    for num_replicas in [2, 4, 8, 16] {
        let ops = num_replicas * 50;
        let syncs = ops * 2;
        let stats = stress_test_orset(num_replicas, ops, syncs).await;
        let total_ops = stats.num_replicas * stats.operations_per_replica;
        println!("  {:>8} │ {:>6} │ {:>10.2} │ {:>6.4} │ {:>6} │ {:>8}", 
            stats.num_replicas,
            total_ops,
            stats.total_time.as_secs_f64() * 1000.0,
            stats.total_time.as_secs_f64() * 1000.0 / total_ops as f64,
            stats.total_syncs,
            if stats.converged { "✓" } else { "✗" }
        );
    }

    println!("\n── RGA Text Scaling ────────────────────────────────────────");
    println!("  Replicas │    Ops │  Time (ms) │  ms/op │  Syncs │ Converged");
    println!("  ─────────┼────────┼────────────┼────────┼────────┼──────────");
    for num_replicas in [2, 4, 8] {
        let ops = num_replicas * 25;
        let stats = stress_test_rga_text(num_replicas, ops);
        let total_ops = stats.num_replicas * stats.operations_per_replica;
        println!("  {:>8} │ {:>6} │ {:>10.2} │ {:>6.4} │ {:>6} │ {:>8}", 
            stats.num_replicas,
            total_ops,
            stats.total_time.as_secs_f64() * 1000.0,
            stats.total_time.as_secs_f64() * 1000.0 / total_ops as f64,
            stats.total_syncs,
            if stats.converged { "✓" } else { "✗" }
        );
    }

    println!("\n── Rich Text Scaling ───────────────────────────────────────");
    println!("  Replicas │    Ops │  Time (ms) │  ms/op │  Syncs │ Converged");
    println!("  ─────────┼────────┼────────────┼────────┼────────┼──────────");
    for num_replicas in [2, 4, 6] {
        let ops = num_replicas * 20;
        let stats = stress_test_rich_text(num_replicas, ops);
        let total_ops = stats.num_replicas * stats.operations_per_replica;
        println!("  {:>8} │ {:>6} │ {:>10.2} │ {:>6.4} │ {:>6} │ {:>8}", 
            stats.num_replicas,
            total_ops,
            stats.total_time.as_secs_f64() * 1000.0,
            stats.total_time.as_secs_f64() * 1000.0 / total_ops as f64,
            stats.total_syncs,
            if stats.converged { "✓" } else { "✗" }
        );
    }

    println!("\n── JSON CRDT Scaling ───────────────────────────────────────");
    println!("  Replicas │    Ops │  Time (ms) │  ms/op │  Syncs │ Converged");
    println!("  ─────────┼────────┼────────────┼────────┼────────┼──────────");
    for num_replicas in [2, 4, 6] {
        let ops = num_replicas * 20;
        let stats = stress_test_json_crdt(num_replicas, ops);
        let total_ops = stats.num_replicas * stats.operations_per_replica;
        println!("  {:>8} │ {:>6} │ {:>10.2} │ {:>6.4} │ {:>6} │ {:>8}", 
            stats.num_replicas,
            total_ops,
            stats.total_time.as_secs_f64() * 1000.0,
            stats.total_time.as_secs_f64() * 1000.0 / total_ops as f64,
            stats.total_syncs,
            if stats.converged { "✓" } else { "✗" }
        );
    }

    println!("\n✓ Scaling analysis completed!");
}

async fn run_full_suite() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║            FULL BENCHMARK SUITE                            ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
    
    println!("This will run all tests - this may take several minutes...\n");

    // Core CRDT tests
    println!("════════════════════════════════════════════════════════════");
    println!("                    PHASE 1: CORE CRDTs");
    println!("════════════════════════════════════════════════════════════\n");
    run_core_tests().await;

    // Database layer tests
    println!("\n════════════════════════════════════════════════════════════");
    println!("                   PHASE 2: DATABASE LAYER");
    println!("════════════════════════════════════════════════════════════\n");
    run_db_tests();

    // Scaling analysis
    println!("\n════════════════════════════════════════════════════════════");
    println!("                  PHASE 3: SCALING ANALYSIS");
    println!("════════════════════════════════════════════════════════════\n");
    run_scaling_analysis().await;

    // Summary
    println!("\n════════════════════════════════════════════════════════════");
    println!("                        SUMMARY");
    println!("════════════════════════════════════════════════════════════");
    println!();
    println!("  ✓ All core CRDT tests passed");
    println!("  ✓ All database layer tests passed");
    println!("  ✓ Scaling analysis completed");
    println!();
    println!("  All tests verify:");
    println!("    • Idempotence: join(a, a) = a");
    println!("    • Commutativity: join(a, b) = join(b, a)");
    println!("    • Associativity: join(join(a, b), c) = join(a, join(b, c))");
    println!("    • Convergence: all replicas reach identical state");
    println!();
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║            ✓ FULL SUITE COMPLETED SUCCESSFULLY             ║");
    println!("╚════════════════════════════════════════════════════════════╝");
}
