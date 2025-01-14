//! A high-performance LRU (Least Recently Used) cache implementation in Rust.
//!
//! This crate provides two LRU cache implementations:
//!
//! 1. [`BasicLruCache`] - A thread-safe LRU cache implementation using fine-grained locking
//! 2. [`ShardedLruCache`] - A sharded LRU cache implementation for better concurrent performance
//!
//! # Features
//!
//! - Thread-safe implementations
//! - High-performance concurrent access
//! - Memory-efficient storage
//! - Configurable capacity
//! - Generic key type support
//! - Automatic sharding for better scalability
//!
//! # Examples
//!
//! ```rust
//! use lrust_cache::{Cache, BasicLruCache, ShardedLruCache};
//!
//! // Create a basic LRU cache for storing strings with string keys
//! let basic_cache: BasicLruCache<String, String> = BasicLruCache::new(1000);
//!
//! // Create a sharded LRU cache for better concurrent performance
//! let sharded_cache: ShardedLruCache<String, String> = ShardedLruCache::new(1000);
//!
//! // You can use any type that implements Clone + Debug + Hash + Eq + Send + Sync + 'static as key
//! let cache: BasicLruCache<u64, String> = BasicLruCache::new(1000);
//! cache.put(42, "answer".to_string());
//! assert_eq!(cache.get(&42), Some("answer".to_string()));
//! ```

pub mod basic_lru_cache;
mod ffi;
pub mod sharded_lru_cache;

pub use basic_lru_cache::{BasicLruCache, Cache};
pub use sharded_lru_cache::ShardedLruCache;
