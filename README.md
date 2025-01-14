# LRust Cache

[![CI](https://github.com/lonless9/LrustCache/actions/workflows/ci.yml/badge.svg)](https://github.com/lonless9/LrustCache/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/lonless9/LrustCache/graph/badge.svg?token=9DALCS21QO)](https://codecov.io/gh/lonless9/LrustCache)
[![Crates.io](https://img.shields.io/crates/v/lrust_cache.svg)](https://crates.io/crates/lrust_cache)
[![Documentation](https://docs.rs/lrust_cache/badge.svg)](https://docs.rs/lrust_cache)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A concurrent LRU (Least Recently Used) cache implementation in Rust, focusing on simplicity and performance.

> ⚠️ **Disclaimer**
>
> 1. This project is in early development stage and the API may undergo significant changes
> 2. The code has not undergone comprehensive security audits or performance testing
> 3. **NOT recommended** for production use or handling critical data

## Planned Features

The following features are under development:
- 🚀 High-performance concurrent access (ShardedLruCache only)
- 🔒 Thread-safe implementation (ShardedLruCache only)
- 🎯 Generic key and value type support
- ⚡ Sharded design for better concurrency
- 📦 Simple and intuitive API
- 🔄 FFI support for C/C++ integration

## Implementation Status

Currently provides two implementations:

1. `BasicLruCache`
   - ✅ Basic LRU cache functionality
   - ✅ Generic type support
   - ❌ **NOT** thread-safe
   - Suitable for single-threaded scenarios

2. `ShardedLruCache`
   - ✅ Sharded design
   - ✅ Basic concurrent access
   - ⚠️ Thread safety under verification
   - Intended for concurrent scenarios (requires validation)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
lrust_cache = "0.1.0"  # Note: Version may be unstable
```

## Usage

### Basic LRU Cache (Single-threaded Version)

```rust
use lrust_cache::{Cache, BasicLruCache};

// Create a cache with capacity of 1000
let mut cache = BasicLruCache::new(1000);

// Warning: BasicLruCache is NOT thread-safe!
cache.put("key1", "value1");
if let Some(value) = cache.get(&"key1") {
    println!("Got value: {}", value);
}
```

### Sharded LRU Cache (Concurrent Version)

For concurrent scenarios, use the sharded implementation:

```rust
use lrust_cache::{Cache, ShardedLruCache};

// Create a sharded cache with total capacity of 1000
let cache = ShardedLruCache::new(1000);

// ShardedLruCache supports concurrent access
cache.put("key1", "value1");
if let Some(value) = cache.get(&"key1") {
    println!("Got value: {}", value);
}
```

## Performance Benchmarks

> ⚠️ Note: The following performance data is for reference only. Actual performance may vary depending on usage scenarios

Test scenario:
- 10,000 unique key-value pairs
- 10 concurrent threads
- Each thread performs random operations (80% reads, 20% writes)
- Tests from 1K to 1M operations per thread
- Key space size: 10,000

Preliminary results (operations per second):
- LRust Cache: ~15.4M ops/sec
- leveldb LRU: ~11.3M ops/sec

Complete benchmark code is available in the `cpp_bench` directory. It's recommended to conduct your own performance testing before use.

## Implementation Details

- `BasicLruCache`: 
  - Single-shard implementation using hash map and doubly-linked list
  - No concurrent access support
  - Suitable for single-threaded scenarios

- `ShardedLruCache`: 
  - Multi-shard implementation for reduced lock contention
  - Basic concurrent access support
  - Thread safety verification in progress

### C++ Integration

> ⚠⚠️ C++ integration API is under development and subject to change

```cpp
#include "rust_cache.h"

// Create cache
RustCache cache(100000);

// Basic operations
cache.put("key1", "value1");
std::string value = cache.get("key1");
```