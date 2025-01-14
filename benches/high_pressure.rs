use lrust_cache::ShardedLruCache;
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Instant;

const THREAD_COUNT: usize = 4;
const OPERATIONS_PER_THREAD: usize = 1_000_000;
const CACHE_CAPACITY: usize = 100_000;
const KEY_SPACE_SIZE: usize = 10_000;

fn main() {
    println!("Starting LRust Cache test with:");
    println!("- {} threads", THREAD_COUNT);
    println!("- {} operations per thread", OPERATIONS_PER_THREAD);
    println!("- Cache capacity: {}", CACHE_CAPACITY);
    println!("- Key space size: {}", KEY_SPACE_SIZE);

    // Create a shared cache
    let cache = Arc::new(ShardedLruCache::new(CACHE_CAPACITY));

    // Create counters for operations
    let get_hits = Arc::new(AtomicUsize::new(0));
    let get_misses = Arc::new(AtomicUsize::new(0));
    let puts = Arc::new(AtomicUsize::new(0));

    // Create threads
    let mut handles = Vec::with_capacity(THREAD_COUNT);
    let start_time = Instant::now();

    for thread_id in 0..THREAD_COUNT {
        let cache = Arc::clone(&cache);
        let get_hits = Arc::clone(&get_hits);
        let get_misses = Arc::clone(&get_misses);
        let puts = Arc::clone(&puts);

        let handle = thread::spawn(move || {
            // Each thread will do a mix of operations
            for i in 0..OPERATIONS_PER_THREAD {
                // Generate a key in the key space
                let key = format!("key_{}", (i + thread_id) % KEY_SPACE_SIZE);

                // Mix of operations: 80% gets, 20% puts
                if i % 5 != 0 {
                    // GET operation
                    if cache.get(&key).is_some() {
                        get_hits.fetch_add(1, Ordering::Relaxed);
                    } else {
                        get_misses.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    // PUT operation
                    let value = format!("value_{}_{}", thread_id, i);
                    cache.put(key, value);
                    puts.fetch_add(1, Ordering::Relaxed);
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start_time.elapsed();
    let total_operations = get_hits.load(Ordering::Relaxed)
        + get_misses.load(Ordering::Relaxed)
        + puts.load(Ordering::Relaxed);

    println!("\nTest completed in {:.2?}", duration);
    println!("\nStatistics:");
    println!("- Total operations: {}", total_operations);
    println!(
        "- Operations per second: {:.0}",
        total_operations as f64 / duration.as_secs_f64()
    );
    println!("- GET hits: {}", get_hits.load(Ordering::Relaxed));
    println!("- GET misses: {}", get_misses.load(Ordering::Relaxed));
    println!("- PUT operations: {}", puts.load(Ordering::Relaxed));
    println!(
        "- Hit rate: {:.2}%",
        get_hits.load(Ordering::Relaxed) as f64 * 100.0
            / (get_hits.load(Ordering::Relaxed) + get_misses.load(Ordering::Relaxed)) as f64
    );
    println!("- Final cache size: {}", cache.len());
    println!("- Final number of shards: {}", cache.num_shards());
}
