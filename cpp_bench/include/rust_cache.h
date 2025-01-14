#pragma once

#include <cstddef>
#include <string>

extern "C" {
    void* cache_create(size_t capacity);
    void cache_destroy(void* cache);
    int cache_put(void* cache, const char* key, const char* value);
    char* cache_get(void* cache, const char* key);
    void cache_free_string(char* str);
    size_t cache_len(void* cache);
}

class RustCache {
public:
    RustCache(size_t capacity) {
        cache = cache_create(capacity);
    }

    ~RustCache() {
        if (cache) {
            cache_destroy(cache);
            cache = nullptr;
        }
    }

    bool put(const std::string& key, const std::string& value) {
        return cache_put(cache, key.c_str(), value.c_str()) == 1;
    }

    std::string get(const std::string& key) {
        char* value = cache_get(cache, key.c_str());
        if (!value) {
            return "";
        }
        std::string result(value);
        cache_free_string(value);
        return result;
    }

    size_t size() const {
        return cache_len(cache);
    }

private:
    void* cache;
}; 