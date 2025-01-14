#![allow(unused_imports)]
use crate::ShardedLruCache;
use std::ffi::{c_void, CStr, CString};
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::ptr;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct CacheKey(String);

#[no_mangle]
pub extern "C" fn cache_create(capacity: usize) -> *mut c_void {
    let cache = Box::new(ShardedLruCache::<CacheKey, String>::new(capacity));
    Box::into_raw(cache) as *mut c_void
}

#[no_mangle]
pub extern "C" fn cache_destroy(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(ptr as *mut ShardedLruCache<CacheKey, String>);
        }
    }
}

#[no_mangle]
pub extern "C" fn cache_put(ptr: *mut c_void, key: *const c_char, value: *const c_char) -> c_int {
    if ptr.is_null() || key.is_null() || value.is_null() {
        return 0;
    }

    unsafe {
        let cache = &mut *(ptr as *mut ShardedLruCache<CacheKey, String>);
        let key_str = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        let value_str = match CStr::from_ptr(value).to_str() {
            Ok(s) => s,
            Err(_) => return 0,
        };

        cache.put(CacheKey(key_str.to_string()), value_str.to_string());
        1
    }
}

#[no_mangle]
pub extern "C" fn cache_get(ptr: *mut c_void, key: *const c_char) -> *mut c_char {
    if ptr.is_null() || key.is_null() {
        return ptr::null_mut();
    }

    unsafe {
        let cache = &mut *(ptr as *mut ShardedLruCache<CacheKey, String>);
        let key_str = match CStr::from_ptr(key).to_str() {
            Ok(s) => s,
            Err(_) => return ptr::null_mut(),
        };

        match cache.get(&CacheKey(key_str.to_string())) {
            Some(value) => match CString::new(value.as_str()) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => ptr::null_mut(),
            },
            None => ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn cache_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[no_mangle]
pub extern "C" fn cache_len(ptr: *mut c_void) -> usize {
    if ptr.is_null() {
        return 0;
    }
    unsafe {
        let cache = &*(ptr as *mut ShardedLruCache<CacheKey, String>);
        cache.len()
    }
}
