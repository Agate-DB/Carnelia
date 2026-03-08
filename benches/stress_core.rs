use carnelia::stress_test::{stress_test_gset, stress_test_orset, stress_test_pncounter};
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn bench_gset(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    c.bench_function("stress_gset_small", |b| {
        b.to_async(&rt).iter(|| async {
            let stats = stress_test_gset(4, 50, 100).await;
            assert!(stats.converged);
        })
    });
}

fn bench_orset(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    c.bench_function("stress_orset_small", |b| {
        b.to_async(&rt).iter(|| async {
            let stats = stress_test_orset(4, 50, 100).await;
            assert!(stats.converged);
        })
    });
}

fn bench_pncounter(c: &mut Criterion) {
    let rt = Runtime::new().expect("tokio runtime");
    c.bench_function("stress_pncounter_small", |b| {
        b.to_async(&rt).iter(|| async {
            let stats = stress_test_pncounter(4, 50, 100).await;
            assert!(stats.converged);
        })
    });
}

criterion_group!(core_stress, bench_gset, bench_orset, bench_pncounter);
criterion_main!(core_stress);
