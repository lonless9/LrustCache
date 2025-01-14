use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::mem;
use std::ptr::NonNull;

#[doc(hidden)]
pub(crate) mod private {
    use super::*;

    #[allow(dead_code)]
    pub(crate) trait Cache<K, V>: Send + Sync
    where
        K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
        V: Clone + Debug + Send + Sync + 'static,
    {
        fn get(&self, key: &K) -> Option<V>;
        fn put(&self, key: K, value: V) -> Option<V>;
        fn remove(&self, key: &K) -> Option<V>;
        fn len(&self) -> usize;
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
        fn clear(&self);
    }
}

use private::Cache;

// Internal node structure for the doubly linked list
struct Node<K, V> {
    key: K,
    value: V,
    prev: *mut Node<K, V>,
    next: *mut Node<K, V>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }
    }
}

// Internal doubly linked list implementation
struct DoublyLinkedList<K, V> {
    head: *mut Node<K, V>,
    tail: *mut Node<K, V>,
    len: usize,
}

impl<K, V> DoublyLinkedList<K, V> {
    fn new() -> Self {
        Self {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            len: 0,
        }
    }

    // Insert node at the front of the list
    fn push_front(&mut self, node: *mut Node<K, V>) {
        unsafe {
            (*node).prev = std::ptr::null_mut();
            (*node).next = self.head;

            if !self.head.is_null() {
                (*self.head).prev = node;
            } else {
                // Empty list case
                self.tail = node;
            }
            self.head = node;
            self.len += 1;
        }
    }

    // Remove specified node
    fn remove(&mut self, node: *mut Node<K, V>) {
        unsafe {
            let prev = (*node).prev;
            let next = (*node).next;

            if !prev.is_null() {
                (*prev).next = next;
            } else {
                self.head = next;
            }

            if !next.is_null() {
                (*next).prev = prev;
            } else {
                self.tail = prev;
            }

            self.len -= 1;
        }
    }

    // Remove node from the back
    fn pop_back(&mut self) -> Option<*mut Node<K, V>> {
        if self.tail.is_null() {
            return None;
        }

        unsafe {
            let old_tail = self.tail;
            let prev = (*old_tail).prev;

            if !prev.is_null() {
                (*prev).next = std::ptr::null_mut();
                self.tail = prev;
            } else {
                self.head = std::ptr::null_mut();
                self.tail = std::ptr::null_mut();
            }

            self.len -= 1;
            Some(old_tail)
        }
    }

    fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    fn is_empty(&self) -> bool {
        self.len == 0
    }

    // Reinsert node at the front of the list
    fn reinsert_front(&mut self, node: *mut Node<K, V>) {
        self.remove(node);
        self.push_front(node);
    }
}

/// A basic LRU cache implementation.
///
/// This implementation uses a combination of a `HashMap` to store the cache entries
/// for O(1) key-value lookups and a doubly linked list for maintaining LRU order.
///
/// # Type Parameters
///
/// * `K` - The type of keys used in the cache. Must implement `Clone + Debug + Hash + Eq + Send + Sync + 'static`
/// * `V` - The type of values stored in the cache. Must implement `Clone + Debug + Send + Sync + 'static`
///
/// # Examples
///
/// ```rust
/// use lrust_cache::BasicLruCache;
///
/// let cache = BasicLruCache::new(2);
/// cache.put("key1".to_string(), "value1".to_string());
/// assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
/// ```
pub struct BasicLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    cap: usize,
    list: UnsafeCell<DoublyLinkedList<K, V>>,
    map: UnsafeCell<HashMap<K, NonNull<Node<K, V>>>>,
}

// Manually implement Send and Sync as we ensure thread safety
unsafe impl<K, V> Send for BasicLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
}

unsafe impl<K, V> Sync for BasicLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
}

impl<K, V> BasicLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be positive");
        Self {
            cap: capacity,
            list: UnsafeCell::new(DoublyLinkedList::new()),
            map: UnsafeCell::new(HashMap::with_capacity(capacity)),
        }
    }

    pub fn capacity(&self) -> usize {
        self.cap
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
        unsafe {
            let map = &mut *self.map.get();
            if let Some(entry) = map.get(key) {
                let node_ptr = entry.as_ptr();
                // Move to front of list
                (*self.list.get()).reinsert_front(node_ptr);
                // Clone and return value
                Some((*node_ptr).value.clone())
            } else {
                None
            }
        }
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
        unsafe {
            let map = &mut *self.map.get();

            // 1. Check if this is an update operation
            if let Some(entry) = map.get(&key) {
                let node_ptr = entry.as_ptr();
                let old_value = mem::replace(&mut (*node_ptr).value, value);
                // Move to front of list
                (*self.list.get()).reinsert_front(node_ptr);
                Some(old_value)
            } else {
                // 2. Create new node
                let new_node = Box::new(Node::new(key.clone(), value));
                let node_ptr = Box::into_raw(new_node);

                let list = &mut *self.list.get();

                // 3. Check capacity and remove expired nodes
                while list.len() >= self.cap {
                    if let Some(last_node) = list.pop_back() {
                        map.remove(&(*last_node).key);
                        // Free node memory
                        drop(Box::from_raw(last_node));
                    }
                }

                // 4. Insert new node
                list.push_front(node_ptr);
                map.insert(key, NonNull::new_unchecked(node_ptr));
                None
            }
        }
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
        unsafe {
            let map = &mut *self.map.get();
            if let Some(entry) = map.remove(key) {
                let node_ptr = entry.as_ptr();
                let list = &mut *self.list.get();
                list.remove(node_ptr);
                let value = (*node_ptr).value.clone();
                drop(Box::from_raw(node_ptr));
                Some(value)
            } else {
                None
            }
        }
    }

    /// Returns the number of entries in the cache.
    pub fn len(&self) -> usize {
        unsafe { (*self.list.get()).len }
    }

    /// Returns true if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all entries from the cache.
    pub fn clear(&self) {
        unsafe {
            let map = &mut *self.map.get();
            let list = &mut *self.list.get();
            // Free all nodes
            let mut current = list.head;
            while !current.is_null() {
                let next = (*current).next;
                drop(Box::from_raw(current));
                current = next;
            }
            // Reset list
            list.head = std::ptr::null_mut();
            list.tail = std::ptr::null_mut();
            list.len = 0;
            map.clear();
        }
    }
}

impl<K, V> Cache<K, V> for BasicLruCache<K, V>
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

impl<K, V> Drop for BasicLruCache<K, V>
where
    K: Clone + Debug + Hash + Eq + Send + Sync + 'static,
    V: Clone + Debug + Send + Sync + 'static,
{
    fn drop(&mut self) {
        unsafe {
            let list = &mut *self.list.get();
            // Free all nodes
            let mut current = list.head;
            while !current.is_null() {
                let next = (*current).next;
                drop(Box::from_raw(current));
                current = next;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_basic_operations() {
        let cache = BasicLruCache::new(2);

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
        assert!(cache.len() <= cache.capacity());

        // Verify LRU behavior
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), Some("two".to_string()));
        assert_eq!(cache.get(&"key3".to_string()), Some("three".to_string()));
    }

    #[test]
    fn test_update_existing() {
        let cache = BasicLruCache::new(2);

        cache.put("key1".to_string(), "one".to_string());
        assert_eq!(
            cache.put("key1".to_string(), "new_one".to_string()),
            Some("one".to_string())
        );
        assert_eq!(cache.get(&"key1".to_string()), Some("new_one".to_string()));
    }

    #[test]
    fn test_clear() {
        let cache = BasicLruCache::new(2);

        // Test initial state
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);

        cache.put("key1".to_string(), "one".to_string());
        cache.put("key2".to_string(), "two".to_string());
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
        assert_eq!(cache.get(&"key1".to_string()), None);
        assert_eq!(cache.get(&"key2".to_string()), None);
    }
}
