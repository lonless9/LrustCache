use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, PlotConfiguration};
use lrust_cache::ShardedLruCache;
use moka::sync::Cache as MokaCache;
use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const THREAD_COUNT: usize = 10;
const OPERATIONS_PER_THREAD: usize = 100_000;

#[derive(Clone)]
struct BenchConfig {
    name: String,
    cache_size: usize,
    key_space: usize,
    write_ratio: usize, // Number of write operations per 10 operations
    value_size: usize,  // Size of values in bytes
}

impl BenchConfig {
    fn new(
        name: &str,
        cache_size: usize,
        key_space: usize,
        write_ratio: usize,
        value_size: usize,
    ) -> Self {
        Self {
            name: name.to_string(),
            cache_size,
            key_space,
            write_ratio,
            value_size,
        }
    }
}

// Define different test scenarios
fn get_cache_size_configs() -> Vec<BenchConfig> {
    vec![
        BenchConfig::new("1K", 1_000, 10_000, 5, 64),
        BenchConfig::new("5K", 5_000, 10_000, 5, 64),
        BenchConfig::new("10K", 10_000, 10_000, 5, 64),
        BenchConfig::new("50K", 50_000, 10_000, 5, 64),
        BenchConfig::new("100K", 100_000, 10_000, 5, 64),
    ]
}

fn get_write_ratio_configs() -> Vec<BenchConfig> {
    vec![
        BenchConfig::new("10% writes", 10_000, 20_000, 1, 64),
        BenchConfig::new("20% writes", 10_000, 20_000, 2, 64),
        BenchConfig::new("50% writes", 10_000, 20_000, 5, 64),
        BenchConfig::new("80% writes", 10_000, 20_000, 8, 64),
    ]
}

fn get_value_size_configs() -> Vec<BenchConfig> {
    vec![
        BenchConfig::new("16B", 10_000, 20_000, 5, 16),
        BenchConfig::new("64B", 10_000, 20_000, 5, 64),
        BenchConfig::new("256B", 10_000, 20_000, 5, 256),
        BenchConfig::new("1KB", 10_000, 20_000, 5, 1024),
        BenchConfig::new("4KB", 10_000, 20_000, 5, 4096),
    ]
}

fn get_key_space_configs() -> Vec<BenchConfig> {
    vec![
        BenchConfig::new("5K keys", 10_000, 5_000, 5, 64),
        BenchConfig::new("10K keys", 10_000, 10_000, 5, 64),
        BenchConfig::new("20K keys", 10_000, 20_000, 5, 64),
        BenchConfig::new("50K keys", 10_000, 50_000, 5, 64),
    ]
}

fn generate_value(size: usize) -> String {
    let mut value = String::with_capacity(size);
    for _ in 0..size {
        value.push('x');
    }
    value
}

fn bench_cache(cache_type: CacheType, config: &BenchConfig) -> Duration {
    match cache_type {
        CacheType::LRust => {
            let cache = Arc::new(ShardedLruCache::new(config.cache_size));
            // Pre-populate with half of the key space
            for i in 0..config.key_space / 2 {
                let key = format!("key_{}", i);
                let value = generate_value(config.value_size);
                cache.put(key, value);
            }

            let start = std::time::Instant::now();
            let mut handles = vec![];

            for _thread_id in 0..THREAD_COUNT {
                let cache = Arc::clone(&cache);
                let config = config.clone();
                handles.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    for i in 0..OPERATIONS_PER_THREAD {
                        let key = format!("key_{}", rng.gen_range(0..config.key_space));
                        if i % 10 < config.write_ratio {
                            // Write operation
                            let value = generate_value(config.value_size);
                            cache.put(key, value);
                        } else {
                            // Read operation
                            let _ = cache.get(&key);
                        }
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
            start.elapsed()
        }
        CacheType::Moka => {
            let cache: Arc<MokaCache<String, String>> =
                Arc::new(MokaCache::new(config.cache_size as u64));
            // Pre-populate with half of the key space
            for i in 0..config.key_space / 2 {
                let key = format!("key_{}", i);
                let value = generate_value(config.value_size);
                cache.insert(key, value);
            }

            let start = std::time::Instant::now();
            let mut handles = vec![];

            for _thread_id in 0..THREAD_COUNT {
                let cache = Arc::clone(&cache);
                let config = config.clone();
                handles.push(thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    for i in 0..OPERATIONS_PER_THREAD {
                        let key = format!("key_{}", rng.gen_range(0..config.key_space));
                        if i % 10 < config.write_ratio {
                            // Write operation
                            let value = generate_value(config.value_size);
                            cache.insert(key, value);
                        } else {
                            // Read operation
                            let _ = cache.get(&key);
                        }
                    }
                }));
            }

            for handle in handles {
                handle.join().unwrap();
            }
            start.elapsed()
        }
    }
}

#[derive(Clone, Copy)]
enum CacheType {
    LRust,
    Moka,
}

fn run_benchmark_group(c: &mut Criterion, name: &str, configs: Vec<BenchConfig>) {
    let plot_config = PlotConfiguration::default().summary_scale(criterion::AxisScale::Linear);

    let mut group = c.benchmark_group(name);
    group.plot_config(plot_config);
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for config in configs.iter() {
        group.bench_with_input(
            BenchmarkId::new("LRust Cache", &config.name),
            config,
            |b, config| {
                b.iter(|| bench_cache(CacheType::LRust, config));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Moka Cache", &config.name),
            config,
            |b, config| {
                b.iter(|| bench_cache(CacheType::Moka, config));
            },
        );
    }
    group.finish();
}

fn concurrent_benchmark(c: &mut Criterion) {
    // Test impact of different cache sizes
    run_benchmark_group(c, "Cache Size Impact", get_cache_size_configs());

    // Test impact of different write ratios
    run_benchmark_group(c, "Write Ratio Impact", get_write_ratio_configs());

    // Test impact of different value sizes
    run_benchmark_group(c, "Value Size Impact", get_value_size_configs());

    // Test impact of different key space sizes
    run_benchmark_group(c, "Key Space Impact", get_key_space_configs());
}

criterion_group!(benches, concurrent_benchmark);
criterion_main!(benches);
