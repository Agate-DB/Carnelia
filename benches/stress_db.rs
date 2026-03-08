use carnelia::stress_test::{
    stress_test_document_store, stress_test_json_crdt, stress_test_rga_text, stress_test_rich_text,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_rga(c: &mut Criterion) {
    c.bench_function("stress_rga_text_small", |b| {
        b.iter(|| {
            let stats = stress_test_rga_text(3, 20);
            assert!(stats.converged);
        })
    });
}

fn bench_rich_text(c: &mut Criterion) {
    c.bench_function("stress_rich_text_small", |b| {
        b.iter(|| {
            let stats = stress_test_rich_text(3, 20);
            assert!(stats.converged);
        })
    });
}

fn bench_json(c: &mut Criterion) {
    c.bench_function("stress_json_crdt_small", |b| {
        b.iter(|| {
            let stats = stress_test_json_crdt(3, 20);
            assert!(stats.converged);
        })
    });
}

fn bench_doc_store(c: &mut Criterion) {
    c.bench_function("stress_document_store_small", |b| {
        b.iter(|| {
            let stats = stress_test_document_store(12, 40);
            assert!(stats.converged);
        })
    });
}

criterion_group!(
    db_stress,
    bench_rga,
    bench_rich_text,
    bench_json,
    bench_doc_store
);
criterion_main!(db_stress);
