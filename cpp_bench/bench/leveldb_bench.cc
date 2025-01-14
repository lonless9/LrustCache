#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string>
#include <thread>
#include <vector>
#include <atomic>
#include <chrono>
#include "leveldb/cache.h"

const int THREAD_COUNT = 4;
const int OPERATIONS_PER_THREAD = 1000000;
const int CACHE_CAPACITY = 100000;
const int KEY_SPACE_SIZE = 10000;

std::atomic<size_t> get_hits{0};
std::atomic<size_t> get_misses{0};
std::atomic<size_t> put_ops{0};

void worker_thread(leveldb::Cache* cache, int thread_id) {
    for (int i = 0; i < OPERATIONS_PER_THREAD; i++) {
        // Generate a key in the key space
        std::string key = "key_" + std::to_string((i + thread_id) % KEY_SPACE_SIZE);
        
        // Mix of operations: 80% gets, 20% puts
        if (i % 5 != 0) {
            // GET operation
            leveldb::Cache::Handle* handle = cache->Lookup(leveldb::Slice(key));
            if (handle != nullptr) {
                get_hits.fetch_add(1, std::memory_order_relaxed);
                cache->Release(handle);
            } else {
                get_misses.fetch_add(1, std::memory_order_relaxed);
            }
        } else {
            // PUT operation
            std::string* value = new std::string("value_" + std::to_string(thread_id) + "_" + std::to_string(i));
            auto handle = cache->Insert(leveldb::Slice(key), value, 1,
                [](const leveldb::Slice& key, void* value) {
                    delete static_cast<std::string*>(value);
                });
            if (handle != nullptr) {
                cache->Release(handle);
            }
            put_ops.fetch_add(1, std::memory_order_relaxed);
        }
    }
}

int main() {
    printf("Starting LevelDB LRU Cache test with:\n");
    printf("- %d threads\n", THREAD_COUNT);
    printf("- %d operations per thread\n", OPERATIONS_PER_THREAD);
    printf("- Cache capacity: %d\n", CACHE_CAPACITY);
    printf("- Key space size: %d\n", KEY_SPACE_SIZE);

    // Create cache
    leveldb::Cache* cache = leveldb::NewLRUCache(CACHE_CAPACITY);

    // Create threads
    std::vector<std::thread> threads;
    
    // Start timing
    auto start_time = std::chrono::high_resolution_clock::now();

    // Create and start threads
    for (int i = 0; i < THREAD_COUNT; i++) {
        threads.emplace_back(worker_thread, cache, i);
    }

    // Wait for all threads to complete
    for (auto& thread : threads) {
        thread.join();
    }

    // End timing after all threads complete
    auto end_time = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::milliseconds>(end_time - start_time);

    size_t total_operations = get_hits.load(std::memory_order_relaxed) + 
                            get_misses.load(std::memory_order_relaxed) + 
                            put_ops.load(std::memory_order_relaxed);

    printf("\nTest completed in %.2fms\n", static_cast<double>(duration.count()));
    printf("\nStatistics:\n");
    printf("- Total operations: %zu\n", total_operations);
    printf("- Operations per second: %.0f\n", 
           total_operations * 1000.0 / duration.count());
    printf("- GET hits: %zu\n", get_hits.load(std::memory_order_relaxed));
    printf("- GET misses: %zu\n", get_misses.load(std::memory_order_relaxed));
    printf("- PUT operations: %zu\n", put_ops.load(std::memory_order_relaxed));
    printf("- Hit rate: %.2f%%\n", 
           get_hits.load(std::memory_order_relaxed) * 100.0 / 
           (get_hits.load(std::memory_order_relaxed) + get_misses.load(std::memory_order_relaxed)));
    printf("- Final cache size: %zu\n", cache->TotalCharge());

    // Clear all entries before deleting the cache
    for (int i = 0; i < KEY_SPACE_SIZE; i++) {
        std::string key = "key_" + std::to_string(i);
        cache->Erase(leveldb::Slice(key));
    }

    delete cache;
    return 0;
} 