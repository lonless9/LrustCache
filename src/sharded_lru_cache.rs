use super::basic_lru_cache::private::Cache;
use super::BasicLruCache;
use parking_lot::Mutex;
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

// Minimum capacity per shard
const MIN_SHARD_CAPACITY: usize = 4;
// Maximum number of shards, must be a power of 2
const MAX_SHARDS: usize = 16;

/// A sharded LRU cache implementation for high-concurrency scenarios.
///
/// This implementation divides the cache into multiple shards, each protected by its own mutex,
/// to reduce contention in concurrent access scenarios. The number of shards is automatically
/// determined based on the total capacity, but will not exceed `MAX_SHARDS`.
///
/// # Type Parameters
///
/// * `K` - The type of keys used in the cache. Must implement `Clone + Debug + Hash + Eq + Send + Sync + 'static`
/// * `V` - The type of values stored in the cache. Must implement `Clone + Debug + Send + Sync + 'static`
///
/// # Examples
///
/// ```rust
/// use lrust_cache::ShardedLruCache;
///
/// let cache = ShardedLruCache::new(1000);
/// cache.put("key1".to_string(), "value1".to_string());
/// assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
/// ```
pub struct ShardedLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    shards: Vec<Mutex<BasicLruCache<K, V>>>,
    total_capacity: usize,
    num_shards: usize, // Actual number of shards in use
}

impl<K, V> ShardedLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    /// Creates a new sharded LRU cache with the specified total capacity.
    ///
    /// The number of shards is automatically determined based on the capacity,
    /// ensuring that each shard has at least `MIN_SHARD_CAPACITY` entries
    /// and the total number of shards is a power of 2 not exceeding `MAX_SHARDS`.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The total capacity of the cache
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be positive");

        // Calculate appropriate number of shards
        let theoretical_shards = capacity / MIN_SHARD_CAPACITY;
        let num_shards = if theoretical_shards >= MAX_SHARDS {
            MAX_SHARDS
        } else {
            let mut n = 1;
            while n * 2 <= theoretical_shards && n < MAX_SHARDS {
                n *= 2;
            }
            n
        };

        let base_shard_capacity = capacity / num_shards;

        let remaining_capacity = capacity % num_shards;

        let mut shards = Vec::with_capacity(num_shards);

        for i in 0..num_shards {
            let shard_capacity = if i < remaining_capacity {
                base_shard_capacity + 1
            } else {
                base_shard_capacity
            };
            shards.push(Mutex::new(BasicLruCache::new(shard_capacity)));
        }

        Self {
            shards,
            total_capacity: capacity,
            num_shards,
        }
    }

    /// Returns the total capacity of the cache.
    pub fn capacity(&self) -> usize {
        self.total_capacity
    }

    /// Returns the number of shards in the cache.
    pub fn num_shards(&self) -> usize {
        self.num_shards
    }

    // Internal method to determine which shard a key belongs to
    fn get_shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        (hash as usize) & (self.num_shards - 1)
    }

    /// Retrieves a value from the cache by its key.
    ///
    /// If the key exists, the value is cloned and returned, and the entry
    /// is marked as most recently used.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to look up
    ///
    /// # Returns
    ///
    /// * `Some(V)` if the key exists
    /// * `None` if the key doesn't exist
    pub fn get(&self, key: &K) -> Option<V> {
        let shard_idx = self.get_shard_index(key);
        let shard = self.shards[shard_idx].lock();
        shard.deref().get(key)
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the key already exists, the value is updated and the old value
    /// is returned. If the cache is at capacity, the least recently used
    /// entry is removed to make space.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to insert
    /// * `value` - The value to insert
    ///
    /// # Returns
    ///
    /// * `Some(V)` if the key already existed (returns the old value)
    /// * `None` if the key didn't exist
    pub fn put(&self, key: K, value: V) -> Option<V> {
        let shard_idx = self.get_shard_index(&key);
        let shard = self.shards[shard_idx].lock();
        shard.deref().put(key, value)
    }

    /// Removes an entry from the cache by its key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key to remove
    ///
    /// # Returns
    ///
    /// * `Some(V)` if the key existed (returns the removed value)
    /// * `None` if the key didn't exist
    pub fn remove(&self, key: &K) -> Option<V> {
        let shard_idx = self.get_shard_index(key);
        let shard = self.shards[shard_idx].lock();
        shard.deref().remove(key)
    }

    /// Returns the number of entries in the cache.
    pub fn len(&self) -> usize {
        self.shards
            .iter()
            .map(|shard| shard.lock().deref().len())
            .sum()
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.shards
            .iter()
            .all(|shard| shard.lock().deref().is_empty())
    }

    /// Removes all entries from the cache.
    pub fn clear(&self) {
        for shard in &self.shards {
            let shard = shard.lock();
            shard.deref().clear();
        }
    }
}

impl<K, V> Cache<K, V> for ShardedLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key)
    }

    fn put(&self, key: K, value: V) -> Option<V> {
        self.put(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.remove(key)
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&self) {
        self.clear()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex as PLMutex;
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_operations() {
        let cache = ShardedLruCache::new(2);

        // Test empty cache
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        assert_eq!(cache.put("key1".to_string(), "one".to_string()), None);
        assert_eq!(cache.put("key2".to_string(), "two".to_string()), None);

        // Test non-empty cache
        assert!(!cache.is_empty());
        assert_eq!(cache.len(), 2);

        assert_eq!(cache.get(&"key1".to_string()), Some("one".to_string()));
        assert_eq!(cache.get(&"key2".to_string()), Some("two".to_string()));

        // Verify capacity limit
        cache.put("key3".to_string(), "three".to_string());
        println!("cache.len(): {}", cache.len());
        println!("cache.capacity(): {}", cache.capacity());
        assert!(cache.len() <= cache.capacity());
    }

    #[test]
    fn test_concurrent_access() {
        let cache = Arc::new(ShardedLruCache::new(1000));
        let mut handles = vec![];

        // Create multiple threads for concurrent access
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    cache.put(key.clone(), format!("value_{}", j));
                    thread::sleep(Duration::from_micros(1));
                    if let Some(value) = cache.get(&key) {
                        assert_eq!(value, format!("value_{}", j));
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cache.len() <= cache.capacity());
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let cache = Arc::new(ShardedLruCache::new(2000));
        let mut handles = vec![];
        let operation_count = 1000;

        // Create writer threads
        for i in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..operation_count {
                    let key = format!("key_{}", j % 100);
                    cache.put(key.clone(), format!("writer_{}_value_{}", i, j));
                }
            });
            handles.push(handle);
        }

        // Create reader threads
        for _i in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..operation_count {
                    let key = format!("key_{}", j % 100);
                    if let Some(value) = cache.get(&key) {
                        assert!(
                            value.starts_with("writer_") || value.starts_with("mixed_"),
                            "Invalid value format: {}",
                            value
                        );
                    }
                }
            });
            handles.push(handle);
        }

        // Create mixed operation threads
        for i in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..operation_count {
                    let key = format!("key_{}", j % 100);
                    if j % 2 == 0 {
                        cache.put(key.clone(), format!("mixed_{}_value_{}", i, j));
                    } else {
                        let _ = cache.get(&key);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        assert!(cache.len() <= cache.capacity());
    }

    #[test]
    fn test_concurrent_capacity_correctness() {
        let capacity = 100;
        let cache = Arc::new(ShardedLruCache::new(capacity));
        let threads_count = 8;
        let operations_per_thread = 1000;
        let total_ops_counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        // Create multiple threads for concurrent writes
        for i in 0..threads_count {
            let cache = Arc::clone(&cache);
            let ops_counter = Arc::clone(&total_ops_counter);
            let handle = thread::spawn(move || {
                for j in 0..operations_per_thread {
                    let key = format!("key_{}_{}", i, j);
                    cache.put(key, j);
                    ops_counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify capacity limit
        assert!(
            cache.len() <= capacity,
            "Cache size {} exceeded capacity {}",
            cache.len(),
            capacity
        );

        // Verify total operations
        assert_eq!(
            total_ops_counter.load(Ordering::SeqCst),
            threads_count * operations_per_thread
        );
    }

    #[test]
    fn test_concurrent_remove_correctness() {
        let cache = Arc::new(ShardedLruCache::new(1000));
        let removed_values = Arc::new(PLMutex::new(HashSet::new()));
        let mut handles = vec![];

        // Pre-fill cache
        for i in 0..100 {
            cache.put(format!("key_{}", i), i);
        }

        // Create writer threads
        for i in 0..4 {
            let cache = Arc::clone(&cache);
            let removed = Arc::clone(&removed_values);
            let handle = thread::spawn(move || {
                for j in 0..25 {
                    let key = format!("key_{}", i * 25 + j);
                    if let Some(value) = cache.remove(&key) {
                        removed.lock().insert(value);
                    }
                }
            });
            handles.push(handle);
        }

        // Create reader threads
        for _ in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for i in 0..100 {
                    let key = format!("key_{}", i);
                    if let Some(value) = cache.get(&key) {
                        // Ensure retrieved value is valid
                        assert!(value >= 0 && value < 100);
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify removed values are unique
        let removed_count = removed_values.lock().len();
        assert!(removed_count > 0, "Should have values removed");
        assert!(
            removed_count <= 100,
            "Removed values should not exceed initial count"
        );
    }

    #[test]
    fn test_concurrent_clear_correctness() {
        let cache = Arc::new(ShardedLruCache::new(1000));
        let mut handles = vec![];

        // Pre-fill cache
        for i in 0..500 {
            cache.put(format!("init_key_{}", i), i);
        }

        // Create clear thread
        let cache_clone = Arc::clone(&cache);
        handles.push(thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            cache_clone.clear();
        }));

        // Create concurrent read/write threads
        for i in 0..4 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    cache.put(key.clone(), j);
                    thread::sleep(Duration::from_micros(10));
                    let _ = cache.get(&key);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final state
        assert!(cache.len() <= cache.capacity());
    }

    #[test]
    fn test_concurrent_shard_distribution() {
        let cache = Arc::new(ShardedLruCache::new(1000));
        let shard_counts = (0..cache.num_shards())
            .map(|_| Arc::new(AtomicUsize::new(0)))
            .collect::<Vec<_>>();
        let mut handles = vec![];

        // Create multiple threads for concurrent writes
        for i in 0..8 {
            let cache = Arc::clone(&cache);
            let shard_counts = shard_counts.clone();
            let handle = thread::spawn(move || {
                for j in 0..100 {
                    let key = format!("key_{}_{}", i, j);
                    let shard_idx = cache.get_shard_index(&key);
                    shard_counts[shard_idx].fetch_add(1, Ordering::SeqCst);
                    cache.put(key.clone(), j);
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify shard distribution
        let total_ops: usize = shard_counts
            .iter()
            .map(|counter| counter.load(Ordering::SeqCst))
            .sum();

        assert_eq!(total_ops, 800); // 8 threads * 100 operations

        // Check if all shards are used
        let unused_shards = shard_counts
            .iter()
            .filter(|counter| counter.load(Ordering::SeqCst) == 0)
            .count();
        assert_eq!(unused_shards, 0, "All shards should be used");
    }
}
