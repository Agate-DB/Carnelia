use stress_test::{stress_test_gset, stress_test_orset, stress_test_scaling};
pub mod stress_test;

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {

    // Run async stress tests
    println!("\n\n╔════════════════════════════════════════════════════════════╗");
    println!("║            ASYNC STRESS TESTS                               ║");
    println!("╚════════════════════════════════════════════════════════════╝");

    // Test 1: GSet with small scale
    let stats = stress_test_gset(4, 100, 200).await;
    stats.print();

    // Test 2: ORSet with small scale
    let stats = stress_test_orset(4, 100, 200).await;
    stats.print();

    // Test 3: GSet with medium scale
    let stats = stress_test_gset(10, 500, 1000).await;
    stats.print();

    // Test 4: ORSet with medium scale
    let stats = stress_test_orset(10, 500, 1000).await;
    stats.print();

    // Test 5: Scaling analysis
    println!("\n\n╔════════════════════════════════════════════════════════════╗");
    println!("║          SCALING ANALYSIS (GSet)                           ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    stress_test_scaling(20, 2).await;

    println!("\n✓ All stress tests completed successfully!");
}
