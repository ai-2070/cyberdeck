     Running benches\auth_guard.rs (target\release\deps\auth_guard-13c7150e4b38b665.exe)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (auth_guard_check_fast_hit/single_thread
                        time:   [24.528 ns 24.607 ns 24.695 ns]
                        thrpt:  [40.493 Melem/s 40.639 Melem/s 40.770 Melem/s]
Found 6 outliers among 50 measurements (12.00%)
  1 (2.00%) low severe
  1 (2.00%) low mild
  2 (4.00%) high mild
  2 (4.00%) high severe

Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s auth_guard_check_fast_miss/single_thread
                        time:   [10.191 ns 10.248 ns 10.282 ns]
                        thrpt:  [97.261 Melem/s 97.578 Melem/s 98.128 Melem/s]
Found 6 outliers among 50 measurements (12.00%)
  1 (2.00%) low severe
  1 (2.00%) low mild
  2 (4.00%) high mild
  2 (4.00%) high severe

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.00auth_guard_check_fast_contended/eight_threads
                        time:   [25.536 ns 25.824 ns 26.165 ns]
                        thrpt:  [38.219 Melem/s 38.724 Melem/s 39.160 Melem/s]
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_allow_channel/insert: Collecting 50 samples in estimated 5.0001 s (18M iterauth_guard_allow_channel/insert
                        time:   [159.03 ns 163.74 ns 169.04 ns]
                        thrpt:  [5.9156 Melem/s 6.1072 Melem/s 6.2881 Melem/s]
Found 2 outliers among 50 measurements (4.00%)
  2 (4.00%) high mild

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 5.3672 s (5auth_guard_hot_hit_ceiling/million_ops
                        time:   [9.7427 ms 9.7504 ms 9.7593 ms]
Found 7 outliers among 50 measurements (14.00%)
  7 (14.00%) high severe

     Running benches\cortex.rs (target\release\deps\cortex-a782eadbe12f1aad.exe)
Gnuplot not found, using plotters backend
Benchmarking cortex_ingest/tasks_create: Collecting 100 samples in estimated 5.0002 s (28M iteratiocortex_ingest/tasks_create
                        time:   [190.39 ns 191.72 ns 193.32 ns]
                        thrpt:  [5.1726 Melem/s 5.2160 Melem/s 5.2525 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
Benchmarking cortex_ingest/memories_store: Collecting 100 samples in estimated 5.0015 s (14M iteratcortex_ingest/memories_store
                        time:   [338.13 ns 341.70 ns 346.12 ns]
                        thrpt:  [2.8891 Melem/s 2.9266 Melem/s 2.9574 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0003 cortex_fold_barrier/tasks_create_and_wait
                        time:   [1.7826 µs 1.8498 µs 1.9280 µs]
                        thrpt:  [518.66 Kelem/s 540.59 Kelem/s 560.98 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) high mild
  11 (11.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.007cortex_fold_barrier/memories_store_and_wait
                        time:   [2.2101 µs 2.4137 µs 2.7459 µs]
                        thrpt:  [364.18 Kelem/s 414.30 Kelem/s 452.46 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) high mild
  13 (13.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0092 s (2.3M icortex_query/tasks_find_many/100
                        time:   [2.1544 µs 2.1629 µs 2.1733 µs]
                        thrpt:  [46.013 Melem/s 46.234 Melem/s 46.417 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0002 s (29M cortex_query/tasks_count_where/100
                        time:   [173.87 ns 174.48 ns 175.15 ns]
                        thrpt:  [570.93 Melem/s 573.14 Melem/s 575.15 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (735Mcortex_query/tasks_find_unique/100
                        time:   [6.7757 ns 6.7905 ns 6.8077 ns]
                        thrpt:  [14.689 Gelem/s 14.726 Gelem/s 14.759 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0162 s cortex_query/memories_find_many_tag/100
                        time:   [12.569 µs 12.603 µs 12.650 µs]
                        thrpt:  [7.9051 Melem/s 7.9344 Melem/s 7.9559 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0006 s (9cortex_query/memories_count_where/100
                        time:   [509.32 ns 511.01 ns 512.83 ns]
                        thrpt:  [195.00 Melem/s 195.69 Melem/s 196.34 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0898 s (273k cortex_query/tasks_find_many/1000
                        time:   [18.664 µs 18.698 µs 18.736 µs]
                        thrpt:  [53.373 Melem/s 53.483 Melem/s 53.579 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0075 s (3.0cortex_query/tasks_count_where/1000
                        time:   [1.6786 µs 1.6832 µs 1.6899 µs]
                        thrpt:  [591.75 Melem/s 594.09 Melem/s 595.75 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (692cortex_query/tasks_find_unique/1000
                        time:   [7.1877 ns 7.1966 ns 7.2059 ns]
                        thrpt:  [138.78 Gelem/s 138.95 Gelem/s 139.13 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  10 (10.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.4636 scortex_query/memories_find_many_tag/1000
                        time:   [97.948 µs 98.122 µs 98.360 µs]
                        thrpt:  [10.167 Melem/s 10.191 Melem/s 10.209 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0238 s (cortex_query/memories_count_where/1000
                        time:   [4.8653 µs 4.9015 µs 4.9438 µs]
                        thrpt:  [202.28 Melem/s 204.02 Melem/s 205.54 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.4255 s (30k cortex_query/tasks_find_many/10000
                        time:   [177.95 µs 178.31 µs 178.73 µs]
                        thrpt:  [55.951 Melem/s 56.084 Melem/s 56.197 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.0720 s (16cortex_query/tasks_count_where/10000
                        time:   [31.270 µs 31.508 µs 31.765 µs]
                        thrpt:  [314.82 Melem/s 317.37 Melem/s 319.79 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (73cortex_query/tasks_find_unique/10000
                        time:   [6.7852 ns 6.8053 ns 6.8283 ns]
                        thrpt:  [1464.5 Gelem/s 1469.5 Gelem/s 1473.8 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 8.4977 cortex_query/memories_find_many_tag/10000
                        time:   [816.90 µs 820.14 µs 824.13 µs]
                        thrpt:  [12.134 Melem/s 12.193 Melem/s 12.241 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.4363 s cortex_query/memories_count_where/10000
                        time:   [106.79 µs 107.27 µs 107.91 µs]
                        thrpt:  [92.671 Melem/s 93.222 Melem/s 93.641 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0045 s (1.6M icortex_snapshot/tasks_encode/100
                        time:   [3.1319 µs 3.1397 µs 3.1490 µs]
                        thrpt:  [31.756 Melem/s 31.850 Melem/s 31.929 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0149 s (924cortex_snapshot/memories_encode/100
                        time:   [5.3951 µs 5.4072 µs 5.4217 µs]
                        thrpt:  [18.444 Melem/s 18.494 Melem/s 18.535 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimatecortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.0911 µs 2.0936 µs 2.0962 µs]
                        thrpt:  [47.705 Melem/s 47.765 Melem/s 47.821 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0049 s cortex_snapshot/netdb_bundle_decode/100
                        time:   [2.5254 µs 2.5280 µs 2.5309 µs]
                        thrpt:  [39.512 Melem/s 39.557 Melem/s 39.597 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.1194 s (152k cortex_snapshot/tasks_encode/1000
                        time:   [33.712 µs 33.771 µs 33.850 µs]
                        thrpt:  [29.542 Melem/s 29.611 Melem/s 29.663 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.2180 s (96cortex_snapshot/memories_encode/1000
                        time:   [54.271 µs 54.363 µs 54.466 µs]
                        thrpt:  [18.360 Melem/s 18.395 Melem/s 18.426 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimacortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [24.812 µs 24.831 µs 24.854 µs]
                        thrpt:  [40.234 Melem/s 40.271 Melem/s 40.303 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0988 scortex_snapshot/netdb_bundle_decode/1000
                        time:   [32.394 µs 32.486 µs 32.590 µs]
                        thrpt:  [30.684 Melem/s 30.782 Melem/s 30.870 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 5.1449 s (35k cortex_snapshot/tasks_encode/10000
                        time:   [145.14 µs 145.53 µs 145.99 µs]
                        thrpt:  [68.500 Melem/s 68.714 Melem/s 68.897 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 5.4703 s (3cortex_snapshot/memories_encode/10000
                        time:   [153.96 µs 154.21 µs 154.49 µs]
                        thrpt:  [64.728 Melem/s 64.847 Melem/s 64.951 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_171034/10000: Collecting 100 samples in esticortex_snapshot/netdb_bundle_encode_bytes_171034/10000
                        time:   [86.148 µs 86.268 µs 86.403 µs]
                        thrpt:  [115.74 Melem/s 115.92 Melem/s 116.08 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 5.1824 cortex_snapshot/netdb_bundle_decode/10000
                        time:   [113.88 µs 114.01 µs 114.15 µs]
                        thrpt:  [87.608 Melem/s 87.715 Melem/s 87.811 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) low severe
  6 (6.00%) high mild
  6 (6.00%) high severe

     Running benches\ingestion.rs (target\release\deps\ingestion-4d0279eb8a0ef245.exe)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [1.2475 ns 1.2496 ns 1.2519 ns]
                        thrpt:  [798.76 Melem/s 800.27 Melem/s 801.62 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking ring_buffer/push_pop/1024: Collecting 100 samples in estimated 5.0000 s (4.9B iteratioring_buffer/push_pop/1024
                        time:   [1.0094 ns 1.0122 ns 1.0154 ns]
                        thrpt:  [984.81 Melem/s 987.93 Melem/s 990.66 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) high mild
  8 (8.00%) high severe
ring_buffer/push/8192   time:   [1.2240 ns 1.2253 ns 1.2267 ns]
                        thrpt:  [815.23 Melem/s 816.13 Melem/s 816.98 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking ring_buffer/push_pop/8192: Collecting 100 samples in estimated 5.0000 s (5.0B iteratioring_buffer/push_pop/8192
                        time:   [1.0022 ns 1.0059 ns 1.0091 ns]
                        thrpt:  [990.94 Melem/s 994.13 Melem/s 997.78 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  12 (12.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
ring_buffer/push/65536  time:   [1.2136 ns 1.2187 ns 1.2230 ns]
                        thrpt:  [817.67 Melem/s 820.57 Melem/s 823.97 Melem/s]
Found 28 outliers among 100 measurements (28.00%)
  18 (18.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking ring_buffer/push_pop/65536: Collecting 100 samples in estimated 5.0000 s (5.0B iteratiring_buffer/push_pop/65536
                        time:   [1.0090 ns 1.0111 ns 1.0130 ns]
                        thrpt:  [987.12 Melem/s 989.06 Melem/s 991.07 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking ring_buffer/push/1048576: Collecting 100 samples in estimated 5.0000 s (4.1B iterationring_buffer/push/1048576
                        time:   [1.2273 ns 1.2329 ns 1.2384 ns]
                        thrpt:  [807.51 Melem/s 811.09 Melem/s 814.80 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  11 (11.00%) low mild
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking ring_buffer/push_pop/1048576: Collecting 100 samples in estimated 5.0000 s (4.9B iteraring_buffer/push_pop/1048576
                        time:   [1.0315 ns 1.0350 ns 1.0384 ns]
                        thrpt:  [963.02 Melem/s 966.20 Melem/s 969.49 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) high mild
  8 (8.00%) high severe

timestamp/next          time:   [13.688 ns 13.750 ns 13.806 ns]
                        thrpt:  [72.434 Melem/s 72.728 Melem/s 73.057 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high severe
timestamp/now_raw       time:   [6.0040 ns 6.0228 ns 6.0371 ns]
                        thrpt:  [165.64 Melem/s 166.04 Melem/s 166.56 Melem/s]
Found 26 outliers among 100 measurements (26.00%)
  8 (8.00%) low severe
  6 (6.00%) low mild
  2 (2.00%) high mild
  10 (10.00%) high severe

Benchmarking event/internal_event_new: Collecting 100 samples in estimated 5.0007 s (24M iterationsevent/internal_event_new
                        time:   [210.16 ns 211.07 ns 211.94 ns]
                        thrpt:  [4.7184 Melem/s 4.7377 Melem/s 4.7582 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
event/json_creation     time:   [132.31 ns 132.70 ns 133.03 ns]
                        thrpt:  [7.5169 Melem/s 7.5358 Melem/s 7.5578 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  8 (8.00%) low severe
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking batch/pop_batch_steady_state/100: Collecting 100 samples in estimated 5.0002 s (34M itbatch/pop_batch_steady_state/100
                        time:   [141.68 ns 142.28 ns 142.88 ns]
                        thrpt:  [699.90 Melem/s 702.84 Melem/s 705.81 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking batch/pop_batch_steady_state/1000: Collecting 100 samples in estimated 5.0000 s (4.1M batch/pop_batch_steady_state/1000
                        time:   [1.1954 µs 1.2008 µs 1.2056 µs]
                        thrpt:  [829.45 Melem/s 832.77 Melem/s 836.57 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  13 (13.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking batch/pop_batch_steady_state/10000: Collecting 100 samples in estimated 5.0071 s (404kbatch/pop_batch_steady_state/10000
                        time:   [12.735 µs 12.824 µs 12.906 µs]
                        thrpt:  [774.83 Melem/s 779.79 Melem/s 785.24 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

     Running benches\mesh.rs (target\release\deps\mesh-81179f2c6d1d65f8.exe)
Gnuplot not found, using plotters backend
Benchmarking mesh_reroute/triangle_failure: Collecting 100 samples in estimated 5.0284 s (217k itermesh_reroute/triangle_failure
                        time:   [23.762 µs 24.050 µs 24.354 µs]
                        thrpt:  [41.061 Kelem/s 41.581 Kelem/s 42.083 Kelem/s]
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimated 5.4409 s (40k itemesh_reroute/10_peers_10_routes
                        time:   [124.03 µs 125.21 µs 126.58 µs]
                        thrpt:  [7.9004 Kelem/s 7.9867 Kelem/s 8.0627 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking mesh_reroute/50_peers_100_routes: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimated 6.0809 s (5050 imesh_reroute/50_peers_100_routes
                        time:   [1.1925 ms 1.1957 ms 1.1990 ms]
                        thrpt:  [834.06  elem/s 836.34  elem/s 838.58  elem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking mesh_proximity/on_pingwave_new: Collecting 100 samples in estimated 5.0053 s (3.0M itemesh_proximity/on_pingwave_new
                        time:   [107.70 ns 109.40 ns 111.58 ns]
                        thrpt:  [8.9620 Melem/s 9.1406 Melem/s 9.2847 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimated 5.0001 s (101M imesh_proximity/on_pingwave_dedup
                        time:   [49.635 ns 49.762 ns 49.866 ns]
                        thrpt:  [20.054 Melem/s 20.096 Melem/s 20.147 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estimated 5.0000 s (5.0B mesh_proximity/pingwave_serialize
                        time:   [1.1911 ns 1.2366 ns 1.2788 ns]
                        thrpt:  [781.98 Melem/s 808.66 Melem/s 839.59 Melem/s]
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in estimated 5.0000 s (3.6mesh_proximity/pingwave_deserialize
                        time:   [1.2234 ns 1.2703 ns 1.3129 ns]
                        thrpt:  [761.65 Melem/s 787.19 Melem/s 817.43 Melem/s]
Benchmarking mesh_proximity/node_count: Collecting 100 samples in estimated 5.0047 s (5.2M iteratiomesh_proximity/node_count
                        time:   [959.09 ns 959.79 ns 960.58 ns]
                        thrpt:  [1.0410 Melem/s 1.0419 Melem/s 1.0427 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 31.2s, or reduce sample count to 10.
Benchmarking mesh_proximity/all_nodes_100: Collecting 100 samples in estimated 31.213 s (100 iteratmesh_proximity/all_nodes_100
                        time:   [313.19 ms 315.10 ms 317.05 ms]
                        thrpt:  [3.1540  elem/s 3.1736  elem/s 3.1929  elem/s]

Benchmarking mesh_dispatch/classify_direct: Collecting 100 samples in estimated 5.0000 s (13B iteramesh_dispatch/classify_direct
                        time:   [399.46 ps 400.80 ps 401.90 ps]
                        thrpt:  [2.4882 Gelem/s 2.4950 Gelem/s 2.5034 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  8 (8.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking mesh_dispatch/classify_routed: Collecting 100 samples in estimated 5.0000 s (16B iteramesh_dispatch/classify_routed
                        time:   [299.87 ps 301.98 ps 304.00 ps]
                        thrpt:  [3.2894 Gelem/s 3.3115 Gelem/s 3.3348 Gelem/s]
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) low mild
  4 (4.00%) high mild
Benchmarking mesh_dispatch/classify_pingwave: Collecting 100 samples in estimated 5.0000 s (25B itemesh_dispatch/classify_pingwave
                        time:   [200.65 ps 201.03 ps 201.30 ps]
                        thrpt:  [4.9676 Gelem/s 4.9744 Gelem/s 4.9839 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  6 (6.00%) high mild
  4 (4.00%) high severe

Benchmarking mesh_routing/lookup_hit: Collecting 100 samples in estimated 5.0001 s (283M iterationsmesh_routing/lookup_hit time:   [17.943 ns 18.036 ns 18.125 ns]
                        thrpt:  [55.171 Melem/s 55.446 Melem/s 55.732 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking mesh_routing/lookup_miss: Collecting 100 samples in estimated 5.0000 s (282M iterationmesh_routing/lookup_miss
                        time:   [17.732 ns 17.755 ns 17.779 ns]
                        thrpt:  [56.246 Melem/s 56.324 Melem/s 56.395 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
mesh_routing/is_local   time:   [211.71 ps 215.90 ps 220.52 ps]
                        thrpt:  [4.5346 Gelem/s 4.6318 Gelem/s 4.7236 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_routing/all_routes/10: Collecting 100 samples in estimated 5.0006 s (889k iteratimesh_routing/all_routes/10
                        time:   [5.5641 µs 5.5798 µs 5.5946 µs]
                        thrpt:  [178.74 Kelem/s 179.22 Kelem/s 179.72 Kelem/s]
Benchmarking mesh_routing/all_routes/100: Collecting 100 samples in estimated 5.0318 s (672k iteratmesh_routing/all_routes/100
                        time:   [7.4200 µs 7.4381 µs 7.4555 µs]
                        thrpt:  [134.13 Kelem/s 134.44 Kelem/s 134.77 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking mesh_routing/all_routes/1000: Collecting 100 samples in estimated 5.0095 s (212k iteramesh_routing/all_routes/1000
                        time:   [23.429 µs 23.477 µs 23.522 µs]
                        thrpt:  [42.513 Kelem/s 42.595 Kelem/s 42.683 Kelem/s]
mesh_routing/add_route  time:   [37.480 ns 37.533 ns 37.585 ns]
                        thrpt:  [26.606 Melem/s 26.643 Melem/s 26.681 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

     Running benches\net.rs (target\release\deps\net-ed41933ae9d02430.exe)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [1.2751 ns 1.3121 ns 1.3443 ns]
                        thrpt:  [743.89 Melem/s 762.11 Melem/s 784.24 Melem/s]
net_header/deserialize  time:   [1.2057 ns 1.2066 ns 1.2077 ns]
                        thrpt:  [828.03 Melem/s 828.74 Melem/s 829.37 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
net_header/roundtrip    time:   [1.2063 ns 1.2073 ns 1.2085 ns]
                        thrpt:  [827.46 Melem/s 828.27 Melem/s 828.99 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimated 5.0000 s (142M itnet_event_frame/write_single/64
                        time:   [35.440 ns 35.636 ns 35.847 ns]
                        thrpt:  [1.6627 GiB/s 1.6726 GiB/s 1.6819 GiB/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high mild
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estimated 5.0001 s (142M inet_event_frame/write_single/256
                        time:   [35.738 ns 35.975 ns 36.229 ns]
                        thrpt:  [6.5808 GiB/s 6.6273 GiB/s 6.6712 GiB/s]
Found 17 outliers among 100 measurements (17.00%)
  16 (16.00%) high mild
  1 (1.00%) high severe
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0001 s (141M net_event_frame/write_single/1024
                        time:   [36.433 ns 36.711 ns 36.953 ns]
                        thrpt:  [25.808 GiB/s 25.978 GiB/s 26.176 GiB/s]
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0002 s (100M net_event_frame/write_single/4096
                        time:   [50.777 ns 51.033 ns 51.290 ns]
                        thrpt:  [74.375 GiB/s 74.749 GiB/s 75.126 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
Benchmarking net_event_frame/write_batch/1: Collecting 100 samples in estimated 5.0000 s (189M iternet_event_frame/write_batch/1
                        time:   [26.571 ns 26.697 ns 26.820 ns]
                        thrpt:  [2.2224 GiB/s 2.2326 GiB/s 2.2433 GiB/s]
Benchmarking net_event_frame/write_batch/10: Collecting 100 samples in estimated 5.0000 s (86M iternet_event_frame/write_batch/10
                        time:   [58.430 ns 58.763 ns 59.077 ns]
                        thrpt:  [10.089 GiB/s 10.143 GiB/s 10.201 GiB/s]
Benchmarking net_event_frame/write_batch/50: Collecting 100 samples in estimated 5.0006 s (34M iternet_event_frame/write_batch/50
                        time:   [145.33 ns 146.16 ns 146.96 ns]
                        thrpt:  [20.279 GiB/s 20.390 GiB/s 20.506 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking net_event_frame/write_batch/100: Collecting 100 samples in estimated 5.0002 s (19M itenet_event_frame/write_batch/100
                        time:   [258.62 ns 259.22 ns 259.73 ns]
                        thrpt:  [22.949 GiB/s 22.994 GiB/s 23.047 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking net_event_frame/read_batch_10: Collecting 100 samples in estimated 5.0000 s (31M iteranet_event_frame/read_batch_10
                        time:   [163.99 ns 164.31 ns 164.66 ns]
                        thrpt:  [60.731 Melem/s 60.861 Melem/s 60.981 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe

Benchmarking net_packet_pool/get_return/16: Collecting 100 samples in estimated 5.0000 s (98M iteranet_packet_pool/get_return/16
                        time:   [50.304 ns 50.593 ns 50.858 ns]
                        thrpt:  [19.662 Melem/s 19.766 Melem/s 19.879 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking net_packet_pool/get_return/64: Collecting 100 samples in estimated 5.0000 s (98M iteranet_packet_pool/get_return/64
                        time:   [51.432 ns 51.531 ns 51.629 ns]
                        thrpt:  [19.369 Melem/s 19.406 Melem/s 19.443 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking net_packet_pool/get_return/256: Collecting 100 samples in estimated 5.0003 s (94M iternet_packet_pool/get_return/256
                        time:   [52.546 ns 52.701 ns 52.839 ns]
                        thrpt:  [18.925 Melem/s 18.975 Melem/s 19.031 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high severe

Benchmarking net_packet_build/build_packet/1: Collecting 100 samples in estimated 5.0003 s (4.4M itnet_packet_build/build_packet/1
                        time:   [1.1351 µs 1.1361 µs 1.1371 µs]
                        thrpt:  [53.675 MiB/s 53.724 MiB/s 53.769 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0002 s (3.3M inet_packet_build/build_packet/10
                        time:   [1.5002 µs 1.5017 µs 1.5034 µs]
                        thrpt:  [405.98 MiB/s 406.44 MiB/s 406.85 MiB/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0006 s (1.7M inet_packet_build/build_packet/50
                        time:   [2.9237 µs 2.9299 µs 2.9348 µs]
                        thrpt:  [1.0155 GiB/s 1.0172 GiB/s 1.0193 GiB/s]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe

Benchmarking net_encryption/encrypt/64: Collecting 100 samples in estimated 5.0033 s (4.4M iterationet_encryption/encrypt/64
                        time:   [1.1346 µs 1.1356 µs 1.1366 µs]
                        thrpt:  [53.699 MiB/s 53.749 MiB/s 53.795 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking net_encryption/encrypt/256: Collecting 100 samples in estimated 5.0048 s (4.2M iteratinet_encryption/encrypt/256
                        time:   [1.2015 µs 1.2027 µs 1.2039 µs]
                        thrpt:  [202.79 MiB/s 203.00 MiB/s 203.20 MiB/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking net_encryption/encrypt/1024: Collecting 100 samples in estimated 5.0076 s (3.2M iteratnet_encryption/encrypt/1024
                        time:   [1.5775 µs 1.5793 µs 1.5812 µs]
                        thrpt:  [617.61 MiB/s 618.35 MiB/s 619.06 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high severe
Benchmarking net_encryption/encrypt/4096: Collecting 100 samples in estimated 5.0055 s (1.6M iteratnet_encryption/encrypt/4096
                        time:   [3.0833 µs 3.0992 µs 3.1137 µs]
                        thrpt:  [1.2251 GiB/s 1.2309 GiB/s 1.2372 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

net_keypair/generate    time:   [10.593 µs 10.661 µs 10.727 µs]
                        thrpt:  [93.222 Kelem/s 93.798 Kelem/s 94.399 Kelem/s]

net_aad/generate        time:   [1.0525 ns 1.0993 ns 1.1418 ns]
                        thrpt:  [875.82 Melem/s 909.65 Melem/s 950.15 Melem/s]

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0002 s (pool_comparison/shared_pool_get_return
                        time:   [52.829 ns 52.956 ns 53.096 ns]
                        thrpt:  [18.834 Melem/s 18.884 Melem/s 18.929 Melem/s]
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.00pool_comparison/thread_local_pool_get_return
                        time:   [64.970 ns 65.543 ns 66.178 ns]
                        thrpt:  [15.111 Melem/s 15.257 Melem/s 15.392 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking pool_comparison/shared_pool_10x: Collecting 100 samples in estimated 5.0011 s (11M itepool_comparison/shared_pool_10x
                        time:   [519.41 ns 526.09 ns 531.73 ns]
                        thrpt:  [1.8806 Melem/s 1.9008 Melem/s 1.9253 Melem/s]
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0047 s (5pool_comparison/thread_local_pool_10x
                        time:   [812.66 ns 819.73 ns 827.14 ns]
                        thrpt:  [1.2090 Melem/s 1.2199 Melem/s 1.2305 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0039 s (4.5M icipher_comparison/shared_pool/64
                        time:   [1.1160 µs 1.1217 µs 1.1271 µs]
                        thrpt:  [54.155 MiB/s 54.411 MiB/s 54.690 MiB/s]
Found 22 outliers among 100 measurements (22.00%)
  18 (18.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0050 s (4.4Mcipher_comparison/fast_chacha20/64
                        time:   [1.1356 µs 1.1373 µs 1.1392 µs]
                        thrpt:  [53.577 MiB/s 53.665 MiB/s 53.749 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0015 s (4.2M cipher_comparison/shared_pool/256
                        time:   [1.1988 µs 1.2014 µs 1.2039 µs]
                        thrpt:  [202.80 MiB/s 203.21 MiB/s 203.65 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0002 s (4.2cipher_comparison/fast_chacha20/256
                        time:   [1.1999 µs 1.2027 µs 1.2048 µs]
                        thrpt:  [202.64 MiB/s 203.00 MiB/s 203.47 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0047 s (3.2Mcipher_comparison/shared_pool/1024
                        time:   [1.5729 µs 1.5765 µs 1.5796 µs]
                        thrpt:  [618.24 MiB/s 619.46 MiB/s 620.87 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0069 s (3.cipher_comparison/fast_chacha20/1024
                        time:   [1.5426 µs 1.5513 µs 1.5593 µs]
                        thrpt:  [626.28 MiB/s 629.52 MiB/s 633.08 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  12 (12.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0071 s (1.6Mcipher_comparison/shared_pool/4096
                        time:   [3.0552 µs 3.0751 µs 3.0940 µs]
                        thrpt:  [1.2329 GiB/s 1.2405 GiB/s 1.2486 GiB/s]
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0107 s (1.cipher_comparison/fast_chacha20/4096
                        time:   [3.0012 µs 3.0207 µs 3.0403 µs]
                        thrpt:  [1.2547 GiB/s 1.2629 GiB/s 1.2711 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking adaptive_batcher/optimal_size: Collecting 100 samples in estimated 5.0000 s (6.3B iteradaptive_batcher/optimal_size
                        time:   [783.39 ps 788.45 ps 793.60 ps]
                        thrpt:  [1.2601 Gelem/s 1.2683 Gelem/s 1.2765 Gelem/s]
Found 18 outliers among 100 measurements (18.00%)
  17 (17.00%) low mild
  1 (1.00%) high severe
Benchmarking adaptive_batcher/record: Collecting 100 samples in estimated 5.0000 s (533M iterationsadaptive_batcher/record time:   [9.2420 ns 9.2967 ns 9.3491 ns]
                        thrpt:  [106.96 Melem/s 107.56 Melem/s 108.20 Melem/s]
Benchmarking adaptive_batcher/full_cycle: Collecting 100 samples in estimated 5.0000 s (629M iteratadaptive_batcher/full_cycle
                        time:   [7.8783 ns 7.9251 ns 7.9680 ns]
                        thrpt:  [125.50 Melem/s 126.18 Melem/s 126.93 Melem/s]

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0015 s (e2e_packet_build/shared_pool_50_events
                        time:   [2.8525 µs 2.8706 µs 2.8883 µs]
                        thrpt:  [1.0318 GiB/s 1.0382 GiB/s 1.0448 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0043 s (1.8M ite2e_packet_build/fast_50_events
                        time:   [2.8204 µs 2.8355 µs 2.8496 µs]
                        thrpt:  [1.0458 GiB/s 1.0510 GiB/s 1.0567 GiB/s]
Found 26 outliers among 100 measurements (26.00%)
  19 (19.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.3s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 8.3384 s (multithread_packet_build/shared_pool/8
                        time:   [1.6483 ms 1.6552 ms 1.6614 ms]
                        thrpt:  [4.8152 Melem/s 4.8333 Melem/s 4.8535 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.4s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 7.44multithread_packet_build/thread_local_pool/8
                        time:   [1.4593 ms 1.4680 ms 1.4766 ms]
                        thrpt:  [5.4180 Melem/s 5.4497 Melem/s 5.4822 Melem/s]
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.2609 s multithread_packet_build/shared_pool/16
                        time:   [2.6146 ms 2.6261 ms 2.6368 ms]
                        thrpt:  [6.0679 Melem/s 6.0926 Melem/s 6.1194 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  8 (8.00%) low severe
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.6s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 9.6multithread_packet_build/thread_local_pool/16
                        time:   [1.8847 ms 1.8989 ms 1.9119 ms]
                        thrpt:  [8.3684 Melem/s 8.4260 Melem/s 8.4894 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  11 (11.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.3110 s multithread_packet_build/shared_pool/24
                        time:   [3.7398 ms 3.7907 ms 3.8616 ms]
                        thrpt:  [6.2150 Melem/s 6.3314 Melem/s 6.4174 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.2multithread_packet_build/thread_local_pool/24
                        time:   [2.6565 ms 2.7032 ms 2.7464 ms]
                        thrpt:  [8.7388 Melem/s 8.8783 Melem/s 9.0346 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  17 (17.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.5003 s multithread_packet_build/shared_pool/32
                        time:   [5.2820 ms 5.5519 ms 5.8548 ms]
                        thrpt:  [5.4656 Melem/s 5.7638 Melem/s 6.0583 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) high mild
  19 (19.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.1multithread_packet_build/thread_local_pool/32
                        time:   [3.2193 ms 3.2369 ms 3.2547 ms]
                        thrpt:  [9.8320 Melem/s 9.8861 Melem/s 9.9401 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.8s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 5.8336 s multithread_mixed_frames/shared_mixed/8
                        time:   [1.1281 ms 1.1449 ms 1.1611 ms]
                        thrpt:  [10.335 Melem/s 10.481 Melem/s 10.637 Melem/s]
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 5.4132 s (5multithread_mixed_frames/fast_mixed/8
                        time:   [1.0396 ms 1.0572 ms 1.0732 ms]
                        thrpt:  [11.182 Melem/s 11.351 Melem/s 11.543 Melem/s]
Benchmarking multithread_mixed_frames/shared_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samples in estimated 8.0923 smultithread_mixed_frames/shared_mixed/16
                        time:   [1.5970 ms 1.6026 ms 1.6080 ms]
                        thrpt:  [14.926 Melem/s 14.976 Melem/s 15.028 Melem/s]
Benchmarking multithread_mixed_frames/fast_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples in estimated 7.0997 s (multithread_mixed_frames/fast_mixed/16
                        time:   [1.3930 ms 1.3995 ms 1.4060 ms]
                        thrpt:  [17.070 Melem/s 17.149 Melem/s 17.229 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samples in estimated 5.1023 smultithread_mixed_frames/shared_mixed/24
                        time:   [2.1492 ms 2.1766 ms 2.2136 ms]
                        thrpt:  [16.263 Melem/s 16.539 Melem/s 16.750 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.9s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples in estimated 8.9100 s (multithread_mixed_frames/fast_mixed/24
                        time:   [1.7656 ms 1.7878 ms 1.8086 ms]
                        thrpt:  [19.905 Melem/s 20.136 Melem/s 20.390 Melem/s]
Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samples in estimated 5.2322 smultithread_mixed_frames/shared_mixed/32
                        time:   [3.0781 ms 3.2686 ms 3.4814 ms]
                        thrpt:  [13.787 Melem/s 14.685 Melem/s 15.594 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  5 (5.00%) high mild
  19 (19.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples in estimated 5.1774 s (multithread_mixed_frames/fast_mixed/32
                        time:   [2.2347 ms 2.2510 ms 2.2669 ms]
                        thrpt:  [21.174 Melem/s 21.324 Melem/s 21.479 Melem/s]

Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samples in estimated 5.7623 spool_contention/shared_acquire_release/8
                        time:   [9.5759 ms 9.5961 ms 9.6163 ms]
                        thrpt:  [8.3192 Melem/s 8.3367 Melem/s 8.3543 Melem/s]
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.6s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples in estimated 6.6197 s (pool_contention/fast_acquire_release/8
                        time:   [1.2197 ms 1.2653 ms 1.3085 ms]
                        thrpt:  [61.139 Melem/s 63.228 Melem/s 65.587 Melem/s]
Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samples in estimated 6.5685 pool_contention/shared_acquire_release/16
                        time:   [21.843 ms 21.895 ms 21.947 ms]
                        thrpt:  [7.2902 Melem/s 7.3077 Melem/s 7.3249 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking pool_contention/fast_acquire_release/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.2s, enable flat sampling, or reduce sample count to 50.
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples in estimated 9.2386 s pool_contention/fast_acquire_release/16
                        time:   [1.8060 ms 1.8178 ms 1.8296 ms]
                        thrpt:  [87.453 Melem/s 88.018 Melem/s 88.594 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samples in estimated 7.1359 pool_contention/shared_acquire_release/24
                        time:   [35.506 ms 35.778 ms 36.194 ms]
                        thrpt:  [6.6310 Melem/s 6.7079 Melem/s 6.7594 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples in estimated 5.0904 s pool_contention/fast_acquire_release/24
                        time:   [2.1873 ms 2.2103 ms 2.2330 ms]
                        thrpt:  [107.48 Melem/s 108.58 Melem/s 109.72 Melem/s]
Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samples in estimated 9.1904 pool_contention/shared_acquire_release/32
                        time:   [45.995 ms 46.426 ms 46.986 ms]
                        thrpt:  [6.8106 Melem/s 6.8927 Melem/s 6.9573 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples in estimated 5.2022 s pool_contention/fast_acquire_release/32
                        time:   [2.8633 ms 2.8982 ms 2.9317 ms]
                        thrpt:  [109.15 Melem/s 110.41 Melem/s 111.76 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples in estimated 5.3707 s (1throughput_scaling/fast_pool_scaling/1
                        time:   [3.6124 ms 3.6389 ms 3.6638 ms]
                        thrpt:  [545.87 Kelem/s 549.62 Kelem/s 553.64 Kelem/s]
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples in estimated 5.4503 s (1throughput_scaling/fast_pool_scaling/2
                        time:   [3.6999 ms 3.7152 ms 3.7299 ms]
                        thrpt:  [1.0724 Melem/s 1.0767 Melem/s 1.0811 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples in estimated 5.4924 s (1throughput_scaling/fast_pool_scaling/4
                        time:   [3.7199 ms 3.7323 ms 3.7439 ms]
                        thrpt:  [2.1368 Melem/s 2.1434 Melem/s 2.1506 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples in estimated 5.8665 s (1throughput_scaling/fast_pool_scaling/8
                        time:   [4.4399 ms 4.6415 ms 4.8064 ms]
                        thrpt:  [3.3289 Melem/s 3.4472 Melem/s 3.6037 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples in estimated 5.7057 s (throughput_scaling/fast_pool_scaling/16
                        time:   [5.4146 ms 5.4369 ms 5.4602 ms]
                        thrpt:  [5.8606 Melem/s 5.8857 Melem/s 5.9099 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples in estimated 5.0732 s (throughput_scaling/fast_pool_scaling/24
                        time:   [7.2465 ms 7.9115 ms 8.4516 ms]
                        thrpt:  [5.6794 Melem/s 6.0671 Melem/s 6.6239 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) low severe
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples in estimated 6.3437 s (throughput_scaling/fast_pool_scaling/32
                        time:   [10.060 ms 10.167 ms 10.264 ms]
                        thrpt:  [6.2353 Melem/s 6.2949 Melem/s 6.3617 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild

Benchmarking routing_header/serialize: Collecting 100 samples in estimated 5.0000 s (13B iterationsrouting_header/serialize
                        time:   [431.70 ps 458.66 ps 491.31 ps]
                        thrpt:  [2.0354 Gelem/s 2.1803 Gelem/s 2.3164 Gelem/s]
Found 34 outliers among 100 measurements (34.00%)
  12 (12.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
  17 (17.00%) high severe
Benchmarking routing_header/deserialize: Collecting 100 samples in estimated 5.0000 s (7.2B iteratirouting_header/deserialize
                        time:   [711.56 ps 720.57 ps 731.28 ps]
                        thrpt:  [1.3675 Gelem/s 1.3878 Gelem/s 1.4054 Gelem/s]
Found 25 outliers among 100 measurements (25.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  19 (19.00%) high severe
Benchmarking routing_header/roundtrip: Collecting 100 samples in estimated 5.0000 s (7.1B iterationrouting_header/roundtrip
                        time:   [702.49 ps 711.16 ps 721.12 ps]
                        thrpt:  [1.3867 Gelem/s 1.4061 Gelem/s 1.4235 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe
routing_header/forward  time:   [196.60 ps 197.75 ps 198.83 ps]
                        thrpt:  [5.0295 Gelem/s 5.0569 Gelem/s 5.0863 Gelem/s]

Benchmarking routing_table/lookup_hit: Collecting 100 samples in estimated 5.0001 s (134M iterationrouting_table/lookup_hit
                        time:   [37.313 ns 37.520 ns 37.716 ns]
                        thrpt:  [26.514 Melem/s 26.653 Melem/s 26.800 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) low severe
  11 (11.00%) low mild
Benchmarking routing_table/lookup_miss: Collecting 100 samples in estimated 5.0001 s (283M iteratiorouting_table/lookup_miss
                        time:   [17.317 ns 17.417 ns 17.515 ns]
                        thrpt:  [57.094 Melem/s 57.416 Melem/s 57.748 Melem/s]
routing_table/is_local  time:   [199.89 ps 200.43 ps 200.88 ps]
                        thrpt:  [4.9780 Gelem/s 4.9892 Gelem/s 5.0027 Gelem/s]
Found 20 outliers among 100 measurements (20.00%)
  6 (6.00%) low severe
  8 (8.00%) high mild
  6 (6.00%) high severe
routing_table/add_route time:   [180.38 ns 185.53 ns 190.43 ns]
                        thrpt:  [5.2512 Melem/s 5.3899 Melem/s 5.5439 Melem/s]
Benchmarking routing_table/record_in: Collecting 100 samples in estimated 5.0001 s (124M iterationsrouting_table/record_in time:   [39.893 ns 40.145 ns 40.390 ns]
                        thrpt:  [24.759 Melem/s 24.910 Melem/s 25.067 Melem/s]
Benchmarking routing_table/record_out: Collecting 100 samples in estimated 5.0001 s (237M iterationrouting_table/record_out
                        time:   [21.075 ns 21.100 ns 21.132 ns]
                        thrpt:  [47.322 Melem/s 47.392 Melem/s 47.451 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking routing_table/aggregate_stats: Collecting 100 samples in estimated 5.0157 s (626k iterrouting_table/aggregate_stats
                        time:   [8.0055 µs 8.0137 µs 8.0213 µs]
                        thrpt:  [124.67 Kelem/s 124.79 Kelem/s 124.91 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild

Benchmarking fair_scheduler/creation: Collecting 100 samples in estimated 5.0001 s (3.1M iterationsfair_scheduler/creation time:   [1.5923 µs 1.6006 µs 1.6080 µs]
                        thrpt:  [621.88 Kelem/s 624.78 Kelem/s 628.01 Kelem/s]
Found 19 outliers among 100 measurements (19.00%)
  12 (12.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estimated 5.0020 s (5.3M fair_scheduler/stream_count_empty
                        time:   [942.78 ns 947.66 ns 952.00 ns]
                        thrpt:  [1.0504 Melem/s 1.0552 Melem/s 1.0607 Melem/s]
Benchmarking fair_scheduler/total_queued: Collecting 100 samples in estimated 5.0000 s (25B iteratifair_scheduler/total_queued
                        time:   [200.44 ps 200.77 ps 201.04 ps]
                        thrpt:  [4.9741 Gelem/s 4.9809 Gelem/s 4.9889 Gelem/s]
Found 18 outliers among 100 measurements (18.00%)
  7 (7.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking fair_scheduler/cleanup_empty: Collecting 100 samples in estimated 5.0045 s (3.9M iterafair_scheduler/cleanup_empty
                        time:   [1.2845 µs 1.2865 µs 1.2883 µs]
                        thrpt:  [776.20 Kelem/s 777.29 Kelem/s 778.52 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) low severe
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samples in estimated 5.37routing_table_concurrent/concurrent_lookup/4
                        time:   [178.01 µs 179.22 µs 180.39 µs]
                        thrpt:  [22.175 Melem/s 22.319 Melem/s 22.471 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samples in estimated 5.726routing_table_concurrent/concurrent_stats/4
                        time:   [225.97 µs 227.20 µs 228.50 µs]
                        thrpt:  [17.505 Melem/s 17.605 Melem/s 17.701 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samples in estimated 6.03routing_table_concurrent/concurrent_lookup/8
                        time:   [282.69 µs 286.48 µs 290.06 µs]
                        thrpt:  [27.580 Melem/s 27.926 Melem/s 28.300 Melem/s]
Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samples in estimated 5.048routing_table_concurrent/concurrent_stats/8
                        time:   [325.20 µs 330.19 µs 335.40 µs]
                        thrpt:  [23.852 Melem/s 24.229 Melem/s 24.601 Melem/s]
Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 samples in estimated 5.4routing_table_concurrent/concurrent_lookup/16
                        time:   [512.11 µs 520.51 µs 528.79 µs]
                        thrpt:  [30.258 Melem/s 30.739 Melem/s 31.244 Melem/s]
Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samples in estimated 5.89routing_table_concurrent/concurrent_stats/16
                        time:   [567.63 µs 578.11 µs 588.68 µs]
                        thrpt:  [27.180 Melem/s 27.676 Melem/s 28.187 Melem/s]

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples in estimated 5.0001 s (1routing_decision/parse_lookup_forward
                        time:   [38.558 ns 38.618 ns 38.674 ns]
                        thrpt:  [25.857 Melem/s 25.894 Melem/s 25.935 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  7 (7.00%) low severe
  3 (3.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estimated 5.0001 s (50M itrouting_decision/full_with_stats
                        time:   [100.77 ns 100.88 ns 100.99 ns]
                        thrpt:  [9.9019 Melem/s 9.9128 Melem/s 9.9239 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  7 (7.00%) high mild
  9 (9.00%) high severe

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in estimated 5.0014 s (14M istream_multiplexing/lookup_all/10
                        time:   [347.39 ns 348.28 ns 348.94 ns]
                        thrpt:  [28.658 Melem/s 28.713 Melem/s 28.786 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estimated 5.0009 s (12M itstream_multiplexing/stats_all/10
                        time:   [410.00 ns 410.47 ns 410.88 ns]
                        thrpt:  [24.338 Melem/s 24.362 Melem/s 24.390 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in estimated 5.0153 s (1.5Mstream_multiplexing/lookup_all/100
                        time:   [3.4302 µs 3.4340 µs 3.4379 µs]
                        thrpt:  [29.088 Melem/s 29.120 Melem/s 29.153 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in estimated 5.0151 s (1.2M stream_multiplexing/stats_all/100
                        time:   [4.1005 µs 4.1103 µs 4.1188 µs]
                        thrpt:  [24.279 Melem/s 24.329 Melem/s 24.387 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estimated 5.0525 s (141stream_multiplexing/lookup_all/1000
                        time:   [35.268 µs 35.485 µs 35.698 µs]
                        thrpt:  [28.012 Melem/s 28.181 Melem/s 28.354 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in estimated 5.1853 s (116kstream_multiplexing/stats_all/1000
                        time:   [43.702 µs 43.948 µs 44.182 µs]
                        thrpt:  [22.633 Melem/s 22.754 Melem/s 22.882 Melem/s]
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in estimated 5.8524 s (15stream_multiplexing/lookup_all/10000
                        time:   [384.16 µs 386.36 µs 388.43 µs]
                        thrpt:  [25.745 Melem/s 25.882 Melem/s 26.031 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estimated 7.0629 s (15kstream_multiplexing/stats_all/10000
                        time:   [460.51 µs 462.94 µs 465.24 µs]
                        thrpt:  [21.494 Melem/s 21.601 Melem/s 21.715 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimated 5.0001 s (123M imultihop_packet_builder/build/64
                        time:   [40.485 ns 40.723 ns 40.949 ns]
                        thrpt:  [1.4556 GiB/s 1.4637 GiB/s 1.4723 GiB/s]
Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samples in estimated 5.0000 multihop_packet_builder/build_priority/64
                        time:   [29.427 ns 29.625 ns 29.813 ns]
                        thrpt:  [1.9993 GiB/s 2.0120 GiB/s 2.0255 GiB/s]
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estimated 5.0000 s (115M multihop_packet_builder/build/256
                        time:   [43.373 ns 43.461 ns 43.555 ns]
                        thrpt:  [5.4739 GiB/s 5.4858 GiB/s 5.4969 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) low mild
Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samples in estimated 5.0000multihop_packet_builder/build_priority/256
                        time:   [31.874 ns 31.959 ns 32.040 ns]
                        thrpt:  [7.4413 GiB/s 7.4601 GiB/s 7.4801 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in estimated 5.0000 s (114Mmultihop_packet_builder/build/1024
                        time:   [43.864 ns 43.981 ns 44.092 ns]
                        thrpt:  [21.629 GiB/s 21.684 GiB/s 21.741 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 samples in estimated 5.000multihop_packet_builder/build_priority/1024
                        time:   [35.151 ns 35.206 ns 35.265 ns]
                        thrpt:  [27.043 GiB/s 27.088 GiB/s 27.131 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in estimated 5.0002 s (80M multihop_packet_builder/build/4096
                        time:   [62.367 ns 62.592 ns 62.808 ns]
                        thrpt:  [60.736 GiB/s 60.946 GiB/s 61.165 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 samples in estimated 5.000multihop_packet_builder/build_priority/4096
                        time:   [52.677 ns 52.824 ns 52.970 ns]
                        thrpt:  [72.016 GiB/s 72.215 GiB/s 72.417 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking multihop_chain/forward_chain/1: Collecting 100 samples in estimated 5.0002 s (94M itermultihop_chain/forward_chain/1
                        time:   [53.314 ns 53.369 ns 53.430 ns]
                        thrpt:  [18.716 Melem/s 18.737 Melem/s 18.757 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_chain/forward_chain/2: Collecting 100 samples in estimated 5.0001 s (58M itermultihop_chain/forward_chain/2
                        time:   [86.711 ns 86.868 ns 87.024 ns]
                        thrpt:  [11.491 Melem/s 11.512 Melem/s 11.532 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_chain/forward_chain/3: Collecting 100 samples in estimated 5.0001 s (42M itermultihop_chain/forward_chain/3
                        time:   [120.50 ns 120.66 ns 120.83 ns]
                        thrpt:  [8.2759 Melem/s 8.2878 Melem/s 8.2984 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_chain/forward_chain/4: Collecting 100 samples in estimated 5.0003 s (32M itermultihop_chain/forward_chain/4
                        time:   [154.57 ns 154.83 ns 155.12 ns]
                        thrpt:  [6.4464 Melem/s 6.4585 Melem/s 6.4694 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_chain/forward_chain/5: Collecting 100 samples in estimated 5.0006 s (26M itermultihop_chain/forward_chain/5
                        time:   [189.42 ns 189.73 ns 190.08 ns]
                        thrpt:  [5.2609 Melem/s 5.2706 Melem/s 5.2792 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking hop_latency/single_hop_process: Collecting 100 samples in estimated 5.0000 s (5.0B itehop_latency/single_hop_process
                        time:   [945.23 ps 956.42 ps 967.09 ps]
                        thrpt:  [1.0340 Gelem/s 1.0456 Gelem/s 1.0579 Gelem/s]
Benchmarking hop_latency/single_hop_full: Collecting 100 samples in estimated 5.0000 s (151M iterathop_latency/single_hop_full
                        time:   [32.858 ns 33.033 ns 33.203 ns]
                        thrpt:  [30.118 Melem/s 30.273 Melem/s 30.434 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  4 (4.00%) low severe
  12 (12.00%) low mild

hop_scaling/64B_1hops   time:   [51.948 ns 51.991 ns 52.035 ns]
                        thrpt:  [1.1455 GiB/s 1.1464 GiB/s 1.1474 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low severe
  2 (2.00%) high mild
  1 (1.00%) high severe
hop_scaling/64B_2hops   time:   [85.388 ns 85.921 ns 86.418 ns]
                        thrpt:  [706.28 MiB/s 710.37 MiB/s 714.79 MiB/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  12 (12.00%) high mild
hop_scaling/64B_3hops   time:   [117.32 ns 117.58 ns 117.87 ns]
                        thrpt:  [517.81 MiB/s 519.09 MiB/s 520.22 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
hop_scaling/64B_4hops   time:   [148.67 ns 148.90 ns 149.12 ns]
                        thrpt:  [409.29 MiB/s 409.92 MiB/s 410.54 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
hop_scaling/64B_5hops   time:   [182.06 ns 182.79 ns 183.45 ns]
                        thrpt:  [332.71 MiB/s 333.92 MiB/s 335.25 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
hop_scaling/256B_1hops  time:   [53.863 ns 53.936 ns 54.006 ns]
                        thrpt:  [4.4147 GiB/s 4.4204 GiB/s 4.4264 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low severe
  2 (2.00%) high mild
  1 (1.00%) high severe
hop_scaling/256B_2hops  time:   [89.405 ns 89.793 ns 90.160 ns]
                        thrpt:  [2.6444 GiB/s 2.6552 GiB/s 2.6667 GiB/s]
hop_scaling/256B_3hops  time:   [120.20 ns 120.47 ns 120.73 ns]
                        thrpt:  [1.9747 GiB/s 1.9791 GiB/s 1.9835 GiB/s]
Found 25 outliers among 100 measurements (25.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  15 (15.00%) high mild
  5 (5.00%) high severe
hop_scaling/256B_4hops  time:   [154.41 ns 154.96 ns 155.48 ns]
                        thrpt:  [1.5335 GiB/s 1.5385 GiB/s 1.5441 GiB/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
hop_scaling/256B_5hops  time:   [190.73 ns 191.03 ns 191.36 ns]
                        thrpt:  [1.2459 GiB/s 1.2481 GiB/s 1.2500 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
hop_scaling/1024B_1hops time:   [54.496 ns 54.602 ns 54.697 ns]
                        thrpt:  [17.436 GiB/s 17.466 GiB/s 17.500 GiB/s]
Found 17 outliers among 100 measurements (17.00%)
  10 (10.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_2hops time:   [89.229 ns 89.531 ns 89.798 ns]
                        thrpt:  [10.620 GiB/s 10.652 GiB/s 10.688 GiB/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_3hops time:   [123.63 ns 124.67 ns 125.79 ns]
                        thrpt:  [7.5814 GiB/s 7.6498 GiB/s 7.7137 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/1024B_4hops time:   [158.04 ns 159.07 ns 160.01 ns]
                        thrpt:  [5.9600 GiB/s 5.9952 GiB/s 6.0343 GiB/s]
hop_scaling/1024B_5hops time:   [196.35 ns 197.50 ns 198.57 ns]
                        thrpt:  [4.8027 GiB/s 4.8288 GiB/s 4.8571 GiB/s]

Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samples in estimated 5.0005 multihop_with_routing/route_and_forward/1
                        time:   [151.17 ns 152.03 ns 152.89 ns]
                        thrpt:  [6.5407 Melem/s 6.5777 Melem/s 6.6149 Melem/s]
Found 32 outliers among 100 measurements (32.00%)
  18 (18.00%) low severe
  5 (5.00%) low mild
  9 (9.00%) high mild
Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samples in estimated 5.0003 multihop_with_routing/route_and_forward/2
                        time:   [282.68 ns 284.39 ns 286.02 ns]
                        thrpt:  [3.4962 Melem/s 3.5163 Melem/s 3.5375 Melem/s]
Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samples in estimated 5.0004 multihop_with_routing/route_and_forward/3
                        time:   [412.56 ns 415.06 ns 417.56 ns]
                        thrpt:  [2.3949 Melem/s 2.4093 Melem/s 2.4239 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samples in estimated 5.0026 multihop_with_routing/route_and_forward/4
                        time:   [559.17 ns 559.76 ns 560.37 ns]
                        thrpt:  [1.7845 Melem/s 1.7865 Melem/s 1.7884 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  7 (7.00%) high mild
  9 (9.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samples in estimated 5.0035 multihop_with_routing/route_and_forward/5
                        time:   [694.34 ns 695.45 ns 696.62 ns]
                        thrpt:  [1.4355 Melem/s 1.4379 Melem/s 1.4402 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low mild
  10 (10.00%) high mild
  9 (9.00%) high severe

Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples in estimated 5.0051 s multihop_concurrent/concurrent_forward/4
                        time:   [560.78 µs 564.56 µs 568.52 µs]
                        thrpt:  [7.0358 Melem/s 7.0851 Melem/s 7.1330 Melem/s]
Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples in estimated 5.0853 s multihop_concurrent/concurrent_forward/8
                        time:   [748.75 µs 803.11 µs 857.57 µs]
                        thrpt:  [9.3287 Melem/s 9.9613 Melem/s 10.684 Melem/s]
Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samples in estimated 5.2429 smultihop_concurrent/concurrent_forward/16
                        time:   [1.2421 ms 1.2483 ms 1.2548 ms]
                        thrpt:  [12.751 Melem/s 12.818 Melem/s 12.882 Melem/s]

pingwave/serialize      time:   [520.56 ps 536.54 ps 551.73 ps]
                        thrpt:  [1.8125 Gelem/s 1.8638 Gelem/s 1.9210 Gelem/s]
pingwave/deserialize    time:   [633.91 ps 655.20 ps 678.84 ps]
                        thrpt:  [1.4731 Gelem/s 1.5262 Gelem/s 1.5775 Gelem/s]
pingwave/roundtrip      time:   [624.63 ps 645.41 ps 668.08 ps]
                        thrpt:  [1.4968 Gelem/s 1.5494 Gelem/s 1.6009 Gelem/s]
pingwave/forward        time:   [528.90 ps 542.53 ps 555.48 ps]
                        thrpt:  [1.8003 Gelem/s 1.8432 Gelem/s 1.8907 Gelem/s]

Benchmarking capabilities/serialize_simple: Collecting 100 samples in estimated 5.0002 s (133M itercapabilities/serialize_simple
                        time:   [37.451 ns 37.537 ns 37.619 ns]
                        thrpt:  [26.582 Melem/s 26.640 Melem/s 26.702 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimated 5.0000 s (579M itcapabilities/deserialize_simple
                        time:   [8.7848 ns 8.8392 ns 8.9012 ns]
                        thrpt:  [112.34 Melem/s 113.13 Melem/s 113.83 Melem/s]
Benchmarking capabilities/serialize_complex: Collecting 100 samples in estimated 5.0000 s (130M itecapabilities/serialize_complex
                        time:   [38.141 ns 38.401 ns 38.652 ns]
                        thrpt:  [25.872 Melem/s 26.041 Melem/s 26.219 Melem/s]
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in estimated 5.0018 s (4.9M icapabilities/deserialize_complex
                        time:   [1.0204 µs 1.0230 µs 1.0255 µs]
                        thrpt:  [975.11 Kelem/s 977.50 Kelem/s 980.02 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking local_graph/create_pingwave: Collecting 100 samples in estimated 5.0000 s (1.0B iteratlocal_graph/create_pingwave
                        time:   [4.9074 ns 4.9381 ns 4.9683 ns]
                        thrpt:  [201.28 Melem/s 202.51 Melem/s 203.78 Melem/s]
Found 23 outliers among 100 measurements (23.00%)
  16 (16.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
Benchmarking local_graph/on_pingwave_new: Collecting 100 samples in estimated 5.0001 s (27M iteratilocal_graph/on_pingwave_new
                        time:   [148.48 ns 151.82 ns 154.95 ns]
                        thrpt:  [6.4537 Melem/s 6.5867 Melem/s 6.7349 Melem/s]
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estimated 5.0001 s (312M local_graph/on_pingwave_duplicate
                        time:   [16.429 ns 16.788 ns 17.174 ns]
                        thrpt:  [58.226 Melem/s 59.566 Melem/s 60.870 Melem/s]
local_graph/get_node    time:   [14.802 ns 14.852 ns 14.907 ns]
                        thrpt:  [67.085 Melem/s 67.330 Melem/s 67.556 Melem/s]
local_graph/node_count  time:   [958.14 ns 959.04 ns 959.91 ns]
                        thrpt:  [1.0418 Melem/s 1.0427 Melem/s 1.0437 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  6 (6.00%) low severe
  1 (1.00%) low mild
  9 (9.00%) high mild
  2 (2.00%) high severe
local_graph/stats       time:   [2.8703 µs 2.8759 µs 2.8807 µs]
                        thrpt:  [347.14 Kelem/s 347.72 Kelem/s 348.40 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking graph_scaling/all_nodes/100: Collecting 100 samples in estimated 5.0048 s (672k iteratgraph_scaling/all_nodes/100
                        time:   [7.4567 µs 7.4721 µs 7.4867 µs]
                        thrpt:  [13.357 Melem/s 13.383 Melem/s 13.411 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in estimated 5.0029 s (667graph_scaling/nodes_within_hops/100
                        time:   [7.5178 µs 7.5288 µs 7.5409 µs]
                        thrpt:  [13.261 Melem/s 13.282 Melem/s 13.302 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
Benchmarking graph_scaling/all_nodes/500: Collecting 100 samples in estimated 5.0654 s (318k iteratgraph_scaling/all_nodes/500
                        time:   [16.040 µs 16.091 µs 16.143 µs]
                        thrpt:  [30.973 Melem/s 31.074 Melem/s 31.172 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in estimated 5.0770 s (313graph_scaling/nodes_within_hops/500
                        time:   [16.219 µs 16.272 µs 16.332 µs]
                        thrpt:  [30.616 Melem/s 30.727 Melem/s 30.827 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
Benchmarking graph_scaling/all_nodes/1000: Collecting 100 samples in estimated 5.0968 s (192k iteragraph_scaling/all_nodes/1000
                        time:   [26.783 µs 26.862 µs 26.948 µs]
                        thrpt:  [37.108 Melem/s 37.228 Melem/s 37.337 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in estimated 5.0738 s (19graph_scaling/nodes_within_hops/1000
                        time:   [26.473 µs 26.595 µs 26.725 µs]
                        thrpt:  [37.418 Melem/s 37.600 Melem/s 37.774 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking graph_scaling/all_nodes/5000: Collecting 100 samples in estimated 5.9869 s (25k iteratgraph_scaling/all_nodes/5000
                        time:   [234.96 µs 237.66 µs 240.22 µs]
                        thrpt:  [20.814 Melem/s 21.039 Melem/s 21.280 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in estimated 6.0223 s (25graph_scaling/nodes_within_hops/5000
                        time:   [236.42 µs 238.64 µs 240.79 µs]
                        thrpt:  [20.765 Melem/s 20.952 Melem/s 21.149 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  11 (11.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimated 5.0622 s (167k itcapability_search/find_with_gpu
                        time:   [30.472 µs 30.519 µs 30.568 µs]
                        thrpt:  [32.714 Kelem/s 32.766 Kelem/s 32.817 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples in estimated 5.2390 s (8capability_search/find_by_tool_python
                        time:   [61.120 µs 61.204 µs 61.293 µs]
                        thrpt:  [16.315 Kelem/s 16.339 Kelem/s 16.361 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in estimated 5.0261 s (66kcapability_search/find_by_tool_rust
                        time:   [76.778 µs 76.874 µs 76.975 µs]
                        thrpt:  [12.991 Kelem/s 13.008 Kelem/s 13.025 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples in estimated 5.0102 s (3graph_concurrent/concurrent_pingwave/4
                        time:   [165.10 µs 167.26 µs 169.44 µs]
                        thrpt:  [11.804 Melem/s 11.957 Melem/s 12.114 Melem/s]
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples in estimated 5.0396 s (1graph_concurrent/concurrent_pingwave/8
                        time:   [274.45 µs 282.09 µs 288.62 µs]
                        thrpt:  [13.859 Melem/s 14.180 Melem/s 14.575 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples in estimated 5.0675 s (graph_concurrent/concurrent_pingwave/16
                        time:   [511.12 µs 524.87 µs 538.52 µs]
                        thrpt:  [14.855 Melem/s 15.242 Melem/s 15.652 Melem/s]

Benchmarking path_finding/path_1_hop: Collecting 100 samples in estimated 5.0193 s (904k iterationspath_finding/path_1_hop time:   [5.4959 µs 5.5278 µs 5.5569 µs]
                        thrpt:  [179.96 Kelem/s 180.91 Kelem/s 181.95 Kelem/s]
Benchmarking path_finding/path_2_hops: Collecting 100 samples in estimated 5.0129 s (899k iterationpath_finding/path_2_hops
                        time:   [5.5025 µs 5.5363 µs 5.5704 µs]
                        thrpt:  [179.52 Kelem/s 180.63 Kelem/s 181.73 Kelem/s]
Benchmarking path_finding/path_4_hops: Collecting 100 samples in estimated 5.0164 s (853k iterationpath_finding/path_4_hops
                        time:   [5.8732 µs 5.8806 µs 5.8886 µs]
                        thrpt:  [169.82 Kelem/s 170.05 Kelem/s 170.26 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking path_finding/path_not_found: Collecting 100 samples in estimated 5.0015 s (869k iteratpath_finding/path_not_found
                        time:   [5.7174 µs 5.7318 µs 5.7441 µs]
                        thrpt:  [174.09 Kelem/s 174.47 Kelem/s 174.90 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking path_finding/path_complex_graph: Collecting 100 samples in estimated 5.3912 s (30k itepath_finding/path_complex_graph
                        time:   [177.20 µs 177.70 µs 178.14 µs]
                        thrpt:  [5.6136 Kelem/s 5.6274 Kelem/s 5.6432 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in estimated 5.0001 s (141failure_detector/heartbeat_existing
                        time:   [35.187 ns 35.267 ns 35.336 ns]
                        thrpt:  [28.300 Melem/s 28.355 Melem/s 28.419 Melem/s]
Found 23 outliers among 100 measurements (23.00%)
  10 (10.00%) low severe
  2 (2.00%) high mild
  11 (11.00%) high severe
Benchmarking failure_detector/heartbeat_new: Collecting 100 samples in estimated 5.0012 s (21M iterfailure_detector/heartbeat_new
                        time:   [170.36 ns 174.10 ns 178.17 ns]
                        thrpt:  [5.6126 Melem/s 5.7438 Melem/s 5.8700 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low mild
  7 (7.00%) high mild
Benchmarking failure_detector/status_check: Collecting 100 samples in estimated 5.0000 s (372M iterfailure_detector/status_check
                        time:   [13.377 ns 13.418 ns 13.453 ns]
                        thrpt:  [74.335 Melem/s 74.527 Melem/s 74.755 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 34.2s, or reduce sample count to 10.
Benchmarking failure_detector/check_all: Collecting 100 samples in estimated 34.182 s (100 iteratiofailure_detector/check_all
                        time:   [329.97 ms 331.65 ms 333.32 ms]
                        thrpt:  [3.0001  elem/s 3.0152  elem/s 3.0306  elem/s]
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 10.0s, or reduce sample count to 40.
failure_detector/stats  time:   [99.931 ms 100.06 ms 100.19 ms]
                        thrpt:  [9.9808  elem/s 9.9939  elem/s 10.007  elem/s]

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimated 5.0000 s (452M itloss_simulator/should_drop_1pct
                        time:   [11.033 ns 11.051 ns 11.066 ns]
                        thrpt:  [90.365 Melem/s 90.492 Melem/s 90.636 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) low severe
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimated 5.0000 s (436M itloss_simulator/should_drop_5pct
                        time:   [11.458 ns 11.468 ns 11.478 ns]
                        thrpt:  [87.125 Melem/s 87.198 Melem/s 87.275 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low severe
  4 (4.00%) high mild
  8 (8.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estimated 5.0000 s (417M iloss_simulator/should_drop_10pct
                        time:   [11.952 ns 11.974 ns 11.992 ns]
                        thrpt:  [83.386 Melem/s 83.518 Melem/s 83.665 Melem/s]
Found 25 outliers among 100 measurements (25.00%)
  11 (11.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estimated 5.0001 s (384M iloss_simulator/should_drop_20pct
                        time:   [12.986 ns 13.007 ns 13.025 ns]
                        thrpt:  [76.777 Melem/s 76.884 Melem/s 77.004 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  7 (7.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  7 (7.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estimated 5.0001 s (432M iloss_simulator/should_drop_burst
                        time:   [11.513 ns 11.540 ns 11.563 ns]
                        thrpt:  [86.481 Melem/s 86.659 Melem/s 86.859 Melem/s]
Found 29 outliers among 100 measurements (29.00%)
  12 (12.00%) low severe
  5 (5.00%) high mild
  12 (12.00%) high severe

Benchmarking circuit_breaker/allow_closed: Collecting 100 samples in estimated 5.0000 s (492M iteracircuit_breaker/allow_closed
                        time:   [10.107 ns 10.166 ns 10.218 ns]
                        thrpt:  [97.863 Melem/s 98.371 Melem/s 98.941 Melem/s]
Benchmarking circuit_breaker/record_success: Collecting 100 samples in estimated 5.0000 s (580M itecircuit_breaker/record_success
                        time:   [8.5931 ns 8.6455 ns 8.6977 ns]
                        thrpt:  [114.97 Melem/s 115.67 Melem/s 116.37 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  6 (6.00%) low severe
  14 (14.00%) low mild
Benchmarking circuit_breaker/record_failure: Collecting 100 samples in estimated 5.0000 s (505M itecircuit_breaker/record_failure
                        time:   [9.8820 ns 9.9009 ns 9.9157 ns]
                        thrpt:  [100.85 Melem/s 101.00 Melem/s 101.19 Melem/s]
Found 26 outliers among 100 measurements (26.00%)
  12 (12.00%) low severe
  3 (3.00%) low mild
  8 (8.00%) high mild
  3 (3.00%) high severe
circuit_breaker/state   time:   [10.249 ns 10.263 ns 10.275 ns]
                        thrpt:  [97.320 Melem/s 97.435 Melem/s 97.571 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  8 (8.00%) high mild
  3 (3.00%) high severe

Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samples in estimated 5.000recovery_manager/on_failure_with_alternates
                        time:   [250.96 ns 254.20 ns 257.40 ns]
                        thrpt:  [3.8850 Melem/s 3.9339 Melem/s 3.9846 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) low mild
  1 (1.00%) high mild
Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samples in estimated 5.0007 recovery_manager/on_failure_no_alternates
                        time:   [212.30 ns 222.76 ns 239.71 ns]
                        thrpt:  [4.1718 Melem/s 4.4891 Melem/s 4.7104 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  11 (11.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking recovery_manager/get_action: Collecting 100 samples in estimated 5.0000 s (132M iteratrecovery_manager/get_action
                        time:   [37.787 ns 37.864 ns 37.936 ns]
                        thrpt:  [26.360 Melem/s 26.410 Melem/s 26.464 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking recovery_manager/is_failed: Collecting 100 samples in estimated 5.0001 s (389M iteratirecovery_manager/is_failed
                        time:   [12.793 ns 12.822 ns 12.846 ns]
                        thrpt:  [77.843 Melem/s 77.993 Melem/s 78.169 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) low severe
  9 (9.00%) high mild
  1 (1.00%) high severe
Benchmarking recovery_manager/on_recovery: Collecting 100 samples in estimated 5.0004 s (38M iteratrecovery_manager/on_recovery
                        time:   [124.21 ns 124.46 ns 124.70 ns]
                        thrpt:  [8.0190 Melem/s 8.0346 Melem/s 8.0510 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high severe
recovery_manager/stats  time:   [1.2051 ns 1.2060 ns 1.2069 ns]
                        thrpt:  [828.60 Melem/s 829.22 Melem/s 829.79 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  14 (14.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking failure_scaling/check_all/100: Collecting 100 samples in estimated 5.0173 s (596k iterfailure_scaling/check_all/100
                        time:   [8.8165 µs 8.8337 µs 8.8491 µs]
                        thrpt:  [11.301 Melem/s 11.320 Melem/s 11.342 Melem/s]
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estimated 5.0129 s (717k failure_scaling/healthy_nodes/100
                        time:   [7.0057 µs 7.0171 µs 7.0270 µs]
                        thrpt:  [14.231 Melem/s 14.251 Melem/s 14.274 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/check_all/500: Collecting 100 samples in estimated 5.0674 s (237k iterfailure_scaling/check_all/500
                        time:   [23.011 µs 23.103 µs 23.182 µs]
                        thrpt:  [21.569 Melem/s 21.642 Melem/s 21.729 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estimated 5.0272 s (510k failure_scaling/healthy_nodes/500
                        time:   [9.7747 µs 9.8395 µs 9.9056 µs]
                        thrpt:  [50.476 Melem/s 50.816 Melem/s 51.153 Melem/s]
Benchmarking failure_scaling/check_all/1000: Collecting 100 samples in estimated 5.1817 s (141k itefailure_scaling/check_all/1000
                        time:   [41.010 µs 41.088 µs 41.152 µs]
                        thrpt:  [24.300 Melem/s 24.338 Melem/s 24.384 Melem/s]
Found 25 outliers among 100 measurements (25.00%)
  20 (20.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in estimated 5.0294 s (338kfailure_scaling/healthy_nodes/1000
                        time:   [14.866 µs 14.881 µs 14.899 µs]
                        thrpt:  [67.120 Melem/s 67.199 Melem/s 67.268 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/check_all/5000: Collecting 100 samples in estimated 5.7941 s (35k iterfailure_scaling/check_all/5000
                        time:   [182.21 µs 182.98 µs 183.68 µs]
                        thrpt:  [27.221 Melem/s 27.325 Melem/s 27.441 Melem/s]
Found 27 outliers among 100 measurements (27.00%)
  15 (15.00%) low severe
  4 (4.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in estimated 5.1263 s (96k failure_scaling/healthy_nodes/5000
                        time:   [53.505 µs 53.553 µs 53.608 µs]
                        thrpt:  [93.270 Melem/s 93.365 Melem/s 93.449 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  6 (6.00%) high severe

Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samples in estimated 5.0257 sfailure_concurrent/concurrent_heartbeat/4
                        time:   [205.05 µs 207.56 µs 210.24 µs]
                        thrpt:  [9.5131 Melem/s 9.6357 Melem/s 9.7539 Melem/s]
Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samples in estimated 5.0116 sfailure_concurrent/concurrent_heartbeat/8
                        time:   [312.64 µs 323.27 µs 334.82 µs]
                        thrpt:  [11.947 Melem/s 12.374 Melem/s 12.794 Melem/s]
Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samples in estimated 5.0126 failure_concurrent/concurrent_heartbeat/16
                        time:   [553.71 µs 568.92 µs 585.09 µs]
                        thrpt:  [13.673 Melem/s 14.062 Melem/s 14.448 Melem/s]

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estimated 5.0002 s (15M ifailure_recovery_cycle/full_cycle
                        time:   [250.67 ns 255.40 ns 260.18 ns]
                        thrpt:  [3.8435 Melem/s 3.9154 Melem/s 3.9893 Melem/s]

capability_set/create   time:   [775.95 ns 777.18 ns 778.40 ns]
                        thrpt:  [1.2847 Melem/s 1.2867 Melem/s 1.2887 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
Benchmarking capability_set/serialize: Collecting 100 samples in estimated 5.0035 s (7.0M iterationcapability_set/serialize
                        time:   [711.19 ns 712.47 ns 713.74 ns]
                        thrpt:  [1.4011 Melem/s 1.4036 Melem/s 1.4061 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
Benchmarking capability_set/deserialize: Collecting 100 samples in estimated 5.0092 s (1.6M iteraticapability_set/deserialize
                        time:   [3.0821 µs 3.0854 µs 3.0890 µs]
                        thrpt:  [323.73 Kelem/s 324.11 Kelem/s 324.45 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low severe
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_set/roundtrip: Collecting 100 samples in estimated 5.0118 s (1.3M iterationcapability_set/roundtrip
                        time:   [3.9690 µs 3.9750 µs 3.9811 µs]
                        thrpt:  [251.19 Kelem/s 251.57 Kelem/s 251.95 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
capability_set/has_tag  time:   [602.73 ps 605.35 ps 609.03 ps]
                        thrpt:  [1.6419 Gelem/s 1.6519 Gelem/s 1.6591 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  11 (11.00%) high severe
Benchmarking capability_set/has_model: Collecting 100 samples in estimated 5.0000 s (11B iterationscapability_set/has_model
                        time:   [441.07 ps 445.66 ps 450.38 ps]
                        thrpt:  [2.2203 Gelem/s 2.2438 Gelem/s 2.2672 Gelem/s]
Benchmarking capability_set/has_tool: Collecting 100 samples in estimated 5.0000 s (8.3B iterationscapability_set/has_tool time:   [603.49 ps 605.69 ps 608.96 ps]
                        thrpt:  [1.6421 Gelem/s 1.6510 Gelem/s 1.6570 Gelem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low mild
  12 (12.00%) high severe
capability_set/has_gpu  time:   [198.95 ps 199.80 ps 200.50 ps]
                        thrpt:  [4.9874 Gelem/s 5.0051 Gelem/s 5.0263 Gelem/s]
Found 23 outliers among 100 measurements (23.00%)
  17 (17.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_announcement/create: Collecting 100 samples in estimated 5.0003 s (2.1M itecapability_announcement/create
                        time:   [2.3342 µs 2.3375 µs 2.3411 µs]
                        thrpt:  [427.15 Kelem/s 427.81 Kelem/s 428.41 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  8 (8.00%) high severe
Benchmarking capability_announcement/serialize: Collecting 100 samples in estimated 5.0006 s (2.9M capability_announcement/serialize
                        time:   [1.7291 µs 1.7309 µs 1.7331 µs]
                        thrpt:  [577.00 Kelem/s 577.72 Kelem/s 578.35 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in estimated 5.0023 s (1.8capability_announcement/deserialize
                        time:   [2.7489 µs 2.7562 µs 2.7622 µs]
                        thrpt:  [362.03 Kelem/s 362.82 Kelem/s 363.78 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_announcement/is_expired: Collecting 100 samples in estimated 5.0001 s (230Mcapability_announcement/is_expired
                        time:   [21.635 ns 21.715 ns 21.782 ns]
                        thrpt:  [45.910 Melem/s 46.051 Melem/s 46.221 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) low severe
  3 (3.00%) low mild
  5 (5.00%) high mild

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in estimated 5.0000 s (1.5Bcapability_filter/match_single_tag
                        time:   [3.4273 ns 3.4333 ns 3.4403 ns]
                        thrpt:  [290.68 Melem/s 291.26 Melem/s 291.78 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in estimated 5.0000 s (2.8capability_filter/match_require_gpu
                        time:   [1.7683 ns 1.7827 ns 1.7978 ns]
                        thrpt:  [556.23 Melem/s 560.96 Melem/s 565.51 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  18 (18.00%) low mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in estimated 5.0000 s (2.5Bcapability_filter/match_gpu_vendor
                        time:   [1.9577 ns 1.9699 ns 1.9819 ns]
                        thrpt:  [504.56 Melem/s 507.64 Melem/s 510.82 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in estimated 5.0000 s (2.8Bcapability_filter/match_min_memory
                        time:   [1.7660 ns 1.7762 ns 1.7867 ns]
                        thrpt:  [559.69 Melem/s 562.99 Melem/s 566.27 Melem/s]
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimated 5.0000 s (870M itcapability_filter/match_complex
                        time:   [5.6560 ns 5.7069 ns 5.7581 ns]
                        thrpt:  [173.67 Melem/s 175.23 Melem/s 176.80 Melem/s]
Benchmarking capability_filter/match_no_match: Collecting 100 samples in estimated 5.0000 s (2.8B icapability_filter/match_no_match
                        time:   [1.7509 ns 1.7608 ns 1.7705 ns]
                        thrpt:  [564.80 Melem/s 567.91 Melem/s 571.12 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples in estimated 5.1111 s capability_index_insert/index_nodes/100
                        time:   [168.23 µs 168.55 µs 168.88 µs]
                        thrpt:  [592.14 Kelem/s 593.28 Kelem/s 594.41 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samples in estimated 7.0532 scapability_index_insert/index_nodes/1000
                        time:   [1.3945 ms 1.3966 ms 1.3986 ms]
                        thrpt:  [714.98 Kelem/s 716.05 Kelem/s 717.11 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samples in estimated 6.2250 capability_index_insert/index_nodes/10000
                        time:   [15.482 ms 15.626 ms 15.773 ms]
                        thrpt:  [633.98 Kelem/s 639.96 Kelem/s 645.91 Kelem/s]

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples in estimated 5.3013 s capability_index_query/query_single_tag
                        time:   [174.76 µs 175.95 µs 177.10 µs]
                        thrpt:  [5.6465 Kelem/s 5.6833 Kelem/s 5.7222 Kelem/s]
Found 26 outliers among 100 measurements (26.00%)
  4 (4.00%) low severe
  10 (10.00%) low mild
  12 (12.00%) high mild
Benchmarking capability_index_query/query_require_gpu: Collecting 100 samples in estimated 5.5302 scapability_index_query/query_require_gpu
                        time:   [179.59 µs 180.63 µs 181.68 µs]
                        thrpt:  [5.5041 Kelem/s 5.5362 Kelem/s 5.5683 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples in estimated 5.1309 s capability_index_query/query_gpu_vendor
                        time:   [505.84 µs 508.12 µs 510.33 µs]
                        thrpt:  [1.9595 Kelem/s 1.9681 Kelem/s 1.9769 Kelem/s]
Found 20 outliers among 100 measurements (20.00%)
  3 (3.00%) low severe
  11 (11.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples in estimated 5.3775 s capability_index_query/query_min_memory
                        time:   [530.99 µs 533.36 µs 535.71 µs]
                        thrpt:  [1.8667 Kelem/s 1.8749 Kelem/s 1.8833 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_query/query_complex: Collecting 100 samples in estimated 5.0790 s (15capability_index_query/query_complex
                        time:   [334.47 µs 335.95 µs 337.39 µs]
                        thrpt:  [2.9639 Kelem/s 2.9766 Kelem/s 2.9898 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_model: Collecting 100 samples in estimated 5.3223 s (56k capability_index_query/query_model
                        time:   [96.022 µs 96.330 µs 96.611 µs]
                        thrpt:  [10.351 Kelem/s 10.381 Kelem/s 10.414 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estimated 7.4855 s (15k icapability_index_query/query_tool
                        time:   [493.19 µs 494.54 µs 496.05 µs]
                        thrpt:  [2.0159 Kelem/s 2.0221 Kelem/s 2.0276 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  10 (10.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_no_results: Collecting 100 samples in estimated 5.0001 s capability_index_query/query_no_results
                        time:   [28.869 ns 28.914 ns 28.961 ns]
                        thrpt:  [34.529 Melem/s 34.585 Melem/s 34.639 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_index_find_best/find_best_simple: Collecting 100 samples in estimated 5.291capability_index_find_best/find_best_simple
                        time:   [209.28 µs 209.77 µs 210.27 µs]
                        thrpt:  [4.7557 Kelem/s 4.7670 Kelem/s 4.7782 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 6capability_index_find_best/find_best_with_prefs
                        time:   [429.91 µs 431.18 µs 432.52 µs]
                        thrpt:  [2.3120 Kelem/s 2.3192 Kelem/s 2.3261 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples in estimated 5.0240 s capability_index_scaling/query_tag/1000
                        time:   [10.289 µs 10.348 µs 10.405 µs]
                        thrpt:  [96.111 Kelem/s 96.634 Kelem/s 97.189 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 samples in estimated 5.021capability_index_scaling/query_complex/1000
                        time:   [27.380 µs 27.562 µs 27.737 µs]
                        thrpt:  [36.053 Kelem/s 36.281 Kelem/s 36.524 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples in estimated 5.2269 s capability_index_scaling/query_tag/5000
                        time:   [54.070 µs 54.408 µs 54.741 µs]
                        thrpt:  [18.268 Kelem/s 18.380 Kelem/s 18.495 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 samples in estimated 5.499capability_index_scaling/query_complex/5000
                        time:   [135.87 µs 136.51 µs 137.12 µs]
                        thrpt:  [7.2931 Kelem/s 7.3252 Kelem/s 7.3599 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samples in estimated 5.2382 scapability_index_scaling/query_tag/10000
                        time:   [169.41 µs 171.99 µs 174.43 µs]
                        thrpt:  [5.7330 Kelem/s 5.8142 Kelem/s 5.9030 Kelem/s]
Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samples in estimated 5.12capability_index_scaling/query_complex/10000
                        time:   [336.52 µs 338.57 µs 340.60 µs]
                        thrpt:  [2.9360 Kelem/s 2.9536 Kelem/s 2.9716 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
Benchmarking capability_index_scaling/query_tag/50000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.3s, enable flat sampling, or reduce sample count to 60.
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samples in estimated 6.2663 scapability_index_scaling/query_tag/50000
                        time:   [1.2132 ms 1.2324 ms 1.2526 ms]
                        thrpt:  [798.34  elem/s 811.41  elem/s 824.24  elem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samples in estimated 5.06capability_index_scaling/query_complex/50000
                        time:   [2.1245 ms 2.1659 ms 2.2095 ms]
                        thrpt:  [452.59  elem/s 461.69  elem/s 470.69  elem/s]

Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 samples in estimated 5.1capability_index_concurrent/concurrent_index/4
                        time:   [561.34 µs 568.09 µs 574.61 µs]
                        thrpt:  [3.4806 Melem/s 3.5206 Melem/s 3.5629 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.1s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 samples in estimated 5.0capability_index_concurrent/concurrent_query/4
                        time:   [247.70 ms 249.54 ms 251.37 ms]
                        thrpt:  [7.9565 Kelem/s 8.0147 Kelem/s 8.0742 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 7.3capability_index_concurrent/concurrent_mixed/4
                        time:   [122.36 ms 123.04 ms 123.76 ms]
                        thrpt:  [16.161 Kelem/s 16.255 Kelem/s 16.346 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 samples in estimated 5.1capability_index_concurrent/concurrent_index/8
                        time:   [1.1191 ms 1.1444 ms 1.1678 ms]
                        thrpt:  [3.4253 Melem/s 3.4954 Melem/s 3.5742 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.2s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 samples in estimated 6.1capability_index_concurrent/concurrent_query/8
                        time:   [294.88 ms 301.72 ms 308.24 ms]
                        thrpt:  [12.977 Kelem/s 13.257 Kelem/s 13.565 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 6.1capability_index_concurrent/concurrent_mixed/8
                        time:   [154.19 ms 157.39 ms 160.53 ms]
                        thrpt:  [24.917 Kelem/s 25.415 Kelem/s 25.942 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.capability_index_concurrent/concurrent_index/16
                        time:   [1.2989 ms 1.3039 ms 1.3081 ms]
                        thrpt:  [6.1157 Melem/s 6.1353 Melem/s 6.1592 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.7s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 samples in estimated 8.capability_index_concurrent/concurrent_query/16
                        time:   [434.49 ms 436.90 ms 439.14 ms]
                        thrpt:  [18.218 Kelem/s 18.311 Kelem/s 18.412 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 9.capability_index_concurrent/concurrent_mixed/16
                        time:   [230.73 ms 231.72 ms 232.73 ms]
                        thrpt:  [34.375 Kelem/s 34.525 Kelem/s 34.672 Kelem/s]

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.capability_index_updates/update_higher_version
                        time:   [812.03 ns 817.06 ns 821.98 ns]
                        thrpt:  [1.2166 Melem/s 1.2239 Melem/s 1.2315 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_updates/update_same_version: Collecting 100 samples in estimated 5.00capability_index_updates/update_same_version
                        time:   [814.02 ns 819.17 ns 824.17 ns]
                        thrpt:  [1.2133 Melem/s 1.2208 Melem/s 1.2285 Melem/s]
Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samples in estimated 5.0065 capability_index_updates/remove_and_readd
                        time:   [1.2859 µs 1.2931 µs 1.2996 µs]
                        thrpt:  [769.49 Kelem/s 773.35 Kelem/s 777.64 Kelem/s]

diff_op/create_add_tag  time:   [24.758 ns 24.805 ns 24.855 ns]
                        thrpt:  [40.234 Melem/s 40.315 Melem/s 40.391 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
Benchmarking diff_op/create_remove_tag: Collecting 100 samples in estimated 5.0001 s (203M iteratiodiff_op/create_remove_tag
                        time:   [24.542 ns 24.595 ns 24.646 ns]
                        thrpt:  [40.575 Melem/s 40.658 Melem/s 40.746 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_op/create_add_model: Collecting 100 samples in estimated 5.0001 s (57M iterationsdiff_op/create_add_model
                        time:   [87.707 ns 87.985 ns 88.343 ns]
                        thrpt:  [11.319 Melem/s 11.366 Melem/s 11.402 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking diff_op/create_update_model: Collecting 100 samples in estimated 5.0001 s (202M iteratdiff_op/create_update_model
                        time:   [24.563 ns 24.611 ns 24.658 ns]
                        thrpt:  [40.554 Melem/s 40.633 Melem/s 40.712 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  8 (8.00%) high mild
diff_op/estimated_size  time:   [979.46 ps 985.91 ps 992.39 ps]
                        thrpt:  [1.0077 Gelem/s 1.0143 Gelem/s 1.0210 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) low mild

capability_diff/create  time:   [84.331 ns 84.842 ns 85.308 ns]
                        thrpt:  [11.722 Melem/s 11.787 Melem/s 11.858 Melem/s]
Benchmarking capability_diff/serialize: Collecting 100 samples in estimated 5.0003 s (36M iterationcapability_diff/serialize
                        time:   [139.58 ns 140.29 ns 140.97 ns]
                        thrpt:  [7.0936 Melem/s 7.1279 Melem/s 7.1642 Melem/s]
Found 25 outliers among 100 measurements (25.00%)
  18 (18.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_diff/deserialize: Collecting 100 samples in estimated 5.0007 s (21M iteraticapability_diff/deserialize
                        time:   [241.00 ns 242.37 ns 243.64 ns]
                        thrpt:  [4.1045 Melem/s 4.1259 Melem/s 4.1494 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  11 (11.00%) low severe
  5 (5.00%) low mild
Benchmarking capability_diff/estimated_size: Collecting 100 samples in estimated 5.0000 s (2.1B itecapability_diff/estimated_size
                        time:   [2.4063 ns 2.4113 ns 2.4156 ns]
                        thrpt:  [413.98 Melem/s 414.72 Melem/s 415.57 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  8 (8.00%) high mild
  7 (7.00%) high severe

Benchmarking diff_generation/no_changes: Collecting 100 samples in estimated 5.0009 s (17M iteratiodiff_generation/no_changes
                        time:   [289.88 ns 290.35 ns 290.81 ns]
                        thrpt:  [3.4387 Melem/s 3.4441 Melem/s 3.4497 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) low severe
  8 (8.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/add_one_tag: Collecting 100 samples in estimated 5.0012 s (12M iteratidiff_generation/add_one_tag
                        time:   [405.98 ns 406.59 ns 407.20 ns]
                        thrpt:  [2.4558 Melem/s 2.4595 Melem/s 2.4632 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  15 (15.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in estimated 5.0012 s (11diff_generation/multiple_tag_changes
                        time:   [459.42 ns 460.15 ns 460.88 ns]
                        thrpt:  [2.1698 Melem/s 2.1732 Melem/s 2.1766 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in estimated 5.0003 s (14Mdiff_generation/update_model_loaded
                        time:   [347.03 ns 347.79 ns 348.63 ns]
                        thrpt:  [2.8684 Melem/s 2.8753 Melem/s 2.8816 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_generation/update_memory: Collecting 100 samples in estimated 5.0011 s (16M iteradiff_generation/update_memory
                        time:   [319.49 ns 321.35 ns 323.01 ns]
                        thrpt:  [3.0958 Melem/s 3.1118 Melem/s 3.1299 Melem/s]
Benchmarking diff_generation/add_model: Collecting 100 samples in estimated 5.0007 s (12M iterationdiff_generation/add_model
                        time:   [432.82 ns 434.98 ns 437.02 ns]
                        thrpt:  [2.2882 Melem/s 2.2989 Melem/s 2.3105 Melem/s]
Benchmarking diff_generation/complex_diff: Collecting 100 samples in estimated 5.0007 s (6.4M iteradiff_generation/complex_diff
                        time:   [786.36 ns 791.00 ns 795.32 ns]
                        thrpt:  [1.2574 Melem/s 1.2642 Melem/s 1.2717 Melem/s]

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estimated 5.0012 s (8.1M idiff_application/apply_single_op
                        time:   [614.99 ns 618.15 ns 621.13 ns]
                        thrpt:  [1.6100 Melem/s 1.6177 Melem/s 1.6261 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  7 (7.00%) low severe
  8 (8.00%) low mild
  5 (5.00%) high mild
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estimated 5.0064 s (3.3M diff_application/apply_small_diff
                        time:   [1.5004 µs 1.5025 µs 1.5049 µs]
                        thrpt:  [664.52 Kelem/s 665.55 Kelem/s 666.49 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in estimated 5.0014 s (2.4Mdiff_application/apply_medium_diff
                        time:   [2.0673 µs 2.0699 µs 2.0726 µs]
                        thrpt:  [482.49 Kelem/s 483.11 Kelem/s 483.72 Kelem/s]
Found 17 outliers among 100 measurements (17.00%)
  8 (8.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in estimated 5.0020 s (8.0Mdiff_application/apply_strict_mode
                        time:   [622.51 ns 624.14 ns 625.92 ns]
                        thrpt:  [1.5976 Melem/s 1.6022 Melem/s 1.6064 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples in estimated 5.0000 s diff_chain_validation/validate_chain_10
                        time:   [4.1697 ns 4.2094 ns 4.2491 ns]
                        thrpt:  [235.34 Melem/s 237.57 Melem/s 239.83 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samples in estimated 5.0001 sdiff_chain_validation/validate_chain_100
                        time:   [48.922 ns 49.164 ns 49.384 ns]
                        thrpt:  [20.249 Melem/s 20.340 Melem/s 20.441 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimated 5.0014 s (732k itdiff_compaction/compact_5_diffs
                        time:   [6.7960 µs 6.8169 µs 6.8343 µs]
                        thrpt:  [146.32 Kelem/s 146.69 Kelem/s 147.15 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estimated 5.0448 s (167k idiff_compaction/compact_20_diffs
                        time:   [30.135 µs 30.198 µs 30.258 µs]
                        thrpt:  [33.049 Kelem/s 33.115 Kelem/s 33.184 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) low severe
  4 (4.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estimated 5.0021 s (7.1M bandwidth_savings/calculate_small
                        time:   [702.07 ns 703.58 ns 704.90 ns]
                        thrpt:  [1.4186 Melem/s 1.4213 Melem/s 1.4244 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in estimated 5.0014 s (7.0Mbandwidth_savings/calculate_medium
                        time:   [710.17 ns 711.47 ns 712.93 ns]
                        thrpt:  [1.4027 Melem/s 1.4055 Melem/s 1.4081 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in estimated 5.0120 s (2.diff_roundtrip/generate_apply_verify
                        time:   [2.4073 µs 2.4136 µs 2.4198 µs]
                        thrpt:  [413.26 Kelem/s 414.32 Kelem/s 415.40 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) low mild
  2 (2.00%) high severe

location_info/create    time:   [54.630 ns 54.822 ns 54.988 ns]
                        thrpt:  [18.186 Melem/s 18.241 Melem/s 18.305 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  3 (3.00%) low mild
  16 (16.00%) high severe
Benchmarking location_info/distance_to: Collecting 100 samples in estimated 5.0000 s (1.9B iteratiolocation_info/distance_to
                        time:   [2.6491 ns 2.6569 ns 2.6647 ns]
                        thrpt:  [375.28 Melem/s 376.37 Melem/s 377.48 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking location_info/same_continent: Collecting 100 samples in estimated 5.0000 s (1.9B iteralocation_info/same_continent
                        time:   [2.6082 ns 2.6155 ns 2.6235 ns]
                        thrpt:  [381.17 Melem/s 382.34 Melem/s 383.40 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  5 (5.00%) low severe
  3 (3.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in estimated 5.0000 s (25B location_info/same_continent_cross
                        time:   [196.09 ps 197.24 ps 198.32 ps]
                        thrpt:  [5.0424 Gelem/s 5.0700 Gelem/s 5.0997 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  19 (19.00%) low mild
Benchmarking location_info/same_region: Collecting 100 samples in estimated 5.0000 s (2.5B iteratiolocation_info/same_region
                        time:   [1.9567 ns 1.9688 ns 1.9807 ns]
                        thrpt:  [504.86 Melem/s 507.92 Melem/s 511.07 Melem/s]

topology_hints/create   time:   [3.4869 ns 3.5173 ns 3.5460 ns]
                        thrpt:  [282.01 Melem/s 284.31 Melem/s 286.79 Melem/s]
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in estimated 5.0000 s (25B itopology_hints/connectivity_score
                        time:   [196.02 ps 197.27 ps 198.51 ps]
                        thrpt:  [5.0376 Gelem/s 5.0692 Gelem/s 5.1014 Gelem/s]
Found 23 outliers among 100 measurements (23.00%)
  19 (19.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in estimated 5.0000 s (19topology_hints/average_latency_empty
                        time:   [261.14 ps 267.17 ps 273.84 ps]
                        thrpt:  [3.6517 Gelem/s 3.7429 Gelem/s 3.8293 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estimated 5.0000 s (106Mtopology_hints/average_latency_100
                        time:   [47.059 ns 47.357 ns 47.637 ns]
                        thrpt:  [20.992 Melem/s 21.116 Melem/s 21.250 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  15 (15.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild

nat_type/difficulty     time:   [197.26 ps 198.41 ps 199.47 ps]
                        thrpt:  [5.0133 Gelem/s 5.0400 Gelem/s 5.0695 Gelem/s]
Benchmarking nat_type/can_connect_direct: Collecting 100 samples in estimated 5.0000 s (25B iteratinat_type/can_connect_direct
                        time:   [197.44 ps 198.48 ps 199.41 ps]
                        thrpt:  [5.0147 Gelem/s 5.0384 Gelem/s 5.0649 Gelem/s]
Benchmarking nat_type/can_connect_symmetric: Collecting 100 samples in estimated 5.0000 s (25B iternat_type/can_connect_symmetric
                        time:   [196.31 ps 197.46 ps 198.53 ps]
                        thrpt:  [5.0370 Gelem/s 5.0643 Gelem/s 5.0939 Gelem/s]

Benchmarking node_metadata/create_simple: Collecting 100 samples in estimated 5.0000 s (162M iteratnode_metadata/create_simple
                        time:   [34.529 ns 35.046 ns 35.496 ns]
                        thrpt:  [28.172 Melem/s 28.534 Melem/s 28.961 Melem/s]
Benchmarking node_metadata/create_full: Collecting 100 samples in estimated 5.0013 s (12M iterationnode_metadata/create_full
                        time:   [428.28 ns 428.89 ns 429.51 ns]
                        thrpt:  [2.3282 Melem/s 2.3316 Melem/s 2.3349 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  9 (9.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking node_metadata/routing_score: Collecting 100 samples in estimated 5.0000 s (25B iteratinode_metadata/routing_score
                        time:   [200.26 ps 200.71 ps 201.07 ps]
                        thrpt:  [4.9735 Gelem/s 4.9824 Gelem/s 4.9934 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  7 (7.00%) high severe
node_metadata/age       time:   [24.972 ns 25.014 ns 25.051 ns]
                        thrpt:  [39.918 Melem/s 39.978 Melem/s 40.044 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
node_metadata/is_stale  time:   [24.390 ns 24.422 ns 24.452 ns]
                        thrpt:  [40.896 Melem/s 40.946 Melem/s 41.000 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking node_metadata/serialize: Collecting 100 samples in estimated 5.0025 s (8.2M iterationsnode_metadata/serialize time:   [604.86 ns 606.02 ns 607.07 ns]
                        thrpt:  [1.6473 Melem/s 1.6501 Melem/s 1.6533 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking node_metadata/deserialize: Collecting 100 samples in estimated 5.0004 s (2.9M iterationode_metadata/deserialize
                        time:   [1.6919 µs 1.6950 µs 1.6979 µs]
                        thrpt:  [588.96 Kelem/s 589.98 Kelem/s 591.06 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking metadata_query/match_status: Collecting 100 samples in estimated 5.0000 s (2.1B iteratmetadata_query/match_status
                        time:   [2.4130 ns 2.4176 ns 2.4238 ns]
                        thrpt:  [412.58 Melem/s 413.64 Melem/s 414.42 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_min_tier: Collecting 100 samples in estimated 5.0000 s (2.1B itermetadata_query/match_min_tier
                        time:   [2.4072 ns 2.4152 ns 2.4233 ns]
                        thrpt:  [412.66 Melem/s 414.04 Melem/s 415.43 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  6 (6.00%) low severe
  2 (2.00%) high mild
  9 (9.00%) high severe
Benchmarking metadata_query/match_continent: Collecting 100 samples in estimated 5.0000 s (1.1B itemetadata_query/match_continent
                        time:   [4.6223 ns 4.6296 ns 4.6373 ns]
                        thrpt:  [215.64 Melem/s 216.00 Melem/s 216.34 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_query/match_complex: Collecting 100 samples in estimated 5.0000 s (1.0B iterametadata_query/match_complex
                        time:   [4.7955 ns 4.8476 ns 4.9045 ns]
                        thrpt:  [203.89 Melem/s 206.29 Melem/s 208.53 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_no_match: Collecting 100 samples in estimated 5.0000 s (3.1B itermetadata_query/match_no_match
                        time:   [1.4957 ns 1.5222 ns 1.5475 ns]
                        thrpt:  [646.19 Melem/s 656.93 Melem/s 668.60 Melem/s]

Benchmarking metadata_store_basic/create: Collecting 100 samples in estimated 5.0053 s (2.5M iteratmetadata_store_basic/create
                        time:   [2.0266 µs 2.0295 µs 2.0326 µs]
                        thrpt:  [491.99 Kelem/s 492.73 Kelem/s 493.43 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimated 5.0089 s (2.5M itmetadata_store_basic/upsert_new
                        time:   [1.7712 µs 1.7931 µs 1.8151 µs]
                        thrpt:  [550.93 Kelem/s 557.71 Kelem/s 564.60 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in estimated 5.0024 s (5.metadata_store_basic/upsert_existing
                        time:   [993.29 ns 995.08 ns 996.99 ns]
                        thrpt:  [1.0030 Melem/s 1.0049 Melem/s 1.0068 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_basic/get: Collecting 100 samples in estimated 5.0001 s (213M iterationmetadata_store_basic/get
                        time:   [23.381 ns 23.448 ns 23.506 ns]
                        thrpt:  [42.543 Melem/s 42.647 Melem/s 42.769 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/get_miss: Collecting 100 samples in estimated 5.0001 s (213M itermetadata_store_basic/get_miss
                        time:   [23.569 ns 23.662 ns 23.791 ns]
                        thrpt:  [42.033 Melem/s 42.263 Melem/s 42.429 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_basic/len: Collecting 100 samples in estimated 5.0023 s (5.2M iterationmetadata_store_basic/len
                        time:   [942.87 ns 948.07 ns 952.90 ns]
                        thrpt:  [1.0494 Melem/s 1.0548 Melem/s 1.0606 Melem/s]
Found 29 outliers among 100 measurements (29.00%)
  17 (17.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 16.7s, or reduce sample count to 20.
Benchmarking metadata_store_basic/stats: Collecting 100 samples in estimated 16.732 s (100 iteratiometadata_store_basic/stats
                        time:   [166.53 ms 167.22 ms 167.91 ms]
                        thrpt:  [5.9555  elem/s 5.9800  elem/s 6.0048  elem/s]

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in estimated 5.7127 s (20metadata_store_query/query_by_status
                        time:   [281.22 µs 282.46 µs 283.66 µs]
                        thrpt:  [3.5253 Kelem/s 3.5403 Kelem/s 3.5559 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples in estimated 5.5359 s metadata_store_query/query_by_continent
                        time:   [137.04 µs 137.45 µs 137.92 µs]
                        thrpt:  [7.2504 Kelem/s 7.2751 Kelem/s 7.2969 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) high mild
  11 (11.00%) high severe
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in estimated 5.9063 s (15k metadata_store_query/query_by_tier
                        time:   [389.07 µs 389.75 µs 390.42 µs]
                        thrpt:  [2.5613 Kelem/s 2.5658 Kelem/s 2.5702 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samples in estimated 7.2492 metadata_store_query/query_accepting_work
                        time:   [476.88 µs 477.67 µs 478.54 µs]
                        thrpt:  [2.0897 Kelem/s 2.0935 Kelem/s 2.0970 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples in estimated 7.1283 s (1metadata_store_query/query_with_limit
                        time:   [471.25 µs 472.04 µs 472.87 µs]
                        thrpt:  [2.1147 Kelem/s 2.1185 Kelem/s 2.1220 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in estimated 6.0374 s (20k metadata_store_query/query_complex
                        time:   [298.50 µs 299.04 µs 299.64 µs]
                        thrpt:  [3.3373 Kelem/s 3.3440 Kelem/s 3.3500 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samples in estimated 5.1649 smetadata_store_spatial/find_nearby_100km
                        time:   [340.90 µs 341.46 µs 342.04 µs]
                        thrpt:  [2.9237 Kelem/s 2.9286 Kelem/s 2.9334 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samples in estimated 5.5033 metadata_store_spatial/find_nearby_1000km
                        time:   [362.41 µs 362.94 µs 363.44 µs]
                        thrpt:  [2.7515 Kelem/s 2.7553 Kelem/s 2.7593 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samples in estimated 6.8395 metadata_store_spatial/find_nearby_5000km
                        time:   [455.39 µs 456.10 µs 456.81 µs]
                        thrpt:  [2.1891 Kelem/s 2.1925 Kelem/s 2.1959 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 samples in estimated 6.21metadata_store_spatial/find_best_for_routing
                        time:   [305.80 µs 307.35 µs 308.87 µs]
                        thrpt:  [3.2376 Kelem/s 3.2536 Kelem/s 3.2701 Kelem/s]
Found 27 outliers among 100 measurements (27.00%)
  13 (13.00%) low severe
  6 (6.00%) low mild
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in estimated 7.4230 s (15k metadata_store_spatial/find_relays
                        time:   [492.55 µs 494.23 µs 495.89 µs]
                        thrpt:  [2.0166 Kelem/s 2.0234 Kelem/s 2.0302 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samples in estimated 5.0562 smetadata_store_scaling/query_status/1000
                        time:   [21.578 µs 21.619 µs 21.660 µs]
                        thrpt:  [46.169 Kelem/s 46.256 Kelem/s 46.344 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0714 metadata_store_scaling/query_complex/1000
                        time:   [20.784 µs 20.813 µs 20.841 µs]
                        thrpt:  [47.983 Kelem/s 48.048 Kelem/s 48.115 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples in estimated 5.1949 s metadata_store_scaling/find_nearby/1000
                        time:   [46.995 µs 47.046 µs 47.099 µs]
                        thrpt:  [21.232 Kelem/s 21.256 Kelem/s 21.279 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samples in estimated 5.0511 smetadata_store_scaling/query_status/5000
                        time:   [109.12 µs 109.91 µs 110.73 µs]
                        thrpt:  [9.0306 Kelem/s 9.0980 Kelem/s 9.1640 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samples in estimated 5.1538 metadata_store_scaling/query_complex/5000
                        time:   [112.13 µs 112.83 µs 113.49 µs]
                        thrpt:  [8.8113 Kelem/s 8.8625 Kelem/s 8.9181 Kelem/s]
Found 22 outliers among 100 measurements (22.00%)
  10 (10.00%) low severe
  9 (9.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples in estimated 5.5212 s metadata_store_scaling/find_nearby/5000
                        time:   [218.92 µs 219.35 µs 219.76 µs]
                        thrpt:  [4.5503 Kelem/s 4.5590 Kelem/s 4.5679 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samples in estimated 5.7380 metadata_store_scaling/query_status/10000
                        time:   [281.90 µs 282.70 µs 283.57 µs]
                        thrpt:  [3.5265 Kelem/s 3.5373 Kelem/s 3.5474 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samples in estimated 6.0314metadata_store_scaling/query_complex/10000
                        time:   [296.21 µs 297.11 µs 297.97 µs]
                        thrpt:  [3.3561 Kelem/s 3.3658 Kelem/s 3.3759 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samples in estimated 7.0825 smetadata_store_scaling/find_nearby/10000
                        time:   [464.74 µs 465.64 µs 466.44 µs]
                        thrpt:  [2.1439 Kelem/s 2.1476 Kelem/s 2.1518 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samples in estimated 5.0845 metadata_store_scaling/query_status/50000
                        time:   [2.0640 ms 2.0895 ms 2.1165 ms]
                        thrpt:  [472.48  elem/s 478.58  elem/s 484.51  elem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samples in estimated 5.0600metadata_store_scaling/query_complex/50000
                        time:   [2.1960 ms 2.2282 ms 2.2625 ms]
                        thrpt:  [441.99  elem/s 448.80  elem/s 455.37  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samples in estimated 5.1052 smetadata_store_scaling/find_nearby/50000
                        time:   [2.3946 ms 2.4115 ms 2.4290 ms]
                        thrpt:  [411.68  elem/s 414.67  elem/s 417.60  elem/s]

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 samples in estimated 5.12metadata_store_concurrent/concurrent_upsert/4
                        time:   [1.2117 ms 1.2188 ms 1.2246 ms]
                        thrpt:  [1.6332 Melem/s 1.6409 Melem/s 1.6506 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 samples in estimated 9.309metadata_store_concurrent/concurrent_query/4
                        time:   [232.11 ms 233.02 ms 233.97 ms]
                        thrpt:  [8.5480 Kelem/s 8.5830 Kelem/s 8.6165 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 8.332metadata_store_concurrent/concurrent_mixed/4
                        time:   [208.65 ms 211.17 ms 215.44 ms]
                        thrpt:  [9.2834 Kelem/s 9.4708 Kelem/s 9.5856 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low mild
  2 (10.00%) high mild
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 samples in estimated 5.33metadata_store_concurrent/concurrent_upsert/8
                        time:   [1.7294 ms 1.7865 ms 1.8369 ms]
                        thrpt:  [2.1776 Melem/s 2.2390 Melem/s 2.3130 Melem/s]
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.4s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 samples in estimated 5.427metadata_store_concurrent/concurrent_query/8
                        time:   [267.26 ms 272.29 ms 277.13 ms]
                        thrpt:  [14.434 Kelem/s 14.690 Kelem/s 14.967 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.0s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 5.046metadata_store_concurrent/concurrent_mixed/8
                        time:   [249.17 ms 254.59 ms 260.18 ms]
                        thrpt:  [15.374 Kelem/s 15.712 Kelem/s 16.053 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 samples in estimated 5.5metadata_store_concurrent/concurrent_upsert/16
                        time:   [3.2668 ms 3.2820 ms 3.2924 ms]
                        thrpt:  [2.4298 Melem/s 2.4375 Melem/s 2.4489 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low severe
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.4s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 samples in estimated 8.37metadata_store_concurrent/concurrent_query/16
                        time:   [414.04 ms 418.90 ms 423.74 ms]
                        thrpt:  [18.880 Kelem/s 19.097 Kelem/s 19.322 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.2s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 9.24metadata_store_concurrent/concurrent_mixed/16
                        time:   [458.32 ms 462.91 ms 467.38 ms]
                        thrpt:  [17.117 Kelem/s 17.282 Kelem/s 17.455 Kelem/s]

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimatemetadata_store_versioning/update_versioned_success
                        time:   [219.15 ns 219.97 ns 220.64 ns]
                        thrpt:  [4.5322 Melem/s 4.5460 Melem/s 4.5631 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) low severe
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimatmetadata_store_versioning/update_versioned_conflict
                        time:   [218.06 ns 218.63 ns 219.17 ns]
                        thrpt:  [4.5626 Melem/s 4.5739 Melem/s 4.5859 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  9 (9.00%) high mild
  1 (1.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estimated 5.0000 s (2.1B schema_validation/validate_string
                        time:   [2.7385 ns 3.0065 ns 3.3317 ns]
                        thrpt:  [300.14 Melem/s 332.62 Melem/s 365.17 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  18 (18.00%) high severe
Benchmarking schema_validation/validate_integer: Collecting 100 samples in estimated 5.0000 s (1.9Bschema_validation/validate_integer
                        time:   [2.9321 ns 3.1834 ns 3.4905 ns]
                        thrpt:  [286.50 Melem/s 314.13 Melem/s 341.06 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  17 (17.00%) high severe
Benchmarking schema_validation/validate_object: Collecting 100 samples in estimated 5.0001 s (82M ischema_validation/validate_object
                        time:   [60.289 ns 60.538 ns 60.778 ns]
                        thrpt:  [16.453 Melem/s 16.519 Melem/s 16.587 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in estimated 5.0000 s (148schema_validation/validate_array_10
                        time:   [33.162 ns 33.439 ns 33.694 ns]
                        thrpt:  [29.679 Melem/s 29.905 Melem/s 30.155 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
Benchmarking schema_validation/validate_complex: Collecting 100 samples in estimated 5.0005 s (33M schema_validation/validate_complex
                        time:   [151.90 ns 152.83 ns 153.69 ns]
                        thrpt:  [6.5065 Melem/s 6.5434 Melem/s 6.5834 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  15 (15.00%) low mild
  2 (2.00%) high mild

Benchmarking endpoint_matching/match_success: Collecting 100 samples in estimated 5.0002 s (23M iteendpoint_matching/match_success
                        time:   [213.25 ns 214.56 ns 215.81 ns]
                        thrpt:  [4.6336 Melem/s 4.6607 Melem/s 4.6893 Melem/s]
Found 23 outliers among 100 measurements (23.00%)
  10 (10.00%) low severe
  9 (9.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking endpoint_matching/match_failure: Collecting 100 samples in estimated 5.0009 s (24M iteendpoint_matching/match_failure
                        time:   [211.16 ns 212.29 ns 213.35 ns]
                        thrpt:  [4.6871 Melem/s 4.7104 Melem/s 4.7357 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in estimated 5.0003 s (11Mendpoint_matching/match_multi_param
                        time:   [468.83 ns 471.86 ns 474.64 ns]
                        thrpt:  [2.1069 Melem/s 2.1193 Melem/s 2.1330 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Benchmarking api_version/is_compatible_with: Collecting 100 samples in estimated 5.0000 s (25B iterapi_version/is_compatible_with
                        time:   [197.28 ps 198.41 ps 199.43 ps]
                        thrpt:  [5.0144 Gelem/s 5.0400 Gelem/s 5.0690 Gelem/s]
api_version/parse       time:   [39.007 ns 39.241 ns 39.471 ns]
                        thrpt:  [25.335 Melem/s 25.483 Melem/s 25.636 Melem/s]
api_version/to_string   time:   [51.673 ns 52.013 ns 52.353 ns]
                        thrpt:  [19.101 Melem/s 19.226 Melem/s 19.353 Melem/s]

api_schema/create       time:   [3.9813 µs 3.9923 µs 4.0028 µs]
                        thrpt:  [249.83 Kelem/s 250.48 Kelem/s 251.17 Kelem/s]
Found 21 outliers among 100 measurements (21.00%)
  15 (15.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
api_schema/serialize    time:   [2.2282 µs 2.2371 µs 2.2450 µs]
                        thrpt:  [445.43 Kelem/s 447.01 Kelem/s 448.78 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
api_schema/deserialize  time:   [9.4141 µs 9.4539 µs 9.4913 µs]
                        thrpt:  [105.36 Kelem/s 105.78 Kelem/s 106.22 Kelem/s]
Found 19 outliers among 100 measurements (19.00%)
  9 (9.00%) low severe
  7 (7.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking api_schema/find_endpoint: Collecting 100 samples in estimated 5.0010 s (22M iterationsapi_schema/find_endpoint
                        time:   [225.28 ns 226.75 ns 228.12 ns]
                        thrpt:  [4.3837 Melem/s 4.4102 Melem/s 4.4390 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) low severe
  11 (11.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking api_schema/endpoints_by_tag: Collecting 100 samples in estimated 5.0003 s (50M iteratiapi_schema/endpoints_by_tag
                        time:   [98.978 ns 99.205 ns 99.409 ns]
                        thrpt:  [10.059 Melem/s 10.080 Melem/s 10.103 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking request_validation/validate_full_request: Collecting 100 samples in estimated 5.0001 srequest_validation/validate_full_request
                        time:   [56.461 ns 56.608 ns 56.765 ns]
                        thrpt:  [17.616 Melem/s 17.665 Melem/s 17.711 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking request_validation/validate_path_only: Collecting 100 samples in estimated 5.0000 s (3request_validation/validate_path_only
                        time:   [15.188 ns 15.336 ns 15.474 ns]
                        thrpt:  [64.624 Melem/s 65.206 Melem/s 65.842 Melem/s]

Benchmarking api_registry_basic/create: Collecting 100 samples in estimated 5.0017 s (4.0M iteratioapi_registry_basic/create
                        time:   [1.2414 µs 1.2429 µs 1.2445 µs]
                        thrpt:  [803.56 Kelem/s 804.57 Kelem/s 805.52 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimated 5.0289 s (722k itapi_registry_basic/register_new
                        time:   [3.7566 µs 3.7955 µs 3.8369 µs]
                        thrpt:  [260.63 Kelem/s 263.47 Kelem/s 266.20 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
api_registry_basic/get  time:   [23.468 ns 23.521 ns 23.568 ns]
                        thrpt:  [42.430 Melem/s 42.516 Melem/s 42.612 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
api_registry_basic/len  time:   [955.77 ns 958.09 ns 960.19 ns]
                        thrpt:  [1.0415 Melem/s 1.0437 Melem/s 1.0463 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  3 (3.00%) low severe
  4 (4.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.3s, or reduce sample count to 70.
Benchmarking api_registry_basic/stats: Collecting 100 samples in estimated 6.3276 s (100 iterationsapi_registry_basic/stats
                        time:   [64.034 ms 64.378 ms 64.727 ms]
                        thrpt:  [15.449  elem/s 15.533  elem/s 15.617  elem/s]

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estimated 5.1303 s (61k itapi_registry_query/query_by_name
                        time:   [83.896 µs 84.282 µs 84.656 µs]
                        thrpt:  [11.812 Kelem/s 11.865 Kelem/s 11.920 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_query/query_by_tag: Collecting 100 samples in estimated 7.2852 s (10k iteapi_registry_query/query_by_tag
                        time:   [711.17 µs 715.85 µs 720.31 µs]
                        thrpt:  [1.3883 Kelem/s 1.3969 Kelem/s 1.4061 Kelem/s]
Benchmarking api_registry_query/query_with_version: Collecting 100 samples in estimated 5.0034 s (1api_registry_query/query_with_version
                        time:   [38.874 µs 39.149 µs 39.429 µs]
                        thrpt:  [25.362 Kelem/s 25.544 Kelem/s 25.724 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in estimated 5.1263 s (140api_registry_query/find_by_endpoint
                        time:   [3.5116 ms 3.5522 ms 3.5961 ms]
                        thrpt:  [278.08  elem/s 281.52  elem/s 284.77  elem/s]
Found 15 outliers among 100 measurements (15.00%)
  8 (8.00%) high mild
  7 (7.00%) high severe
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in estimated 5.1368 s (86k api_registry_query/find_compatible
                        time:   [58.850 µs 59.213 µs 59.557 µs]
                        thrpt:  [16.791 Kelem/s 16.888 Kelem/s 16.992 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples in estimated 5.0214 s api_registry_scaling/query_by_name/1000
                        time:   [7.3857 µs 7.4280 µs 7.4678 µs]
                        thrpt:  [133.91 Kelem/s 134.63 Kelem/s 135.40 Kelem/s]
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples in estimated 5.1514 s (api_registry_scaling/query_by_tag/1000
                        time:   [37.563 µs 37.754 µs 37.940 µs]
                        thrpt:  [26.357 Kelem/s 26.487 Kelem/s 26.622 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples in estimated 5.0351 s api_registry_scaling/query_by_name/5000
                        time:   [36.545 µs 36.775 µs 36.995 µs]
                        thrpt:  [27.030 Kelem/s 27.193 Kelem/s 27.364 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples in estimated 5.1191 s (api_registry_scaling/query_by_tag/5000
                        time:   [252.69 µs 254.20 µs 255.73 µs]
                        thrpt:  [3.9104 Kelem/s 3.9339 Kelem/s 3.9574 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samples in estimated 5.3704 sapi_registry_scaling/query_by_name/10000
                        time:   [81.006 µs 81.479 µs 81.945 µs]
                        thrpt:  [12.203 Kelem/s 12.273 Kelem/s 12.345 Kelem/s]
Found 24 outliers among 100 measurements (24.00%)
  16 (16.00%) low mild
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples in estimated 7.1063 s api_registry_scaling/query_by_tag/10000
                        time:   [698.31 µs 702.98 µs 707.45 µs]
                        thrpt:  [1.4135 Kelem/s 1.4225 Kelem/s 1.4320 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.8s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 samples in estimated 9.8061 api_registry_concurrent/concurrent_query/4
                        time:   [475.93 ms 480.29 ms 485.38 ms]
                        thrpt:  [4.1204 Kelem/s 4.1641 Kelem/s 4.2023 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) high mild
  1 (5.00%) high severe
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.3s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 8.2698 api_registry_concurrent/concurrent_mixed/4
                        time:   [410.71 ms 412.16 ms 413.60 ms]
                        thrpt:  [4.8355 Kelem/s 4.8525 Kelem/s 4.8696 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 samples in estimated 10.749 api_registry_concurrent/concurrent_query/8
                        time:   [523.46 ms 529.91 ms 536.25 ms]
                        thrpt:  [7.4593 Kelem/s 7.5484 Kelem/s 7.6414 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.3s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 9.3248 api_registry_concurrent/concurrent_mixed/8
                        time:   [465.53 ms 471.18 ms 477.66 ms]
                        thrpt:  [8.3741 Kelem/s 8.4893 Kelem/s 8.5923 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 14.1s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samples in estimated 14.052api_registry_concurrent/concurrent_query/16
                        time:   [700.33 ms 704.47 ms 708.92 ms]
                        thrpt:  [11.285 Kelem/s 11.356 Kelem/s 11.423 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 12.9s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 12.905api_registry_concurrent/concurrent_mixed/16
                        time:   [640.34 ms 643.82 ms 647.18 ms]
                        thrpt:  [12.361 Kelem/s 12.426 Kelem/s 12.493 Kelem/s]

compare_op/eq           time:   [1.3794 ns 1.3870 ns 1.3939 ns]
                        thrpt:  [717.40 Melem/s 721.00 Melem/s 724.93 Melem/s]
compare_op/gt           time:   [898.30 ps 904.25 ps 909.78 ps]
                        thrpt:  [1.0992 Gelem/s 1.1059 Gelem/s 1.1132 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) low mild
  1 (1.00%) high mild
Benchmarking compare_op/contains_string: Collecting 100 samples in estimated 5.0001 s (262M iteraticompare_op/contains_string
                        time:   [18.779 ns 18.926 ns 19.077 ns]
                        thrpt:  [52.420 Melem/s 52.839 Melem/s 53.251 Melem/s]
compare_op/in_array     time:   [2.9633 ns 2.9845 ns 3.0067 ns]
                        thrpt:  [332.59 Melem/s 335.06 Melem/s 337.46 Melem/s]

condition/simple        time:   [44.456 ns 44.698 ns 44.916 ns]
                        thrpt:  [22.264 Melem/s 22.373 Melem/s 22.494 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
condition/nested_field  time:   [571.37 ns 575.01 ns 578.56 ns]
                        thrpt:  [1.7284 Melem/s 1.7391 Melem/s 1.7502 Melem/s]
condition/string_eq     time:   [71.046 ns 71.509 ns 71.967 ns]
                        thrpt:  [13.895 Melem/s 13.984 Melem/s 14.075 Melem/s]

condition_expr/single   time:   [45.341 ns 45.672 ns 45.998 ns]
                        thrpt:  [21.740 Melem/s 21.895 Melem/s 22.055 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
condition_expr/and_2    time:   [98.682 ns 99.257 ns 99.823 ns]
                        thrpt:  [10.018 Melem/s 10.075 Melem/s 10.134 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  17 (17.00%) low mild
  2 (2.00%) high mild
condition_expr/and_5    time:   [298.11 ns 299.91 ns 301.69 ns]
                        thrpt:  [3.3146 Melem/s 3.3343 Melem/s 3.3544 Melem/s]
condition_expr/or_3     time:   [171.11 ns 172.01 ns 172.88 ns]
                        thrpt:  [5.7842 Melem/s 5.8137 Melem/s 5.8443 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  12 (12.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
condition_expr/nested   time:   [129.21 ns 129.98 ns 130.74 ns]
                        thrpt:  [7.6489 Melem/s 7.6935 Melem/s 7.7390 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

rule/create             time:   [381.37 ns 383.58 ns 385.72 ns]
                        thrpt:  [2.5926 Melem/s 2.6070 Melem/s 2.6221 Melem/s]
Found 26 outliers among 100 measurements (26.00%)
  19 (19.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
rule/matches            time:   [98.821 ns 99.302 ns 99.738 ns]
                        thrpt:  [10.026 Melem/s 10.070 Melem/s 10.119 Melem/s]

rule_context/create     time:   [1.7748 µs 1.7810 µs 1.7867 µs]
                        thrpt:  [559.68 Kelem/s 561.48 Kelem/s 563.45 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  14 (14.00%) low mild
  2 (2.00%) high mild
Benchmarking rule_context/get_simple: Collecting 100 samples in estimated 5.0000 s (117M iterationsrule_context/get_simple time:   [42.396 ns 42.667 ns 42.933 ns]
                        thrpt:  [23.292 Melem/s 23.437 Melem/s 23.587 Melem/s]
Benchmarking rule_context/get_nested: Collecting 100 samples in estimated 5.0012 s (8.7M iterationsrule_context/get_nested time:   [568.91 ns 572.60 ns 576.00 ns]
                        thrpt:  [1.7361 Melem/s 1.7464 Melem/s 1.7577 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_context/get_deep_nested: Collecting 100 samples in estimated 5.0022 s (8.7M iterarule_context/get_deep_nested
                        time:   [565.33 ns 568.91 ns 572.46 ns]
                        thrpt:  [1.7469 Melem/s 1.7578 Melem/s 1.7689 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  7 (7.00%) low severe
  13 (13.00%) low mild
  1 (1.00%) high mild

Benchmarking rule_engine_basic/create: Collecting 100 samples in estimated 5.0000 s (431M iterationrule_engine_basic/create
                        time:   [11.785 ns 11.845 ns 11.911 ns]
                        thrpt:  [83.955 Melem/s 84.422 Melem/s 84.854 Melem/s]
Benchmarking rule_engine_basic/add_rule: Collecting 100 samples in estimated 5.0522 s (76k iteratiorule_engine_basic/add_rule
                        time:   [1.6860 µs 1.7446 µs 1.7931 µs]
                        thrpt:  [557.70 Kelem/s 573.20 Kelem/s 593.12 Kelem/s]
Benchmarking rule_engine_basic/get_rule: Collecting 100 samples in estimated 5.0001 s (308M iteratirule_engine_basic/get_rule
                        time:   [16.227 ns 16.242 ns 16.258 ns]
                        thrpt:  [61.508 Melem/s 61.569 Melem/s 61.625 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  13 (13.00%) high severe
Benchmarking rule_engine_basic/rules_by_tag: Collecting 100 samples in estimated 5.0005 s (2.9M iterule_engine_basic/rules_by_tag
                        time:   [1.7127 µs 1.7153 µs 1.7181 µs]
                        thrpt:  [582.02 Kelem/s 582.99 Kelem/s 583.87 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking rule_engine_basic/stats: Collecting 100 samples in estimated 5.0373 s (651k iterationsrule_engine_basic/stats time:   [7.6849 µs 7.7052 µs 7.7235 µs]
                        thrpt:  [129.47 Kelem/s 129.78 Kelem/s 130.12 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples in estimated 5.0018 s (rule_engine_evaluate/evaluate_10_rules
                        time:   [3.7025 µs 3.7080 µs 3.7137 µs]
                        thrpt:  [269.27 Kelem/s 269.69 Kelem/s 270.09 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samples in estimated 5.00rule_engine_evaluate/evaluate_first_10_rules
                        time:   [325.40 ns 326.06 ns 326.67 ns]
                        thrpt:  [3.0612 Melem/s 3.0669 Melem/s 3.0731 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples in estimated 5.0401 s rule_engine_evaluate/evaluate_100_rules
                        time:   [32.132 µs 32.190 µs 32.247 µs]
                        thrpt:  [31.011 Kelem/s 31.065 Kelem/s 31.122 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 samples in estimated 5.0rule_engine_evaluate/evaluate_first_100_rules
                        time:   [324.89 ns 325.62 ns 326.25 ns]
                        thrpt:  [3.0651 Melem/s 3.0711 Melem/s 3.0780 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated rule_engine_evaluate/evaluate_matching_100_rules
                        time:   [40.652 µs 40.861 µs 41.064 µs]
                        thrpt:  [24.352 Kelem/s 24.473 Kelem/s 24.599 Kelem/s]
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samples in estimated 5.9389 srule_engine_evaluate/evaluate_1000_rules
                        time:   [291.48 µs 293.16 µs 294.87 µs]
                        thrpt:  [3.3914 Kelem/s 3.4112 Kelem/s 3.4307 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 samples in estimated 5.rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [308.76 ns 310.43 ns 311.95 ns]
                        thrpt:  [3.2056 Melem/s 3.2213 Melem/s 3.2387 Melem/s]

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimated 5.0038 s (1.4M itrule_engine_scaling/evaluate/10
                        time:   [3.6605 µs 3.6777 µs 3.6940 µs]
                        thrpt:  [270.71 Kelem/s 271.91 Kelem/s 273.19 Kelem/s]
Benchmarking rule_engine_scaling/evaluate_first/10: Collecting 100 samples in estimated 5.0006 s (1rule_engine_scaling/evaluate_first/10
                        time:   [312.96 ns 314.50 ns 315.93 ns]
                        thrpt:  [3.1652 Melem/s 3.1797 Melem/s 3.1953 Melem/s]
Benchmarking rule_engine_scaling/evaluate/50: Collecting 100 samples in estimated 5.0339 s (273k itrule_engine_scaling/evaluate/50
                        time:   [18.351 µs 18.449 µs 18.547 µs]
                        thrpt:  [53.917 Kelem/s 54.203 Kelem/s 54.494 Kelem/s]
Benchmarking rule_engine_scaling/evaluate_first/50: Collecting 100 samples in estimated 5.0004 s (1rule_engine_scaling/evaluate_first/50
                        time:   [320.81 ns 321.43 ns 322.01 ns]
                        thrpt:  [3.1055 Melem/s 3.1111 Melem/s 3.1172 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate/100: Collecting 100 samples in estimated 5.1160 s (157k irule_engine_scaling/evaluate/100
                        time:   [32.637 µs 32.691 µs 32.743 µs]
                        thrpt:  [30.541 Kelem/s 30.589 Kelem/s 30.640 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/100: Collecting 100 samples in estimated 5.0011 s (rule_engine_scaling/evaluate_first/100
                        time:   [328.10 ns 328.63 ns 329.18 ns]
                        thrpt:  [3.0378 Melem/s 3.0430 Melem/s 3.0479 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/500: Collecting 100 samples in estimated 5.3926 s (35k itrule_engine_scaling/evaluate/500
                        time:   [152.57 µs 152.75 µs 152.94 µs]
                        thrpt:  [6.5383 Kelem/s 6.5465 Kelem/s 6.5542 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/500: Collecting 100 samples in estimated 5.0011 s (rule_engine_scaling/evaluate_first/500
                        time:   [317.73 ns 318.28 ns 318.80 ns]
                        thrpt:  [3.1368 Melem/s 3.1419 Melem/s 3.1473 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/1000: Collecting 100 samples in estimated 5.9800 s (20k irule_engine_scaling/evaluate/1000
                        time:   [290.87 µs 292.39 µs 293.94 µs]
                        thrpt:  [3.4020 Kelem/s 3.4201 Kelem/s 3.4380 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_engine_scaling/evaluate_first/1000: Collecting 100 samples in estimated 5.0011 s rule_engine_scaling/evaluate_first/1000
                        time:   [316.21 ns 317.98 ns 319.68 ns]
                        thrpt:  [3.1281 Melem/s 3.1449 Melem/s 3.1624 Melem/s]

rule_set/create         time:   [6.9881 µs 6.9961 µs 7.0055 µs]
                        thrpt:  [142.74 Kelem/s 142.94 Kelem/s 143.10 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) low severe
  1 (1.00%) high mild
  9 (9.00%) high severe
Benchmarking rule_set/load_into_engine: Collecting 100 samples in estimated 5.0025 s (460k iteratiorule_set/load_into_engine
                        time:   [10.831 µs 10.856 µs 10.879 µs]
                        thrpt:  [91.917 Kelem/s 92.115 Kelem/s 92.329 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe

trace_id/generate       time:   [46.404 ns 46.683 ns 46.942 ns]
                        thrpt:  [21.303 Melem/s 21.421 Melem/s 21.550 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  3 (3.00%) low severe
  11 (11.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
trace_id/to_hex         time:   [122.74 ns 123.53 ns 124.30 ns]
                        thrpt:  [8.0453 Melem/s 8.0952 Melem/s 8.1473 Melem/s]
Found 25 outliers among 100 measurements (25.00%)
  17 (17.00%) low severe
  4 (4.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
trace_id/from_hex       time:   [18.156 ns 18.198 ns 18.236 ns]
                        thrpt:  [54.837 Melem/s 54.951 Melem/s 55.079 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  11 (11.00%) high mild
  2 (2.00%) high severe

Benchmarking context_operations/create: Collecting 100 samples in estimated 5.0000 s (62M iterationcontext_operations/create
                        time:   [80.774 ns 81.011 ns 81.274 ns]
                        thrpt:  [12.304 Melem/s 12.344 Melem/s 12.380 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking context_operations/child: Collecting 100 samples in estimated 5.0001 s (139M iterationcontext_operations/child
                        time:   [36.532 ns 36.667 ns 36.812 ns]
                        thrpt:  [27.165 Melem/s 27.273 Melem/s 27.374 Melem/s]
Benchmarking context_operations/for_remote: Collecting 100 samples in estimated 5.0001 s (138M itercontext_operations/for_remote
                        time:   [36.713 ns 36.968 ns 37.247 ns]
                        thrpt:  [26.848 Melem/s 27.050 Melem/s 27.239 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
Benchmarking context_operations/to_traceparent: Collecting 100 samples in estimated 5.0014 s (15M icontext_operations/to_traceparent
                        time:   [325.57 ns 326.28 ns 327.12 ns]
                        thrpt:  [3.0570 Melem/s 3.0649 Melem/s 3.0715 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking context_operations/from_traceparent: Collecting 100 samples in estimated 5.0003 s (44Mcontext_operations/from_traceparent
                        time:   [113.68 ns 113.97 ns 114.32 ns]
                        thrpt:  [8.7476 Melem/s 8.7739 Melem/s 8.7965 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

baggage/create          time:   [4.5919 ns 4.6217 ns 4.6504 ns]
                        thrpt:  [215.04 Melem/s 216.37 Melem/s 217.78 Melem/s]
baggage/get             time:   [7.8309 ns 7.8838 ns 7.9349 ns]
                        thrpt:  [126.03 Melem/s 126.84 Melem/s 127.70 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
baggage/set             time:   [59.948 ns 60.263 ns 60.553 ns]
                        thrpt:  [16.514 Melem/s 16.594 Melem/s 16.681 Melem/s]
baggage/merge           time:   [1.7258 µs 1.7319 µs 1.7379 µs]
                        thrpt:  [575.40 Kelem/s 577.39 Kelem/s 579.43 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

span/create             time:   [81.911 ns 82.455 ns 82.989 ns]
                        thrpt:  [12.050 Melem/s 12.128 Melem/s 12.208 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  13 (13.00%) low mild
  1 (1.00%) high severe
span/set_attribute      time:   [56.499 ns 56.887 ns 57.276 ns]
                        thrpt:  [17.459 Melem/s 17.579 Melem/s 17.699 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  14 (14.00%) low mild
span/add_event          time:   [96.233 ns 125.06 ns 183.52 ns]
                        thrpt:  [5.4490 Melem/s 7.9959 Melem/s 10.391 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
span/with_kind          time:   [82.201 ns 82.678 ns 83.135 ns]
                        thrpt:  [12.029 Melem/s 12.095 Melem/s 12.165 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  12 (12.00%) low mild

Benchmarking context_store/create_context: Collecting 100 samples in estimated 5.2744 s (66k iteratcontext_store/create_context
                        time:   [95.413 µs 96.213 µs 97.002 µs]
                        thrpt:  [10.309 Kelem/s 10.394 Kelem/s 10.481 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild
