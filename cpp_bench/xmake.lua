add_rules("mode.debug", "mode.release")

add_requires("leveldb")

target("leveldb_bench")
    set_kind("binary")
    add_files("bench/leveldb_bench.cc")
    add_packages("leveldb")
    add_links("pthread", "leveldb")
    set_languages("c++17")

    if is_mode("debug") then
        add_cxflags("-g", "-O0", "-fno-omit-frame-pointer")
    else
        add_cxflags("-O3")
    end

target("cache_bench")
    set_kind("binary")
    add_files("bench/cache_bench.cc")
    add_includedirs("include")
    add_linkdirs("../target/release")
    add_links("lrust_cache", "pthread")
    add_rpathdirs("../target/release")
    set_languages("c++17")

    if is_mode("debug") then
        add_cxflags("-g", "-O0", "-fno-omit-frame-pointer")
    else
        add_cxflags("-O3")
    end 