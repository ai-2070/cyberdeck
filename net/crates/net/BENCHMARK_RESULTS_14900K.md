     Running benches\auth_guard.rs (target\release\deps\auth_guard-ce5e3162b4989bfa.exe)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples iauth_guard_check_fast_hit/single_thread
                        time:   [24.660 ns 24.682 ns 24.704 ns]
                        thrpt:  [40.479 Melem/s 40.515 Melem/s 40.551 Melem/s]
                 change:
                        time:   [−4.3522% −3.6862% −3.0672%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1642% +3.8273% +4.5503%]
                        Performance has improved.

Benchmarking auth_guard_check_fast_miss/single_thread: Warming up for 3.0000 Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples auth_guard_check_fast_miss/single_thread
                        time:   [10.276 ns 10.289 ns 10.300 ns]
                        thrpt:  [97.087 Melem/s 97.195 Melem/s 97.312 Melem/s]
                 change:
                        time:   [−2.6040% −2.2120% −1.8831%] (p = 0.00 < 0.05)
                        thrpt:  [+1.9193% +2.2620% +2.6736%]
                        Performance has improved.
Found 2 outliers among 50 measurements (4.00%)
  2 (4.00%) high mild

Benchmarking auth_guard_check_fast_contended/eight_threads: Warming up for 3.Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samauth_guard_check_fast_contended/eight_threads
                        time:   [25.767 ns 26.023 ns 26.353 ns]
                        thrpt:  [37.946 Melem/s 38.428 Melem/s 38.809 Melem/s]
                 change:
                        time:   [−6.4236% −4.3506% −2.3426%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3988% +4.5485% +6.8645%]
                        Performance has improved.
Found 5 outliers among 50 measurements (10.00%)
  5 (10.00%) high severe

Benchmarking auth_guard_allow_channel/insert: Collecting 50 samples in estimaauth_guard_allow_channel/insert
                        time:   [176.07 ns 182.32 ns 187.63 ns]
                        thrpt:  [5.3297 Melem/s 5.4850 Melem/s 5.6795 Melem/s]
                 change:
                        time:   [−12.167% −6.9756% −1.6855%] (p = 0.01 < 0.05)
                        thrpt:  [+1.7144% +7.4987% +13.853%]
                        Performance has improved.

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples inauth_guard_hot_hit_ceiling/million_ops
                        time:   [9.7560 ms 9.7661 ms 9.7769 ms]
                        change: [−2.0727% −1.8619% −1.6485%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

     Running benches\cortex.rs (target\release\deps\cortex-7d646f8f356d44f2.exe)
Gnuplot not found, using plotters backend
Benchmarking cortex_ingest/tasks_create: Collecting 100 samples in estimated cortex_ingest/tasks_create
                        time:   [465.33 ns 513.52 ns 577.19 ns]
                        thrpt:  [1.7325 Melem/s 1.9473 Melem/s 2.1490 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_ingest/memories_store: Collecting 100 samples in estimatecortex_ingest/memories_store
                        time:   [804.28 ns 904.01 ns 1.0166 µs]
                        thrpt:  [983.65 Kelem/s 1.1062 Melem/s 1.2434 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  16 (16.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Warming up for 3.0000Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samplecortex_fold_barrier/tasks_create_and_wait
                        time:   [1.7864 µs 1.8425 µs 1.9113 µs]
                        thrpt:  [523.20 Kelem/s 542.74 Kelem/s 559.78 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Warming up for 3.00Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 sampcortex_fold_barrier/memories_store_and_wait
                        time:   [2.5979 µs 3.2935 µs 4.1402 µs]
                        thrpt:  [241.53 Kelem/s 303.63 Kelem/s 384.93 Kelem/s]
Found 23 outliers among 100 measurements (23.00%)
  6 (6.00%) high mild
  17 (17.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in esticortex_query/tasks_find_many/100
                        time:   [2.0774 µs 2.0798 µs 2.0823 µs]
                        thrpt:  [48.023 Melem/s 48.082 Melem/s 48.136 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in escortex_query/tasks_count_where/100
                        time:   [169.41 ns 170.38 ns 171.44 ns]
                        thrpt:  [583.29 Melem/s 586.91 Melem/s 590.28 Melem/s]
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in escortex_query/tasks_find_unique/100
                        time:   [6.7693 ns 6.7758 ns 6.7831 ns]
                        thrpt:  [14.743 Gelem/s 14.758 Gelem/s 14.773 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples cortex_query/memories_find_many_tag/100
                        time:   [12.866 µs 12.883 µs 12.900 µs]
                        thrpt:  [7.7517 Melem/s 7.7624 Melem/s 7.7722 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples incortex_query/memories_count_where/100
                        time:   [577.29 ns 578.61 ns 580.00 ns]
                        thrpt:  [172.41 Melem/s 172.83 Melem/s 173.22 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estcortex_query/tasks_find_many/1000
                        time:   [18.908 µs 18.943 µs 18.978 µs]
                        thrpt:  [52.692 Melem/s 52.791 Melem/s 52.888 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in ecortex_query/tasks_count_where/1000
                        time:   [1.6235 µs 1.6264 µs 1.6296 µs]
                        thrpt:  [613.65 Melem/s 614.87 Melem/s 615.94 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in ecortex_query/tasks_find_unique/1000
                        time:   [6.7510 ns 6.7668 ns 6.7823 ns]
                        thrpt:  [147.44 Gelem/s 147.78 Gelem/s 148.13 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/1000: Warming up for 3.0000 Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samplescortex_query/memories_find_many_tag/1000
                        time:   [98.035 µs 98.148 µs 98.270 µs]
                        thrpt:  [10.176 Melem/s 10.189 Melem/s 10.200 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples icortex_query/memories_count_where/1000
                        time:   [5.8386 µs 5.8572 µs 5.8745 µs]
                        thrpt:  [170.23 Melem/s 170.73 Melem/s 171.27 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in escortex_query/tasks_find_many/10000
                        time:   [158.44 µs 158.63 µs 158.84 µs]
                        thrpt:  [62.957 Melem/s 63.039 Melem/s 63.115 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in cortex_query/tasks_count_where/10000
                        time:   [30.380 µs 30.534 µs 30.727 µs]
                        thrpt:  [325.45 Melem/s 327.51 Melem/s 329.16 Melem/s]
                 change:
                        time:   [−14.396% −13.090% −11.686%] (p = 0.00 < 0.05)
                        thrpt:  [+13.232% +15.062% +16.817%]
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in cortex_query/tasks_find_unique/10000
                        time:   [7.1719 ns 7.1795 ns 7.1875 ns]
                        thrpt:  [1391.3 Gelem/s 1392.9 Gelem/s 1394.3 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Warming up for 3.0000Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samplecortex_query/memories_find_many_tag/10000
                        time:   [859.99 µs 861.28 µs 862.68 µs]
                        thrpt:  [11.592 Melem/s 11.611 Melem/s 11.628 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples cortex_query/memories_count_where/10000
                        time:   [111.67 µs 111.85 µs 112.03 µs]
                        thrpt:  [89.265 Melem/s 89.402 Melem/s 89.550 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in esticortex_snapshot/tasks_encode/100
                        time:   [3.0889 µs 3.0928 µs 3.0969 µs]
                        thrpt:  [32.291 Melem/s 32.333 Melem/s 32.374 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in ecortex_snapshot/memories_encode/100
                        time:   [5.3478 µs 5.3528 µs 5.3584 µs]
                        thrpt:  [18.662 Melem/s 18.682 Melem/s 18.699 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Warming up fBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 1cortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.0352 µs 2.0421 µs 2.0505 µs]
                        thrpt:  [48.770 Melem/s 48.969 Melem/s 49.135 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples cortex_snapshot/netdb_bundle_decode/100
                        time:   [2.4673 µs 2.4698 µs 2.4724 µs]
                        thrpt:  [40.446 Melem/s 40.488 Melem/s 40.530 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  8 (8.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estcortex_snapshot/tasks_encode/1000
                        time:   [33.635 µs 33.656 µs 33.681 µs]
                        thrpt:  [29.691 Melem/s 29.712 Melem/s 29.731 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in cortex_snapshot/memories_encode/1000
                        time:   [53.225 µs 53.272 µs 53.323 µs]
                        thrpt:  [18.754 Melem/s 18.772 Melem/s 18.788 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Warming upBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collectingcortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [25.393 µs 25.421 µs 25.452 µs]
                        thrpt:  [39.289 Melem/s 39.337 Melem/s 39.381 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  4 (4.00%) high mild
  10 (10.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Warming up for 3.0000 Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samplescortex_snapshot/netdb_bundle_decode/1000
                        time:   [32.280 µs 32.305 µs 32.333 µs]
                        thrpt:  [30.928 Melem/s 30.955 Melem/s 30.979 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  9 (9.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in escortex_snapshot/tasks_encode/10000
                        time:   [334.82 µs 335.16 µs 335.51 µs]
                        thrpt:  [29.805 Melem/s 29.837 Melem/s 29.867 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples incortex_snapshot/memories_encode/10000
                        time:   [592.27 µs 593.56 µs 594.98 µs]
                        thrpt:  [16.807 Melem/s 16.847 Melem/s 16.884 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Warming Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: CollectiBenchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Analyzincortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [255.42 µs 256.00 µs 256.76 µs]
                        thrpt:  [38.947 Melem/s 39.063 Melem/s 39.151 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Warming up for 3.0000Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samplecortex_snapshot/netdb_bundle_decode/10000
                        time:   [352.35 µs 356.05 µs 360.06 µs]
                        thrpt:  [27.773 Melem/s 28.086 Melem/s 28.381 Melem/s]

     Running benches\ingestion.rs (target\release\deps\ingestion-c39fdde612e9c88c.exe)
Gnuplot not found, using plotters backend
Benchmarking ring_buffer/push/1024: Collecting 100 samples in estimated 5.000ring_buffer/push/1024   time:   [1.2075 ns 1.2089 ns 1.2106 ns]
                        thrpt:  [826.03 Melem/s 827.23 Melem/s 828.17 Melem/s]
                 change:
                        time:   [−5.1518% −4.2280% −3.4433%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5661% +4.4146% +5.4316%]
                        Performance has improved.
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low severe
  5 (5.00%) high mild
  8 (8.00%) high severe
Benchmarking ring_buffer/push_pop/1024: Collecting 100 samples in estimated 5ring_buffer/push_pop/1024
                        time:   [1.0101 ns 1.0130 ns 1.0164 ns]
                        thrpt:  [983.85 Melem/s 987.13 Melem/s 989.96 Melem/s]
                 change:
                        time:   [−4.9132% −4.0581% −3.2865%] (p = 0.00 < 0.05)
                        thrpt:  [+3.3981% +4.2298% +5.1671%]
                        Performance has improved.
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe
Benchmarking ring_buffer/push/8192: Collecting 100 samples in estimated 5.000ring_buffer/push/8192   time:   [1.2274 ns 1.2291 ns 1.2310 ns]
                        thrpt:  [812.35 Melem/s 813.61 Melem/s 814.71 Melem/s]
                 change:
                        time:   [−4.2050% −3.6005% −3.0597%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1563% +3.7350% +4.3896%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking ring_buffer/push_pop/8192: Collecting 100 samples in estimated 5ring_buffer/push_pop/8192
                        time:   [1.0125 ns 1.0141 ns 1.0161 ns]
                        thrpt:  [984.12 Melem/s 986.12 Melem/s 987.65 Melem/s]
                 change:
                        time:   [−50.076% −48.011% −45.544%] (p = 0.00 < 0.05)
                        thrpt:  [+83.634% +92.347% +100.31%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking ring_buffer/push/65536: Collecting 100 samples in estimated 5.00ring_buffer/push/65536  time:   [1.2231 ns 1.2263 ns 1.2289 ns]
                        thrpt:  [813.71 Melem/s 815.44 Melem/s 817.61 Melem/s]
                 change:
                        time:   [−54.052% −53.912% −53.758%] (p = 0.00 < 0.05)
                        thrpt:  [+116.25% +116.97% +117.64%]
                        Performance has improved.
Found 19 outliers among 100 measurements (19.00%)
  5 (5.00%) low severe
  5 (5.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking ring_buffer/push_pop/65536: Collecting 100 samples in estimated ring_buffer/push_pop/65536
                        time:   [997.73 ps 1.0018 ns 1.0058 ns]
                        thrpt:  [994.27 Melem/s 998.20 Melem/s 1.0023 Gelem/s]
                 change:
                        time:   [−53.653% −53.414% −53.149%] (p = 0.00 < 0.05)
                        thrpt:  [+113.44% +114.66% +115.76%]
                        Performance has improved.
Found 20 outliers among 100 measurements (20.00%)
  4 (4.00%) low severe
  12 (12.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking ring_buffer/push/1048576: Collecting 100 samples in estimated 5.ring_buffer/push/1048576
                        time:   [1.2088 ns 1.2142 ns 1.2193 ns]
                        thrpt:  [820.12 Melem/s 823.58 Melem/s 827.25 Melem/s]
                 change:
                        time:   [−53.283% −52.627% −51.884%] (p = 0.00 < 0.05)
                        thrpt:  [+107.83% +111.09% +114.06%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking ring_buffer/push_pop/1048576: Collecting 100 samples in estimatering_buffer/push_pop/1048576
                        time:   [1.0060 ns 1.0115 ns 1.0174 ns]
                        thrpt:  [982.88 Melem/s 988.59 Melem/s 994.08 Melem/s]
                 change:
                        time:   [−53.715% −52.950% −51.939%] (p = 0.00 < 0.05)
                        thrpt:  [+108.07% +112.54% +116.05%]
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

Benchmarking timestamp/next: Collecting 100 samples in estimated 5.0001 s (36timestamp/next          time:   [13.331 ns 13.392 ns 13.456 ns]
                        thrpt:  [74.316 Melem/s 74.671 Melem/s 75.015 Melem/s]
                 change:
                        time:   [−42.124% −40.276% −38.157%] (p = 0.00 < 0.05)
                        thrpt:  [+61.699% +67.436% +72.782%]
                        Performance has improved.
Benchmarking timestamp/now_raw: Collecting 100 samples in estimated 5.0000 s timestamp/now_raw       time:   [5.8451 ns 5.8736 ns 5.9035 ns]
                        thrpt:  [169.39 Melem/s 170.25 Melem/s 171.08 Melem/s]
                 change:
                        time:   [−6.2776% −5.5962% −5.0155%] (p = 0.00 < 0.05)
                        thrpt:  [+5.2803% +5.9280% +6.6981%]
                        Performance has improved.

Benchmarking event/internal_event_new: Collecting 100 samples in estimated 5.event/internal_event_new
                        time:   [209.50 ns 210.27 ns 210.96 ns]
                        thrpt:  [4.7402 Melem/s 4.7558 Melem/s 4.7732 Melem/s]
                 change:
                        time:   [−9.1084% −8.0125% −7.0348%] (p = 0.00 < 0.05)
                        thrpt:  [+7.5671% +8.7104% +10.021%]
                        Performance has improved.
Found 20 outliers among 100 measurements (20.00%)
  8 (8.00%) low severe
  10 (10.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking event/json_creation: Collecting 100 samples in estimated 5.0006 event/json_creation     time:   [130.47 ns 130.63 ns 130.80 ns]
                        thrpt:  [7.6455 Melem/s 7.6553 Melem/s 7.6645 Melem/s]
                 change:
                        time:   [−6.7965% −5.8476% −5.0248%] (p = 0.00 < 0.05)
                        thrpt:  [+5.2906% +6.2108% +7.2922%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

Benchmarking batch/pop_batch/100: Collecting 100 samples in estimated 5.0007 batch/pop_batch/100     time:   [882.31 ns 882.67 ns 883.06 ns]
                        thrpt:  [113.24 Melem/s 113.29 Melem/s 113.34 Melem/s]
                 change:
                        time:   [+489.58% +493.39% +496.98%] (p = 0.00 < 0.05)
                        thrpt:  [−83.249% −83.148% −83.039%]
                        Performance has regressed.
Benchmarking batch/pop_batch/1000: Collecting 100 samples in estimated 5.0023batch/pop_batch/1000    time:   [1.1416 µs 1.1433 µs 1.1452 µs]
                        thrpt:  [873.17 Melem/s 874.67 Melem/s 875.93 Melem/s]
                 change:
                        time:   [−9.8139% −9.2419% −8.7405%] (p = 0.00 < 0.05)
                        thrpt:  [+9.5777% +10.183% +10.882%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking batch/pop_batch/10000: Collecting 100 samples in estimated 5.032batch/pop_batch/10000   time:   [12.415 µs 12.429 µs 12.445 µs]
                        thrpt:  [803.54 Melem/s 804.57 Melem/s 805.48 Melem/s]
                 change:
                        time:   [−6.7489% −5.1477% −3.7321%] (p = 0.00 < 0.05)
                        thrpt:  [+3.8767% +5.4271% +7.2374%]
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

     Running benches\mesh.rs (target\release\deps\mesh-aeaef1ea2452e6b9.exe)
Gnuplot not found, using plotters backend
Benchmarking mesh_reroute/triangle_failure: Collecting 100 samples in estimatmesh_reroute/triangle_failure
                        time:   [23.477 µs 23.735 µs 24.008 µs]
                        thrpt:  [41.653 Kelem/s 42.132 Kelem/s 42.595 Kelem/s]
                 change:
                        time:   [−2.3155% −0.8788% +0.5972%] (p = 0.23 > 0.05)
                        thrpt:  [−0.5937% +0.8866% +2.3704%]
                        No change in performance detected.
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimmesh_reroute/10_peers_10_routes
                        time:   [124.03 µs 125.15 µs 126.42 µs]
                        thrpt:  [7.9104 Kelem/s 7.9901 Kelem/s 8.0623 Kelem/s]
                 change:
                        time:   [−11.005% −9.9705% −8.9778%] (p = 0.00 < 0.05)
                        thrpt:  [+9.8633% +11.075% +12.365%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_reroute/50_peers_100_routes: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.0s, enable flat sampling, or reduce sample count to 60.
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimesh_reroute/50_peers_100_routes
                        time:   [1.1889 ms 1.1929 ms 1.1978 ms]
                        thrpt:  [834.87  elem/s 838.26  elem/s 841.10  elem/s]
                 change:
                        time:   [−10.953% −10.102% −9.3684%] (p = 0.00 < 0.05)
                        thrpt:  [+10.337% +11.237% +12.301%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

Benchmarking mesh_proximity/on_pingwave_new: Collecting 100 samples in estimamesh_proximity/on_pingwave_new
                        time:   [164.95 ns 170.91 ns 176.68 ns]
                        thrpt:  [5.6598 Melem/s 5.8509 Melem/s 6.0624 Melem/s]
                 change:
                        time:   [−11.100% −6.7749% −2.2796%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3328% +7.2673% +12.486%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimesh_proximity/on_pingwave_dedup
                        time:   [50.507 ns 50.558 ns 50.610 ns]
                        thrpt:  [19.759 Melem/s 19.779 Melem/s 19.799 Melem/s]
                 change:
                        time:   [−2.1059% −1.6158% −1.1365%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1496% +1.6423% +2.1512%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estmesh_proximity/pingwave_serialize
                        time:   [1.1183 ns 1.1559 ns 1.1974 ns]
                        thrpt:  [835.13 Melem/s 865.14 Melem/s 894.22 Melem/s]
                 change:
                        time:   [−7.1334% −4.0910% −1.2620%] (p = 0.01 < 0.05)
                        thrpt:  [+1.2781% +4.2655% +7.6814%]
                        Performance has improved.
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in emesh_proximity/pingwave_deserialize
                        time:   [1.1466 ns 1.1901 ns 1.2372 ns]
                        thrpt:  [808.28 Melem/s 840.28 Melem/s 872.15 Melem/s]
                 change:
                        time:   [−10.100% −6.6816% −3.5435%] (p = 0.00 < 0.05)
                        thrpt:  [+3.6737% +7.1600% +11.234%]
                        Performance has improved.
Benchmarking mesh_proximity/node_count: Collecting 100 samples in estimated 5mesh_proximity/node_count
                        time:   [958.25 ns 959.27 ns 960.36 ns]
                        thrpt:  [1.0413 Melem/s 1.0425 Melem/s 1.0436 Melem/s]
                 change:
                        time:   [−4.5863% −3.9913% −3.4178%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5388% +4.1573% +4.8068%]
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 123.0s, or reduce sample count to 10.
Benchmarking mesh_proximity/all_nodes_100: Collecting 100 samples in estimatemesh_proximity/all_nodes_100
                        time:   [1.2570 s 1.2605 s 1.2641 s]
                        thrpt:  [0.7911  elem/s 0.7933  elem/s 0.7955  elem/s]
                 change:
                        time:   [+65.258% +69.631% +74.181%] (p = 0.00 < 0.05)
                        thrpt:  [−42.588% −41.049% −39.489%]
                        Performance has regressed.

Benchmarking mesh_dispatch/classify_direct: Collecting 100 samples in estimatmesh_dispatch/classify_direct
                        time:   [402.03 ps 402.50 ps 403.07 ps]
                        thrpt:  [2.4810 Gelem/s 2.4845 Gelem/s 2.4873 Gelem/s]
                 change:
                        time:   [−44.031% −41.185% −38.783%] (p = 0.00 < 0.05)
                        thrpt:  [+63.354% +70.024% +78.672%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_dispatch/classify_routed: Collecting 100 samples in estimatmesh_dispatch/classify_routed
                        time:   [301.49 ps 301.78 ps 302.10 ps]
                        thrpt:  [3.3102 Gelem/s 3.3137 Gelem/s 3.3169 Gelem/s]
                 change:
                        time:   [−41.745% −41.587% −41.429%] (p = 0.00 < 0.05)
                        thrpt:  [+70.733% +71.196% +71.659%]
                        Performance has improved.
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  5 (5.00%) high mild
  10 (10.00%) high severe
Benchmarking mesh_dispatch/classify_pingwave: Collecting 100 samples in estimmesh_dispatch/classify_pingwave
                        time:   [200.89 ps 201.05 ps 201.23 ps]
                        thrpt:  [4.9695 Gelem/s 4.9739 Gelem/s 4.9779 Gelem/s]
                 change:
                        time:   [−37.853% −37.724% −37.596%] (p = 0.00 < 0.05)
                        thrpt:  [+60.245% +60.575% +60.908%]
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) high mild
  8 (8.00%) high severe

Benchmarking mesh_routing/lookup_hit: Collecting 100 samples in estimated 5.0mesh_routing/lookup_hit time:   [17.754 ns 17.785 ns 17.818 ns]
                        thrpt:  [56.124 Melem/s 56.228 Melem/s 56.327 Melem/s]
                 change:
                        time:   [−34.533% −33.871% −33.217%] (p = 0.00 < 0.05)
                        thrpt:  [+49.739% +51.220% +52.749%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking mesh_routing/lookup_miss: Collecting 100 samples in estimated 5.mesh_routing/lookup_miss
                        time:   [17.770 ns 17.796 ns 17.825 ns]
                        thrpt:  [56.102 Melem/s 56.192 Melem/s 56.275 Melem/s]
                 change:
                        time:   [−34.426% −33.761% −33.114%] (p = 0.00 < 0.05)
                        thrpt:  [+49.508% +50.968% +52.499%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking mesh_routing/is_local: Collecting 100 samples in estimated 5.000mesh_routing/is_local   time:   [200.98 ps 201.12 ps 201.28 ps]
                        thrpt:  [4.9682 Gelem/s 4.9722 Gelem/s 4.9757 Gelem/s]
                 change:
                        time:   [−61.462% −61.220% −61.029%] (p = 0.00 < 0.05)
                        thrpt:  [+156.60% +157.87% +159.49%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking mesh_routing/all_routes/10: Collecting 100 samples in estimated mesh_routing/all_routes/10
                        time:   [6.4066 µs 6.4181 µs 6.4307 µs]
                        thrpt:  [155.50 Kelem/s 155.81 Kelem/s 156.09 Kelem/s]
                 change:
                        time:   [−34.198% −34.010% −33.834%] (p = 0.00 < 0.05)
                        thrpt:  [+51.135% +51.538% +51.971%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking mesh_routing/all_routes/100: Collecting 100 samples in estimatedmesh_routing/all_routes/100
                        time:   [7.3570 µs 7.3734 µs 7.3919 µs]
                        thrpt:  [135.28 Kelem/s 135.62 Kelem/s 135.93 Kelem/s]
                 change:
                        time:   [−38.256% −38.001% −37.773%] (p = 0.00 < 0.05)
                        thrpt:  [+60.702% +61.293% +61.960%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_routing/all_routes/1000: Collecting 100 samples in estimatemesh_routing/all_routes/1000
                        time:   [23.031 µs 23.086 µs 23.153 µs]
                        thrpt:  [43.192 Kelem/s 43.315 Kelem/s 43.420 Kelem/s]
                 change:
                        time:   [−28.857% −28.499% −28.127%] (p = 0.00 < 0.05)
                        thrpt:  [+39.135% +39.858% +40.562%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Benchmarking mesh_routing/add_route: Collecting 100 samples in estimated 5.00mesh_routing/add_route  time:   [37.964 ns 38.010 ns 38.058 ns]
                        thrpt:  [26.275 Melem/s 26.309 Melem/s 26.341 Melem/s]
                 change:
                        time:   [−44.176% −44.045% −43.915%] (p = 0.00 < 0.05)
                        thrpt:  [+78.302% +78.715% +79.134%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

     Running benches\net.rs (target\release\deps\net-439462c3eb84b252.exe)
Gnuplot not found, using plotters backend
Benchmarking net_header/serialize: Collecting 100 samples in estimated 5.0000net_header/serialize    time:   [1.2742 ns 1.3110 ns 1.3433 ns]
                        thrpt:  [744.44 Melem/s 762.77 Melem/s 784.83 Melem/s]
                 change:
                        time:   [−5.5995% −2.4879% +0.8647%] (p = 0.14 > 0.05)
                        thrpt:  [−0.8573% +2.5514% +5.9316%]
                        No change in performance detected.
Benchmarking net_header/deserialize: Collecting 100 samples in estimated 5.00net_header/deserialize  time:   [1.2056 ns 1.2066 ns 1.2077 ns]
                        thrpt:  [828.04 Melem/s 828.80 Melem/s 829.44 Melem/s]
                 change:
                        time:   [−21.535% −21.305% −21.091%] (p = 0.00 < 0.05)
                        thrpt:  [+26.729% +27.073% +27.446%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) high mild
  10 (10.00%) high severe
Benchmarking net_header/roundtrip: Collecting 100 samples in estimated 5.0000net_header/roundtrip    time:   [1.2062 ns 1.2074 ns 1.2087 ns]
                        thrpt:  [827.35 Melem/s 828.24 Melem/s 829.03 Melem/s]
                 change:
                        time:   [−22.307% −21.670% −21.225%] (p = 0.00 < 0.05)
                        thrpt:  [+26.944% +27.665% +28.712%]
                        Performance has improved.
Found 17 outliers among 100 measurements (17.00%)
  5 (5.00%) high mild
  12 (12.00%) high severe

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimnet_event_frame/write_single/64
                        time:   [35.300 ns 35.381 ns 35.464 ns]
                        thrpt:  [1.6807 GiB/s 1.6847 GiB/s 1.6885 GiB/s]
                 change:
                        time:   [−41.240% −40.722% −40.254%] (p = 0.00 < 0.05)
                        thrpt:  [+67.376% +68.697% +70.184%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estinet_event_frame/write_single/256
                        time:   [35.299 ns 35.402 ns 35.526 ns]
                        thrpt:  [6.7111 GiB/s 6.7346 GiB/s 6.7543 GiB/s]
                 change:
                        time:   [−41.263% −40.749% −40.275%] (p = 0.00 < 0.05)
                        thrpt:  [+67.435% +68.774% +70.251%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estnet_event_frame/write_single/1024
                        time:   [35.397 ns 35.436 ns 35.479 ns]
                        thrpt:  [26.880 GiB/s 26.912 GiB/s 26.942 GiB/s]
                 change:
                        time:   [−48.193% −47.820% −47.484%] (p = 0.00 < 0.05)
                        thrpt:  [+90.417% +91.644% +93.026%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estnet_event_frame/write_single/4096
                        time:   [50.563 ns 50.736 ns 50.913 ns]
                        thrpt:  [74.925 GiB/s 75.187 GiB/s 75.445 GiB/s]
                 change:
                        time:   [−54.354% −54.023% −53.705%] (p = 0.00 < 0.05)
                        thrpt:  [+116.00% +117.50% +119.08%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking net_event_frame/write_batch/1: Collecting 100 samples in estimatnet_event_frame/write_batch/1
                        time:   [27.827 ns 27.900 ns 27.980 ns]
                        thrpt:  [2.1302 GiB/s 2.1364 GiB/s 2.1420 GiB/s]
                 change:
                        time:   [−51.780% −51.540% −51.310%] (p = 0.00 < 0.05)
                        thrpt:  [+105.38% +106.36% +107.38%]
                        Performance has improved.
Benchmarking net_event_frame/write_batch/10: Collecting 100 samples in estimanet_event_frame/write_batch/10
                        time:   [55.226 ns 55.337 ns 55.456 ns]
                        thrpt:  [10.748 GiB/s 10.771 GiB/s 10.793 GiB/s]
                 change:
                        time:   [−50.628% −50.254% −49.897%] (p = 0.00 < 0.05)
                        thrpt:  [+99.590% +101.02% +102.54%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking net_event_frame/write_batch/50: Collecting 100 samples in estimanet_event_frame/write_batch/50
                        time:   [145.49 ns 145.63 ns 145.80 ns]
                        thrpt:  [20.440 GiB/s 20.464 GiB/s 20.485 GiB/s]
                 change:
                        time:   [−59.573% −59.299% −59.038%] (p = 0.00 < 0.05)
                        thrpt:  [+144.13% +145.69% +147.36%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking net_event_frame/write_batch/100: Collecting 100 samples in estimnet_event_frame/write_batch/100
                        time:   [254.93 ns 255.28 ns 255.75 ns]
                        thrpt:  [23.306 GiB/s 23.348 GiB/s 23.381 GiB/s]
                 change:
                        time:   [−64.719% −63.561% −62.565%] (p = 0.00 < 0.05)
                        thrpt:  [+167.13% +174.43% +183.44%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking net_event_frame/read_batch_10: Collecting 100 samples in estimatnet_event_frame/read_batch_10
                        time:   [163.45 ns 163.76 ns 164.10 ns]
                        thrpt:  [60.937 Melem/s 61.064 Melem/s 61.182 Melem/s]
                 change:
                        time:   [−31.817% −31.592% −31.381%] (p = 0.00 < 0.05)
                        thrpt:  [+45.732% +46.181% +46.664%]
                        Performance has improved.

Benchmarking net_packet_pool/get_return/16: Collecting 100 samples in estimatnet_packet_pool/get_return/16
                        time:   [50.519 ns 50.811 ns 51.097 ns]
                        thrpt:  [19.571 Melem/s 19.681 Melem/s 19.795 Melem/s]
                 change:
                        time:   [−35.045% −34.739% −34.448%] (p = 0.00 < 0.05)
                        thrpt:  [+52.551% +53.232% +53.952%]
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  8 (8.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high severe
Benchmarking net_packet_pool/get_return/64: Collecting 100 samples in estimatnet_packet_pool/get_return/64
                        time:   [51.655 ns 51.882 ns 52.230 ns]
                        thrpt:  [19.146 Melem/s 19.274 Melem/s 19.359 Melem/s]
                 change:
                        time:   [−34.352% −34.010% −33.660%] (p = 0.00 < 0.05)
                        thrpt:  [+50.740% +51.539% +52.327%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking net_packet_pool/get_return/256: Collecting 100 samples in estimanet_packet_pool/get_return/256
                        time:   [52.890 ns 52.962 ns 53.034 ns]
                        thrpt:  [18.856 Melem/s 18.881 Melem/s 18.907 Melem/s]
                 change:
                        time:   [−33.399% −33.140% −32.890%] (p = 0.00 < 0.05)
                        thrpt:  [+49.008% +49.566% +50.147%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking net_packet_build/build_packet/1: Collecting 100 samples in estimnet_packet_build/build_packet/1
                        time:   [1.1335 µs 1.1343 µs 1.1351 µs]
                        thrpt:  [53.769 MiB/s 53.809 MiB/s 53.846 MiB/s]
                 change:
                        time:   [+1.2982% +1.5814% +1.8618%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8278% −1.5568% −1.2816%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estinet_packet_build/build_packet/10
                        time:   [1.5083 µs 1.5098 µs 1.5114 µs]
                        thrpt:  [403.83 MiB/s 404.27 MiB/s 404.67 MiB/s]
                 change:
                        time:   [−3.4344% −2.9621% −2.5080%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5725% +3.0525% +3.5566%]
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estinet_packet_build/build_packet/50
                        time:   [2.9353 µs 2.9381 µs 2.9411 µs]
                        thrpt:  [1.0133 GiB/s 1.0144 GiB/s 1.0153 GiB/s]
                 change:
                        time:   [−5.0729% −4.1531% −3.3903%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5093% +4.3331% +5.3440%]
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe

Benchmarking net_encryption/encrypt/64: Collecting 100 samples in estimated 5net_encryption/encrypt/64
                        time:   [1.1359 µs 1.1370 µs 1.1382 µs]
                        thrpt:  [53.626 MiB/s 53.682 MiB/s 53.732 MiB/s]
Found 23 outliers among 100 measurements (23.00%)
  9 (9.00%) low mild
  6 (6.00%) high mild
  8 (8.00%) high severe
Benchmarking net_encryption/encrypt/256: Collecting 100 samples in estimated net_encryption/encrypt/256
                        time:   [1.2009 µs 1.2018 µs 1.2027 µs]
                        thrpt:  [202.99 MiB/s 203.15 MiB/s 203.30 MiB/s]
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  12 (12.00%) high severe
Benchmarking net_encryption/encrypt/1024: Collecting 100 samples in estimatednet_encryption/encrypt/1024
                        time:   [1.5750 µs 1.5765 µs 1.5781 µs]
                        thrpt:  [618.83 MiB/s 619.45 MiB/s 620.03 MiB/s]
Found 20 outliers among 100 measurements (20.00%)
  2 (2.00%) low severe
  2 (2.00%) high mild
  16 (16.00%) high severe
Benchmarking net_encryption/encrypt/4096: Collecting 100 samples in estimatednet_encryption/encrypt/4096
                        time:   [3.1323 µs 3.1348 µs 3.1377 µs]
                        thrpt:  [1.2158 GiB/s 1.2169 GiB/s 1.2178 GiB/s]
Found 19 outliers among 100 measurements (19.00%)
  4 (4.00%) high mild
  15 (15.00%) high severe

Benchmarking net_keypair/generate: Collecting 100 samples in estimated 5.0093net_keypair/generate    time:   [10.837 µs 10.848 µs 10.859 µs]
                        thrpt:  [92.088 Kelem/s 92.183 Kelem/s 92.279 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe

Benchmarking net_aad/generate: Collecting 100 samples in estimated 5.0000 s (net_aad/generate        time:   [1.0555 ns 1.1023 ns 1.1448 ns]
                        thrpt:  [873.53 Melem/s 907.22 Melem/s 947.42 Melem/s]

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples ipool_comparison/shared_pool_get_return
                        time:   [52.810 ns 53.122 ns 53.490 ns]
                        thrpt:  [18.695 Melem/s 18.825 Melem/s 18.936 Melem/s]
Benchmarking pool_comparison/thread_local_pool_get_return: Warming up for 3.0Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 sampool_comparison/thread_local_pool_get_return
                        time:   [63.544 ns 64.089 ns 64.701 ns]
                        thrpt:  [15.456 Melem/s 15.603 Melem/s 15.737 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
Benchmarking pool_comparison/shared_pool_10x: Collecting 100 samples in estimpool_comparison/shared_pool_10x
                        time:   [502.52 ns 511.15 ns 520.78 ns]
                        thrpt:  [1.9202 Melem/s 1.9564 Melem/s 1.9900 Melem/s]
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples inpool_comparison/thread_local_pool_10x
                        time:   [806.25 ns 808.74 ns 811.44 ns]
                        thrpt:  [1.2324 Melem/s 1.2365 Melem/s 1.2403 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in esticipher_comparison/shared_pool/64
                        time:   [1.1337 µs 1.1347 µs 1.1357 µs]
                        thrpt:  [53.741 MiB/s 53.791 MiB/s 53.836 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in escipher_comparison/fast_chacha20/64
                        time:   [1.1344 µs 1.1352 µs 1.1362 µs]
                        thrpt:  [53.721 MiB/s 53.764 MiB/s 53.802 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estcipher_comparison/shared_pool/256
                        time:   [1.2019 µs 1.2033 µs 1.2049 µs]
                        thrpt:  [202.63 MiB/s 202.90 MiB/s 203.13 MiB/s]
Found 18 outliers among 100 measurements (18.00%)
  6 (6.00%) high mild
  12 (12.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in ecipher_comparison/fast_chacha20/256
                        time:   [1.2017 µs 1.2027 µs 1.2038 µs]
                        thrpt:  [202.81 MiB/s 203.00 MiB/s 203.17 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in escipher_comparison/shared_pool/1024
                        time:   [1.5751 µs 1.5763 µs 1.5776 µs]
                        thrpt:  [619.01 MiB/s 619.54 MiB/s 620.00 MiB/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in cipher_comparison/fast_chacha20/1024
                        time:   [1.5733 µs 1.5747 µs 1.5763 µs]
                        thrpt:  [619.54 MiB/s 620.16 MiB/s 620.70 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in escipher_comparison/shared_pool/4096
                        time:   [3.1426 µs 3.1452 µs 3.1478 µs]
                        thrpt:  [1.2118 GiB/s 1.2129 GiB/s 1.2138 GiB/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in cipher_comparison/fast_chacha20/4096
                        time:   [3.0885 µs 3.0927 µs 3.0987 µs]
                        thrpt:  [1.2311 GiB/s 1.2334 GiB/s 1.2351 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking adaptive_batcher/optimal_size: Collecting 100 samples in estimatadaptive_batcher/optimal_size
                        time:   [803.67 ps 804.54 ps 805.53 ps]
                        thrpt:  [1.2414 Gelem/s 1.2429 Gelem/s 1.2443 Gelem/s]
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe
Benchmarking adaptive_batcher/record: Collecting 100 samples in estimated 5.0adaptive_batcher/record time:   [9.4798 ns 9.4873 ns 9.4957 ns]
                        thrpt:  [105.31 Melem/s 105.40 Melem/s 105.49 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking adaptive_batcher/full_cycle: Collecting 100 samples in estimatedadaptive_batcher/full_cycle
                        time:   [8.0453 ns 8.0530 ns 8.0616 ns]
                        thrpt:  [124.04 Melem/s 124.18 Melem/s 124.30 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples ie2e_packet_build/shared_pool_50_events
                        time:   [2.9270 µs 2.9299 µs 2.9332 µs]
                        thrpt:  [1.0160 GiB/s 1.0172 GiB/s 1.0182 GiB/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estime2e_packet_build/fast_50_events
                        time:   [2.8717 µs 2.8743 µs 2.8771 µs]
                        thrpt:  [1.0359 GiB/s 1.0369 GiB/s 1.0378 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.2s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples imultithread_packet_build/shared_pool/8
                        time:   [1.6110 ms 1.6198 ms 1.6289 ms]
                        thrpt:  [4.9113 Melem/s 4.9390 Melem/s 4.9658 Melem/s]
Benchmarking multithread_packet_build/thread_local_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.3s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 sammultithread_packet_build/thread_local_pool/8
                        time:   [1.4295 ms 1.4381 ms 1.4470 ms]
                        thrpt:  [5.5285 Melem/s 5.5628 Melem/s 5.5962 Melem/s]
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples multithread_packet_build/shared_pool/16
                        time:   [2.5547 ms 2.5714 ms 2.5879 ms]
                        thrpt:  [6.1826 Melem/s 6.2222 Melem/s 6.2629 Melem/s]
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.5s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samultithread_packet_build/thread_local_pool/16
                        time:   [1.8473 ms 1.8659 ms 1.8851 ms]
                        thrpt:  [8.4876 Melem/s 8.5751 Melem/s 8.6614 Melem/s]
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples multithread_packet_build/shared_pool/24
                        time:   [3.8343 ms 3.8969 ms 3.9776 ms]
                        thrpt:  [6.0339 Melem/s 6.1588 Melem/s 6.2593 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Warming up for 3.Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samultithread_packet_build/thread_local_pool/24
                        time:   [2.3896 ms 2.4499 ms 2.5103 ms]
                        thrpt:  [9.5605 Melem/s 9.7963 Melem/s 10.043 Melem/s]
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples multithread_packet_build/shared_pool/32
                        time:   [5.1752 ms 5.3719 ms 5.5925 ms]
                        thrpt:  [5.7220 Melem/s 5.9569 Melem/s 6.1834 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) high mild
  16 (16.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Warming up for 3.Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samultithread_packet_build/thread_local_pool/32
                        time:   [3.0844 ms 3.0901 ms 3.0961 ms]
                        thrpt:  [10.336 Melem/s 10.356 Melem/s 10.375 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples multithread_mixed_frames/shared_mixed/8
                        time:   [1.0504 ms 1.0543 ms 1.0583 ms]
                        thrpt:  [11.339 Melem/s 11.382 Melem/s 11.424 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples inmultithread_mixed_frames/fast_mixed/8
                        time:   [947.38 µs 949.77 µs 952.25 µs]
                        thrpt:  [12.602 Melem/s 12.635 Melem/s 12.667 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.9s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samplesmultithread_mixed_frames/shared_mixed/16
                        time:   [1.5566 ms 1.5601 ms 1.5639 ms]
                        thrpt:  [15.347 Melem/s 15.384 Melem/s 15.419 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.8s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples imultithread_mixed_frames/fast_mixed/16
                        time:   [1.3487 ms 1.3516 ms 1.3550 ms]
                        thrpt:  [17.712 Melem/s 17.756 Melem/s 17.795 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) high mild
  9 (9.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/24: Warming up for 3.0000 Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samplesmultithread_mixed_frames/shared_mixed/24
                        time:   [2.3873 ms 2.4229 ms 2.4701 ms]
                        thrpt:  [14.574 Melem/s 14.858 Melem/s 15.080 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.9s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples imultithread_mixed_frames/fast_mixed/24
                        time:   [1.7399 ms 1.7637 ms 1.7878 ms]
                        thrpt:  [20.136 Melem/s 20.411 Melem/s 20.691 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/shared_mixed/32: Warming up for 3.0000 Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samplesmultithread_mixed_frames/shared_mixed/32
                        time:   [3.0568 ms 3.2392 ms 3.4406 ms]
                        thrpt:  [13.951 Melem/s 14.818 Melem/s 15.703 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) high mild
  18 (18.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples imultithread_mixed_frames/fast_mixed/32
                        time:   [2.1928 ms 2.2101 ms 2.2276 ms]
                        thrpt:  [21.548 Melem/s 21.718 Melem/s 21.890 Melem/s]

Benchmarking pool_contention/shared_acquire_release/8: Warming up for 3.0000 Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samplespool_contention/shared_acquire_release/8
                        time:   [9.6311 ms 9.6604 ms 9.6889 ms]
                        thrpt:  [8.2569 Melem/s 8.2813 Melem/s 8.3064 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.9s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples ipool_contention/fast_acquire_release/8
                        time:   [1.1184 ms 1.1538 ms 1.1905 ms]
                        thrpt:  [67.197 Melem/s 69.335 Melem/s 71.532 Melem/s]
Benchmarking pool_contention/shared_acquire_release/16: Warming up for 3.0000Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samplepool_contention/shared_acquire_release/16
                        time:   [20.247 ms 20.305 ms 20.363 ms]
                        thrpt:  [7.8574 Melem/s 7.8797 Melem/s 7.9023 Melem/s]
Benchmarking pool_contention/fast_acquire_release/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples pool_contention/fast_acquire_release/16
                        time:   [1.7914 ms 1.8001 ms 1.8083 ms]
                        thrpt:  [88.482 Melem/s 88.885 Melem/s 89.316 Melem/s]
Benchmarking pool_contention/shared_acquire_release/24: Warming up for 3.0000Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samplepool_contention/shared_acquire_release/24
                        time:   [35.690 ms 35.825 ms 35.972 ms]
                        thrpt:  [6.6719 Melem/s 6.6992 Melem/s 6.7247 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples pool_contention/fast_acquire_release/24
                        time:   [2.0892 ms 2.1117 ms 2.1347 ms]
                        thrpt:  [112.43 Melem/s 113.65 Melem/s 114.88 Melem/s]
Benchmarking pool_contention/shared_acquire_release/32: Warming up for 3.0000Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samplepool_contention/shared_acquire_release/32
                        time:   [46.391 ms 46.578 ms 46.777 ms]
                        thrpt:  [6.8410 Melem/s 6.8702 Melem/s 6.8978 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples pool_contention/fast_acquire_release/32
                        time:   [2.7798 ms 2.8109 ms 2.8423 ms]
                        thrpt:  [112.59 Melem/s 113.84 Melem/s 115.11 Melem/s]

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples inthroughput_scaling/fast_pool_scaling/1
                        time:   [3.6078 ms 3.6150 ms 3.6219 ms]
                        thrpt:  [552.19 Kelem/s 553.24 Kelem/s 554.36 Kelem/s]
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples inthroughput_scaling/fast_pool_scaling/2
                        time:   [3.6852 ms 3.6904 ms 3.6955 ms]
                        thrpt:  [1.0824 Melem/s 1.0839 Melem/s 1.0854 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples inthroughput_scaling/fast_pool_scaling/4
                        time:   [3.7202 ms 3.7270 ms 3.7346 ms]
                        thrpt:  [2.1421 Melem/s 2.1465 Melem/s 2.1504 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples inthroughput_scaling/fast_pool_scaling/8
                        time:   [4.2618 ms 4.4588 ms 4.6409 ms]
                        thrpt:  [3.4476 Melem/s 3.5884 Melem/s 3.7543 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples ithroughput_scaling/fast_pool_scaling/16
                        time:   [5.4574 ms 5.4653 ms 5.4749 ms]
                        thrpt:  [5.8448 Melem/s 5.8551 Melem/s 5.8636 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples ithroughput_scaling/fast_pool_scaling/24
                        time:   [6.8515 ms 7.3116 ms 7.7008 ms]
                        thrpt:  [6.2331 Melem/s 6.5649 Melem/s 7.0058 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples ithroughput_scaling/fast_pool_scaling/32
                        time:   [9.9807 ms 10.059 ms 10.145 ms]
                        thrpt:  [6.3088 Melem/s 6.3627 Melem/s 6.4124 Melem/s]

Benchmarking routing_header/serialize: Collecting 100 samples in estimated 5.routing_header/serialize
                        time:   [437.85 ps 466.02 ps 499.81 ps]
                        thrpt:  [2.0008 Gelem/s 2.1458 Gelem/s 2.2839 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) high mild
  18 (18.00%) high severe
Benchmarking routing_header/deserialize: Collecting 100 samples in estimated routing_header/deserialize
                        time:   [712.16 ps 721.06 ps 731.72 ps]
                        thrpt:  [1.3666 Gelem/s 1.3868 Gelem/s 1.4042 Gelem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) high mild
  19 (19.00%) high severe
Benchmarking routing_header/roundtrip: Collecting 100 samples in estimated 5.routing_header/roundtrip
                        time:   [713.55 ps 720.67 ps 729.17 ps]
                        thrpt:  [1.3714 Gelem/s 1.3876 Gelem/s 1.4014 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) high mild
  18 (18.00%) high severe
Benchmarking routing_header/forward: Collecting 100 samples in estimated 5.00routing_header/forward  time:   [247.30 ps 259.74 ps 272.23 ps]
                        thrpt:  [3.6734 Gelem/s 3.8500 Gelem/s 4.0436 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild

Benchmarking routing_table/lookup_hit: Collecting 100 samples in estimated 5.routing_table/lookup_hit
                        time:   [38.482 ns 38.556 ns 38.632 ns]
                        thrpt:  [25.885 Melem/s 25.936 Melem/s 25.986 Melem/s]
Benchmarking routing_table/lookup_miss: Collecting 100 samples in estimated 5routing_table/lookup_miss
                        time:   [17.790 ns 17.821 ns 17.851 ns]
                        thrpt:  [56.018 Melem/s 56.112 Melem/s 56.212 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking routing_table/is_local: Collecting 100 samples in estimated 5.00routing_table/is_local  time:   [205.25 ps 207.15 ps 209.15 ps]
                        thrpt:  [4.7812 Gelem/s 4.8274 Gelem/s 4.8722 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe
Benchmarking routing_table/add_route: Collecting 100 samples in estimated 5.0routing_table/add_route time:   [206.95 ns 210.56 ns 214.17 ns]
                        thrpt:  [4.6692 Melem/s 4.7493 Melem/s 4.8321 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) low mild
Benchmarking routing_table/record_in: Collecting 100 samples in estimated 5.0routing_table/record_in time:   [40.929 ns 40.967 ns 41.008 ns]
                        thrpt:  [24.385 Melem/s 24.410 Melem/s 24.433 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe
Benchmarking routing_table/record_out: Collecting 100 samples in estimated 5.routing_table/record_out
                        time:   [21.097 ns 21.115 ns 21.135 ns]
                        thrpt:  [47.314 Melem/s 47.359 Melem/s 47.399 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  12 (12.00%) high severe
Benchmarking routing_table/aggregate_stats: Collecting 100 samples in estimatrouting_table/aggregate_stats
                        time:   [8.0644 µs 8.0750 µs 8.0859 µs]
                        thrpt:  [123.67 Kelem/s 123.84 Kelem/s 124.00 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

Benchmarking fair_scheduler/creation: Collecting 100 samples in estimated 5.0fair_scheduler/creation time:   [1.5919 µs 1.5933 µs 1.5949 µs]
                        thrpt:  [626.98 Kelem/s 627.61 Kelem/s 628.18 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estfair_scheduler/stream_count_empty
                        time:   [959.27 ns 960.19 ns 961.21 ns]
                        thrpt:  [1.0404 Melem/s 1.0415 Melem/s 1.0425 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
Benchmarking fair_scheduler/total_queued: Collecting 100 samples in estimatedfair_scheduler/total_queued
                        time:   [200.54 ps 200.85 ps 201.13 ps]
                        thrpt:  [4.9719 Gelem/s 4.9788 Gelem/s 4.9864 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  8 (8.00%) low severe
  2 (2.00%) low mild
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking fair_scheduler/cleanup_empty: Collecting 100 samples in estimatefair_scheduler/cleanup_empty
                        time:   [1.2874 µs 1.2886 µs 1.2900 µs]
                        thrpt:  [775.19 Kelem/s 776.03 Kelem/s 776.78 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  7 (7.00%) high mild
  7 (7.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Warming up for 3.0Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samrouting_table_concurrent/concurrent_lookup/4
                        time:   [175.88 µs 176.99 µs 178.19 µs]
                        thrpt:  [22.448 Melem/s 22.600 Melem/s 22.743 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/4: Warming up for 3.00Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samprouting_table_concurrent/concurrent_stats/4
                        time:   [223.21 µs 224.60 µs 226.07 µs]
                        thrpt:  [17.694 Melem/s 17.809 Melem/s 17.920 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/8: Warming up for 3.0Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samrouting_table_concurrent/concurrent_lookup/8
                        time:   [276.25 µs 279.97 µs 283.88 µs]
                        thrpt:  [28.181 Melem/s 28.575 Melem/s 28.959 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/8: Warming up for 3.00Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samprouting_table_concurrent/concurrent_stats/8
                        time:   [324.21 µs 328.53 µs 333.10 µs]
                        thrpt:  [24.017 Melem/s 24.351 Melem/s 24.675 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking routing_table_concurrent/concurrent_lookup/16: Warming up for 3.Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 sarouting_table_concurrent/concurrent_lookup/16
                        time:   [497.35 µs 504.12 µs 511.94 µs]
                        thrpt:  [31.254 Melem/s 31.738 Melem/s 32.170 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/16: Warming up for 3.0Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samrouting_table_concurrent/concurrent_stats/16
                        time:   [557.96 µs 566.68 µs 575.89 µs]
                        thrpt:  [27.783 Melem/s 28.235 Melem/s 28.676 Melem/s]

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples inrouting_decision/parse_lookup_forward
                        time:   [38.695 ns 38.730 ns 38.769 ns]
                        thrpt:  [25.794 Melem/s 25.820 Melem/s 25.843 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estirouting_decision/full_with_stats
                        time:   [100.81 ns 100.90 ns 101.01 ns]
                        thrpt:  [9.9004 Melem/s 9.9105 Melem/s 9.9196 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in eststream_multiplexing/lookup_all/10
                        time:   [348.25 ns 348.55 ns 348.89 ns]
                        thrpt:  [28.663 Melem/s 28.690 Melem/s 28.715 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  9 (9.00%) high severe
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estistream_multiplexing/stats_all/10
                        time:   [409.68 ns 410.06 ns 410.48 ns]
                        thrpt:  [24.362 Melem/s 24.387 Melem/s 24.409 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in esstream_multiplexing/lookup_all/100
                        time:   [3.4415 µs 3.4441 µs 3.4469 µs]
                        thrpt:  [29.011 Melem/s 29.035 Melem/s 29.057 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  7 (7.00%) high mild
  7 (7.00%) high severe
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in eststream_multiplexing/stats_all/100
                        time:   [4.1171 µs 4.1210 µs 4.1252 µs]
                        thrpt:  [24.241 Melem/s 24.266 Melem/s 24.289 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estream_multiplexing/lookup_all/1000
                        time:   [36.097 µs 36.136 µs 36.179 µs]
                        thrpt:  [27.640 Melem/s 27.673 Melem/s 27.703 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in esstream_multiplexing/stats_all/1000
                        time:   [44.566 µs 44.654 µs 44.738 µs]
                        thrpt:  [22.353 Melem/s 22.395 Melem/s 22.439 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in stream_multiplexing/lookup_all/10000
                        time:   [391.57 µs 391.95 µs 392.35 µs]
                        thrpt:  [25.488 Melem/s 25.513 Melem/s 25.538 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estream_multiplexing/stats_all/10000
                        time:   [470.83 µs 471.39 µs 471.96 µs]
                        thrpt:  [21.188 Melem/s 21.214 Melem/s 21.239 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimultihop_packet_builder/build/64
                        time:   [41.009 ns 41.064 ns 41.126 ns]
                        thrpt:  [1.4493 GiB/s 1.4515 GiB/s 1.4534 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build_priority/64: Warming up for 3.0000Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samplemultihop_packet_builder/build_priority/64
                        time:   [30.008 ns 30.060 ns 30.117 ns]
                        thrpt:  [1.9791 GiB/s 1.9829 GiB/s 1.9863 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estmultihop_packet_builder/build/256
                        time:   [42.725 ns 42.797 ns 42.880 ns]
                        thrpt:  [5.5601 GiB/s 5.5709 GiB/s 5.5803 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build_priority/256: Warming up for 3.000Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samplmultihop_packet_builder/build_priority/256
                        time:   [31.826 ns 31.894 ns 31.969 ns]
                        thrpt:  [7.4578 GiB/s 7.4754 GiB/s 7.4913 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in esmultihop_packet_builder/build/1024
                        time:   [44.090 ns 44.150 ns 44.218 ns]
                        thrpt:  [21.567 GiB/s 21.601 GiB/s 21.630 GiB/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking multihop_packet_builder/build_priority/1024: Warming up for 3.00Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 sampmultihop_packet_builder/build_priority/1024
                        time:   [35.198 ns 35.236 ns 35.277 ns]
                        thrpt:  [27.034 GiB/s 27.065 GiB/s 27.095 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in esmultihop_packet_builder/build/4096
                        time:   [79.784 ns 80.072 ns 80.376 ns]
                        thrpt:  [47.460 GiB/s 47.641 GiB/s 47.813 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multihop_packet_builder/build_priority/4096: Warming up for 3.00Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 sampmultihop_packet_builder/build_priority/4096
                        time:   [69.620 ns 69.770 ns 69.929 ns]
                        thrpt:  [54.551 GiB/s 54.676 GiB/s 54.793 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking multihop_chain/forward_chain/1: Collecting 100 samples in estimamultihop_chain/forward_chain/1
                        time:   [53.254 ns 53.321 ns 53.390 ns]
                        thrpt:  [18.730 Melem/s 18.754 Melem/s 18.778 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking multihop_chain/forward_chain/2: Collecting 100 samples in estimamultihop_chain/forward_chain/2
                        time:   [87.527 ns 87.654 ns 87.781 ns]
                        thrpt:  [11.392 Melem/s 11.408 Melem/s 11.425 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking multihop_chain/forward_chain/3: Collecting 100 samples in estimamultihop_chain/forward_chain/3
                        time:   [121.62 ns 121.82 ns 122.04 ns]
                        thrpt:  [8.1938 Melem/s 8.2085 Melem/s 8.2221 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_chain/forward_chain/4: Collecting 100 samples in estimamultihop_chain/forward_chain/4
                        time:   [156.01 ns 156.27 ns 156.55 ns]
                        thrpt:  [6.3877 Melem/s 6.3991 Melem/s 6.4097 Melem/s]
Benchmarking multihop_chain/forward_chain/5: Collecting 100 samples in estimamultihop_chain/forward_chain/5
                        time:   [190.41 ns 190.77 ns 191.15 ns]
                        thrpt:  [5.2314 Melem/s 5.2419 Melem/s 5.2518 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking hop_latency/single_hop_process: Collecting 100 samples in estimahop_latency/single_hop_process
                        time:   [925.98 ps 936.23 ps 947.32 ps]
                        thrpt:  [1.0556 Gelem/s 1.0681 Gelem/s 1.0799 Gelem/s]
Benchmarking hop_latency/single_hop_full: Collecting 100 samples in estimatedhop_latency/single_hop_full
                        time:   [33.439 ns 33.514 ns 33.586 ns]
                        thrpt:  [29.774 Melem/s 29.839 Melem/s 29.905 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking hop_scaling/64B_1hops: Collecting 100 samples in estimated 5.000hop_scaling/64B_1hops   time:   [52.555 ns 52.617 ns 52.682 ns]
                        thrpt:  [1.1314 GiB/s 1.1328 GiB/s 1.1341 GiB/s]
Found 18 outliers among 100 measurements (18.00%)
  9 (9.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking hop_scaling/64B_2hops: Collecting 100 samples in estimated 5.000hop_scaling/64B_2hops   time:   [86.414 ns 86.528 ns 86.648 ns]
                        thrpt:  [704.41 MiB/s 705.38 MiB/s 706.31 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking hop_scaling/64B_3hops: Collecting 100 samples in estimated 5.000hop_scaling/64B_3hops   time:   [117.61 ns 117.78 ns 117.96 ns]
                        thrpt:  [517.41 MiB/s 518.23 MiB/s 518.96 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking hop_scaling/64B_4hops: Collecting 100 samples in estimated 5.000hop_scaling/64B_4hops   time:   [151.92 ns 152.32 ns 152.72 ns]
                        thrpt:  [399.64 MiB/s 400.70 MiB/s 401.77 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking hop_scaling/64B_5hops: Collecting 100 samples in estimated 5.000hop_scaling/64B_5hops   time:   [182.10 ns 182.40 ns 182.68 ns]
                        thrpt:  [334.11 MiB/s 334.63 MiB/s 335.18 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
Benchmarking hop_scaling/256B_1hops: Collecting 100 samples in estimated 5.00hop_scaling/256B_1hops  time:   [53.854 ns 53.912 ns 53.970 ns]
                        thrpt:  [4.4176 GiB/s 4.4224 GiB/s 4.4271 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking hop_scaling/256B_2hops: Collecting 100 samples in estimated 5.00hop_scaling/256B_2hops  time:   [87.722 ns 87.880 ns 88.045 ns]
                        thrpt:  [2.7079 GiB/s 2.7130 GiB/s 2.7179 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking hop_scaling/256B_3hops: Collecting 100 samples in estimated 5.00hop_scaling/256B_3hops  time:   [121.70 ns 122.01 ns 122.37 ns]
                        thrpt:  [1.9484 GiB/s 1.9541 GiB/s 1.9591 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking hop_scaling/256B_4hops: Collecting 100 samples in estimated 5.00hop_scaling/256B_4hops  time:   [155.81 ns 156.12 ns 156.44 ns]
                        thrpt:  [1.5240 GiB/s 1.5272 GiB/s 1.5302 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low severe
  2 (2.00%) high mild
Benchmarking hop_scaling/256B_5hops: Collecting 100 samples in estimated 5.00hop_scaling/256B_5hops  time:   [191.45 ns 191.78 ns 192.14 ns]
                        thrpt:  [1.2409 GiB/s 1.2432 GiB/s 1.2453 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking hop_scaling/1024B_1hops: Collecting 100 samples in estimated 5.0hop_scaling/1024B_1hops time:   [54.744 ns 54.806 ns 54.869 ns]
                        thrpt:  [17.381 GiB/s 17.401 GiB/s 17.421 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking hop_scaling/1024B_2hops: Collecting 100 samples in estimated 5.0hop_scaling/1024B_2hops time:   [89.793 ns 89.920 ns 90.058 ns]
                        thrpt:  [10.590 GiB/s 10.606 GiB/s 10.621 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking hop_scaling/1024B_3hops: Collecting 100 samples in estimated 5.0hop_scaling/1024B_3hops time:   [125.73 ns 125.91 ns 126.11 ns]
                        thrpt:  [7.5619 GiB/s 7.5740 GiB/s 7.5849 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking hop_scaling/1024B_4hops: Collecting 100 samples in estimated 5.0hop_scaling/1024B_4hops time:   [161.02 ns 161.38 ns 161.84 ns]
                        thrpt:  [5.8925 GiB/s 5.9096 GiB/s 5.9228 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking hop_scaling/1024B_5hops: Collecting 100 samples in estimated 5.0hop_scaling/1024B_5hops time:   [200.49 ns 200.82 ns 201.15 ns]
                        thrpt:  [4.7411 GiB/s 4.7490 GiB/s 4.7568 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking multihop_with_routing/route_and_forward/1: Warming up for 3.0000Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samplemultihop_with_routing/route_and_forward/1
                        time:   [153.94 ns 154.06 ns 154.19 ns]
                        thrpt:  [6.4853 Melem/s 6.4912 Melem/s 6.4962 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/2: Warming up for 3.0000Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samplemultihop_with_routing/route_and_forward/2
                        time:   [288.16 ns 288.89 ns 290.13 ns]
                        thrpt:  [3.4467 Melem/s 3.4615 Melem/s 3.4703 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/3: Warming up for 3.0000Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samplemultihop_with_routing/route_and_forward/3
                        time:   [421.79 ns 422.21 ns 422.65 ns]
                        thrpt:  [2.3660 Melem/s 2.3685 Melem/s 2.3709 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/4: Warming up for 3.0000Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samplemultihop_with_routing/route_and_forward/4
                        time:   [557.46 ns 558.40 ns 559.60 ns]
                        thrpt:  [1.7870 Melem/s 1.7908 Melem/s 1.7939 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/5: Warming up for 3.0000Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samplemultihop_with_routing/route_and_forward/5
                        time:   [692.34 ns 693.01 ns 693.71 ns]
                        thrpt:  [1.4415 Melem/s 1.4430 Melem/s 1.4444 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking multihop_concurrent/concurrent_forward/4: Warming up for 3.0000 Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples multihop_concurrent/concurrent_forward/4
                        time:   [560.16 µs 563.87 µs 568.06 µs]
                        thrpt:  [7.0415 Melem/s 7.0938 Melem/s 7.1408 Melem/s]
Benchmarking multihop_concurrent/concurrent_forward/8: Warming up for 3.0000 Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples multihop_concurrent/concurrent_forward/8
                        time:   [713.58 µs 750.29 µs 796.67 µs]
                        thrpt:  [10.042 Melem/s 10.663 Melem/s 11.211 Melem/s]
Benchmarking multihop_concurrent/concurrent_forward/16: Warming up for 3.0000Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samplesmultihop_concurrent/concurrent_forward/16
                        time:   [1.3447 ms 1.3628 ms 1.3794 ms]
                        thrpt:  [11.599 Melem/s 11.740 Melem/s 11.899 Melem/s]

Benchmarking pingwave/serialize: Collecting 100 samples in estimated 5.0000 spingwave/serialize      time:   [519.29 ps 530.02 ps 541.87 ps]
                        thrpt:  [1.8455 Gelem/s 1.8867 Gelem/s 1.9257 Gelem/s]
Benchmarking pingwave/deserialize: Collecting 100 samples in estimated 5.0000pingwave/deserialize    time:   [621.92 ps 636.07 ps 653.27 ps]
                        thrpt:  [1.5308 Gelem/s 1.5722 Gelem/s 1.6079 Gelem/s]
Found 20 outliers among 100 measurements (20.00%)
  2 (2.00%) high mild
  18 (18.00%) high severe
Benchmarking pingwave/roundtrip: Collecting 100 samples in estimated 5.0000 spingwave/roundtrip      time:   [621.47 ps 635.60 ps 652.80 ps]
                        thrpt:  [1.5319 Gelem/s 1.5733 Gelem/s 1.6091 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  19 (19.00%) high severe
Benchmarking pingwave/forward: Collecting 100 samples in estimated 5.0000 s (pingwave/forward        time:   [519.32 ps 529.88 ps 541.40 ps]
                        thrpt:  [1.8471 Gelem/s 1.8872 Gelem/s 1.9256 Gelem/s]

Benchmarking capabilities/serialize_simple: Collecting 100 samples in estimatcapabilities/serialize_simple
                        time:   [37.303 ns 37.370 ns 37.442 ns]
                        thrpt:  [26.708 Melem/s 26.759 Melem/s 26.807 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimcapabilities/deserialize_simple
                        time:   [9.0036 ns 9.0720 ns 9.1352 ns]
                        thrpt:  [109.47 Melem/s 110.23 Melem/s 111.07 Melem/s]
Benchmarking capabilities/serialize_complex: Collecting 100 samples in estimacapabilities/serialize_complex
                        time:   [37.991 ns 38.043 ns 38.099 ns]
                        thrpt:  [26.247 Melem/s 26.286 Melem/s 26.322 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in esticapabilities/deserialize_complex
                        time:   [233.56 ns 233.99 ns 234.46 ns]
                        thrpt:  [4.2651 Melem/s 4.2736 Melem/s 4.2816 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking local_graph/create_pingwave: Collecting 100 samples in estimatedlocal_graph/create_pingwave
                        time:   [5.0255 ns 5.0295 ns 5.0341 ns]
                        thrpt:  [198.65 Melem/s 198.83 Melem/s 198.99 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking local_graph/on_pingwave_new: Collecting 100 samples in estimatedlocal_graph/on_pingwave_new
                        time:   [176.99 ns 179.72 ns 182.54 ns]
                        thrpt:  [5.4781 Melem/s 5.5641 Melem/s 5.6500 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) low mild
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estlocal_graph/on_pingwave_duplicate
                        time:   [16.450 ns 16.687 ns 16.976 ns]
                        thrpt:  [58.906 Melem/s 59.929 Melem/s 60.790 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  19 (19.00%) high severe
Benchmarking local_graph/get_node: Collecting 100 samples in estimated 5.0001local_graph/get_node    time:   [14.846 ns 14.884 ns 14.922 ns]
                        thrpt:  [67.015 Melem/s 67.188 Melem/s 67.356 Melem/s]
Benchmarking local_graph/node_count: Collecting 100 samples in estimated 5.00local_graph/node_count  time:   [959.48 ns 960.29 ns 961.15 ns]
                        thrpt:  [1.0404 Melem/s 1.0414 Melem/s 1.0422 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking local_graph/stats: Collecting 100 samples in estimated 5.0134 s local_graph/stats       time:   [2.8859 µs 2.8885 µs 2.8914 µs]
                        thrpt:  [345.85 Kelem/s 346.19 Kelem/s 346.51 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe

Benchmarking graph_scaling/all_nodes/100: Collecting 100 samples in estimatedgraph_scaling/all_nodes/100
                        time:   [7.5476 µs 7.5661 µs 7.5886 µs]
                        thrpt:  [13.178 Melem/s 13.217 Melem/s 13.249 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  6 (6.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in egraph_scaling/nodes_within_hops/100
                        time:   [7.5768 µs 7.5908 µs 7.6044 µs]
                        thrpt:  [13.150 Melem/s 13.174 Melem/s 13.198 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking graph_scaling/all_nodes/500: Collecting 100 samples in estimatedgraph_scaling/all_nodes/500
                        time:   [16.292 µs 16.336 µs 16.377 µs]
                        thrpt:  [30.531 Melem/s 30.607 Melem/s 30.690 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in egraph_scaling/nodes_within_hops/500
                        time:   [16.501 µs 16.575 µs 16.642 µs]
                        thrpt:  [30.044 Melem/s 30.165 Melem/s 30.300 Melem/s]
Benchmarking graph_scaling/all_nodes/1000: Collecting 100 samples in estimategraph_scaling/all_nodes/1000
                        time:   [27.196 µs 27.246 µs 27.296 µs]
                        thrpt:  [36.636 Melem/s 36.702 Melem/s 36.770 Melem/s]
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in graph_scaling/nodes_within_hops/1000
                        time:   [27.105 µs 27.232 µs 27.347 µs]
                        thrpt:  [36.568 Melem/s 36.721 Melem/s 36.894 Melem/s]
Benchmarking graph_scaling/all_nodes/5000: Collecting 100 samples in estimategraph_scaling/all_nodes/5000
                        time:   [228.37 µs 230.49 µs 233.03 µs]
                        thrpt:  [21.457 Melem/s 21.693 Melem/s 21.894 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  12 (12.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in graph_scaling/nodes_within_hops/5000
                        time:   [228.58 µs 230.87 µs 233.70 µs]
                        thrpt:  [21.395 Melem/s 21.657 Melem/s 21.875 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimcapability_search/find_with_gpu
                        time:   [30.579 µs 30.636 µs 30.692 µs]
                        thrpt:  [32.581 Kelem/s 32.641 Kelem/s 32.703 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples incapability_search/find_by_tool_python
                        time:   [60.717 µs 60.779 µs 60.848 µs]
                        thrpt:  [16.434 Kelem/s 16.453 Kelem/s 16.470 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in ecapability_search/find_by_tool_rust
                        time:   [81.180 µs 81.270 µs 81.372 µs]
                        thrpt:  [12.289 Kelem/s 12.305 Kelem/s 12.318 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples ingraph_concurrent/concurrent_pingwave/4
                        time:   [166.73 µs 168.66 µs 171.17 µs]
                        thrpt:  [11.684 Melem/s 11.858 Melem/s 11.995 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples ingraph_concurrent/concurrent_pingwave/8
                        time:   [268.45 µs 272.86 µs 278.38 µs]
                        thrpt:  [14.369 Melem/s 14.659 Melem/s 14.901 Melem/s]
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples igraph_concurrent/concurrent_pingwave/16
                        time:   [499.28 µs 512.19 µs 524.82 µs]
                        thrpt:  [15.243 Melem/s 15.619 Melem/s 16.023 Melem/s]

Benchmarking path_finding/path_1_hop: Collecting 100 samples in estimated 5.0path_finding/path_1_hop time:   [5.6513 µs 5.6573 µs 5.6640 µs]
                        thrpt:  [176.55 Kelem/s 176.76 Kelem/s 176.95 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking path_finding/path_2_hops: Collecting 100 samples in estimated 5.path_finding/path_2_hops
                        time:   [5.7113 µs 5.7191 µs 5.7275 µs]
                        thrpt:  [174.60 Kelem/s 174.85 Kelem/s 175.09 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking path_finding/path_4_hops: Collecting 100 samples in estimated 5.path_finding/path_4_hops
                        time:   [5.8795 µs 5.8860 µs 5.8929 µs]
                        thrpt:  [169.70 Kelem/s 169.90 Kelem/s 170.08 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking path_finding/path_not_found: Collecting 100 samples in estimatedpath_finding/path_not_found
                        time:   [5.7840 µs 5.7897 µs 5.7960 µs]
                        thrpt:  [172.53 Kelem/s 172.72 Kelem/s 172.89 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking path_finding/path_complex_graph: Collecting 100 samples in estimpath_finding/path_complex_graph
                        time:   [181.40 µs 181.59 µs 181.78 µs]
                        thrpt:  [5.5012 Kelem/s 5.5069 Kelem/s 5.5125 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in efailure_detector/heartbeat_existing
                        time:   [35.315 ns 35.349 ns 35.388 ns]
                        thrpt:  [28.259 Melem/s 28.289 Melem/s 28.317 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking failure_detector/heartbeat_new: Collecting 100 samples in estimafailure_detector/heartbeat_new
                        time:   [203.93 ns 207.59 ns 211.45 ns]
                        thrpt:  [4.7292 Melem/s 4.8172 Melem/s 4.9037 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) low mild
Benchmarking failure_detector/status_check: Collecting 100 samples in estimatfailure_detector/status_check
                        time:   [13.490 ns 13.525 ns 13.577 ns]
                        thrpt:  [73.654 Melem/s 73.939 Melem/s 74.126 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  7 (7.00%) high mild
  5 (5.00%) high severe
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 68.6s, or reduce sample count to 10.
Benchmarking failure_detector/check_all: Collecting 100 samples in estimated failure_detector/check_all
                        time:   [663.14 ms 664.73 ms 666.28 ms]
                        thrpt:  [1.5009  elem/s 1.5044  elem/s 1.5080  elem/s]
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 20.1s, or reduce sample count to 20.
Benchmarking failure_detector/stats: Collecting 100 samples in estimated 20.0failure_detector/stats  time:   [200.59 ms 200.77 ms 200.93 ms]
                        thrpt:  [4.9767  elem/s 4.9809  elem/s 4.9852  elem/s]

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimloss_simulator/should_drop_1pct
                        time:   [11.061 ns 11.071 ns 11.083 ns]
                        thrpt:  [90.229 Melem/s 90.323 Melem/s 90.410 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  11 (11.00%) high severe
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimloss_simulator/should_drop_5pct
                        time:   [11.467 ns 11.475 ns 11.484 ns]
                        thrpt:  [87.075 Melem/s 87.143 Melem/s 87.204 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) high mild
  11 (11.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estiloss_simulator/should_drop_10pct
                        time:   [11.980 ns 11.991 ns 12.003 ns]
                        thrpt:  [83.312 Melem/s 83.397 Melem/s 83.474 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  12 (12.00%) high severe
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estiloss_simulator/should_drop_20pct
                        time:   [13.019 ns 13.033 ns 13.047 ns]
                        thrpt:  [76.646 Melem/s 76.730 Melem/s 76.809 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  11 (11.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estiloss_simulator/should_drop_burst
                        time:   [11.568 ns 11.578 ns 11.589 ns]
                        thrpt:  [86.290 Melem/s 86.372 Melem/s 86.445 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  18 (18.00%) high severe

Benchmarking circuit_breaker/allow_closed: Collecting 100 samples in estimatecircuit_breaker/allow_closed
                        time:   [10.284 ns 10.300 ns 10.325 ns]
                        thrpt:  [96.849 Melem/s 97.085 Melem/s 97.243 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  13 (13.00%) high severe
Benchmarking circuit_breaker/record_success: Collecting 100 samples in estimacircuit_breaker/record_success
                        time:   [8.8067 ns 8.8206 ns 8.8361 ns]
                        thrpt:  [113.17 Melem/s 113.37 Melem/s 113.55 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking circuit_breaker/record_failure: Collecting 100 samples in estimacircuit_breaker/record_failure
                        time:   [9.9075 ns 9.9178 ns 9.9278 ns]
                        thrpt:  [100.73 Melem/s 100.83 Melem/s 100.93 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking circuit_breaker/state: Collecting 100 samples in estimated 5.000circuit_breaker/state   time:   [10.265 ns 10.276 ns 10.287 ns]
                        thrpt:  [97.215 Melem/s 97.317 Melem/s 97.414 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  6 (6.00%) high mild

Benchmarking recovery_manager/on_failure_with_alternates: Warming up for 3.00Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samprecovery_manager/on_failure_with_alternates
                        time:   [262.12 ns 264.70 ns 267.35 ns]
                        thrpt:  [3.7404 Melem/s 3.7779 Melem/s 3.8151 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) low severe
  4 (4.00%) low mild
Benchmarking recovery_manager/on_failure_no_alternates: Warming up for 3.0000Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samplerecovery_manager/on_failure_no_alternates
                        time:   [220.34 ns 229.99 ns 247.18 ns]
                        thrpt:  [4.0457 Melem/s 4.3480 Melem/s 4.5385 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  10 (10.00%) low mild
  2 (2.00%) high severe
Benchmarking recovery_manager/get_action: Collecting 100 samples in estimatedrecovery_manager/get_action
                        time:   [37.693 ns 37.769 ns 37.857 ns]
                        thrpt:  [26.415 Melem/s 26.477 Melem/s 26.530 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking recovery_manager/is_failed: Collecting 100 samples in estimated recovery_manager/is_failed
                        time:   [12.839 ns 12.850 ns 12.863 ns]
                        thrpt:  [77.744 Melem/s 77.820 Melem/s 77.887 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) high mild
  13 (13.00%) high severe
Benchmarking recovery_manager/on_recovery: Collecting 100 samples in estimaterecovery_manager/on_recovery
                        time:   [124.37 ns 124.49 ns 124.63 ns]
                        thrpt:  [8.0238 Melem/s 8.0325 Melem/s 8.0403 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking recovery_manager/stats: Collecting 100 samples in estimated 5.00recovery_manager/stats  time:   [1.2058 ns 1.2068 ns 1.2080 ns]
                        thrpt:  [827.79 Melem/s 828.61 Melem/s 829.33 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
  13 (13.00%) high severe

Benchmarking failure_scaling/check_all/100: Collecting 100 samples in estimatfailure_scaling/check_all/100
                        time:   [8.8954 µs 8.9082 µs 8.9192 µs]
                        thrpt:  [11.212 Melem/s 11.226 Melem/s 11.242 Melem/s]
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estfailure_scaling/healthy_nodes/100
                        time:   [6.3201 µs 6.3261 µs 6.3322 µs]
                        thrpt:  [15.792 Melem/s 15.807 Melem/s 15.822 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking failure_scaling/check_all/500: Collecting 100 samples in estimatfailure_scaling/check_all/500
                        time:   [23.220 µs 23.251 µs 23.284 µs]
                        thrpt:  [21.474 Melem/s 21.504 Melem/s 21.533 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estfailure_scaling/healthy_nodes/500
                        time:   [10.013 µs 10.023 µs 10.034 µs]
                        thrpt:  [49.829 Melem/s 49.884 Melem/s 49.934 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking failure_scaling/check_all/1000: Collecting 100 samples in estimafailure_scaling/check_all/1000
                        time:   [41.189 µs 41.233 µs 41.273 µs]
                        thrpt:  [24.229 Melem/s 24.253 Melem/s 24.278 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  16 (16.00%) low severe
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in esfailure_scaling/healthy_nodes/1000
                        time:   [14.854 µs 14.875 µs 14.898 µs]
                        thrpt:  [67.122 Melem/s 67.226 Melem/s 67.323 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking failure_scaling/check_all/5000: Collecting 100 samples in estimafailure_scaling/check_all/5000
                        time:   [185.44 µs 185.63 µs 185.85 µs]
                        thrpt:  [26.904 Melem/s 26.935 Melem/s 26.962 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in esfailure_scaling/healthy_nodes/5000
                        time:   [53.559 µs 53.617 µs 53.675 µs]
                        thrpt:  [93.154 Melem/s 93.255 Melem/s 93.355 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  9 (9.00%) high mild
  2 (2.00%) high severe

Benchmarking failure_concurrent/concurrent_heartbeat/4: Warming up for 3.0000Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samplesfailure_concurrent/concurrent_heartbeat/4
                        time:   [201.60 µs 204.32 µs 206.98 µs]
                        thrpt:  [9.6626 Melem/s 9.7888 Melem/s 9.9204 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking failure_concurrent/concurrent_heartbeat/8: Warming up for 3.0000Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samplesfailure_concurrent/concurrent_heartbeat/8
                        time:   [307.93 µs 315.26 µs 322.00 µs]
                        thrpt:  [12.422 Melem/s 12.688 Melem/s 12.990 Melem/s]
Benchmarking failure_concurrent/concurrent_heartbeat/16: Warming up for 3.000Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samplefailure_concurrent/concurrent_heartbeat/16
                        time:   [546.27 µs 563.77 µs 579.40 µs]
                        thrpt:  [13.807 Melem/s 14.190 Melem/s 14.645 Melem/s]

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estfailure_recovery_cycle/full_cycle
                        time:   [257.02 ns 260.79 ns 264.70 ns]
                        thrpt:  [3.7778 Melem/s 3.8345 Melem/s 3.8907 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) low mild

Benchmarking capability_set/create: Collecting 100 samples in estimated 5.002capability_set/create   time:   [776.91 ns 778.09 ns 779.38 ns]
                        thrpt:  [1.2831 Melem/s 1.2852 Melem/s 1.2872 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_set/serialize: Collecting 100 samples in estimated 5.capability_set/serialize
                        time:   [667.17 ns 668.08 ns 669.03 ns]
                        thrpt:  [1.4947 Melem/s 1.4968 Melem/s 1.4989 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking capability_set/deserialize: Collecting 100 samples in estimated capability_set/deserialize
                        time:   [3.0029 µs 3.0060 µs 3.0095 µs]
                        thrpt:  [332.28 Kelem/s 332.67 Kelem/s 333.01 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_set/roundtrip: Collecting 100 samples in estimated 5.capability_set/roundtrip
                        time:   [4.5771 µs 4.5812 µs 4.5857 µs]
                        thrpt:  [218.07 Kelem/s 218.28 Kelem/s 218.48 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_set/has_tag: Collecting 100 samples in estimated 5.00capability_set/has_tag  time:   [603.18 ps 603.84 ps 604.54 ps]
                        thrpt:  [1.6542 Gelem/s 1.6561 Gelem/s 1.6579 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_set/has_model: Collecting 100 samples in estimated 5.capability_set/has_model
                        time:   [441.55 ps 445.58 ps 449.26 ps]
                        thrpt:  [2.2259 Gelem/s 2.2443 Gelem/s 2.2647 Gelem/s]
Benchmarking capability_set/has_tool: Collecting 100 samples in estimated 5.0capability_set/has_tool time:   [603.14 ps 604.18 ps 605.83 ps]
                        thrpt:  [1.6506 Gelem/s 1.6551 Gelem/s 1.6580 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_set/has_gpu: Collecting 100 samples in estimated 5.00capability_set/has_gpu  time:   [201.11 ps 201.32 ps 201.57 ps]
                        thrpt:  [4.9610 Gelem/s 4.9672 Gelem/s 4.9725 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe

Benchmarking capability_announcement/create: Collecting 100 samples in estimacapability_announcement/create
                        time:   [1.5122 µs 1.5135 µs 1.5149 µs]
                        thrpt:  [660.11 Kelem/s 660.71 Kelem/s 661.29 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_announcement/serialize: Collecting 100 samples in estcapability_announcement/serialize
                        time:   [1.6633 µs 1.6651 µs 1.6671 µs]
                        thrpt:  [599.86 Kelem/s 600.58 Kelem/s 601.23 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in ecapability_announcement/deserialize
                        time:   [2.6550 µs 2.6587 µs 2.6627 µs]
                        thrpt:  [375.56 Kelem/s 376.13 Kelem/s 376.65 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_announcement/is_expired: Collecting 100 samples in escapability_announcement/is_expired
                        time:   [21.819 ns 21.838 ns 21.859 ns]
                        thrpt:  [45.749 Melem/s 45.792 Melem/s 45.832 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in escapability_filter/match_single_tag
                        time:   [3.4308 ns 3.4359 ns 3.4404 ns]
                        thrpt:  [290.67 Melem/s 291.04 Melem/s 291.48 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in ecapability_filter/match_require_gpu
                        time:   [1.8096 ns 1.8113 ns 1.8133 ns]
                        thrpt:  [551.49 Melem/s 552.08 Melem/s 552.62 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in escapability_filter/match_gpu_vendor
                        time:   [2.0108 ns 2.0124 ns 2.0143 ns]
                        thrpt:  [496.44 Melem/s 496.91 Melem/s 497.33 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) high mild
  6 (6.00%) high severe
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in escapability_filter/match_min_memory
                        time:   [1.8109 ns 1.8126 ns 1.8145 ns]
                        thrpt:  [551.11 Melem/s 551.69 Melem/s 552.21 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimcapability_filter/match_complex
                        time:   [5.8215 ns 5.8556 ns 5.8928 ns]
                        thrpt:  [169.70 Melem/s 170.78 Melem/s 171.78 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) low mild
  5 (5.00%) high mild
Benchmarking capability_filter/match_no_match: Collecting 100 samples in esticapability_filter/match_no_match
                        time:   [1.9569 ns 1.9699 ns 1.9824 ns]
                        thrpt:  [504.45 Melem/s 507.64 Melem/s 511.02 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples capability_index_insert/index_nodes/100
                        time:   [189.13 µs 189.33 µs 189.56 µs]
                        thrpt:  [527.53 Kelem/s 528.18 Kelem/s 528.73 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.6s, enable flat sampling, or reduce sample count to 50.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samplescapability_index_insert/index_nodes/1000
                        time:   [1.4975 ms 1.4997 ms 1.5019 ms]
                        thrpt:  [665.84 Kelem/s 666.82 Kelem/s 667.80 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_insert/index_nodes/10000: Warming up for 3.0000Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samplecapability_index_insert/index_nodes/10000
                        time:   [16.674 ms 16.834 ms 17.001 ms]
                        thrpt:  [588.19 Kelem/s 594.03 Kelem/s 599.72 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples capability_index_query/query_single_tag
                        time:   [177.93 µs 179.32 µs 180.64 µs]
                        thrpt:  [5.5360 Kelem/s 5.5765 Kelem/s 5.6203 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_query/query_require_gpu: Warming up for 3.0000 Benchmarking capability_index_query/query_require_gpu: Collecting 100 samplescapability_index_query/query_require_gpu
                        time:   [188.09 µs 188.52 µs 188.98 µs]
                        thrpt:  [5.2915 Kelem/s 5.3044 Kelem/s 5.3166 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples capability_index_query/query_gpu_vendor
                        time:   [517.76 µs 518.78 µs 519.81 µs]
                        thrpt:  [1.9238 Kelem/s 1.9276 Kelem/s 1.9314 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples capability_index_query/query_min_memory
                        time:   [538.44 µs 539.60 µs 540.72 µs]
                        thrpt:  [1.8494 Kelem/s 1.8532 Kelem/s 1.8572 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_query/query_complex: Collecting 100 samples in capability_index_query/query_complex
                        time:   [342.34 µs 342.95 µs 343.55 µs]
                        thrpt:  [2.9108 Kelem/s 2.9159 Kelem/s 2.9211 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_model: Collecting 100 samples in escapability_index_query/query_model
                        time:   [94.974 µs 95.210 µs 95.468 µs]
                        thrpt:  [10.475 Kelem/s 10.503 Kelem/s 10.529 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  9 (9.00%) high severe
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estcapability_index_query/query_tool
                        time:   [492.61 µs 493.70 µs 495.15 µs]
                        thrpt:  [2.0196 Kelem/s 2.0255 Kelem/s 2.0300 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_query/query_no_results: Collecting 100 samples capability_index_query/query_no_results
                        time:   [26.391 ns 26.413 ns 26.438 ns]
                        thrpt:  [37.824 Melem/s 37.860 Melem/s 37.892 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  5 (5.00%) high mild
  10 (10.00%) high severe

Benchmarking capability_index_find_best/find_best_simple: Warming up for 3.00Benchmarking capability_index_find_best/find_best_simple: Collecting 100 sampcapability_index_find_best/find_best_simple
                        time:   [342.49 µs 343.25 µs 344.05 µs]
                        thrpt:  [2.9066 Kelem/s 2.9134 Kelem/s 2.9198 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_find_best/find_best_with_prefs: Warming up for Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 capability_index_find_best/find_best_with_prefs
                        time:   [545.82 µs 546.49 µs 547.19 µs]
                        thrpt:  [1.8275 Kelem/s 1.8299 Kelem/s 1.8321 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples capability_index_scaling/query_tag/1000
                        time:   [10.580 µs 10.593 µs 10.607 µs]
                        thrpt:  [94.274 Kelem/s 94.398 Kelem/s 94.514 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) high mild
  9 (9.00%) high severe
Benchmarking capability_index_scaling/query_complex/1000: Warming up for 3.00Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 sampcapability_index_scaling/query_complex/1000
                        time:   [27.825 µs 27.860 µs 27.900 µs]
                        thrpt:  [35.842 Kelem/s 35.893 Kelem/s 35.939 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples capability_index_scaling/query_tag/5000
                        time:   [55.089 µs 55.177 µs 55.271 µs]
                        thrpt:  [18.093 Kelem/s 18.123 Kelem/s 18.152 Kelem/s]
Found 17 outliers among 100 measurements (17.00%)
  4 (4.00%) high mild
  13 (13.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Warming up for 3.00Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 sampcapability_index_scaling/query_complex/5000
                        time:   [136.17 µs 136.36 µs 136.58 µs]
                        thrpt:  [7.3219 Kelem/s 7.3334 Kelem/s 7.3440 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_scaling/query_tag/10000: Warming up for 3.0000 Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samplescapability_index_scaling/query_tag/10000
                        time:   [171.90 µs 173.09 µs 174.28 µs]
                        thrpt:  [5.7379 Kelem/s 5.7774 Kelem/s 5.8172 Kelem/s]
Found 18 outliers among 100 measurements (18.00%)
  12 (12.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_complex/10000: Warming up for 3.0Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samcapability_index_scaling/query_complex/10000
                        time:   [345.70 µs 346.43 µs 347.19 µs]
                        thrpt:  [2.8802 Kelem/s 2.8866 Kelem/s 2.8927 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_tag/50000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samplescapability_index_scaling/query_tag/50000
                        time:   [1.2411 ms 1.2576 ms 1.2761 ms]
                        thrpt:  [783.66  elem/s 795.15  elem/s 805.76  elem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_complex/50000: Warming up for 3.0Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samcapability_index_scaling/query_complex/50000
                        time:   [2.1776 ms 2.2123 ms 2.2491 ms]
                        thrpt:  [444.62  elem/s 452.02  elem/s 459.23  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking capability_index_concurrent/concurrent_index/4: Warming up for 3Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 sacapability_index_concurrent/concurrent_index/4
                        time:   [637.52 µs 644.18 µs 650.09 µs]
                        thrpt:  [3.0765 Melem/s 3.1047 Melem/s 3.1371 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 sacapability_index_concurrent/concurrent_query/4
                        time:   [245.38 ms 246.76 ms 248.19 ms]
                        thrpt:  [8.0584 Kelem/s 8.1051 Kelem/s 8.1505 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/4: Warming up for 3Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 sacapability_index_concurrent/concurrent_mixed/4
                        time:   [120.63 ms 121.26 ms 121.88 ms]
                        thrpt:  [16.409 Kelem/s 16.493 Kelem/s 16.580 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/8: Warming up for 3Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 sacapability_index_concurrent/concurrent_index/8
                        time:   [864.81 µs 883.34 µs 903.84 µs]
                        thrpt:  [4.4256 Melem/s 4.5283 Melem/s 4.6253 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.0s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 sacapability_index_concurrent/concurrent_query/8
                        time:   [290.06 ms 298.07 ms 306.83 ms]
                        thrpt:  [13.036 Kelem/s 13.420 Kelem/s 13.790 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/8: Warming up for 3Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 sacapability_index_concurrent/concurrent_mixed/8
                        time:   [156.03 ms 159.29 ms 162.91 ms]
                        thrpt:  [24.554 Kelem/s 25.111 Kelem/s 25.636 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
Benchmarking capability_index_concurrent/concurrent_index/16: Warming up for Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 scapability_index_concurrent/concurrent_index/16
                        time:   [1.5913 ms 1.6118 ms 1.6371 ms]
                        thrpt:  [4.8866 Melem/s 4.9633 Melem/s 5.0272 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.8s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 scapability_index_concurrent/concurrent_query/16
                        time:   [435.34 ms 437.50 ms 439.59 ms]
                        thrpt:  [18.199 Kelem/s 18.286 Kelem/s 18.377 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 scapability_index_concurrent/concurrent_mixed/16
                        time:   [231.56 ms 232.85 ms 234.18 ms]
                        thrpt:  [34.162 Kelem/s 34.357 Kelem/s 34.549 Kelem/s]

Benchmarking capability_index_updates/update_higher_version: Warming up for 3Benchmarking capability_index_updates/update_higher_version: Collecting 100 scapability_index_updates/update_higher_version
                        time:   [814.15 ns 816.33 ns 818.80 ns]
                        thrpt:  [1.2213 Melem/s 1.2250 Melem/s 1.2283 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_updates/update_same_version: Warming up for 3.0Benchmarking capability_index_updates/update_same_version: Collecting 100 samcapability_index_updates/update_same_version
                        time:   [815.54 ns 816.78 ns 818.15 ns]
                        thrpt:  [1.2223 Melem/s 1.2243 Melem/s 1.2262 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking capability_index_updates/remove_and_readd: Warming up for 3.0000Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samplecapability_index_updates/remove_and_readd
                        time:   [2.0229 µs 2.0303 µs 2.0432 µs]
                        thrpt:  [489.42 Kelem/s 492.54 Kelem/s 494.35 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe

Benchmarking diff_op/create_add_tag: Collecting 100 samples in estimated 5.00diff_op/create_add_tag  time:   [24.727 ns 24.764 ns 24.802 ns]
                        thrpt:  [40.319 Melem/s 40.382 Melem/s 40.441 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low severe
  1 (1.00%) high mild
Benchmarking diff_op/create_remove_tag: Collecting 100 samples in estimated 5diff_op/create_remove_tag
                        time:   [24.559 ns 24.593 ns 24.627 ns]
                        thrpt:  [40.605 Melem/s 40.663 Melem/s 40.717 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_op/create_add_model: Collecting 100 samples in estimated 5.diff_op/create_add_model
                        time:   [88.206 ns 88.385 ns 88.552 ns]
                        thrpt:  [11.293 Melem/s 11.314 Melem/s 11.337 Melem/s]
Benchmarking diff_op/create_update_model: Collecting 100 samples in estimateddiff_op/create_update_model
                        time:   [24.546 ns 24.598 ns 24.657 ns]
                        thrpt:  [40.556 Melem/s 40.654 Melem/s 40.740 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking diff_op/estimated_size: Collecting 100 samples in estimated 5.00diff_op/estimated_size  time:   [1.0037 ns 1.0053 ns 1.0069 ns]
                        thrpt:  [993.19 Melem/s 994.71 Melem/s 996.32 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  8 (8.00%) high mild
  10 (10.00%) high severe

Benchmarking capability_diff/create: Collecting 100 samples in estimated 5.00capability_diff/create  time:   [86.969 ns 87.132 ns 87.313 ns]
                        thrpt:  [11.453 Melem/s 11.477 Melem/s 11.498 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking capability_diff/serialize: Collecting 100 samples in estimated 5capability_diff/serialize
                        time:   [878.77 ns 880.31 ns 881.87 ns]
                        thrpt:  [1.1340 Melem/s 1.1360 Melem/s 1.1380 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking capability_diff/deserialize: Collecting 100 samples in estimatedcapability_diff/deserialize
                        time:   [237.80 ns 238.08 ns 238.40 ns]
                        thrpt:  [4.1947 Melem/s 4.2003 Melem/s 4.2053 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_diff/estimated_size: Collecting 100 samples in estimacapability_diff/estimated_size
                        time:   [2.4116 ns 2.4140 ns 2.4164 ns]
                        thrpt:  [413.83 Melem/s 414.25 Melem/s 414.67 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  9 (9.00%) high severe

Benchmarking diff_generation/no_changes: Collecting 100 samples in estimated diff_generation/no_changes
                        time:   [293.71 ns 294.48 ns 295.28 ns]
                        thrpt:  [3.3866 Melem/s 3.3958 Melem/s 3.4047 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking diff_generation/add_one_tag: Collecting 100 samples in estimateddiff_generation/add_one_tag
                        time:   [405.22 ns 405.91 ns 406.57 ns]
                        thrpt:  [2.4596 Melem/s 2.4636 Melem/s 2.4678 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in diff_generation/multiple_tag_changes
                        time:   [458.33 ns 459.06 ns 459.74 ns]
                        thrpt:  [2.1752 Melem/s 2.1783 Melem/s 2.1818 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in ediff_generation/update_model_loaded
                        time:   [356.92 ns 357.88 ns 358.99 ns]
                        thrpt:  [2.7856 Melem/s 2.7942 Melem/s 2.8018 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking diff_generation/update_memory: Collecting 100 samples in estimatdiff_generation/update_memory
                        time:   [321.53 ns 323.42 ns 325.69 ns]
                        thrpt:  [3.0705 Melem/s 3.0919 Melem/s 3.1101 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking diff_generation/add_model: Collecting 100 samples in estimated 5diff_generation/add_model
                        time:   [426.90 ns 428.44 ns 430.15 ns]
                        thrpt:  [2.3248 Melem/s 2.3340 Melem/s 2.3425 Melem/s]
Benchmarking diff_generation/complex_diff: Collecting 100 samples in estimatediff_generation/complex_diff
                        time:   [785.17 ns 789.35 ns 793.62 ns]
                        thrpt:  [1.2600 Melem/s 1.2669 Melem/s 1.2736 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estidiff_application/apply_single_op
                        time:   [604.17 ns 607.23 ns 610.43 ns]
                        thrpt:  [1.6382 Melem/s 1.6468 Melem/s 1.6552 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estdiff_application/apply_small_diff
                        time:   [616.58 ns 619.86 ns 623.26 ns]
                        thrpt:  [1.6045 Melem/s 1.6133 Melem/s 1.6218 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in esdiff_application/apply_medium_diff
                        time:   [2.7052 µs 2.7122 µs 2.7197 µs]
                        thrpt:  [367.68 Kelem/s 368.70 Kelem/s 369.66 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in esdiff_application/apply_strict_mode
                        time:   [1.4527 µs 1.4547 µs 1.4566 µs]
                        thrpt:  [686.54 Kelem/s 687.45 Kelem/s 688.38 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples diff_chain_validation/validate_chain_10
                        time:   [3.8431 ns 3.8507 ns 3.8581 ns]
                        thrpt:  [259.19 Melem/s 259.69 Melem/s 260.21 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_chain_validation/validate_chain_100: Warming up for 3.0000 Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samplesdiff_chain_validation/validate_chain_100
                        time:   [48.422 ns 48.473 ns 48.528 ns]
                        thrpt:  [20.607 Melem/s 20.630 Melem/s 20.652 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  6 (6.00%) high severe

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimdiff_compaction/compact_5_diffs
                        time:   [6.9436 µs 6.9517 µs 6.9601 µs]
                        thrpt:  [143.68 Kelem/s 143.85 Kelem/s 144.02 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estidiff_compaction/compact_20_diffs
                        time:   [34.100 µs 34.138 µs 34.182 µs]
                        thrpt:  [29.255 Kelem/s 29.292 Kelem/s 29.326 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estbandwidth_savings/calculate_small
                        time:   [651.05 ns 652.10 ns 653.23 ns]
                        thrpt:  [1.5308 Melem/s 1.5335 Melem/s 1.5360 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in esbandwidth_savings/calculate_medium
                        time:   [658.78 ns 659.83 ns 660.91 ns]
                        thrpt:  [1.5131 Melem/s 1.5155 Melem/s 1.5180 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in diff_roundtrip/generate_apply_verify
                        time:   [2.4005 µs 2.4039 µs 2.4076 µs]
                        thrpt:  [415.35 Kelem/s 415.99 Kelem/s 416.58 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

Benchmarking location_info/create: Collecting 100 samples in estimated 5.0002location_info/create    time:   [54.699 ns 54.791 ns 54.880 ns]
                        thrpt:  [18.222 Melem/s 18.251 Melem/s 18.282 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking location_info/distance_to: Collecting 100 samples in estimated 5location_info/distance_to
                        time:   [2.6488 ns 2.6519 ns 2.6556 ns]
                        thrpt:  [376.56 Melem/s 377.09 Melem/s 377.53 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking location_info/same_continent: Collecting 100 samples in estimatelocation_info/same_continent
                        time:   [2.6134 ns 2.6156 ns 2.6181 ns]
                        thrpt:  [381.96 Melem/s 382.32 Melem/s 382.65 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in eslocation_info/same_continent_cross
                        time:   [209.79 ps 213.29 ps 217.41 ps]
                        thrpt:  [4.5996 Gelem/s 4.6884 Gelem/s 4.7668 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
Benchmarking location_info/same_region: Collecting 100 samples in estimated 5location_info/same_region
                        time:   [2.0120 ns 2.0137 ns 2.0156 ns]
                        thrpt:  [496.14 Melem/s 496.59 Melem/s 497.01 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe

Benchmarking topology_hints/create: Collecting 100 samples in estimated 5.000topology_hints/create   time:   [3.5680 ns 3.5900 ns 3.6088 ns]
                        thrpt:  [277.10 Melem/s 278.55 Melem/s 280.27 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  19 (19.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in esttopology_hints/connectivity_score
                        time:   [201.17 ps 201.42 ps 201.75 ps]
                        thrpt:  [4.9565 Gelem/s 4.9647 Gelem/s 4.9709 Gelem/s]
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in topology_hints/average_latency_empty
                        time:   [265.46 ps 267.63 ps 270.13 ps]
                        thrpt:  [3.7020 Gelem/s 3.7365 Gelem/s 3.7671 Gelem/s]
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estopology_hints/average_latency_100
                        time:   [47.934 ns 47.995 ns 48.062 ns]
                        thrpt:  [20.806 Melem/s 20.835 Melem/s 20.862 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  10 (10.00%) high mild
  2 (2.00%) high severe

Benchmarking nat_type/difficulty: Collecting 100 samples in estimated 5.0000 nat_type/difficulty     time:   [200.99 ps 201.17 ps 201.37 ps]
                        thrpt:  [4.9659 Gelem/s 4.9709 Gelem/s 4.9755 Gelem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low severe
  4 (4.00%) high mild
  8 (8.00%) high severe
Benchmarking nat_type/can_connect_direct: Collecting 100 samples in estimatednat_type/can_connect_direct
                        time:   [200.93 ps 201.11 ps 201.30 ps]
                        thrpt:  [4.9678 Gelem/s 4.9723 Gelem/s 4.9768 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking nat_type/can_connect_symmetric: Collecting 100 samples in estimanat_type/can_connect_symmetric
                        time:   [200.54 ps 200.71 ps 200.90 ps]
                        thrpt:  [4.9776 Gelem/s 4.9824 Gelem/s 4.9865 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking node_metadata/create_simple: Collecting 100 samples in estimatednode_metadata/create_simple
                        time:   [31.611 ns 31.669 ns 31.732 ns]
                        thrpt:  [31.514 Melem/s 31.577 Melem/s 31.635 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking node_metadata/create_full: Collecting 100 samples in estimated 5node_metadata/create_full
                        time:   [432.48 ns 433.47 ns 434.38 ns]
                        thrpt:  [2.3022 Melem/s 2.3070 Melem/s 2.3123 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking node_metadata/routing_score: Collecting 100 samples in estimatednode_metadata/routing_score
                        time:   [200.97 ps 201.15 ps 201.36 ps]
                        thrpt:  [4.9662 Gelem/s 4.9714 Gelem/s 4.9759 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking node_metadata/age: Collecting 100 samples in estimated 5.0001 s node_metadata/age       time:   [25.045 ns 25.068 ns 25.091 ns]
                        thrpt:  [39.855 Melem/s 39.892 Melem/s 39.927 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking node_metadata/is_stale: Collecting 100 samples in estimated 5.00node_metadata/is_stale  time:   [24.424 ns 24.446 ns 24.469 ns]
                        thrpt:  [40.868 Melem/s 40.907 Melem/s 40.943 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking node_metadata/serialize: Collecting 100 samples in estimated 5.0node_metadata/serialize time:   [560.00 ns 560.89 ns 561.84 ns]
                        thrpt:  [1.7799 Melem/s 1.7829 Melem/s 1.7857 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking node_metadata/deserialize: Collecting 100 samples in estimated 5node_metadata/deserialize
                        time:   [1.6220 µs 1.6249 µs 1.6278 µs]
                        thrpt:  [614.33 Kelem/s 615.42 Kelem/s 616.53 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) low severe
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_query/match_status: Collecting 100 samples in estimatedmetadata_query/match_status
                        time:   [2.4132 ns 2.4156 ns 2.4183 ns]
                        thrpt:  [413.51 Melem/s 413.97 Melem/s 414.39 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_query/match_min_tier: Collecting 100 samples in estimatmetadata_query/match_min_tier
                        time:   [2.4125 ns 2.4147 ns 2.4170 ns]
                        thrpt:  [413.73 Melem/s 414.14 Melem/s 414.51 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  12 (12.00%) high severe
Benchmarking metadata_query/match_continent: Collecting 100 samples in estimametadata_query/match_continent
                        time:   [4.6296 ns 4.6352 ns 4.6408 ns]
                        thrpt:  [215.48 Melem/s 215.74 Melem/s 216.00 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_complex: Collecting 100 samples in estimatemetadata_query/match_complex
                        time:   [4.8191 ns 4.8867 ns 4.9599 ns]
                        thrpt:  [201.62 Melem/s 204.64 Melem/s 207.51 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_query/match_no_match: Collecting 100 samples in estimatmetadata_query/match_no_match
                        time:   [1.4728 ns 1.4973 ns 1.5227 ns]
                        thrpt:  [656.71 Melem/s 667.87 Melem/s 678.99 Melem/s]

Benchmarking metadata_store_basic/create: Collecting 100 samples in estimatedmetadata_store_basic/create
                        time:   [1.9691 µs 1.9738 µs 1.9813 µs]
                        thrpt:  [504.71 Kelem/s 506.63 Kelem/s 507.85 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  9 (9.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimmetadata_store_basic/upsert_new
                        time:   [1.8322 µs 1.8483 µs 1.8649 µs]
                        thrpt:  [536.23 Kelem/s 541.05 Kelem/s 545.80 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  14 (14.00%) low mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in metadata_store_basic/upsert_existing
                        time:   [1.0060 µs 1.0074 µs 1.0088 µs]
                        thrpt:  [991.30 Kelem/s 992.70 Kelem/s 994.04 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking metadata_store_basic/get: Collecting 100 samples in estimated 5.metadata_store_basic/get
                        time:   [23.489 ns 23.523 ns 23.559 ns]
                        thrpt:  [42.447 Melem/s 42.511 Melem/s 42.572 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/get_miss: Collecting 100 samples in estimatmetadata_store_basic/get_miss
                        time:   [23.554 ns 23.590 ns 23.623 ns]
                        thrpt:  [42.331 Melem/s 42.392 Melem/s 42.455 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking metadata_store_basic/len: Collecting 100 samples in estimated 5.metadata_store_basic/len
                        time:   [960.12 ns 961.74 ns 963.67 ns]
                        thrpt:  [1.0377 Melem/s 1.0398 Melem/s 1.0415 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low mild
  8 (8.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 16.8s, or reduce sample count to 20.
Benchmarking metadata_store_basic/stats: Collecting 100 samples in estimated metadata_store_basic/stats
                        time:   [167.26 ms 167.88 ms 168.48 ms]
                        thrpt:  [5.9353  elem/s 5.9568  elem/s 5.9787  elem/s]

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in metadata_store_query/query_by_status
                        time:   [287.15 µs 287.94 µs 288.74 µs]
                        thrpt:  [3.4633 Kelem/s 3.4729 Kelem/s 3.4825 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples metadata_store_query/query_by_continent
                        time:   [139.74 µs 140.18 µs 140.70 µs]
                        thrpt:  [7.1072 Kelem/s 7.1337 Kelem/s 7.1560 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in esmetadata_store_query/query_by_tier
                        time:   [392.30 µs 393.03 µs 393.88 µs]
                        thrpt:  [2.5388 Kelem/s 2.5444 Kelem/s 2.5491 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_query/query_accepting_work: Warming up for 3.0000Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samplemetadata_store_query/query_accepting_work
                        time:   [488.89 µs 489.83 µs 490.86 µs]
                        thrpt:  [2.0372 Kelem/s 2.0415 Kelem/s 2.0454 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples inmetadata_store_query/query_with_limit
                        time:   [482.30 µs 482.83 µs 483.37 µs]
                        thrpt:  [2.0688 Kelem/s 2.0711 Kelem/s 2.0734 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in esmetadata_store_query/query_complex
                        time:   [303.85 µs 304.40 µs 304.98 µs]
                        thrpt:  [3.2789 Kelem/s 3.2851 Kelem/s 3.2911 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_spatial/find_nearby_100km: Warming up for 3.0000 Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samplesmetadata_store_spatial/find_nearby_100km
                        time:   [321.29 µs 321.83 µs 322.31 µs]
                        thrpt:  [3.1026 Kelem/s 3.1072 Kelem/s 3.1124 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking metadata_store_spatial/find_nearby_1000km: Warming up for 3.0000Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samplemetadata_store_spatial/find_nearby_1000km
                        time:   [366.53 µs 367.13 µs 367.74 µs]
                        thrpt:  [2.7193 Kelem/s 2.7238 Kelem/s 2.7283 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_5000km: Warming up for 3.0000Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samplemetadata_store_spatial/find_nearby_5000km
                        time:   [461.35 µs 461.88 µs 462.42 µs]
                        thrpt:  [2.1625 Kelem/s 2.1651 Kelem/s 2.1675 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_spatial/find_best_for_routing: Warming up for 3.0Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 sammetadata_store_spatial/find_best_for_routing
                        time:   [309.98 µs 311.19 µs 312.33 µs]
                        thrpt:  [3.2018 Kelem/s 3.2135 Kelem/s 3.2260 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in esmetadata_store_spatial/find_relays
                        time:   [491.06 µs 492.04 µs 492.92 µs]
                        thrpt:  [2.0287 Kelem/s 2.0323 Kelem/s 2.0364 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Warming up for 3.0000 Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samplesmetadata_store_scaling/query_status/1000
                        time:   [21.517 µs 21.553 µs 21.589 µs]
                        thrpt:  [46.321 Kelem/s 46.397 Kelem/s 46.475 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/query_complex/1000: Warming up for 3.0000Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samplemetadata_store_scaling/query_complex/1000
                        time:   [20.544 µs 20.588 µs 20.636 µs]
                        thrpt:  [48.459 Kelem/s 48.571 Kelem/s 48.676 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  12 (12.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples metadata_store_scaling/find_nearby/1000
                        time:   [46.477 µs 46.514 µs 46.555 µs]
                        thrpt:  [21.480 Kelem/s 21.499 Kelem/s 21.516 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Warming up for 3.0000 Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samplesmetadata_store_scaling/query_status/5000
                        time:   [111.88 µs 112.07 µs 112.27 µs]
                        thrpt:  [8.9072 Kelem/s 8.9233 Kelem/s 8.9382 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_complex/5000: Warming up for 3.0000Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samplemetadata_store_scaling/query_complex/5000
                        time:   [116.17 µs 116.45 µs 116.79 µs]
                        thrpt:  [8.5624 Kelem/s 8.5872 Kelem/s 8.6082 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples metadata_store_scaling/find_nearby/5000
                        time:   [222.22 µs 222.47 µs 222.75 µs]
                        thrpt:  [4.4893 Kelem/s 4.4949 Kelem/s 4.5000 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_status/10000: Warming up for 3.0000Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samplemetadata_store_scaling/query_status/10000
                        time:   [284.31 µs 285.06 µs 285.81 µs]
                        thrpt:  [3.4989 Kelem/s 3.5080 Kelem/s 3.5173 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/10000: Warming up for 3.000Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samplmetadata_store_scaling/query_complex/10000
                        time:   [300.72 µs 301.37 µs 302.07 µs]
                        thrpt:  [3.3105 Kelem/s 3.3181 Kelem/s 3.3254 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/10000: Warming up for 3.0000 Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samplesmetadata_store_scaling/find_nearby/10000
                        time:   [471.85 µs 472.67 µs 473.45 µs]
                        thrpt:  [2.1122 Kelem/s 2.1156 Kelem/s 2.1193 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Warming up for 3.0000Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samplemetadata_store_scaling/query_status/50000
                        time:   [2.1246 ms 2.1559 ms 2.1899 ms]
                        thrpt:  [456.64  elem/s 463.85  elem/s 470.67  elem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/50000: Warming up for 3.000Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samplmetadata_store_scaling/query_complex/50000
                        time:   [2.2361 ms 2.2702 ms 2.3064 ms]
                        thrpt:  [433.57  elem/s 440.48  elem/s 447.20  elem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking metadata_store_scaling/find_nearby/50000: Warming up for 3.0000 Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samplesmetadata_store_scaling/find_nearby/50000
                        time:   [2.4726 ms 2.4895 ms 2.5094 ms]
                        thrpt:  [398.50  elem/s 401.68  elem/s 404.43  elem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Warming up for 3.Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 sammetadata_store_concurrent/concurrent_upsert/4
                        time:   [1.2192 ms 1.2260 ms 1.2313 ms]
                        thrpt:  [1.6243 Melem/s 1.6313 Melem/s 1.6404 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 sampmetadata_store_concurrent/concurrent_query/4
                        time:   [231.61 ms 232.47 ms 233.35 ms]
                        thrpt:  [8.5708 Kelem/s 8.6034 Kelem/s 8.6353 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Warming up for 3.0Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 sampmetadata_store_concurrent/concurrent_mixed/4
                        time:   [206.99 ms 209.59 ms 214.08 ms]
                        thrpt:  [9.3425 Kelem/s 9.5425 Kelem/s 9.6624 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Warming up for 3.Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 sammetadata_store_concurrent/concurrent_upsert/8
                        time:   [1.6416 ms 1.6869 ms 1.7393 ms]
                        thrpt:  [2.2998 Melem/s 2.3712 Melem/s 2.4367 Melem/s]
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.5s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 sampmetadata_store_concurrent/concurrent_query/8
                        time:   [267.21 ms 271.51 ms 275.72 ms]
                        thrpt:  [14.508 Kelem/s 14.733 Kelem/s 14.969 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.1s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 sampmetadata_store_concurrent/concurrent_mixed/8
                        time:   [249.11 ms 254.44 ms 260.09 ms]
                        thrpt:  [15.379 Kelem/s 15.721 Kelem/s 16.057 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Warming up for 3Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 sametadata_store_concurrent/concurrent_upsert/16
                        time:   [3.4295 ms 3.4412 ms 3.4546 ms]
                        thrpt:  [2.3158 Melem/s 2.3248 Melem/s 2.3327 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.4s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 sammetadata_store_concurrent/concurrent_query/16
                        time:   [418.19 ms 422.16 ms 426.12 ms]
                        thrpt:  [18.774 Kelem/s 18.950 Kelem/s 19.130 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.2s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 sammetadata_store_concurrent/concurrent_mixed/16
                        time:   [463.08 ms 468.08 ms 472.91 ms]
                        thrpt:  [16.916 Kelem/s 17.091 Kelem/s 17.276 Kelem/s]

Benchmarking metadata_store_versioning/update_versioned_success: Warming up fBenchmarking metadata_store_versioning/update_versioned_success: Collecting 1metadata_store_versioning/update_versioned_success
                        time:   [220.94 ns 221.20 ns 221.50 ns]
                        thrpt:  [4.5147 Melem/s 4.5208 Melem/s 4.5262 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Warming up Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting metadata_store_versioning/update_versioned_conflict
                        time:   [219.08 ns 219.32 ns 219.57 ns]
                        thrpt:  [4.5544 Melem/s 4.5596 Melem/s 4.5645 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estschema_validation/validate_string
                        time:   [2.7211 ns 3.0568 ns 3.4573 ns]
                        thrpt:  [289.24 Melem/s 327.14 Melem/s 367.50 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  20 (20.00%) high severe
Benchmarking schema_validation/validate_integer: Collecting 100 samples in esschema_validation/validate_integer
                        time:   [2.9064 ns 3.2272 ns 3.6083 ns]
                        thrpt:  [277.14 Melem/s 309.87 Melem/s 344.07 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) high mild
  20 (20.00%) high severe
Benchmarking schema_validation/validate_object: Collecting 100 samples in estschema_validation/validate_object
                        time:   [58.413 ns 58.556 ns 58.703 ns]
                        thrpt:  [17.035 Melem/s 17.078 Melem/s 17.119 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in eschema_validation/validate_array_10
                        time:   [34.219 ns 34.292 ns 34.364 ns]
                        thrpt:  [29.101 Melem/s 29.161 Melem/s 29.223 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking schema_validation/validate_complex: Collecting 100 samples in esschema_validation/validate_complex
                        time:   [145.16 ns 145.41 ns 145.66 ns]
                        thrpt:  [6.8653 Melem/s 6.8773 Melem/s 6.8891 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking endpoint_matching/match_success: Collecting 100 samples in estimendpoint_matching/match_success
                        time:   [209.98 ns 210.20 ns 210.43 ns]
                        thrpt:  [4.7522 Melem/s 4.7574 Melem/s 4.7624 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking endpoint_matching/match_failure: Collecting 100 samples in estimendpoint_matching/match_failure
                        time:   [207.80 ns 208.02 ns 208.27 ns]
                        thrpt:  [4.8015 Melem/s 4.8071 Melem/s 4.8123 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in eendpoint_matching/match_multi_param
                        time:   [468.83 ns 469.38 ns 469.97 ns]
                        thrpt:  [2.1278 Melem/s 2.1305 Melem/s 2.1330 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking api_version/is_compatible_with: Collecting 100 samples in estimaapi_version/is_compatible_with
                        time:   [201.08 ps 201.26 ps 201.45 ps]
                        thrpt:  [4.9639 Gelem/s 4.9686 Gelem/s 4.9731 Gelem/s]
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) high mild
  4 (4.00%) high severe
Benchmarking api_version/parse: Collecting 100 samples in estimated 5.0002 s api_version/parse       time:   [40.288 ns 40.355 ns 40.418 ns]
                        thrpt:  [24.741 Melem/s 24.780 Melem/s 24.822 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking api_version/to_string: Collecting 100 samples in estimated 5.000api_version/to_string   time:   [52.256 ns 52.326 ns 52.401 ns]
                        thrpt:  [19.084 Melem/s 19.111 Melem/s 19.137 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

Benchmarking api_schema/create: Collecting 100 samples in estimated 5.0005 s api_schema/create       time:   [3.1543 µs 3.1575 µs 3.1608 µs]
                        thrpt:  [316.38 Kelem/s 316.71 Kelem/s 317.02 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking api_schema/serialize: Collecting 100 samples in estimated 5.0101api_schema/serialize    time:   [2.1710 µs 2.1732 µs 2.1755 µs]
                        thrpt:  [459.66 Kelem/s 460.15 Kelem/s 460.61 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking api_schema/deserialize: Collecting 100 samples in estimated 5.00api_schema/deserialize  time:   [9.4819 µs 9.4906 µs 9.4999 µs]
                        thrpt:  [105.26 Kelem/s 105.37 Kelem/s 105.46 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe
Benchmarking api_schema/find_endpoint: Collecting 100 samples in estimated 5.api_schema/find_endpoint
                        time:   [231.99 ns 232.29 ns 232.60 ns]
                        thrpt:  [4.2992 Melem/s 4.3049 Melem/s 4.3105 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_schema/endpoints_by_tag: Collecting 100 samples in estimatedapi_schema/endpoints_by_tag
                        time:   [99.034 ns 99.183 ns 99.333 ns]
                        thrpt:  [10.067 Melem/s 10.082 Melem/s 10.098 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking request_validation/validate_full_request: Warming up for 3.0000 Benchmarking request_validation/validate_full_request: Collecting 100 samplesrequest_validation/validate_full_request
                        time:   [54.832 ns 55.057 ns 55.282 ns]
                        thrpt:  [18.089 Melem/s 18.163 Melem/s 18.237 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking request_validation/validate_path_only: Collecting 100 samples inrequest_validation/validate_path_only
                        time:   [14.437 ns 14.629 ns 14.839 ns]
                        thrpt:  [67.389 Melem/s 68.358 Melem/s 69.265 Melem/s]

Benchmarking api_registry_basic/create: Collecting 100 samples in estimated 5api_registry_basic/create
                        time:   [1.2767 µs 1.2794 µs 1.2824 µs]
                        thrpt:  [779.78 Kelem/s 781.59 Kelem/s 783.24 Kelem/s]
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimapi_registry_basic/register_new
                        time:   [4.1307 µs 4.1457 µs 4.1615 µs]
                        thrpt:  [240.30 Kelem/s 241.22 Kelem/s 242.09 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
Benchmarking api_registry_basic/get: Collecting 100 samples in estimated 5.00api_registry_basic/get  time:   [23.510 ns 23.540 ns 23.572 ns]
                        thrpt:  [42.423 Melem/s 42.481 Melem/s 42.534 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_basic/len: Collecting 100 samples in estimated 5.00api_registry_basic/len  time:   [960.14 ns 961.47 ns 963.32 ns]
                        thrpt:  [1.0381 Melem/s 1.0401 Melem/s 1.0415 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 15.2s, or reduce sample count to 30.
Benchmarking api_registry_basic/stats: Collecting 100 samples in estimated 15api_registry_basic/stats
                        time:   [152.08 ms 152.61 ms 153.17 ms]
                        thrpt:  [6.5287  elem/s 6.5525  elem/s 6.5755  elem/s]

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estiapi_registry_query/query_by_name
                        time:   [86.365 µs 86.484 µs 86.619 µs]
                        thrpt:  [11.545 Kelem/s 11.563 Kelem/s 11.579 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_query/query_by_tag: Collecting 100 samples in estimapi_registry_query/query_by_tag
                        time:   [759.88 µs 761.24 µs 762.68 µs]
                        thrpt:  [1.3112 Kelem/s 1.3136 Kelem/s 1.3160 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_query/query_with_version: Collecting 100 samples inapi_registry_query/query_with_version
                        time:   [40.637 µs 40.760 µs 40.883 µs]
                        thrpt:  [24.460 Kelem/s 24.534 Kelem/s 24.608 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in eapi_registry_query/find_by_endpoint
                        time:   [3.7501 ms 3.7994 ms 3.8520 ms]
                        thrpt:  [259.61  elem/s 263.20  elem/s 266.66  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in esapi_registry_query/find_compatible
                        time:   [60.699 µs 60.784 µs 60.871 µs]
                        thrpt:  [16.428 Kelem/s 16.452 Kelem/s 16.475 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples api_registry_scaling/query_by_name/1000
                        time:   [7.6079 µs 7.6190 µs 7.6300 µs]
                        thrpt:  [131.06 Kelem/s 131.25 Kelem/s 131.44 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples iapi_registry_scaling/query_by_tag/1000
                        time:   [39.354 µs 39.399 µs 39.446 µs]
                        thrpt:  [25.351 Kelem/s 25.381 Kelem/s 25.411 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples api_registry_scaling/query_by_name/5000
                        time:   [38.274 µs 38.312 µs 38.350 µs]
                        thrpt:  [26.075 Kelem/s 26.101 Kelem/s 26.127 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples iapi_registry_scaling/query_by_tag/5000
                        time:   [275.63 µs 277.33 µs 279.15 µs]
                        thrpt:  [3.5823 Kelem/s 3.6058 Kelem/s 3.6280 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Warming up for 3.0000 Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samplesapi_registry_scaling/query_by_name/10000
                        time:   [85.465 µs 85.657 µs 85.873 µs]
                        thrpt:  [11.645 Kelem/s 11.674 Kelem/s 11.701 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples api_registry_scaling/query_by_tag/10000
                        time:   [742.56 µs 745.38 µs 747.94 µs]
                        thrpt:  [1.3370 Kelem/s 1.3416 Kelem/s 1.3467 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 sampleapi_registry_concurrent/concurrent_query/4
                        time:   [515.17 ms 522.20 ms 530.34 ms]
                        thrpt:  [3.7712 Kelem/s 3.8299 Kelem/s 3.8822 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 sampleapi_registry_concurrent/concurrent_mixed/4
                        time:   [429.89 ms 431.93 ms 433.74 ms]
                        thrpt:  [4.6111 Kelem/s 4.6304 Kelem/s 4.6523 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) low mild
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 11.8s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 sampleapi_registry_concurrent/concurrent_query/8
                        time:   [577.77 ms 586.66 ms 596.06 ms]
                        thrpt:  [6.7108 Kelem/s 6.8182 Kelem/s 6.9232 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.1s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 sampleapi_registry_concurrent/concurrent_mixed/8
                        time:   [496.41 ms 503.13 ms 510.20 ms]
                        thrpt:  [7.8401 Kelem/s 7.9503 Kelem/s 8.0578 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 17.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samplapi_registry_concurrent/concurrent_query/16
                        time:   [886.08 ms 891.24 ms 896.88 ms]
                        thrpt:  [8.9198 Kelem/s 8.9763 Kelem/s 9.0285 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 14.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samplapi_registry_concurrent/concurrent_mixed/16
                        time:   [716.10 ms 718.80 ms 721.45 ms]
                        thrpt:  [11.089 Kelem/s 11.130 Kelem/s 11.172 Kelem/s]

Benchmarking compare_op/eq: Collecting 100 samples in estimated 5.0000 s (3.6compare_op/eq           time:   [1.4051 ns 1.4068 ns 1.4087 ns]
                        thrpt:  [709.89 Melem/s 710.83 Melem/s 711.70 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking compare_op/gt: Collecting 100 samples in estimated 5.0000 s (5.5compare_op/gt           time:   [912.59 ps 913.53 ps 914.63 ps]
                        thrpt:  [1.0933 Gelem/s 1.0947 Gelem/s 1.0958 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe
Benchmarking compare_op/contains_string: Collecting 100 samples in estimated compare_op/contains_string
                        time:   [18.840 ns 18.896 ns 18.948 ns]
                        thrpt:  [52.776 Melem/s 52.922 Melem/s 53.078 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
Benchmarking compare_op/in_array: Collecting 100 samples in estimated 5.0000 compare_op/in_array     time:   [3.0413 ns 3.0491 ns 3.0606 ns]
                        thrpt:  [326.74 Melem/s 327.96 Melem/s 328.81 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe

Benchmarking condition/simple: Collecting 100 samples in estimated 5.0001 s (condition/simple        time:   [44.247 ns 44.298 ns 44.351 ns]
                        thrpt:  [22.547 Melem/s 22.574 Melem/s 22.600 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking condition/nested_field: Collecting 100 samples in estimated 5.00condition/nested_field  time:   [585.94 ns 586.66 ns 587.40 ns]
                        thrpt:  [1.7024 Melem/s 1.7046 Melem/s 1.7067 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking condition/string_eq: Collecting 100 samples in estimated 5.0001 condition/string_eq     time:   [73.621 ns 73.728 ns 73.848 ns]
                        thrpt:  [13.541 Melem/s 13.563 Melem/s 13.583 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe

Benchmarking condition_expr/single: Collecting 100 samples in estimated 5.000condition_expr/single   time:   [45.317 ns 45.376 ns 45.440 ns]
                        thrpt:  [22.007 Melem/s 22.038 Melem/s 22.067 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking condition_expr/and_2: Collecting 100 samples in estimated 5.0000condition_expr/and_2    time:   [96.482 ns 96.582 ns 96.689 ns]
                        thrpt:  [10.342 Melem/s 10.354 Melem/s 10.365 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking condition_expr/and_5: Collecting 100 samples in estimated 5.0009condition_expr/and_5    time:   [293.65 ns 294.01 ns 294.39 ns]
                        thrpt:  [3.3969 Melem/s 3.4013 Melem/s 3.4054 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking condition_expr/or_3: Collecting 100 samples in estimated 5.0006 condition_expr/or_3     time:   [168.51 ns 168.72 ns 168.94 ns]
                        thrpt:  [5.9194 Melem/s 5.9271 Melem/s 5.9343 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking condition_expr/nested: Collecting 100 samples in estimated 5.000condition_expr/nested   time:   [126.53 ns 126.70 ns 126.89 ns]
                        thrpt:  [7.8810 Melem/s 7.8926 Melem/s 7.9035 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

Benchmarking rule/create: Collecting 100 samples in estimated 5.0008 s (13M irule/create             time:   [389.52 ns 390.24 ns 391.05 ns]
                        thrpt:  [2.5572 Melem/s 2.5625 Melem/s 2.5673 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rule/matches: Collecting 100 samples in estimated 5.0002 s (53M rule/matches            time:   [94.774 ns 94.885 ns 95.006 ns]
                        thrpt:  [10.526 Melem/s 10.539 Melem/s 10.551 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking rule_context/create: Collecting 100 samples in estimated 5.0074 rule_context/create     time:   [1.7604 µs 1.7627 µs 1.7653 µs]
                        thrpt:  [566.48 Kelem/s 567.30 Kelem/s 568.06 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking rule_context/get_simple: Collecting 100 samples in estimated 5.0rule_context/get_simple time:   [43.361 ns 43.425 ns 43.493 ns]
                        thrpt:  [22.992 Melem/s 23.028 Melem/s 23.062 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_context/get_nested: Collecting 100 samples in estimated 5.0rule_context/get_nested time:   [1.3942 µs 1.3960 µs 1.3982 µs]
                        thrpt:  [715.23 Kelem/s 716.33 Kelem/s 717.28 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking rule_context/get_deep_nested: Collecting 100 samples in estimaterule_context/get_deep_nested
                        time:   [591.37 ns 592.44 ns 593.54 ns]
                        thrpt:  [1.6848 Melem/s 1.6879 Melem/s 1.6910 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

Benchmarking rule_engine_basic/create: Collecting 100 samples in estimated 5.rule_engine_basic/create
                        time:   [11.800 ns 11.840 ns 11.884 ns]
                        thrpt:  [84.145 Melem/s 84.460 Melem/s 84.747 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  18 (18.00%) high mild
Benchmarking rule_engine_basic/add_rule: Collecting 100 samples in estimated rule_engine_basic/add_rule
                        time:   [1.6420 µs 1.6947 µs 1.7382 µs]
                        thrpt:  [575.30 Kelem/s 590.07 Kelem/s 609.01 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_engine_basic/get_rule: Collecting 100 samples in estimated rule_engine_basic/get_rule
                        time:   [16.263 ns 16.279 ns 16.297 ns]
                        thrpt:  [61.359 Melem/s 61.428 Melem/s 61.488 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  9 (9.00%) high severe
Benchmarking rule_engine_basic/rules_by_tag: Collecting 100 samples in estimarule_engine_basic/rules_by_tag
                        time:   [971.06 ns 972.93 ns 974.81 ns]
                        thrpt:  [1.0258 Melem/s 1.0278 Melem/s 1.0298 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_engine_basic/stats: Collecting 100 samples in estimated 5.0rule_engine_basic/stats time:   [7.7200 µs 7.7484 µs 7.7722 µs]
                        thrpt:  [128.66 Kelem/s 129.06 Kelem/s 129.53 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples irule_engine_evaluate/evaluate_10_rules
                        time:   [2.8938 µs 2.8981 µs 2.9026 µs]
                        thrpt:  [344.52 Kelem/s 345.06 Kelem/s 345.56 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  15 (15.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Warming up for 3.0Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samrule_engine_evaluate/evaluate_first_10_rules
                        time:   [316.95 ns 317.31 ns 317.69 ns]
                        thrpt:  [3.1477 Melem/s 3.1515 Melem/s 3.1551 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  14 (14.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples rule_engine_evaluate/evaluate_100_rules
                        time:   [33.796 µs 33.844 µs 33.897 µs]
                        thrpt:  [29.501 Kelem/s 29.547 Kelem/s 29.589 Kelem/s]
Found 17 outliers among 100 measurements (17.00%)
  16 (16.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Warming up for 3.Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 sarule_engine_evaluate/evaluate_first_100_rules
                        time:   [319.77 ns 320.10 ns 320.45 ns]
                        thrpt:  [3.1207 Melem/s 3.1241 Melem/s 3.1273 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Warming up forBenchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100rule_engine_evaluate/evaluate_matching_100_rules
                        time:   [42.120 µs 42.230 µs 42.348 µs]
                        thrpt:  [23.614 Kelem/s 23.680 Kelem/s 23.741 Kelem/s]
Found 20 outliers among 100 measurements (20.00%)
  18 (18.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Warming up for 3.0000 Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samplesrule_engine_evaluate/evaluate_1000_rules
                        time:   [300.78 µs 301.29 µs 301.83 µs]
                        thrpt:  [3.3131 Kelem/s 3.3190 Kelem/s 3.3247 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Warming up for 3Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 srule_engine_evaluate/evaluate_first_1000_rules
                        time:   [317.63 ns 318.20 ns 319.01 ns]
                        thrpt:  [3.1347 Melem/s 3.1427 Melem/s 3.1484 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimrule_engine_scaling/evaluate/10
                        time:   [3.7417 µs 3.7457 µs 3.7500 µs]
                        thrpt:  [266.67 Kelem/s 266.97 Kelem/s 267.26 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/10: Collecting 100 samples inrule_engine_scaling/evaluate_first/10
                        time:   [318.16 ns 318.44 ns 318.75 ns]
                        thrpt:  [3.1373 Melem/s 3.1403 Melem/s 3.1431 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/50: Collecting 100 samples in estimrule_engine_scaling/evaluate/50
                        time:   [19.539 µs 19.563 µs 19.587 µs]
                        thrpt:  [51.055 Kelem/s 51.118 Kelem/s 51.179 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/50: Collecting 100 samples inrule_engine_scaling/evaluate_first/50
                        time:   [316.99 ns 317.34 ns 317.71 ns]
                        thrpt:  [3.1475 Melem/s 3.1512 Melem/s 3.1547 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low severe
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_scaling/evaluate/100: Collecting 100 samples in estirule_engine_scaling/evaluate/100
                        time:   [33.253 µs 33.300 µs 33.353 µs]
                        thrpt:  [29.982 Kelem/s 30.030 Kelem/s 30.073 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/100: Collecting 100 samples irule_engine_scaling/evaluate_first/100
                        time:   [318.65 ns 318.97 ns 319.29 ns]
                        thrpt:  [3.1319 Melem/s 3.1351 Melem/s 3.1382 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate/500: Collecting 100 samples in estirule_engine_scaling/evaluate/500
                        time:   [151.43 µs 151.60 µs 151.78 µs]
                        thrpt:  [6.5885 Kelem/s 6.5963 Kelem/s 6.6037 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/500: Collecting 100 samples irule_engine_scaling/evaluate_first/500
                        time:   [319.57 ns 319.87 ns 320.20 ns]
                        thrpt:  [3.1231 Melem/s 3.1262 Melem/s 3.1292 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/1000: Collecting 100 samples in estrule_engine_scaling/evaluate/1000
                        time:   [298.75 µs 299.03 µs 299.33 µs]
                        thrpt:  [3.3407 Kelem/s 3.3442 Kelem/s 3.3473 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/1000: Collecting 100 samples rule_engine_scaling/evaluate_first/1000
                        time:   [315.37 ns 315.81 ns 316.22 ns]
                        thrpt:  [3.1623 Melem/s 3.1665 Melem/s 3.1709 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking rule_set/create: Collecting 100 samples in estimated 5.0138 s (6rule_set/create         time:   [7.6781 µs 7.6869 µs 7.6968 µs]
                        thrpt:  [129.92 Kelem/s 130.09 Kelem/s 130.24 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_set/load_into_engine: Collecting 100 samples in estimated 5rule_set/load_into_engine
                        time:   [10.413 µs 10.428 µs 10.444 µs]
                        thrpt:  [95.748 Kelem/s 95.898 Kelem/s 96.030 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking trace_id/generate: Collecting 100 samples in estimated 5.0000 s trace_id/generate       time:   [47.585 ns 47.821 ns 48.067 ns]
                        thrpt:  [20.804 Melem/s 20.911 Melem/s 21.015 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking trace_id/to_hex: Collecting 100 samples in estimated 5.0004 s (5trace_id/to_hex         time:   [89.998 ns 90.114 ns 90.241 ns]
                        thrpt:  [11.081 Melem/s 11.097 Melem/s 11.111 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking trace_id/from_hex: Collecting 100 samples in estimated 5.0000 s trace_id/from_hex       time:   [18.211 ns 18.279 ns 18.374 ns]
                        thrpt:  [54.425 Melem/s 54.708 Melem/s 54.911 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe

Benchmarking context_operations/create: Collecting 100 samples in estimated 5context_operations/create
                        time:   [80.784 ns 80.982 ns 81.203 ns]
                        thrpt:  [12.315 Melem/s 12.348 Melem/s 12.379 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking context_operations/child: Collecting 100 samples in estimated 5.context_operations/child
                        time:   [36.705 ns 36.834 ns 36.960 ns]
                        thrpt:  [27.057 Melem/s 27.149 Melem/s 27.245 Melem/s]
Benchmarking context_operations/for_remote: Collecting 100 samples in estimatcontext_operations/for_remote
                        time:   [36.677 ns 36.810 ns 36.939 ns]
                        thrpt:  [27.072 Melem/s 27.166 Melem/s 27.265 Melem/s]
Benchmarking context_operations/to_traceparent: Collecting 100 samples in estcontext_operations/to_traceparent
                        time:   [257.87 ns 258.12 ns 258.37 ns]
                        thrpt:  [3.8704 Melem/s 3.8742 Melem/s 3.8780 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking context_operations/from_traceparent: Collecting 100 samples in econtext_operations/from_traceparent
                        time:   [115.83 ns 115.98 ns 116.14 ns]
                        thrpt:  [8.6100 Melem/s 8.6223 Melem/s 8.6334 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking baggage/create: Collecting 100 samples in estimated 5.0000 s (1.baggage/create          time:   [4.7679 ns 4.7796 ns 4.7897 ns]
                        thrpt:  [208.78 Melem/s 209.22 Melem/s 209.74 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
Benchmarking baggage/get: Collecting 100 samples in estimated 5.0000 s (623M baggage/get             time:   [8.0003 ns 8.0121 ns 8.0261 ns]
                        thrpt:  [124.59 Melem/s 124.81 Melem/s 125.00 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  2 (2.00%) high mild
  15 (15.00%) high severe
Benchmarking baggage/set: Collecting 100 samples in estimated 5.0001 s (82M ibaggage/set             time:   [61.190 ns 61.304 ns 61.419 ns]
                        thrpt:  [16.282 Melem/s 16.312 Melem/s 16.342 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking baggage/merge: Collecting 100 samples in estimated 5.0074 s (2.8baggage/merge           time:   [1.7722 µs 1.7740 µs 1.7760 µs]
                        thrpt:  [563.05 Kelem/s 563.70 Kelem/s 564.28 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

Benchmarking span/create: Collecting 100 samples in estimated 5.0004 s (60M ispan/create             time:   [83.735 ns 83.949 ns 84.233 ns]
                        thrpt:  [11.872 Melem/s 11.912 Melem/s 11.942 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking span/set_attribute: Collecting 100 samples in estimated 5.0002 sspan/set_attribute      time:   [57.014 ns 57.109 ns 57.212 ns]
                        thrpt:  [17.479 Melem/s 17.510 Melem/s 17.540 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking span/add_event: Collecting 100 samples in estimated 5.0003 s (65span/add_event          time:   [63.118 ns 63.429 ns 63.909 ns]
                        thrpt:  [15.647 Melem/s 15.766 Melem/s 15.843 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking span/with_kind: Collecting 100 samples in estimated 5.0002 s (60span/with_kind          time:   [83.472 ns 83.591 ns 83.725 ns]
                        thrpt:  [11.944 Melem/s 11.963 Melem/s 11.980 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking context_store/create_context: Collecting 100 samples in estimatecontext_store/create_context
                        time:   [95.030 µs 95.185 µs 95.358 µs]
                        thrpt:  [10.487 Kelem/s 10.506 Kelem/s 10.523 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe

thread 'main' (27172) panicked at benches\net.rs:4552:45:
called `Result::unwrap()` on an `Err` value: CapacityExceeded
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

error: bench failed, to rerun pass `--bench net`
PS C:\Users\chief\Desktop\github\cyberdeck\net\crates\net>
