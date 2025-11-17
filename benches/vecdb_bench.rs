use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use vecdb_wasm::distance::{
    angular_distance, calculate_distance, chebyshev_distance, cosine_similarity,
    euclidean_distance, hamming_distance, manhattan_distance, normalize_vector, DistanceMetric,
};
use vecdb_wasm::index::{FlatIndex, Index, HNSWIndex};

// Generate random vector
fn random_vector(dim: usize) -> Vec<f32> {
    (0..dim).map(|_| rand::random::<f32>()).collect()
}

// Generate dataset
fn generate_dataset(count: usize, dim: usize) -> Vec<Vec<f32>> {
    (0..count).map(|_| random_vector(dim)).collect()
}

// Benchmark distance calculations
fn bench_distance_functions(c: &mut Criterion) {
    let dimensions = vec![64, 128, 256, 512, 1024];

    for &dim in &dimensions {
        let v1 = random_vector(dim);
        let v2 = random_vector(dim);

        let mut group = c.benchmark_group(format!("distance_{}d", dim));
        group.throughput(Throughput::Elements(1));

        group.bench_function("cosine", |b| {
            b.iter(|| cosine_similarity(black_box(&v1), black_box(&v2)))
        });

        group.bench_function("euclidean", |b| {
            b.iter(|| euclidean_distance(black_box(&v1), black_box(&v2)))
        });

        group.bench_function("manhattan", |b| {
            b.iter(|| manhattan_distance(black_box(&v1), black_box(&v2)))
        });

        group.bench_function("chebyshev", |b| {
            b.iter(|| chebyshev_distance(black_box(&v1), black_box(&v2)))
        });

        group.bench_function("hamming", |b| {
            b.iter(|| hamming_distance(black_box(&v1), black_box(&v2)))
        });

        group.bench_function("angular", |b| {
            b.iter(|| angular_distance(black_box(&v1), black_box(&v2)))
        });

        group.finish();
    }
}

// Benchmark normalization
fn bench_normalization(c: &mut Criterion) {
    let dimensions = vec![64, 128, 256, 512, 1024];

    for &dim in &dimensions {
        let vector = random_vector(dim);

        let mut group = c.benchmark_group(format!("normalize_{}d", dim));
        group.throughput(Throughput::Elements(1));

        group.bench_function("normalize", |b| {
            b.iter(|| normalize_vector(black_box(&vector)))
        });

        group.finish();
    }
}

// Benchmark flat index insertion
fn bench_flat_insert(c: &mut Criterion) {
    let configs = vec![
        (1_000, 128, "1k_128d"),
        (10_000, 128, "10k_128d"),
        (1_000, 512, "1k_512d"),
    ];

    for (count, dim, label) in configs {
        let dataset = generate_dataset(count, dim);
        let mut group = c.benchmark_group(format!("flat_insert_{}", label));
        group.throughput(Throughput::Elements(count as u64));

        group.bench_function("insert", |b| {
            b.iter(|| {
                let mut index = FlatIndex::new(dim, DistanceMetric::Cosine);
                for (id, vec) in dataset.iter().enumerate() {
                    index.insert(id as u64, vec);
                }
            })
        });

        group.finish();
    }
}

// Benchmark HNSW index insertion
fn bench_hnsw_insert(c: &mut Criterion) {
    let configs = vec![
        (1_000, 128, "1k_128d"),
        (10_000, 128, "10k_128d"),
        (1_000, 512, "1k_512d"),
    ];

    for (count, dim, label) in configs {
        let dataset = generate_dataset(count, dim);
        let mut group = c.benchmark_group(format!("hnsw_insert_{}", label));
        group.throughput(Throughput::Elements(count as u64));

        group.bench_function("insert", |b| {
            b.iter(|| {
                let mut index = HNSWIndex::new(dim, DistanceMetric::Cosine, 16, 200);
                for (id, vec) in dataset.iter().enumerate() {
                    index.insert(id as u64, vec);
                }
            })
        });

        group.finish();
    }
}

// Benchmark flat index search
fn bench_flat_search(c: &mut Criterion) {
    let configs = vec![
        (1_000, 128, 10, "1k_128d_k10"),
        (10_000, 128, 10, "10k_128d_k10"),
        (10_000, 128, 100, "10k_128d_k100"),
        (1_000, 512, 10, "1k_512d_k10"),
    ];

    for (count, dim, k, label) in configs {
        let dataset = generate_dataset(count, dim);
        let mut index = FlatIndex::new(dim, DistanceMetric::Cosine);
        for (id, vec) in dataset.iter().enumerate() {
            index.insert(id as u64, vec);
        }

        let query = random_vector(dim);
        let mut group = c.benchmark_group(format!("flat_search_{}", label));
        group.throughput(Throughput::Elements(1));

        group.bench_function("search", |b| {
            b.iter(|| index.search(black_box(&query), black_box(k)))
        });

        group.finish();
    }
}

// Benchmark HNSW index search
fn bench_hnsw_search(c: &mut Criterion) {
    let configs = vec![
        (1_000, 128, 10, "1k_128d_k10"),
        (10_000, 128, 10, "10k_128d_k10"),
        (10_000, 128, 100, "10k_128d_k100"),
        (1_000, 512, 10, "1k_512d_k10"),
    ];

    for (count, dim, k, label) in configs {
        let dataset = generate_dataset(count, dim);
        let mut index = HNSWIndex::new(dim, DistanceMetric::Cosine, 16, 200);
        for (id, vec) in dataset.iter().enumerate() {
            index.insert(id as u64, vec);
        }

        let query = random_vector(dim);
        let mut group = c.benchmark_group(format!("hnsw_search_{}", label));
        group.throughput(Throughput::Elements(1));

        group.bench_function("search", |b| {
            b.iter(|| index.search(black_box(&query), black_box(k)))
        });

        group.finish();
    }
}

// Benchmark different metrics comparison
fn bench_metrics_comparison(c: &mut Criterion) {
    let dim = 128;
    let count = 10_000;
    let k = 10;

    let dataset = generate_dataset(count, dim);
    let query = random_vector(dim);

    let metrics = vec![
        (DistanceMetric::Cosine, "cosine"),
        (DistanceMetric::Euclidean, "euclidean"),
        (DistanceMetric::Manhattan, "manhattan"),
        (DistanceMetric::Chebyshev, "chebyshev"),
        (DistanceMetric::Angular, "angular"),
    ];

    let mut group = c.benchmark_group("metric_comparison_10k_128d");
    group.throughput(Throughput::Elements(1));

    for (metric, name) in metrics {
        let mut index = FlatIndex::new(dim, metric);
        for (id, vec) in dataset.iter().enumerate() {
            index.insert(id as u64, vec);
        }

        group.bench_function(name, |b| {
            b.iter(|| index.search(black_box(&query), black_box(k)))
        });
    }

    group.finish();
}

// Benchmark batch operations
fn bench_batch_search(c: &mut Criterion) {
    let dim = 128;
    let count = 10_000;
    let k = 10;
    let batch_sizes = vec![10, 100, 1000];

    let dataset = generate_dataset(count, dim);
    let mut index = HNSWIndex::new(dim, DistanceMetric::Cosine, 16, 200);
    for (id, vec) in dataset.iter().enumerate() {
        index.insert(id as u64, vec);
    }

    for batch_size in batch_sizes {
        let queries: Vec<Vec<f32>> = (0..batch_size).map(|_| random_vector(dim)).collect();

        let mut group = c.benchmark_group(format!("batch_search_{}queries", batch_size));
        group.throughput(Throughput::Elements(batch_size as u64));

        group.bench_function("hnsw", |b| {
            b.iter(|| {
                for query in &queries {
                    index.search(black_box(query), black_box(k));
                }
            })
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    bench_distance_functions,
    bench_normalization,
    bench_flat_insert,
    bench_hnsw_insert,
    bench_flat_search,
    bench_hnsw_search,
    bench_metrics_comparison,
    bench_batch_search
);
criterion_main!(benches);
