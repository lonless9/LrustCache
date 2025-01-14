#!/bin/bash

# Set library path
export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:../target/release

# Build project
xmake build || exit 1

# Run benchmarks
./build/linux/x86_64/release/cache_bench || exit 1
./build/linux/x86_64/release/leveldb_bench || exit 1
