#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <thread>
#include <vector>
#include <atomic>
#include <chrono>
#include "rust_cache.h"

const int THREAD_COUNT = 4;
const int OPERATIONS_PER_THREAD = 1000000;
const int CACHE_CAPACITY = 100000;
const int KEY_SPACE_SIZE = 10000;

std::atomic<size_t> get_hits{0};
std::atomic<size_t> get_misses{0};
std::atomic<size_t> put_ops{0};

void worker_thread(RustCache* cache, int thread_id) {
    for (int i = 0; i < OPERATIONS_PER_THREAD; i++) {
        // Generate a key in the key space
        std::string key = "key_" + std::to_string((i + thread_id) % KEY_SPACE_SIZE);
        
        // Mix of operations: 80% gets, 20% puts
        if (i % 5 != 0) {
            // GET operation
            std::string value = cache->get(key);
            if (!value.empty()) {
                get_hits.fetch_add(1, std::memory_order_relaxed);
            } else {
                get_misses.fetch_add(1, std::memory_order_relaxed);
            }
        } else {
            // PUT operation
            std::string value = "value_" + std::to_string(thread_id) + "_" + std::to_string(i);
            cache->put(key, value);
            put_ops.fetch_add(1, std::memory_order_relaxed);
        }
    }
}

int main() {
    printf("Starting LRust Cache FFI test with:\n");
    printf("- %d threads\n", THREAD_COUNT);
    printf("- %d operations per thread\n", OPERATIONS_PER_THREAD);
    printf("- Cache capacity: %d\n", CACHE_CAPACITY);
    printf("- Key space size: %d\n", KEY_SPACE_SIZE);

    // Create cache
    RustCache cache(CACHE_CAPACITY);

    // Create and start threads
    std::vector<std::thread> threads;
    auto start = std::chrono::high_resolution_clock::now();

    for (int i = 0; i < THREAD_COUNT; i++) {
        threads.emplace_back(worker_thread, &cache, i);
    }

    // Wait for all threads to finish
    for (auto& t : threads) {
        t.join();
    }

    auto end = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end - start);

    // Calculate statistics
    size_t total_ops = get_hits + get_misses + put_ops;
    double ops_per_sec = total_ops * 1000.0 / duration.count();
    double hit_rate = get_hits * 100.0 / (get_hits + get_misses);

    printf("\nTest completed in %.2fms\n\n", static_cast<double>(duration.count()));
    printf("Statistics:\n");
    printf("- Total operations: %zu\n", total_ops);
    printf("- Operations per second: %.0f\n", ops_per_sec);
    printf("- GET hits: %zu\n", get_hits.load());
    printf("- GET misses: %zu\n", get_misses.load());
    printf("- PUT operations: %zu\n", put_ops.load());
    printf("- Hit rate: %.2f%%\n", hit_rate);
    printf("- Final cache size: %zu\n", cache.size());

    return 0;
} 