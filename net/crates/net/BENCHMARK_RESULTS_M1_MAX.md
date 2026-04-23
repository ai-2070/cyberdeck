     Running benches/auth_guard.rs (target/release/deps/auth_guard-e28391f2567de2f6)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_threaBenchmarking auth_guard_check_fast_hit/single_threaBenchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (247Benchmarking auth_guard_check_fast_hit/single_threaauth_guard_check_fast_hit/single_thread
                        time:   [20.269 ns 20.290 ns 20.316 ns]
                        thrpt:  [49.222 Melem/s 49.285 Melem/s 49.336 Melem/s]
Found 4 outliers among 50 measurements (8.00%)
  2 (4.00%) high mild
  2 (4.00%) high severe

Benchmarking auth_guard_check_fast_miss/single_threBenchmarking auth_guard_check_fast_miss/single_threBenchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (1.Benchmarking auth_guard_check_fast_miss/single_threauth_guard_check_fast_miss/single_thread
                        time:   [4.0803 ns 4.0897 ns 4.1020 ns]
                        thrpt:  [243.78 Melem/s 244.51 Melem/s 245.08 Melem/s]
Found 4 outliers among 50 measurements (8.00%)
  3 (6.00%) high mild
  1 (2.00%) high severe

Benchmarking auth_guard_check_fast_contended/eight_Benchmarking auth_guard_check_fast_contended/eight_Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 Benchmarking auth_guard_check_fast_contended/eight_auth_guard_check_fast_contended/eight_threads
                        time:   [29.766 ns 29.906 ns 30.052 ns]
                        thrpt:  [33.276 Melem/s 33.438 Melem/s 33.596 Melem/s]
Found 3 outliers among 50 measurements (6.00%)
  2 (4.00%) high mild
  1 (2.00%) high severe

Benchmarking auth_guard_allow_channel/insert: WarmiBenchmarking auth_guard_allow_channel/insert: Collecting 50 samples in estimated 5.0000 s (13M iteratiBenchmarking auth_guard_allow_channel/insert: Analyauth_guard_allow_channel/insert
                        time:   [187.42 ns 199.94 ns 211.59 ns]
                        thrpt:  [4.7262 Melem/s 5.0015 Melem/s 5.3355 Melem/s]
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_hot_hit_ceiling/million_opsBenchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 7.6641 s (2550Benchmarking auth_guard_hot_hit_ceiling/million_opsauth_guard_hot_hit_ceiling/million_ops
                        time:   [3.0000 ms 3.0091 ms 3.0252 ms]
Found 4 outliers among 50 measurements (8.00%)
  4 (8.00%) high severe

     Running benches/cortex.rs (target/release/deps/cortex-2c69ff93417a778e)
Gnuplot not found, using plotters backend
Benchmarking cortex_ingest/tasks_create: Warming upBenchmarking cortex_ingest/tasks_create: Collectingcortex_ingest/tasks_create
                        time:   [265.26 ns 270.68 ns 276.41 ns]
                        thrpt:  [3.6178 Melem/s 3.6944 Melem/s 3.7699 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_ingest/memories_store: Warming Benchmarking cortex_ingest/memories_store: Collecting 100 samples in estimated 5.0008 s (13M iterationBenchmarking cortex_ingest/memories_store: Analyzincortex_ingest/memories_store
                        time:   [403.06 ns 411.03 ns 420.10 ns]
                        thrpt:  [2.3804 Melem/s 2.4329 Melem/s 2.4810 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wBenchmarking cortex_fold_barrier/tasks_create_and_wBenchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0246 s (Benchmarking cortex_fold_barrier/tasks_create_and_wcortex_fold_barrier/tasks_create_and_wait
                        time:   [5.7080 µs 5.7345 µs 5.7613 µs]
                        thrpt:  [173.57 Kelem/s 174.38 Kelem/s 175.19 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_fold_barrier/memories_store_andBenchmarking cortex_fold_barrier/memories_store_andBenchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0013 sBenchmarking cortex_fold_barrier/memories_store_andcortex_fold_barrier/memories_store_and_wait
                        time:   [5.9454 µs 5.9740 µs 6.0036 µs]
                        thrpt:  [166.57 Kelem/s 167.39 Kelem/s 168.20 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: WarmBenchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0004 s (5.0M iterBenchmarking cortex_query/tasks_find_many/100: Analcortex_query/tasks_find_many/100
                        time:   [992.66 ns 994.60 ns 997.03 ns]
                        thrpt:  [100.30 Melem/s 100.54 Melem/s 100.74 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_count_where/100: WaBenchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0005 s (32M iteBenchmarking cortex_query/tasks_count_where/100: Ancortex_query/tasks_count_where/100
                        time:   [154.76 ns 154.89 ns 155.03 ns]
                        thrpt:  [645.05 Melem/s 645.63 Melem/s 646.15 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_find_unique/100: WaBenchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (559M itBenchmarking cortex_query/tasks_find_unique/100: Ancortex_query/tasks_find_unique/100
                        time:   [8.9621 ns 9.0109 ns 9.0902 ns]
                        thrpt:  [11.001 Gelem/s 11.098 Gelem/s 11.158 Gelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0173 s (1.Benchmarking cortex_query/memories_find_many_tag/10cortex_query/memories_find_many_tag/100
                        time:   [4.8594 µs 4.8875 µs 4.9227 µs]
                        thrpt:  [20.314 Melem/s 20.461 Melem/s 20.579 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/memories_count_where/100:Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0003 s (5.4MBenchmarking cortex_query/memories_count_where/100:cortex_query/memories_count_where/100
                        time:   [910.42 ns 914.97 ns 919.73 ns]
                        thrpt:  [108.73 Melem/s 109.29 Melem/s 109.84 Melem/s]
Benchmarking cortex_query/tasks_find_many/1000: WarBenchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0212 s (651k iteBenchmarking cortex_query/tasks_find_many/1000: Anacortex_query/tasks_find_many/1000
                        time:   [7.6750 µs 7.6923 µs 7.7103 µs]
                        thrpt:  [129.70 Melem/s 130.00 Melem/s 130.29 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/1000: WBenchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0028 s (3.2M iBenchmarking cortex_query/tasks_count_where/1000: Acortex_query/tasks_count_where/1000
                        time:   [1.5401 µs 1.5414 µs 1.5428 µs]
                        thrpt:  [648.17 Melem/s 648.77 Melem/s 649.30 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_find_unique/1000: WBenchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (559M iBenchmarking cortex_query/tasks_find_unique/1000: Acortex_query/tasks_find_unique/1000
                        time:   [9.1160 ns 9.1441 ns 9.1678 ns]
                        thrpt:  [109.08 Gelem/s 109.36 Gelem/s 109.70 Gelem/s]
Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.0983 s (9Benchmarking cortex_query/memories_find_many_tag/10cortex_query/memories_find_many_tag/1000
                        time:   [53.085 µs 53.142 µs 53.204 µs]
                        thrpt:  [18.796 Melem/s 18.818 Melem/s 18.838 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_count_where/1000Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0387 s (475Benchmarking cortex_query/memories_count_where/1000cortex_query/memories_count_where/1000
                        time:   [10.390 µs 10.437 µs 10.485 µs]
                        thrpt:  [95.373 Melem/s 95.817 Melem/s 96.245 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) low mild
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_many/10000: WaBenchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.1129 s (30k iteBenchmarking cortex_query/tasks_find_many/10000: Ancortex_query/tasks_find_many/10000
                        time:   [163.79 µs 168.64 µs 173.33 µs]
                        thrpt:  [57.694 Melem/s 59.299 Melem/s 61.053 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_count_where/10000: Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.0781 s (197k Benchmarking cortex_query/tasks_count_where/10000: cortex_query/tasks_count_where/10000
                        time:   [26.065 µs 26.750 µs 27.515 µs]
                        thrpt:  [363.44 Melem/s 373.83 Melem/s 383.65 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_unique/10000: Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (560M Benchmarking cortex_query/tasks_find_unique/10000: cortex_query/tasks_find_unique/10000
                        time:   [8.9599 ns 8.9863 ns 9.0194 ns]
                        thrpt:  [1108.7 Gelem/s 1112.8 Gelem/s 1116.1 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/10Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 6.5577 s (Benchmarking cortex_query/memories_find_many_tag/10cortex_query/memories_find_many_tag/10000
                        time:   [638.44 µs 645.75 µs 652.97 µs]
                        thrpt:  [15.315 Melem/s 15.486 Melem/s 15.663 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking cortex_query/memories_count_where/1000Benchmarking cortex_query/memories_count_where/1000Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.6711 s (40Benchmarking cortex_query/memories_count_where/1000cortex_query/memories_count_where/10000
                        time:   [140.43 µs 141.00 µs 141.66 µs]
                        thrpt:  [70.592 Melem/s 70.923 Melem/s 71.209 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: WarmBenchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0041 s (1.6M iterBenchmarking cortex_snapshot/tasks_encode/100: Analcortex_snapshot/tasks_encode/100
                        time:   [3.1653 µs 3.1815 µs 3.2111 µs]
                        thrpt:  [31.142 Melem/s 31.432 Melem/s 31.592 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: WBenchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0188 s (924k iBenchmarking cortex_snapshot/memories_encode/100: Acortex_snapshot/memories_encode/100
                        time:   [5.4259 µs 5.4297 µs 5.4336 µs]
                        thrpt:  [18.404 Melem/s 18.417 Melem/s 18.430 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5Benchmarking cortex_snapshot/netdb_bundle_encode_bycortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.1573 µs 2.1601 µs 2.1629 µs]
                        thrpt:  [46.234 Melem/s 46.294 Melem/s 46.354 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0091 s (2.Benchmarking cortex_snapshot/netdb_bundle_decode/10cortex_snapshot/netdb_bundle_decode/100
                        time:   [2.2529 µs 2.2560 µs 2.2593 µs]
                        thrpt:  [44.261 Melem/s 44.326 Melem/s 44.386 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_snapshot/tasks_encode/1000: WarBenchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.0765 s (167k iteBenchmarking cortex_snapshot/tasks_encode/1000: Anacortex_snapshot/tasks_encode/1000
                        time:   [30.410 µs 30.476 µs 30.551 µs]
                        thrpt:  [32.732 Melem/s 32.813 Melem/s 32.884 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/memories_encode/1000: Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.1620 s (91k iBenchmarking cortex_snapshot/memories_encode/1000: cortex_snapshot/memories_encode/1000
                        time:   [56.637 µs 56.692 µs 56.749 µs]
                        thrpt:  [17.622 Melem/s 17.639 Melem/s 17.656 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimatedBenchmarking cortex_snapshot/netdb_bundle_encode_bycortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [22.255 µs 22.290 µs 22.327 µs]
                        thrpt:  [44.789 Melem/s 44.863 Melem/s 44.933 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0882 s (1Benchmarking cortex_snapshot/netdb_bundle_decode/10cortex_snapshot/netdb_bundle_decode/1000
                        time:   [26.507 µs 26.542 µs 26.591 µs]
                        thrpt:  [37.606 Melem/s 37.677 Melem/s 37.726 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: WaBenchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 6.1486 s (15k iteBenchmarking cortex_snapshot/tasks_encode/10000: Ancortex_snapshot/tasks_encode/10000
                        time:   [405.13 µs 411.16 µs 416.37 µs]
                        thrpt:  [24.017 Melem/s 24.321 Melem/s 24.684 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking cortex_snapshot/memories_encode/10000:Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 7.7692 s (10k Benchmarking cortex_snapshot/memories_encode/10000:cortex_snapshot/memories_encode/10000
                        time:   [785.04 µs 791.43 µs 798.07 µs]
                        thrpt:  [12.530 Melem/s 12.635 Melem/s 12.738 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_byBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Collecting 100 samples in estimatBenchmarking cortex_snapshot/netdb_bundle_encode_bycortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [304.45 µs 310.37 µs 316.21 µs]
                        thrpt:  [31.625 Melem/s 32.219 Melem/s 32.846 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/10Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 6.2037 s (Benchmarking cortex_snapshot/netdb_bundle_decode/10cortex_snapshot/netdb_bundle_decode/10000
                        time:   [304.67 µs 307.73 µs 310.98 µs]
                        thrpt:  [32.157 Melem/s 32.496 Melem/s 32.822 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking netdb_build/open_both: Warming up for 3.0000 smemory allocation of 67108864 bytes failed
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
error: bench failed, to rerun pass `--bench cortex`

Caused by:
  process didn't exit successfully: `/Users/lazlo/Documents/git/cyberdeck/net/crates/net/target/release/deps/cortex-2c69ff93417a778e --bench` (signal: 6, SIGABRT: process abort signal)
lazlo@Lazlos-MBP net % git reset HEAD^
Unstaged changes after reset:
M	net/crates/net/benches/cortex.rs
lazlo@Lazlos-MBP net % cargo bench --features net,redex,cortex,netdb
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /Users/lazlo/Documents/git/cyberdeck/net/crates/net/bindings/node/Cargo.toml
workspace: /Users/lazlo/Documents/git/cyberdeck/net/crates/net/Cargo.toml
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /Users/lazlo/Documents/git/cyberdeck/net/crates/net/bindings/python/Cargo.toml
workspace: /Users/lazlo/Documents/git/cyberdeck/net/crates/net/Cargo.toml
warning: profiles for the non root package will be ignored, specify profiles at the workspace root:
package:   /Users/lazlo/Documents/git/cyberdeck/net/crates/net/bindings/go/compute-ffi/Cargo.toml
workspace: /Users/lazlo/Documents/git/cyberdeck/net/crates/net/Cargo.toml
   Compiling net v0.5.0 (/Users/lazlo/Documents/git/cyberdeck/net/crates/net)
    Finished `bench` profile [optimized] target(s) in 30.00s
     Running unittests src/lib.rs (target/release/deps/net-b5ed8e9d55ff601b)

running 1094 tests
test adapter::net::batch::tests::test_adaptive_batcher_custom_config ... ignored
test adapter::net::batch::tests::test_adaptive_batcher_default ... ignored
test adapter::net::batch::tests::test_burst_detection ... ignored
test adapter::net::batch::tests::test_debug_format ... ignored
test adapter::net::batch::tests::test_ema_convergence ... ignored
test adapter::net::batch::tests::test_ema_init_does_not_overflow_on_huge_target ... ignored
test adapter::net::batch::tests::test_ema_update_does_not_overflow_on_huge_latency ... ignored
test adapter::net::batch::tests::test_optimal_size_burst_mode ... ignored
test adapter::net::batch::tests::test_optimal_size_latency_pressure ... ignored
test adapter::net::batch::tests::test_optimal_size_normal ... ignored
test adapter::net::batch::tests::test_record_updates_metrics ... ignored
test adapter::net::batch::tests::test_reset_metrics ... ignored
test adapter::net::behavior::api::tests::test_api_endpoint_path_matching ... ignored
test adapter::net::behavior::api::tests::test_api_method_properties ... ignored
test adapter::net::behavior::api::tests::test_api_registry_basic ... ignored
test adapter::net::behavior::api::tests::test_api_registry_query ... ignored
test adapter::net::behavior::api::tests::test_api_registry_version_compatibility ... ignored
test adapter::net::behavior::api::tests::test_api_schema ... ignored
test adapter::net::behavior::api::tests::test_api_version_compatibility ... ignored
test adapter::net::behavior::api::tests::test_request_validation ... ignored
test adapter::net::behavior::api::tests::test_schema_type_validation ... ignored
test adapter::net::behavior::api::tests::test_stats ... ignored
test adapter::net::behavior::capability::tests::announcement_is_expired_table_driven_across_ttl_buckets ... ignored
test adapter::net::behavior::capability::tests::gc_and_index_concurrent_race_is_panic_free_and_does_not_evict_live_entries ... ignored
test adapter::net::behavior::capability::tests::gc_evicts_entries_with_ttl_zero ... ignored
test adapter::net::behavior::capability::tests::gc_respects_ttl_bounds_on_freshly_indexed_entries ... ignored
test adapter::net::behavior::capability::tests::gc_retains_entries_with_max_ttl_no_wraparound ... ignored
test adapter::net::behavior::capability::tests::hop_count_defaults_to_zero ... ignored
test adapter::net::behavior::capability::tests::hop_count_roundtrips_through_serde ... ignored
test adapter::net::behavior::capability::tests::hop_count_zero_omits_key_while_nonzero_keeps_it ... ignored
test adapter::net::behavior::capability::tests::index_reflex_addr_none_when_unset_on_announcement ... ignored
test adapter::net::behavior::capability::tests::index_stores_and_returns_reflex_addr ... ignored
test adapter::net::behavior::capability::tests::max_capability_hops_matches_pingwave_contract ... ignored
test adapter::net::behavior::capability::tests::old_format_without_hop_count_parses_as_zero ... ignored
test adapter::net::behavior::capability::tests::reflex_addr_none_is_omitted_from_wire_bytes ... ignored
test adapter::net::behavior::capability::tests::reflex_addr_roundtrips_through_serde_when_set ... ignored
test adapter::net::behavior::capability::tests::signature_rejects_tampered_payload_even_at_hop_zero ... ignored
test adapter::net::behavior::capability::tests::signature_verifies_across_hop_count_bumps ... ignored
test adapter::net::behavior::capability::tests::signed_payload_stays_compatible_with_pre_hop_count_format ... ignored
test adapter::net::behavior::capability::tests::test_capability_announcement_expiry ... ignored
test adapter::net::behavior::capability::tests::test_capability_filter_matches ... ignored
test adapter::net::behavior::capability::tests::test_capability_index ... ignored
test adapter::net::behavior::capability::tests::test_capability_index_version_handling ... ignored
test adapter::net::behavior::capability::tests::test_capability_requirement_scoring ... ignored
test adapter::net::behavior::capability::tests::test_capability_set_creation ... ignored
test adapter::net::behavior::capability::tests::test_capability_set_serialization ... ignored
test adapter::net::behavior::capability::tests::with_ttl_mutation_round_trips_through_is_expired_and_gc ... ignored
test adapter::net::behavior::context::tests::test_baggage ... ignored
test adapter::net::behavior::context::tests::test_context_child ... ignored
test adapter::net::behavior::context::tests::test_context_creation ... ignored
test adapter::net::behavior::context::tests::test_context_max_hops ... ignored
test adapter::net::behavior::context::tests::test_context_remote ... ignored
test adapter::net::behavior::context::tests::test_context_store ... ignored
test adapter::net::behavior::context::tests::test_context_store_capacity ... ignored
test adapter::net::behavior::context::tests::test_context_store_stats ... ignored
test adapter::net::behavior::context::tests::test_context_timeout ... ignored
test adapter::net::behavior::context::tests::test_propagation_context ... ignored
test adapter::net::behavior::context::tests::test_sampler_always_off ... ignored
test adapter::net::behavior::context::tests::test_sampler_always_on ... ignored
test adapter::net::behavior::context::tests::test_sampler_parent_based ... ignored
test adapter::net::behavior::context::tests::test_span_id ... ignored
test adapter::net::behavior::context::tests::test_span_lifecycle ... ignored
test adapter::net::behavior::context::tests::test_trace_id ... ignored
test adapter::net::behavior::context::tests::test_traceparent ... ignored
test adapter::net::behavior::diff::tests::test_apply_diff ... ignored
test adapter::net::behavior::diff::tests::test_apply_strict_error ... ignored
test adapter::net::behavior::diff::tests::test_bandwidth_savings ... ignored
test adapter::net::behavior::diff::tests::test_compact_diffs ... ignored
test adapter::net::behavior::diff::tests::test_diff_add_model ... ignored
test adapter::net::behavior::diff::tests::test_diff_add_tag ... ignored
test adapter::net::behavior::diff::tests::test_diff_no_changes ... ignored
test adapter::net::behavior::diff::tests::test_diff_remove_tag ... ignored
test adapter::net::behavior::diff::tests::test_diff_serialization ... ignored
test adapter::net::behavior::diff::tests::test_diff_update_memory ... ignored
test adapter::net::behavior::diff::tests::test_diff_update_model_loaded ... ignored
test adapter::net::behavior::diff::tests::test_roundtrip_diff ... ignored
test adapter::net::behavior::diff::tests::test_validate_chain ... ignored
test adapter::net::behavior::loadbalance::tests::test_endpoint ... ignored
test adapter::net::behavior::loadbalance::tests::test_health_status ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_circuit_breaker ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_consistent_hash ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_health ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_least_connections ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_round_robin ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_stats ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_weighted ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_balancer_zone_affinity ... ignored
test adapter::net::behavior::loadbalance::tests::test_load_metrics ... ignored
test adapter::net::behavior::loadbalance::tests::test_no_endpoints_error ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_circuit_breaker_half_open_single_probe ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_consistent_hash_deterministic ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_max_connections_cap_enforced_concurrently ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_nan_load_score_no_panic ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_nan_metrics_no_panic ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_random_f64_never_reaches_one ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_weighted_lc_preserves_fractional_weights ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_weighted_rr_precision_past_f64_mantissa ... ignored
test adapter::net::behavior::loadbalance::tests::test_regression_zone_fallback_respected ... ignored
test adapter::net::behavior::loadbalance::tests::test_required_tags ... ignored
test adapter::net::behavior::loadbalance::tests::test_zone_fallback_true_allows_cross_zone ... ignored
test adapter::net::behavior::metadata::tests::test_capacity_limit ... ignored
test adapter::net::behavior::metadata::tests::test_find_nearby ... ignored
test adapter::net::behavior::metadata::tests::test_find_relays ... ignored
test adapter::net::behavior::metadata::tests::test_location_distance ... ignored
test adapter::net::behavior::metadata::tests::test_metadata_query ... ignored
test adapter::net::behavior::metadata::tests::test_metadata_store_basic ... ignored
test adapter::net::behavior::metadata::tests::test_nat_connectivity ... ignored
test adapter::net::behavior::metadata::tests::test_node_metadata ... ignored
test adapter::net::behavior::metadata::tests::test_stats ... ignored
test adapter::net::behavior::metadata::tests::test_topology_score ... ignored
test adapter::net::behavior::metadata::tests::test_version_conflict ... ignored
test adapter::net::behavior::proximity::tests::test_cleanup ... ignored
test adapter::net::behavior::proximity::tests::test_edge_insert_on_pingwave_receipt ... ignored
test adapter::net::behavior::proximity::tests::test_edge_sweep_removes_stale ... ignored
test adapter::net::behavior::proximity::tests::test_enhanced_pingwave_roundtrip ... ignored
test adapter::net::behavior::proximity::tests::test_latency_ewma_smooths_successive_samples ... ignored
test adapter::net::behavior::proximity::tests::test_origin_self_check_drops_pingwave ... ignored
test adapter::net::behavior::proximity::tests::test_pingwave_forward ... ignored
test adapter::net::behavior::proximity::tests::test_primary_capabilities_roundtrip ... ignored
test adapter::net::behavior::proximity::tests::test_proximity_graph_find_matching ... ignored
test adapter::net::behavior::proximity::tests::test_proximity_graph_pingwave_processing ... ignored
test adapter::net::behavior::proximity::tests::test_proximity_graph_to_endpoints ... ignored
test adapter::net::behavior::proximity::tests::test_regression_find_best_no_panic_on_nan ... ignored
test adapter::net::behavior::proximity::tests::test_regression_hop_count_saturates ... ignored
test adapter::net::behavior::proximity::tests::test_regression_pingwave_primary_caps_survive_roundtrip ... ignored
test adapter::net::behavior::proximity::tests::test_routing_score ... ignored
test adapter::net::behavior::rules::tests::test_action_chain ... ignored
test adapter::net::behavior::rules::tests::test_compare_op ... ignored
test adapter::net::behavior::rules::tests::test_condition ... ignored
test adapter::net::behavior::rules::tests::test_condition_expr ... ignored
test adapter::net::behavior::rules::tests::test_disabled_rule ... ignored
test adapter::net::behavior::rules::tests::test_nested_field_access ... ignored
test adapter::net::behavior::rules::tests::test_rule ... ignored
test adapter::net::behavior::rules::tests::test_rule_engine ... ignored
test adapter::net::behavior::rules::tests::test_rule_set ... ignored
test adapter::net::behavior::rules::tests::test_rules_by_tag ... ignored
test adapter::net::behavior::rules::tests::test_stats ... ignored
test adapter::net::behavior::rules::tests::test_stop_on_match ... ignored
test adapter::net::behavior::safety::tests::test_audit_entries ... ignored
test adapter::net::behavior::safety::tests::test_audit_only_mode ... ignored
test adapter::net::behavior::safety::tests::test_concurrent_limit ... ignored
test adapter::net::behavior::safety::tests::test_content_policy_block_patterns ... ignored
test adapter::net::behavior::safety::tests::test_content_policy_max_size ... ignored
test adapter::net::behavior::safety::tests::test_default_envelope ... ignored
test adapter::net::behavior::safety::tests::test_disabled_mode ... ignored
test adapter::net::behavior::safety::tests::test_kill_switch ... ignored
test adapter::net::behavior::safety::tests::test_rate_limiting ... ignored
test adapter::net::behavior::safety::tests::test_regression_check_tokens_overflow_is_rejected ... ignored
test adapter::net::behavior::safety::tests::test_regression_release_decrements_tokens_and_cost ... ignored
test adapter::net::behavior::safety::tests::test_regression_update_tokens_no_underflow ... ignored
test adapter::net::behavior::safety::tests::test_resource_acquisition ... ignored
test adapter::net::behavior::safety::tests::test_safety_enforcer_check_passes ... ignored
test adapter::net::behavior::safety::tests::test_usage_stats ... ignored
test adapter::net::channel::config::tests::test_capability_restricted_channel ... ignored
test adapter::net::channel::config::tests::test_caps_and_token_combined ... ignored
test adapter::net::channel::config::tests::test_config_registry ... ignored
test adapter::net::channel::config::tests::test_open_channel ... ignored
test adapter::net::channel::config::tests::test_regression_config_registry_get_returns_none_on_collision ... ignored
test adapter::net::channel::config::tests::test_regression_config_registry_hash_collision_no_overwrite ... ignored
test adapter::net::channel::config::tests::test_regression_remove_by_hash_returns_none_on_collision ... ignored
test adapter::net::channel::config::tests::test_remove_by_hash_works_when_unique ... ignored
test adapter::net::channel::config::tests::test_token_required_channel ... ignored
test adapter::net::channel::config::tests::test_visibility_default ... ignored
test adapter::net::channel::guard::tests::concurrent_allow_and_revoke_channel_on_same_key_is_panic_free ... ignored
test adapter::net::channel::guard::tests::concurrent_authorize_and_revoke_on_same_key_is_panic_free ... ignored
test adapter::net::channel::guard::tests::test_authorize_then_allow ... ignored
test adapter::net::channel::guard::tests::test_bloom_false_positive_rate ... ignored
test adapter::net::channel::guard::tests::test_different_pair_denied ... ignored
test adapter::net::channel::guard::tests::test_empty_guard_denies ... ignored
test adapter::net::channel::guard::tests::test_is_authorized ... ignored
test adapter::net::channel::guard::tests::test_multiple_authorizations ... ignored
test adapter::net::channel::guard::tests::test_rebuild_bloom_after_revoke ... ignored
test adapter::net::channel::guard::tests::test_regression_channel_hash_collision_distinguishable_by_exact_name ... ignored
test adapter::net::channel::guard::tests::test_regression_concurrent_authorize_and_check ... ignored
test adapter::net::channel::guard::tests::test_regression_u64_origin_hash_defeats_32bit_collision ... ignored
test adapter::net::channel::guard::tests::test_revoke ... ignored
test adapter::net::channel::membership::tests::test_decode_ack_strict_boolean_rejects_non_01 ... ignored
test adapter::net::channel::membership::tests::test_decode_empty_fails ... ignored
test adapter::net::channel::membership::tests::test_decode_invalid_channel_name ... ignored
test adapter::net::channel::membership::tests::test_decode_overflow_name_len ... ignored
test adapter::net::channel::membership::tests::test_decode_truncated_subscribe ... ignored
test adapter::net::channel::membership::tests::test_decode_unknown_tag ... ignored
test adapter::net::channel::membership::tests::test_decode_zero_name_len_rejected ... ignored
test adapter::net::channel::membership::tests::test_legacy_subscribe_no_trailing_token_len_decodes_as_none ... ignored
test adapter::net::channel::membership::tests::test_regression_subscribe_one_byte_token_len_rejected ... ignored
test adapter::net::channel::membership::tests::test_roundtrip_ack_accepted ... ignored
test adapter::net::channel::membership::tests::test_roundtrip_ack_rejected ... ignored
test adapter::net::channel::membership::tests::test_roundtrip_subscribe_no_token ... ignored
test adapter::net::channel::membership::tests::test_roundtrip_subscribe_with_token ... ignored
test adapter::net::channel::membership::tests::test_roundtrip_unsubscribe ... ignored
test adapter::net::channel::name::tests::test_channel_id ... ignored
test adapter::net::channel::name::tests::test_depth ... ignored
test adapter::net::channel::name::tests::test_hash_deterministic ... ignored
test adapter::net::channel::name::tests::test_hash_differs ... ignored
test adapter::net::channel::name::tests::test_invalid_names ... ignored
test adapter::net::channel::name::tests::test_is_prefix_of ... ignored
test adapter::net::channel::name::tests::test_name_too_long ... ignored
test adapter::net::channel::name::tests::test_registry_basic ... ignored
test adapter::net::channel::name::tests::test_registry_duplicate ... ignored
test adapter::net::channel::name::tests::test_registry_get_by_hash ... ignored
test adapter::net::channel::name::tests::test_registry_remove ... ignored
test adapter::net::channel::name::tests::test_regression_rejects_path_traversal_segments ... ignored
test adapter::net::channel::name::tests::test_valid_names ... ignored
test adapter::net::channel::publisher::tests::test_config_builder ... ignored
test adapter::net::channel::publisher::tests::test_config_defaults ... ignored
test adapter::net::channel::publisher::tests::test_max_inflight_clamp ... ignored
test adapter::net::channel::publisher::tests::test_publisher_new ... ignored
test adapter::net::channel::publisher::tests::test_report_helpers ... ignored
test adapter::net::channel::roster::tests::test_add_and_members ... ignored
test adapter::net::channel::roster::tests::test_channels_for ... ignored
test adapter::net::channel::roster::tests::test_peer_count_and_channel_count ... ignored
test adapter::net::channel::roster::tests::test_regression_concurrent_add_remove_same_channel_no_orphan ... ignored
test adapter::net::channel::roster::tests::test_remove ... ignored
test adapter::net::channel::roster::tests::test_remove_peer_evicts_everywhere ... ignored
test adapter::net::channel::roster::tests::test_remove_peer_unknown_is_noop ... ignored
test adapter::net::compute::bindings::tests::default_is_empty ... ignored
test adapter::net::compute::bindings::tests::empty_bytes_decode_as_empty_ledger ... ignored
test adapter::net::compute::bindings::tests::rejects_invalid_channel_name ... ignored
test adapter::net::compute::bindings::tests::rejects_trailing_garbage ... ignored
test adapter::net::compute::bindings::tests::rejects_truncated_header ... ignored
test adapter::net::compute::bindings::tests::rejects_unknown_token_flag ... ignored
test adapter::net::compute::bindings::tests::roundtrip_multi_binding ... ignored
test adapter::net::compute::daemon_factory::tests::register_and_take_returns_entry_once ... ignored
test adapter::net::compute::daemon_factory::tests::take_missing_returns_none ... ignored
test adapter::net::compute::daemon_factory::tests::test_regression_construct_preserves_entry_for_retry ... ignored
test adapter::net::compute::daemon_factory::tests::test_regression_register_always_uses_keypair_origin ... ignored
test adapter::net::compute::daemon_factory::tests::test_regression_register_fails_on_collision_without_clobbering ... ignored
test adapter::net::compute::daemon_factory::tests::test_regression_register_placeholder_fails_on_collision ... ignored
test adapter::net::compute::fork_group::tests::test_fork_group_spawn ... ignored
test adapter::net::compute::fork_group::tests::test_fork_identities_all_different ... ignored
test adapter::net::compute::fork_group::tests::test_fork_lineage_verifiable ... ignored
test adapter::net::compute::fork_group::tests::test_fork_node_failure_preserves_identity ... ignored
test adapter::net::compute::fork_group::tests::test_fork_node_recovery ... ignored
test adapter::net::compute::fork_group::tests::test_fork_route_event ... ignored
test adapter::net::compute::fork_group::tests::test_fork_scale_down ... ignored
test adapter::net::compute::fork_group::tests::test_fork_scale_up ... ignored
test adapter::net::compute::fork_group::tests::test_fork_zero_rejected ... ignored
test adapter::net::compute::fork_group::tests::test_regression_spread_rejects_when_all_nodes_excluded ... ignored
test adapter::net::compute::host::tests::test_chain_continuity_across_events ... ignored
test adapter::net::compute::host::tests::test_counter_daemon ... ignored
test adapter::net::compute::host::tests::test_echo_daemon ... ignored
test adapter::net::compute::host::tests::test_horizon_updated_before_process ... ignored
test adapter::net::compute::host::tests::test_regression_from_snapshot_rejects_wrong_keypair ... ignored
test adapter::net::compute::host::tests::test_stateful_snapshot_and_restore ... ignored
test adapter::net::compute::host::tests::test_stateless_snapshot_is_none ... ignored
test adapter::net::compute::migration::tests::test_event_buffering ... ignored
test adapter::net::compute::migration::tests::test_migration_phase_progression ... ignored
test adapter::net::compute::migration::tests::test_regression_set_snapshot_rejects_wrong_origin ... ignored
test adapter::net::compute::migration::tests::test_wrong_phase_error ... ignored
test adapter::net::compute::migration_source::tests::test_abort ... ignored
test adapter::net::compute::migration_source::tests::test_buffer_event_no_migration ... ignored
test adapter::net::compute::migration_source::tests::test_buffer_events ... ignored
test adapter::net::compute::migration_source::tests::test_cleanup ... ignored
test adapter::net::compute::migration_source::tests::test_cutover_rejects_writes ... ignored
test adapter::net::compute::migration_source::tests::test_duplicate_snapshot_rejected ... ignored
test adapter::net::compute::migration_source::tests::test_start_snapshot ... ignored
test adapter::net::compute::migration_source::tests::test_start_snapshot_not_found ... ignored
test adapter::net::compute::migration_target::tests::test_abort ... ignored
test adapter::net::compute::migration_target::tests::test_activate_and_complete ... ignored
test adapter::net::compute::migration_target::tests::test_out_of_order_buffering ... ignored
test adapter::net::compute::migration_target::tests::test_regression_activate_prefers_active_over_completed ... ignored
test adapter::net::compute::migration_target::tests::test_regression_activate_target_idempotent_after_ack_loss ... ignored
test adapter::net::compute::migration_target::tests::test_regression_complete_prefers_active_over_completed ... ignored
test adapter::net::compute::migration_target::tests::test_restore_and_replay ... ignored
test adapter::net::compute::migration_target::tests::test_restore_wrong_origin_rejected ... ignored
test adapter::net::compute::orchestrator::tests::test_abort_migration ... ignored
test adapter::net::compute::orchestrator::tests::test_chunk_snapshot_large ... ignored
test adapter::net::compute::orchestrator::tests::test_chunk_snapshot_small ... ignored
test adapter::net::compute::orchestrator::tests::test_duplicate_migration_rejected ... ignored
test adapter::net::compute::orchestrator::tests::test_event_buffering ... ignored
test adapter::net::compute::orchestrator::tests::test_list_migrations ... ignored
test adapter::net::compute::orchestrator::tests::test_reassembler_cancel ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_caps_total_chunks ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_distinct_daemons_coexist ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_end_to_end_forged_chunk_cannot_complete ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_evicts_older_seq_per_daemon ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_rejects_chunk_index_out_of_range ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_rejects_oversized_chunk ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_rejects_total_chunks_mismatch ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_reassembler_rejects_zero_total_chunks ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_wire_decode_rejects_chunk_index_out_of_range ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_wire_decode_rejects_total_chunks_overflow ... ignored
test adapter::net::compute::orchestrator::tests::test_regression_wire_decode_rejects_zero_total_chunks ... ignored
test adapter::net::compute::orchestrator::tests::test_start_migration_local_source ... ignored
test adapter::net::compute::orchestrator::tests::test_start_migration_remote_source ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_encode_rejects_oversized_failure_reason ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_rejects_unknown_failure_code ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_roundtrip_buffered_events ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_roundtrip_chunked_snapshot ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_roundtrip_failed ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_roundtrip_snapshot_ready ... ignored
test adapter::net::compute::orchestrator::tests::test_wire_roundtrip_take_snapshot ... ignored
test adapter::net::compute::registry::tests::test_deliver_not_found ... ignored
test adapter::net::compute::registry::tests::test_duplicate_register_rejected ... ignored
test adapter::net::compute::registry::tests::test_list ... ignored
test adapter::net::compute::registry::tests::test_register_and_deliver ... ignored
test adapter::net::compute::registry::tests::test_snapshot_stateless ... ignored
test adapter::net::compute::registry::tests::test_unregister ... ignored
test adapter::net::compute::registry::tests::with_host_closure_can_mutate_same_shard_without_deadlock ... ignored
test adapter::net::compute::registry::tests::with_host_does_not_block_other_daemons ... ignored
test adapter::net::compute::replica_group::tests::test_deterministic_keypairs ... ignored
test adapter::net::compute::replica_group::tests::test_group_health_dead ... ignored
test adapter::net::compute::replica_group::tests::test_group_id_deterministic ... ignored
test adapter::net::compute::replica_group::tests::test_node_failure_and_replacement ... ignored
test adapter::net::compute::replica_group::tests::test_node_recovery ... ignored
test adapter::net::compute::replica_group::tests::test_route_event ... ignored
test adapter::net::compute::replica_group::tests::test_scale_down ... ignored
test adapter::net::compute::replica_group::tests::test_scale_up ... ignored
test adapter::net::compute::replica_group::tests::test_spawn_group ... ignored
test adapter::net::compute::replica_group::tests::test_zero_replicas_rejected ... ignored
test adapter::net::compute::scheduler::tests::test_can_run_locally ... ignored
test adapter::net::compute::scheduler::tests::test_find_migration_targets ... ignored
test adapter::net::compute::scheduler::tests::test_find_migration_targets_excludes_source ... ignored
test adapter::net::compute::scheduler::tests::test_find_subprotocol_nodes ... ignored
test adapter::net::compute::scheduler::tests::test_local_preferred ... ignored
test adapter::net::compute::scheduler::tests::test_no_candidate ... ignored
test adapter::net::compute::scheduler::tests::test_pin ... ignored
test adapter::net::compute::scheduler::tests::test_place_migration ... ignored
test adapter::net::compute::scheduler::tests::test_place_migration_no_targets ... ignored
test adapter::net::compute::scheduler::tests::test_place_migration_prefers_local ... ignored
test adapter::net::compute::scheduler::tests::test_remote_when_local_insufficient ... ignored
test adapter::net::compute::standby_group::tests::test_active_origin_delivers ... ignored
test adapter::net::compute::standby_group::tests::test_deterministic_identity ... ignored
test adapter::net::compute::standby_group::tests::test_minimum_two_members ... ignored
test adapter::net::compute::standby_group::tests::test_node_recovery ... ignored
test adapter::net::compute::standby_group::tests::test_on_node_failure_active ... ignored
test adapter::net::compute::standby_group::tests::test_on_node_failure_standby_only ... ignored
test adapter::net::compute::standby_group::tests::test_promote_on_active_failure ... ignored
test adapter::net::compute::standby_group::tests::test_spawn_standby_group ... ignored
test adapter::net::compute::standby_group::tests::test_sync_standbys ... ignored
test adapter::net::config::tests::test_builder_methods ... ignored
test adapter::net::config::tests::test_initiator_config ... ignored
test adapter::net::config::tests::test_initiator_missing_pubkey ... ignored
test adapter::net::config::tests::test_invalid_timeout_order ... ignored
test adapter::net::config::tests::test_reliability_config ... ignored
test adapter::net::config::tests::test_responder_config ... ignored
test adapter::net::config::tests::test_responder_missing_keypair ... ignored
test adapter::net::config::tests::test_zero_num_shards_rejected ... ignored
test adapter::net::contested::correlation::tests::test_broad_outage ... ignored
test adapter::net::contested::correlation::tests::test_clear_window ... ignored
test adapter::net::contested::correlation::tests::test_independent_failures ... ignored
test adapter::net::contested::correlation::tests::test_mass_failure ... ignored
test adapter::net::contested::correlation::tests::test_no_subnet_data ... ignored
test adapter::net::contested::correlation::tests::test_subnet_correlated ... ignored
test adapter::net::contested::partition::tests::test_confirm ... ignored
test adapter::net::contested::partition::tests::test_detect_partition ... ignored
test adapter::net::contested::partition::tests::test_duplicate_recovery_ignored ... ignored
test adapter::net::contested::partition::tests::test_healing ... ignored
test adapter::net::contested::partition::tests::test_healing_progress ... ignored
test adapter::net::contested::partition::tests::test_no_partition_for_broad_outage ... ignored
test adapter::net::contested::partition::tests::test_no_partition_for_independent ... ignored
test adapter::net::contested::partition::tests::test_take_healed ... ignored
test adapter::net::contested::reconcile::tests::test_already_converged ... ignored
test adapter::net::contested::reconcile::tests::test_catchup_they_are_behind ... ignored
test adapter::net::contested::reconcile::tests::test_catchup_we_are_behind ... ignored
test adapter::net::contested::reconcile::tests::test_conflict_longest_wins ... ignored
test adapter::net::contested::reconcile::tests::test_conflict_they_win ... ignored
test adapter::net::contested::reconcile::tests::test_conflict_tiebreak_deterministic ... ignored
test adapter::net::contested::reconcile::tests::test_regression_reconcile_rejects_broken_remote_chain ... ignored
test adapter::net::contested::reconcile::tests::test_regression_reconcile_rejects_foreign_origin_chain ... ignored
test adapter::net::contested::reconcile::tests::test_regression_tiebreak_perspective_independent ... ignored
test adapter::net::contested::reconcile::tests::test_regression_verify_remote_chain_rejects_origin_forgery ... ignored
test adapter::net::contested::reconcile::tests::test_regression_verify_remote_chain_rejects_wrong_start_sequence ... ignored
test adapter::net::contested::reconcile::tests::test_verify_remote_chain_broken ... ignored
test adapter::net::contested::reconcile::tests::test_verify_remote_chain_empty_is_ok ... ignored
test adapter::net::contested::reconcile::tests::test_verify_remote_chain_valid ... ignored
test adapter::net::continuity::chain::tests::test_assess_continuous ... ignored
test adapter::net::continuity::chain::tests::test_assess_empty_log ... ignored
test adapter::net::continuity::chain::tests::test_proof_from_empty_log ... ignored
test adapter::net::continuity::chain::tests::test_proof_roundtrip ... ignored
test adapter::net::continuity::chain::tests::test_proof_verify_against_same_log ... ignored
test adapter::net::continuity::chain::tests::test_proof_verify_wrong_origin ... ignored
test adapter::net::continuity::cone::tests::test_approximate_horizon ... ignored
test adapter::net::continuity::cone::tests::test_concurrent_events ... ignored
test adapter::net::continuity::cone::tests::test_exact_horizon ... ignored
test adapter::net::continuity::cone::tests::test_merge_cones ... ignored
test adapter::net::continuity::cone::tests::test_merge_empty ... ignored
test adapter::net::continuity::cone::tests::test_regression_merge_mixed_exact_approximate_not_exact ... ignored
test adapter::net::continuity::cone::tests::test_same_entity_ordering ... ignored
test adapter::net::continuity::discontinuity::tests::test_fork_entity ... ignored
test adapter::net::continuity::discontinuity::tests::test_fork_record_roundtrip ... ignored
test adapter::net::continuity::discontinuity::tests::test_fork_record_with_snapshot_seq ... ignored
test adapter::net::continuity::discontinuity::tests::test_fork_sentinel_deterministic ... ignored
test adapter::net::continuity::discontinuity::tests::test_fork_sentinel_differs ... ignored
test adapter::net::continuity::discontinuity::tests::test_tampered_sentinel_fails_verification ... ignored
test adapter::net::continuity::observation::tests::test_divergence_different ... ignored
test adapter::net::continuity::observation::tests::test_divergence_identical ... ignored
test adapter::net::continuity::observation::tests::test_is_within_cone ... ignored
test adapter::net::continuity::observation::tests::test_observe_and_staleness ... ignored
test adapter::net::continuity::observation::tests::test_observe_with_context ... ignored
test adapter::net::continuity::observation::tests::test_reachable_entities ... ignored
test adapter::net::continuity::propagation::tests::test_calibrate ... ignored
test adapter::net::continuity::propagation::tests::test_crossing_depth_different_region ... ignored
test adapter::net::continuity::propagation::tests::test_crossing_depth_global ... ignored
test adapter::net::continuity::propagation::tests::test_crossing_depth_same ... ignored
test adapter::net::continuity::propagation::tests::test_crossing_depth_sibling ... ignored
test adapter::net::continuity::propagation::tests::test_estimate_latency_cross_region ... ignored
test adapter::net::continuity::propagation::tests::test_estimate_latency_same_subnet ... ignored
test adapter::net::continuity::propagation::tests::test_max_depth_within ... ignored
test adapter::net::continuity::superposition::tests::test_auto_ready_to_collapse ... ignored
test adapter::net::continuity::superposition::tests::test_continuity_proof ... ignored
test adapter::net::continuity::superposition::tests::test_lifecycle ... ignored
test adapter::net::continuity::superposition::tests::test_replay_gap ... ignored
test adapter::net::cortex::adapter::tests::test_close_stops_fold_task ... ignored
test adapter::net::cortex::adapter::tests::test_log_and_continue_skips_errors ... ignored
test adapter::net::cortex::adapter::tests::test_open_ingest_wait_query ... ignored
test adapter::net::cortex::adapter::tests::test_stop_policy_halts_on_first_error ... ignored
test adapter::net::cortex::envelope::tests::test_envelope_into_payload_roundtrip ... ignored
test adapter::net::cortex::envelope::tests::test_tuple_impl ... ignored
test adapter::net::cortex::memories::query::tests::test_composed_tag_and_pinned ... ignored
test adapter::net::cortex::memories::query::tests::test_content_contains_case_insensitive ... ignored
test adapter::net::cortex::memories::query::tests::test_empty_state_queries_empty ... ignored
test adapter::net::cortex::memories::query::tests::test_first_and_exists ... ignored
test adapter::net::cortex::memories::query::tests::test_order_by_created_desc_limit ... ignored
test adapter::net::cortex::memories::query::tests::test_where_all_tags_is_and ... ignored
test adapter::net::cortex::memories::query::tests::test_where_any_tag_is_or ... ignored
test adapter::net::cortex::memories::query::tests::test_where_id_in ... ignored
test adapter::net::cortex::memories::query::tests::test_where_pinned_toggles ... ignored
test adapter::net::cortex::memories::query::tests::test_where_source ... ignored
test adapter::net::cortex::memories::query::tests::test_where_tag_single ... ignored
test adapter::net::cortex::memories::state::tests::test_empty_state ... ignored
test adapter::net::cortex::memories::state::tests::test_pin_split ... ignored
test adapter::net::cortex::meta::tests::test_field_boundaries_isolated ... ignored
test adapter::net::cortex::meta::tests::test_has_flag ... ignored
test adapter::net::cortex::meta::tests::test_nonzero_pad_tolerated_on_read ... ignored
test adapter::net::cortex::meta::tests::test_regression_pad_is_zeroed_on_write ... ignored
test adapter::net::cortex::meta::tests::test_roundtrip_all_fields_distinct ... ignored
test adapter::net::cortex::meta::tests::test_short_slice_returns_none ... ignored
test adapter::net::cortex::meta::tests::test_size_is_twenty ... ignored
test adapter::net::cortex::meta::tests::test_unknown_dispatch_decodes_fine ... ignored
test adapter::net::cortex::meta::tests::test_zero_roundtrip ... ignored
test adapter::net::cortex::tasks::query::tests::test_composed_filters ... ignored
test adapter::net::cortex::tasks::query::tests::test_count_ignores_limit ... ignored
test adapter::net::cortex::tasks::query::tests::test_created_after ... ignored
test adapter::net::cortex::tasks::query::tests::test_created_before ... ignored
test adapter::net::cortex::tasks::query::tests::test_empty_state_queries_return_empty ... ignored
test adapter::net::cortex::tasks::query::tests::test_exists_short_circuits ... ignored
test adapter::net::cortex::tasks::query::tests::test_first_none_when_no_match ... ignored
test adapter::net::cortex::tasks::query::tests::test_first_returns_ordered_head ... ignored
test adapter::net::cortex::tasks::query::tests::test_limit_truncates_after_order ... ignored
test adapter::net::cortex::tasks::query::tests::test_no_filters_returns_all ... ignored
test adapter::net::cortex::tasks::query::tests::test_order_by_created ... ignored
test adapter::net::cortex::tasks::query::tests::test_order_by_id_asc_desc ... ignored
test adapter::net::cortex::tasks::query::tests::test_order_by_updated_desc ... ignored
test adapter::net::cortex::tasks::query::tests::test_title_contains_case_insensitive ... ignored
test adapter::net::cortex::tasks::query::tests::test_updated_after_and_before ... ignored
test adapter::net::cortex::tasks::query::tests::test_where_id_in ... ignored
test adapter::net::cortex::tasks::query::tests::test_where_status_pending ... ignored
test adapter::net::cortex::tasks::state::tests::test_empty_state ... ignored
test adapter::net::cortex::tasks::state::tests::test_queries_filter_by_status ... ignored
test adapter::net::crypto::tests::test_derive_key_uses_cryptographic_prf ... ignored
test adapter::net::crypto::tests::test_fast_cipher_counter_increments ... ignored
test adapter::net::crypto::tests::test_fast_cipher_different_sessions ... ignored
test adapter::net::crypto::tests::test_fast_cipher_in_place ... ignored
test adapter::net::crypto::tests::test_fast_cipher_not_clone ... ignored
test adapter::net::crypto::tests::test_fast_cipher_replay_protection ... ignored
test adapter::net::crypto::tests::test_fast_cipher_roundtrip ... ignored
test adapter::net::crypto::tests::test_fast_cipher_session_keys_integration ... ignored
test adapter::net::crypto::tests::test_fast_cipher_tamper_detection ... ignored
test adapter::net::crypto::tests::test_fast_cipher_wrong_counter ... ignored
test adapter::net::crypto::tests::test_noise_handshake ... ignored
test adapter::net::crypto::tests::test_regression_replay_rejected_at_u64_max_boundary ... ignored
test adapter::net::crypto::tests::test_regression_rx_counter_u64_max_no_wrap ... ignored
test adapter::net::crypto::tests::test_replay_bitmap_rejects_duplicate_counter ... ignored
test adapter::net::crypto::tests::test_replay_commit_rejects_out_of_window_counter ... ignored
test adapter::net::crypto::tests::test_replay_window_tracks_bits_across_slide ... ignored
test adapter::net::crypto::tests::test_static_keypair_generate ... ignored
test adapter::net::failure::tests::test_burst_drops_exactly_burst_length_packets ... ignored
test adapter::net::failure::tests::test_circuit_breaker ... ignored
test adapter::net::failure::tests::test_failure_detector_basic ... ignored
test adapter::net::failure::tests::test_failure_detector_elapsed_based_missed_count ... ignored
test adapter::net::failure::tests::test_failure_detector_failure ... ignored
test adapter::net::failure::tests::test_failure_detector_recovery ... ignored
test adapter::net::failure::tests::test_loss_simulator ... ignored
test adapter::net::failure::tests::test_loss_simulator_burst ... ignored
test adapter::net::failure::tests::test_recovery_manager ... ignored
test adapter::net::failure::tests::test_regression_circuit_breaker_concurrent_transitions ... ignored
test adapter::net::failure::tests::test_regression_loss_simulator_burst_no_underflow ... ignored
test adapter::net::identity::entity::tests::clone_of_public_only_is_public_only ... ignored
test adapter::net::identity::entity::tests::public_only_preserves_identity_queries ... ignored
test adapter::net::identity::entity::tests::public_only_secret_bytes_panics - should panic ... ignored
test adapter::net::identity::entity::tests::public_only_sign_panics - should panic ... ignored
test adapter::net::identity::entity::tests::public_only_try_secret_bytes_returns_read_only ... ignored
test adapter::net::identity::entity::tests::public_only_try_sign_returns_read_only ... ignored
test adapter::net::identity::entity::tests::test_clone_preserves_identity ... ignored
test adapter::net::identity::entity::tests::test_entity_id_display ... ignored
test adapter::net::identity::entity::tests::test_keypair_from_bytes_deterministic ... ignored
test adapter::net::identity::entity::tests::test_keypair_generate ... ignored
test adapter::net::identity::entity::tests::test_node_id_nonzero ... ignored
test adapter::net::identity::entity::tests::test_origin_hash_nonzero ... ignored
test adapter::net::identity::entity::tests::test_sign_verify ... ignored
test adapter::net::identity::entity::tests::test_verify_wrong_key ... ignored
test adapter::net::identity::entity::tests::test_verify_wrong_message ... ignored
test adapter::net::identity::entity::tests::try_sign_on_full_keypair_matches_sign ... ignored
test adapter::net::identity::entity::tests::zeroize_converts_full_to_public_only ... ignored
test adapter::net::identity::entity::tests::zeroize_is_idempotent ... ignored
test adapter::net::identity::envelope::tests::from_bytes_rejects_trailing_garbage ... ignored
test adapter::net::identity::envelope::tests::from_bytes_rejects_truncated ... ignored
test adapter::net::identity::envelope::tests::new_refuses_public_only_source ... ignored
test adapter::net::identity::envelope::tests::opened_keypair_matches_signer_pub ... ignored
test adapter::net::identity::envelope::tests::roundtrip_preserves_all_fields ... ignored
test adapter::net::identity::envelope::tests::seal_open_rejects_replay_at_different_chain_link ... ignored
test adapter::net::identity::envelope::tests::seal_open_rejects_retargeted_envelope ... ignored
test adapter::net::identity::envelope::tests::seal_open_rejects_tampered_ciphertext ... ignored
test adapter::net::identity::envelope::tests::seal_open_rejects_tampered_signature ... ignored
test adapter::net::identity::envelope::tests::seal_open_rejects_wrong_target_key ... ignored
test adapter::net::identity::envelope::tests::seal_open_roundtrip ... ignored
test adapter::net::identity::envelope::tests::wire_size_is_208_bytes ... ignored
test adapter::net::identity::origin::tests::test_origin_stamp_from_entity_id ... ignored
test adapter::net::identity::origin::tests::test_origin_stamp_from_keypair ... ignored
test adapter::net::identity::token::tests::cache_coexists_tokens_of_different_scopes_for_same_channel ... ignored
test adapter::net::identity::token::tests::cache_len_reports_total_tokens_not_slot_count ... ignored
test adapter::net::identity::token::tests::cache_same_scope_reinsert_replaces_not_stacks ... ignored
test adapter::net::identity::token::tests::concurrent_insert_check_evict_is_panic_free ... ignored
test adapter::net::identity::token::tests::delegate_preserves_parent_not_after ... ignored
test adapter::net::identity::token::tests::evict_expired_races_with_check_without_panic ... ignored
test adapter::net::identity::token::tests::from_bytes_rejects_trailing_garbage ... ignored
test adapter::net::identity::token::tests::is_expired_ignores_signature_tampering ... ignored
test adapter::net::identity::token::tests::is_valid_and_is_expired_agree_at_not_after_boundary ... ignored
test adapter::net::identity::token::tests::issue_with_huge_ttl_saturates_rather_than_panics ... ignored
test adapter::net::identity::token::tests::test_channel_filter ... ignored
test adapter::net::identity::token::tests::test_delegation ... ignored
test adapter::net::identity::token::tests::test_delegation_depth_exhausted ... ignored
test adapter::net::identity::token::tests::test_delegation_wrong_signer ... ignored
test adapter::net::identity::token::tests::test_expired_token ... ignored
test adapter::net::identity::token::tests::test_issue_and_verify ... ignored
test adapter::net::identity::token::tests::test_regression_channel_hash_zero_is_not_wildcard ... ignored
test adapter::net::identity::token::tests::test_regression_delegate_rejects_expired_parent ... ignored
test adapter::net::identity::token::tests::test_regression_insert_rejects_tampered_token ... ignored
test adapter::net::identity::token::tests::test_regression_wildcard_fallback_not_blocked_by_expired_channel_token ... ignored
test adapter::net::identity::token::tests::test_serialization_roundtrip ... ignored
test adapter::net::identity::token::tests::test_tampered_token ... ignored
test adapter::net::identity::token::tests::test_token_cache ... ignored
test adapter::net::identity::token::tests::test_token_cache_wildcard ... ignored
test adapter::net::identity::token::tests::test_wildcard_channel ... ignored
test adapter::net::pool::tests::test_concurrent_pool_no_nonce_collision ... ignored
test adapter::net::pool::tests::test_regression_pool_builders_share_tx_counter ... ignored
test adapter::net::pool::tests::test_regression_set_key_drains_stale_builders ... ignored
test adapter::net::pool::tests::test_regression_thread_local_pool_builders_share_tx_counter ... ignored
test adapter::net::pool::tests::test_regression_thread_local_pool_isolation ... ignored
test adapter::net::pool::tests::test_shared_local_pool ... ignored
test adapter::net::pool::tests::test_thread_local_pool_acquire_release ... ignored
test adapter::net::pool::tests::test_thread_local_pool_basic ... ignored
test adapter::net::pool::tests::test_thread_local_pool_batch_refill ... ignored
test adapter::net::pool::tests::test_thread_local_pool_overflow_to_shared ... ignored
test adapter::net::pool::tests::test_thread_local_pool_raii_guard ... ignored
test adapter::net::protocol::tests::test_aad ... ignored
test adapter::net::protocol::tests::test_event_frame_roundtrip ... ignored
test adapter::net::protocol::tests::test_header_roundtrip ... ignored
test adapter::net::protocol::tests::test_header_size ... ignored
test adapter::net::protocol::tests::test_header_validation ... ignored
test adapter::net::protocol::tests::test_nack_payload_rejects_trailing_bytes ... ignored
test adapter::net::protocol::tests::test_nack_payload_roundtrip ... ignored
test adapter::net::protocol::tests::test_packet_flags ... ignored
test adapter::net::protocol::tests::test_read_events_caps_allocation ... ignored
test adapter::net::protocol::tests::test_regression_hop_count_excluded_from_aad ... ignored
test adapter::net::protocol::tests::test_validate_rejects_excessive_event_count ... ignored
test adapter::net::proxy::tests::test_control_packet ... ignored
test adapter::net::proxy::tests::test_forward_result ... ignored
test adapter::net::proxy::tests::test_forward_returns_packet_data ... ignored
test adapter::net::proxy::tests::test_hop_stats ... ignored
test adapter::net::proxy::tests::test_priority_packet ... ignored
test adapter::net::proxy::tests::test_proxy_creation ... ignored
test adapter::net::proxy::tests::test_proxy_forward ... ignored
test adapter::net::proxy::tests::test_proxy_local_delivery ... ignored
test adapter::net::proxy::tests::test_proxy_no_route ... ignored
test adapter::net::proxy::tests::test_proxy_routing ... ignored
test adapter::net::proxy::tests::test_proxy_ttl_expired ... ignored
test adapter::net::redex::entry::tests::test_checksum_does_not_alias_flags ... ignored
test adapter::net::redex::entry::tests::test_entry_size ... ignored
test adapter::net::redex::entry::tests::test_flags_are_preserved ... ignored
test adapter::net::redex::entry::tests::test_heap_roundtrip ... ignored
test adapter::net::redex::entry::tests::test_inline_all_high_bits ... ignored
test adapter::net::redex::entry::tests::test_inline_roundtrip ... ignored
test adapter::net::redex::entry::tests::test_inline_zeros ... ignored
test adapter::net::redex::entry::tests::test_non_inline_entry_reports_no_inline_payload ... ignored
test adapter::net::redex::entry::tests::test_payload_checksum_deterministic ... ignored
test adapter::net::redex::file::tests::test_append_assigns_monotonic_seq ... ignored
test adapter::net::redex::file::tests::test_append_batch_sequential ... ignored
test adapter::net::redex::file::tests::test_append_inline_roundtrip ... ignored
test adapter::net::redex::file::tests::test_append_postcard_roundtrip ... ignored
test adapter::net::redex::file::tests::test_close_rejects_further_append ... ignored
test adapter::net::redex::file::tests::test_close_signals_outstanding_tails ... ignored
test adapter::net::redex::file::tests::test_read_range_empty_when_end_le_start ... ignored
test adapter::net::redex::file::tests::test_read_range_returns_events_in_order ... ignored
test adapter::net::redex::file::tests::test_regression_append_fails_when_base_offset_overflows_u32 ... ignored
test adapter::net::redex::file::tests::test_regression_batch_seq_gap_on_offset_overflow ... ignored
test adapter::net::redex::file::tests::test_regression_offset_to_u32_boundary ... ignored
test adapter::net::redex::file::tests::test_regression_ordered_batch_seq_gap_on_offset_overflow ... ignored
test adapter::net::redex::file::tests::test_retention_count ... ignored
test adapter::net::redex::file::tests::test_retention_respects_payload_slicing ... ignored
test adapter::net::redex::file::tests::test_tail_backfills_then_lives ... ignored
test adapter::net::redex::file::tests::test_tail_boundary_no_dupes_no_drops ... ignored
test adapter::net::redex::file::tests::test_tail_from_mid_sequence ... ignored
test adapter::net::redex::index::tests::test_index_decode_error_skips_entry ... ignored
test adapter::net::redex::index::tests::test_index_from_seq_skips_earlier_events ... ignored
test adapter::net::redex::index::tests::test_index_insert_remove_symmetry ... ignored
test adapter::net::redex::index::tests::test_index_multiple_ops_per_event ... ignored
test adapter::net::redex::index::tests::test_index_populates_from_existing_entries ... ignored
test adapter::net::redex::manager::tests::test_auth_allows_authorized_origin ... ignored
test adapter::net::redex::manager::tests::test_auth_denies_unknown_origin ... ignored
test adapter::net::redex::manager::tests::test_auth_fast_path_alone_does_not_authorize_open_file ... ignored
test adapter::net::redex::manager::tests::test_close_file_rejects_append_on_existing_handle ... ignored
test adapter::net::redex::manager::tests::test_get_file_missing_returns_none ... ignored
test adapter::net::redex::manager::tests::test_open_and_get ... ignored
test adapter::net::redex::manager::tests::test_reopen_returns_same_file ... ignored
test adapter::net::redex::manager::tests::test_sweep_retention_runs_on_all_open_files ... ignored
test adapter::net::redex::retention::tests::test_age_retention_drops_all_when_all_old ... ignored
test adapter::net::redex::retention::tests::test_age_retention_drops_older_than_cutoff ... ignored
test adapter::net::redex::retention::tests::test_age_retention_no_drops_when_all_young ... ignored
test adapter::net::redex::retention::tests::test_age_retention_now_before_any_timestamp_drops_nothing ... ignored
test adapter::net::redex::retention::tests::test_both_count_and_size_takes_larger_drop ... ignored
test adapter::net::redex::retention::tests::test_combined_age_larger_than_count ... ignored
test adapter::net::redex::retention::tests::test_combined_count_and_age_takes_larger_drop ... ignored
test adapter::net::redex::retention::tests::test_count_retention ... ignored
test adapter::net::redex::retention::tests::test_empty_index ... ignored
test adapter::net::redex::retention::tests::test_no_retention_drops_nothing ... ignored
test adapter::net::redex::retention::tests::test_regression_size_retention_counts_index_overhead ... ignored
test adapter::net::redex::retention::tests::test_size_retention ... ignored
test adapter::net::redex::segment::tests::test_append_and_read ... ignored
test adapter::net::redex::segment::tests::test_evict_below_base_is_noop ... ignored
test adapter::net::redex::segment::tests::test_evict_beyond_live_clamps ... ignored
test adapter::net::redex::segment::tests::test_evict_prefix ... ignored
test adapter::net::redex::segment::tests::test_read_out_of_range_returns_none ... ignored
test adapter::net::reliability::tests::test_create_reliability_mode ... ignored
test adapter::net::reliability::tests::test_fire_and_forget ... ignored
test adapter::net::reliability::tests::test_regression_duplicate_seq_zero_rejected ... ignored
test adapter::net::reliability::tests::test_regression_first_received_duplicate_rejected ... ignored
test adapter::net::reliability::tests::test_regression_first_received_large_seq_bounded_by_window ... ignored
test adapter::net::reliability::tests::test_regression_first_received_seq_one_nacks_seq_zero ... ignored
test adapter::net::reliability::tests::test_regression_has_gaps_misses_interior_holes ... ignored
test adapter::net::reliability::tests::test_regression_has_gaps_with_filled_first_slot ... ignored
test adapter::net::reliability::tests::test_regression_on_send_evicts_oldest_when_full ... ignored
test adapter::net::reliability::tests::test_regression_seq_zero_after_higher_seqs_rejected ... ignored
test adapter::net::reliability::tests::test_reliable_stream_duplicate ... ignored
test adapter::net::reliability::tests::test_reliable_stream_fill_gap ... ignored
test adapter::net::reliability::tests::test_reliable_stream_gap ... ignored
test adapter::net::reliability::tests::test_reliable_stream_in_order ... ignored
test adapter::net::reliability::tests::test_reliable_stream_max_retries_exhausted ... ignored
test adapter::net::reliability::tests::test_reliable_stream_nack_bitmap_full_window ... ignored
test adapter::net::reliability::tests::test_reliable_stream_nack_retransmit ... ignored
test adapter::net::reliability::tests::test_reliable_stream_nack_retransmit_full_cycle ... ignored
test adapter::net::reliability::tests::test_reliable_stream_pending ... ignored
test adapter::net::reliability::tests::test_reliable_stream_retransmit_timeout ... ignored
test adapter::net::reliability::tests::test_reliable_stream_too_far_ahead ... ignored
test adapter::net::reroute::tests::test_multiple_routes_through_failed_peer ... ignored
test adapter::net::reroute::tests::test_no_alternate_does_nothing ... ignored
test adapter::net::reroute::tests::test_recovery_restores_original ... ignored
test adapter::net::reroute::tests::test_regression_repeated_failures_preserve_original_next_hop ... ignored
test adapter::net::reroute::tests::test_reroute_on_failure ... ignored
test adapter::net::route::tests::concurrent_add_route_with_metric_converges_on_lowest_metric ... ignored
test adapter::net::route::tests::direct_route_survives_concurrent_worse_indirect_inserts ... ignored
test adapter::net::route::tests::test_add_route_with_metric_preserves_better_direct_route ... ignored
test adapter::net::route::tests::test_lookup_alternate ... ignored
test adapter::net::route::tests::test_lookup_alternate_respects_staleness ... ignored
test adapter::net::route::tests::test_regression_remove_route_if_next_hop_is ... ignored
test adapter::net::route::tests::test_regression_routing_discriminator_survives_magic_collision_node_id ... ignored
test adapter::net::route::tests::test_route_flags_combined ... ignored
test adapter::net::route::tests::test_route_flags_roundtrip ... ignored
test adapter::net::route::tests::test_routing_header_flags ... ignored
test adapter::net::route::tests::test_routing_header_forward ... ignored
test adapter::net::route::tests::test_routing_header_magic_at_offset_zero ... ignored
test adapter::net::route::tests::test_routing_header_rejects_wrong_magic ... ignored
test adapter::net::route::tests::test_routing_header_roundtrip ... ignored
test adapter::net::route::tests::test_routing_table_basic ... ignored
test adapter::net::route::tests::test_routing_table_deactivate ... ignored
test adapter::net::route::tests::test_routing_table_stats ... ignored
test adapter::net::route::tests::test_stream_stats ... ignored
test adapter::net::route::tests::test_sweep_stale_and_staleness_aware_lookup ... ignored
test adapter::net::router::tests::test_fair_scheduler_basic ... ignored
test adapter::net::router::tests::test_fair_scheduler_no_starvation ... ignored
test adapter::net::router::tests::test_fair_scheduler_priority ... ignored
test adapter::net::router::tests::test_fair_scheduler_respects_stream_weight ... ignored
test adapter::net::router::tests::test_regression_fair_scheduler_cleanup_called ... ignored
test adapter::net::router::tests::test_regression_scheduler_sees_streams_added_after_quantum_exhaustion ... ignored
test adapter::net::router::tests::test_router_creation ... ignored
test adapter::net::router::tests::test_router_extracts_stream_id_at_correct_offset ... ignored
test adapter::net::router::tests::test_router_routing_table ... ignored
test adapter::net::routing::tests::test_route_to_shard_distribution ... ignored
test adapter::net::routing::tests::test_route_to_shard_range ... ignored
test adapter::net::routing::tests::test_route_to_shard_zero_shards_panics - should panic ... ignored
test adapter::net::routing::tests::test_stream_id_deterministic ... ignored
test adapter::net::routing::tests::test_stream_id_different_for_different_data ... ignored
test adapter::net::routing::tests::test_stream_id_from_key ... ignored
test adapter::net::session::tests::test_authoritative_grant_clamps_to_window_on_malformed_total_consumed ... ignored
test adapter::net::session::tests::test_authoritative_grant_does_not_clobber_concurrent_acquire ... ignored
test adapter::net::session::tests::test_authoritative_grant_invariant_under_thread_contention ... ignored
test adapter::net::session::tests::test_authoritative_grant_monotonic_ignores_stale ... ignored
test adapter::net::session::tests::test_authoritative_grant_recomputes_from_absolute_consumed ... ignored
test adapter::net::session::tests::test_authoritative_grant_self_heals_lost_grants ... ignored
test adapter::net::session::tests::test_close_stream_removes_state ... ignored
test adapter::net::session::tests::test_evict_idle_streams_timeout_and_cap ... ignored
test adapter::net::session::tests::test_grant_quarantine_does_not_fire_without_close ... ignored
test adapter::net::session::tests::test_open_stream_with_idempotent ... ignored
test adapter::net::session::tests::test_regression_acquire_with_expected_epoch_rejects_after_reopen ... ignored
test adapter::net::session::tests::test_regression_admit_and_seq_atomic_across_reopen_race ... ignored
test adapter::net::session::tests::test_regression_control_seq_isolated_from_user_stream ... ignored
test adapter::net::session::tests::test_regression_guard_drop_after_reopen_does_not_corrupt_new_stream ... ignored
test adapter::net::session::tests::test_regression_no_double_counting_grant_and_refund ... ignored
test adapter::net::session::tests::test_regression_reliable_duplicate_must_not_mint_grant ... ignored
test adapter::net::session::tests::test_regression_stale_grant_quarantined_after_close_reopen ... ignored
test adapter::net::session::tests::test_regression_tx_credit_guard_refunds_on_drop ... ignored
test adapter::net::session::tests::test_rx_credit_emits_authoritative_total_consumed ... ignored
test adapter::net::session::tests::test_rx_credit_window_zero_disables_grants ... ignored
test adapter::net::session::tests::test_session_creation ... ignored
test adapter::net::session::tests::test_session_manager ... ignored
test adapter::net::session::tests::test_session_manager_arc_shares_touch_updates ... ignored
test adapter::net::session::tests::test_session_streams ... ignored
test adapter::net::session::tests::test_session_timeout ... ignored
test adapter::net::session::tests::test_stream_state ... ignored
test adapter::net::session::tests::test_stream_state_refund_restores_credit ... ignored
test adapter::net::session::tests::test_stream_state_refund_saturates_at_u32_max ... ignored
test adapter::net::session::tests::test_stream_state_tx_credit_trips_backpressure ... ignored
test adapter::net::session::tests::test_stream_state_tx_window_zero_is_unbounded ... ignored
test adapter::net::session::tests::test_tx_credit_guard_close_between_acquire_and_drop_no_panic ... ignored
test adapter::net::session::tests::test_tx_credit_guard_commit_suppresses_refund ... ignored
test adapter::net::session::tests::test_tx_credit_guard_forget_leaves_credit_consumed ... ignored
test adapter::net::session::tests::test_tx_credit_guard_stream_closed_variant ... ignored
test adapter::net::state::causal::tests::test_causal_event_framing_roundtrip ... ignored
test adapter::net::state::causal::tests::test_causal_link_roundtrip ... ignored
test adapter::net::state::causal::tests::test_chain_builder ... ignored
test adapter::net::state::causal::tests::test_chain_next ... ignored
test adapter::net::state::causal::tests::test_genesis ... ignored
test adapter::net::state::causal::tests::test_long_chain_integrity ... ignored
test adapter::net::state::causal::tests::test_parent_hash_deterministic ... ignored
test adapter::net::state::causal::tests::test_parent_hash_differs_on_payload_change ... ignored
test adapter::net::state::causal::tests::test_regression_causal_link_wire_size_is_24 ... ignored
test adapter::net::state::causal::tests::test_validate_chain_link ... ignored
test adapter::net::state::causal::tests::test_validate_rejects_bad_parent_hash ... ignored
test adapter::net::state::causal::tests::test_validate_rejects_origin_mismatch ... ignored
test adapter::net::state::causal::tests::test_validate_rejects_sequence_gap ... ignored
test adapter::net::state::horizon::tests::test_empty_horizon ... ignored
test adapter::net::state::horizon::tests::test_encode_nonzero ... ignored
test adapter::net::state::horizon::tests::test_horizon_encode_decode_seq_consistency ... ignored
test adapter::net::state::horizon::tests::test_merge ... ignored
test adapter::net::state::horizon::tests::test_might_contain ... ignored
test adapter::net::state::horizon::tests::test_observe ... ignored
test adapter::net::state::horizon::tests::test_observe_max_only ... ignored
test adapter::net::state::horizon::tests::test_potentially_concurrent ... ignored
test adapter::net::state::horizon::tests::test_regression_seq_encoding_preserves_ordering ... ignored
test adapter::net::state::horizon::tests::test_regression_seq_roundtrip_exact_for_small ... ignored
test adapter::net::state::horizon::tests::test_seq_encoding_boundary_at_1024 ... ignored
test adapter::net::state::horizon::tests::test_seq_encoding_decode_preserves_magnitude ... ignored
test adapter::net::state::horizon::tests::test_seq_encoding_does_not_overflow_u16 ... ignored
test adapter::net::state::log::tests::test_after_query ... ignored
test adapter::net::state::log::tests::test_append_chain ... ignored
test adapter::net::state::log::tests::test_log_index ... ignored
test adapter::net::state::log::tests::test_prune ... ignored
test adapter::net::state::log::tests::test_range_query ... ignored
test adapter::net::state::log::tests::test_regression_duplicate_genesis_rejected ... ignored
test adapter::net::state::log::tests::test_regression_prune_all_then_append ... ignored
test adapter::net::state::log::tests::test_rejects_broken_chain ... ignored
test adapter::net::state::log::tests::test_rejects_wrong_origin ... ignored
test adapter::net::state::snapshot::tests::envelope_open_on_snapshot_without_envelope_returns_none ... ignored
test adapter::net::state::snapshot::tests::envelope_open_rejects_wrong_entity_id ... ignored
test adapter::net::state::snapshot::tests::envelope_roundtrip_seals_and_opens_through_wire ... ignored
test adapter::net::state::snapshot::tests::test_from_bytes_too_short ... ignored
test adapter::net::state::snapshot::tests::test_regression_from_bytes_rejects_sequence_mismatch ... ignored
test adapter::net::state::snapshot::tests::test_snapshot_replaces_older ... ignored
test adapter::net::state::snapshot::tests::test_snapshot_roundtrip ... ignored
test adapter::net::state::snapshot::tests::test_snapshot_store ... ignored
test adapter::net::state::snapshot::tests::v0_bytes_decode_as_v1_with_empty_extras ... ignored
test adapter::net::state::snapshot::tests::v1_rejects_trailing_garbage ... ignored
test adapter::net::state::snapshot::tests::v1_rejects_unknown_version_byte ... ignored
test adapter::net::state::snapshot::tests::v1_roundtrip_preserves_bindings_and_envelope ... ignored
test adapter::net::state::snapshot::tests::v1_without_envelope_uses_single_zero_byte_trailer ... ignored
test adapter::net::subnet::assignment::tests::duplicate_prefix_different_levels_both_apply ... ignored
test adapter::net::subnet::assignment::tests::duplicate_prefix_same_level_later_rule_wins ... ignored
test adapter::net::subnet::assignment::tests::first_tag_wins_within_a_single_rule ... ignored
test adapter::net::subnet::assignment::tests::partial_prefix_on_value_does_not_match ... ignored
test adapter::net::subnet::assignment::tests::rule_order_dependency_later_rule_overwrites_earlier_level_write ... ignored
test adapter::net::subnet::assignment::tests::test_empty_policy ... ignored
test adapter::net::subnet::assignment::tests::test_four_levels ... ignored
test adapter::net::subnet::assignment::tests::test_multi_level ... ignored
test adapter::net::subnet::assignment::tests::test_partial_match ... ignored
test adapter::net::subnet::assignment::tests::test_single_level ... ignored
test adapter::net::subnet::assignment::tests::test_unmatched_tag ... ignored
test adapter::net::subnet::gateway::tests::test_exported_channel ... ignored
test adapter::net::subnet::gateway::tests::test_global_always_forwards ... ignored
test adapter::net::subnet::gateway::tests::test_parent_visible_allows_ancestor ... ignored
test adapter::net::subnet::gateway::tests::test_regression_collision_between_subnet_local_and_global_drops ... ignored
test adapter::net::subnet::gateway::tests::test_stats ... ignored
test adapter::net::subnet::gateway::tests::test_subnet_local_always_drops ... ignored
test adapter::net::subnet::gateway::tests::test_ttl_expired ... ignored
test adapter::net::subnet::gateway::tests::test_unknown_channel_defaults_subnet_local ... ignored
test adapter::net::subnet::id::tests::test_display ... ignored
test adapter::net::subnet::id::tests::test_from_raw ... ignored
test adapter::net::subnet::id::tests::test_full_depth ... ignored
test adapter::net::subnet::id::tests::test_global ... ignored
test adapter::net::subnet::id::tests::test_is_ancestor_of ... ignored
test adapter::net::subnet::id::tests::test_is_sibling ... ignored
test adapter::net::subnet::id::tests::test_mask_for_depth ... ignored
test adapter::net::subnet::id::tests::test_new ... ignored
test adapter::net::subnet::id::tests::test_parent ... ignored
test adapter::net::subprotocol::descriptor::tests::test_capability_tag ... ignored
test adapter::net::subprotocol::descriptor::tests::test_descriptor_compatibility ... ignored
test adapter::net::subprotocol::descriptor::tests::test_descriptor_different_id ... ignored
test adapter::net::subprotocol::descriptor::tests::test_descriptor_display ... ignored
test adapter::net::subprotocol::descriptor::tests::test_descriptor_incompatible ... ignored
test adapter::net::subprotocol::descriptor::tests::test_manifest_entry_roundtrip ... ignored
test adapter::net::subprotocol::descriptor::tests::test_version_display ... ignored
test adapter::net::subprotocol::descriptor::tests::test_version_ordering ... ignored
test adapter::net::subprotocol::descriptor::tests::test_version_satisfies ... ignored
test adapter::net::subprotocol::migration_handler::tests::envelope_keypair_overrides_fallback_placeholder ... ignored
test adapter::net::subprotocol::migration_handler::tests::envelope_present_but_unseal_returns_none_fails_rather_than_fallback ... ignored
test adapter::net::subprotocol::migration_handler::tests::maybe_seal_envelope_propagates_seal_failures ... ignored
test adapter::net::subprotocol::migration_handler::tests::migration_identity_context_has_no_plaintext_secret_field_regression ... ignored
test adapter::net::subprotocol::migration_handler::tests::take_snapshot_seal_failure_emits_migration_failed_reply ... ignored
test adapter::net::subprotocol::migration_handler::tests::test_handle_migration_failed ... ignored
test adapter::net::subprotocol::migration_handler::tests::test_handle_take_snapshot ... ignored
test adapter::net::subprotocol::negotiation::tests::test_from_registry ... ignored
test adapter::net::subprotocol::negotiation::tests::test_manifest_from_bytes_too_short ... ignored
test adapter::net::subprotocol::negotiation::tests::test_manifest_roundtrip ... ignored
test adapter::net::subprotocol::negotiation::tests::test_negotiate_compatible ... ignored
test adapter::net::subprotocol::negotiation::tests::test_negotiate_disjoint ... ignored
test adapter::net::subprotocol::negotiation::tests::test_negotiate_empty ... ignored
test adapter::net::subprotocol::negotiation::tests::test_negotiate_incompatible ... ignored
test adapter::net::subprotocol::negotiation::tests::test_negotiate_partial_overlap ... ignored
test adapter::net::subprotocol::registry::tests::test_capability_filter_for ... ignored
test adapter::net::subprotocol::registry::tests::test_capability_tags ... ignored
test adapter::net::subprotocol::registry::tests::test_empty_registry ... ignored
test adapter::net::subprotocol::registry::tests::test_enrich_capabilities ... ignored
test adapter::net::subprotocol::registry::tests::test_enrich_capabilities_with_defaults ... ignored
test adapter::net::subprotocol::registry::tests::test_enrich_preserves_existing_tags ... ignored
test adapter::net::subprotocol::registry::tests::test_forwarding_only ... ignored
test adapter::net::subprotocol::registry::tests::test_list ... ignored
test adapter::net::subprotocol::registry::tests::test_register_and_lookup ... ignored
test adapter::net::subprotocol::registry::tests::test_register_upgrade ... ignored
test adapter::net::subprotocol::registry::tests::test_unregister ... ignored
test adapter::net::subprotocol::registry::tests::test_with_defaults ... ignored
test adapter::net::subprotocol::stream_window::tests::test_decode_empty_rejected ... ignored
test adapter::net::subprotocol::stream_window::tests::test_decode_oversize_rejected ... ignored
test adapter::net::subprotocol::stream_window::tests::test_decode_truncated_rejected ... ignored
test adapter::net::subprotocol::stream_window::tests::test_endianness_is_little_endian ... ignored
test adapter::net::subprotocol::stream_window::tests::test_round_trip ... ignored
test adapter::net::swarm::tests::test_capabilities_large_strings_capped ... ignored
test adapter::net::swarm::tests::test_capabilities_many_items_capped ... ignored
test adapter::net::swarm::tests::test_capabilities_roundtrip ... ignored
test adapter::net::swarm::tests::test_capability_ad_creates_unknown_node ... ignored
test adapter::net::swarm::tests::test_capability_ad_roundtrip ... ignored
test adapter::net::swarm::tests::test_local_graph_capability_search ... ignored
test adapter::net::swarm::tests::test_local_graph_path_finding ... ignored
test adapter::net::swarm::tests::test_local_graph_pingwave_processing ... ignored
test adapter::net::swarm::tests::test_pingwave_forward ... ignored
test adapter::net::swarm::tests::test_pingwave_roundtrip ... ignored
test adapter::net::tests::test_adapter_creation ... ignored
test adapter::net::tests::test_build_then_process_packet_both_directions ... ignored
test adapter::net::tests::test_build_then_process_packet_roundtrip ... ignored
test adapter::net::tests::test_event_id_gt_edge_cases ... ignored
test adapter::net::tests::test_event_id_gt_numeric_ordering ... ignored
test adapter::net::tests::test_invalid_config ... ignored
test adapter::net::tests::test_poll_shard_cursor_drops_consumed_events ... ignored
test adapter::net::tests::test_process_packet_far_future_counter_rejected ... ignored
test adapter::net::tests::test_process_packet_multi_packet_batch_all_events_arrive ... ignored
test adapter::net::tests::test_process_packet_old_counter_rejected ... ignored
test adapter::net::tests::test_process_packet_rejects_tampered_payload ... ignored
test adapter::net::tests::test_process_packet_rejects_truncated_packet ... ignored
test adapter::net::tests::test_process_packet_rejects_wrong_session_id ... ignored
test adapter::net::tests::test_shard_id_from_stream_id_uses_modulo ... ignored
test adapter::net::transport::tests::test_parsed_packet ... ignored
test adapter::net::transport::tests::test_sender_receiver ... ignored
test adapter::net::transport::tests::test_socket_creation ... ignored
test adapter::net::transport::tests::test_socket_send_recv ... ignored
test adapter::noop::tests::test_noop_counts ... ignored
test adapter::noop::tests::test_noop_poll_empty ... ignored
test adapter::tests::test_arc_adapter ... ignored
test adapter::tests::test_boxed_adapter ... ignored
test adapter::tests::test_noop_adapter ... ignored
test adapter::tests::test_noop_adapter_is_healthy ... ignored
test adapter::tests::test_noop_adapter_name ... ignored
test adapter::tests::test_shard_poll_result_clone ... ignored
test adapter::tests::test_shard_poll_result_debug ... ignored
test adapter::tests::test_shard_poll_result_empty ... ignored
test bus::tests::test_event_bus_basic ... ignored
test bus::tests::test_event_bus_batch_ingest ... ignored
test bus::tests::test_event_bus_with_dynamic_scaling ... ignored
test bus::tests::test_manual_scale_up ... ignored
test bus::tests::test_regression_eventbus_drop_signals_shutdown ... ignored
test bus::tests::test_shard_metrics ... ignored
test bus::tests::test_with_dynamic_scaling_builder ... ignored
test config::tests::test_batch_config_presets ... ignored
test config::tests::test_builder ... ignored
test config::tests::test_builder_enables_scaling_by_default ... ignored
test config::tests::test_builder_without_scaling ... ignored
test config::tests::test_config_validates_scaling_policy ... ignored
test config::tests::test_default_config ... ignored
test config::tests::test_high_throughput_max_shards_no_overflow ... ignored
test config::tests::test_invalid_ring_buffer_capacity ... ignored
test config::tests::test_scaling_enabled_by_default ... ignored
test config::tests::test_scaling_policy_presets ... ignored
test config::tests::test_scaling_policy_validation ... ignored
test config::tests::test_with_dynamic_scaling_respects_num_shards ... ignored
test consumer::filter::tests::from_json_rejects_adversarially_nested_filter ... ignored
test consumer::filter::tests::matches_handles_modest_depth_on_small_stack ... ignored
test consumer::filter::tests::test_and_filter ... ignored
test consumer::filter::tests::test_array_indexing ... ignored
test consumer::filter::tests::test_both_eq_formats_work ... ignored
test consumer::filter::tests::test_complex_filter ... ignored
test consumer::filter::tests::test_empty_and_filter ... ignored
test consumer::filter::tests::test_empty_or_filter ... ignored
test consumer::filter::tests::test_eq_filter ... ignored
test consumer::filter::tests::test_eq_wrapped_filter_deserialization ... ignored
test consumer::filter::tests::test_eq_wrapped_in_and ... ignored
test consumer::filter::tests::test_eq_wrapped_in_not ... ignored
test consumer::filter::tests::test_eq_wrapped_in_or ... ignored
test consumer::filter::tests::test_eq_wrapped_with_boolean_value ... ignored
test consumer::filter::tests::test_eq_wrapped_with_nested_path ... ignored
test consumer::filter::tests::test_eq_wrapped_with_numeric_value ... ignored
test consumer::filter::tests::test_filter_builder ... ignored
test consumer::filter::tests::test_filter_builder_default ... ignored
test consumer::filter::tests::test_filter_builder_multiple_or ... ignored
test consumer::filter::tests::test_filter_builder_single ... ignored
test consumer::filter::tests::test_filter_clone ... ignored
test consumer::filter::tests::test_filter_debug ... ignored
test consumer::filter::tests::test_filter_partial_eq ... ignored
test consumer::filter::tests::test_filter_serialization ... ignored
test consumer::filter::tests::test_json_path_get ... ignored
test consumer::filter::tests::test_json_path_get_invalid_array_index ... ignored
test consumer::filter::tests::test_json_path_get_primitive ... ignored
test consumer::filter::tests::test_nested_path ... ignored
test consumer::filter::tests::test_not_filter ... ignored
test consumer::filter::tests::test_or_filter ... ignored
test consumer::merge::tests::test_composite_cursor_clone ... ignored
test consumer::merge::tests::test_composite_cursor_debug ... ignored
test consumer::merge::tests::test_composite_cursor_default ... ignored
test consumer::merge::tests::test_composite_cursor_empty_encode ... ignored
test consumer::merge::tests::test_composite_cursor_get_nonexistent ... ignored
test consumer::merge::tests::test_composite_cursor_new ... ignored
test consumer::merge::tests::test_composite_cursor_set_overwrites ... ignored
test consumer::merge::tests::test_consume_request_builder ... ignored
test consumer::merge::tests::test_consume_request_clone ... ignored
test consumer::merge::tests::test_consume_request_debug ... ignored
test consumer::merge::tests::test_consume_request_default ... ignored
test consumer::merge::tests::test_consume_request_empty_shards ... ignored
test consumer::merge::tests::test_consume_request_from_string ... ignored
test consumer::merge::tests::test_consume_request_new ... ignored
test consumer::merge::tests::test_consume_request_ordering_none ... ignored
test consumer::merge::tests::test_consume_response_clone ... ignored
test consumer::merge::tests::test_consume_response_debug ... ignored
test consumer::merge::tests::test_consume_response_empty ... ignored
test consumer::merge::tests::test_cursor_encode_decode ... ignored
test consumer::merge::tests::test_cursor_many_shards ... ignored
test consumer::merge::tests::test_cursor_update_from_empty_events ... ignored
test consumer::merge::tests::test_cursor_update_from_events ... ignored
test consumer::merge::tests::test_invalid_cursor ... ignored
test consumer::merge::tests::test_ordering_clone_copy ... ignored
test consumer::merge::tests::test_ordering_debug ... ignored
test consumer::merge::tests::test_ordering_default ... ignored
test consumer::merge::tests::test_ordering_equality ... ignored
test consumer::merge::tests::test_poll_merger_cursor_tracks_returned_events_only ... ignored
test consumer::merge::tests::test_poll_merger_empty_limit ... ignored
test consumer::merge::tests::test_poll_merger_empty_shards ... ignored
test consumer::merge::tests::test_poll_merger_new ... ignored
test consumer::merge::tests::test_poll_merger_pagination_multi_shard ... ignored
test consumer::merge::tests::test_poll_merger_pagination_no_duplicates ... ignored
test consumer::merge::tests::test_poll_merger_pagination_with_ordering ... ignored
test consumer::merge::tests::test_poll_merger_small_limit_many_shards ... ignored
test consumer::merge::tests::test_poll_merger_specific_shards ... ignored
test consumer::merge::tests::test_poll_merger_with_cursor ... ignored
test consumer::merge::tests::test_poll_merger_with_events ... ignored
test consumer::merge::tests::test_poll_merger_with_filter ... ignored
test consumer::merge::tests::test_poll_merger_with_limit ... ignored
test consumer::merge::tests::test_poll_merger_with_ordering ... ignored
test consumer::merge::tests::test_regression_all_events_filtered_returns_cursor ... ignored
test consumer::merge::tests::test_regression_filtered_shards_cursor_advances ... ignored
test consumer::merge::tests::test_regression_poll_merger_filter_does_not_infinite_loop ... ignored
test error::tests::test_adapter_error_is_fatal ... ignored
test error::tests::test_adapter_error_is_retryable ... ignored
test error::tests::test_connection_error_not_retryable ... ignored
test error::tests::test_consumer_error_from_adapter ... ignored
test error::tests::test_error_display ... ignored
test event::tests::test_batch_empty ... ignored
test event::tests::test_batch_new ... ignored
test event::tests::test_event_from_json_value ... ignored
test event::tests::test_event_from_slice ... ignored
test event::tests::test_event_from_str ... ignored
test event::tests::test_event_from_str_invalid ... ignored
test event::tests::test_event_into_inner ... ignored
test event::tests::test_event_into_json_value ... ignored
test event::tests::test_event_into_raw ... ignored
test event::tests::test_event_new ... ignored
test event::tests::test_internal_event_as_bytes ... ignored
test event::tests::test_internal_event_from_value ... ignored
test event::tests::test_internal_event_new ... ignored
test event::tests::test_raw_event_bytes_clone ... ignored
test event::tests::test_raw_event_debug ... ignored
test event::tests::test_raw_event_from_bytes ... ignored
test event::tests::test_raw_event_from_bytes_validated ... ignored
test event::tests::test_raw_event_from_event ... ignored
test event::tests::test_raw_event_from_json_value ... ignored
test event::tests::test_raw_event_from_str ... ignored
test event::tests::test_raw_event_from_value ... ignored
test event::tests::test_raw_event_hash_consistency ... ignored
test event::tests::test_stored_event_new ... ignored
test event::tests::test_stored_event_raw_str_invalid_utf8_returns_err ... ignored
test event::tests::test_stored_event_raw_str_valid_utf8 ... ignored
test event::tests::test_stored_event_serialize_invalid_raw_returns_error ... ignored
test event::tests::test_stored_event_serialize_valid ... ignored
test ffi::mesh::nat_traversal_stub_tests::clear_reflex_override_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::connect_direct_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::nat_type_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::peer_nat_type_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::probe_reflex_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::reclassify_nat_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::reflex_addr_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::set_reflex_override_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::traversal_stats_stub_returns_unsupported ... ignored
test ffi::mesh::nat_traversal_stub_tests::unsupported_code_is_stable ... ignored
test ffi::mesh::tests::hardware_from_json_saturates_overflow_cpu_fields ... ignored
test ffi::mesh::tests::net_mesh_shutdown_runs_even_with_outstanding_arc_refs ... ignored
test ffi::mesh::tests::saturating_u16_cap_clamps_at_u16_max ... ignored
test ffi::tests::test_parse_config_empty ... ignored
test ffi::tests::test_parse_config_invalid_json ... ignored
test ffi::tests::test_parse_config_num_shards_max_valid ... ignored
test ffi::tests::test_parse_config_num_shards_overflow ... ignored
test ffi::tests::test_parse_config_valid ... ignored
test shard::batch::tests::test_adaptive_batch_sizing ... ignored
test shard::batch::tests::test_batch_size_threshold ... ignored
test shard::batch::tests::test_batch_timeout ... ignored
test shard::batch::tests::test_force_flush ... ignored
test shard::batch::tests::test_sequence_numbers ... ignored
test shard::mapper::tests::cooldown_is_enforced_under_concurrent_scale_up ... ignored
test shard::mapper::tests::test_draining_excludes_from_selection ... ignored
test shard::mapper::tests::test_evaluate_scaling_auto_scale_disabled ... ignored
test shard::mapper::tests::test_evaluate_scaling_ignores_draining_shards ... ignored
test shard::mapper::tests::test_evaluate_scaling_in_cooldown ... ignored
test shard::mapper::tests::test_evaluate_scaling_no_scale_down_at_min ... ignored
test shard::mapper::tests::test_evaluate_scaling_no_scale_up_at_max ... ignored
test shard::mapper::tests::test_evaluate_scaling_scale_down_on_underutilized ... ignored
test shard::mapper::tests::test_evaluate_scaling_scale_up_on_high_fill_ratio ... ignored
test shard::mapper::tests::test_evaluate_scaling_scale_up_on_high_latency ... ignored
test shard::mapper::tests::test_metrics_collection ... ignored
test shard::mapper::tests::test_policy_normalize_auto_adjusts_max_shards ... ignored
test shard::mapper::tests::test_policy_normalize_preserves_valid_config ... ignored
test shard::mapper::tests::test_policy_validation ... ignored
test shard::mapper::tests::test_scale_down ... ignored
test shard::mapper::tests::test_scale_down_min_limit ... ignored
test shard::mapper::tests::test_scale_up ... ignored
test shard::mapper::tests::test_scale_up_max_limit ... ignored
test shard::mapper::tests::test_scale_up_max_shards_concurrent ... ignored
test shard::mapper::tests::test_scale_up_overflow_protection ... ignored
test shard::mapper::tests::test_scaling_decision_debug ... ignored
test shard::mapper::tests::test_select_shard_distributes ... ignored
test shard::mapper::tests::test_shard_mapper_adjusts_max_shards_to_initial_count ... ignored
test shard::mapper::tests::test_shard_mapper_creation ... ignored
test shard::mapper::tests::test_shard_mapper_normalizes_policy ... ignored
test shard::mapper::tests::test_shard_metrics_compute_weight ... ignored
test shard::mapper::tests::test_shard_metrics_draining_max_weight ... ignored
test shard::mapper::tests::test_shard_metrics_new ... ignored
test shard::ring_buffer::tests::test_buffer_full_error_debug ... ignored
test shard::ring_buffer::tests::test_buffer_full_error_display ... ignored
test shard::ring_buffer::tests::test_buffer_full_error_is_error ... ignored
test shard::ring_buffer::tests::test_capacity_and_free_slots ... ignored
test shard::ring_buffer::tests::test_capacity_too_small - should panic ... ignored
test shard::ring_buffer::tests::test_concurrent_spsc ... ignored
test shard::ring_buffer::tests::test_drop ... ignored
test shard::ring_buffer::tests::test_is_full ... ignored
test shard::ring_buffer::tests::test_non_power_of_two_capacity - should panic ... ignored
test shard::ring_buffer::tests::test_pop_batch ... ignored
test shard::ring_buffer::tests::test_pop_batch_empty ... ignored
test shard::ring_buffer::tests::test_push_pop ... ignored
test shard::ring_buffer::tests::test_push_pop_at_exact_capacity ... ignored
test shard::ring_buffer::tests::test_push_pop_boundary_stress ... ignored
test shard::ring_buffer::tests::test_regression_spsc_multi_consumer_detected ... ignored
test shard::ring_buffer::tests::test_regression_spsc_multi_producer_detected ... ignored
test shard::ring_buffer::tests::test_wraparound ... ignored
test shard::tests::sample_mode_currently_returns_sampled_after_buffer_fills ... ignored
test shard::tests::test_add_shard_requires_dynamic_scaling ... ignored
test shard::tests::test_backpressure_drop_newest ... ignored
test shard::tests::test_backpressure_drop_oldest ... ignored
test shard::tests::test_drain_shard_requires_dynamic_scaling ... ignored
test shard::tests::test_drop_oldest_counts_dropped_events ... ignored
test shard::tests::test_drop_oldest_multiple_cycles ... ignored
test shard::tests::test_drop_oldest_raw_counts_dropped_events ... ignored
test shard::tests::test_raw_event_ingestion ... ignored
test shard::tests::test_remove_shard_requires_dynamic_scaling ... ignored
test shard::tests::test_shard_manager_ingest ... ignored
test shard::tests::test_shard_manager_routing ... ignored
test shard::tests::test_shard_push_pop ... ignored
test timestamp::tests::test_last_after_next ... ignored
test timestamp::tests::test_monotonicity ... ignored
test timestamp::tests::test_monotonicity_concurrent ... ignored
test timestamp::tests::test_multiple_generators_independent ... ignored
test timestamp::tests::test_next_panics_at_u64_max - should panic ... ignored
test timestamp::tests::test_next_returns_increasing_values ... ignored
test timestamp::tests::test_no_syscall_performance ... ignored
test timestamp::tests::test_now_raw ... ignored
test timestamp::tests::test_now_raw_does_not_affect_last ... ignored
test timestamp::tests::test_rapid_calls ... ignored
test timestamp::tests::test_raw_to_nanos ... ignored
test timestamp::tests::test_raw_to_nanos_zero ... ignored
test timestamp::tests::test_send_sync ... ignored
test timestamp::tests::test_timestamp_generator_default ... ignored
test timestamp::tests::test_timestamp_generator_new ... ignored

test result: ok. 0 passed; 0 failed; 1094 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/auth_guard.rs (target/release/deps/auth_guard-e28391f2567de2f6)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (248M iteauth_guard_check_fast_hit/single_thread
                        time:   [20.041 ns 20.076 ns 20.112 ns]
                        thrpt:  [49.721 Melem/s 49.810 Melem/s 49.897 Melem/s]
                 change:
                        time:   [−1.3619% −1.1268% −0.8962%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9043% +1.1396% +1.3807%]
                        Change within noise threshold.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (1.2B itauth_guard_check_fast_miss/single_thread
                        time:   [4.0690 ns 4.0732 ns 4.0775 ns]
                        thrpt:  [245.25 Melem/s 245.51 Melem/s 245.76 Melem/s]
                 change:
                        time:   [−0.6781% −0.4261% −0.1631%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1633% +0.4279% +0.6828%]
                        Change within noise threshold.
Found 5 outliers among 50 measurements (10.00%)
  2 (4.00%) low mild
  3 (6.00%) high mild

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 s (16auth_guard_check_fast_contended/eight_threads
                        time:   [30.719 ns 30.900 ns 31.084 ns]
                        thrpt:  [32.170 Melem/s 32.363 Melem/s 32.553 Melem/s]
                 change:
                        time:   [+2.8849% +3.5477% +4.1499%] (p = 0.00 < 0.05)
                        thrpt:  [−3.9845% −3.4262% −2.8040%]
                        Performance has regressed.
Found 2 outliers among 50 measurements (4.00%)
  2 (4.00%) high mild

auth_guard_allow_channel/insert
                        time:   [198.61 ns 206.91 ns 214.95 ns]
                        thrpt:  [4.6523 Melem/s 4.8330 Melem/s 5.0350 Melem/s]
                 change:
                        time:   [−0.6129% +5.6314% +12.530%] (p = 0.10 > 0.05)
                        thrpt:  [−11.135% −5.3312% +0.6167%]
                        No change in performance detected.

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 7.7543 s (2550 iterauth_guard_hot_hit_ceiling/million_ops
                        time:   [3.0226 ms 3.0380 ms 3.0584 ms]
                        change: [+0.7634% +1.2956% +1.9106%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

     Running benches/cortex.rs (target/release/deps/cortex-2c69ff93417a778e)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [262.19 ns 269.90 ns 277.80 ns]
                        thrpt:  [3.5998 Melem/s 3.7050 Melem/s 3.8140 Melem/s]
                 change:
                        time:   [−5.2978% −0.5247% +4.5513%] (p = 0.84 > 0.05)
                        thrpt:  [−4.3532% +0.5274% +5.5942%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
cortex_ingest/memories_store
                        time:   [397.14 ns 404.30 ns 411.60 ns]
                        thrpt:  [2.4295 Melem/s 2.4734 Melem/s 2.5180 Melem/s]
                 change:
                        time:   [−4.7771% −1.6892% +1.4959%] (p = 0.31 > 0.05)
                        thrpt:  [−1.4738% +1.7182% +5.0168%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0070 s (874k cortex_fold_barrier/tasks_create_and_wait
                        time:   [5.5892 µs 5.6061 µs 5.6245 µs]
                        thrpt:  [177.79 Kelem/s 178.38 Kelem/s 178.92 Kelem/s]
                 change:
                        time:   [−2.8850% −2.1986% −1.4314%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4521% +2.2480% +2.9707%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0003 s (838cortex_fold_barrier/memories_store_and_wait
                        time:   [5.7800 µs 5.7974 µs 5.8153 µs]
                        thrpt:  [171.96 Kelem/s 172.49 Kelem/s 173.01 Kelem/s]
                 change:
                        time:   [−3.7476% −2.9904% −2.2177%] (p = 0.00 < 0.05)
                        thrpt:  [+2.2680% +3.0825% +3.8935%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0047 s (5.1M iterationcortex_query/tasks_find_many/100
                        time:   [988.87 ns 989.78 ns 990.74 ns]
                        thrpt:  [100.93 Melem/s 101.03 Melem/s 101.13 Melem/s]
                 change:
                        time:   [−0.7659% −0.4978% −0.2503%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2509% +0.5003% +0.7718%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0004 s (33M iteratiocortex_query/tasks_count_where/100
                        time:   [151.15 ns 151.27 ns 151.41 ns]
                        thrpt:  [660.48 Melem/s 661.06 Melem/s 661.59 Melem/s]
                 change:
                        time:   [−2.8407% −2.4663% −2.1875%] (p = 0.00 < 0.05)
                        thrpt:  [+2.2364% +2.5287% +2.9237%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (561M iteraticortex_query/tasks_find_unique/100
                        time:   [8.9099 ns 8.9246 ns 8.9395 ns]
                        thrpt:  [11.186 Gelem/s 11.205 Gelem/s 11.224 Gelem/s]
                 change:
                        time:   [−1.7878% −0.9643% −0.4303%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4322% +0.9737% +1.8204%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0238 s (1.1M itcortex_query/memories_find_many_tag/100
                        time:   [4.7280 µs 4.7472 µs 4.7661 µs]
                        thrpt:  [20.981 Melem/s 21.065 Melem/s 21.151 Melem/s]
                 change:
                        time:   [−3.1982% −2.8476% −2.5130%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5778% +2.9311% +3.3038%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0013 s (5.4M itercortex_query/memories_count_where/100
                        time:   [916.61 ns 921.39 ns 926.28 ns]
                        thrpt:  [107.96 Melem/s 108.53 Melem/s 109.10 Melem/s]
                 change:
                        time:   [+0.4764% +1.0592% +1.6463%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6196% −1.0481% −0.4741%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0282 s (636k iteratiocortex_query/tasks_find_many/1000
                        time:   [7.8857 µs 7.9021 µs 7.9209 µs]
                        thrpt:  [126.25 Melem/s 126.55 Melem/s 126.81 Melem/s]
                 change:
                        time:   [+3.1399% +3.6110% +4.0640%] (p = 0.00 < 0.05)
                        thrpt:  [−3.9053% −3.4852% −3.0443%]
                        Performance has regressed.
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0054 s (3.2M iteratcortex_query/tasks_count_where/1000
                        time:   [1.5294 µs 1.5310 µs 1.5328 µs]
                        thrpt:  [652.41 Melem/s 653.16 Melem/s 653.85 Melem/s]
                 change:
                        time:   [−0.9478% −0.7729% −0.5979%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6015% +0.7789% +0.9569%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (557M iteratcortex_query/tasks_find_unique/1000
                        time:   [8.9034 ns 8.9146 ns 8.9269 ns]
                        thrpt:  [112.02 Gelem/s 112.18 Gelem/s 112.32 Gelem/s]
                 change:
                        time:   [−1.6383% −1.2820% −0.9297%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9384% +1.2987% +1.6656%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.2301 s (106k icortex_query/memories_find_many_tag/1000
                        time:   [49.209 µs 49.263 µs 49.319 µs]
                        thrpt:  [20.276 Melem/s 20.299 Melem/s 20.321 Melem/s]
                 change:
                        time:   [−7.4783% −7.2713% −7.0669%] (p = 0.00 < 0.05)
                        thrpt:  [+7.6043% +7.8415% +8.0827%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0391 s (500k itecortex_query/memories_count_where/1000
                        time:   [10.032 µs 10.046 µs 10.060 µs]
                        thrpt:  [99.404 Melem/s 99.545 Melem/s 99.681 Melem/s]
                 change:
                        time:   [−4.4823% −4.1926% −3.8850%] (p = 0.00 < 0.05)
                        thrpt:  [+4.0421% +4.3761% +4.6926%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.3528 s (30k iteratiocortex_query/tasks_find_many/10000
                        time:   [166.02 µs 171.58 µs 177.20 µs]
                        thrpt:  [56.433 Melem/s 58.283 Melem/s 60.235 Melem/s]
                 change:
                        time:   [−0.7278% +4.1437% +9.1919%] (p = 0.10 > 0.05)
                        thrpt:  [−8.4181% −3.9788% +0.7331%]
                        No change in performance detected.
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.0096 s (212k iteracortex_query/tasks_count_where/10000
                        time:   [22.845 µs 23.320 µs 23.821 µs]
                        thrpt:  [419.79 Melem/s 428.83 Melem/s 437.73 Melem/s]
                 change:
                        time:   [−17.729% −14.867% −11.941%] (p = 0.00 < 0.05)
                        thrpt:  [+13.560% +17.463% +21.550%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (561M iteracortex_query/tasks_find_unique/10000
                        time:   [8.9054 ns 8.9198 ns 8.9366 ns]
                        thrpt:  [1119.0 Gelem/s 1121.1 Gelem/s 1122.9 Gelem/s]
                 change:
                        time:   [−0.7377% −0.3070% +0.0770%] (p = 0.15 > 0.05)
                        thrpt:  [−0.0769% +0.3079% +0.7432%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 6.2602 s (10k icortex_query/memories_find_many_tag/10000
                        time:   [601.61 µs 611.74 µs 622.44 µs]
                        thrpt:  [16.066 Melem/s 16.347 Melem/s 16.622 Melem/s]
                 change:
                        time:   [−5.2323% −2.4315% +0.6034%] (p = 0.13 > 0.05)
                        thrpt:  [−0.5997% +2.4921% +5.5212%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.5470 s (40k itecortex_query/memories_count_where/10000
                        time:   [137.06 µs 137.20 µs 137.34 µs]
                        thrpt:  [72.810 Melem/s 72.885 Melem/s 72.960 Melem/s]
                 change:
                        time:   [−2.7750% −2.4350% −2.1266%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1728% +2.4957% +2.8542%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0030 s (1.6M iterationcortex_snapshot/tasks_encode/100
                        time:   [3.1410 µs 3.1444 µs 3.1478 µs]
                        thrpt:  [31.769 Melem/s 31.803 Melem/s 31.837 Melem/s]
                 change:
                        time:   [−1.3024% −0.8477% −0.5352%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5381% +0.8550% +1.3196%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0168 s (924k iteratcortex_snapshot/memories_encode/100
                        time:   [5.4001 µs 5.4049 µs 5.4098 µs]
                        thrpt:  [18.485 Melem/s 18.502 Melem/s 18.518 Melem/s]
                 change:
                        time:   [−0.5665% −0.4097% −0.2549%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2556% +0.4114% +0.5698%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5.0011cortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.1446 µs 2.1474 µs 2.1503 µs]
                        thrpt:  [46.506 Melem/s 46.568 Melem/s 46.628 Melem/s]
                 change:
                        time:   [−0.6388% −0.4270% −0.2134%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2139% +0.4288% +0.6429%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0076 s (2.2M itcortex_snapshot/netdb_bundle_decode/100
                        time:   [2.2503 µs 2.2536 µs 2.2571 µs]
                        thrpt:  [44.304 Melem/s 44.373 Melem/s 44.438 Melem/s]
                 change:
                        time:   [−0.4130% −0.1880% +0.0346%] (p = 0.10 > 0.05)
                        thrpt:  [−0.0346% +0.1884% +0.4148%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.0379 s (167k iteratiocortex_snapshot/tasks_encode/1000
                        time:   [30.220 µs 30.255 µs 30.290 µs]
                        thrpt:  [33.014 Melem/s 33.052 Melem/s 33.090 Melem/s]
                 change:
                        time:   [−1.1972% −0.8703% −0.5735%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5768% +0.8779% +1.2117%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.1929 s (91k iteratcortex_snapshot/memories_encode/1000
                        time:   [56.912 µs 56.955 µs 56.997 µs]
                        thrpt:  [17.545 Melem/s 17.558 Melem/s 17.571 Melem/s]
                 change:
                        time:   [+0.2545% +0.4944% +0.7217%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7165% −0.4920% −0.2538%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimated 5.04cortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [22.160 µs 22.180 µs 22.200 µs]
                        thrpt:  [45.045 Melem/s 45.085 Melem/s 45.126 Melem/s]
                 change:
                        time:   [−0.7556% −0.5276% −0.3074%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3084% +0.5304% +0.7613%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0799 s (192k icortex_snapshot/netdb_bundle_decode/1000
                        time:   [26.459 µs 26.482 µs 26.506 µs]
                        thrpt:  [37.727 Melem/s 37.762 Melem/s 37.795 Melem/s]
                 change:
                        time:   [−1.7124% −0.6951% −0.0991%] (p = 0.10 > 0.05)
                        thrpt:  [+0.0992% +0.7000% +1.7422%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 5.5750 s (15k iteratiocortex_snapshot/tasks_encode/10000
                        time:   [357.43 µs 363.01 µs 368.21 µs]
                        thrpt:  [27.159 Melem/s 27.548 Melem/s 27.977 Melem/s]
                 change:
                        time:   [−11.350% −8.9486% −6.3787%] (p = 0.00 < 0.05)
                        thrpt:  [+6.8133% +9.8281% +12.803%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 6.9755 s (10k iteracortex_snapshot/memories_encode/10000
                        time:   [656.16 µs 669.41 µs 683.10 µs]
                        thrpt:  [14.639 Melem/s 14.938 Melem/s 15.240 Melem/s]
                 change:
                        time:   [−14.933% −13.223% −11.417%] (p = 0.00 < 0.05)
                        thrpt:  [+12.888% +15.238% +17.555%]
                        Performance has improved.
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Collecting 100 samples in estimated 5.cortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [244.61 µs 248.60 µs 252.92 µs]
                        thrpt:  [39.538 Melem/s 40.225 Melem/s 40.881 Melem/s]
                 change:
                        time:   [−23.268% −21.321% −19.381%] (p = 0.00 < 0.05)
                        thrpt:  [+24.040% +27.099% +30.324%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 6.2153 s (20k icortex_snapshot/netdb_bundle_decode/10000
                        time:   [304.48 µs 308.86 µs 313.36 µs]
                        thrpt:  [31.912 Melem/s 32.378 Melem/s 32.843 Melem/s]
                 change:
                        time:   [−2.3124% −0.4251% +1.6807%] (p = 0.68 > 0.05)
                        thrpt:  [−1.6529% +0.4269% +2.3672%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

     Running benches/ingestion.rs (target/release/deps/ingestion-bbc247d2b33ac693)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [10.891 ns 10.916 ns 10.951 ns]
                        thrpt:  [91.318 Melem/s 91.607 Melem/s 91.816 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
ring_buffer/push_pop/1024
                        time:   [10.385 ns 10.395 ns 10.404 ns]
                        thrpt:  [96.116 Melem/s 96.204 Melem/s 96.290 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
ring_buffer/push/8192   time:   [10.888 ns 10.896 ns 10.903 ns]
                        thrpt:  [91.715 Melem/s 91.778 Melem/s 91.841 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
ring_buffer/push_pop/8192
                        time:   [10.390 ns 10.399 ns 10.408 ns]
                        thrpt:  [96.076 Melem/s 96.164 Melem/s 96.249 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
ring_buffer/push/65536  time:   [10.856 ns 10.867 ns 10.877 ns]
                        thrpt:  [91.937 Melem/s 92.024 Melem/s 92.118 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
ring_buffer/push_pop/65536
                        time:   [10.406 ns 10.416 ns 10.426 ns]
                        thrpt:  [95.915 Melem/s 96.009 Melem/s 96.097 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
ring_buffer/push/1048576
                        time:   [10.247 ns 10.308 ns 10.357 ns]
                        thrpt:  [96.553 Melem/s 97.009 Melem/s 97.586 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) low mild
ring_buffer/push_pop/1048576
                        time:   [10.556 ns 10.572 ns 10.589 ns]
                        thrpt:  [94.434 Melem/s 94.589 Melem/s 94.731 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe

timestamp/next          time:   [7.5063 ns 7.5121 ns 7.5182 ns]
                        thrpt:  [133.01 Melem/s 133.12 Melem/s 133.22 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
timestamp/now_raw       time:   [623.80 ps 624.21 ps 624.63 ps]
                        thrpt:  [1.6009 Gelem/s 1.6020 Gelem/s 1.6031 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

event/internal_event_new
                        time:   [172.65 ns 172.88 ns 173.15 ns]
                        thrpt:  [5.7755 Melem/s 5.7845 Melem/s 5.7922 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
event/json_creation     time:   [105.20 ns 105.31 ns 105.43 ns]
                        thrpt:  [9.4846 Melem/s 9.4954 Melem/s 9.5061 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

batch/pop_batch/100     time:   [775.79 ns 776.54 ns 777.33 ns]
                        thrpt:  [128.65 Melem/s 128.78 Melem/s 128.90 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
batch/pop_batch/1000    time:   [37.396 µs 39.312 µs 41.270 µs]
                        thrpt:  [24.231 Melem/s 25.437 Melem/s 26.741 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
batch/pop_batch/10000   time:   [616.98 µs 655.29 µs 692.59 µs]
                        thrpt:  [14.438 Melem/s 15.260 Melem/s 16.208 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

     Running benches/mesh.rs (target/release/deps/mesh-e438b0934772a80c)
Gnuplot not found, using plotters backend
mesh_reroute/triangle_failure
                        time:   [5.0539 µs 5.1320 µs 5.2097 µs]
                        thrpt:  [191.95 Kelem/s 194.86 Kelem/s 197.87 Kelem/s]
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimated 5.0791 s (177k iterationsmesh_reroute/10_peers_10_routes
                        time:   [28.460 µs 28.676 µs 28.923 µs]
                        thrpt:  [34.574 Kelem/s 34.872 Kelem/s 35.137 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimated 6.2461 s (20k iterationsmesh_reroute/50_peers_100_routes
                        time:   [308.68 µs 309.48 µs 310.33 µs]
                        thrpt:  [3.2223 Kelem/s 3.2313 Kelem/s 3.2396 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

mesh_proximity/on_pingwave_new
                        time:   [166.30 ns 172.24 ns 178.96 ns]
                        thrpt:  [5.5878 Melem/s 5.8058 Melem/s 6.0134 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimated 5.0002 s (73M iterationsmesh_proximity/on_pingwave_dedup
                        time:   [68.091 ns 68.156 ns 68.223 ns]
                        thrpt:  [14.658 Melem/s 14.672 Melem/s 14.686 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estimated 5.0000 s (2.5B iteratiomesh_proximity/pingwave_serialize
                        time:   [1.9782 ns 1.9797 ns 1.9812 ns]
                        thrpt:  [504.73 Melem/s 505.13 Melem/s 505.51 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in estimated 5.0000 s (2.7B iteratmesh_proximity/pingwave_deserialize
                        time:   [1.8712 ns 1.8724 ns 1.8737 ns]
                        thrpt:  [533.69 Melem/s 534.07 Melem/s 534.42 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
mesh_proximity/node_count
                        time:   [200.26 ns 200.40 ns 200.54 ns]
                        thrpt:  [4.9866 Melem/s 4.9901 Melem/s 4.9936 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 44.5s, or reduce sample count to 10.
mesh_proximity/all_nodes_100
                        time:   [434.18 ms 434.95 ms 435.71 ms]
                        thrpt:  [2.2951  elem/s 2.2991  elem/s 2.3032  elem/s]

mesh_dispatch/classify_direct
                        time:   [623.70 ps 624.12 ps 624.55 ps]
                        thrpt:  [1.6011 Gelem/s 1.6023 Gelem/s 1.6033 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
mesh_dispatch/classify_routed
                        time:   [480.25 ps 480.70 ps 481.21 ps]
                        thrpt:  [2.0781 Gelem/s 2.0803 Gelem/s 2.0823 Gelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
mesh_dispatch/classify_pingwave
                        time:   [313.86 ps 314.26 ps 314.67 ps]
                        thrpt:  [3.1779 Gelem/s 3.1820 Gelem/s 3.1861 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

mesh_routing/lookup_hit time:   [15.230 ns 15.393 ns 15.550 ns]
                        thrpt:  [64.308 Melem/s 64.966 Melem/s 65.662 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
mesh_routing/lookup_miss
                        time:   [14.664 ns 14.751 ns 14.830 ns]
                        thrpt:  [67.430 Melem/s 67.792 Melem/s 68.194 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
mesh_routing/is_local   time:   [313.95 ps 314.50 ps 315.07 ps]
                        thrpt:  [3.1739 Gelem/s 3.1797 Gelem/s 3.1852 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
mesh_routing/all_routes/10
                        time:   [1.3234 µs 1.3243 µs 1.3253 µs]
                        thrpt:  [754.57 Kelem/s 755.12 Kelem/s 755.65 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
mesh_routing/all_routes/100
                        time:   [2.2140 µs 2.2189 µs 2.2244 µs]
                        thrpt:  [449.56 Kelem/s 450.68 Kelem/s 451.68 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
mesh_routing/all_routes/1000
                        time:   [11.679 µs 11.712 µs 11.749 µs]
                        thrpt:  [85.113 Kelem/s 85.384 Kelem/s 85.626 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
mesh_routing/add_route  time:   [40.475 ns 41.193 ns 41.876 ns]
                        thrpt:  [23.880 Melem/s 24.276 Melem/s 24.707 Melem/s]

     Running benches/net.rs (target/release/deps/net-42b0b1dc514d7de4)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [1.9789 ns 1.9806 ns 1.9822 ns]
                        thrpt:  [504.48 Melem/s 504.90 Melem/s 505.32 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
net_header/deserialize  time:   [2.1036 ns 2.1051 ns 2.1066 ns]
                        thrpt:  [474.70 Melem/s 475.04 Melem/s 475.37 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
net_header/roundtrip    time:   [2.1033 ns 2.1047 ns 2.1062 ns]
                        thrpt:  [474.80 Melem/s 475.12 Melem/s 475.44 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimated 5.0000 s (280M iterationsnet_event_frame/write_single/64
                        time:   [17.963 ns 18.001 ns 18.045 ns]
                        thrpt:  [3.3030 GiB/s 3.3112 GiB/s 3.3181 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estimated 5.0002 s (108M iterationnet_event_frame/write_single/256
                        time:   [47.965 ns 48.418 ns 48.867 ns]
                        thrpt:  [4.8789 GiB/s 4.9242 GiB/s 4.9707 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0002 s (140M iterationet_event_frame/write_single/1024
                        time:   [35.849 ns 35.877 ns 35.905 ns]
                        thrpt:  [26.561 GiB/s 26.582 GiB/s 26.603 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0000 s (64M iterationnet_event_frame/write_single/4096
                        time:   [78.334 ns 78.991 ns 79.691 ns]
                        thrpt:  [47.869 GiB/s 48.293 GiB/s 48.698 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
net_event_frame/write_batch/1
                        time:   [18.003 ns 18.034 ns 18.064 ns]
                        thrpt:  [3.2996 GiB/s 3.3052 GiB/s 3.3108 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
net_event_frame/write_batch/10
                        time:   [69.768 ns 69.844 ns 69.917 ns]
                        thrpt:  [8.5250 GiB/s 8.5340 GiB/s 8.5432 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
net_event_frame/write_batch/50
                        time:   [148.09 ns 148.44 ns 148.97 ns]
                        thrpt:  [20.005 GiB/s 20.077 GiB/s 20.124 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
net_event_frame/write_batch/100
                        time:   [272.65 ns 272.90 ns 273.23 ns]
                        thrpt:  [21.815 GiB/s 21.841 GiB/s 21.861 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
net_event_frame/read_batch_10
                        time:   [138.52 ns 139.33 ns 140.21 ns]
                        thrpt:  [71.319 Melem/s 71.772 Melem/s 72.190 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

net_packet_pool/get_return/16
                        time:   [38.304 ns 38.353 ns 38.403 ns]
                        thrpt:  [26.040 Melem/s 26.073 Melem/s 26.107 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
net_packet_pool/get_return/64
                        time:   [37.962 ns 38.003 ns 38.047 ns]
                        thrpt:  [26.284 Melem/s 26.314 Melem/s 26.342 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
net_packet_pool/get_return/256
                        time:   [37.881 ns 37.929 ns 37.979 ns]
                        thrpt:  [26.330 Melem/s 26.365 Melem/s 26.398 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

net_packet_build/build_packet/1
                        time:   [485.35 ns 490.60 ns 501.05 ns]
                        thrpt:  [121.82 MiB/s 124.41 MiB/s 125.75 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0003 s (2.7M iterationnet_packet_build/build_packet/10
                        time:   [1.8462 µs 1.8492 µs 1.8526 µs]
                        thrpt:  [329.46 MiB/s 330.05 MiB/s 330.60 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0071 s (611k iterationnet_packet_build/build_packet/50
                        time:   [8.1918 µs 8.2019 µs 8.2129 µs]
                        thrpt:  [371.58 MiB/s 372.08 MiB/s 372.54 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

net_encryption/encrypt/64
                        time:   [485.16 ns 486.18 ns 487.41 ns]
                        thrpt:  [125.22 MiB/s 125.54 MiB/s 125.80 MiB/s]
net_encryption/encrypt/256
                        time:   [921.70 ns 923.05 ns 924.60 ns]
                        thrpt:  [264.05 MiB/s 264.49 MiB/s 264.88 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
net_encryption/encrypt/1024
                        time:   [2.6945 µs 2.6985 µs 2.7028 µs]
                        thrpt:  [361.32 MiB/s 361.89 MiB/s 362.43 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
net_encryption/encrypt/4096
                        time:   [9.7628 µs 9.7699 µs 9.7771 µs]
                        thrpt:  [399.53 MiB/s 399.82 MiB/s 400.11 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

net_keypair/generate    time:   [12.356 µs 12.396 µs 12.442 µs]
                        thrpt:  [80.372 Kelem/s 80.672 Kelem/s 80.932 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

net_aad/generate        time:   [2.0263 ns 2.0277 ns 2.0291 ns]
                        thrpt:  [492.82 Melem/s 493.18 Melem/s 493.52 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0002 s (132M itepool_comparison/shared_pool_get_return
                        time:   [37.958 ns 38.013 ns 38.071 ns]
                        thrpt:  [26.267 Melem/s 26.306 Melem/s 26.345 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.0001 s (61pool_comparison/thread_local_pool_get_return
                        time:   [82.725 ns 83.050 ns 83.443 ns]
                        thrpt:  [11.984 Melem/s 12.041 Melem/s 12.088 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
pool_comparison/shared_pool_10x
                        time:   [339.37 ns 339.86 ns 340.34 ns]
                        thrpt:  [2.9382 Melem/s 2.9424 Melem/s 2.9466 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0018 s (5.3M iterpool_comparison/thread_local_pool_10x
                        time:   [972.21 ns 985.47 ns 1.0007 µs]
                        thrpt:  [999.32 Kelem/s 1.0147 Melem/s 1.0286 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0009 s (10M iterationscipher_comparison/shared_pool/64
                        time:   [497.07 ns 498.00 ns 499.13 ns]
                        thrpt:  [122.28 MiB/s 122.56 MiB/s 122.79 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0003 s (9.2M iteraticipher_comparison/fast_chacha20/64
                        time:   [534.48 ns 536.01 ns 537.82 ns]
                        thrpt:  [113.49 MiB/s 113.87 MiB/s 114.20 MiB/s]
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0036 s (5.4M iteratiocipher_comparison/shared_pool/256
                        time:   [922.05 ns 923.53 ns 925.21 ns]
                        thrpt:  [263.88 MiB/s 264.36 MiB/s 264.78 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0006 s (5.2M iteratcipher_comparison/fast_chacha20/256
                        time:   [964.88 ns 965.65 ns 966.45 ns]
                        thrpt:  [252.62 MiB/s 252.82 MiB/s 253.03 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0061 s (1.9M iteraticipher_comparison/shared_pool/1024
                        time:   [2.6952 µs 2.7012 µs 2.7091 µs]
                        thrpt:  [360.47 MiB/s 361.53 MiB/s 362.33 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0104 s (1.8M iteracipher_comparison/fast_chacha20/1024
                        time:   [2.7160 µs 2.7181 µs 2.7202 µs]
                        thrpt:  [359.00 MiB/s 359.28 MiB/s 359.55 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0334 s (515k iteraticipher_comparison/shared_pool/4096
                        time:   [9.7552 µs 9.7638 µs 9.7730 µs]
                        thrpt:  [399.70 MiB/s 400.07 MiB/s 400.43 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0162 s (515k iteracipher_comparison/fast_chacha20/4096
                        time:   [9.7318 µs 9.7392 µs 9.7467 µs]
                        thrpt:  [400.78 MiB/s 401.08 MiB/s 401.39 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

adaptive_batcher/optimal_size
                        time:   [974.70 ps 975.48 ps 976.27 ps]
                        thrpt:  [1.0243 Gelem/s 1.0251 Gelem/s 1.0260 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
adaptive_batcher/record time:   [3.8784 ns 3.8810 ns 3.8836 ns]
                        thrpt:  [257.50 Melem/s 257.67 Melem/s 257.84 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
adaptive_batcher/full_cycle
                        time:   [4.3939 ns 4.3973 ns 4.4006 ns]
                        thrpt:  [227.24 Melem/s 227.41 Melem/s 227.59 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0160 s (611k itee2e_packet_build/shared_pool_50_events
                        time:   [8.1984 µs 8.2074 µs 8.2165 µs]
                        thrpt:  [371.42 MiB/s 371.83 MiB/s 372.24 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0227 s (611k iterationse2e_packet_build/fast_50_events
                        time:   [8.1978 µs 8.2034 µs 8.2090 µs]
                        thrpt:  [371.76 MiB/s 372.01 MiB/s 372.27 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.2s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 9.1698 s (5050 itemultithread_packet_build/shared_pool/8
                        time:   [1.8173 ms 1.8221 ms 1.8275 ms]
                        thrpt:  [4.3776 Melem/s 4.3906 Melem/s 4.4021 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 8.9911 s (10multithread_packet_build/thread_local_pool/8
                        time:   [888.52 µs 891.72 µs 896.04 µs]
                        thrpt:  [8.9282 Melem/s 8.9714 Melem/s 9.0037 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.0346 s (1100 itmultithread_packet_build/shared_pool/16
                        time:   [4.4351 ms 4.5237 ms 4.6201 ms]
                        thrpt:  [3.4631 Melem/s 3.5369 Melem/s 3.6076 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 8.7325 s (5multithread_packet_build/thread_local_pool/16
                        time:   [1.7253 ms 1.7291 ms 1.7330 ms]
                        thrpt:  [9.2323 Melem/s 9.2533 Melem/s 9.2736 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.1229 s (700 itemultithread_packet_build/shared_pool/24
                        time:   [7.0297 ms 7.2028 ms 7.3851 ms]
                        thrpt:  [3.2498 Melem/s 3.3320 Melem/s 3.4141 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.0143 s (2multithread_packet_build/thread_local_pool/24
                        time:   [2.5026 ms 2.5086 ms 2.5154 ms]
                        thrpt:  [9.5410 Melem/s 9.5671 Melem/s 9.5901 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.0282 s (500 itemultithread_packet_build/shared_pool/32
                        time:   [9.7882 ms 10.232 ms 10.723 ms]
                        thrpt:  [2.9842 Melem/s 3.1273 Melem/s 3.2693 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.2524 s (1multithread_packet_build/thread_local_pool/32
                        time:   [3.2783 ms 3.2853 ms 3.2925 ms]
                        thrpt:  [9.7189 Melem/s 9.7403 Melem/s 9.7611 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 7.0520 s (5050 itmultithread_mixed_frames/shared_mixed/8
                        time:   [1.3933 ms 1.3970 ms 1.4008 ms]
                        thrpt:  [8.5666 Melem/s 8.5899 Melem/s 8.6124 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 5.3599 s (5050 itermultithread_mixed_frames/fast_mixed/8
                        time:   [1.0586 ms 1.0635 ms 1.0686 ms]
                        thrpt:  [11.230 Melem/s 11.283 Melem/s 11.335 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samples in estimated 5.2001 s (1700 imultithread_mixed_frames/shared_mixed/16
                        time:   [3.0283 ms 3.0807 ms 3.1389 ms]
                        thrpt:  [7.6459 Melem/s 7.7904 Melem/s 7.9252 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  9 (9.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples in estimated 5.1345 s (2500 itemultithread_mixed_frames/fast_mixed/16
                        time:   [2.0478 ms 2.0532 ms 2.0587 ms]
                        thrpt:  [11.658 Melem/s 11.689 Melem/s 11.720 Melem/s]
Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samples in estimated 5.2108 s (1100 imultithread_mixed_frames/shared_mixed/24
                        time:   [4.6144 ms 4.7466 ms 4.8928 ms]
                        thrpt:  [7.3577 Melem/s 7.5844 Melem/s 7.8016 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples in estimated 5.1161 s (1700 itemultithread_mixed_frames/fast_mixed/24
                        time:   [3.0074 ms 3.0157 ms 3.0245 ms]
                        thrpt:  [11.903 Melem/s 11.938 Melem/s 11.971 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samples in estimated 5.1943 s (800 itmultithread_mixed_frames/shared_mixed/32
                        time:   [6.3321 ms 6.6194 ms 6.9349 ms]
                        thrpt:  [6.9215 Melem/s 7.2514 Melem/s 7.5804 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples in estimated 5.1405 s (1300 itemultithread_mixed_frames/fast_mixed/32
                        time:   [3.9541 ms 3.9629 ms 3.9721 ms]
                        thrpt:  [12.084 Melem/s 12.112 Melem/s 12.139 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samples in estimated 5.2893 s (300 itpool_contention/shared_acquire_release/8
                        time:   [17.589 ms 17.625 ms 17.662 ms]
                        thrpt:  [4.5294 Melem/s 4.5391 Melem/s 4.5483 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.6s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples in estimated 5.6017 s (5050 itepool_contention/fast_acquire_release/8
                        time:   [1.1043 ms 1.1100 ms 1.1157 ms]
                        thrpt:  [71.702 Melem/s 72.069 Melem/s 72.442 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samples in estimated 8.5912 s (200 ipool_contention/shared_acquire_release/16
                        time:   [42.212 ms 42.609 ms 43.031 ms]
                        thrpt:  [3.7183 Melem/s 3.7551 Melem/s 3.7904 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples in estimated 5.0459 s (2200 itpool_contention/fast_acquire_release/16
                        time:   [2.2729 ms 2.2840 ms 2.2952 ms]
                        thrpt:  [69.712 Melem/s 70.053 Melem/s 70.395 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking pool_contention/shared_acquire_release/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.4s, or reduce sample count to 70.
Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samples in estimated 6.4341 s (100 ipool_contention/shared_acquire_release/24
                        time:   [63.484 ms 64.591 ms 65.769 ms]
                        thrpt:  [3.6491 Melem/s 3.7157 Melem/s 3.7805 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples in estimated 5.2334 s (1600 itpool_contention/fast_acquire_release/24
                        time:   [3.2718 ms 3.2835 ms 3.2952 ms]
                        thrpt:  [72.834 Melem/s 73.092 Melem/s 73.353 Melem/s]
Benchmarking pool_contention/shared_acquire_release/32: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.0s, or reduce sample count to 50.
Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samples in estimated 9.0101 s (100 ipool_contention/shared_acquire_release/32
                        time:   [83.530 ms 84.946 ms 86.448 ms]
                        thrpt:  [3.7017 Melem/s 3.7671 Melem/s 3.8310 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples in estimated 5.0989 s (1100 itpool_contention/fast_acquire_release/32
                        time:   [4.7187 ms 4.7333 ms 4.7479 ms]
                        thrpt:  [67.398 Melem/s 67.606 Melem/s 67.816 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples in estimated 5.7859 s (840 iterathroughput_scaling/fast_pool_scaling/1
                        time:   [6.8740 ms 6.8805 ms 6.8864 ms]
                        thrpt:  [290.43 Kelem/s 290.68 Kelem/s 290.95 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples in estimated 6.0148 s (840 iterathroughput_scaling/fast_pool_scaling/2
                        time:   [6.9796 ms 6.9830 ms 6.9874 ms]
                        thrpt:  [572.46 Kelem/s 572.82 Kelem/s 573.10 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples in estimated 6.2160 s (840 iterathroughput_scaling/fast_pool_scaling/4
                        time:   [7.4068 ms 7.4118 ms 7.4168 ms]
                        thrpt:  [1.0786 Melem/s 1.0794 Melem/s 1.0801 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples in estimated 6.5394 s (840 iterathroughput_scaling/fast_pool_scaling/8
                        time:   [7.7809 ms 7.8405 ms 7.9048 ms]
                        thrpt:  [2.0241 Melem/s 2.0407 Melem/s 2.0563 Melem/s]
Found 5 outliers among 20 measurements (25.00%)
  3 (15.00%) low mild
  2 (10.00%) high severe
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples in estimated 6.5875 s (420 iterthroughput_scaling/fast_pool_scaling/16
                        time:   [15.475 ms 15.533 ms 15.604 ms]
                        thrpt:  [2.0507 Melem/s 2.0601 Melem/s 2.0678 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples in estimated 9.6204 s (420 iterthroughput_scaling/fast_pool_scaling/24
                        time:   [22.925 ms 22.995 ms 23.082 ms]
                        thrpt:  [2.0796 Melem/s 2.0874 Melem/s 2.0937 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe
Benchmarking throughput_scaling/fast_pool_scaling/32: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.3s, enable flat sampling, or reduce sample count to 10.
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples in estimated 6.3096 s (210 iterthroughput_scaling/fast_pool_scaling/32
                        time:   [29.912 ms 29.987 ms 30.082 ms]
                        thrpt:  [2.1275 Melem/s 2.1343 Melem/s 2.1396 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

routing_header/serialize
                        time:   [630.44 ps 631.55 ps 632.76 ps]
                        thrpt:  [1.5804 Gelem/s 1.5834 Gelem/s 1.5862 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
routing_header/deserialize
                        time:   [938.67 ps 939.48 ps 940.36 ps]
                        thrpt:  [1.0634 Gelem/s 1.0644 Gelem/s 1.0653 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
routing_header/roundtrip
                        time:   [936.81 ps 937.35 ps 937.90 ps]
                        thrpt:  [1.0662 Gelem/s 1.0668 Gelem/s 1.0675 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
routing_header/forward  time:   [572.20 ps 573.36 ps 574.53 ps]
                        thrpt:  [1.7406 Gelem/s 1.7441 Gelem/s 1.7477 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

routing_table/lookup_hit
                        time:   [37.682 ns 38.272 ns 38.925 ns]
                        thrpt:  [25.691 Melem/s 26.129 Melem/s 26.538 Melem/s]
routing_table/lookup_miss
                        time:   [14.874 ns 14.983 ns 15.068 ns]
                        thrpt:  [66.365 Melem/s 66.741 Melem/s 67.230 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low severe
  4 (4.00%) low mild
routing_table/is_local  time:   [314.00 ps 314.51 ps 315.03 ps]
                        thrpt:  [3.1743 Gelem/s 3.1796 Gelem/s 3.1847 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
routing_table/add_route time:   [235.13 ns 240.93 ns 246.07 ns]
                        thrpt:  [4.0638 Melem/s 4.1506 Melem/s 4.2529 Melem/s]
routing_table/record_in time:   [47.826 ns 48.542 ns 49.234 ns]
                        thrpt:  [20.311 Melem/s 20.601 Melem/s 20.909 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
routing_table/record_out
                        time:   [17.824 ns 18.196 ns 18.577 ns]
                        thrpt:  [53.831 Melem/s 54.958 Melem/s 56.103 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
routing_table/aggregate_stats
                        time:   [2.0815 µs 2.0831 µs 2.0848 µs]
                        thrpt:  [479.67 Kelem/s 480.05 Kelem/s 480.43 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

fair_scheduler/creation time:   [288.23 ns 288.81 ns 289.64 ns]
                        thrpt:  [3.4526 Melem/s 3.4625 Melem/s 3.4694 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estimated 5.0010 s (25M iterationfair_scheduler/stream_count_empty
                        time:   [200.49 ns 200.62 ns 200.77 ns]
                        thrpt:  [4.9808 Melem/s 4.9844 Melem/s 4.9879 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
fair_scheduler/total_queued
                        time:   [312.33 ps 312.81 ps 313.64 ps]
                        thrpt:  [3.1883 Gelem/s 3.1968 Gelem/s 3.2017 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
fair_scheduler/cleanup_empty
                        time:   [201.50 ns 201.79 ns 202.26 ns]
                        thrpt:  [4.9442 Melem/s 4.9557 Melem/s 4.9627 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samples in estimated 5.6578 s (35routing_table_concurrent/concurrent_lookup/4
                        time:   [161.29 µs 163.70 µs 165.69 µs]
                        thrpt:  [24.141 Melem/s 24.434 Melem/s 24.801 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samples in estimated 5.6734 s (20krouting_table_concurrent/concurrent_stats/4
                        time:   [280.34 µs 280.79 µs 281.23 µs]
                        thrpt:  [14.223 Melem/s 14.245 Melem/s 14.268 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samples in estimated 5.0088 s (20routing_table_concurrent/concurrent_lookup/8
                        time:   [246.05 µs 247.37 µs 248.43 µs]
                        thrpt:  [32.202 Melem/s 32.340 Melem/s 32.514 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samples in estimated 6.2378 s (15krouting_table_concurrent/concurrent_stats/8
                        time:   [407.15 µs 410.01 µs 414.71 µs]
                        thrpt:  [19.290 Melem/s 19.512 Melem/s 19.649 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 samples in estimated 6.4196 s (1routing_table_concurrent/concurrent_lookup/16
                        time:   [423.49 µs 424.01 µs 424.54 µs]
                        thrpt:  [37.688 Melem/s 37.735 Melem/s 37.782 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samples in estimated 8.1077 s (10routing_table_concurrent/concurrent_stats/16
                        time:   [800.29 µs 802.18 µs 803.72 µs]
                        thrpt:  [19.907 Melem/s 19.946 Melem/s 19.993 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples in estimated 5.0001 s (132M iterrouting_decision/parse_lookup_forward
                        time:   [37.610 ns 37.828 ns 38.084 ns]
                        thrpt:  [26.258 Melem/s 26.435 Melem/s 26.589 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estimated 5.0001 s (45M iterationsrouting_decision/full_with_stats
                        time:   [109.73 ns 109.89 ns 110.07 ns]
                        thrpt:  [9.0854 Melem/s 9.1003 Melem/s 9.1136 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in estimated 5.0012 s (17M iterationstream_multiplexing/lookup_all/10
                        time:   [292.30 ns 292.61 ns 293.08 ns]
                        thrpt:  [34.120 Melem/s 34.175 Melem/s 34.212 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estimated 5.0000 s (11M iterationsstream_multiplexing/stats_all/10
                        time:   [471.44 ns 476.61 ns 481.93 ns]
                        thrpt:  [20.750 Melem/s 20.981 Melem/s 21.212 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in estimated 5.0086 s (1.7M iteratistream_multiplexing/lookup_all/100
                        time:   [2.9240 µs 2.9255 µs 2.9270 µs]
                        thrpt:  [34.165 Melem/s 34.182 Melem/s 34.199 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in estimated 5.0037 s (1.0M iteratiostream_multiplexing/stats_all/100
                        time:   [4.7638 µs 4.8267 µs 4.8881 µs]
                        thrpt:  [20.458 Melem/s 20.718 Melem/s 20.992 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estimated 5.0221 s (172k iteratstream_multiplexing/lookup_all/1000
                        time:   [29.227 µs 29.242 µs 29.258 µs]
                        thrpt:  [34.178 Melem/s 34.197 Melem/s 34.215 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in estimated 5.0509 s (96k iteratiostream_multiplexing/stats_all/1000
                        time:   [52.499 µs 52.571 µs 52.647 µs]
                        thrpt:  [18.994 Melem/s 19.022 Melem/s 19.048 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in estimated 5.9344 s (20k iteratstream_multiplexing/lookup_all/10000
                        time:   [293.24 µs 293.53 µs 293.82 µs]
                        thrpt:  [34.035 Melem/s 34.069 Melem/s 34.101 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estimated 5.7367 s (10k iteratistream_multiplexing/stats_all/10000
                        time:   [567.28 µs 568.57 µs 569.87 µs]
                        thrpt:  [17.548 Melem/s 17.588 Melem/s 17.628 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimated 5.0000 s (214M iterationmultihop_packet_builder/build/64
                        time:   [23.305 ns 23.388 ns 23.473 ns]
                        thrpt:  [2.5392 GiB/s 2.5485 GiB/s 2.5575 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samples in estimated 5.0000 s (242M multihop_packet_builder/build_priority/64
                        time:   [20.703 ns 20.757 ns 20.818 ns]
                        thrpt:  [2.8631 GiB/s 2.8716 GiB/s 2.8790 GiB/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estimated 5.0002 s (96M iterationmultihop_packet_builder/build/256
                        time:   [51.456 ns 51.626 ns 51.807 ns]
                        thrpt:  [4.6021 GiB/s 4.6182 GiB/s 4.6334 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samples in estimated 5.0001 s (101Mmultihop_packet_builder/build_priority/256
                        time:   [49.197 ns 49.400 ns 49.611 ns]
                        thrpt:  [4.8058 GiB/s 4.8263 GiB/s 4.8462 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in estimated 5.0001 s (122M iteratimultihop_packet_builder/build/1024
                        time:   [41.018 ns 41.060 ns 41.102 ns]
                        thrpt:  [23.203 GiB/s 23.226 GiB/s 23.250 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 samples in estimated 5.0000 s (130multihop_packet_builder/build_priority/1024
                        time:   [38.314 ns 38.338 ns 38.363 ns]
                        thrpt:  [24.859 GiB/s 24.875 GiB/s 24.891 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in estimated 5.0001 s (53M iteratiomultihop_packet_builder/build/4096
                        time:   [93.739 ns 94.450 ns 95.199 ns]
                        thrpt:  [40.071 GiB/s 40.389 GiB/s 40.695 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 samples in estimated 5.0000 s (54Mmultihop_packet_builder/build_priority/4096
                        time:   [92.115 ns 93.279 ns 94.774 ns]
                        thrpt:  [40.250 GiB/s 40.896 GiB/s 41.412 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe

multihop_chain/forward_chain/1
                        time:   [59.741 ns 60.075 ns 60.404 ns]
                        thrpt:  [16.555 Melem/s 16.646 Melem/s 16.739 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
multihop_chain/forward_chain/2
                        time:   [115.26 ns 115.48 ns 115.76 ns]
                        thrpt:  [8.6386 Melem/s 8.6592 Melem/s 8.6761 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
multihop_chain/forward_chain/3
                        time:   [162.61 ns 163.31 ns 164.04 ns]
                        thrpt:  [6.0959 Melem/s 6.1232 Melem/s 6.1496 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
multihop_chain/forward_chain/4
                        time:   [220.20 ns 221.16 ns 222.16 ns]
                        thrpt:  [4.5012 Melem/s 4.5217 Melem/s 4.5413 Melem/s]
multihop_chain/forward_chain/5
                        time:   [282.89 ns 298.76 ns 330.37 ns]
                        thrpt:  [3.0269 Melem/s 3.3471 Melem/s 3.5349 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

hop_latency/single_hop_process
                        time:   [1.4910 ns 1.4917 ns 1.4924 ns]
                        thrpt:  [670.05 Melem/s 670.36 Melem/s 670.67 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
hop_latency/single_hop_full
                        time:   [54.835 ns 54.924 ns 55.029 ns]
                        thrpt:  [18.172 Melem/s 18.207 Melem/s 18.237 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  7 (7.00%) high mild
  3 (3.00%) high severe

hop_scaling/64B_1hops   time:   [29.726 ns 29.787 ns 29.849 ns]
                        thrpt:  [1.9969 GiB/s 2.0010 GiB/s 2.0051 GiB/s]
hop_scaling/64B_2hops   time:   [52.291 ns 52.402 ns 52.524 ns]
                        thrpt:  [1.1348 GiB/s 1.1375 GiB/s 1.1399 GiB/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe
hop_scaling/64B_3hops   time:   [76.195 ns 76.510 ns 76.858 ns]
                        thrpt:  [794.13 MiB/s 797.74 MiB/s 801.04 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
hop_scaling/64B_4hops   time:   [99.725 ns 100.14 ns 100.59 ns]
                        thrpt:  [606.79 MiB/s 609.50 MiB/s 612.03 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/64B_5hops   time:   [130.14 ns 130.65 ns 131.18 ns]
                        thrpt:  [465.28 MiB/s 467.15 MiB/s 468.99 MiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
hop_scaling/256B_1hops  time:   [56.916 ns 57.408 ns 57.985 ns]
                        thrpt:  [4.1117 GiB/s 4.1530 GiB/s 4.1890 GiB/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe
hop_scaling/256B_2hops  time:   [117.24 ns 118.98 ns 120.78 ns]
                        thrpt:  [1.9739 GiB/s 2.0038 GiB/s 2.0335 GiB/s]
hop_scaling/256B_3hops  time:   [160.85 ns 161.98 ns 163.45 ns]
                        thrpt:  [1.4587 GiB/s 1.4719 GiB/s 1.4823 GiB/s]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) high mild
  16 (16.00%) high severe
hop_scaling/256B_4hops  time:   [227.58 ns 229.50 ns 231.28 ns]
                        thrpt:  [1.0309 GiB/s 1.0389 GiB/s 1.0476 GiB/s]
Found 19 outliers among 100 measurements (19.00%)
  15 (15.00%) low severe
  4 (4.00%) low mild
hop_scaling/256B_5hops  time:   [274.28 ns 276.93 ns 279.69 ns]
                        thrpt:  [872.89 MiB/s 881.59 MiB/s 890.12 MiB/s]
hop_scaling/1024B_1hops time:   [49.309 ns 49.380 ns 49.456 ns]
                        thrpt:  [19.283 GiB/s 19.313 GiB/s 19.341 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
hop_scaling/1024B_2hops time:   [116.42 ns 118.54 ns 121.30 ns]
                        thrpt:  [7.8621 GiB/s 8.0452 GiB/s 8.1920 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
hop_scaling/1024B_3hops time:   [154.22 ns 155.43 ns 156.77 ns]
                        thrpt:  [6.0831 GiB/s 6.1359 GiB/s 6.1838 GiB/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
hop_scaling/1024B_4hops time:   [213.48 ns 214.08 ns 214.70 ns]
                        thrpt:  [4.4419 GiB/s 4.4548 GiB/s 4.4672 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/1024B_5hops time:   [263.71 ns 266.20 ns 268.62 ns]
                        thrpt:  [3.5503 GiB/s 3.5825 GiB/s 3.6164 GiB/s]

Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samples in estimated 5.0006 s (28M imultihop_with_routing/route_and_forward/1
                        time:   [175.62 ns 176.18 ns 176.79 ns]
                        thrpt:  [5.6566 Melem/s 5.6761 Melem/s 5.6941 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samples in estimated 5.0002 s (14M imultihop_with_routing/route_and_forward/2
                        time:   [349.32 ns 350.09 ns 350.91 ns]
                        thrpt:  [2.8497 Melem/s 2.8564 Melem/s 2.8627 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samples in estimated 5.0020 s (9.4M multihop_with_routing/route_and_forward/3
                        time:   [525.58 ns 527.48 ns 529.35 ns]
                        thrpt:  [1.8891 Melem/s 1.8958 Melem/s 1.9027 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samples in estimated 5.0014 s (7.0M multihop_with_routing/route_and_forward/4
                        time:   [704.88 ns 706.86 ns 709.07 ns]
                        thrpt:  [1.4103 Melem/s 1.4147 Melem/s 1.4187 Melem/s]
Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samples in estimated 5.0025 s (5.7M multihop_with_routing/route_and_forward/5
                        time:   [910.51 ns 929.86 ns 966.16 ns]
                        thrpt:  [1.0350 Melem/s 1.0754 Melem/s 1.0983 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples in estimated 5.1423 s (6930 itmultihop_concurrent/concurrent_forward/4
                        time:   [724.36 µs 742.05 µs 774.27 µs]
                        thrpt:  [5.1662 Melem/s 5.3904 Melem/s 5.5221 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low severe
  1 (5.00%) high severe
Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples in estimated 5.2246 s (4200 itmultihop_concurrent/concurrent_forward/8
                        time:   [978.21 µs 1.0698 ms 1.1942 ms]
                        thrpt:  [6.6990 Melem/s 7.4782 Melem/s 8.1782 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) low severe
Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samples in estimated 5.2992 s (2310 imultihop_concurrent/concurrent_forward/16
                        time:   [2.2592 ms 2.2729 ms 2.2893 ms]
                        thrpt:  [6.9890 Melem/s 7.0396 Melem/s 7.0822 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

pingwave/serialize      time:   [776.39 ps 776.77 ps 777.27 ps]
                        thrpt:  [1.2866 Gelem/s 1.2874 Gelem/s 1.2880 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
pingwave/deserialize    time:   [931.28 ps 931.49 ps 931.73 ps]
                        thrpt:  [1.0733 Gelem/s 1.0735 Gelem/s 1.0738 Gelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
pingwave/roundtrip      time:   [931.35 ps 931.65 ps 932.02 ps]
                        thrpt:  [1.0729 Gelem/s 1.0734 Gelem/s 1.0737 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
pingwave/forward        time:   [626.21 ps 627.24 ps 628.39 ps]
                        thrpt:  [1.5914 Gelem/s 1.5943 Gelem/s 1.5969 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild

capabilities/serialize_simple
                        time:   [19.644 ns 19.657 ns 19.668 ns]
                        thrpt:  [50.845 Melem/s 50.873 Melem/s 50.905 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimated 5.0000 s (1.0B iterationscapabilities/deserialize_simple
                        time:   [4.8872 ns 4.8917 ns 4.8967 ns]
                        thrpt:  [204.22 Melem/s 204.43 Melem/s 204.62 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
capabilities/serialize_complex
                        time:   [40.987 ns 40.999 ns 41.012 ns]
                        thrpt:  [24.383 Melem/s 24.391 Melem/s 24.398 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in estimated 5.0001 s (33M iterationscapabilities/deserialize_complex
                        time:   [152.96 ns 153.11 ns 153.24 ns]
                        thrpt:  [6.5257 Melem/s 6.5313 Melem/s 6.5375 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) low mild
  1 (1.00%) high severe

local_graph/create_pingwave
                        time:   [2.0988 ns 2.1019 ns 2.1052 ns]
                        thrpt:  [475.03 Melem/s 475.77 Melem/s 476.45 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
local_graph/on_pingwave_new
                        time:   [110.82 ns 112.51 ns 113.86 ns]
                        thrpt:  [8.7825 Melem/s 8.8880 Melem/s 9.0239 Melem/s]
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estimated 5.0001 s (151M iteratiolocal_graph/on_pingwave_duplicate
                        time:   [33.106 ns 33.135 ns 33.171 ns]
                        thrpt:  [30.147 Melem/s 30.180 Melem/s 30.206 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
local_graph/get_node    time:   [26.217 ns 31.479 ns 42.936 ns]
                        thrpt:  [23.291 Melem/s 31.767 Melem/s 38.142 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
local_graph/node_count  time:   [206.84 ns 208.29 ns 210.23 ns]
                        thrpt:  [4.7566 Melem/s 4.8010 Melem/s 4.8348 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  14 (14.00%) high severe
local_graph/stats       time:   [615.48 ns 615.99 ns 616.49 ns]
                        thrpt:  [1.6221 Melem/s 1.6234 Melem/s 1.6247 Melem/s]
Found 29 outliers among 100 measurements (29.00%)
  8 (8.00%) low severe
  8 (8.00%) low mild
  9 (9.00%) high mild
  4 (4.00%) high severe

graph_scaling/all_nodes/100
                        time:   [2.4612 µs 2.4677 µs 2.4737 µs]
                        thrpt:  [40.426 Melem/s 40.524 Melem/s 40.631 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in estimated 5.0056 s (1.8M iteratgraph_scaling/nodes_within_hops/100
                        time:   [2.8362 µs 2.8428 µs 2.8493 µs]
                        thrpt:  [35.096 Melem/s 35.177 Melem/s 35.259 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
graph_scaling/all_nodes/500
                        time:   [7.8859 µs 7.9031 µs 7.9209 µs]
                        thrpt:  [63.124 Melem/s 63.266 Melem/s 63.404 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in estimated 5.0434 s (540k iteratgraph_scaling/nodes_within_hops/500
                        time:   [9.2916 µs 9.3159 µs 9.3453 µs]
                        thrpt:  [53.503 Melem/s 53.671 Melem/s 53.812 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
graph_scaling/all_nodes/1000
                        time:   [154.72 µs 160.51 µs 165.82 µs]
                        thrpt:  [6.0308 Melem/s 6.2300 Melem/s 6.4634 Melem/s]
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in estimated 5.1744 s (30k iteratgraph_scaling/nodes_within_hops/1000
                        time:   [166.04 µs 171.31 µs 176.33 µs]
                        thrpt:  [5.6713 Melem/s 5.8375 Melem/s 6.0226 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
graph_scaling/all_nodes/5000
                        time:   [218.79 µs 223.50 µs 228.34 µs]
                        thrpt:  [21.898 Melem/s 22.372 Melem/s 22.853 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in estimated 5.9435 s (25k iteratgraph_scaling/nodes_within_hops/5000
                        time:   [235.48 µs 240.81 µs 245.95 µs]
                        thrpt:  [20.329 Melem/s 20.763 Melem/s 21.233 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low severe
  5 (5.00%) low mild

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimated 5.0376 s (288k iterationscapability_search/find_with_gpu
                        time:   [17.499 µs 17.512 µs 17.524 µs]
                        thrpt:  [57.065 Kelem/s 57.105 Kelem/s 57.147 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples in estimated 5.0508 s (162k itercapability_search/find_by_tool_python
                        time:   [31.224 µs 31.281 µs 31.353 µs]
                        thrpt:  [31.895 Kelem/s 31.968 Kelem/s 32.027 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in estimated 5.0465 s (126k iteratcapability_search/find_by_tool_rust
                        time:   [39.997 µs 40.041 µs 40.083 µs]
                        thrpt:  [24.948 Kelem/s 24.974 Kelem/s 25.002 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples in estimated 5.0236 s (44k iteragraph_concurrent/concurrent_pingwave/4
                        time:   [112.32 µs 113.91 µs 115.79 µs]
                        thrpt:  [17.273 Melem/s 17.557 Melem/s 17.807 Melem/s]
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples in estimated 5.0017 s (28k iteragraph_concurrent/concurrent_pingwave/8
                        time:   [177.67 µs 179.90 µs 182.57 µs]
                        thrpt:  [21.910 Melem/s 22.234 Melem/s 22.514 Melem/s]
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples in estimated 5.0145 s (16k itergraph_concurrent/concurrent_pingwave/16
                        time:   [316.74 µs 317.45 µs 318.16 µs]
                        thrpt:  [25.145 Melem/s 25.201 Melem/s 25.257 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low severe

path_finding/path_1_hop time:   [1.5139 µs 1.5153 µs 1.5167 µs]
                        thrpt:  [659.34 Kelem/s 659.93 Kelem/s 660.53 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
path_finding/path_2_hops
                        time:   [1.5953 µs 1.5971 µs 1.5987 µs]
                        thrpt:  [625.50 Kelem/s 626.15 Kelem/s 626.84 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
path_finding/path_4_hops
                        time:   [1.8235 µs 1.8255 µs 1.8275 µs]
                        thrpt:  [547.20 Kelem/s 547.79 Kelem/s 548.39 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
path_finding/path_not_found
                        time:   [1.8286 µs 1.8306 µs 1.8327 µs]
                        thrpt:  [545.65 Kelem/s 546.26 Kelem/s 546.88 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
path_finding/path_complex_graph
                        time:   [310.52 µs 313.18 µs 316.12 µs]
                        thrpt:  [3.1634 Kelem/s 3.1930 Kelem/s 3.2204 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in estimated 5.0001 s (172M iteratfailure_detector/heartbeat_existing
                        time:   [28.810 ns 28.981 ns 29.244 ns]
                        thrpt:  [34.195 Melem/s 34.505 Melem/s 34.710 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  8 (8.00%) high mild
  6 (6.00%) high severe
failure_detector/heartbeat_new
                        time:   [250.29 ns 253.98 ns 257.30 ns]
                        thrpt:  [3.8865 Melem/s 3.9373 Melem/s 3.9954 Melem/s]
failure_detector/status_check
                        time:   [13.291 ns 13.420 ns 13.560 ns]
                        thrpt:  [73.744 Melem/s 74.515 Melem/s 75.238 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 35.2s, or reduce sample count to 10.
failure_detector/check_all
                        time:   [344.01 ms 344.25 ms 344.54 ms]
                        thrpt:  [2.9024  elem/s 2.9049  elem/s 2.9069  elem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.1s, or reduce sample count to 60.
failure_detector/stats  time:   [81.044 ms 81.085 ms 81.127 ms]
                        thrpt:  [12.326  elem/s 12.333  elem/s 12.339  elem/s]

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimated 5.0000 s (1.8B iterationsloss_simulator/should_drop_1pct
                        time:   [2.8006 ns 2.8028 ns 2.8050 ns]
                        thrpt:  [356.50 Melem/s 356.79 Melem/s 357.07 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimated 5.0000 s (1.6B iterationsloss_simulator/should_drop_5pct
                        time:   [3.1611 ns 3.1644 ns 3.1679 ns]
                        thrpt:  [315.67 Melem/s 316.01 Melem/s 316.34 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estimated 5.0000 s (1.4B iterationloss_simulator/should_drop_10pct
                        time:   [3.6452 ns 3.6482 ns 3.6511 ns]
                        thrpt:  [273.89 Melem/s 274.11 Melem/s 274.33 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estimated 5.0000 s (1.1B iterationloss_simulator/should_drop_20pct
                        time:   [4.5895 ns 4.5941 ns 4.5991 ns]
                        thrpt:  [217.43 Melem/s 217.67 Melem/s 217.89 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estimated 5.0000 s (1.7B iterationloss_simulator/should_drop_burst
                        time:   [2.9374 ns 2.9408 ns 2.9445 ns]
                        thrpt:  [339.62 Melem/s 340.05 Melem/s 340.44 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

circuit_breaker/allow_closed
                        time:   [13.471 ns 13.481 ns 13.491 ns]
                        thrpt:  [74.124 Melem/s 74.178 Melem/s 74.232 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
circuit_breaker/record_success
                        time:   [9.6464 ns 9.6531 ns 9.6596 ns]
                        thrpt:  [103.52 Melem/s 103.59 Melem/s 103.67 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
circuit_breaker/record_failure
                        time:   [9.6107 ns 9.6192 ns 9.6280 ns]
                        thrpt:  [103.86 Melem/s 103.96 Melem/s 104.05 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
circuit_breaker/state   time:   [13.814 ns 13.827 ns 13.839 ns]
                        thrpt:  [72.257 Melem/s 72.323 Melem/s 72.391 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samples in estimated 5.0013 s (19Mrecovery_manager/on_failure_with_alternates
                        time:   [271.78 ns 275.92 ns 279.40 ns]
                        thrpt:  [3.5791 Melem/s 3.6243 Melem/s 3.6794 Melem/s]
Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samples in estimated 5.0014 s (17M irecovery_manager/on_failure_no_alternates
                        time:   [285.06 ns 291.96 ns 299.27 ns]
                        thrpt:  [3.3415 Melem/s 3.4252 Melem/s 3.5080 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
recovery_manager/get_action
                        time:   [35.966 ns 36.084 ns 36.213 ns]
                        thrpt:  [27.615 Melem/s 27.713 Melem/s 27.804 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
recovery_manager/is_failed
                        time:   [12.185 ns 12.319 ns 12.453 ns]
                        thrpt:  [80.299 Melem/s 81.174 Melem/s 82.067 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
recovery_manager/on_recovery
                        time:   [105.71 ns 106.07 ns 106.52 ns]
                        thrpt:  [9.3875 Melem/s 9.4274 Melem/s 9.4594 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
recovery_manager/stats  time:   [702.29 ps 702.69 ps 703.10 ps]
                        thrpt:  [1.4223 Gelem/s 1.4231 Gelem/s 1.4239 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

failure_scaling/check_all/100
                        time:   [4.7779 µs 4.7979 µs 4.8131 µs]
                        thrpt:  [20.777 Melem/s 20.842 Melem/s 20.930 Melem/s]
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estimated 5.0021 s (2.9M iteratiofailure_scaling/healthy_nodes/100
                        time:   [1.6980 µs 1.7015 µs 1.7067 µs]
                        thrpt:  [58.593 Melem/s 58.771 Melem/s 58.893 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
failure_scaling/check_all/500
                        time:   [20.884 µs 20.948 µs 20.997 µs]
                        thrpt:  [23.813 Melem/s 23.869 Melem/s 23.942 Melem/s]
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estimated 5.0059 s (919k iteratiofailure_scaling/healthy_nodes/500
                        time:   [5.4450 µs 5.4484 µs 5.4517 µs]
                        thrpt:  [91.714 Melem/s 91.770 Melem/s 91.827 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
failure_scaling/check_all/1000
                        time:   [40.943 µs 41.121 µs 41.269 µs]
                        thrpt:  [24.231 Melem/s 24.318 Melem/s 24.424 Melem/s]
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in estimated 5.0174 s (490k iteratifailure_scaling/healthy_nodes/1000
                        time:   [10.235 µs 10.242 µs 10.249 µs]
                        thrpt:  [97.574 Melem/s 97.638 Melem/s 97.703 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
failure_scaling/check_all/5000
                        time:   [203.73 µs 204.15 µs 204.49 µs]
                        thrpt:  [24.451 Melem/s 24.492 Melem/s 24.542 Melem/s]
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in estimated 5.1972 s (106k iteratifailure_scaling/healthy_nodes/5000
                        time:   [48.976 µs 48.999 µs 49.023 µs]
                        thrpt:  [101.99 Melem/s 102.04 Melem/s 102.09 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samples in estimated 5.0247 s (27k itfailure_concurrent/concurrent_heartbeat/4
                        time:   [184.99 µs 185.37 µs 185.76 µs]
                        thrpt:  [10.766 Melem/s 10.789 Melem/s 10.812 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samples in estimated 5.0326 s (20k itfailure_concurrent/concurrent_heartbeat/8
                        time:   [256.99 µs 258.94 µs 261.99 µs]
                        thrpt:  [15.268 Melem/s 15.448 Melem/s 15.565 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  2 (10.00%) high mild
  2 (10.00%) high severe
Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samples in estimated 5.0027 s (11k ifailure_concurrent/concurrent_heartbeat/16
                        time:   [466.47 µs 467.35 µs 468.36 µs]
                        thrpt:  [17.081 Melem/s 17.118 Melem/s 17.150 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estimated 5.0016 s (16M iterationfailure_recovery_cycle/full_cycle
                        time:   [296.16 ns 303.14 ns 309.00 ns]
                        thrpt:  [3.2363 Melem/s 3.2989 Melem/s 3.3766 Melem/s]

capability_set/create   time:   [519.41 ns 520.96 ns 522.88 ns]
                        thrpt:  [1.9125 Melem/s 1.9195 Melem/s 1.9252 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_set/serialize
                        time:   [915.38 ns 915.93 ns 916.48 ns]
                        thrpt:  [1.0911 Melem/s 1.0918 Melem/s 1.0924 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
capability_set/deserialize
                        time:   [1.7230 µs 1.7249 µs 1.7265 µs]
                        thrpt:  [579.19 Kelem/s 579.75 Kelem/s 580.37 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_set/roundtrip
                        time:   [2.6794 µs 2.6818 µs 2.6838 µs]
                        thrpt:  [372.60 Kelem/s 372.89 Kelem/s 373.21 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_set/has_tag  time:   [748.92 ps 749.37 ps 749.81 ps]
                        thrpt:  [1.3337 Gelem/s 1.3345 Gelem/s 1.3352 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
capability_set/has_model
                        time:   [936.80 ps 937.63 ps 938.54 ps]
                        thrpt:  [1.0655 Gelem/s 1.0665 Gelem/s 1.0675 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
capability_set/has_tool time:   [748.96 ps 749.69 ps 750.50 ps]
                        thrpt:  [1.3324 Gelem/s 1.3339 Gelem/s 1.3352 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
capability_set/has_gpu  time:   [312.07 ps 312.29 ps 312.51 ps]
                        thrpt:  [3.1999 Gelem/s 3.2021 Gelem/s 3.2044 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

capability_announcement/create
                        time:   [373.97 ns 374.32 ns 374.64 ns]
                        thrpt:  [2.6692 Melem/s 2.6715 Melem/s 2.6740 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_announcement/serialize: Collecting 100 samples in estimated 5.0019 s (4.1M iteratiocapability_announcement/serialize
                        time:   [1.2195 µs 1.2213 µs 1.2237 µs]
                        thrpt:  [817.16 Kelem/s 818.81 Kelem/s 820.04 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in estimated 5.0082 s (2.3M iteratcapability_announcement/deserialize
                        time:   [2.1310 µs 2.1485 µs 2.1652 µs]
                        thrpt:  [461.84 Kelem/s 465.44 Kelem/s 469.27 Kelem/s]
Benchmarking capability_announcement/is_expired: Collecting 100 samples in estimated 5.0001 s (198M iteraticapability_announcement/is_expired
                        time:   [25.295 ns 25.327 ns 25.370 ns]
                        thrpt:  [39.416 Melem/s 39.483 Melem/s 39.534 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in estimated 5.0000 s (500M iteraticapability_filter/match_single_tag
                        time:   [9.9856 ns 9.9923 ns 9.9990 ns]
                        thrpt:  [100.01 Melem/s 100.08 Melem/s 100.14 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in estimated 5.0000 s (1.2B iteratcapability_filter/match_require_gpu
                        time:   [4.0583 ns 4.0679 ns 4.0831 ns]
                        thrpt:  [244.91 Melem/s 245.83 Melem/s 246.41 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in estimated 5.0000 s (1.3B iteraticapability_filter/match_gpu_vendor
                        time:   [3.7453 ns 3.7483 ns 3.7515 ns]
                        thrpt:  [266.56 Melem/s 266.79 Melem/s 267.00 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in estimated 5.0000 s (1.3B iteraticapability_filter/match_min_memory
                        time:   [3.7453 ns 3.7478 ns 3.7502 ns]
                        thrpt:  [266.65 Melem/s 266.83 Melem/s 267.00 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimated 5.0000 s (485M iterationscapability_filter/match_complex
                        time:   [10.300 ns 10.305 ns 10.311 ns]
                        thrpt:  [96.984 Melem/s 97.038 Melem/s 97.091 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_no_match: Collecting 100 samples in estimated 5.0000 s (1.6B iterationcapability_filter/match_no_match
                        time:   [3.1202 ns 3.1225 ns 3.1248 ns]
                        thrpt:  [320.02 Melem/s 320.26 Melem/s 320.49 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples in estimated 5.2291 s (45k itecapability_index_insert/index_nodes/100
                        time:   [114.53 µs 114.68 µs 114.83 µs]
                        thrpt:  [870.87 Kelem/s 872.00 Kelem/s 873.14 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samples in estimated 6.1451 s (5050 icapability_index_insert/index_nodes/1000
                        time:   [1.2156 ms 1.2188 ms 1.2221 ms]
                        thrpt:  [818.24 Kelem/s 820.47 Kelem/s 822.63 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samples in estimated 5.8284 s (300 icapability_index_insert/index_nodes/10000
                        time:   [19.491 ms 19.690 ms 19.894 ms]
                        thrpt:  [502.67 Kelem/s 507.87 Kelem/s 513.05 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples in estimated 5.7852 s (30k itecapability_index_query/query_single_tag
                        time:   [180.17 µs 184.06 µs 187.97 µs]
                        thrpt:  [5.3200 Kelem/s 5.4329 Kelem/s 5.5503 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_require_gpu: Collecting 100 samples in estimated 5.7527 s (25k itcapability_index_query/query_require_gpu
                        time:   [229.70 µs 239.69 µs 250.84 µs]
                        thrpt:  [3.9866 Kelem/s 4.1720 Kelem/s 4.3534 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples in estimated 7.6292 s (10k itecapability_index_query/query_gpu_vendor
                        time:   [736.80 µs 747.74 µs 758.61 µs]
                        thrpt:  [1.3182 Kelem/s 1.3374 Kelem/s 1.3572 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples in estimated 7.9806 s (10k itecapability_index_query/query_min_memory
                        time:   [768.58 µs 779.25 µs 789.83 µs]
                        thrpt:  [1.2661 Kelem/s 1.2833 Kelem/s 1.3011 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_query/query_complex: Collecting 100 samples in estimated 5.1515 s (10k iteratcapability_index_query/query_complex
                        time:   [502.25 µs 515.85 µs 529.44 µs]
                        thrpt:  [1.8888 Kelem/s 1.9386 Kelem/s 1.9910 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_query/query_model: Collecting 100 samples in estimated 5.0952 s (50k iteratiocapability_index_query/query_model
                        time:   [100.26 µs 100.41 µs 100.55 µs]
                        thrpt:  [9.9455 Kelem/s 9.9596 Kelem/s 9.9742 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estimated 6.3327 s (10k iterationcapability_index_query/query_tool
                        time:   [588.26 µs 604.28 µs 619.68 µs]
                        thrpt:  [1.6137 Kelem/s 1.6549 Kelem/s 1.6999 Kelem/s]
Benchmarking capability_index_query/query_no_results: Collecting 100 samples in estimated 5.0001 s (219M itcapability_index_query/query_no_results
                        time:   [22.721 ns 22.764 ns 22.811 ns]
                        thrpt:  [43.839 Melem/s 43.929 Melem/s 44.012 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking capability_index_find_best/find_best_simple: Collecting 100 samples in estimated 6.6958 s (15kcapability_index_find_best/find_best_simple
                        time:   [447.18 µs 456.38 µs 465.76 µs]
                        thrpt:  [2.1470 Kelem/s 2.1912 Kelem/s 2.2362 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 8.0890 s capability_index_find_best/find_best_with_prefs
                        time:   [769.85 µs 781.65 µs 793.39 µs]
                        thrpt:  [1.2604 Kelem/s 1.2793 Kelem/s 1.2990 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples in estimated 5.0190 s (399k itcapability_index_scaling/query_tag/1000
                        time:   [12.564 µs 12.573 µs 12.582 µs]
                        thrpt:  [79.477 Kelem/s 79.536 Kelem/s 79.594 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 samples in estimated 5.1329 s (126capability_index_scaling/query_complex/1000
                        time:   [40.533 µs 40.690 µs 40.842 µs]
                        thrpt:  [24.485 Kelem/s 24.576 Kelem/s 24.671 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples in estimated 5.3312 s (76k itecapability_index_scaling/query_tag/5000
                        time:   [70.203 µs 70.294 µs 70.384 µs]
                        thrpt:  [14.208 Kelem/s 14.226 Kelem/s 14.244 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 samples in estimated 5.1638 s (25kcapability_index_scaling/query_complex/5000
                        time:   [200.31 µs 201.85 µs 203.42 µs]
                        thrpt:  [4.9159 Kelem/s 4.9541 Kelem/s 4.9922 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samples in estimated 5.8814 s (20k itcapability_index_scaling/query_tag/10000
                        time:   [283.46 µs 294.00 µs 304.67 µs]
                        thrpt:  [3.2822 Kelem/s 3.4014 Kelem/s 3.5279 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samples in estimated 6.8793 s (10capability_index_scaling/query_complex/10000
                        time:   [675.52 µs 680.21 µs 684.94 µs]
                        thrpt:  [1.4600 Kelem/s 1.4701 Kelem/s 1.4803 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samples in estimated 5.1364 s (1900 icapability_index_scaling/query_tag/50000
                        time:   [2.6925 ms 2.7127 ms 2.7331 ms]
                        thrpt:  [365.89  elem/s 368.63  elem/s 371.40  elem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samples in estimated 5.2389 s (13capability_index_scaling/query_complex/50000
                        time:   [3.9568 ms 3.9879 ms 4.0185 ms]
                        thrpt:  [248.85  elem/s 250.76  elem/s 252.73  elem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild

Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 samples in estimated 5.0553 s (1capability_index_concurrent/concurrent_index/4
                        time:   [394.28 µs 394.70 µs 395.19 µs]
                        thrpt:  [5.0608 Melem/s 5.0672 Melem/s 5.0725 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.3s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 samples in estimated 9.3449 s (2capability_index_concurrent/concurrent_query/4
                        time:   [466.51 ms 470.11 ms 474.10 ms]
                        thrpt:  [4.2185 Kelem/s 4.2544 Kelem/s 4.2871 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 7.1395 s (4capability_index_concurrent/concurrent_mixed/4
                        time:   [176.36 ms 180.53 ms 184.67 ms]
                        thrpt:  [10.830 Kelem/s 11.079 Kelem/s 11.340 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 samples in estimated 5.0499 s (1capability_index_concurrent/concurrent_index/8
                        time:   [452.37 µs 453.22 µs 454.03 µs]
                        thrpt:  [8.8099 Melem/s 8.8257 Melem/s 8.8423 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.6s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 samples in estimated 10.644 s (2capability_index_concurrent/concurrent_query/8
                        time:   [531.63 ms 533.76 ms 535.96 ms]
                        thrpt:  [7.4632 Kelem/s 7.4940 Kelem/s 7.5240 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.1s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 5.1478 s (2capability_index_concurrent/concurrent_mixed/8
                        time:   [257.67 ms 258.55 ms 259.50 ms]
                        thrpt:  [15.414 Kelem/s 15.471 Kelem/s 15.524 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.1493 s (capability_index_concurrent/concurrent_index/16
                        time:   [907.07 µs 908.31 µs 909.42 µs]
                        thrpt:  [8.7968 Melem/s 8.8075 Melem/s 8.8196 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 20.5s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 samples in estimated 20.539 s (capability_index_concurrent/concurrent_query/16
                        time:   [1.0167 s 1.0204 s 1.0259 s]
                        thrpt:  [7.7979 Kelem/s 7.8404 Kelem/s 7.8683 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.6s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 10.582 s (capability_index_concurrent/concurrent_mixed/16
                        time:   [534.96 ms 535.74 ms 536.62 ms]
                        thrpt:  [14.908 Kelem/s 14.932 Kelem/s 14.954 Kelem/s]

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.0039 s (capability_index_updates/update_higher_version
                        time:   [561.17 ns 561.91 ns 562.66 ns]
                        thrpt:  [1.7773 Melem/s 1.7796 Melem/s 1.7820 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_updates/update_same_version: Collecting 100 samples in estimated 5.0015 s (8.capability_index_updates/update_same_version
                        time:   [559.12 ns 560.03 ns 561.02 ns]
                        thrpt:  [1.7825 Melem/s 1.7856 Melem/s 1.7885 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samples in estimated 5.0006 s (3.6M capability_index_updates/remove_and_readd
                        time:   [1.3823 µs 1.3919 µs 1.4013 µs]
                        thrpt:  [713.60 Kelem/s 718.42 Kelem/s 723.45 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild

diff_op/create_add_tag  time:   [14.347 ns 14.375 ns 14.406 ns]
                        thrpt:  [69.418 Melem/s 69.566 Melem/s 69.702 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high severe
diff_op/create_remove_tag
                        time:   [14.623 ns 14.678 ns 14.732 ns]
                        thrpt:  [67.880 Melem/s 68.129 Melem/s 68.385 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
diff_op/create_add_model
                        time:   [48.265 ns 48.317 ns 48.372 ns]
                        thrpt:  [20.673 Melem/s 20.697 Melem/s 20.719 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
diff_op/create_update_model
                        time:   [15.227 ns 15.305 ns 15.377 ns]
                        thrpt:  [65.032 Melem/s 65.340 Melem/s 65.672 Melem/s]
diff_op/estimated_size  time:   [1.9219 ns 1.9235 ns 1.9249 ns]
                        thrpt:  [519.51 Melem/s 519.89 Melem/s 520.31 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

capability_diff/create  time:   [102.02 ns 102.28 ns 102.53 ns]
                        thrpt:  [9.7528 Melem/s 9.7773 Melem/s 9.8016 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_diff/serialize
                        time:   [150.84 ns 151.05 ns 151.36 ns]
                        thrpt:  [6.6069 Melem/s 6.6205 Melem/s 6.6295 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
capability_diff/deserialize
                        time:   [247.85 ns 248.62 ns 249.35 ns]
                        thrpt:  [4.0104 Melem/s 4.0223 Melem/s 4.0347 Melem/s]
capability_diff/estimated_size
                        time:   [5.3058 ns 5.3089 ns 5.3121 ns]
                        thrpt:  [188.25 Melem/s 188.36 Melem/s 188.47 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

diff_generation/no_changes
                        time:   [347.30 ns 348.06 ns 348.90 ns]
                        thrpt:  [2.8662 Melem/s 2.8730 Melem/s 2.8794 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
diff_generation/add_one_tag
                        time:   [460.79 ns 463.52 ns 466.56 ns]
                        thrpt:  [2.1433 Melem/s 2.1574 Melem/s 2.1702 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in estimated 5.0024 s (9.7M iteradiff_generation/multiple_tag_changes
                        time:   [511.63 ns 513.28 ns 515.11 ns]
                        thrpt:  [1.9413 Melem/s 1.9482 Melem/s 1.9545 Melem/s]
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in estimated 5.0013 s (12M iteratidiff_generation/update_model_loaded
                        time:   [409.04 ns 410.18 ns 411.43 ns]
                        thrpt:  [2.4305 Melem/s 2.4379 Melem/s 2.4447 Melem/s]
diff_generation/update_memory
                        time:   [396.58 ns 398.16 ns 399.92 ns]
                        thrpt:  [2.5005 Melem/s 2.5116 Melem/s 2.5215 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
diff_generation/add_model
                        time:   [490.39 ns 491.68 ns 493.12 ns]
                        thrpt:  [2.0279 Melem/s 2.0338 Melem/s 2.0392 Melem/s]
diff_generation/complex_diff
                        time:   [780.22 ns 782.32 ns 784.94 ns]
                        thrpt:  [1.2740 Melem/s 1.2782 Melem/s 1.2817 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estimated 5.0013 s (14M iterationsdiff_application/apply_single_op
                        time:   [360.81 ns 361.13 ns 361.46 ns]
                        thrpt:  [2.7666 Melem/s 2.7691 Melem/s 2.7716 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estimated 5.0007 s (14M iterationdiff_application/apply_small_diff
                        time:   [365.39 ns 365.67 ns 365.95 ns]
                        thrpt:  [2.7327 Melem/s 2.7347 Melem/s 2.7368 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in estimated 5.0010 s (5.9M iteratidiff_application/apply_medium_diff
                        time:   [841.47 ns 842.16 ns 842.83 ns]
                        thrpt:  [1.1865 Melem/s 1.1874 Melem/s 1.1884 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in estimated 5.0013 s (14M iteratiodiff_application/apply_strict_mode
                        time:   [363.15 ns 363.41 ns 363.67 ns]
                        thrpt:  [2.7497 Melem/s 2.7517 Melem/s 2.7537 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples in estimated 5.0000 s (826M itdiff_chain_validation/validate_chain_10
                        time:   [6.0471 ns 6.0513 ns 6.0556 ns]
                        thrpt:  [165.14 Melem/s 165.25 Melem/s 165.37 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samples in estimated 5.0001 s (74M itdiff_chain_validation/validate_chain_100
                        time:   [67.715 ns 67.765 ns 67.816 ns]
                        thrpt:  [14.746 Melem/s 14.757 Melem/s 14.768 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimated 5.0068 s (1.5M iterationsdiff_compaction/compact_5_diffs
                        time:   [3.2576 µs 3.2598 µs 3.2621 µs]
                        thrpt:  [306.55 Kelem/s 306.76 Kelem/s 306.98 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estimated 5.0501 s (338k iterationdiff_compaction/compact_20_diffs
                        time:   [14.957 µs 14.969 µs 14.981 µs]
                        thrpt:  [66.750 Kelem/s 66.804 Kelem/s 66.857 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estimated 5.0002 s (5.5M iteratiobandwidth_savings/calculate_small
                        time:   [901.81 ns 902.58 ns 903.46 ns]
                        thrpt:  [1.1069 Melem/s 1.1079 Melem/s 1.1089 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in estimated 5.0040 s (5.5M iteratibandwidth_savings/calculate_medium
                        time:   [908.12 ns 908.81 ns 909.50 ns]
                        thrpt:  [1.0995 Melem/s 1.1003 Melem/s 1.1012 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in estimated 5.0012 s (3.7M iteradiff_roundtrip/generate_apply_verify
                        time:   [1.3444 µs 1.3457 µs 1.3469 µs]
                        thrpt:  [742.42 Kelem/s 743.12 Kelem/s 743.80 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

location_info/create    time:   [28.239 ns 28.272 ns 28.305 ns]
                        thrpt:  [35.330 Melem/s 35.371 Melem/s 35.411 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
location_info/distance_to
                        time:   [3.9378 ns 3.9408 ns 3.9436 ns]
                        thrpt:  [253.58 Melem/s 253.76 Melem/s 253.95 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  5 (5.00%) high severe
location_info/same_continent
                        time:   [7.1780 ns 7.1823 ns 7.1867 ns]
                        thrpt:  [139.15 Melem/s 139.23 Melem/s 139.31 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in estimated 5.0000 s (16B iteratiolocation_info/same_continent_cross
                        time:   [312.04 ps 312.24 ps 312.44 ps]
                        thrpt:  [3.2006 Gelem/s 3.2026 Gelem/s 3.2047 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
location_info/same_region
                        time:   [4.0612 ns 4.0657 ns 4.0706 ns]
                        thrpt:  [245.67 Melem/s 245.96 Melem/s 246.23 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe

topology_hints/create   time:   [3.1479 ns 3.1574 ns 3.1686 ns]
                        thrpt:  [315.60 Melem/s 316.72 Melem/s 317.67 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in estimated 5.0000 s (16B iterationtopology_hints/connectivity_score
                        time:   [312.10 ps 312.36 ps 312.66 ps]
                        thrpt:  [3.1984 Gelem/s 3.2014 Gelem/s 3.2041 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in estimated 5.0000 s (8.0B iteratopology_hints/average_latency_empty
                        time:   [625.02 ps 632.32 ps 643.21 ps]
                        thrpt:  [1.5547 Gelem/s 1.5815 Gelem/s 1.5999 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estimated 5.0002 s (70M iteratiotopology_hints/average_latency_100
                        time:   [70.804 ns 70.881 ns 70.964 ns]
                        thrpt:  [14.092 Melem/s 14.108 Melem/s 14.124 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

nat_type/difficulty     time:   [312.53 ps 313.89 ps 315.97 ps]
                        thrpt:  [3.1648 Gelem/s 3.1858 Gelem/s 3.1997 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
nat_type/can_connect_direct
                        time:   [312.09 ps 312.34 ps 312.60 ps]
                        thrpt:  [3.1990 Gelem/s 3.2017 Gelem/s 3.2042 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
nat_type/can_connect_symmetric
                        time:   [312.47 ps 313.72 ps 316.07 ps]
                        thrpt:  [3.1639 Gelem/s 3.1875 Gelem/s 3.2004 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

node_metadata/create_simple
                        time:   [45.545 ns 45.588 ns 45.635 ns]
                        thrpt:  [21.913 Melem/s 21.936 Melem/s 21.956 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
node_metadata/create_full
                        time:   [357.80 ns 360.65 ns 366.28 ns]
                        thrpt:  [2.7301 Melem/s 2.7727 Melem/s 2.7948 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
node_metadata/routing_score
                        time:   [2.8725 ns 2.8754 ns 2.8787 ns]
                        thrpt:  [347.38 Melem/s 347.78 Melem/s 348.13 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
node_metadata/age       time:   [27.553 ns 27.583 ns 27.615 ns]
                        thrpt:  [36.212 Melem/s 36.254 Melem/s 36.294 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
node_metadata/is_stale  time:   [25.882 ns 25.924 ns 25.979 ns]
                        thrpt:  [38.492 Melem/s 38.574 Melem/s 38.637 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
node_metadata/serialize time:   [743.89 ns 744.61 ns 745.37 ns]
                        thrpt:  [1.3416 Melem/s 1.3430 Melem/s 1.3443 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
node_metadata/deserialize
                        time:   [1.4143 µs 1.4225 µs 1.4388 µs]
                        thrpt:  [695.02 Kelem/s 703.00 Kelem/s 707.08 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

metadata_query/match_status
                        time:   [3.4357 ns 3.4391 ns 3.4428 ns]
                        thrpt:  [290.46 Melem/s 290.77 Melem/s 291.06 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
metadata_query/match_min_tier
                        time:   [3.4392 ns 3.4437 ns 3.4487 ns]
                        thrpt:  [289.96 Melem/s 290.39 Melem/s 290.76 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
metadata_query/match_continent
                        time:   [11.239 ns 11.250 ns 11.261 ns]
                        thrpt:  [88.801 Melem/s 88.892 Melem/s 88.977 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
metadata_query/match_complex
                        time:   [10.613 ns 10.623 ns 10.634 ns]
                        thrpt:  [94.041 Melem/s 94.133 Melem/s 94.220 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
metadata_query/match_no_match
                        time:   [3.4362 ns 3.4450 ns 3.4577 ns]
                        thrpt:  [289.21 Melem/s 290.28 Melem/s 291.02 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

metadata_store_basic/create
                        time:   [764.45 ns 765.63 ns 766.93 ns]
                        thrpt:  [1.3039 Melem/s 1.3061 Melem/s 1.3081 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimated 5.0063 s (2.3M iterationsmetadata_store_basic/upsert_new
                        time:   [2.3842 µs 2.4165 µs 2.4523 µs]
                        thrpt:  [407.79 Kelem/s 413.81 Kelem/s 419.42 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) low mild
  1 (1.00%) high mild
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in estimated 5.0003 s (4.4M iterametadata_store_basic/upsert_existing
                        time:   [1.1459 µs 1.1474 µs 1.1490 µs]
                        thrpt:  [870.30 Kelem/s 871.53 Kelem/s 872.65 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
metadata_store_basic/get
                        time:   [25.763 ns 26.113 ns 26.473 ns]
                        thrpt:  [37.774 Melem/s 38.295 Melem/s 38.816 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_basic/get_miss
                        time:   [25.747 ns 26.194 ns 26.650 ns]
                        thrpt:  [37.523 Melem/s 38.177 Melem/s 38.839 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_basic/len
                        time:   [200.75 ns 200.99 ns 201.25 ns]
                        thrpt:  [4.9689 Melem/s 4.9754 Melem/s 4.9814 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 22.0s, or reduce sample count to 20.
metadata_store_basic/stats
                        time:   [218.45 ms 218.75 ms 219.14 ms]
                        thrpt:  [4.5632  elem/s 4.5714  elem/s 4.5776  elem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in estimated 5.5188 s (25k iteratmetadata_store_query/query_by_status
                        time:   [211.88 µs 214.63 µs 217.72 µs]
                        thrpt:  [4.5930 Kelem/s 4.6593 Kelem/s 4.7197 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples in estimated 5.2121 s (35k itemetadata_store_query/query_by_continent
                        time:   [147.03 µs 147.38 µs 147.75 µs]
                        thrpt:  [6.7680 Kelem/s 6.7853 Kelem/s 6.8016 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in estimated 5.8288 s (15k iteratiometadata_store_query/query_by_tier
                        time:   [380.31 µs 385.45 µs 392.07 µs]
                        thrpt:  [2.5506 Kelem/s 2.5944 Kelem/s 2.6294 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samples in estimated 6.2456 s (15k imetadata_store_query/query_accepting_work
                        time:   [416.13 µs 425.17 µs 436.52 µs]
                        thrpt:  [2.2908 Kelem/s 2.3520 Kelem/s 2.4031 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples in estimated 5.5300 s (15k iterametadata_store_query/query_with_limit
                        time:   [358.87 µs 363.65 µs 368.86 µs]
                        thrpt:  [2.7111 Kelem/s 2.7499 Kelem/s 2.7865 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in estimated 5.6509 s (20k iteratiometadata_store_query/query_complex
                        time:   [278.07 µs 279.19 µs 280.42 µs]
                        thrpt:  [3.5661 Kelem/s 3.5818 Kelem/s 3.5962 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samples in estimated 5.0578 s (15k itmetadata_store_spatial/find_nearby_100km
                        time:   [331.19 µs 332.98 µs 334.78 µs]
                        thrpt:  [2.9870 Kelem/s 3.0032 Kelem/s 3.0194 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samples in estimated 6.2015 s (15k imetadata_store_spatial/find_nearby_1000km
                        time:   [405.16 µs 407.33 µs 409.61 µs]
                        thrpt:  [2.4413 Kelem/s 2.4550 Kelem/s 2.4682 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samples in estimated 6.7063 s (15k imetadata_store_spatial/find_nearby_5000km
                        time:   [455.92 µs 463.82 µs 473.03 µs]
                        thrpt:  [2.1140 Kelem/s 2.1560 Kelem/s 2.1933 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 samples in estimated 6.2270 s (25metadata_store_spatial/find_best_for_routing
                        time:   [239.11 µs 243.80 µs 249.33 µs]
                        thrpt:  [4.0108 Kelem/s 4.1018 Kelem/s 4.1821 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in estimated 5.0843 s (10k iteratiometadata_store_spatial/find_relays
                        time:   [470.89 µs 478.01 µs 485.73 µs]
                        thrpt:  [2.0587 Kelem/s 2.0920 Kelem/s 2.1236 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samples in estimated 5.0106 s (268k imetadata_store_scaling/query_status/1000
                        time:   [18.608 µs 18.636 µs 18.665 µs]
                        thrpt:  [53.575 Kelem/s 53.660 Kelem/s 53.740 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0373 s (237k metadata_store_scaling/query_complex/1000
                        time:   [21.237 µs 21.260 µs 21.284 µs]
                        thrpt:  [46.983 Kelem/s 47.036 Kelem/s 47.087 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples in estimated 5.1451 s (96k itemetadata_store_scaling/find_nearby/1000
                        time:   [53.109 µs 53.188 µs 53.271 µs]
                        thrpt:  [18.772 Kelem/s 18.801 Kelem/s 18.829 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samples in estimated 5.0076 s (50k itmetadata_store_scaling/query_status/5000
                        time:   [99.550 µs 101.17 µs 103.67 µs]
                        thrpt:  [9.6457 Kelem/s 9.8844 Kelem/s 10.045 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samples in estimated 5.4704 s (45k imetadata_store_scaling/query_complex/5000
                        time:   [120.22 µs 120.47 µs 120.74 µs]
                        thrpt:  [8.2822 Kelem/s 8.3005 Kelem/s 8.3184 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples in estimated 5.4885 s (20k itemetadata_store_scaling/find_nearby/5000
                        time:   [271.55 µs 273.94 µs 276.49 µs]
                        thrpt:  [3.6168 Kelem/s 3.6504 Kelem/s 3.6826 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samples in estimated 5.6019 s (25k imetadata_store_scaling/query_status/10000
                        time:   [211.85 µs 213.79 µs 216.11 µs]
                        thrpt:  [4.6272 Kelem/s 4.6775 Kelem/s 4.7203 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samples in estimated 5.2731 s (20k metadata_store_scaling/query_complex/10000
                        time:   [259.59 µs 261.97 µs 264.55 µs]
                        thrpt:  [3.7800 Kelem/s 3.8173 Kelem/s 3.8522 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samples in estimated 5.5971 s (10k itmetadata_store_scaling/find_nearby/10000
                        time:   [548.73 µs 565.11 µs 585.13 µs]
                        thrpt:  [1.7090 Kelem/s 1.7696 Kelem/s 1.8224 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samples in estimated 5.3034 s (1500 metadata_store_scaling/query_status/50000
                        time:   [3.2056 ms 3.2758 ms 3.3473 ms]
                        thrpt:  [298.75  elem/s 305.27  elem/s 311.95  elem/s]
Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samples in estimated 5.2931 s (1500metadata_store_scaling/query_complex/50000
                        time:   [3.3193 ms 3.3889 ms 3.4639 ms]
                        thrpt:  [288.69  elem/s 295.08  elem/s 301.27  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samples in estimated 5.3172 s (1500 imetadata_store_scaling/find_nearby/50000
                        time:   [3.4955 ms 3.5729 ms 3.6513 ms]
                        thrpt:  [273.87  elem/s 279.88  elem/s 286.08  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 samples in estimated 5.1006 s (29metadata_store_concurrent/concurrent_upsert/4
                        time:   [1.7175 ms 1.7330 ms 1.7442 ms]
                        thrpt:  [1.1467 Melem/s 1.1541 Melem/s 1.1645 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.9s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 samples in estimated 7.8863 s (20 metadata_store_concurrent/concurrent_query/4
                        time:   [370.52 ms 387.62 ms 404.17 ms]
                        thrpt:  [4.9484 Kelem/s 5.1597 Kelem/s 5.3978 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.1s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 7.0679 s (20 metadata_store_concurrent/concurrent_mixed/4
                        time:   [334.68 ms 344.28 ms 354.71 ms]
                        thrpt:  [5.6384 Kelem/s 5.8092 Kelem/s 5.9758 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 samples in estimated 5.2074 s (16metadata_store_concurrent/concurrent_upsert/8
                        time:   [3.0949 ms 3.1349 ms 3.2120 ms]
                        thrpt:  [1.2453 Melem/s 1.2760 Melem/s 1.2924 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.0s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 samples in estimated 15.993 s (20 metadata_store_concurrent/concurrent_query/8
                        time:   [798.08 ms 803.28 ms 808.48 ms]
                        thrpt:  [4.9476 Kelem/s 4.9796 Kelem/s 5.0120 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low mild
  3 (15.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 18.0s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 17.959 s (20 metadata_store_concurrent/concurrent_mixed/8
                        time:   [871.77 ms 882.84 ms 895.00 ms]
                        thrpt:  [4.4693 Kelem/s 4.5309 Kelem/s 4.5883 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 samples in estimated 5.9693 s (8metadata_store_concurrent/concurrent_upsert/16
                        time:   [7.0332 ms 7.0558 ms 7.0811 ms]
                        thrpt:  [1.1298 Melem/s 1.1338 Melem/s 1.1375 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 28.9s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 samples in estimated 28.897 s (20metadata_store_concurrent/concurrent_query/16
                        time:   [1.4287 s 1.4336 s 1.4388 s]
                        thrpt:  [5.5603 Kelem/s 5.5805 Kelem/s 5.5997 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 33.9s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 33.925 s (20metadata_store_concurrent/concurrent_mixed/16
                        time:   [1.7041 s 1.7085 s 1.7144 s]
                        thrpt:  [4.6665 Kelem/s 4.6825 Kelem/s 4.6945 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimated 5.0012metadata_store_versioning/update_versioned_success
                        time:   [178.64 ns 179.46 ns 180.38 ns]
                        thrpt:  [5.5439 Melem/s 5.5723 Melem/s 5.5977 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimated 5.000metadata_store_versioning/update_versioned_conflict
                        time:   [177.53 ns 178.04 ns 178.62 ns]
                        thrpt:  [5.5985 Melem/s 5.6169 Melem/s 5.6328 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estimated 5.0000 s (1.4B iteratioschema_validation/validate_string
                        time:   [3.4805 ns 3.4836 ns 3.4868 ns]
                        thrpt:  [286.79 Melem/s 287.06 Melem/s 287.31 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking schema_validation/validate_integer: Collecting 100 samples in estimated 5.0000 s (1.4B iteratischema_validation/validate_integer
                        time:   [3.4757 ns 3.5025 ns 3.5575 ns]
                        thrpt:  [281.09 Melem/s 285.51 Melem/s 287.71 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking schema_validation/validate_object: Collecting 100 samples in estimated 5.0001 s (65M iterationschema_validation/validate_object
                        time:   [76.552 ns 76.677 ns 76.806 ns]
                        thrpt:  [13.020 Melem/s 13.042 Melem/s 13.063 Melem/s]
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in estimated 5.0002 s (138M iteratschema_validation/validate_array_10
                        time:   [36.208 ns 36.252 ns 36.296 ns]
                        thrpt:  [27.551 Melem/s 27.584 Melem/s 27.618 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking schema_validation/validate_complex: Collecting 100 samples in estimated 5.0003 s (24M iteratioschema_validation/validate_complex
                        time:   [205.89 ns 206.52 ns 207.28 ns]
                        thrpt:  [4.8243 Melem/s 4.8421 Melem/s 4.8571 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

endpoint_matching/match_success
                        time:   [171.34 ns 173.49 ns 177.18 ns]
                        thrpt:  [5.6440 Melem/s 5.7640 Melem/s 5.8363 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
endpoint_matching/match_failure
                        time:   [171.10 ns 171.23 ns 171.38 ns]
                        thrpt:  [5.8351 Melem/s 5.8402 Melem/s 5.8446 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in estimated 5.0018 s (13M iteratiendpoint_matching/match_multi_param
                        time:   [370.52 ns 370.85 ns 371.23 ns]
                        thrpt:  [2.6938 Melem/s 2.6965 Melem/s 2.6989 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe

api_version/is_compatible_with
                        time:   [311.28 ps 311.51 ps 311.79 ps]
                        thrpt:  [3.2073 Gelem/s 3.2102 Gelem/s 3.2125 Gelem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) high mild
  12 (12.00%) high severe
api_version/parse       time:   [35.484 ns 35.507 ns 35.534 ns]
                        thrpt:  [28.142 Melem/s 28.163 Melem/s 28.182 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
api_version/to_string   time:   [43.418 ns 43.471 ns 43.546 ns]
                        thrpt:  [22.964 Melem/s 23.004 Melem/s 23.032 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe

api_schema/create       time:   [1.2226 µs 1.2239 µs 1.2252 µs]
                        thrpt:  [816.21 Kelem/s 817.09 Kelem/s 817.91 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
api_schema/serialize    time:   [1.8944 µs 1.9066 µs 1.9313 µs]
                        thrpt:  [517.78 Kelem/s 524.49 Kelem/s 527.88 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
api_schema/deserialize  time:   [5.1558 µs 5.1759 µs 5.1999 µs]
                        thrpt:  [192.31 Kelem/s 193.20 Kelem/s 193.96 Kelem/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe
api_schema/find_endpoint
                        time:   [197.65 ns 197.77 ns 197.91 ns]
                        thrpt:  [5.0528 Melem/s 5.0563 Melem/s 5.0595 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
api_schema/endpoints_by_tag
                        time:   [57.062 ns 57.106 ns 57.147 ns]
                        thrpt:  [17.499 Melem/s 17.511 Melem/s 17.525 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking request_validation/validate_full_request: Collecting 100 samples in estimated 5.0000 s (70M itrequest_validation/validate_full_request
                        time:   [71.389 ns 71.454 ns 71.524 ns]
                        thrpt:  [13.981 Melem/s 13.995 Melem/s 14.008 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking request_validation/validate_path_only: Collecting 100 samples in estimated 5.0000 s (233M iterrequest_validation/validate_path_only
                        time:   [21.393 ns 21.451 ns 21.507 ns]
                        thrpt:  [46.496 Melem/s 46.617 Melem/s 46.745 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild

api_registry_basic/create
                        time:   [413.79 ns 414.64 ns 415.49 ns]
                        thrpt:  [2.4068 Melem/s 2.4117 Melem/s 2.4167 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimated 5.0078 s (1.4M iterationsapi_registry_basic/register_new
                        time:   [4.3545 µs 4.4061 µs 4.4552 µs]
                        thrpt:  [224.46 Kelem/s 226.96 Kelem/s 229.65 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
api_registry_basic/get  time:   [25.410 ns 25.821 ns 26.242 ns]
                        thrpt:  [38.107 Melem/s 38.728 Melem/s 39.354 Melem/s]
api_registry_basic/len  time:   [199.80 ns 199.87 ns 199.96 ns]
                        thrpt:  [5.0011 Melem/s 5.0032 Melem/s 5.0051 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 24.5s, or reduce sample count to 20.
api_registry_basic/stats
                        time:   [238.83 ms 241.13 ms 243.46 ms]
                        thrpt:  [4.1075  elem/s 4.1472  elem/s 4.1870  elem/s]

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estimated 5.2706 s (50k iterationsapi_registry_query/query_by_name
                        time:   [103.78 µs 104.14 µs 104.47 µs]
                        thrpt:  [9.5723 Kelem/s 9.6024 Kelem/s 9.6357 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
api_registry_query/query_by_tag
                        time:   [562.48 µs 564.08 µs 565.92 µs]
                        thrpt:  [1.7670 Kelem/s 1.7728 Kelem/s 1.7778 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_query/query_with_version: Collecting 100 samples in estimated 5.0691 s (86k iteraapi_registry_query/query_with_version
                        time:   [59.083 µs 59.106 µs 59.131 µs]
                        thrpt:  [16.912 Kelem/s 16.919 Kelem/s 16.925 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in estimated 5.0912 s (1700 iteratapi_registry_query/find_by_endpoint
                        time:   [3.0679 ms 3.1007 ms 3.1332 ms]
                        thrpt:  [319.16  elem/s 322.51  elem/s 325.95  elem/s]
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in estimated 5.0873 s (76k iteratioapi_registry_query/find_compatible
                        time:   [67.655 µs 67.785 µs 67.932 µs]
                        thrpt:  [14.721 Kelem/s 14.753 Kelem/s 14.781 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples in estimated 5.0270 s (631k itapi_registry_scaling/query_by_name/1000
                        time:   [7.9882 µs 8.0105 µs 8.0298 µs]
                        thrpt:  [124.54 Kelem/s 124.84 Kelem/s 125.18 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples in estimated 5.0505 s (101k iteapi_registry_scaling/query_by_tag/1000
                        time:   [50.030 µs 50.065 µs 50.099 µs]
                        thrpt:  [19.960 Kelem/s 19.974 Kelem/s 19.988 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples in estimated 5.1610 s (106k itapi_registry_scaling/query_by_name/5000
                        time:   [48.557 µs 48.633 µs 48.708 µs]
                        thrpt:  [20.530 Kelem/s 20.562 Kelem/s 20.594 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples in estimated 5.3740 s (20k iterapi_registry_scaling/query_by_tag/5000
                        time:   [265.25 µs 265.67 µs 266.09 µs]
                        thrpt:  [3.7582 Kelem/s 3.7641 Kelem/s 3.7701 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samples in estimated 5.2086 s (50k itapi_registry_scaling/query_by_name/10000
                        time:   [101.49 µs 101.89 µs 102.32 µs]
                        thrpt:  [9.7737 Kelem/s 9.8145 Kelem/s 9.8531 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples in estimated 5.6419 s (10k iteapi_registry_scaling/query_by_tag/10000
                        time:   [558.05 µs 559.10 µs 560.23 µs]
                        thrpt:  [1.7850 Kelem/s 1.7886 Kelem/s 1.7919 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.2s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 samples in estimated 16.191 s (20 itapi_registry_concurrent/concurrent_query/4
                        time:   [805.10 ms 811.98 ms 818.94 ms]
                        thrpt:  [2.4422 Kelem/s 2.4631 Kelem/s 2.4842 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.3s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 10.344 s (20 itapi_registry_concurrent/concurrent_mixed/4
                        time:   [503.25 ms 509.32 ms 515.20 ms]
                        thrpt:  [3.8820 Kelem/s 3.9268 Kelem/s 3.9742 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 21.8s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 samples in estimated 21.817 s (20 itapi_registry_concurrent/concurrent_query/8
                        time:   [1.0911 s 1.0946 s 1.0977 s]
                        thrpt:  [3.6441 Kelem/s 3.6544 Kelem/s 3.6662 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 18.5s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 18.471 s (20 itapi_registry_concurrent/concurrent_mixed/8
                        time:   [918.60 ms 935.50 ms 950.52 ms]
                        thrpt:  [4.2082 Kelem/s 4.2758 Kelem/s 4.3544 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) low mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 35.5s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samples in estimated 35.550 s (20 iapi_registry_concurrent/concurrent_query/16
                        time:   [1.7885 s 1.7928 s 1.7973 s]
                        thrpt:  [4.4512 Kelem/s 4.4624 Kelem/s 4.4729 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 32.2s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 32.219 s (20 iapi_registry_concurrent/concurrent_mixed/16
                        time:   [1.6331 s 1.6401 s 1.6467 s]
                        thrpt:  [4.8582 Kelem/s 4.8779 Kelem/s 4.8985 Kelem/s]

compare_op/eq           time:   [1.9797 ns 1.9802 ns 1.9806 ns]
                        thrpt:  [504.89 Melem/s 505.01 Melem/s 505.13 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  7 (7.00%) high severe
compare_op/gt           time:   [1.2458 ns 1.2468 ns 1.2480 ns]
                        thrpt:  [801.28 Melem/s 802.07 Melem/s 802.73 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) high mild
  12 (12.00%) high severe
compare_op/contains_string
                        time:   [25.213 ns 25.222 ns 25.232 ns]
                        thrpt:  [39.632 Melem/s 39.647 Melem/s 39.662 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
compare_op/in_array     time:   [6.8491 ns 6.8531 ns 6.8579 ns]
                        thrpt:  [145.82 Melem/s 145.92 Melem/s 146.00 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

condition/simple        time:   [51.111 ns 51.168 ns 51.230 ns]
                        thrpt:  [19.520 Melem/s 19.543 Melem/s 19.565 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  6 (6.00%) high severe
condition/nested_field  time:   [558.70 ns 559.45 ns 560.22 ns]
                        thrpt:  [1.7850 Melem/s 1.7875 Melem/s 1.7899 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
condition/string_eq     time:   [70.381 ns 70.453 ns 70.524 ns]
                        thrpt:  [14.180 Melem/s 14.194 Melem/s 14.208 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

condition_expr/single   time:   [51.018 ns 51.140 ns 51.286 ns]
                        thrpt:  [19.499 Melem/s 19.554 Melem/s 19.601 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
condition_expr/and_2    time:   [103.94 ns 104.07 ns 104.21 ns]
                        thrpt:  [9.5957 Melem/s 9.6091 Melem/s 9.6213 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
condition_expr/and_5    time:   [306.83 ns 307.54 ns 308.57 ns]
                        thrpt:  [3.2408 Melem/s 3.2516 Melem/s 3.2591 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
condition_expr/or_3     time:   [172.52 ns 172.71 ns 172.91 ns]
                        thrpt:  [5.7833 Melem/s 5.7902 Melem/s 5.7964 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
condition_expr/nested   time:   [124.34 ns 124.59 ns 124.95 ns]
                        thrpt:  [8.0029 Melem/s 8.0266 Melem/s 8.0422 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

rule/create             time:   [281.84 ns 282.12 ns 282.40 ns]
                        thrpt:  [3.5410 Melem/s 3.5446 Melem/s 3.5482 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
rule/matches            time:   [103.54 ns 103.86 ns 104.31 ns]
                        thrpt:  [9.5872 Melem/s 9.6287 Melem/s 9.6580 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe

rule_context/create     time:   [838.21 ns 838.86 ns 839.52 ns]
                        thrpt:  [1.1912 Melem/s 1.1921 Melem/s 1.1930 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
rule_context/get_simple time:   [49.552 ns 49.635 ns 49.720 ns]
                        thrpt:  [20.113 Melem/s 20.147 Melem/s 20.181 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
rule_context/get_nested time:   [564.12 ns 564.61 ns 565.14 ns]
                        thrpt:  [1.7695 Melem/s 1.7711 Melem/s 1.7727 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
rule_context/get_deep_nested
                        time:   [570.48 ns 571.02 ns 571.60 ns]
                        thrpt:  [1.7495 Melem/s 1.7512 Melem/s 1.7529 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe

rule_engine_basic/create
                        time:   [8.1619 ns 8.1653 ns 8.1689 ns]
                        thrpt:  [122.42 Melem/s 122.47 Melem/s 122.52 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
rule_engine_basic/add_rule
                        time:   [2.8072 µs 2.9638 µs 3.0945 µs]
                        thrpt:  [323.15 Kelem/s 337.41 Kelem/s 356.23 Kelem/s]
rule_engine_basic/get_rule
                        time:   [17.398 ns 17.705 ns 18.016 ns]
                        thrpt:  [55.508 Melem/s 56.480 Melem/s 57.478 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
rule_engine_basic/rules_by_tag
                        time:   [1.1246 µs 1.1260 µs 1.1273 µs]
                        thrpt:  [887.05 Kelem/s 888.12 Kelem/s 889.19 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
rule_engine_basic/stats time:   [6.9180 µs 6.9330 µs 6.9554 µs]
                        thrpt:  [143.77 Kelem/s 144.24 Kelem/s 144.55 Kelem/s]

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples in estimated 5.0079 s (2.3M iterule_engine_evaluate/evaluate_10_rules
                        time:   [2.1624 µs 2.1643 µs 2.1663 µs]
                        thrpt:  [461.62 Kelem/s 462.05 Kelem/s 462.46 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samples in estimated 5.0003 s (21rule_engine_evaluate/evaluate_first_10_rules
                        time:   [233.39 ns 233.79 ns 234.21 ns]
                        thrpt:  [4.2696 Melem/s 4.2773 Melem/s 4.2847 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples in estimated 5.0613 s (232k itrule_engine_evaluate/evaluate_100_rules
                        time:   [21.731 µs 21.755 µs 21.780 µs]
                        thrpt:  [45.914 Kelem/s 45.967 Kelem/s 46.018 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 samples in estimated 5.0011 s (2rule_engine_evaluate/evaluate_first_100_rules
                        time:   [234.06 ns 234.79 ns 235.81 ns]
                        thrpt:  [4.2406 Melem/s 4.2591 Melem/s 4.2725 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated 5.0573 srule_engine_evaluate/evaluate_matching_100_rules
                        time:   [22.212 µs 22.235 µs 22.258 µs]
                        thrpt:  [44.927 Kelem/s 44.974 Kelem/s 45.021 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samples in estimated 5.4325 s (25k itrule_engine_evaluate/evaluate_1000_rules
                        time:   [214.96 µs 215.17 µs 215.42 µs]
                        thrpt:  [4.6421 Kelem/s 4.6476 Kelem/s 4.6519 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  8 (8.00%) high mild
  7 (7.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 samples in estimated 5.0005 s (rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [233.41 ns 233.76 ns 234.13 ns]
                        thrpt:  [4.2711 Melem/s 4.2778 Melem/s 4.2844 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimated 5.0098 s (2.3M iterationsrule_engine_scaling/evaluate/10
                        time:   [2.1804 µs 2.1822 µs 2.1841 µs]
                        thrpt:  [457.85 Kelem/s 458.25 Kelem/s 458.63 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/10: Collecting 100 samples in estimated 5.0003 s (21M iterarule_engine_scaling/evaluate_first/10
                        time:   [233.95 ns 234.24 ns 234.52 ns]
                        thrpt:  [4.2640 Melem/s 4.2692 Melem/s 4.2745 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/50: Collecting 100 samples in estimated 5.0042 s (460k iterationsrule_engine_scaling/evaluate/50
                        time:   [10.858 µs 10.867 µs 10.877 µs]
                        thrpt:  [91.939 Kelem/s 92.018 Kelem/s 92.098 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/50: Collecting 100 samples in estimated 5.0006 s (22M iterarule_engine_scaling/evaluate_first/50
                        time:   [234.83 ns 235.11 ns 235.40 ns]
                        thrpt:  [4.2481 Melem/s 4.2532 Melem/s 4.2584 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking rule_engine_scaling/evaluate/100: Collecting 100 samples in estimated 5.1050 s (232k iterationrule_engine_scaling/evaluate/100
                        time:   [21.931 µs 21.955 µs 21.978 µs]
                        thrpt:  [45.499 Kelem/s 45.549 Kelem/s 45.598 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/100: Collecting 100 samples in estimated 5.0006 s (22M iterrule_engine_scaling/evaluate_first/100
                        time:   [234.65 ns 234.98 ns 235.30 ns]
                        thrpt:  [4.2500 Melem/s 4.2557 Melem/s 4.2617 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate/500: Collecting 100 samples in estimated 5.4981 s (50k iterationsrule_engine_scaling/evaluate/500
                        time:   [108.66 µs 108.73 µs 108.80 µs]
                        thrpt:  [9.1914 Kelem/s 9.1973 Kelem/s 9.2030 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/500: Collecting 100 samples in estimated 5.0002 s (21M iterrule_engine_scaling/evaluate_first/500
                        time:   [234.45 ns 234.84 ns 235.22 ns]
                        thrpt:  [4.2513 Melem/s 4.2581 Melem/s 4.2653 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate/1000: Collecting 100 samples in estimated 5.4067 s (25k iterationrule_engine_scaling/evaluate/1000
                        time:   [213.71 µs 213.85 µs 213.99 µs]
                        thrpt:  [4.6732 Kelem/s 4.6762 Kelem/s 4.6792 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/1000: Collecting 100 samples in estimated 5.0010 s (21M iterule_engine_scaling/evaluate_first/1000
                        time:   [235.61 ns 236.10 ns 236.60 ns]
                        thrpt:  [4.2265 Melem/s 4.2356 Melem/s 4.2442 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

rule_set/create         time:   [2.7854 µs 2.7898 µs 2.7944 µs]
                        thrpt:  [357.85 Kelem/s 358.44 Kelem/s 359.01 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
rule_set/load_into_engine
                        time:   [5.5194 µs 5.5223 µs 5.5254 µs]
                        thrpt:  [180.98 Kelem/s 181.08 Kelem/s 181.18 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe

trace_id/generate       time:   [540.95 ns 541.57 ns 542.16 ns]
                        thrpt:  [1.8445 Melem/s 1.8465 Melem/s 1.8486 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
trace_id/to_hex         time:   [76.616 ns 76.675 ns 76.737 ns]
                        thrpt:  [13.031 Melem/s 13.042 Melem/s 13.052 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
trace_id/from_hex       time:   [23.236 ns 23.243 ns 23.250 ns]
                        thrpt:  [43.010 Melem/s 43.024 Melem/s 43.037 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe

context_operations/create
                        time:   [822.85 ns 824.92 ns 827.63 ns]
                        thrpt:  [1.2083 Melem/s 1.2122 Melem/s 1.2153 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
context_operations/child
                        time:   [284.95 ns 285.29 ns 285.63 ns]
                        thrpt:  [3.5011 Melem/s 3.5052 Melem/s 3.5094 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
context_operations/for_remote
                        time:   [285.37 ns 285.67 ns 285.98 ns]
                        thrpt:  [3.4968 Melem/s 3.5005 Melem/s 3.5043 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking context_operations/to_traceparent: Collecting 100 samples in estimated 5.0005 s (22M iterationcontext_operations/to_traceparent
                        time:   [225.64 ns 226.21 ns 226.84 ns]
                        thrpt:  [4.4083 Melem/s 4.4206 Melem/s 4.4319 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking context_operations/from_traceparent: Collecting 100 samples in estimated 5.0017 s (13M iteraticontext_operations/from_traceparent
                        time:   [372.71 ns 373.01 ns 373.33 ns]
                        thrpt:  [2.6786 Melem/s 2.6809 Melem/s 2.6831 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

baggage/create          time:   [2.0406 ns 2.0411 ns 2.0417 ns]
                        thrpt:  [489.79 Melem/s 489.92 Melem/s 490.04 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
baggage/get             time:   [19.272 ns 19.765 ns 20.248 ns]
                        thrpt:  [49.387 Melem/s 50.593 Melem/s 51.889 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
baggage/set             time:   [48.997 ns 49.032 ns 49.070 ns]
                        thrpt:  [20.379 Melem/s 20.395 Melem/s 20.409 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  12 (12.00%) high mild
  8 (8.00%) high severe
baggage/merge           time:   [864.67 ns 864.98 ns 865.29 ns]
                        thrpt:  [1.1557 Melem/s 1.1561 Melem/s 1.1565 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

span/create             time:   [323.55 ns 324.58 ns 326.16 ns]
                        thrpt:  [3.0660 Melem/s 3.0809 Melem/s 3.0907 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
span/set_attribute      time:   [47.548 ns 47.696 ns 47.833 ns]
                        thrpt:  [20.906 Melem/s 20.966 Melem/s 21.031 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
span/add_event          time:   [50.859 ns 51.357 ns 51.801 ns]
                        thrpt:  [19.305 Melem/s 19.472 Melem/s 19.662 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild
span/with_kind          time:   [322.71 ns 323.20 ns 323.70 ns]
                        thrpt:  [3.0893 Melem/s 3.0940 Melem/s 3.0987 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

context_store/create_context
                        time:   [96.826 µs 96.858 µs 96.893 µs]
                        thrpt:  [10.321 Kelem/s 10.324 Kelem/s 10.328 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
