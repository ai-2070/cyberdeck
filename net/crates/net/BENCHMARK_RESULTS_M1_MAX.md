     Running benches/auth_guard.rs (target/release/deps/auth_guard-3b26338e5efa800d)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: WarminBenchmarking auth_guard_check_fast_hit/single_thread: CollecBenchmarking auth_guard_check_fast_hit/single_thread: Analyzauth_guard_check_fast_hit/single_thread
                        time:   [20.273 ns 20.287 ns 20.304 ns]
                        thrpt:  [49.251 Melem/s 49.292 Melem/s 49.327 Melem/s]
                 change:
                        time:   [+0.4781% +0.8158% +1.1925%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1784% −0.8092% −0.4758%]
                        Change within noise threshold.
Found 6 outliers among 50 measurements (12.00%)
  4 (8.00%) high mild
  2 (4.00%) high severe

auth_guard_check_fast_miss/single_thread
                        time:   [4.0335 ns 4.0400 ns 4.0474 ns]
                        thrpt:  [247.07 Melem/s 247.53 Melem/s 247.93 Melem/s]
                 change:
                        time:   [−1.4801% −1.1891% −0.9119%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9203% +1.2035% +1.5023%]
                        Change within noise threshold.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

auth_guard_check_fast_contended/eight_threads
                        time:   [30.561 ns 30.712 ns 30.859 ns]
                        thrpt:  [32.406 Melem/s 32.561 Melem/s 32.721 Melem/s]
                 change:
                        time:   [−6.5268% −5.2862% −3.5210%] (p = 0.00 < 0.05)
                        thrpt:  [+3.6495% +5.5813% +6.9825%]
                        Performance has improved.
Found 7 outliers among 50 measurements (14.00%)
  1 (2.00%) low severe
  3 (6.00%) low mild
  1 (2.00%) high mild
  2 (4.00%) high severe

auth_guard_allow_channel/insert
                        time:   [177.11 ns 187.74 ns 196.81 ns]
                        thrpt:  [5.0811 Melem/s 5.3266 Melem/s 5.6463 Melem/s]
                 change:
                        time:   [−13.708% −8.1976% −2.3431%] (p = 0.01 < 0.05)
                        thrpt:  [+2.3993% +8.9296% +15.886%]
                        Performance has improved.

auth_guard_hot_hit_ceiling/million_ops
                        time:   [2.9881 ms 2.9893 ms 2.9905 ms]
                        change: [−0.4611% −0.2591% −0.0336%] (p = 0.01 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 50 measurements (4.00%)
  1 (2.00%) high mild
  1 (2.00%) high severe

     Running benches/cortex.rs (target/release/deps/cortex-87bd118533c67b55)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [112.26 ns 112.75 ns 113.32 ns]
                        thrpt:  [8.8249 Melem/s 8.8688 Melem/s 8.9078 Melem/s]
                 change:
                        time:   [−62.149% −60.156% −58.171%] (p = 0.00 < 0.05)
                        thrpt:  [+139.07% +150.98% +164.19%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
cortex_ingest/memories_store
                        time:   [217.13 ns 218.25 ns 219.53 ns]
                        thrpt:  [4.5553 Melem/s 4.5819 Melem/s 4.6055 Melem/s]
                 change:
                        time:   [−47.264% −44.890% −42.458%] (p = 0.00 < 0.05)
                        thrpt:  [+73.786% +81.456% +89.624%]
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe

cortex_fold_barrier/tasks_create_and_wait
                        time:   [5.5787 µs 5.5931 µs 5.6090 µs]
                        thrpt:  [178.29 Kelem/s 178.79 Kelem/s 179.25 Kelem/s]
                 change:
                        time:   [−0.8481% −0.3900% +0.0414%] (p = 0.10 > 0.05)
                        thrpt:  [−0.0414% +0.3915% +0.8553%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
cortex_fold_barrier/memories_store_and_wait
                        time:   [5.7467 µs 5.7658 µs 5.7866 µs]
                        thrpt:  [172.81 Kelem/s 173.44 Kelem/s 174.01 Kelem/s]
                 change:
                        time:   [−0.8758% −0.4010% +0.0965%] (p = 0.12 > 0.05)
                        thrpt:  [−0.0964% +0.4026% +0.8836%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

cortex_query/tasks_find_many/100
                        time:   [989.05 ns 991.79 ns 994.59 ns]
                        thrpt:  [100.54 Melem/s 100.83 Melem/s 101.11 Melem/s]
                 change:
                        time:   [+0.4516% +0.8008% +1.1095%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0973% −0.7945% −0.4496%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
cortex_query/tasks_count_where/100
                        time:   [152.23 ns 152.32 ns 152.42 ns]
                        thrpt:  [656.08 Melem/s 656.51 Melem/s 656.91 Melem/s]
                 change:
                        time:   [−0.8773% −0.6754% −0.5028%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5054% +0.6800% +0.8850%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
cortex_query/tasks_find_unique/100
                        time:   [8.9591 ns 8.9921 ns 9.0287 ns]
                        thrpt:  [11.076 Gelem/s 11.121 Gelem/s 11.162 Gelem/s]
                 change:
                        time:   [−0.5364% −0.0443% +0.4382%] (p = 0.86 > 0.05)
                        thrpt:  [−0.4363% +0.0443% +0.5393%]
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
cortex_query/memories_find_many_tag/100
                        time:   [4.5973 µs 4.6041 µs 4.6108 µs]
                        thrpt:  [21.688 Melem/s 21.720 Melem/s 21.752 Melem/s]
                 change:
                        time:   [−0.6543% −0.4559% −0.2616%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2623% +0.4580% +0.6586%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
cortex_query/memories_count_where/100
                        time:   [893.63 ns 896.26 ns 898.80 ns]
                        thrpt:  [111.26 Melem/s 111.57 Melem/s 111.90 Melem/s]
                 change:
                        time:   [−1.0045% −0.3673% +0.2711%] (p = 0.26 > 0.05)
                        thrpt:  [−0.2703% +0.3687% +1.0147%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
cortex_query/tasks_find_many/1000
                        time:   [7.5922 µs 7.6140 µs 7.6367 µs]
                        thrpt:  [130.95 Melem/s 131.34 Melem/s 131.71 Melem/s]
                 change:
                        time:   [+1.5613% +2.0279% +2.4957%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4349% −1.9876% −1.5373%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
cortex_query/tasks_count_where/1000
                        time:   [1.5239 µs 1.5249 µs 1.5260 µs]
                        thrpt:  [655.31 Melem/s 655.77 Melem/s 656.20 Melem/s]
                 change:
                        time:   [−1.7502% −1.3907% −1.0535%] (p = 0.00 < 0.05)
                        thrpt:  [+1.0648% +1.4103% +1.7814%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
cortex_query/tasks_find_unique/1000
                        time:   [8.9263 ns 8.9482 ns 8.9745 ns]
                        thrpt:  [111.43 Gelem/s 111.75 Gelem/s 112.03 Gelem/s]
                 change:
                        time:   [−0.5371% −0.0006% +0.5669%] (p = 1.00 > 0.05)
                        thrpt:  [−0.5637% +0.0006% +0.5400%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) high mild
  9 (9.00%) high severe
cortex_query/memories_find_many_tag/1000
                        time:   [49.313 µs 49.368 µs 49.422 µs]
                        thrpt:  [20.234 Melem/s 20.256 Melem/s 20.279 Melem/s]
                 change:
                        time:   [+0.6529% +0.8539% +1.0389%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0282% −0.8467% −0.6487%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
cortex_query/memories_count_where/1000
                        time:   [10.819 µs 10.846 µs 10.873 µs]
                        thrpt:  [91.969 Melem/s 92.204 Melem/s 92.432 Melem/s]
                 change:
                        time:   [+3.5893% +3.9330% +4.2773%] (p = 0.00 < 0.05)
                        thrpt:  [−4.1018% −3.7842% −3.4649%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
cortex_query/tasks_find_many/10000
                        time:   [116.26 µs 124.76 µs 134.06 µs]
                        thrpt:  [74.596 Melem/s 80.156 Melem/s 86.016 Melem/s]
                 change:
                        time:   [+7.7606% +14.958% +23.169%] (p = 0.00 < 0.05)
                        thrpt:  [−18.811% −13.012% −7.2017%]
                        Performance has regressed.
cortex_query/tasks_count_where/10000
                        time:   [6.6437 µs 6.6716 µs 6.7021 µs]
                        thrpt:  [1.4921 Gelem/s 1.4989 Gelem/s 1.5052 Gelem/s]
                 change:
                        time:   [−82.406% −82.174% −81.933%] (p = 0.00 < 0.05)
                        thrpt:  [+453.50% +460.99% +468.37%]
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
cortex_query/tasks_find_unique/10000
                        time:   [8.9412 ns 8.9721 ns 9.0071 ns]
                        thrpt:  [1110.2 Gelem/s 1114.6 Gelem/s 1118.4 Gelem/s]
                 change:
                        time:   [−0.9696% −0.5001% −0.0737%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0737% +0.5026% +0.9791%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe
cortex_query/memories_find_many_tag/10000
                        time:   [99.351 µs 107.02 µs 115.28 µs]
                        thrpt:  [86.745 Melem/s 93.444 Melem/s 100.65 Melem/s]
                 change:
                        time:   [−82.544% −81.155% −79.724%] (p = 0.00 < 0.05)
                        thrpt:  [+393.19% +430.63% +472.87%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
cortex_query/memories_count_where/10000
                        time:   [16.761 µs 16.788 µs 16.814 µs]
                        thrpt:  [594.75 Melem/s 595.68 Melem/s 596.64 Melem/s]
                 change:
                        time:   [−87.978% −87.950% −87.920%] (p = 0.00 < 0.05)
                        thrpt:  [+727.78% +729.85% +731.79%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

cortex_snapshot/tasks_encode/100
                        time:   [3.1103 µs 3.1135 µs 3.1169 µs]
                        thrpt:  [32.084 Melem/s 32.118 Melem/s 32.152 Melem/s]
                 change:
                        time:   [−0.4477% −0.2629% −0.0606%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0606% +0.2636% +0.4497%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
cortex_snapshot/memories_encode/100
                        time:   [5.3844 µs 5.3918 µs 5.3998 µs]
                        thrpt:  [18.519 Melem/s 18.547 Melem/s 18.572 Melem/s]
                 change:
                        time:   [−0.7797% −0.5354% −0.2855%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2863% +0.5382% +0.7858%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5.0081 s (1.7M iteratcortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.9370 µs 2.9416 µs 2.9462 µs]
                        thrpt:  [33.942 Melem/s 33.995 Melem/s 34.048 Melem/s]
                 change:
                        time:   [+36.419% +36.819% +37.241%] (p = 0.00 < 0.05)
                        thrpt:  [−27.135% −26.911% −26.697%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
cortex_snapshot/netdb_bundle_decode/100
                        time:   [2.2920 µs 2.3086 µs 2.3252 µs]
                        thrpt:  [43.008 Melem/s 43.316 Melem/s 43.630 Melem/s]
                 change:
                        time:   [+0.3837% +0.8851% +1.4174%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3976% −0.8773% −0.3822%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild
cortex_snapshot/tasks_encode/1000
                        time:   [30.630 µs 30.695 µs 30.762 µs]
                        thrpt:  [32.507 Melem/s 32.579 Melem/s 32.648 Melem/s]
                 change:
                        time:   [−1.5911% −1.3658% −1.1433%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1565% +1.3847% +1.6168%]
                        Performance has improved.
cortex_snapshot/memories_encode/1000
                        time:   [56.568 µs 56.660 µs 56.770 µs]
                        thrpt:  [17.615 Melem/s 17.649 Melem/s 17.678 Melem/s]
                 change:
                        time:   [−2.1691% −1.8747% −1.6038%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6300% +1.9105% +2.2172%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimated 5.1418 s (162k itercortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [31.719 µs 31.774 µs 31.830 µs]
                        thrpt:  [31.417 Melem/s 31.473 Melem/s 31.527 Melem/s]
                 change:
                        time:   [+42.793% +43.125% +43.461%] (p = 0.00 < 0.05)
                        thrpt:  [−30.295% −30.131% −29.968%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
cortex_snapshot/netdb_bundle_decode/1000
                        time:   [26.470 µs 26.500 µs 26.538 µs]
                        thrpt:  [37.682 Melem/s 37.735 Melem/s 37.779 Melem/s]
                 change:
                        time:   [+0.2882% +0.5018% +0.7320%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7267% −0.4993% −0.2874%]
                        Change within noise threshold.
cortex_snapshot/tasks_encode/10000
                        time:   [81.978 µs 83.170 µs 84.469 µs]
                        thrpt:  [118.39 Melem/s 120.24 Melem/s 121.98 Melem/s]
                 change:
                        time:   [−75.593% −74.953% −74.348%] (p = 0.00 < 0.05)
                        thrpt:  [+289.83% +299.25% +309.72%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
cortex_snapshot/memories_encode/10000
                        time:   [679.48 µs 696.68 µs 714.46 µs]
                        thrpt:  [13.997 Melem/s 14.354 Melem/s 14.717 Melem/s]
                 change:
                        time:   [−2.9275% +0.0967% +3.3868%] (p = 0.95 > 0.05)
                        thrpt:  [−3.2758% −0.0966% +3.0158%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_371022/10000: Collecting 100 samples in estimated 6.5443 s (20k itecortex_snapshot/netdb_bundle_encode_bytes_371022/10000
                        time:   [302.87 µs 316.66 µs 330.85 µs]
                        thrpt:  [30.225 Melem/s 31.580 Melem/s 33.018 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
cortex_snapshot/netdb_bundle_decode/10000
                        time:   [202.67 µs 202.92 µs 203.20 µs]
                        thrpt:  [49.214 Melem/s 49.280 Melem/s 49.341 Melem/s]
                 change:
                        time:   [−27.955% −27.797% −27.647%] (p = 0.00 < 0.05)
                        thrpt:  [+38.211% +38.499% +38.803%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

     Running benches/ingestion.rs (target/release/deps/ingestion-0caf80d2a1d93407)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [10.839 ns 10.851 ns 10.863 ns]
                        thrpt:  [92.057 Melem/s 92.158 Melem/s 92.261 Melem/s]
                 change:
                        time:   [−0.4199% −0.0863% +0.2134%] (p = 0.62 > 0.05)
                        thrpt:  [−0.2129% +0.0864% +0.4217%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
ring_buffer/push_pop/1024
                        time:   [10.375 ns 10.387 ns 10.405 ns]
                        thrpt:  [96.103 Melem/s 96.273 Melem/s 96.388 Melem/s]
                 change:
                        time:   [−0.2767% −0.1016% +0.1004%] (p = 0.27 > 0.05)
                        thrpt:  [−0.1003% +0.1017% +0.2775%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
ring_buffer/push/8192   time:   [10.835 ns 10.846 ns 10.857 ns]
                        thrpt:  [92.107 Melem/s 92.199 Melem/s 92.291 Melem/s]
                 change:
                        time:   [−0.3930% −0.1461% +0.1074%] (p = 0.27 > 0.05)
                        thrpt:  [−0.1073% +0.1463% +0.3946%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
ring_buffer/push_pop/8192
                        time:   [10.453 ns 10.473 ns 10.496 ns]
                        thrpt:  [95.274 Melem/s 95.481 Melem/s 95.668 Melem/s]
                 change:
                        time:   [+0.2714% +0.5173% +0.7563%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7507% −0.5147% −0.2707%]
                        Change within noise threshold.
ring_buffer/push/65536  time:   [10.798 ns 10.810 ns 10.820 ns]
                        thrpt:  [92.421 Melem/s 92.510 Melem/s 92.610 Melem/s]
                 change:
                        time:   [−1.1603% −0.2470% +0.6556%] (p = 0.60 > 0.05)
                        thrpt:  [−0.6513% +0.2476% +1.1740%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) low severe
  4 (4.00%) low mild
ring_buffer/push_pop/65536
                        time:   [10.385 ns 10.392 ns 10.400 ns]
                        thrpt:  [96.157 Melem/s 96.227 Melem/s 96.294 Melem/s]
                 change:
                        time:   [−0.7281% −0.5649% −0.4149%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4166% +0.5681% +0.7334%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
ring_buffer/push/1048576
                        time:   [10.189 ns 10.246 ns 10.291 ns]
                        thrpt:  [97.172 Melem/s 97.600 Melem/s 98.146 Melem/s]
                 change:
                        time:   [−3.7186% −0.4601% +2.8804%] (p = 0.78 > 0.05)
                        thrpt:  [−2.7997% +0.4622% +3.8622%]
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  13 (13.00%) low mild
ring_buffer/push_pop/1048576
                        time:   [10.528 ns 10.546 ns 10.567 ns]
                        thrpt:  [94.636 Melem/s 94.827 Melem/s 94.983 Melem/s]
                 change:
                        time:   [−1.1618% −0.4643% +0.1121%] (p = 0.17 > 0.05)
                        thrpt:  [−0.1120% +0.4664% +1.1754%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

timestamp/next          time:   [7.4991 ns 7.5042 ns 7.5091 ns]
                        thrpt:  [133.17 Melem/s 133.26 Melem/s 133.35 Melem/s]
                 change:
                        time:   [−0.3153% −0.1338% +0.0292%] (p = 0.14 > 0.05)
                        thrpt:  [−0.0292% +0.1339% +0.3163%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
timestamp/now_raw       time:   [623.31 ps 623.62 ps 623.91 ps]
                        thrpt:  [1.6028 Gelem/s 1.6036 Gelem/s 1.6043 Gelem/s]
                 change:
                        time:   [−0.2143% −0.0629% +0.0928%] (p = 0.42 > 0.05)
                        thrpt:  [−0.0927% +0.0629% +0.2147%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

event/internal_event_new
                        time:   [176.66 ns 177.31 ns 177.95 ns]
                        thrpt:  [5.6194 Melem/s 5.6398 Melem/s 5.6607 Melem/s]
                 change:
                        time:   [−0.7273% −0.2765% +0.1658%] (p = 0.22 > 0.05)
                        thrpt:  [−0.1655% +0.2772% +0.7327%]
                        No change in performance detected.
event/json_creation     time:   [106.57 ns 106.78 ns 107.00 ns]
                        thrpt:  [9.3460 Melem/s 9.3647 Melem/s 9.3831 Melem/s]
                 change:
                        time:   [+0.8218% +1.1100% +1.4042%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3848% −1.0978% −0.8151%]
                        Change within noise threshold.

batch/pop_batch_steady_state/100
                        time:   [777.01 ns 777.73 ns 778.46 ns]
                        thrpt:  [128.46 Melem/s 128.58 Melem/s 128.70 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
batch/pop_batch_steady_state/1000
                        time:   [38.694 µs 39.968 µs 41.264 µs]
                        thrpt:  [24.234 Melem/s 25.020 Melem/s 25.844 Melem/s]
batch/pop_batch_steady_state/10000
                        time:   [635.22 µs 672.23 µs 709.57 µs]
                        thrpt:  [14.093 Melem/s 14.876 Melem/s 15.743 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

     Running benches/mesh.rs (target/release/deps/mesh-e0bd8a73608dc5f1)
Gnuplot not found, using plotters backend
mesh_reroute/triangle_failure
                        time:   [5.0017 µs 5.0632 µs 5.1263 µs]
                        thrpt:  [195.07 Kelem/s 197.50 Kelem/s 199.93 Kelem/s]
                 change:
                        time:   [−1.5009% −0.2176% +1.1918%] (p = 0.75 > 0.05)
                        thrpt:  [−1.1778% +0.2181% +1.5238%]
                        No change in performance detected.
mesh_reroute/10_peers_10_routes
                        time:   [28.328 µs 28.561 µs 28.814 µs]
                        thrpt:  [34.705 Kelem/s 35.013 Kelem/s 35.301 Kelem/s]
                 change:
                        time:   [−0.7237% +0.1173% +1.0429%] (p = 0.80 > 0.05)
                        thrpt:  [−1.0322% −0.1172% +0.7289%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
mesh_reroute/50_peers_100_routes
                        time:   [312.11 µs 312.62 µs 313.18 µs]
                        thrpt:  [3.1931 Kelem/s 3.1988 Kelem/s 3.2040 Kelem/s]
                 change:
                        time:   [−1.0004% −0.6604% −0.3181%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3191% +0.6647% +1.0105%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

mesh_proximity/on_pingwave_new
                        time:   [174.22 ns 178.63 ns 183.38 ns]
                        thrpt:  [5.4531 Melem/s 5.5980 Melem/s 5.7398 Melem/s]
                 change:
                        time:   [+0.4019% +6.1837% +12.466%] (p = 0.04 < 0.05)
                        thrpt:  [−11.084% −5.8236% −0.4003%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
mesh_proximity/on_pingwave_dedup
                        time:   [68.427 ns 68.568 ns 68.733 ns]
                        thrpt:  [14.549 Melem/s 14.584 Melem/s 14.614 Melem/s]
                 change:
                        time:   [−0.0914% +0.1394% +0.4504%] (p = 0.32 > 0.05)
                        thrpt:  [−0.4484% −0.1392% +0.0915%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
mesh_proximity/pingwave_serialize
                        time:   [1.9775 ns 1.9786 ns 1.9798 ns]
                        thrpt:  [505.11 Melem/s 505.40 Melem/s 505.70 Melem/s]
                 change:
                        time:   [−0.5049% −0.2701% −0.0508%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0509% +0.2709% +0.5075%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
mesh_proximity/pingwave_deserialize
                        time:   [1.8700 ns 1.8709 ns 1.8719 ns]
                        thrpt:  [534.22 Melem/s 534.50 Melem/s 534.77 Melem/s]
                 change:
                        time:   [−0.2616% −0.1156% +0.0333%] (p = 0.13 > 0.05)
                        thrpt:  [−0.0333% +0.1158% +0.2623%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
mesh_proximity/node_count
                        time:   [200.15 ns 200.25 ns 200.35 ns]
                        thrpt:  [4.9912 Melem/s 4.9938 Melem/s 4.9963 Melem/s]
                 change:
                        time:   [−0.3406% −0.1791% −0.0296%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0296% +0.1794% +0.3418%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 87.2s, or reduce sample count to 10.
mesh_proximity/all_nodes_100
                        time:   [853.80 ms 854.91 ms 856.00 ms]
                        thrpt:  [1.1682  elem/s 1.1697  elem/s 1.1712  elem/s]
                 change:
                        time:   [+98.826% +99.210% +99.608%] (p = 0.00 < 0.05)
                        thrpt:  [−49.902% −49.802% −49.705%]
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) low mild
  3 (3.00%) high mild

mesh_dispatch/classify_direct
                        time:   [623.62 ps 623.99 ps 624.36 ps]
                        thrpt:  [1.6016 Gelem/s 1.6026 Gelem/s 1.6035 Gelem/s]
                 change:
                        time:   [−0.4842% −0.2049% +0.0152%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0152% +0.2053% +0.4865%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
mesh_dispatch/classify_routed
                        time:   [442.96 ps 443.14 ps 443.34 ps]
                        thrpt:  [2.2556 Gelem/s 2.2566 Gelem/s 2.2576 Gelem/s]
                 change:
                        time:   [−0.8675% −0.5706% −0.3435%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3447% +0.5738% +0.8751%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
mesh_dispatch/classify_pingwave
                        time:   [311.69 ps 311.94 ps 312.20 ps]
                        thrpt:  [3.2031 Gelem/s 3.2058 Gelem/s 3.2083 Gelem/s]
                 change:
                        time:   [−1.3321% −0.9889% −0.6539%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6582% +0.9988% +1.3501%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

mesh_routing/lookup_hit time:   [14.957 ns 14.988 ns 15.016 ns]
                        thrpt:  [66.598 Melem/s 66.720 Melem/s 66.859 Melem/s]
                 change:
                        time:   [+0.9485% +1.7068% +2.6366%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5689% −1.6781% −0.9396%]
                        Change within noise threshold.
Found 17 outliers among 100 measurements (17.00%)
  5 (5.00%) low severe
  4 (4.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
mesh_routing/lookup_miss
                        time:   [14.854 ns 14.916 ns 14.971 ns]
                        thrpt:  [66.797 Melem/s 67.044 Melem/s 67.322 Melem/s]
                 change:
                        time:   [−0.7636% −0.0780% +0.6402%] (p = 0.83 > 0.05)
                        thrpt:  [−0.6361% +0.0781% +0.7695%]
                        No change in performance detected.
Found 28 outliers among 100 measurements (28.00%)
  12 (12.00%) low severe
  5 (5.00%) low mild
  11 (11.00%) high mild
mesh_routing/is_local   time:   [311.78 ps 312.14 ps 312.56 ps]
                        thrpt:  [3.1994 Gelem/s 3.2037 Gelem/s 3.2074 Gelem/s]
                 change:
                        time:   [−1.0828% −0.8332% −0.5883%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5918% +0.8402% +1.0947%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
mesh_routing/all_routes/10
                        time:   [1.3098 µs 1.3107 µs 1.3116 µs]
                        thrpt:  [762.44 Kelem/s 762.96 Kelem/s 763.45 Kelem/s]
                 change:
                        time:   [+0.6143% +0.7727% +0.9451%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9363% −0.7667% −0.6105%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
mesh_routing/all_routes/100
                        time:   [2.2069 µs 2.2143 µs 2.2220 µs]
                        thrpt:  [450.05 Kelem/s 451.61 Kelem/s 453.12 Kelem/s]
                 change:
                        time:   [−2.4340% −2.0314% −1.6368%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6641% +2.0735% +2.4947%]
                        Performance has improved.
mesh_routing/all_routes/1000
                        time:   [11.767 µs 11.805 µs 11.843 µs]
                        thrpt:  [84.436 Kelem/s 84.708 Kelem/s 84.983 Kelem/s]
                 change:
                        time:   [−0.3526% +0.0069% +0.3470%] (p = 0.97 > 0.05)
                        thrpt:  [−0.3458% −0.0069% +0.3539%]
                        No change in performance detected.
mesh_routing/add_route  time:   [43.113 ns 43.761 ns 44.420 ns]
                        thrpt:  [22.512 Melem/s 22.851 Melem/s 23.195 Melem/s]
                 change:
                        time:   [+1.2119% +3.5115% +5.8762%] (p = 0.00 < 0.05)
                        thrpt:  [−5.5501% −3.3924% −1.1974%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild

     Running benches/net.rs (target/release/deps/net-80cf30609c80a35b)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [1.9757 ns 1.9795 ns 1.9851 ns]
                        thrpt:  [503.75 Melem/s 505.17 Melem/s 506.14 Melem/s]
                 change:
                        time:   [−0.3384% −0.1819% +0.0003%] (p = 0.03 < 0.05)
                        thrpt:  [−0.0003% +0.1822% +0.3395%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
net_header/deserialize  time:   [2.1017 ns 2.1051 ns 2.1087 ns]
                        thrpt:  [474.24 Melem/s 475.04 Melem/s 475.81 Melem/s]
                 change:
                        time:   [−0.3323% −0.1697% −0.0166%] (p = 0.04 < 0.05)
                        thrpt:  [+0.0166% +0.1700% +0.3334%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
net_header/roundtrip    time:   [2.1161 ns 2.1236 ns 2.1308 ns]
                        thrpt:  [469.30 Melem/s 470.89 Melem/s 472.57 Melem/s]
                 change:
                        time:   [+0.0968% +0.3269% +0.5684%] (p = 0.01 < 0.05)
                        thrpt:  [−0.5652% −0.3259% −0.0967%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  14 (14.00%) high mild
  1 (1.00%) high severe

net_event_frame/write_single/64
                        time:   [18.271 ns 18.313 ns 18.362 ns]
                        thrpt:  [3.2460 GiB/s 3.2547 GiB/s 3.2623 GiB/s]
                 change:
                        time:   [−0.1601% +0.1319% +0.4146%] (p = 0.37 > 0.05)
                        thrpt:  [−0.4129% −0.1317% +0.1603%]
                        No change in performance detected.
net_event_frame/write_single/256
                        time:   [44.768 ns 45.154 ns 45.584 ns]
                        thrpt:  [5.2303 GiB/s 5.2802 GiB/s 5.3257 GiB/s]
                 change:
                        time:   [−11.312% −10.252% −9.0988%] (p = 0.00 < 0.05)
                        thrpt:  [+10.010% +11.423% +12.755%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
net_event_frame/write_single/1024
                        time:   [35.940 ns 35.994 ns 36.057 ns]
                        thrpt:  [26.449 GiB/s 26.495 GiB/s 26.535 GiB/s]
                 change:
                        time:   [+0.0100% +0.4568% +0.8828%] (p = 0.04 < 0.05)
                        thrpt:  [−0.8751% −0.4547% −0.0100%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
net_event_frame/write_single/4096
                        time:   [76.899 ns 77.377 ns 77.918 ns]
                        thrpt:  [48.958 GiB/s 49.300 GiB/s 49.607 GiB/s]
                 change:
                        time:   [−1.1752% −0.5475% +0.1249%] (p = 0.11 > 0.05)
                        thrpt:  [−0.1247% +0.5505% +1.1892%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
net_event_frame/write_batch/1
                        time:   [18.315 ns 18.358 ns 18.400 ns]
                        thrpt:  [3.2393 GiB/s 3.2468 GiB/s 3.2544 GiB/s]
                 change:
                        time:   [−0.5050% −0.2179% +0.0637%] (p = 0.15 > 0.05)
                        thrpt:  [−0.0636% +0.2184% +0.5076%]
                        No change in performance detected.
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
  2 (2.00%) high mild
net_event_frame/write_batch/10
                        time:   [70.523 ns 70.986 ns 71.481 ns]
                        thrpt:  [8.3385 GiB/s 8.3967 GiB/s 8.4518 GiB/s]
                 change:
                        time:   [−3.4824% −2.7865% −2.0936%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1384% +2.8664% +3.6080%]
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
net_event_frame/write_batch/50
                        time:   [147.47 ns 147.66 ns 147.88 ns]
                        thrpt:  [20.153 GiB/s 20.184 GiB/s 20.210 GiB/s]
                 change:
                        time:   [−0.4295% −0.2544% −0.0908%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0909% +0.2550% +0.4313%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
net_event_frame/write_batch/100
                        time:   [273.23 ns 273.70 ns 274.21 ns]
                        thrpt:  [21.737 GiB/s 21.777 GiB/s 21.815 GiB/s]
                 change:
                        time:   [+0.0173% +0.1992% +0.3825%] (p = 0.03 < 0.05)
                        thrpt:  [−0.3810% −0.1988% −0.0173%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
net_event_frame/read_batch_10
                        time:   [139.72 ns 140.85 ns 141.98 ns]
                        thrpt:  [70.432 Melem/s 70.998 Melem/s 71.571 Melem/s]
                 change:
                        time:   [−0.6259% +0.3669% +1.3854%] (p = 0.48 > 0.05)
                        thrpt:  [−1.3664% −0.3656% +0.6298%]
                        No change in performance detected.

net_packet_pool/get_return/16
                        time:   [39.017 ns 39.087 ns 39.158 ns]
                        thrpt:  [25.538 Melem/s 25.584 Melem/s 25.630 Melem/s]
                 change:
                        time:   [+0.7553% +0.9658% +1.1763%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1626% −0.9566% −0.7496%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
net_packet_pool/get_return/64
                        time:   [38.687 ns 38.736 ns 38.784 ns]
                        thrpt:  [25.784 Melem/s 25.816 Melem/s 25.849 Melem/s]
                 change:
                        time:   [+0.6636% +0.8779% +1.0769%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0654% −0.8702% −0.6592%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
net_packet_pool/get_return/256
                        time:   [38.601 ns 38.654 ns 38.709 ns]
                        thrpt:  [25.834 Melem/s 25.870 Melem/s 25.906 Melem/s]
                 change:
                        time:   [+0.7845% +1.0008% +1.2273%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2124% −0.9908% −0.7784%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

net_packet_build/build_packet/1
                        time:   [484.59 ns 485.51 ns 486.54 ns]
                        thrpt:  [125.45 MiB/s 125.71 MiB/s 125.95 MiB/s]
                 change:
                        time:   [+0.3329% +0.5842% +0.8300%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8231% −0.5808% −0.3318%]
                        Change within noise threshold.
net_packet_build/build_packet/10
                        time:   [1.8409 µs 1.8459 µs 1.8516 µs]
                        thrpt:  [329.63 MiB/s 330.66 MiB/s 331.54 MiB/s]
                 change:
                        time:   [−0.3915% −0.1328% +0.1277%] (p = 0.32 > 0.05)
                        thrpt:  [−0.1275% +0.1330% +0.3930%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
net_packet_build/build_packet/50
                        time:   [8.1971 µs 8.2080 µs 8.2207 µs]
                        thrpt:  [371.23 MiB/s 371.80 MiB/s 372.30 MiB/s]
                 change:
                        time:   [−0.3612% −0.1397% +0.0711%] (p = 0.22 > 0.05)
                        thrpt:  [−0.0710% +0.1398% +0.3625%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

net_encryption/encrypt/64
                        time:   [482.64 ns 483.14 ns 483.75 ns]
                        thrpt:  [126.17 MiB/s 126.33 MiB/s 126.46 MiB/s]
                 change:
                        time:   [+0.0484% +0.3131% +0.5571%] (p = 0.02 < 0.05)
                        thrpt:  [−0.5540% −0.3121% −0.0484%]
                        Change within noise threshold.
net_encryption/encrypt/256
                        time:   [920.65 ns 922.69 ns 925.11 ns]
                        thrpt:  [263.90 MiB/s 264.60 MiB/s 265.18 MiB/s]
                 change:
                        time:   [−0.1061% +0.1485% +0.3856%] (p = 0.25 > 0.05)
                        thrpt:  [−0.3841% −0.1483% +0.1062%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
net_encryption/encrypt/1024
                        time:   [2.6889 µs 2.6917 µs 2.6951 µs]
                        thrpt:  [362.34 MiB/s 362.80 MiB/s 363.18 MiB/s]
                 change:
                        time:   [−0.3368% −0.1058% +0.1127%] (p = 0.37 > 0.05)
                        thrpt:  [−0.1126% +0.1059% +0.3379%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
net_encryption/encrypt/4096
                        time:   [9.7370 µs 9.7427 µs 9.7483 µs]
                        thrpt:  [400.71 MiB/s 400.94 MiB/s 401.17 MiB/s]
                 change:
                        time:   [−0.6585% −0.4066% −0.2100%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2104% +0.4083% +0.6629%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

net_keypair/generate    time:   [12.386 µs 12.426 µs 12.472 µs]
                        thrpt:  [80.182 Kelem/s 80.477 Kelem/s 80.735 Kelem/s]
                 change:
                        time:   [−3.6363% −2.8356% −2.0550%] (p = 0.00 < 0.05)
                        thrpt:  [+2.0982% +2.9184% +3.7735%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

net_aad/generate        time:   [2.0270 ns 2.0290 ns 2.0320 ns]
                        thrpt:  [492.12 Melem/s 492.84 Melem/s 493.34 Melem/s]
                 change:
                        time:   [−0.2959% −0.1238% +0.0533%] (p = 0.16 > 0.05)
                        thrpt:  [−0.0533% +0.1240% +0.2968%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

pool_comparison/shared_pool_get_return
                        time:   [38.579 ns 38.626 ns 38.672 ns]
                        thrpt:  [25.858 Melem/s 25.889 Melem/s 25.921 Melem/s]
                 change:
                        time:   [+0.4384% +0.6126% +0.7853%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7792% −0.6089% −0.4365%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
pool_comparison/thread_local_pool_get_return
                        time:   [82.438 ns 82.768 ns 83.169 ns]
                        thrpt:  [12.024 Melem/s 12.082 Melem/s 12.130 Melem/s]
                 change:
                        time:   [+2.8823% +4.9878% +7.1686%] (p = 0.00 < 0.05)
                        thrpt:  [−6.6891% −4.7509% −2.8016%]
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
pool_comparison/shared_pool_10x
                        time:   [340.09 ns 340.51 ns 340.95 ns]
                        thrpt:  [2.9330 Melem/s 2.9368 Melem/s 2.9404 Melem/s]
                 change:
                        time:   [−0.5563% −0.2588% +0.0071%] (p = 0.07 > 0.05)
                        thrpt:  [−0.0071% +0.2595% +0.5594%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
pool_comparison/thread_local_pool_10x
                        time:   [946.42 ns 959.35 ns 974.23 ns]
                        thrpt:  [1.0265 Melem/s 1.0424 Melem/s 1.0566 Melem/s]
                 change:
                        time:   [−13.710% −13.229% −12.505%] (p = 0.00 < 0.05)
                        thrpt:  [+14.292% +15.246% +15.888%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

cipher_comparison/shared_pool/64
                        time:   [484.10 ns 485.15 ns 486.45 ns]
                        thrpt:  [125.47 MiB/s 125.81 MiB/s 126.08 MiB/s]
                 change:
                        time:   [+0.4948% +0.7696% +1.0366%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0260% −0.7637% −0.4924%]
                        Change within noise threshold.
cipher_comparison/fast_chacha20/64
                        time:   [531.58 ns 532.17 ns 532.78 ns]
                        thrpt:  [114.56 MiB/s 114.69 MiB/s 114.82 MiB/s]
                 change:
                        time:   [−0.0313% +0.1566% +0.3175%] (p = 0.08 > 0.05)
                        thrpt:  [−0.3165% −0.1563% +0.0313%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
cipher_comparison/shared_pool/256
                        time:   [922.27 ns 923.29 ns 924.48 ns]
                        thrpt:  [264.08 MiB/s 264.43 MiB/s 264.72 MiB/s]
                 change:
                        time:   [−0.2048% +0.0616% +0.3203%] (p = 0.66 > 0.05)
                        thrpt:  [−0.3193% −0.0616% +0.2052%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
cipher_comparison/fast_chacha20/256
                        time:   [963.17 ns 963.82 ns 964.52 ns]
                        thrpt:  [253.12 MiB/s 253.30 MiB/s 253.48 MiB/s]
                 change:
                        time:   [−0.3551% −0.1500% +0.0335%] (p = 0.14 > 0.05)
                        thrpt:  [−0.0335% +0.1502% +0.3563%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
cipher_comparison/shared_pool/1024
                        time:   [2.6865 µs 2.6892 µs 2.6926 µs]
                        thrpt:  [362.69 MiB/s 363.14 MiB/s 363.51 MiB/s]
                 change:
                        time:   [−0.4148% −0.2213% −0.0339%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0339% +0.2218% +0.4165%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
cipher_comparison/fast_chacha20/1024
                        time:   [2.7308 µs 2.7331 µs 2.7355 µs]
                        thrpt:  [356.99 MiB/s 357.31 MiB/s 357.61 MiB/s]
                 change:
                        time:   [+0.3459% +0.5734% +0.8894%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8815% −0.5702% −0.3447%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
cipher_comparison/shared_pool/4096
                        time:   [9.7704 µs 9.7786 µs 9.7877 µs]
                        thrpt:  [399.10 MiB/s 399.47 MiB/s 399.80 MiB/s]
                 change:
                        time:   [−0.4880% −0.1451% +0.1323%] (p = 0.39 > 0.05)
                        thrpt:  [−0.1321% +0.1453% +0.4903%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
cipher_comparison/fast_chacha20/4096
                        time:   [9.7360 µs 9.7533 µs 9.7743 µs]
                        thrpt:  [399.64 MiB/s 400.50 MiB/s 401.22 MiB/s]
                 change:
                        time:   [−0.2447% +0.0613% +0.3429%] (p = 0.70 > 0.05)
                        thrpt:  [−0.3417% −0.0613% +0.2453%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild

adaptive_batcher/optimal_size
                        time:   [972.28 ps 972.95 ps 973.73 ps]
                        thrpt:  [1.0270 Gelem/s 1.0278 Gelem/s 1.0285 Gelem/s]
                 change:
                        time:   [−0.6510% −0.4001% −0.0842%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0843% +0.4017% +0.6553%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
adaptive_batcher/record time:   [3.8839 ns 3.8946 ns 3.9070 ns]
                        thrpt:  [255.95 Melem/s 256.77 Melem/s 257.47 Melem/s]
                 change:
                        time:   [+0.6793% +0.9422% +1.1928%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1788% −0.9334% −0.6748%]
                        Change within noise threshold.
adaptive_batcher/full_cycle
                        time:   [4.3859 ns 4.3907 ns 4.3960 ns]
                        thrpt:  [227.48 Melem/s 227.75 Melem/s 228.00 Melem/s]
                 change:
                        time:   [−0.4926% −0.3489% −0.2118%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2123% +0.3502% +0.4950%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

e2e_packet_build/shared_pool_50_events
                        time:   [8.1727 µs 8.1780 µs 8.1838 µs]
                        thrpt:  [372.90 MiB/s 373.17 MiB/s 373.41 MiB/s]
                 change:
                        time:   [−0.7761% −0.5863% −0.4061%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4077% +0.5898% +0.7822%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
e2e_packet_build/fast_50_events
                        time:   [8.1817 µs 8.1857 µs 8.1897 µs]
                        thrpt:  [372.63 MiB/s 372.82 MiB/s 373.00 MiB/s]
                 change:
                        time:   [−0.4881% −0.3692% −0.2460%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2466% +0.3706% +0.4905%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.2s, enable flat sampling, or reduce sample count to 50.
multithread_packet_build/shared_pool/8
                        time:   [1.8202 ms 1.8232 ms 1.8267 ms]
                        thrpt:  [4.3796 Melem/s 4.3879 Melem/s 4.3952 Melem/s]
                 change:
                        time:   [−2.5820% −1.6973% −0.9186%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9271% +1.7266% +2.6505%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
multithread_packet_build/thread_local_pool/8
                        time:   [868.49 µs 871.34 µs 874.76 µs]
                        thrpt:  [9.1454 Melem/s 9.1812 Melem/s 9.2113 Melem/s]
                 change:
                        time:   [−2.7775% −2.0720% −1.3627%] (p = 0.00 < 0.05)
                        thrpt:  [+1.3815% +2.1158% +2.8568%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
multithread_packet_build/shared_pool/16
                        time:   [4.2393 ms 4.3219 ms 4.4142 ms]
                        thrpt:  [3.6247 Melem/s 3.7021 Melem/s 3.7742 Melem/s]
                 change:
                        time:   [−5.9296% −3.4413% −0.8285%] (p = 0.01 < 0.05)
                        thrpt:  [+0.8355% +3.5640% +6.3034%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.6s, enable flat sampling, or reduce sample count to 50.
multithread_packet_build/thread_local_pool/16
                        time:   [1.6955 ms 1.6980 ms 1.7004 ms]
                        thrpt:  [9.4093 Melem/s 9.4230 Melem/s 9.4367 Melem/s]
                 change:
                        time:   [−1.7759% −1.3238% −0.8522%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8595% +1.3415% +1.8080%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
multithread_packet_build/shared_pool/24
                        time:   [6.3895 ms 6.5734 ms 6.7809 ms]
                        thrpt:  [3.5394 Melem/s 3.6511 Melem/s 3.7562 Melem/s]
                 change:
                        time:   [−15.250% −11.752% −8.0058%] (p = 0.00 < 0.05)
                        thrpt:  [+8.7025% +13.317% +17.994%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
multithread_packet_build/thread_local_pool/24
                        time:   [2.4867 ms 2.4907 ms 2.4948 ms]
                        thrpt:  [9.6201 Melem/s 9.6358 Melem/s 9.6513 Melem/s]
                 change:
                        time:   [−11.074% −10.765% −10.452%] (p = 0.00 < 0.05)
                        thrpt:  [+11.673% +12.064% +12.454%]
                        Performance has improved.
multithread_packet_build/shared_pool/32
                        time:   [8.3375 ms 8.6432 ms 8.9889 ms]
                        thrpt:  [3.5599 Melem/s 3.7023 Melem/s 3.8381 Melem/s]
                 change:
                        time:   [−17.657% −13.136% −8.1143%] (p = 0.00 < 0.05)
                        thrpt:  [+8.8308% +15.122% +21.444%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
multithread_packet_build/thread_local_pool/32
                        time:   [3.2532 ms 3.2641 ms 3.2763 ms]
                        thrpt:  [9.7672 Melem/s 9.8037 Melem/s 9.8364 Melem/s]
                 change:
                        time:   [−1.1781% −0.8044% −0.3590%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3603% +0.8109% +1.1921%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
multithread_mixed_frames/shared_mixed/8
                        time:   [1.3826 ms 1.3890 ms 1.3972 ms]
                        thrpt:  [8.5888 Melem/s 8.6394 Melem/s 8.6791 Melem/s]
                 change:
                        time:   [−2.4749% −1.2086% −0.0041%] (p = 0.07 > 0.05)
                        thrpt:  [+0.0041% +1.2233% +2.5377%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.3s, enable flat sampling, or reduce sample count to 60.
multithread_mixed_frames/fast_mixed/8
                        time:   [1.0421 ms 1.0463 ms 1.0505 ms]
                        thrpt:  [11.423 Melem/s 11.469 Melem/s 11.515 Melem/s]
                 change:
                        time:   [−4.3155% −2.9785% −1.6460%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6736% +3.0700% +4.5101%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
multithread_mixed_frames/shared_mixed/16
                        time:   [3.0406 ms 3.1078 ms 3.1817 ms]
                        thrpt:  [7.5431 Melem/s 7.7225 Melem/s 7.8933 Melem/s]
                 change:
                        time:   [−0.9323% +1.9316% +4.6967%] (p = 0.18 > 0.05)
                        thrpt:  [−4.4860% −1.8950% +0.9411%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
multithread_mixed_frames/fast_mixed/16
                        time:   [2.0362 ms 2.0450 ms 2.0552 ms]
                        thrpt:  [11.678 Melem/s 11.736 Melem/s 11.786 Melem/s]
                 change:
                        time:   [−1.2532% −0.7224% −0.1213%] (p = 0.01 < 0.05)
                        thrpt:  [+0.1214% +0.7276% +1.2691%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
multithread_mixed_frames/shared_mixed/24
                        time:   [4.6100 ms 4.7286 ms 4.8543 ms]
                        thrpt:  [7.4162 Melem/s 7.6132 Melem/s 7.8090 Melem/s]
                 change:
                        time:   [−3.5403% +0.4459% +4.4778%] (p = 0.83 > 0.05)
                        thrpt:  [−4.2858% −0.4439% +3.6703%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
multithread_mixed_frames/fast_mixed/24
                        time:   [2.9880 ms 2.9936 ms 2.9994 ms]
                        thrpt:  [12.002 Melem/s 12.026 Melem/s 12.048 Melem/s]
                 change:
                        time:   [−1.2030% −0.8345% −0.4831%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4855% +0.8415% +1.2177%]
                        Change within noise threshold.
multithread_mixed_frames/shared_mixed/32
                        time:   [6.0223 ms 6.2144 ms 6.4280 ms]
                        thrpt:  [7.4673 Melem/s 7.7240 Melem/s 7.9703 Melem/s]
                 change:
                        time:   [−2.9326% +1.4802% +5.6098%] (p = 0.51 > 0.05)
                        thrpt:  [−5.3118% −1.4586% +3.0212%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
multithread_mixed_frames/fast_mixed/32
                        time:   [3.9242 ms 3.9297 ms 3.9353 ms]
                        thrpt:  [12.197 Melem/s 12.215 Melem/s 12.232 Melem/s]
                 change:
                        time:   [−1.0963% −0.7957% −0.4997%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5022% +0.8021% +1.1084%]
                        Change within noise threshold.

pool_contention/shared_acquire_release/8
                        time:   [17.721 ms 17.757 ms 17.795 ms]
                        thrpt:  [4.4957 Melem/s 4.5052 Melem/s 4.5145 Melem/s]
                 change:
                        time:   [+0.1299% +0.4028% +0.6971%] (p = 0.01 < 0.05)
                        thrpt:  [−0.6923% −0.4012% −0.1298%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.6s, enable flat sampling, or reduce sample count to 60.
pool_contention/fast_acquire_release/8
                        time:   [1.0995 ms 1.1039 ms 1.1083 ms]
                        thrpt:  [72.185 Melem/s 72.473 Melem/s 72.760 Melem/s]
                 change:
                        time:   [−3.0749% −1.9388% −0.8022%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8086% +1.9772% +3.1725%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
pool_contention/shared_acquire_release/16
                        time:   [42.217 ms 42.666 ms 43.127 ms]
                        thrpt:  [3.7099 Melem/s 3.7501 Melem/s 3.7899 Melem/s]
                 change:
                        time:   [+0.7313% +2.2621% +3.7587%] (p = 0.00 < 0.05)
                        thrpt:  [−3.6225% −2.2120% −0.7259%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
pool_contention/fast_acquire_release/16
                        time:   [2.2615 ms 2.2715 ms 2.2816 ms]
                        thrpt:  [70.125 Melem/s 70.439 Melem/s 70.751 Melem/s]
                 change:
                        time:   [−1.9906% −1.3667% −0.7197%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7249% +1.3857% +2.0310%]
                        Change within noise threshold.
Benchmarking pool_contention/shared_acquire_release/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.0s, or reduce sample count to 80.
pool_contention/shared_acquire_release/24
                        time:   [61.140 ms 62.141 ms 63.169 ms]
                        thrpt:  [3.7993 Melem/s 3.8622 Melem/s 3.9254 Melem/s]
                 change:
                        time:   [−1.7368% +0.3661% +2.7837%] (p = 0.74 > 0.05)
                        thrpt:  [−2.7083% −0.3648% +1.7674%]
                        No change in performance detected.
pool_contention/fast_acquire_release/24
                        time:   [3.2605 ms 3.2728 ms 3.2852 ms]
                        thrpt:  [73.055 Melem/s 73.332 Melem/s 73.608 Melem/s]
                 change:
                        time:   [−1.7081% −1.1135% −0.5311%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5340% +1.1260% +1.7378%]
                        Change within noise threshold.
Benchmarking pool_contention/shared_acquire_release/32: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.9s, or reduce sample count to 50.
pool_contention/shared_acquire_release/32
                        time:   [85.916 ms 88.574 ms 91.970 ms]
                        thrpt:  [3.4794 Melem/s 3.6128 Melem/s 3.7246 Melem/s]
                 change:
                        time:   [−2.7274% +0.9312% +4.9891%] (p = 0.67 > 0.05)
                        thrpt:  [−4.7520% −0.9226% +2.8039%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
pool_contention/fast_acquire_release/32
                        time:   [4.1520 ms 4.1638 ms 4.1758 ms]
                        thrpt:  [76.632 Melem/s 76.853 Melem/s 77.072 Melem/s]
                 change:
                        time:   [−1.7170% −1.2392% −0.7662%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7722% +1.2547% +1.7470%]
                        Change within noise threshold.

throughput_scaling/fast_pool_scaling/1
                        time:   [6.7122 ms 6.7210 ms 6.7362 ms]
                        thrpt:  [296.90 Kelem/s 297.58 Kelem/s 297.97 Kelem/s]
                 change:
                        time:   [−0.4904% −0.1464% +0.2570%] (p = 0.48 > 0.05)
                        thrpt:  [−0.2563% +0.1466% +0.4928%]
                        No change in performance detected.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) high mild
  2 (10.00%) high severe
throughput_scaling/fast_pool_scaling/2
                        time:   [6.9700 ms 6.9737 ms 6.9766 ms]
                        thrpt:  [573.34 Kelem/s 573.58 Kelem/s 573.89 Kelem/s]
                 change:
                        time:   [−0.5200% −0.3466% −0.1728%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1731% +0.3478% +0.5227%]
                        Change within noise threshold.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe
throughput_scaling/fast_pool_scaling/4
                        time:   [7.4299 ms 7.4390 ms 7.4483 ms]
                        thrpt:  [1.0741 Melem/s 1.0754 Melem/s 1.0767 Melem/s]
                 change:
                        time:   [+0.0710% +0.2143% +0.3668%] (p = 0.01 < 0.05)
                        thrpt:  [−0.3654% −0.2138% −0.0710%]
                        Change within noise threshold.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
throughput_scaling/fast_pool_scaling/8
                        time:   [7.7524 ms 7.8256 ms 7.9583 ms]
                        thrpt:  [2.0105 Melem/s 2.0446 Melem/s 2.0639 Melem/s]
                 change:
                        time:   [−1.7501% −0.5856% +0.7291%] (p = 0.44 > 0.05)
                        thrpt:  [−0.7238% +0.5891% +1.7813%]
                        No change in performance detected.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) high mild
  2 (10.00%) high severe
throughput_scaling/fast_pool_scaling/16
                        time:   [15.371 ms 15.423 ms 15.486 ms]
                        thrpt:  [2.0664 Melem/s 2.0748 Melem/s 2.0818 Melem/s]
                 change:
                        time:   [−1.1947% +0.1871% +1.6372%] (p = 0.81 > 0.05)
                        thrpt:  [−1.6108% −0.1868% +1.2092%]
                        No change in performance detected.
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) high mild
  3 (15.00%) high severe
throughput_scaling/fast_pool_scaling/24
                        time:   [22.767 ms 22.826 ms 22.890 ms]
                        thrpt:  [2.0970 Melem/s 2.1028 Melem/s 2.1084 Melem/s]
                 change:
                        time:   [−1.3776% −0.8978% −0.4046%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4062% +0.9060% +1.3968%]
                        Change within noise threshold.
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/32: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.3s, enable flat sampling, or reduce sample count to 10.
throughput_scaling/fast_pool_scaling/32
                        time:   [29.899 ms 29.946 ms 29.987 ms]
                        thrpt:  [2.1343 Melem/s 2.1372 Melem/s 2.1405 Melem/s]
                 change:
                        time:   [−0.6519% −0.0342% +0.8028%] (p = 0.94 > 0.05)
                        thrpt:  [−0.7964% +0.0343% +0.6561%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe

routing_header/serialize
                        time:   [625.82 ps 627.06 ps 628.72 ps]
                        thrpt:  [1.5905 Gelem/s 1.5947 Gelem/s 1.5979 Gelem/s]
                 change:
                        time:   [−0.5829% −0.2677% +0.0521%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0520% +0.2684% +0.5863%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
routing_header/deserialize
                        time:   [936.11 ps 936.61 ps 937.12 ps]
                        thrpt:  [1.0671 Gelem/s 1.0677 Gelem/s 1.0682 Gelem/s]
                 change:
                        time:   [−1.6303% −0.6726% −0.0835%] (p = 0.08 > 0.05)
                        thrpt:  [+0.0836% +0.6772% +1.6574%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
routing_header/roundtrip
                        time:   [935.58 ps 936.94 ps 939.36 ps]
                        thrpt:  [1.0646 Gelem/s 1.0673 Gelem/s 1.0689 Gelem/s]
                 change:
                        time:   [−0.4266% −0.1426% +0.0951%] (p = 0.32 > 0.05)
                        thrpt:  [−0.0950% +0.1428% +0.4284%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
routing_header/forward  time:   [571.46 ps 572.90 ps 574.16 ps]
                        thrpt:  [1.7417 Gelem/s 1.7455 Gelem/s 1.7499 Gelem/s]
                 change:
                        time:   [−0.2399% +0.1660% +0.5871%] (p = 0.44 > 0.05)
                        thrpt:  [−0.5837% −0.1658% +0.2405%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

routing_table/lookup_hit
                        time:   [37.396 ns 38.087 ns 38.802 ns]
                        thrpt:  [25.772 Melem/s 26.256 Melem/s 26.741 Melem/s]
                 change:
                        time:   [−1.4251% +0.8285% +3.0243%] (p = 0.47 > 0.05)
                        thrpt:  [−2.9355% −0.8217% +1.4457%]
                        No change in performance detected.
routing_table/lookup_miss
                        time:   [15.110 ns 15.194 ns 15.273 ns]
                        thrpt:  [65.474 Melem/s 65.817 Melem/s 66.180 Melem/s]
                 change:
                        time:   [+0.2203% +0.9002% +1.5725%] (p = 0.01 < 0.05)
                        thrpt:  [−1.5482% −0.8922% −0.2199%]
                        Change within noise threshold.
Found 18 outliers among 100 measurements (18.00%)
  7 (7.00%) low severe
  2 (2.00%) low mild
  9 (9.00%) high mild
routing_table/is_local  time:   [313.95 ps 314.60 ps 315.31 ps]
                        thrpt:  [3.1715 Gelem/s 3.1786 Gelem/s 3.1852 Gelem/s]
                 change:
                        time:   [−0.2674% +0.0275% +0.3276%] (p = 0.85 > 0.05)
                        thrpt:  [−0.3266% −0.0275% +0.2681%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
routing_table/add_route time:   [221.28 ns 227.91 ns 234.84 ns]
                        thrpt:  [4.2582 Melem/s 4.3877 Melem/s 4.5191 Melem/s]
                 change:
                        time:   [−10.595% −5.5675% +0.0233%] (p = 0.05 < 0.05)
                        thrpt:  [−0.0233% +5.8958% +11.850%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
routing_table/record_in time:   [48.983 ns 49.706 ns 50.420 ns]
                        thrpt:  [19.833 Melem/s 20.118 Melem/s 20.415 Melem/s]
                 change:
                        time:   [−0.4911% +1.6052% +3.6024%] (p = 0.13 > 0.05)
                        thrpt:  [−3.4771% −1.5799% +0.4935%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild
routing_table/record_out
                        time:   [20.566 ns 20.988 ns 21.397 ns]
                        thrpt:  [46.734 Melem/s 47.647 Melem/s 48.625 Melem/s]
                 change:
                        time:   [+1.4527% +4.6222% +7.8666%] (p = 0.01 < 0.05)
                        thrpt:  [−7.2929% −4.4180% −1.4319%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild
routing_table/aggregate_stats
                        time:   [2.1140 µs 2.1182 µs 2.1234 µs]
                        thrpt:  [470.95 Kelem/s 472.10 Kelem/s 473.04 Kelem/s]
                 change:
                        time:   [+1.4448% +1.7454% +2.0113%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9717% −1.7155% −1.4242%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

fair_scheduler/creation time:   [287.77 ns 288.21 ns 288.64 ns]
                        thrpt:  [3.4645 Melem/s 3.4697 Melem/s 3.4750 Melem/s]
                 change:
                        time:   [+0.1063% +0.3219% +0.5387%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5358% −0.3209% −0.1062%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
fair_scheduler/stream_count_empty
                        time:   [200.34 ns 200.77 ns 201.47 ns]
                        thrpt:  [4.9636 Melem/s 4.9809 Melem/s 4.9915 Melem/s]
                 change:
                        time:   [−0.2461% −0.0811% +0.0983%] (p = 0.36 > 0.05)
                        thrpt:  [−0.0983% +0.0811% +0.2467%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
fair_scheduler/total_queued
                        time:   [311.91 ps 312.04 ps 312.17 ps]
                        thrpt:  [3.2033 Gelem/s 3.2047 Gelem/s 3.2060 Gelem/s]
                 change:
                        time:   [−0.3074% −0.1260% +0.1140%] (p = 0.32 > 0.05)
                        thrpt:  [−0.1139% +0.1261% +0.3084%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
fair_scheduler/cleanup_empty
                        time:   [201.17 ns 201.26 ns 201.35 ns]
                        thrpt:  [4.9664 Melem/s 4.9686 Melem/s 4.9709 Melem/s]
                 change:
                        time:   [−0.6151% −0.2997% −0.0611%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0611% +0.3006% +0.6189%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

routing_table_concurrent/concurrent_lookup/4
                        time:   [146.05 µs 150.82 µs 155.22 µs]
                        thrpt:  [25.770 Melem/s 26.522 Melem/s 27.389 Melem/s]
                 change:
                        time:   [−14.650% −11.685% −8.7486%] (p = 0.00 < 0.05)
                        thrpt:  [+9.5874% +13.231% +17.165%]
                        Performance has improved.
routing_table_concurrent/concurrent_stats/4
                        time:   [285.36 µs 285.91 µs 286.46 µs]
                        thrpt:  [13.963 Melem/s 13.990 Melem/s 14.017 Melem/s]
                 change:
                        time:   [−3.1589% −2.8002% −2.4547%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5165% +2.8809% +3.2619%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
routing_table_concurrent/concurrent_lookup/8
                        time:   [246.94 µs 247.27 µs 247.59 µs]
                        thrpt:  [32.311 Melem/s 32.354 Melem/s 32.396 Melem/s]
                 change:
                        time:   [+0.4020% +0.9094% +1.5288%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5058% −0.9012% −0.4004%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
routing_table_concurrent/concurrent_stats/8
                        time:   [401.59 µs 402.88 µs 404.52 µs]
                        thrpt:  [19.777 Melem/s 19.857 Melem/s 19.921 Melem/s]
                 change:
                        time:   [−4.5144% −3.8731% −3.2595%] (p = 0.00 < 0.05)
                        thrpt:  [+3.3693% +4.0292% +4.7278%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  3 (3.00%) high severe
routing_table_concurrent/concurrent_lookup/16
                        time:   [426.43 µs 427.39 µs 428.13 µs]
                        thrpt:  [37.371 Melem/s 37.437 Melem/s 37.521 Melem/s]
                 change:
                        time:   [−0.8553% −0.2498% +0.3561%] (p = 0.44 > 0.05)
                        thrpt:  [−0.3549% +0.2504% +0.8626%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
routing_table_concurrent/concurrent_stats/16
                        time:   [796.18 µs 797.54 µs 799.02 µs]
                        thrpt:  [20.025 Melem/s 20.062 Melem/s 20.096 Melem/s]
                 change:
                        time:   [−3.8564% −3.4739% −3.0773%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1750% +3.5989% +4.0111%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe

routing_decision/parse_lookup_forward
                        time:   [38.357 ns 38.888 ns 39.472 ns]
                        thrpt:  [25.334 Melem/s 25.715 Melem/s 26.071 Melem/s]
                 change:
                        time:   [−1.1985% +0.3720% +1.9796%] (p = 0.66 > 0.05)
                        thrpt:  [−1.9412% −0.3707% +1.2130%]
                        No change in performance detected.
routing_decision/full_with_stats
                        time:   [109.74 ns 109.96 ns 110.21 ns]
                        thrpt:  [9.0733 Melem/s 9.0945 Melem/s 9.1125 Melem/s]
                 change:
                        time:   [−0.0168% +0.5200% +1.1437%] (p = 0.10 > 0.05)
                        thrpt:  [−1.1308% −0.5173% +0.0168%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe

stream_multiplexing/lookup_all/10
                        time:   [291.75 ns 291.86 ns 291.97 ns]
                        thrpt:  [34.251 Melem/s 34.263 Melem/s 34.276 Melem/s]
                 change:
                        time:   [−0.3206% −0.1935% −0.0646%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0647% +0.1938% +0.3216%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
stream_multiplexing/stats_all/10
                        time:   [478.59 ns 484.33 ns 490.20 ns]
                        thrpt:  [20.400 Melem/s 20.647 Melem/s 20.895 Melem/s]
                 change:
                        time:   [−1.5285% −0.0714% +1.4101%] (p = 0.92 > 0.05)
                        thrpt:  [−1.3905% +0.0715% +1.5522%]
                        No change in performance detected.
stream_multiplexing/lookup_all/100
                        time:   [2.9145 µs 2.9157 µs 2.9168 µs]
                        thrpt:  [34.284 Melem/s 34.297 Melem/s 34.311 Melem/s]
                 change:
                        time:   [−0.4951% −0.3441% −0.1851%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1854% +0.3453% +0.4976%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
stream_multiplexing/stats_all/100
                        time:   [4.9057 µs 4.9629 µs 5.0195 µs]
                        thrpt:  [19.922 Melem/s 20.149 Melem/s 20.384 Melem/s]
                 change:
                        time:   [−0.6581% +0.8844% +2.5628%] (p = 0.28 > 0.05)
                        thrpt:  [−2.4987% −0.8767% +0.6625%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild
stream_multiplexing/lookup_all/1000
                        time:   [29.207 µs 29.223 µs 29.239 µs]
                        thrpt:  [34.200 Melem/s 34.220 Melem/s 34.238 Melem/s]
                 change:
                        time:   [−2.6174% −2.4898% −2.3516%] (p = 0.00 < 0.05)
                        thrpt:  [+2.4083% +2.5533% +2.6878%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
stream_multiplexing/stats_all/1000
                        time:   [52.366 µs 52.473 µs 52.574 µs]
                        thrpt:  [19.021 Melem/s 19.058 Melem/s 19.096 Melem/s]
                 change:
                        time:   [−2.6809% −2.4170% −2.1447%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1917% +2.4769% +2.7548%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
stream_multiplexing/lookup_all/10000
                        time:   [292.79 µs 292.93 µs 293.07 µs]
                        thrpt:  [34.121 Melem/s 34.138 Melem/s 34.155 Melem/s]
                 change:
                        time:   [−0.3534% −0.1563% +0.0369%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0369% +0.1566% +0.3546%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
stream_multiplexing/stats_all/10000
                        time:   [567.75 µs 569.47 µs 571.46 µs]
                        thrpt:  [17.499 Melem/s 17.560 Melem/s 17.613 Melem/s]
                 change:
                        time:   [−0.8930% +0.1580% +1.1351%] (p = 0.77 > 0.05)
                        thrpt:  [−1.1224% −0.1578% +0.9010%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe

multihop_packet_builder/build/64
                        time:   [23.101 ns 23.215 ns 23.339 ns]
                        thrpt:  [2.5539 GiB/s 2.5675 GiB/s 2.5802 GiB/s]
                 change:
                        time:   [−0.7608% −0.1242% +0.4871%] (p = 0.70 > 0.05)
                        thrpt:  [−0.4847% +0.1244% +0.7667%]
                        No change in performance detected.
multihop_packet_builder/build_priority/64
                        time:   [20.623 ns 20.647 ns 20.671 ns]
                        thrpt:  [2.8835 GiB/s 2.8868 GiB/s 2.8902 GiB/s]
                 change:
                        time:   [−2.9117% −2.7265% −2.5570%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6241% +2.8029% +2.9990%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
multihop_packet_builder/build/256
                        time:   [50.844 ns 51.113 ns 51.395 ns]
                        thrpt:  [4.6389 GiB/s 4.6646 GiB/s 4.6892 GiB/s]
                 change:
                        time:   [−6.7583% −5.9395% −5.0477%] (p = 0.00 < 0.05)
                        thrpt:  [+5.3161% +6.3145% +7.2482%]
                        Performance has improved.
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) high mild
  9 (9.00%) high severe
multihop_packet_builder/build_priority/256
                        time:   [49.590 ns 50.016 ns 50.471 ns]
                        thrpt:  [4.7239 GiB/s 4.7669 GiB/s 4.8078 GiB/s]
                 change:
                        time:   [−6.1517% −4.9694% −3.7715%] (p = 0.00 < 0.05)
                        thrpt:  [+3.9193% +5.2292% +6.5549%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
multihop_packet_builder/build/1024
                        time:   [41.080 ns 41.145 ns 41.218 ns]
                        thrpt:  [23.137 GiB/s 23.178 GiB/s 23.215 GiB/s]
                 change:
                        time:   [−27.387% −12.788% −2.6392%] (p = 0.18 > 0.05)
                        thrpt:  [+2.7107% +14.663% +37.716%]
                        No change in performance detected.
multihop_packet_builder/build_priority/1024
                        time:   [38.363 ns 38.402 ns 38.447 ns]
                        thrpt:  [24.805 GiB/s 24.834 GiB/s 24.859 GiB/s]
                 change:
                        time:   [−2.4830% −2.2913% −2.1117%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1573% +2.3451% +2.5462%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild
multihop_packet_builder/build/4096
                        time:   [81.613 ns 82.055 ns 82.531 ns]
                        thrpt:  [46.222 GiB/s 46.489 GiB/s 46.741 GiB/s]
                 change:
                        time:   [−5.8096% −3.2206% −1.0461%] (p = 0.01 < 0.05)
                        thrpt:  [+1.0572% +3.3278% +6.1679%]
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
multihop_packet_builder/build_priority/4096
                        time:   [80.016 ns 80.475 ns 80.962 ns]
                        thrpt:  [47.117 GiB/s 47.402 GiB/s 47.674 GiB/s]
                 change:
                        time:   [−1.2632% −0.6371% +0.0273%] (p = 0.05 > 0.05)
                        thrpt:  [−0.0273% +0.6412% +1.2794%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

multihop_chain/forward_chain/1
                        time:   [58.643 ns 59.066 ns 59.517 ns]
                        thrpt:  [16.802 Melem/s 16.930 Melem/s 17.052 Melem/s]
                 change:
                        time:   [−2.5700% −1.4075% −0.1924%] (p = 0.02 < 0.05)
                        thrpt:  [+0.1928% +1.4276% +2.6378%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
multihop_chain/forward_chain/2
                        time:   [116.62 ns 117.32 ns 118.08 ns]
                        thrpt:  [8.4690 Melem/s 8.5239 Melem/s 8.5751 Melem/s]
                 change:
                        time:   [+2.4110% +2.9878% +3.6502%] (p = 0.00 < 0.05)
                        thrpt:  [−3.5216% −2.9011% −2.3542%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
multihop_chain/forward_chain/3
                        time:   [162.25 ns 163.16 ns 164.18 ns]
                        thrpt:  [6.0907 Melem/s 6.1291 Melem/s 6.1631 Melem/s]
                 change:
                        time:   [+1.8762% +2.5180% +3.1697%] (p = 0.00 < 0.05)
                        thrpt:  [−3.0723% −2.4561% −1.8416%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
multihop_chain/forward_chain/4
                        time:   [216.70 ns 216.95 ns 217.21 ns]
                        thrpt:  [4.6038 Melem/s 4.6094 Melem/s 4.6146 Melem/s]
                 change:
                        time:   [−13.547% −5.2658% −0.2216%] (p = 0.22 > 0.05)
                        thrpt:  [+0.2221% +5.5585% +15.669%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
multihop_chain/forward_chain/5
                        time:   [272.03 ns 273.51 ns 275.15 ns]
                        thrpt:  [3.6344 Melem/s 3.6561 Melem/s 3.6760 Melem/s]
                 change:
                        time:   [−1.6687% −1.2839% −0.8383%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8454% +1.3006% +1.6970%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

hop_latency/single_hop_process
                        time:   [1.4527 ns 1.4534 ns 1.4540 ns]
                        thrpt:  [687.76 Melem/s 688.06 Melem/s 688.37 Melem/s]
                 change:
                        time:   [−2.4513% −2.3548% −2.2587%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3109% +2.4116% +2.5129%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
hop_latency/single_hop_full
                        time:   [57.312 ns 57.810 ns 58.304 ns]
                        thrpt:  [17.152 Melem/s 17.298 Melem/s 17.448 Melem/s]
                 change:
                        time:   [−7.2658% −5.7011% −4.0230%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1916% +6.0458% +7.8351%]
                        Performance has improved.

hop_scaling/64B_1hops   time:   [29.663 ns 29.697 ns 29.731 ns]
                        thrpt:  [2.0048 GiB/s 2.0071 GiB/s 2.0094 GiB/s]
                 change:
                        time:   [−0.1446% +0.0599% +0.2561%] (p = 0.56 > 0.05)
                        thrpt:  [−0.2554% −0.0598% +0.1449%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
hop_scaling/64B_2hops   time:   [52.682 ns 52.765 ns 52.850 ns]
                        thrpt:  [1.1278 GiB/s 1.1296 GiB/s 1.1314 GiB/s]
                 change:
                        time:   [+0.3814% +0.7702% +1.1838%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1699% −0.7644% −0.3800%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
hop_scaling/64B_3hops   time:   [76.252 ns 76.365 ns 76.481 ns]
                        thrpt:  [798.04 MiB/s 799.26 MiB/s 800.44 MiB/s]
                 change:
                        time:   [+0.1330% +0.3886% +0.6453%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6411% −0.3871% −0.1328%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/64B_4hops   time:   [100.00 ns 100.17 ns 100.33 ns]
                        thrpt:  [608.34 MiB/s 609.34 MiB/s 610.35 MiB/s]
                 change:
                        time:   [+0.9729% +1.2673% +1.6042%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5789% −1.2515% −0.9636%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
hop_scaling/64B_5hops   time:   [130.28 ns 130.59 ns 130.92 ns]
                        thrpt:  [466.19 MiB/s 467.37 MiB/s 468.50 MiB/s]
                 change:
                        time:   [+1.0380% +1.4971% +1.9905%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9516% −1.4750% −1.0274%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
hop_scaling/256B_1hops  time:   [58.344 ns 58.795 ns 59.304 ns]
                        thrpt:  [4.0203 GiB/s 4.0551 GiB/s 4.0864 GiB/s]
                 change:
                        time:   [−5.3541% −4.1051% −2.8038%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8847% +4.2809% +5.6570%]
                        Performance has improved.
Found 15 outliers among 100 measurements (15.00%)
  13 (13.00%) high mild
  2 (2.00%) high severe
hop_scaling/256B_2hops  time:   [115.22 ns 115.71 ns 116.24 ns]
                        thrpt:  [2.0510 GiB/s 2.0604 GiB/s 2.0692 GiB/s]
                 change:
                        time:   [−3.6122% −3.0324% −2.4592%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5212% +3.1272% +3.7475%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
hop_scaling/256B_3hops  time:   [161.75 ns 162.42 ns 163.19 ns]
                        thrpt:  [1.4610 GiB/s 1.4679 GiB/s 1.4740 GiB/s]
                 change:
                        time:   [−7.0169% −6.0438% −5.0575%] (p = 0.00 < 0.05)
                        thrpt:  [+5.3269% +6.4326% +7.5464%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
hop_scaling/256B_4hops  time:   [220.36 ns 221.76 ns 223.33 ns]
                        thrpt:  [1.0675 GiB/s 1.0751 GiB/s 1.0820 GiB/s]
                 change:
                        time:   [+0.0221% +1.0426% +2.1088%] (p = 0.05 > 0.05)
                        thrpt:  [−2.0653% −1.0319% −0.0220%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
hop_scaling/256B_5hops  time:   [281.67 ns 283.72 ns 285.75 ns]
                        thrpt:  [854.38 MiB/s 860.50 MiB/s 866.75 MiB/s]
                 change:
                        time:   [−2.5905% −1.5990% −0.6098%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6136% +1.6250% +2.6594%]
                        Change within noise threshold.
hop_scaling/1024B_1hops time:   [48.088 ns 48.124 ns 48.159 ns]
                        thrpt:  [19.802 GiB/s 19.817 GiB/s 19.832 GiB/s]
                 change:
                        time:   [+0.3049% +0.5296% +0.7364%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7310% −0.5268% −0.3039%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_2hops time:   [116.31 ns 117.18 ns 118.09 ns]
                        thrpt:  [8.0757 GiB/s 8.1383 GiB/s 8.1997 GiB/s]
                 change:
                        time:   [+3.4894% +4.2862% +4.8752%] (p = 0.00 < 0.05)
                        thrpt:  [−4.6486% −4.1100% −3.3718%]
                        Performance has regressed.
hop_scaling/1024B_3hops time:   [158.73 ns 159.92 ns 161.16 ns]
                        thrpt:  [5.9177 GiB/s 5.9634 GiB/s 6.0083 GiB/s]
                 change:
                        time:   [−0.9838% −0.0070% +0.9174%] (p = 0.99 > 0.05)
                        thrpt:  [−0.9090% +0.0070% +0.9935%]
                        No change in performance detected.
hop_scaling/1024B_4hops time:   [215.22 ns 215.82 ns 216.46 ns]
                        thrpt:  [4.4058 GiB/s 4.4189 GiB/s 4.4311 GiB/s]
                 change:
                        time:   [−14.130% −5.7227% −0.6051%] (p = 0.19 > 0.05)
                        thrpt:  [+0.6088% +6.0701% +16.455%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/1024B_5hops time:   [257.23 ns 258.70 ns 260.15 ns]
                        thrpt:  [3.6659 GiB/s 3.6864 GiB/s 3.7075 GiB/s]
                 change:
                        time:   [−4.3580% −3.6568% −2.9348%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0236% +3.7956% +4.5566%]
                        Performance has improved.

multihop_with_routing/route_and_forward/1
                        time:   [177.39 ns 178.04 ns 178.84 ns]
                        thrpt:  [5.5917 Melem/s 5.6166 Melem/s 5.6373 Melem/s]
                 change:
                        time:   [−4.2415% −3.7336% −3.2049%] (p = 0.00 < 0.05)
                        thrpt:  [+3.3110% +3.8784% +4.4294%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
multihop_with_routing/route_and_forward/2
                        time:   [352.02 ns 353.15 ns 354.37 ns]
                        thrpt:  [2.8219 Melem/s 2.8317 Melem/s 2.8408 Melem/s]
                 change:
                        time:   [+0.7378% +1.1729% +1.6150%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5893% −1.1593% −0.7324%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
multihop_with_routing/route_and_forward/3
                        time:   [525.86 ns 527.53 ns 529.35 ns]
                        thrpt:  [1.8891 Melem/s 1.8956 Melem/s 1.9016 Melem/s]
                 change:
                        time:   [+0.6400% +1.1723% +1.7434%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7135% −1.1587% −0.6360%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
multihop_with_routing/route_and_forward/4
                        time:   [710.84 ns 713.90 ns 717.12 ns]
                        thrpt:  [1.3945 Melem/s 1.4007 Melem/s 1.4068 Melem/s]
                 change:
                        time:   [+0.3925% +0.8722% +1.3316%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3141% −0.8646% −0.3909%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
multihop_with_routing/route_and_forward/5
                        time:   [877.60 ns 880.92 ns 884.43 ns]
                        thrpt:  [1.1307 Melem/s 1.1352 Melem/s 1.1395 Melem/s]
                 change:
                        time:   [−0.4661% +0.0372% +0.5123%] (p = 0.88 > 0.05)
                        thrpt:  [−0.5097% −0.0372% +0.4683%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

multihop_concurrent/concurrent_forward/4
                        time:   [842.05 µs 853.81 µs 869.23 µs]
                        thrpt:  [4.6018 Melem/s 4.6849 Melem/s 4.7503 Melem/s]
                 change:
                        time:   [+22.805% +24.160% +25.725%] (p = 0.00 < 0.05)
                        thrpt:  [−20.461% −19.459% −18.570%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
multihop_concurrent/concurrent_forward/8
                        time:   [969.21 µs 982.85 µs 1.0012 ms]
                        thrpt:  [7.9905 Melem/s 8.1396 Melem/s 8.2542 Melem/s]
                 change:
                        time:   [+0.0667% +2.6640% +5.4888%] (p = 0.06 > 0.05)
                        thrpt:  [−5.2032% −2.5949% −0.0666%]
                        No change in performance detected.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
  1 (5.00%) high severe
multihop_concurrent/concurrent_forward/16
                        time:   [1.9939 ms 2.0173 ms 2.0414 ms]
                        thrpt:  [7.8376 Melem/s 7.9312 Melem/s 8.0245 Melem/s]
                 change:
                        time:   [−5.5889% −4.1285% −2.6424%] (p = 0.00 < 0.05)
                        thrpt:  [+2.7141% +4.3063% +5.9198%]
                        Performance has improved.

pingwave/serialize      time:   [778.33 ps 778.96 ps 779.80 ps]
                        thrpt:  [1.2824 Gelem/s 1.2838 Gelem/s 1.2848 Gelem/s]
                 change:
                        time:   [−0.5915% −0.3045% −0.0131%] (p = 0.04 < 0.05)
                        thrpt:  [+0.0131% +0.3055% +0.5950%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
pingwave/deserialize    time:   [933.52 ps 934.41 ps 935.56 ps]
                        thrpt:  [1.0689 Gelem/s 1.0702 Gelem/s 1.0712 Gelem/s]
                 change:
                        time:   [−2.0333% −1.8249% −1.6010%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6270% +1.8588% +2.0755%]
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) high mild
  15 (15.00%) high severe
pingwave/roundtrip      time:   [931.39 ps 931.54 ps 931.70 ps]
                        thrpt:  [1.0733 Gelem/s 1.0735 Gelem/s 1.0737 Gelem/s]
                 change:
                        time:   [−2.6324% −2.4993% −2.3586%] (p = 0.00 < 0.05)
                        thrpt:  [+2.4155% +2.5634% +2.7035%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
pingwave/forward        time:   [627.62 ps 667.72 ps 752.46 ps]
                        thrpt:  [1.3290 Gelem/s 1.4976 Gelem/s 1.5933 Gelem/s]
                 change:
                        time:   [−1.4696% +1.3238% +6.5203%] (p = 0.75 > 0.05)
                        thrpt:  [−6.1212% −1.3065% +1.4916%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe

capabilities/serialize_simple
                        time:   [19.091 ns 19.123 ns 19.163 ns]
                        thrpt:  [52.184 Melem/s 52.294 Melem/s 52.381 Melem/s]
                 change:
                        time:   [−0.4455% −0.2208% +0.0079%] (p = 0.06 > 0.05)
                        thrpt:  [−0.0079% +0.2213% +0.4475%]
                        No change in performance detected.
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) high mild
  10 (10.00%) high severe
capabilities/deserialize_simple
                        time:   [4.7483 ns 4.7538 ns 4.7596 ns]
                        thrpt:  [210.10 Melem/s 210.36 Melem/s 210.60 Melem/s]
                 change:
                        time:   [−0.5050% −0.2948% −0.0892%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0893% +0.2957% +0.5076%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
capabilities/serialize_complex
                        time:   [40.979 ns 40.989 ns 41.002 ns]
                        thrpt:  [24.389 Melem/s 24.397 Melem/s 24.403 Melem/s]
                 change:
                        time:   [−0.6828% −0.5802% −0.4829%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4852% +0.5836% +0.6875%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
capabilities/deserialize_complex
                        time:   [152.28 ns 153.56 ns 155.97 ns]
                        thrpt:  [6.4116 Melem/s 6.5121 Melem/s 6.5668 Melem/s]
                 change:
                        time:   [−1.3061% −0.8333% −0.0556%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0556% +0.8403% +1.3234%]
                        Change within noise threshold.
Found 18 outliers among 100 measurements (18.00%)
  2 (2.00%) high mild
  16 (16.00%) high severe

local_graph/create_pingwave
                        time:   [2.0978 ns 2.0999 ns 2.1021 ns]
                        thrpt:  [475.73 Melem/s 476.20 Melem/s 476.68 Melem/s]
                 change:
                        time:   [−0.8032% −0.4338% −0.0839%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0840% +0.4356% +0.8097%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  4 (4.00%) high mild
local_graph/on_pingwave_new
                        time:   [111.50 ns 113.27 ns 114.73 ns]
                        thrpt:  [8.7161 Melem/s 8.8287 Melem/s 8.9683 Melem/s]
                 change:
                        time:   [−11.542% −7.7538% −3.8240%] (p = 0.00 < 0.05)
                        thrpt:  [+3.9760% +8.4056% +13.048%]
                        Performance has improved.
local_graph/on_pingwave_duplicate
                        time:   [33.060 ns 33.091 ns 33.119 ns]
                        thrpt:  [30.194 Melem/s 30.219 Melem/s 30.248 Melem/s]
                 change:
                        time:   [−3.7159% −3.5314% −3.3544%] (p = 0.00 < 0.05)
                        thrpt:  [+3.4708% +3.6607% +3.8593%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
local_graph/get_node    time:   [26.053 ns 26.430 ns 27.283 ns]
                        thrpt:  [36.653 Melem/s 37.835 Melem/s 38.383 Melem/s]
                 change:
                        time:   [+0.4524% +12.091% +34.588%] (p = 0.22 > 0.05)
                        thrpt:  [−25.699% −10.787% −0.4504%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
local_graph/node_count  time:   [199.18 ns 199.38 ns 199.72 ns]
                        thrpt:  [5.0069 Melem/s 5.0155 Melem/s 5.0207 Melem/s]
                 change:
                        time:   [−0.8632% −0.7063% −0.5628%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5660% +0.7114% +0.8707%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
local_graph/stats       time:   [597.31 ns 597.46 ns 597.63 ns]
                        thrpt:  [1.6733 Melem/s 1.6737 Melem/s 1.6742 Melem/s]
                 change:
                        time:   [−0.7551% −0.6073% −0.4435%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4455% +0.6111% +0.7608%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

graph_scaling/all_nodes/100
                        time:   [2.5039 µs 2.5183 µs 2.5314 µs]
                        thrpt:  [39.504 Melem/s 39.709 Melem/s 39.938 Melem/s]
                 change:
                        time:   [+1.9393% +2.4547% +2.9438%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8596% −2.3959% −1.9024%]
                        Performance has regressed.
graph_scaling/nodes_within_hops/100
                        time:   [2.8487 µs 2.8790 µs 2.9119 µs]
                        thrpt:  [34.342 Melem/s 34.734 Melem/s 35.104 Melem/s]
                 change:
                        time:   [+0.6098% +1.4671% +2.2319%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1832% −1.4459% −0.6061%]
                        Change within noise threshold.
graph_scaling/all_nodes/500
                        time:   [8.0029 µs 8.0948 µs 8.1829 µs]
                        thrpt:  [61.103 Melem/s 61.768 Melem/s 62.477 Melem/s]
                 change:
                        time:   [−0.7033% +0.0948% +0.9314%] (p = 0.82 > 0.05)
                        thrpt:  [−0.9228% −0.0947% +0.7083%]
                        No change in performance detected.
graph_scaling/nodes_within_hops/500
                        time:   [9.3952 µs 9.4634 µs 9.5333 µs]
                        thrpt:  [52.448 Melem/s 52.835 Melem/s 53.219 Melem/s]
                 change:
                        time:   [−3.5560% −3.0081% −2.3899%] (p = 0.00 < 0.05)
                        thrpt:  [+2.4484% +3.1014% +3.6871%]
                        Performance has improved.
graph_scaling/all_nodes/1000
                        time:   [132.69 µs 132.72 µs 132.76 µs]
                        thrpt:  [7.5324 Melem/s 7.5345 Melem/s 7.5364 Melem/s]
                 change:
                        time:   [−35.803% −33.467% −30.629%] (p = 0.00 < 0.05)
                        thrpt:  [+44.152% +50.301% +55.771%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
graph_scaling/nodes_within_hops/1000
                        time:   [47.723 µs 60.290 µs 74.490 µs]
                        thrpt:  [13.425 Melem/s 16.587 Melem/s 20.954 Melem/s]
                 change:
                        time:   [−74.614% −70.361% −66.104%] (p = 0.00 < 0.05)
                        thrpt:  [+195.02% +237.39% +293.92%]
                        Performance has improved.
graph_scaling/all_nodes/5000
                        time:   [92.368 µs 113.29 µs 149.50 µs]
                        thrpt:  [33.444 Melem/s 44.133 Melem/s 54.131 Melem/s]
                 change:
                        time:   [−58.734% −52.590% −44.239%] (p = 0.00 < 0.05)
                        thrpt:  [+79.336% +110.92% +142.33%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
graph_scaling/nodes_within_hops/5000
                        time:   [165.03 µs 176.48 µs 186.40 µs]
                        thrpt:  [26.824 Melem/s 28.333 Melem/s 30.297 Melem/s]
                 change:
                        time:   [−50.090% −46.352% −41.970%] (p = 0.00 < 0.05)
                        thrpt:  [+72.324% +86.400% +100.36%]
                        Performance has improved.

capability_search/find_with_gpu
                        time:   [17.188 µs 17.250 µs 17.303 µs]
                        thrpt:  [57.792 Kelem/s 57.972 Kelem/s 58.179 Kelem/s]
                 change:
                        time:   [−0.5514% −0.1980% +0.1374%] (p = 0.27 > 0.05)
                        thrpt:  [−0.1372% +0.1984% +0.5544%]
                        No change in performance detected.
capability_search/find_by_tool_python
                        time:   [31.114 µs 31.243 µs 31.364 µs]
                        thrpt:  [31.883 Kelem/s 32.007 Kelem/s 32.140 Kelem/s]
                 change:
                        time:   [+1.3584% +1.8235% +2.2477%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1983% −1.7909% −1.3402%]
                        Performance has regressed.
Found 23 outliers among 100 measurements (23.00%)
  21 (21.00%) low severe
  2 (2.00%) high severe
capability_search/find_by_tool_rust
                        time:   [39.394 µs 39.598 µs 39.803 µs]
                        thrpt:  [25.124 Kelem/s 25.254 Kelem/s 25.385 Kelem/s]
                 change:
                        time:   [−0.7115% −0.2622% +0.1821%] (p = 0.25 > 0.05)
                        thrpt:  [−0.1818% +0.2629% +0.7166%]
                        No change in performance detected.

graph_concurrent/concurrent_pingwave/4
                        time:   [107.22 µs 107.57 µs 108.12 µs]
                        thrpt:  [18.497 Melem/s 18.592 Melem/s 18.654 Melem/s]
                 change:
                        time:   [−5.4264% −4.3439% −3.3332%] (p = 0.00 < 0.05)
                        thrpt:  [+3.4482% +4.5412% +5.7378%]
                        Performance has improved.
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe
graph_concurrent/concurrent_pingwave/8
                        time:   [171.05 µs 173.56 µs 176.12 µs]
                        thrpt:  [22.712 Melem/s 23.047 Melem/s 23.385 Melem/s]
                 change:
                        time:   [−5.8354% −4.2653% −2.5572%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6243% +4.4554% +6.1970%]
                        Performance has improved.
graph_concurrent/concurrent_pingwave/16
                        time:   [305.78 µs 308.13 µs 311.24 µs]
                        thrpt:  [25.704 Melem/s 25.963 Melem/s 26.162 Melem/s]
                 change:
                        time:   [−3.5300% −2.6075% −1.5773%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6025% +2.6773% +3.6592%]
                        Performance has improved.
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high mild

path_finding/path_1_hop time:   [1.5399 µs 1.5412 µs 1.5427 µs]
                        thrpt:  [648.23 Kelem/s 648.85 Kelem/s 649.40 Kelem/s]
                 change:
                        time:   [−1.2753% −1.1167% −0.9643%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9737% +1.1293% +1.2918%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
path_finding/path_2_hops
                        time:   [1.5814 µs 1.5826 µs 1.5840 µs]
                        thrpt:  [631.32 Kelem/s 631.87 Kelem/s 632.35 Kelem/s]
                 change:
                        time:   [−1.5442% −1.3687% −1.2075%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2223% +1.3877% +1.5684%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
path_finding/path_4_hops
                        time:   [1.8373 µs 1.8451 µs 1.8552 µs]
                        thrpt:  [539.03 Kelem/s 541.99 Kelem/s 544.27 Kelem/s]
                 change:
                        time:   [−1.1464% −0.9175% −0.6477%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6519% +0.9260% +1.1597%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
path_finding/path_not_found
                        time:   [1.7489 µs 1.7515 µs 1.7544 µs]
                        thrpt:  [570.00 Kelem/s 570.93 Kelem/s 571.78 Kelem/s]
                 change:
                        time:   [−0.2378% +0.0027% +0.2540%] (p = 0.98 > 0.05)
                        thrpt:  [−0.2533% −0.0027% +0.2383%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
path_finding/path_complex_graph
                        time:   [218.64 µs 220.18 µs 221.72 µs]
                        thrpt:  [4.5103 Kelem/s 4.5417 Kelem/s 4.5737 Kelem/s]
                 change:
                        time:   [+0.9653% +2.0890% +3.2623%] (p = 0.00 < 0.05)
                        thrpt:  [−3.1593% −2.0463% −0.9561%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

failure_detector/heartbeat_existing
                        time:   [28.778 ns 29.027 ns 29.368 ns]
                        thrpt:  [34.051 Melem/s 34.451 Melem/s 34.748 Melem/s]
                 change:
                        time:   [−1.4950% +0.1699% +1.7606%] (p = 0.84 > 0.05)
                        thrpt:  [−1.7302% −0.1696% +1.5177%]
                        No change in performance detected.
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) high mild
  16 (16.00%) high severe
failure_detector/heartbeat_new
                        time:   [238.71 ns 242.62 ns 246.37 ns]
                        thrpt:  [4.0589 Melem/s 4.1216 Melem/s 4.1892 Melem/s]
                 change:
                        time:   [−9.2181% −4.5856% +0.3713%] (p = 0.07 > 0.05)
                        thrpt:  [−0.3699% +4.8059% +10.154%]
                        No change in performance detected.
failure_detector/status_check
                        time:   [14.535 ns 14.742 ns 14.929 ns]
                        thrpt:  [66.983 Melem/s 67.832 Melem/s 68.801 Melem/s]
                 change:
                        time:   [−1.8326% +0.3673% +2.5070%] (p = 0.74 > 0.05)
                        thrpt:  [−2.4457% −0.3660% +1.8668%]
                        No change in performance detected.
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 35.2s, or reduce sample count to 10.
failure_detector/check_all
                        time:   [342.60 ms 342.96 ms 343.44 ms]
                        thrpt:  [2.9117  elem/s 2.9158  elem/s 2.9188  elem/s]
                 change:
                        time:   [−0.2455% −0.1240% +0.0102%] (p = 0.07 > 0.05)
                        thrpt:  [−0.0102% +0.1242% +0.2461%]
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.1s, or reduce sample count to 60.
failure_detector/stats  time:   [80.531 ms 80.568 ms 80.607 ms]
                        thrpt:  [12.406  elem/s 12.412  elem/s 12.418  elem/s]
                 change:
                        time:   [−0.0582% +0.0113% +0.0781%] (p = 0.75 > 0.05)
                        thrpt:  [−0.0780% −0.0113% +0.0582%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

loss_simulator/should_drop_1pct
                        time:   [2.7943 ns 2.7977 ns 2.8016 ns]
                        thrpt:  [356.93 Melem/s 357.44 Melem/s 357.87 Melem/s]
                 change:
                        time:   [−0.4041% −0.2200% −0.0535%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0535% +0.2205% +0.4057%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
loss_simulator/should_drop_5pct
                        time:   [3.1541 ns 3.1565 ns 3.1592 ns]
                        thrpt:  [316.54 Melem/s 316.80 Melem/s 317.05 Melem/s]
                 change:
                        time:   [−0.3457% −0.1497% +0.0390%] (p = 0.13 > 0.05)
                        thrpt:  [−0.0390% +0.1499% +0.3469%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
loss_simulator/should_drop_10pct
                        time:   [3.6227 ns 3.6260 ns 3.6296 ns]
                        thrpt:  [275.52 Melem/s 275.79 Melem/s 276.04 Melem/s]
                 change:
                        time:   [−0.3036% −0.1406% +0.0369%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0368% +0.1408% +0.3046%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
loss_simulator/should_drop_20pct
                        time:   [4.5760 ns 4.5804 ns 4.5852 ns]
                        thrpt:  [218.09 Melem/s 218.32 Melem/s 218.53 Melem/s]
                 change:
                        time:   [−0.5053% −0.2578% +0.0082%] (p = 0.06 > 0.05)
                        thrpt:  [−0.0082% +0.2584% +0.5079%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
loss_simulator/should_drop_burst
                        time:   [2.9295 ns 2.9330 ns 2.9366 ns]
                        thrpt:  [340.53 Melem/s 340.95 Melem/s 341.35 Melem/s]
                 change:
                        time:   [−0.5511% −0.3553% −0.1499%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1501% +0.3565% +0.5542%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

circuit_breaker/allow_closed
                        time:   [13.430 ns 13.439 ns 13.447 ns]
                        thrpt:  [74.365 Melem/s 74.412 Melem/s 74.458 Melem/s]
                 change:
                        time:   [−0.4061% −0.2213% −0.0234%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0234% +0.2218% +0.4078%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
circuit_breaker/record_success
                        time:   [9.6481 ns 9.6538 ns 9.6598 ns]
                        thrpt:  [103.52 Melem/s 103.59 Melem/s 103.65 Melem/s]
                 change:
                        time:   [−0.2667% −0.1019% +0.0626%] (p = 0.24 > 0.05)
                        thrpt:  [−0.0626% +0.1020% +0.2674%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
circuit_breaker/record_failure
                        time:   [9.6006 ns 9.6089 ns 9.6173 ns]
                        thrpt:  [103.98 Melem/s 104.07 Melem/s 104.16 Melem/s]
                 change:
                        time:   [−0.4486% −0.1605% +0.1456%] (p = 0.33 > 0.05)
                        thrpt:  [−0.1454% +0.1608% +0.4506%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
circuit_breaker/state   time:   [13.426 ns 13.437 ns 13.447 ns]
                        thrpt:  [74.364 Melem/s 74.423 Melem/s 74.481 Melem/s]
                 change:
                        time:   [−0.4325% −0.2607% −0.0950%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0951% +0.2614% +0.4343%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe

recovery_manager/on_failure_with_alternates
                        time:   [247.30 ns 251.67 ns 255.63 ns]
                        thrpt:  [3.9119 Melem/s 3.9734 Melem/s 4.0436 Melem/s]
                 change:
                        time:   [−9.7249% −3.5486% +3.5173%] (p = 0.33 > 0.05)
                        thrpt:  [−3.3978% +3.6791% +10.772%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
recovery_manager/on_failure_no_alternates
                        time:   [288.96 ns 322.50 ns 359.71 ns]
                        thrpt:  [2.7800 Melem/s 3.1008 Melem/s 3.4606 Melem/s]
                 change:
                        time:   [−9.6025% −1.1401% +8.4868%] (p = 0.81 > 0.05)
                        thrpt:  [−7.8229% +1.1532% +10.623%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
recovery_manager/get_action
                        time:   [36.744 ns 37.080 ns 37.411 ns]
                        thrpt:  [26.730 Melem/s 26.968 Melem/s 27.215 Melem/s]
                 change:
                        time:   [−4.7296% −3.8947% −3.0678%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1649% +4.0525% +4.9644%]
                        Performance has improved.
recovery_manager/is_failed
                        time:   [13.241 ns 13.599 ns 13.955 ns]
                        thrpt:  [71.660 Melem/s 73.536 Melem/s 75.521 Melem/s]
                 change:
                        time:   [−3.5775% −0.5504% +2.3764%] (p = 0.72 > 0.05)
                        thrpt:  [−2.3212% +0.5535% +3.7103%]
                        No change in performance detected.
recovery_manager/on_recovery
                        time:   [105.92 ns 106.20 ns 106.54 ns]
                        thrpt:  [9.3866 Melem/s 9.4162 Melem/s 9.4410 Melem/s]
                 change:
                        time:   [−0.8231% −0.3115% +0.3125%] (p = 0.30 > 0.05)
                        thrpt:  [−0.3116% +0.3125% +0.8299%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) high mild
  9 (9.00%) high severe
recovery_manager/stats  time:   [700.65 ps 701.68 ps 703.30 ps]
                        thrpt:  [1.4219 Gelem/s 1.4251 Gelem/s 1.4273 Gelem/s]
                 change:
                        time:   [−0.5235% −0.3354% −0.1456%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1458% +0.3366% +0.5262%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

failure_scaling/check_all/100
                        time:   [4.7857 µs 4.8056 µs 4.8207 µs]
                        thrpt:  [20.744 Melem/s 20.809 Melem/s 20.895 Melem/s]
                 change:
                        time:   [−2.0051% −0.1810% +1.5780%] (p = 0.85 > 0.05)
                        thrpt:  [−1.5535% +0.1814% +2.0461%]
                        No change in performance detected.
failure_scaling/healthy_nodes/100
                        time:   [1.7147 µs 1.7158 µs 1.7170 µs]
                        thrpt:  [58.243 Melem/s 58.281 Melem/s 58.319 Melem/s]
                 change:
                        time:   [−0.7012% −0.5437% −0.3990%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4006% +0.5466% +0.7061%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
failure_scaling/check_all/500
                        time:   [20.826 µs 20.892 µs 20.943 µs]
                        thrpt:  [23.874 Melem/s 23.933 Melem/s 24.009 Melem/s]
                 change:
                        time:   [−2.0988% −0.0720% +1.9771%] (p = 0.95 > 0.05)
                        thrpt:  [−1.9387% +0.0720% +2.1438%]
                        No change in performance detected.
failure_scaling/healthy_nodes/500
                        time:   [5.4341 µs 5.4386 µs 5.4438 µs]
                        thrpt:  [91.848 Melem/s 91.936 Melem/s 92.011 Melem/s]
                 change:
                        time:   [−0.4241% −0.2487% −0.0818%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0819% +0.2494% +0.4259%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
failure_scaling/check_all/1000
                        time:   [40.812 µs 40.976 µs 41.103 µs]
                        thrpt:  [24.329 Melem/s 24.405 Melem/s 24.502 Melem/s]
                 change:
                        time:   [−2.3427% −0.2879% +1.9522%] (p = 0.79 > 0.05)
                        thrpt:  [−1.9148% +0.2887% +2.3989%]
                        No change in performance detected.
failure_scaling/healthy_nodes/1000
                        time:   [10.212 µs 10.222 µs 10.234 µs]
                        thrpt:  [97.711 Melem/s 97.831 Melem/s 97.921 Melem/s]
                 change:
                        time:   [−0.7608% −0.3095% +0.0530%] (p = 0.15 > 0.05)
                        thrpt:  [−0.0529% +0.3105% +0.7666%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
failure_scaling/check_all/5000
                        time:   [203.07 µs 203.50 µs 203.85 µs]
                        thrpt:  [24.527 Melem/s 24.570 Melem/s 24.622 Melem/s]
                 change:
                        time:   [−0.5832% −0.3253% −0.0738%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0739% +0.3264% +0.5866%]
                        Change within noise threshold.
failure_scaling/healthy_nodes/5000
                        time:   [48.807 µs 48.838 µs 48.869 µs]
                        thrpt:  [102.31 Melem/s 102.38 Melem/s 102.44 Melem/s]
                 change:
                        time:   [−2.7161% −2.5354% −2.3343%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3901% +2.6014% +2.7919%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe

failure_concurrent/concurrent_heartbeat/4
                        time:   [186.05 µs 186.32 µs 186.59 µs]
                        thrpt:  [10.719 Melem/s 10.734 Melem/s 10.750 Melem/s]
                 change:
                        time:   [+1.2065% +1.7278% +2.2904%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2391% −1.6984% −1.1921%]
                        Performance has regressed.
failure_concurrent/concurrent_heartbeat/8
                        time:   [253.57 µs 254.05 µs 254.44 µs]
                        thrpt:  [15.721 Melem/s 15.745 Melem/s 15.775 Melem/s]
                 change:
                        time:   [−2.8222% −2.0878% −1.2100%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2248% +2.1323% +2.9042%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
failure_concurrent/concurrent_heartbeat/16
                        time:   [465.88 µs 466.77 µs 467.32 µs]
                        thrpt:  [17.119 Melem/s 17.139 Melem/s 17.172 Melem/s]
                 change:
                        time:   [−0.8331% −0.5579% −0.2602%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2609% +0.5610% +0.8401%]
                        Change within noise threshold.

failure_recovery_cycle/full_cycle
                        time:   [282.25 ns 287.97 ns 292.96 ns]
                        thrpt:  [3.4134 Melem/s 3.4726 Melem/s 3.5429 Melem/s]
                 change:
                        time:   [−9.8273% −5.4621% −0.8343%] (p = 0.02 < 0.05)
                        thrpt:  [+0.8414% +5.7777% +10.898%]
                        Change within noise threshold.

capability_set/create   time:   [532.82 ns 533.54 ns 534.30 ns]
                        thrpt:  [1.8716 Melem/s 1.8743 Melem/s 1.8768 Melem/s]
                 change:
                        time:   [+2.0142% +2.4753% +2.8952%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8137% −2.4155% −1.9744%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_set/serialize
                        time:   [927.33 ns 930.29 ns 933.79 ns]
                        thrpt:  [1.0709 Melem/s 1.0749 Melem/s 1.0784 Melem/s]
                 change:
                        time:   [+0.2887% +0.5986% +0.8548%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8475% −0.5950% −0.2879%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
capability_set/deserialize
                        time:   [1.7155 µs 1.7227 µs 1.7313 µs]
                        thrpt:  [577.60 Kelem/s 580.48 Kelem/s 582.92 Kelem/s]
                 change:
                        time:   [−0.2387% +0.2118% +0.6621%] (p = 0.38 > 0.05)
                        thrpt:  [−0.6578% −0.2113% +0.2393%]
                        No change in performance detected.
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe
capability_set/roundtrip
                        time:   [2.6859 µs 2.6938 µs 2.7032 µs]
                        thrpt:  [369.93 Kelem/s 371.22 Kelem/s 372.31 Kelem/s]
                 change:
                        time:   [+0.4120% +0.7435% +1.1232%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1107% −0.7380% −0.4103%]
                        Change within noise threshold.
Found 19 outliers among 100 measurements (19.00%)
  5 (5.00%) high mild
  14 (14.00%) high severe
capability_set/has_tag  time:   [746.52 ps 747.24 ps 748.28 ps]
                        thrpt:  [1.3364 Gelem/s 1.3383 Gelem/s 1.3396 Gelem/s]
                 change:
                        time:   [−0.6157% −0.4610% −0.3069%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3079% +0.4632% +0.6195%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
capability_set/has_model
                        time:   [933.85 ps 934.44 ps 935.05 ps]
                        thrpt:  [1.0695 Gelem/s 1.0702 Gelem/s 1.0708 Gelem/s]
                 change:
                        time:   [−0.6079% −0.3951% −0.2275%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2280% +0.3967% +0.6116%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
capability_set/has_tool time:   [747.29 ps 747.85 ps 748.46 ps]
                        thrpt:  [1.3361 Gelem/s 1.3372 Gelem/s 1.3382 Gelem/s]
                 change:
                        time:   [−0.4467% −0.3108% −0.1597%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1599% +0.3118% +0.4487%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
capability_set/has_gpu  time:   [311.32 ps 311.50 ps 311.68 ps]
                        thrpt:  [3.2084 Gelem/s 3.2103 Gelem/s 3.2122 Gelem/s]
                 change:
                        time:   [−0.5089% −0.3624% −0.2096%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2100% +0.3637% +0.5115%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

capability_announcement/create
                        time:   [373.63 ns 374.61 ns 375.90 ns]
                        thrpt:  [2.6603 Melem/s 2.6695 Melem/s 2.6764 Melem/s]
                 change:
                        time:   [−0.9927% −0.7142% −0.4417%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4437% +0.7193% +1.0026%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
capability_announcement/serialize
                        time:   [1.2327 µs 1.2338 µs 1.2350 µs]
                        thrpt:  [809.70 Kelem/s 810.48 Kelem/s 811.24 Kelem/s]
                 change:
                        time:   [+0.7025% +0.9391% +1.1686%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1551% −0.9303% −0.6976%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
capability_announcement/deserialize
                        time:   [2.0830 µs 2.0844 µs 2.0859 µs]
                        thrpt:  [479.41 Kelem/s 479.75 Kelem/s 480.07 Kelem/s]
                 change:
                        time:   [−1.7392% −1.2277% −0.7276%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7330% +1.2430% +1.7700%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
capability_announcement/is_expired
                        time:   [25.232 ns 25.249 ns 25.267 ns]
                        thrpt:  [39.578 Melem/s 39.605 Melem/s 39.632 Melem/s]
                 change:
                        time:   [−0.5101% −0.3154% −0.1139%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1140% +0.3164% +0.5127%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

capability_filter/match_single_tag
                        time:   [9.9611 ns 9.9682 ns 9.9765 ns]
                        thrpt:  [100.24 Melem/s 100.32 Melem/s 100.39 Melem/s]
                 change:
                        time:   [−0.3856% −0.2590% −0.1202%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1203% +0.2597% +0.3871%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
capability_filter/match_require_gpu
                        time:   [4.0463 ns 4.0489 ns 4.0516 ns]
                        thrpt:  [246.81 Melem/s 246.98 Melem/s 247.14 Melem/s]
                 change:
                        time:   [−0.4783% −0.2352% +0.0501%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0500% +0.2358% +0.4806%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
capability_filter/match_gpu_vendor
                        time:   [3.7357 ns 3.7378 ns 3.7400 ns]
                        thrpt:  [267.38 Melem/s 267.54 Melem/s 267.69 Melem/s]
                 change:
                        time:   [−0.5348% −0.3582% −0.2091%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2095% +0.3595% +0.5377%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
capability_filter/match_min_memory
                        time:   [3.7363 ns 3.7389 ns 3.7416 ns]
                        thrpt:  [267.27 Melem/s 267.46 Melem/s 267.64 Melem/s]
                 change:
                        time:   [−0.4883% −0.2988% −0.1347%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1349% +0.2997% +0.4907%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
capability_filter/match_complex
                        time:   [10.273 ns 10.279 ns 10.286 ns]
                        thrpt:  [97.222 Melem/s 97.282 Melem/s 97.339 Melem/s]
                 change:
                        time:   [−3.5543% −3.2672% −3.0518%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1479% +3.3776% +3.6853%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
capability_filter/match_no_match
                        time:   [3.1138 ns 3.1157 ns 3.1178 ns]
                        thrpt:  [320.74 Melem/s 320.95 Melem/s 321.15 Melem/s]
                 change:
                        time:   [−0.5199% −0.3389% −0.1625%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1627% +0.3401% +0.5226%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

capability_index_insert/index_nodes/100
                        time:   [111.27 µs 111.54 µs 111.86 µs]
                        thrpt:  [893.96 Kelem/s 896.51 Kelem/s 898.72 Kelem/s]
                 change:
                        time:   [−7.7042% −6.2582% −4.8764%] (p = 0.00 < 0.05)
                        thrpt:  [+5.1264% +6.6760% +8.3473%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.9s, enable flat sampling, or reduce sample count to 60.
capability_index_insert/index_nodes/1000
                        time:   [1.1712 ms 1.1739 ms 1.1768 ms]
                        thrpt:  [849.78 Kelem/s 851.86 Kelem/s 853.81 Kelem/s]
                 change:
                        time:   [−11.113% −9.0133% −7.0132%] (p = 0.00 < 0.05)
                        thrpt:  [+7.5421% +9.9061% +12.502%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
capability_index_insert/index_nodes/10000
                        time:   [17.087 ms 17.241 ms 17.401 ms]
                        thrpt:  [574.68 Kelem/s 580.00 Kelem/s 585.24 Kelem/s]
                 change:
                        time:   [−8.8964% −7.2774% −5.6263%] (p = 0.00 < 0.05)
                        thrpt:  [+5.9617% +7.8486% +9.7651%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

capability_index_query/query_single_tag
                        time:   [235.30 µs 253.51 µs 272.34 µs]
                        thrpt:  [3.6719 Kelem/s 3.9446 Kelem/s 4.2499 Kelem/s]
                 change:
                        time:   [+27.234% +37.521% +47.989%] (p = 0.00 < 0.05)
                        thrpt:  [−32.428% −27.284% −21.405%]
                        Performance has regressed.
capability_index_query/query_require_gpu
                        time:   [304.70 µs 325.43 µs 344.57 µs]
                        thrpt:  [2.9022 Kelem/s 3.0728 Kelem/s 3.2819 Kelem/s]
                 change:
                        time:   [+22.381% +31.466% +41.389%] (p = 0.00 < 0.05)
                        thrpt:  [−29.273% −23.935% −18.288%]
                        Performance has regressed.
capability_index_query/query_gpu_vendor
                        time:   [810.85 µs 827.10 µs 842.51 µs]
                        thrpt:  [1.1869 Kelem/s 1.2090 Kelem/s 1.2333 Kelem/s]
                 change:
                        time:   [+6.7972% +10.258% +13.367%] (p = 0.00 < 0.05)
                        thrpt:  [−11.791% −9.3037% −6.3646%]
                        Performance has regressed.
capability_index_query/query_min_memory
                        time:   [820.91 µs 841.27 µs 861.17 µs]
                        thrpt:  [1.1612 Kelem/s 1.1887 Kelem/s 1.2182 Kelem/s]
                 change:
                        time:   [+9.6431% +12.433% +15.043%] (p = 0.00 < 0.05)
                        thrpt:  [−13.076% −11.058% −8.7950%]
                        Performance has regressed.
capability_index_query/query_complex
                        time:   [590.51 µs 611.13 µs 629.90 µs]
                        thrpt:  [1.5875 Kelem/s 1.6363 Kelem/s 1.6934 Kelem/s]
                 change:
                        time:   [+18.959% +25.520% +31.134%] (p = 0.00 < 0.05)
                        thrpt:  [−23.742% −20.332% −15.937%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
capability_index_query/query_model
                        time:   [98.936 µs 99.107 µs 99.277 µs]
                        thrpt:  [10.073 Kelem/s 10.090 Kelem/s 10.108 Kelem/s]
                 change:
                        time:   [−1.4550% −1.0198% −0.6367%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6408% +1.0303% +1.4765%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
capability_index_query/query_tool
                        time:   [743.10 µs 771.71 µs 799.59 µs]
                        thrpt:  [1.2506 Kelem/s 1.2958 Kelem/s 1.3457 Kelem/s]
                 change:
                        time:   [+17.259% +23.531% +29.802%] (p = 0.00 < 0.05)
                        thrpt:  [−22.960% −19.049% −14.718%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
capability_index_query/query_no_results
                        time:   [21.982 ns 22.229 ns 22.478 ns]
                        thrpt:  [44.488 Melem/s 44.987 Melem/s 45.491 Melem/s]
                 change:
                        time:   [−3.7567% −2.8730% −1.9816%] (p = 0.00 < 0.05)
                        thrpt:  [+2.0217% +2.9579% +3.9034%]
                        Performance has improved.

capability_index_find_best/find_best_simple
                        time:   [312.34 µs 313.38 µs 314.41 µs]
                        thrpt:  [3.1806 Kelem/s 3.1910 Kelem/s 3.2016 Kelem/s]
                 change:
                        time:   [−24.999% −22.503% −19.927%] (p = 0.00 < 0.05)
                        thrpt:  [+24.886% +29.037% +33.332%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 6.6776 s (10k iterationscapability_index_find_best/find_best_with_prefs
                        time:   [636.04 µs 653.69 µs 671.55 µs]
                        thrpt:  [1.4891 Kelem/s 1.5298 Kelem/s 1.5722 Kelem/s]
                 change:
                        time:   [−8.8941% −4.9628% −0.8061%] (p = 0.02 < 0.05)
                        thrpt:  [+0.8127% +5.2220% +9.7624%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

capability_index_scaling/query_tag/1000
                        time:   [12.525 µs 12.534 µs 12.543 µs]
                        thrpt:  [79.727 Kelem/s 79.785 Kelem/s 79.841 Kelem/s]
                 change:
                        time:   [−0.2441% −0.0838% +0.0862%] (p = 0.32 > 0.05)
                        thrpt:  [−0.0862% +0.0839% +0.2447%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
capability_index_scaling/query_complex/1000
                        time:   [40.311 µs 40.487 µs 40.656 µs]
                        thrpt:  [24.597 Kelem/s 24.699 Kelem/s 24.807 Kelem/s]
                 change:
                        time:   [−0.5581% +0.1472% +0.7936%] (p = 0.68 > 0.05)
                        thrpt:  [−0.7873% −0.1469% +0.5612%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
capability_index_scaling/query_tag/5000
                        time:   [70.112 µs 70.269 µs 70.415 µs]
                        thrpt:  [14.202 Kelem/s 14.231 Kelem/s 14.263 Kelem/s]
                 change:
                        time:   [−1.0096% −0.5284% −0.0982%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0983% +0.5312% +1.0199%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
capability_index_scaling/query_complex/5000
                        time:   [286.65 µs 302.95 µs 318.95 µs]
                        thrpt:  [3.1352 Kelem/s 3.3009 Kelem/s 3.4885 Kelem/s]
                 change:
                        time:   [+33.984% +41.458% +48.985%] (p = 0.00 < 0.05)
                        thrpt:  [−32.879% −29.308% −25.364%]
                        Performance has regressed.
capability_index_scaling/query_tag/10000
                        time:   [154.12 µs 154.98 µs 155.86 µs]
                        thrpt:  [6.4160 Kelem/s 6.4525 Kelem/s 6.4886 Kelem/s]
                 change:
                        time:   [−16.089% −12.791% −9.4058%] (p = 0.00 < 0.05)
                        thrpt:  [+10.382% +14.667% +19.173%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
capability_index_scaling/query_complex/10000
                        time:   [581.90 µs 601.95 µs 620.67 µs]
                        thrpt:  [1.6112 Kelem/s 1.6613 Kelem/s 1.7185 Kelem/s]
                 change:
                        time:   [+28.197% +32.690% +37.271%] (p = 0.00 < 0.05)
                        thrpt:  [−27.151% −24.636% −21.995%]
                        Performance has regressed.
Found 13 outliers among 100 measurements (13.00%)
  11 (11.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_index_scaling/query_tag/50000
                        time:   [2.5223 ms 2.5575 ms 2.5925 ms]
                        thrpt:  [385.72  elem/s 391.00  elem/s 396.46  elem/s]
                 change:
                        time:   [+3.8865% +5.6543% +7.3278%] (p = 0.00 < 0.05)
                        thrpt:  [−6.8275% −5.3517% −3.7411%]
                        Performance has regressed.
capability_index_scaling/query_complex/50000
                        time:   [3.7709 ms 3.8061 ms 3.8413 ms]
                        thrpt:  [260.33  elem/s 262.73  elem/s 265.19  elem/s]
                 change:
                        time:   [+3.2612% +4.4025% +5.5645%] (p = 0.00 < 0.05)
                        thrpt:  [−5.2712% −4.2168% −3.1582%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

capability_index_concurrent/concurrent_index/4
                        time:   [392.69 µs 393.20 µs 393.67 µs]
                        thrpt:  [5.0803 Melem/s 5.0865 Melem/s 5.0930 Melem/s]
                 change:
                        time:   [−0.7670% −0.4099% +0.0125%] (p = 0.04 < 0.05)
                        thrpt:  [−0.0125% +0.4116% +0.7729%]
                        Change within noise threshold.
Found 4 outliers among 20 measurements (20.00%)
  3 (15.00%) low mild
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.8s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/4
                        time:   [380.59 ms 387.20 ms 394.61 ms]
                        thrpt:  [5.0683 Kelem/s 5.1653 Kelem/s 5.2550 Kelem/s]
                 change:
                        time:   [−6.9346% −5.3085% −3.4559%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5796% +5.6061% +7.4514%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
capability_index_concurrent/concurrent_mixed/4
                        time:   [149.76 ms 151.74 ms 153.58 ms]
                        thrpt:  [13.022 Kelem/s 13.180 Kelem/s 13.354 Kelem/s]
                 change:
                        time:   [−18.772% −16.542% −14.181%] (p = 0.00 < 0.05)
                        thrpt:  [+16.524% +19.821% +23.110%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
capability_index_concurrent/concurrent_index/8
                        time:   [446.10 µs 446.81 µs 447.60 µs]
                        thrpt:  [8.9365 Melem/s 8.9523 Melem/s 8.9666 Melem/s]
                 change:
                        time:   [−5.4713% −4.7691% −4.0698%] (p = 0.00 < 0.05)
                        thrpt:  [+4.2424% +5.0079% +5.7880%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 12.3s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/8
                        time:   [600.18 ms 604.06 ms 607.82 ms]
                        thrpt:  [6.5809 Kelem/s 6.6218 Kelem/s 6.6646 Kelem/s]
                 change:
                        time:   [+14.013% +15.078% +16.122%] (p = 0.00 < 0.05)
                        thrpt:  [−13.884% −13.102% −12.290%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low severe
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.7s, or reduce sample count to 10.
capability_index_concurrent/concurrent_mixed/8
                        time:   [276.74 ms 281.64 ms 286.70 ms]
                        thrpt:  [13.952 Kelem/s 14.203 Kelem/s 14.454 Kelem/s]
                 change:
                        time:   [+5.9009% +8.4745% +10.822%] (p = 0.00 < 0.05)
                        thrpt:  [−9.7653% −7.8125% −5.5721%]
                        Performance has regressed.
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.1707 s (5250 iterationscapability_index_concurrent/concurrent_index/16
                        time:   [980.40 µs 982.61 µs 985.17 µs]
                        thrpt:  [8.1205 Melem/s 8.1416 Melem/s 8.1600 Melem/s]
                 change:
                        time:   [+3.0796% +3.4940% +3.9579%] (p = 0.00 < 0.05)
                        thrpt:  [−3.8072% −3.3760% −2.9876%]
                        Performance has regressed.
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) high mild
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 20.5s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/16
                        time:   [1.0168 s 1.0196 s 1.0229 s]
                        thrpt:  [7.8211 Kelem/s 7.8460 Kelem/s 7.8675 Kelem/s]
                 change:
                        time:   [−3.0575% −2.4704% −1.8787%] (p = 0.00 < 0.05)
                        thrpt:  [+1.9147% +2.5330% +3.1539%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.8s, or reduce sample count to 10.
capability_index_concurrent/concurrent_mixed/16
                        time:   [541.43 ms 543.31 ms 545.51 ms]
                        thrpt:  [14.665 Kelem/s 14.724 Kelem/s 14.776 Kelem/s]
                 change:
                        time:   [−1.7441% −1.2543% −0.6831%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6878% +1.2702% +1.7750%]
                        Change within noise threshold.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.0040 s (5.5M iterationscapability_index_updates/update_higher_version
                        time:   [563.34 ns 564.65 ns 565.99 ns]
                        thrpt:  [1.7668 Melem/s 1.7710 Melem/s 1.7751 Melem/s]
                 change:
                        time:   [−0.5085% +0.0433% +0.5226%] (p = 0.87 > 0.05)
                        thrpt:  [−0.5199% −0.0433% +0.5111%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_index_updates/update_same_version
                        time:   [560.75 ns 561.22 ns 561.67 ns]
                        thrpt:  [1.7804 Melem/s 1.7818 Melem/s 1.7833 Melem/s]
                 change:
                        time:   [−0.1264% +0.1055% +0.3327%] (p = 0.37 > 0.05)
                        thrpt:  [−0.3316% −0.1054% +0.1266%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
capability_index_updates/remove_and_readd
                        time:   [1.3266 µs 1.3333 µs 1.3403 µs]
                        thrpt:  [746.12 Kelem/s 750.03 Kelem/s 753.83 Kelem/s]
                 change:
                        time:   [−5.2447% −4.6475% −4.0121%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1798% +4.8740% +5.5350%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild

diff_op/create_add_tag  time:   [14.517 ns 14.594 ns 14.681 ns]
                        thrpt:  [68.116 Melem/s 68.521 Melem/s 68.885 Melem/s]
                 change:
                        time:   [−3.0499% −2.3518% −1.6326%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6597% +2.4085% +3.1458%]
                        Performance has improved.
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) high mild
  3 (3.00%) high severe
diff_op/create_remove_tag
                        time:   [15.206 ns 15.329 ns 15.447 ns]
                        thrpt:  [64.739 Melem/s 65.237 Melem/s 65.765 Melem/s]
                 change:
                        time:   [+0.9573% +1.9379% +2.8221%] (p = 0.00 < 0.05)
                        thrpt:  [−2.7446% −1.9011% −0.9482%]
                        Change within noise threshold.
diff_op/create_add_model
                        time:   [48.605 ns 48.651 ns 48.698 ns]
                        thrpt:  [20.535 Melem/s 20.555 Melem/s 20.574 Melem/s]
                 change:
                        time:   [−0.1816% +0.0612% +0.2886%] (p = 0.61 > 0.05)
                        thrpt:  [−0.2878% −0.0612% +0.1820%]
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low mild
  10 (10.00%) high mild
  4 (4.00%) high severe
diff_op/create_update_model
                        time:   [15.156 ns 15.251 ns 15.343 ns]
                        thrpt:  [65.177 Melem/s 65.568 Melem/s 65.979 Melem/s]
                 change:
                        time:   [−0.5572% +0.2008% +0.9964%] (p = 0.62 > 0.05)
                        thrpt:  [−0.9866% −0.2004% +0.5603%]
                        No change in performance detected.
diff_op/estimated_size  time:   [1.8686 ns 1.8701 ns 1.8718 ns]
                        thrpt:  [534.23 Melem/s 534.72 Melem/s 535.17 Melem/s]
                 change:
                        time:   [−2.8611% −2.7024% −2.5424%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6088% +2.7775% +2.9453%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

capability_diff/create  time:   [100.06 ns 100.47 ns 100.88 ns]
                        thrpt:  [9.9124 Melem/s 9.9529 Melem/s 9.9944 Melem/s]
                 change:
                        time:   [−3.7567% −3.1266% −2.4788%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5418% +3.2275% +3.9033%]
                        Performance has improved.
capability_diff/serialize
                        time:   [156.77 ns 156.87 ns 156.98 ns]
                        thrpt:  [6.3703 Melem/s 6.3746 Melem/s 6.3786 Melem/s]
                 change:
                        time:   [+2.5625% +2.8137% +3.0794%] (p = 0.00 < 0.05)
                        thrpt:  [−2.9874% −2.7367% −2.4984%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe
capability_diff/deserialize
                        time:   [242.31 ns 243.09 ns 243.95 ns]
                        thrpt:  [4.0992 Melem/s 4.1136 Melem/s 4.1269 Melem/s]
                 change:
                        time:   [−1.0579% −0.6837% −0.3090%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3099% +0.6884% +1.0692%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_diff/estimated_size
                        time:   [5.2924 ns 5.2955 ns 5.2989 ns]
                        thrpt:  [188.72 Melem/s 188.84 Melem/s 188.95 Melem/s]
                 change:
                        time:   [−0.4860% −0.2683% −0.0565%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0565% +0.2690% +0.4883%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe

diff_generation/no_changes
                        time:   [344.33 ns 344.54 ns 344.76 ns]
                        thrpt:  [2.9005 Melem/s 2.9024 Melem/s 2.9042 Melem/s]
                 change:
                        time:   [−1.2348% −0.9694% −0.7190%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7242% +0.9789% +1.2502%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
diff_generation/add_one_tag
                        time:   [454.83 ns 455.20 ns 455.57 ns]
                        thrpt:  [2.1951 Melem/s 2.1968 Melem/s 2.1986 Melem/s]
                 change:
                        time:   [−0.5386% −0.1675% +0.2250%] (p = 0.39 > 0.05)
                        thrpt:  [−0.2245% +0.1678% +0.5415%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
diff_generation/multiple_tag_changes
                        time:   [514.02 ns 514.82 ns 515.65 ns]
                        thrpt:  [1.9393 Melem/s 1.9424 Melem/s 1.9455 Melem/s]
                 change:
                        time:   [−0.6818% −0.3730% −0.0785%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0786% +0.3744% +0.6865%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
diff_generation/update_model_loaded
                        time:   [406.57 ns 407.17 ns 407.82 ns]
                        thrpt:  [2.4521 Melem/s 2.4560 Melem/s 2.4596 Melem/s]
                 change:
                        time:   [−0.2999% +0.0152% +0.3172%] (p = 0.92 > 0.05)
                        thrpt:  [−0.3162% −0.0152% +0.3008%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
diff_generation/update_memory
                        time:   [393.12 ns 393.42 ns 393.75 ns]
                        thrpt:  [2.5397 Melem/s 2.5418 Melem/s 2.5438 Melem/s]
                 change:
                        time:   [−0.8822% −0.5756% −0.2806%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2814% +0.5790% +0.8900%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
diff_generation/add_model
                        time:   [486.15 ns 486.45 ns 486.74 ns]
                        thrpt:  [2.0545 Melem/s 2.0557 Melem/s 2.0570 Melem/s]
                 change:
                        time:   [−0.7282% −0.4918% −0.2604%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2610% +0.4942% +0.7336%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
diff_generation/complex_diff
                        time:   [778.51 ns 779.13 ns 779.77 ns]
                        thrpt:  [1.2824 Melem/s 1.2835 Melem/s 1.2845 Melem/s]
                 change:
                        time:   [−1.1176% −0.7742% −0.4762%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4785% +0.7803% +1.1302%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe

diff_application/apply_single_op
                        time:   [360.83 ns 361.65 ns 362.60 ns]
                        thrpt:  [2.7579 Melem/s 2.7651 Melem/s 2.7714 Melem/s]
                 change:
                        time:   [−1.0361% −0.7594% −0.4919%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4944% +0.7652% +1.0469%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
diff_application/apply_small_diff
                        time:   [365.23 ns 365.72 ns 366.30 ns]
                        thrpt:  [2.7300 Melem/s 2.7343 Melem/s 2.7380 Melem/s]
                 change:
                        time:   [−0.8445% −0.6335% −0.4176%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4193% +0.6375% +0.8517%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  5 (5.00%) high severe
diff_application/apply_medium_diff
                        time:   [838.15 ns 838.84 ns 839.54 ns]
                        thrpt:  [1.1911 Melem/s 1.1921 Melem/s 1.1931 Melem/s]
                 change:
                        time:   [−0.7191% −0.4434% −0.0948%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0949% +0.4453% +0.7243%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
diff_application/apply_strict_mode
                        time:   [362.87 ns 363.16 ns 363.44 ns]
                        thrpt:  [2.7515 Melem/s 2.7536 Melem/s 2.7558 Melem/s]
                 change:
                        time:   [−0.3332% −0.1481% +0.0383%] (p = 0.13 > 0.05)
                        thrpt:  [−0.0383% +0.1483% +0.3343%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  6 (6.00%) high severe

diff_chain_validation/validate_chain_10
                        time:   [6.0400 ns 6.0434 ns 6.0469 ns]
                        thrpt:  [165.38 Melem/s 165.47 Melem/s 165.56 Melem/s]
                 change:
                        time:   [−0.0573% +0.1647% +0.4202%] (p = 0.17 > 0.05)
                        thrpt:  [−0.4184% −0.1644% +0.0573%]
                        No change in performance detected.
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high severe
diff_chain_validation/validate_chain_100
                        time:   [67.851 ns 67.899 ns 67.947 ns]
                        thrpt:  [14.717 Melem/s 14.728 Melem/s 14.738 Melem/s]
                 change:
                        time:   [+0.2450% +0.4101% +0.5769%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5736% −0.4084% −0.2444%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe

diff_compaction/compact_5_diffs
                        time:   [3.2554 µs 3.2589 µs 3.2627 µs]
                        thrpt:  [306.50 Kelem/s 306.85 Kelem/s 307.18 Kelem/s]
                 change:
                        time:   [−0.5151% −0.3296% −0.1516%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1518% +0.3307% +0.5178%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
diff_compaction/compact_20_diffs
                        time:   [14.847 µs 14.877 µs 14.917 µs]
                        thrpt:  [67.037 Kelem/s 67.219 Kelem/s 67.355 Kelem/s]
                 change:
                        time:   [−0.4843% −0.2652% −0.0367%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0367% +0.2659% +0.4867%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

bandwidth_savings/calculate_small
                        time:   [918.89 ns 920.95 ns 923.36 ns]
                        thrpt:  [1.0830 Melem/s 1.0858 Melem/s 1.0883 Melem/s]
                 change:
                        time:   [+0.6559% +0.9954% +1.3819%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3631% −0.9856% −0.6516%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
bandwidth_savings/calculate_medium
                        time:   [922.54 ns 923.43 ns 924.33 ns]
                        thrpt:  [1.0819 Melem/s 1.0829 Melem/s 1.0840 Melem/s]
                 change:
                        time:   [+0.6417% +0.8010% +0.9554%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9463% −0.7946% −0.6376%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

diff_roundtrip/generate_apply_verify
                        time:   [1.3438 µs 1.3448 µs 1.3459 µs]
                        thrpt:  [742.98 Kelem/s 743.59 Kelem/s 744.18 Kelem/s]
                 change:
                        time:   [−0.3602% −0.2151% −0.0607%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0607% +0.2155% +0.3615%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

location_info/create    time:   [28.274 ns 28.333 ns 28.395 ns]
                        thrpt:  [35.217 Melem/s 35.294 Melem/s 35.368 Melem/s]
                 change:
                        time:   [−0.5981% −0.2763% +0.0492%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0492% +0.2770% +0.6017%]
                        No change in performance detected.
location_info/distance_to
                        time:   [3.9369 ns 3.9396 ns 3.9424 ns]
                        thrpt:  [253.65 Melem/s 253.83 Melem/s 254.01 Melem/s]
                 change:
                        time:   [−0.4608% −0.2472% −0.0231%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0231% +0.2478% +0.4629%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  8 (8.00%) high mild
  4 (4.00%) high severe
location_info/same_continent
                        time:   [7.1685 ns 7.1724 ns 7.1763 ns]
                        thrpt:  [139.35 Melem/s 139.42 Melem/s 139.50 Melem/s]
                 change:
                        time:   [−0.7695% −0.4569% −0.2095%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2100% +0.4590% +0.7755%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
location_info/same_continent_cross
                        time:   [311.70 ps 311.86 ps 312.03 ps]
                        thrpt:  [3.2048 Gelem/s 3.2065 Gelem/s 3.2083 Gelem/s]
                 change:
                        time:   [−0.4098% −0.2516% −0.0939%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0940% +0.2523% +0.4115%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
location_info/same_region
                        time:   [4.0521 ns 4.0545 ns 4.0569 ns]
                        thrpt:  [246.49 Melem/s 246.64 Melem/s 246.79 Melem/s]
                 change:
                        time:   [−1.6502% −1.3553% −1.0655%] (p = 0.00 < 0.05)
                        thrpt:  [+1.0770% +1.3739% +1.6779%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

topology_hints/create   time:   [3.2055 ns 3.2173 ns 3.2294 ns]
                        thrpt:  [309.65 Melem/s 310.82 Melem/s 311.97 Melem/s]
                 change:
                        time:   [−0.0709% +0.3821% +0.8136%] (p = 0.10 > 0.05)
                        thrpt:  [−0.8071% −0.3807% +0.0709%]
                        No change in performance detected.
topology_hints/connectivity_score
                        time:   [311.70 ps 311.86 ps 312.02 ps]
                        thrpt:  [3.2049 Gelem/s 3.2065 Gelem/s 3.2082 Gelem/s]
                 change:
                        time:   [−0.3143% −0.1561% −0.0024%] (p = 0.05 < 0.05)
                        thrpt:  [+0.0024% +0.1563% +0.3153%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
topology_hints/average_latency_empty
                        time:   [623.43 ps 623.77 ps 624.11 ps]
                        thrpt:  [1.6023 Gelem/s 1.6032 Gelem/s 1.6040 Gelem/s]
                 change:
                        time:   [−0.3295% −0.1794% −0.0246%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0246% +0.1798% +0.3306%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
topology_hints/average_latency_100
                        time:   [70.511 ns 70.548 ns 70.584 ns]
                        thrpt:  [14.168 Melem/s 14.175 Melem/s 14.182 Melem/s]
                 change:
                        time:   [−1.0684% −0.7879% −0.5510%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5540% +0.7942% +1.0799%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

nat_type/difficulty     time:   [311.66 ps 311.82 ps 311.98 ps]
                        thrpt:  [3.2053 Gelem/s 3.2070 Gelem/s 3.2086 Gelem/s]
                 change:
                        time:   [−0.3385% −0.1792% −0.0160%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0160% +0.1795% +0.3397%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
nat_type/can_connect_direct
                        time:   [311.68 ps 311.84 ps 312.00 ps]
                        thrpt:  [3.2051 Gelem/s 3.2068 Gelem/s 3.2085 Gelem/s]
                 change:
                        time:   [−0.4400% −0.2805% −0.1365%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1367% +0.2813% +0.4419%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
nat_type/can_connect_symmetric
                        time:   [311.73 ps 311.88 ps 312.03 ps]
                        thrpt:  [3.2048 Gelem/s 3.2063 Gelem/s 3.2079 Gelem/s]
                 change:
                        time:   [−0.4650% −0.2726% −0.0955%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0956% +0.2734% +0.4672%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

node_metadata/create_simple
                        time:   [44.855 ns 44.954 ns 45.063 ns]
                        thrpt:  [22.191 Melem/s 22.245 Melem/s 22.294 Melem/s]
                 change:
                        time:   [+3.5056% +6.2249% +9.1525%] (p = 0.00 < 0.05)
                        thrpt:  [−8.3851% −5.8601% −3.3869%]
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
node_metadata/create_full
                        time:   [357.67 ns 358.11 ns 358.51 ns]
                        thrpt:  [2.7893 Melem/s 2.7924 Melem/s 2.7959 Melem/s]
                 change:
                        time:   [−0.9296% −0.3786% +0.0986%] (p = 0.16 > 0.05)
                        thrpt:  [−0.0985% +0.3801% +0.9384%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
node_metadata/routing_score
                        time:   [2.8698 ns 2.8731 ns 2.8767 ns]
                        thrpt:  [347.62 Melem/s 348.06 Melem/s 348.45 Melem/s]
                 change:
                        time:   [−0.6574% −0.3410% −0.0767%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0768% +0.3422% +0.6618%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
node_metadata/age       time:   [27.478 ns 27.492 ns 27.506 ns]
                        thrpt:  [36.356 Melem/s 36.375 Melem/s 36.393 Melem/s]
                 change:
                        time:   [−0.5987% −0.3146% −0.1028%] (p = 0.01 < 0.05)
                        thrpt:  [+0.1029% +0.3155% +0.6023%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
node_metadata/is_stale  time:   [25.830 ns 25.850 ns 25.874 ns]
                        thrpt:  [38.649 Melem/s 38.685 Melem/s 38.714 Melem/s]
                 change:
                        time:   [−0.2886% −0.1188% +0.0520%] (p = 0.20 > 0.05)
                        thrpt:  [−0.0520% +0.1189% +0.2894%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
node_metadata/serialize time:   [751.68 ns 752.25 ns 752.87 ns]
                        thrpt:  [1.3282 Melem/s 1.3294 Melem/s 1.3304 Melem/s]
                 change:
                        time:   [+0.5612% +0.7087% +0.8575%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8502% −0.7037% −0.5581%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
node_metadata/deserialize
                        time:   [1.4404 µs 1.4531 µs 1.4651 µs]
                        thrpt:  [682.53 Kelem/s 688.19 Kelem/s 694.23 Kelem/s]
                 change:
                        time:   [+1.8220% +2.3401% +2.9086%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8264% −2.2866% −1.7894%]
                        Performance has regressed.

metadata_query/match_status
                        time:   [3.4293 ns 3.4311 ns 3.4329 ns]
                        thrpt:  [291.30 Melem/s 291.45 Melem/s 291.61 Melem/s]
                 change:
                        time:   [−0.3971% −0.2446% −0.0890%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0891% +0.2452% +0.3987%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
metadata_query/match_min_tier
                        time:   [3.4288 ns 3.4308 ns 3.4329 ns]
                        thrpt:  [291.30 Melem/s 291.48 Melem/s 291.65 Melem/s]
                 change:
                        time:   [−0.4268% −0.2362% −0.0541%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0541% +0.2367% +0.4287%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
metadata_query/match_continent
                        time:   [11.221 ns 11.227 ns 11.233 ns]
                        thrpt:  [89.026 Melem/s 89.072 Melem/s 89.119 Melem/s]
                 change:
                        time:   [−1.8502% −1.4726% −1.1254%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1382% +1.4946% +1.8851%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
metadata_query/match_complex
                        time:   [10.601 ns 10.606 ns 10.612 ns]
                        thrpt:  [94.235 Melem/s 94.283 Melem/s 94.331 Melem/s]
                 change:
                        time:   [−1.8117% −1.4139% −1.0459%] (p = 0.00 < 0.05)
                        thrpt:  [+1.0569% +1.4342% +1.8451%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
metadata_query/match_no_match
                        time:   [3.4283 ns 3.4303 ns 3.4324 ns]
                        thrpt:  [291.34 Melem/s 291.52 Melem/s 291.69 Melem/s]
                 change:
                        time:   [−0.3834% −0.2179% −0.0601%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0602% +0.2184% +0.3849%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

metadata_store_basic/create
                        time:   [769.04 ns 770.21 ns 771.46 ns]
                        thrpt:  [1.2962 Melem/s 1.2983 Melem/s 1.3003 Melem/s]
                 change:
                        time:   [+0.5254% +0.8087% +1.0873%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0756% −0.8022% −0.5227%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
metadata_store_basic/upsert_new
                        time:   [2.0475 µs 2.0720 µs 2.0948 µs]
                        thrpt:  [477.37 Kelem/s 482.62 Kelem/s 488.40 Kelem/s]
                 change:
                        time:   [−8.3254% −5.5915% −2.7906%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8707% +5.9227% +9.0815%]
                        Performance has improved.
metadata_store_basic/upsert_existing
                        time:   [1.1512 µs 1.1535 µs 1.1562 µs]
                        thrpt:  [864.90 Kelem/s 866.95 Kelem/s 868.64 Kelem/s]
                 change:
                        time:   [+0.6202% +0.9464% +1.3015%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2847% −0.9375% −0.6164%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  15 (15.00%) high mild
metadata_store_basic/get
                        time:   [26.246 ns 27.025 ns 27.785 ns]
                        thrpt:  [35.991 Melem/s 37.002 Melem/s 38.101 Melem/s]
                 change:
                        time:   [+3.0031% +6.2006% +9.5370%] (p = 0.00 < 0.05)
                        thrpt:  [−8.7067% −5.8386% −2.9155%]
                        Performance has regressed.
metadata_store_basic/get_miss
                        time:   [25.938 ns 26.598 ns 27.276 ns]
                        thrpt:  [36.662 Melem/s 37.596 Melem/s 38.553 Melem/s]
                 change:
                        time:   [+1.6094% +4.8551% +8.2077%] (p = 0.00 < 0.05)
                        thrpt:  [−7.5851% −4.6303% −1.5840%]
                        Performance has regressed.
metadata_store_basic/len
                        time:   [200.19 ns 200.43 ns 200.80 ns]
                        thrpt:  [4.9800 Melem/s 4.9892 Melem/s 4.9952 Melem/s]
                 change:
                        time:   [−0.2905% −0.1260% +0.0400%] (p = 0.14 > 0.05)
                        thrpt:  [−0.0400% +0.1261% +0.2914%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 20.2s, or reduce sample count to 20.
metadata_store_basic/stats
                        time:   [196.47 ms 198.01 ms 199.53 ms]
                        thrpt:  [5.0119  elem/s 5.0502  elem/s 5.0897  elem/s]
                 change:
                        time:   [−5.0504% −4.2696% −3.4621%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5862% +4.4600% +5.3190%]
                        Performance has improved.

metadata_store_query/query_by_status
                        time:   [247.51 µs 260.17 µs 273.61 µs]
                        thrpt:  [3.6548 Kelem/s 3.8437 Kelem/s 4.0403 Kelem/s]
                 change:
                        time:   [−27.992% −23.162% −17.813%] (p = 0.00 < 0.05)
                        thrpt:  [+21.674% +30.143% +38.873%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
metadata_store_query/query_by_continent
                        time:   [146.92 µs 147.12 µs 147.33 µs]
                        thrpt:  [6.7874 Kelem/s 6.7970 Kelem/s 6.8064 Kelem/s]
                 change:
                        time:   [−0.7620% +0.0074% +1.0093%] (p = 0.99 > 0.05)
                        thrpt:  [−0.9992% −0.0074% +0.7678%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
metadata_store_query/query_by_tier
                        time:   [533.05 µs 557.84 µs 580.72 µs]
                        thrpt:  [1.7220 Kelem/s 1.7926 Kelem/s 1.8760 Kelem/s]
                 change:
                        time:   [−3.7835% +2.3541% +8.4706%] (p = 0.43 > 0.05)
                        thrpt:  [−7.8091% −2.3000% +3.9323%]
                        No change in performance detected.
metadata_store_query/query_accepting_work
                        time:   [539.87 µs 565.27 µs 590.55 µs]
                        thrpt:  [1.6933 Kelem/s 1.7691 Kelem/s 1.8523 Kelem/s]
                 change:
                        time:   [−7.9885% −2.8120% +2.6228%] (p = 0.30 > 0.05)
                        thrpt:  [−2.5557% +2.8933% +8.6821%]
                        No change in performance detected.
metadata_store_query/query_with_limit
                        time:   [493.47 µs 514.42 µs 535.10 µs]
                        thrpt:  [1.8688 Kelem/s 1.9439 Kelem/s 2.0265 Kelem/s]
                 change:
                        time:   [−5.0037% +1.7426% +8.5846%] (p = 0.61 > 0.05)
                        thrpt:  [−7.9059% −1.7127% +5.2672%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_query/query_complex
                        time:   [277.83 µs 278.41 µs 279.01 µs]
                        thrpt:  [3.5841 Kelem/s 3.5918 Kelem/s 3.5993 Kelem/s]
                 change:
                        time:   [−1.9847% −1.4826% −0.9699%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9794% +1.5049% +2.0249%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

metadata_store_spatial/find_nearby_100km
                        time:   [326.66 µs 327.33 µs 328.04 µs]
                        thrpt:  [3.0484 Kelem/s 3.0550 Kelem/s 3.0613 Kelem/s]
                 change:
                        time:   [−1.6612% −1.1544% −0.6881%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6929% +1.1679% +1.6893%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
metadata_store_spatial/find_nearby_1000km
                        time:   [402.01 µs 402.77 µs 403.60 µs]
                        thrpt:  [2.4777 Kelem/s 2.4828 Kelem/s 2.4875 Kelem/s]
                 change:
                        time:   [−1.6729% −0.8302% −0.0472%] (p = 0.05 < 0.05)
                        thrpt:  [+0.0472% +0.8371% +1.7014%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  8 (8.00%) high mild
  7 (7.00%) high severe
metadata_store_spatial/find_nearby_5000km
                        time:   [488.83 µs 512.05 µs 536.64 µs]
                        thrpt:  [1.8634 Kelem/s 1.9530 Kelem/s 2.0457 Kelem/s]
                 change:
                        time:   [−6.0437% −0.8960% +4.5662%] (p = 0.74 > 0.05)
                        thrpt:  [−4.3668% +0.9041% +6.4324%]
                        No change in performance detected.
metadata_store_spatial/find_best_for_routing
                        time:   [257.56 µs 273.23 µs 290.41 µs]
                        thrpt:  [3.4434 Kelem/s 3.6599 Kelem/s 3.8826 Kelem/s]
                 change:
                        time:   [+19.703% +25.723% +32.623%] (p = 0.00 < 0.05)
                        thrpt:  [−24.598% −20.460% −16.460%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
metadata_store_spatial/find_relays
                        time:   [513.02 µs 532.85 µs 553.39 µs]
                        thrpt:  [1.8070 Kelem/s 1.8767 Kelem/s 1.9492 Kelem/s]
                 change:
                        time:   [−9.3858% −2.1879% +5.1330%] (p = 0.58 > 0.05)
                        thrpt:  [−4.8824% +2.2368% +10.358%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

metadata_store_scaling/query_status/1000
                        time:   [18.701 µs 18.715 µs 18.729 µs]
                        thrpt:  [53.394 Kelem/s 53.433 Kelem/s 53.473 Kelem/s]
                 change:
                        time:   [−1.9190% −1.7526% −1.5693%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5943% +1.7838% +1.9565%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
metadata_store_scaling/query_complex/1000
                        time:   [20.882 µs 20.908 µs 20.933 µs]
                        thrpt:  [47.772 Kelem/s 47.830 Kelem/s 47.888 Kelem/s]
                 change:
                        time:   [−3.1921% −2.9035% −2.5845%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6531% +2.9903% +3.2973%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
metadata_store_scaling/find_nearby/1000
                        time:   [53.635 µs 53.690 µs 53.745 µs]
                        thrpt:  [18.606 Kelem/s 18.625 Kelem/s 18.645 Kelem/s]
                 change:
                        time:   [−0.1363% +0.1553% +0.4376%] (p = 0.29 > 0.05)
                        thrpt:  [−0.4357% −0.1550% +0.1365%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
metadata_store_scaling/query_status/5000
                        time:   [99.330 µs 99.460 µs 99.592 µs]
                        thrpt:  [10.041 Kelem/s 10.054 Kelem/s 10.067 Kelem/s]
                 change:
                        time:   [+1.3629% +1.6321% +1.8728%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8384% −1.6059% −1.3445%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
metadata_store_scaling/query_complex/5000
                        time:   [119.18 µs 119.32 µs 119.47 µs]
                        thrpt:  [8.3706 Kelem/s 8.3809 Kelem/s 8.3908 Kelem/s]
                 change:
                        time:   [−0.0541% +0.1838% +0.4234%] (p = 0.12 > 0.05)
                        thrpt:  [−0.4216% −0.1834% +0.0541%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
metadata_store_scaling/find_nearby/5000
                        time:   [362.25 µs 380.89 µs 398.87 µs]
                        thrpt:  [2.5071 Kelem/s 2.6254 Kelem/s 2.7605 Kelem/s]
                 change:
                        time:   [−10.312% −5.1967% +0.3680%] (p = 0.05 > 0.05)
                        thrpt:  [−0.3667% +5.4815% +11.497%]
                        No change in performance detected.
metadata_store_scaling/query_status/10000
                        time:   [301.97 µs 323.62 µs 346.23 µs]
                        thrpt:  [2.8882 Kelem/s 3.0900 Kelem/s 3.3115 Kelem/s]
                 change:
                        time:   [−1.7123% +9.8765% +22.062%] (p = 0.09 > 0.05)
                        thrpt:  [−18.075% −8.9888% +1.7421%]
                        No change in performance detected.
metadata_store_scaling/query_complex/10000
                        time:   [367.11 µs 388.82 µs 410.75 µs]
                        thrpt:  [2.4346 Kelem/s 2.5719 Kelem/s 2.7239 Kelem/s]
                 change:
                        time:   [+6.2966% +13.478% +20.556%] (p = 0.00 < 0.05)
                        thrpt:  [−17.051% −11.877% −5.9236%]
                        Performance has regressed.
metadata_store_scaling/find_nearby/10000
                        time:   [571.44 µs 584.05 µs 597.52 µs]
                        thrpt:  [1.6736 Kelem/s 1.7122 Kelem/s 1.7500 Kelem/s]
                 change:
                        time:   [−2.6172% +0.3286% +3.3032%] (p = 0.83 > 0.05)
                        thrpt:  [−3.1976% −0.3276% +2.6875%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
metadata_store_scaling/query_status/50000
                        time:   [2.3102 ms 2.3467 ms 2.3845 ms]
                        thrpt:  [419.38  elem/s 426.12  elem/s 432.86  elem/s]
                 change:
                        time:   [−1.6599% +0.1299% +1.9362%] (p = 0.88 > 0.05)
                        thrpt:  [−1.8994% −0.1297% +1.6880%]
                        No change in performance detected.
metadata_store_scaling/query_complex/50000
                        time:   [2.6460 ms 2.6788 ms 2.7123 ms]
                        thrpt:  [368.69  elem/s 373.30  elem/s 377.92  elem/s]
                 change:
                        time:   [−2.1929% −0.4789% +1.4265%] (p = 0.61 > 0.05)
                        thrpt:  [−1.4065% +0.4812% +2.2421%]
                        No change in performance detected.
metadata_store_scaling/find_nearby/50000
                        time:   [3.3765 ms 3.4158 ms 3.4563 ms]
                        thrpt:  [289.33  elem/s 292.76  elem/s 296.16  elem/s]
                 change:
                        time:   [+1.3964% +2.9130% +4.4385%] (p = 0.00 < 0.05)
                        thrpt:  [−4.2498% −2.8305% −1.3771%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

metadata_store_concurrent/concurrent_upsert/4
                        time:   [1.6816 ms 1.6960 ms 1.7087 ms]
                        thrpt:  [1.1705 Melem/s 1.1792 Melem/s 1.1894 Melem/s]
                 change:
                        time:   [−2.3133% −0.2794% +2.6386%] (p = 0.85 > 0.05)
                        thrpt:  [−2.5708% +0.2802% +2.3680%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.7s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/4
                        time:   [410.60 ms 420.49 ms 433.34 ms]
                        thrpt:  [4.6153 Kelem/s 4.7563 Kelem/s 4.8709 Kelem/s]
                 change:
                        time:   [+30.915% +36.248% +42.056%] (p = 0.00 < 0.05)
                        thrpt:  [−29.605% −26.604% −23.615%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.0s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/4
                        time:   [398.78 ms 412.56 ms 430.26 ms]
                        thrpt:  [4.6484 Kelem/s 4.8478 Kelem/s 5.0153 Kelem/s]
                 change:
                        time:   [−47.645% −41.749% −32.932%] (p = 0.00 < 0.05)
                        thrpt:  [+49.102% +71.670% +91.002%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
metadata_store_concurrent/concurrent_upsert/8
                        time:   [2.9571 ms 2.9656 ms 2.9753 ms]
                        thrpt:  [1.3444 Melem/s 1.3488 Melem/s 1.3527 Melem/s]
                 change:
                        time:   [−9.1093% −6.3861% −3.7924%] (p = 0.00 < 0.05)
                        thrpt:  [+3.9419% +6.8217% +10.022%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.2s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/8
                        time:   [811.07 ms 814.78 ms 818.61 ms]
                        thrpt:  [4.8863 Kelem/s 4.9093 Kelem/s 4.9318 Kelem/s]
                 change:
                        time:   [+1.2494% +2.3388% +3.3820%] (p = 0.00 < 0.05)
                        thrpt:  [−3.2714% −2.2853% −1.2340%]
                        Performance has regressed.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 19.3s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/8
                        time:   [939.53 ms 955.42 ms 971.33 ms]
                        thrpt:  [4.1181 Kelem/s 4.1866 Kelem/s 4.2574 Kelem/s]
                 change:
                        time:   [−7.4652% −2.9648% +0.7836%] (p = 0.21 > 0.05)
                        thrpt:  [−0.7775% +3.0554% +8.0674%]
                        No change in performance detected.
metadata_store_concurrent/concurrent_upsert/16
                        time:   [7.2270 ms 7.3021 ms 7.4542 ms]
                        thrpt:  [1.0732 Melem/s 1.0956 Melem/s 1.1070 Melem/s]
                 change:
                        time:   [−8.6904% −7.5499% −6.1838%] (p = 0.00 < 0.05)
                        thrpt:  [+6.5914% +8.1664% +9.5175%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 29.8s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/16
                        time:   [1.4763 s 1.4812 s 1.4862 s]
                        thrpt:  [5.3828 Kelem/s 5.4011 Kelem/s 5.4189 Kelem/s]
                 change:
                        time:   [−4.6403% −2.9947% −1.4833%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5056% +3.0872% +4.8661%]
                        Performance has improved.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 34.0s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/16
                        time:   [1.7220 s 1.7257 s 1.7297 s]
                        thrpt:  [4.6252 Kelem/s 4.6358 Kelem/s 4.6458 Kelem/s]
                 change:
                        time:   [−1.7861% −1.4825% −1.1496%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1629% +1.5048% +1.8185%]
                        Performance has improved.

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimated 5.0008 s (10M iteratimetadata_store_versioning/update_versioned_success
                        time:   [180.21 ns 180.52 ns 180.81 ns]
                        thrpt:  [5.5306 Melem/s 5.5397 Melem/s 5.5491 Melem/s]
                 change:
                        time:   [+0.9212% +1.5902% +2.1607%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1150% −1.5653% −0.9128%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimated 5.0005 s (28M iteratmetadata_store_versioning/update_versioned_conflict
                        time:   [180.17 ns 180.48 ns 180.85 ns]
                        thrpt:  [5.5293 Melem/s 5.5407 Melem/s 5.5502 Melem/s]
                 change:
                        time:   [+1.0828% +1.7741% +2.3537%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2996% −1.7432% −1.0712%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

schema_validation/validate_string
                        time:   [3.4838 ns 3.4871 ns 3.4911 ns]
                        thrpt:  [286.44 Melem/s 286.77 Melem/s 287.04 Melem/s]
                 change:
                        time:   [+1.8274% +1.9142% +2.0113%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9717% −1.8783% −1.7946%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
schema_validation/validate_integer
                        time:   [3.4750 ns 3.4765 ns 3.4778 ns]
                        thrpt:  [287.54 Melem/s 287.64 Melem/s 287.77 Melem/s]
                 change:
                        time:   [−21.504% −9.1650% −1.1208%] (p = 0.20 > 0.05)
                        thrpt:  [+1.1335% +10.090% +27.395%]
                        No change in performance detected.
Found 18 outliers among 100 measurements (18.00%)
  8 (8.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
schema_validation/validate_object
                        time:   [74.178 ns 74.256 ns 74.331 ns]
                        thrpt:  [13.453 Melem/s 13.467 Melem/s 13.481 Melem/s]
                 change:
                        time:   [−6.2865% −6.1066% −5.9147%] (p = 0.00 < 0.05)
                        thrpt:  [+6.2865% +6.5038% +6.7082%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
schema_validation/validate_array_10
                        time:   [36.059 ns 36.123 ns 36.196 ns]
                        thrpt:  [27.627 Melem/s 27.683 Melem/s 27.732 Melem/s]
                 change:
                        time:   [−0.0574% +0.2130% +0.4723%] (p = 0.11 > 0.05)
                        thrpt:  [−0.4701% −0.2126% +0.0574%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
schema_validation/validate_complex
                        time:   [204.94 ns 206.96 ns 210.05 ns]
                        thrpt:  [4.7609 Melem/s 4.8318 Melem/s 4.8796 Melem/s]
                 change:
                        time:   [−8.4688% −6.4090% −4.3034%] (p = 0.00 < 0.05)
                        thrpt:  [+4.4969% +6.8478% +9.2524%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

endpoint_matching/match_success
                        time:   [170.32 ns 170.65 ns 170.99 ns]
                        thrpt:  [5.8482 Melem/s 5.8601 Melem/s 5.8711 Melem/s]
                 change:
                        time:   [+0.5799% +0.7338% +0.9195%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9112% −0.7285% −0.5765%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
endpoint_matching/match_failure
                        time:   [173.38 ns 173.72 ns 174.09 ns]
                        thrpt:  [5.7441 Melem/s 5.7565 Melem/s 5.7677 Melem/s]
                 change:
                        time:   [+1.4778% +1.6625% +1.8420%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8087% −1.6353% −1.4563%]
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe
endpoint_matching/match_multi_param
                        time:   [374.08 ns 374.77 ns 375.95 ns]
                        thrpt:  [2.6599 Melem/s 2.6683 Melem/s 2.6732 Melem/s]
                 change:
                        time:   [+0.0842% +0.3862% +0.6558%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6515% −0.3847% −0.0842%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe

api_version/is_compatible_with
                        time:   [311.11 ps 311.29 ps 311.47 ps]
                        thrpt:  [3.2105 Gelem/s 3.2125 Gelem/s 3.2143 Gelem/s]
                 change:
                        time:   [−15.135% −6.0370% −0.5624%] (p = 0.20 > 0.05)
                        thrpt:  [+0.5656% +6.4249% +17.834%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
api_version/parse       time:   [35.472 ns 35.493 ns 35.514 ns]
                        thrpt:  [28.158 Melem/s 28.175 Melem/s 28.191 Melem/s]
                 change:
                        time:   [−2.5871% −2.4949% −2.3873%] (p = 0.00 < 0.05)
                        thrpt:  [+2.4457% +2.5587% +2.6558%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
api_version/to_string   time:   [43.379 ns 43.406 ns 43.436 ns]
                        thrpt:  [23.022 Melem/s 23.038 Melem/s 23.053 Melem/s]
                 change:
                        time:   [−3.9975% −3.8130% −3.4720%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5969% +3.9642% +4.1639%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

api_schema/create       time:   [1.2290 µs 1.2306 µs 1.2322 µs]
                        thrpt:  [811.55 Kelem/s 812.61 Kelem/s 813.69 Kelem/s]
                 change:
                        time:   [+0.4536% +0.7372% +0.9994%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9895% −0.7318% −0.4515%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
api_schema/serialize    time:   [1.9252 µs 1.9271 µs 1.9290 µs]
                        thrpt:  [518.40 Kelem/s 518.91 Kelem/s 519.41 Kelem/s]
                 change:
                        time:   [+1.5291% +1.8454% +2.1356%] (p = 0.00 < 0.05)
                        thrpt:  [−2.0909% −1.8120% −1.5060%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
api_schema/deserialize  time:   [5.1160 µs 5.1236 µs 5.1338 µs]
                        thrpt:  [194.79 Kelem/s 195.18 Kelem/s 195.47 Kelem/s]
                 change:
                        time:   [−1.2970% −0.8159% −0.3577%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3590% +0.8226% +1.3141%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
api_schema/find_endpoint
                        time:   [200.03 ns 200.21 ns 200.40 ns]
                        thrpt:  [4.9900 Melem/s 4.9947 Melem/s 4.9992 Melem/s]
                 change:
                        time:   [+0.7443% +0.9234% +1.1010%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0890% −0.9150% −0.7388%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
api_schema/endpoints_by_tag
                        time:   [57.419 ns 57.492 ns 57.558 ns]
                        thrpt:  [17.374 Melem/s 17.394 Melem/s 17.416 Melem/s]
                 change:
                        time:   [+0.2804% +0.4663% +0.6635%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6591% −0.4641% −0.2796%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe

request_validation/validate_full_request
                        time:   [71.020 ns 71.100 ns 71.177 ns]
                        thrpt:  [14.050 Melem/s 14.065 Melem/s 14.080 Melem/s]
                 change:
                        time:   [−5.3916% −5.0984% −4.8061%] (p = 0.00 < 0.05)
                        thrpt:  [+5.0487% +5.3723% +5.6989%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
request_validation/validate_path_only
                        time:   [21.356 ns 21.463 ns 21.567 ns]
                        thrpt:  [46.368 Melem/s 46.592 Melem/s 46.825 Melem/s]
                 change:
                        time:   [−4.2366% −3.8222% −3.4109%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5313% +3.9741% +4.4241%]
                        Performance has improved.

api_registry_basic/create
                        time:   [417.83 ns 418.58 ns 419.33 ns]
                        thrpt:  [2.3847 Melem/s 2.3890 Melem/s 2.3933 Melem/s]
                 change:
                        time:   [−2.6140% −2.2795% −1.9285%] (p = 0.00 < 0.05)
                        thrpt:  [+1.9664% +2.3327% +2.6842%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
api_registry_basic/register_new
                        time:   [3.8524 µs 3.9043 µs 3.9532 µs]
                        thrpt:  [252.96 Kelem/s 256.13 Kelem/s 259.58 Kelem/s]
                 change:
                        time:   [−1.7525% +0.5259% +2.5192%] (p = 0.63 > 0.05)
                        thrpt:  [−2.4573% −0.5232% +1.7837%]
                        No change in performance detected.
api_registry_basic/get  time:   [25.582 ns 26.368 ns 27.164 ns]
                        thrpt:  [36.814 Melem/s 37.925 Melem/s 39.090 Melem/s]
                 change:
                        time:   [−9.2092% −5.6130% −1.9047%] (p = 0.00 < 0.05)
                        thrpt:  [+1.9417% +5.9468% +10.143%]
                        Performance has improved.
api_registry_basic/len  time:   [199.88 ns 200.03 ns 200.19 ns]
                        thrpt:  [4.9953 Melem/s 4.9993 Melem/s 5.0029 Melem/s]
                 change:
                        time:   [+0.2863% +0.4583% +0.7621%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7563% −0.4562% −0.2855%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 24.1s, or reduce sample count to 20.
api_registry_basic/stats
                        time:   [237.79 ms 240.04 ms 242.32 ms]
                        thrpt:  [4.1267  elem/s 4.1661  elem/s 4.2054  elem/s]
                 change:
                        time:   [−7.3918% −3.4843% −0.3434%] (p = 0.05 > 0.05)
                        thrpt:  [+0.3446% +3.6101% +7.9817%]
                        No change in performance detected.

api_registry_query/query_by_name
                        time:   [106.25 µs 106.63 µs 107.01 µs]
                        thrpt:  [9.3452 Kelem/s 9.3783 Kelem/s 9.4121 Kelem/s]
                 change:
                        time:   [−4.1584% −3.5117% −2.8137%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8952% +3.6396% +4.3389%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high severe
api_registry_query/query_by_tag
                        time:   [670.50 µs 696.72 µs 723.91 µs]
                        thrpt:  [1.3814 Kelem/s 1.4353 Kelem/s 1.4914 Kelem/s]
                 change:
                        time:   [−1.6261% +4.0680% +10.390%] (p = 0.18 > 0.05)
                        thrpt:  [−9.4121% −3.9090% +1.6530%]
                        No change in performance detected.
api_registry_query/query_with_version
                        time:   [59.074 µs 59.132 µs 59.192 µs]
                        thrpt:  [16.894 Kelem/s 16.911 Kelem/s 16.928 Kelem/s]
                 change:
                        time:   [+0.7112% +0.8266% +0.9408%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9320% −0.8198% −0.7062%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
api_registry_query/find_by_endpoint
                        time:   [2.9806 ms 3.0332 ms 3.0875 ms]
                        thrpt:  [323.89  elem/s 329.69  elem/s 335.50  elem/s]
                 change:
                        time:   [−8.2853% −6.0316% −3.5761%] (p = 0.00 < 0.05)
                        thrpt:  [+3.7088% +6.4188% +9.0338%]
                        Performance has improved.
api_registry_query/find_compatible
                        time:   [64.760 µs 64.801 µs 64.846 µs]
                        thrpt:  [15.421 Kelem/s 15.432 Kelem/s 15.442 Kelem/s]
                 change:
                        time:   [+0.2443% +0.5623% +0.8523%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8451% −0.5592% −0.2438%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

api_registry_scaling/query_by_name/1000
                        time:   [7.6708 µs 7.6826 µs 7.6946 µs]
                        thrpt:  [129.96 Kelem/s 130.17 Kelem/s 130.36 Kelem/s]
                 change:
                        time:   [+0.0347% +0.2155% +0.3866%] (p = 0.02 < 0.05)
                        thrpt:  [−0.3851% −0.2151% −0.0347%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
api_registry_scaling/query_by_tag/1000
                        time:   [49.972 µs 50.025 µs 50.078 µs]
                        thrpt:  [19.969 Kelem/s 19.990 Kelem/s 20.011 Kelem/s]
                 change:
                        time:   [+3.2915% +3.8563% +4.4469%] (p = 0.00 < 0.05)
                        thrpt:  [−4.2576% −3.7131% −3.1866%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
api_registry_scaling/query_by_name/5000
                        time:   [48.554 µs 48.659 µs 48.759 µs]
                        thrpt:  [20.509 Kelem/s 20.551 Kelem/s 20.596 Kelem/s]
                 change:
                        time:   [−0.1926% +0.1606% +0.5062%] (p = 0.39 > 0.05)
                        thrpt:  [−0.5037% −0.1603% +0.1930%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
api_registry_scaling/query_by_tag/5000
                        time:   [385.10 µs 401.57 µs 417.69 µs]
                        thrpt:  [2.3941 Kelem/s 2.4903 Kelem/s 2.5967 Kelem/s]
                 change:
                        time:   [−14.454% −9.6749% −4.5259%] (p = 0.00 < 0.05)
                        thrpt:  [+4.7404% +10.711% +16.896%]
                        Performance has improved.
api_registry_scaling/query_by_name/10000
                        time:   [104.78 µs 105.25 µs 105.71 µs]
                        thrpt:  [9.4602 Kelem/s 9.5012 Kelem/s 9.5439 Kelem/s]
                 change:
                        time:   [+10.873% +11.482% +12.095%] (p = 0.00 < 0.05)
                        thrpt:  [−10.790% −10.299% −9.8067%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
api_registry_scaling/query_by_tag/10000
                        time:   [697.50 µs 735.40 µs 775.95 µs]
                        thrpt:  [1.2887 Kelem/s 1.3598 Kelem/s 1.4337 Kelem/s]
                 change:
                        time:   [+57.661% +66.790% +76.439%] (p = 0.00 < 0.05)
                        thrpt:  [−43.323% −40.044% −36.573%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.5s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/4
                        time:   [826.30 ms 832.51 ms 838.51 ms]
                        thrpt:  [2.3852 Kelem/s 2.4024 Kelem/s 2.4204 Kelem/s]
                 change:
                        time:   [−2.7280% +3.6709% +9.7562%] (p = 0.26 > 0.05)
                        thrpt:  [−8.8890% −3.5409% +2.8045%]
                        No change in performance detected.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 12.4s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/4
                        time:   [611.92 ms 618.97 ms 626.56 ms]
                        thrpt:  [3.1920 Kelem/s 3.2312 Kelem/s 3.2684 Kelem/s]
                 change:
                        time:   [+19.521% +23.058% +27.240%] (p = 0.00 < 0.05)
                        thrpt:  [−21.409% −18.737% −16.333%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 22.5s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/8
                        time:   [1.1278 s 1.1324 s 1.1372 s]
                        thrpt:  [3.5174 Kelem/s 3.5323 Kelem/s 3.5468 Kelem/s]
                 change:
                        time:   [+0.5191% +1.1484% +1.7729%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7420% −1.1353% −0.5165%]
                        Change within noise threshold.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 19.0s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/8
                        time:   [938.74 ms 944.36 ms 949.28 ms]
                        thrpt:  [4.2137 Kelem/s 4.2357 Kelem/s 4.2610 Kelem/s]
                 change:
                        time:   [−4.0923% −2.7603% −1.4597%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4813% +2.8386% +4.2669%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 37.9s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/16
                        time:   [1.8773 s 1.8814 s 1.8856 s]
                        thrpt:  [4.2427 Kelem/s 4.2521 Kelem/s 4.2615 Kelem/s]
                 change:
                        time:   [+0.4157% +0.8515% +1.2600%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2443% −0.8444% −0.4140%]
                        Change within noise threshold.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 33.3s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/16
                        time:   [1.6743 s 1.6825 s 1.6916 s]
                        thrpt:  [4.7293 Kelem/s 4.7547 Kelem/s 4.7780 Kelem/s]
                 change:
                        time:   [−4.7084% −4.1348% −3.6021%] (p = 0.00 < 0.05)
                        thrpt:  [+3.7368% +4.3131% +4.9410%]
                        Performance has improved.

compare_op/eq           time:   [1.9805 ns 1.9813 ns 1.9820 ns]
                        thrpt:  [504.54 Melem/s 504.73 Melem/s 504.91 Melem/s]
                 change:
                        time:   [+2.1241% +2.1957% +2.2659%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2157% −2.1485% −2.0799%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
compare_op/gt           time:   [1.2454 ns 1.2463 ns 1.2474 ns]
                        thrpt:  [801.67 Melem/s 802.35 Melem/s 802.97 Melem/s]
                 change:
                        time:   [+0.3600% +0.4405% +0.5263%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5235% −0.4386% −0.3587%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
compare_op/contains_string
                        time:   [25.532 ns 25.548 ns 25.564 ns]
                        thrpt:  [39.117 Melem/s 39.143 Melem/s 39.167 Melem/s]
                 change:
                        time:   [+1.6025% +1.7406% +1.9201%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8840% −1.7108% −1.5772%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
compare_op/in_array     time:   [6.8577 ns 6.8671 ns 6.8773 ns]
                        thrpt:  [145.41 Melem/s 145.62 Melem/s 145.82 Melem/s]
                 change:
                        time:   [+0.4693% +0.6245% +0.8085%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8020% −0.6206% −0.4671%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

condition/simple        time:   [50.213 ns 50.328 ns 50.480 ns]
                        thrpt:  [19.810 Melem/s 19.870 Melem/s 19.915 Melem/s]
                 change:
                        time:   [−0.0479% +0.1833% +0.4311%] (p = 0.13 > 0.05)
                        thrpt:  [−0.4293% −0.1830% +0.0480%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
condition/nested_field  time:   [565.49 ns 566.42 ns 567.46 ns]
                        thrpt:  [1.7622 Melem/s 1.7655 Melem/s 1.7684 Melem/s]
                 change:
                        time:   [+0.9846% +1.3264% +1.6855%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6576% −1.3090% −0.9750%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low mild
  5 (5.00%) high mild
  2 (2.00%) high severe
condition/string_eq     time:   [70.603 ns 70.764 ns 71.007 ns]
                        thrpt:  [14.083 Melem/s 14.132 Melem/s 14.164 Melem/s]
                 change:
                        time:   [+0.2250% +0.4075% +0.6043%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6007% −0.4058% −0.2245%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

condition_expr/single   time:   [50.606 ns 50.680 ns 50.759 ns]
                        thrpt:  [19.701 Melem/s 19.732 Melem/s 19.760 Melem/s]
                 change:
                        time:   [−0.8903% −0.6853% −0.4885%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4909% +0.6900% +0.8983%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
condition_expr/and_2    time:   [102.41 ns 102.55 ns 102.71 ns]
                        thrpt:  [9.7363 Melem/s 9.7512 Melem/s 9.7649 Melem/s]
                 change:
                        time:   [−1.1491% −0.9182% −0.6916%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6964% +0.9267% +1.1624%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  10 (10.00%) high mild
  3 (3.00%) high severe
condition_expr/and_5    time:   [304.35 ns 304.75 ns 305.14 ns]
                        thrpt:  [3.2771 Melem/s 3.2814 Melem/s 3.2857 Melem/s]
                 change:
                        time:   [−0.5754% −0.3557% −0.1419%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1421% +0.3570% +0.5787%]
                        Change within noise threshold.
condition_expr/or_3     time:   [171.08 ns 171.40 ns 171.82 ns]
                        thrpt:  [5.8201 Melem/s 5.8343 Melem/s 5.8453 Melem/s]
                 change:
                        time:   [−2.5830% −2.3421% −2.0888%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1334% +2.3983% +2.6515%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
condition_expr/nested   time:   [123.49 ns 123.71 ns 123.95 ns]
                        thrpt:  [8.0678 Melem/s 8.0831 Melem/s 8.0981 Melem/s]
                 change:
                        time:   [+0.7912% +1.1161% +1.4613%] (p = 0.00 < 0.05)
                        thrpt:  [−1.4402% −1.1038% −0.7850%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

rule/create             time:   [284.62 ns 285.43 ns 286.27 ns]
                        thrpt:  [3.4932 Melem/s 3.5034 Melem/s 3.5135 Melem/s]
                 change:
                        time:   [+1.2491% +1.6077% +1.9675%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9295% −1.5822% −1.2337%]
                        Performance has regressed.
rule/matches            time:   [102.80 ns 102.92 ns 103.06 ns]
                        thrpt:  [9.7032 Melem/s 9.7159 Melem/s 9.7276 Melem/s]
                 change:
                        time:   [−0.8930% −0.6281% −0.3835%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3850% +0.6321% +0.9011%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe

rule_context/create     time:   [839.02 ns 839.89 ns 840.79 ns]
                        thrpt:  [1.1894 Melem/s 1.1906 Melem/s 1.1919 Melem/s]
                 change:
                        time:   [−0.2783% −0.0326% +0.2247%] (p = 0.80 > 0.05)
                        thrpt:  [−0.2242% +0.0326% +0.2791%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
rule_context/get_simple time:   [48.274 ns 48.333 ns 48.387 ns]
                        thrpt:  [20.667 Melem/s 20.690 Melem/s 20.715 Melem/s]
                 change:
                        time:   [−2.1003% −1.8409% −1.5956%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6214% +1.8755% +2.1453%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
rule_context/get_nested time:   [564.22 ns 564.86 ns 565.54 ns]
                        thrpt:  [1.7682 Melem/s 1.7703 Melem/s 1.7724 Melem/s]
                 change:
                        time:   [+0.4285% +0.6204% +0.8196%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8129% −0.6166% −0.4267%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
rule_context/get_deep_nested
                        time:   [570.11 ns 571.03 ns 571.93 ns]
                        thrpt:  [1.7485 Melem/s 1.7512 Melem/s 1.7540 Melem/s]
                 change:
                        time:   [+0.6200% +0.8435% +1.0862%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0745% −0.8364% −0.6162%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

rule_engine_basic/create
                        time:   [8.2097 ns 8.2187 ns 8.2280 ns]
                        thrpt:  [121.54 Melem/s 121.67 Melem/s 121.81 Melem/s]
                 change:
                        time:   [−0.2921% −0.0228% +0.2325%] (p = 0.86 > 0.05)
                        thrpt:  [−0.2320% +0.0228% +0.2930%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
rule_engine_basic/add_rule
                        time:   [2.8037 µs 2.9628 µs 3.0954 µs]
                        thrpt:  [323.06 Kelem/s 337.52 Kelem/s 356.67 Kelem/s]
                 change:
                        time:   [−10.375% +0.7369% +14.092%] (p = 0.91 > 0.05)
                        thrpt:  [−12.351% −0.7315% +11.576%]
                        No change in performance detected.
rule_engine_basic/get_rule
                        time:   [18.272 ns 19.005 ns 19.736 ns]
                        thrpt:  [50.668 Melem/s 52.618 Melem/s 54.730 Melem/s]
                 change:
                        time:   [−24.369% −20.741% −16.818%] (p = 0.00 < 0.05)
                        thrpt:  [+20.218% +26.169% +32.221%]
                        Performance has improved.
rule_engine_basic/rules_by_tag
                        time:   [1.0927 µs 1.0941 µs 1.0957 µs]
                        thrpt:  [912.62 Kelem/s 913.98 Kelem/s 915.17 Kelem/s]
                 change:
                        time:   [−3.1681% −3.0450% −2.9324%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0209% +3.1406% +3.2718%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
rule_engine_basic/stats time:   [6.9115 µs 6.9191 µs 6.9288 µs]
                        thrpt:  [144.32 Kelem/s 144.53 Kelem/s 144.69 Kelem/s]
                 change:
                        time:   [−0.1905% −0.0809% +0.0239%] (p = 0.16 > 0.05)
                        thrpt:  [−0.0239% +0.0809% +0.1909%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

rule_engine_evaluate/evaluate_10_rules
                        time:   [2.1538 µs 2.1581 µs 2.1642 µs]
                        thrpt:  [462.07 Kelem/s 463.37 Kelem/s 464.29 Kelem/s]
                 change:
                        time:   [−1.0329% −0.8917% −0.7344%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7398% +0.8997% +1.0437%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
rule_engine_evaluate/evaluate_first_10_rules
                        time:   [234.35 ns 234.84 ns 235.43 ns]
                        thrpt:  [4.2475 Melem/s 4.2582 Melem/s 4.2672 Melem/s]
                 change:
                        time:   [+0.2926% +0.5046% +0.7356%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7303% −0.5021% −0.2917%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
rule_engine_evaluate/evaluate_100_rules
                        time:   [21.948 µs 21.974 µs 21.998 µs]
                        thrpt:  [45.459 Kelem/s 45.507 Kelem/s 45.561 Kelem/s]
                 change:
                        time:   [+0.5685% +0.7102% +0.8448%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8377% −0.7052% −0.5653%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_evaluate/evaluate_first_100_rules
                        time:   [234.50 ns 234.77 ns 235.06 ns]
                        thrpt:  [4.2543 Melem/s 4.2594 Melem/s 4.2644 Melem/s]
                 change:
                        time:   [+0.2710% +0.5273% +0.7838%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7777% −0.5245% −0.2703%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated 5.1001 s (227k iteratiorule_engine_evaluate/evaluate_matching_100_rules
                        time:   [22.461 µs 22.488 µs 22.516 µs]
                        thrpt:  [44.413 Kelem/s 44.468 Kelem/s 44.522 Kelem/s]
                 change:
                        time:   [+0.9900% +1.1349% +1.2828%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2666% −1.1221% −0.9803%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
rule_engine_evaluate/evaluate_1000_rules
                        time:   [240.28 µs 252.76 µs 267.11 µs]
                        thrpt:  [3.7437 Kelem/s 3.9564 Kelem/s 4.1619 Kelem/s]
                 change:
                        time:   [+9.8774% +14.722% +19.866%] (p = 0.00 < 0.05)
                        thrpt:  [−16.574% −12.833% −8.9894%]
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  9 (9.00%) high mild
  5 (5.00%) high severe
rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [235.05 ns 235.46 ns 235.94 ns]
                        thrpt:  [4.2384 Melem/s 4.2469 Melem/s 4.2544 Melem/s]
                 change:
                        time:   [+0.6558% +0.9359% +1.2217%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2069% −0.9273% −0.6515%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

rule_engine_scaling/evaluate/10
                        time:   [2.1716 µs 2.1737 µs 2.1757 µs]
                        thrpt:  [459.62 Kelem/s 460.05 Kelem/s 460.50 Kelem/s]
                 change:
                        time:   [+0.0030% +0.1929% +0.3764%] (p = 0.04 < 0.05)
                        thrpt:  [−0.3750% −0.1925% −0.0030%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low mild
  7 (7.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate_first/10
                        time:   [234.90 ns 235.14 ns 235.37 ns]
                        thrpt:  [4.2485 Melem/s 4.2527 Melem/s 4.2571 Melem/s]
                 change:
                        time:   [−0.0383% +0.2182% +0.4914%] (p = 0.10 > 0.05)
                        thrpt:  [−0.4890% −0.2177% +0.0383%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate/50
                        time:   [10.919 µs 10.930 µs 10.941 µs]
                        thrpt:  [91.395 Kelem/s 91.495 Kelem/s 91.587 Kelem/s]
                 change:
                        time:   [−0.6831% −0.4603% −0.2370%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2376% +0.4624% +0.6878%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate_first/50
                        time:   [235.13 ns 235.38 ns 235.61 ns]
                        thrpt:  [4.2444 Melem/s 4.2485 Melem/s 4.2529 Melem/s]
                 change:
                        time:   [+0.1441% +0.4242% +0.7235%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7183% −0.4224% −0.1439%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate/100
                        time:   [21.951 µs 21.983 µs 22.014 µs]
                        thrpt:  [45.425 Kelem/s 45.489 Kelem/s 45.555 Kelem/s]
                 change:
                        time:   [−0.2143% −0.0341% +0.1654%] (p = 0.72 > 0.05)
                        thrpt:  [−0.1651% +0.0341% +0.2147%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) low mild
  1 (1.00%) high mild
rule_engine_scaling/evaluate_first/100
                        time:   [235.17 ns 235.64 ns 236.29 ns]
                        thrpt:  [4.2321 Melem/s 4.2437 Melem/s 4.2522 Melem/s]
                 change:
                        time:   [+0.4422% +0.6951% +0.9620%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9528% −0.6903% −0.4402%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high severe
rule_engine_scaling/evaluate/500
                        time:   [127.53 µs 137.78 µs 149.30 µs]
                        thrpt:  [6.6978 Kelem/s 7.2579 Kelem/s 7.8415 Kelem/s]
                 change:
                        time:   [−40.406% −33.350% −24.813%] (p = 0.00 < 0.05)
                        thrpt:  [+33.002% +50.038% +67.802%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
rule_engine_scaling/evaluate_first/500
                        time:   [234.81 ns 235.12 ns 235.42 ns]
                        thrpt:  [4.2477 Melem/s 4.2531 Melem/s 4.2587 Melem/s]
                 change:
                        time:   [−0.5534% −0.1757% +0.2076%] (p = 0.37 > 0.05)
                        thrpt:  [−0.2071% +0.1761% +0.5565%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate/1000
                        time:   [240.41 µs 250.91 µs 262.65 µs]
                        thrpt:  [3.8073 Kelem/s 3.9855 Kelem/s 4.1596 Kelem/s]
                 change:
                        time:   [−18.755% −11.844% −3.9969%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1633% +13.435% +23.085%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
rule_engine_scaling/evaluate_first/1000
                        time:   [234.80 ns 235.10 ns 235.38 ns]
                        thrpt:  [4.2485 Melem/s 4.2536 Melem/s 4.2589 Melem/s]
                 change:
                        time:   [+0.0877% +0.5837% +1.0904%] (p = 0.02 < 0.05)
                        thrpt:  [−1.0786% −0.5803% −0.0876%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

rule_set/create         time:   [2.8239 µs 2.8266 µs 2.8292 µs]
                        thrpt:  [353.46 Kelem/s 353.79 Kelem/s 354.12 Kelem/s]
                 change:
                        time:   [+0.7633% +1.0024% +1.2300%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2150% −0.9925% −0.7575%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
rule_set/load_into_engine
                        time:   [5.5318 µs 5.5485 µs 5.5691 µs]
                        thrpt:  [179.56 Kelem/s 180.23 Kelem/s 180.77 Kelem/s]
                 change:
                        time:   [+0.3600% +0.5419% +0.7658%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7600% −0.5390% −0.3587%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

trace_id/generate       time:   [542.35 ns 543.51 ns 544.72 ns]
                        thrpt:  [1.8358 Melem/s 1.8399 Melem/s 1.8438 Melem/s]
                 change:
                        time:   [+0.2887% +0.5936% +0.9236%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9151% −0.5901% −0.2879%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
trace_id/to_hex         time:   [76.136 ns 76.208 ns 76.281 ns]
                        thrpt:  [13.109 Melem/s 13.122 Melem/s 13.134 Melem/s]
                 change:
                        time:   [−0.0668% +0.1795% +0.4733%] (p = 0.22 > 0.05)
                        thrpt:  [−0.4710% −0.1792% +0.0668%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
trace_id/from_hex       time:   [23.275 ns 23.291 ns 23.310 ns]
                        thrpt:  [42.900 Melem/s 42.934 Melem/s 42.964 Melem/s]
                 change:
                        time:   [+0.4483% +0.5705% +0.6996%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6948% −0.5672% −0.4463%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

context_operations/create
                        time:   [828.27 ns 830.04 ns 831.80 ns]
                        thrpt:  [1.2022 Melem/s 1.2048 Melem/s 1.2073 Melem/s]
                 change:
                        time:   [+0.9078% +1.2741% +1.6453%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6187% −1.2581% −0.8996%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
context_operations/child
                        time:   [287.10 ns 287.82 ns 288.57 ns]
                        thrpt:  [3.4653 Melem/s 3.4744 Melem/s 3.4831 Melem/s]
                 change:
                        time:   [+0.8464% +1.2040% +1.5651%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5410% −1.1896% −0.8393%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild
context_operations/for_remote
                        time:   [286.68 ns 287.37 ns 288.05 ns]
                        thrpt:  [3.4716 Melem/s 3.4798 Melem/s 3.4882 Melem/s]
                 change:
                        time:   [−3.7910% −3.2523% −2.7435%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8208% +3.3616% +3.9404%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
context_operations/to_traceparent
                        time:   [231.80 ns 232.28 ns 232.86 ns]
                        thrpt:  [4.2944 Melem/s 4.3051 Melem/s 4.3141 Melem/s]
                 change:
                        time:   [−2.4481% −2.0842% −1.7049%] (p = 0.00 < 0.05)
                        thrpt:  [+1.7345% +2.1286% +2.5095%]
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  9 (9.00%) high mild
  5 (5.00%) high severe
context_operations/from_traceparent
                        time:   [388.67 ns 389.63 ns 390.48 ns]
                        thrpt:  [2.5609 Melem/s 2.5665 Melem/s 2.5729 Melem/s]
                 change:
                        time:   [+3.8724% +4.1032% +4.3198%] (p = 0.00 < 0.05)
                        thrpt:  [−4.1409% −3.9415% −3.7281%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high severe

baggage/create          time:   [2.0567 ns 2.0585 ns 2.0604 ns]
                        thrpt:  [485.34 Melem/s 485.79 Melem/s 486.21 Melem/s]
                 change:
                        time:   [+0.8403% +0.9683% +1.0877%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0760% −0.9591% −0.8333%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
baggage/get             time:   [18.287 ns 18.877 ns 19.445 ns]
                        thrpt:  [51.426 Melem/s 52.976 Melem/s 54.684 Melem/s]
                 change:
                        time:   [+2.6959% +8.2587% +14.731%] (p = 0.00 < 0.05)
                        thrpt:  [−12.840% −7.6287% −2.6251%]
                        Performance has regressed.
baggage/set             time:   [49.228 ns 49.273 ns 49.322 ns]
                        thrpt:  [20.275 Melem/s 20.295 Melem/s 20.314 Melem/s]
                 change:
                        time:   [+0.4969% +0.6134% +0.7382%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7328% −0.6097% −0.4944%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
baggage/merge           time:   [863.81 ns 864.40 ns 864.98 ns]
                        thrpt:  [1.1561 Melem/s 1.1569 Melem/s 1.1577 Melem/s]
                 change:
                        time:   [−0.0294% +0.1923% +0.3485%] (p = 0.04 < 0.05)
                        thrpt:  [−0.3472% −0.1919% +0.0294%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

span/create             time:   [330.26 ns 331.14 ns 332.12 ns]
                        thrpt:  [3.0109 Melem/s 3.0199 Melem/s 3.0280 Melem/s]
                 change:
                        time:   [+2.5511% +3.0162% +3.4843%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3669% −2.9279% −2.4877%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
span/set_attribute      time:   [47.595 ns 47.699 ns 47.807 ns]
                        thrpt:  [20.917 Melem/s 20.965 Melem/s 21.011 Melem/s]
                 change:
                        time:   [−0.0300% +0.3732% +0.7727%] (p = 0.07 > 0.05)
                        thrpt:  [−0.7668% −0.3718% +0.0300%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
span/add_event          time:   [49.625 ns 50.237 ns 50.744 ns]
                        thrpt:  [19.707 Melem/s 19.905 Melem/s 20.151 Melem/s]
                 change:
                        time:   [−5.4342% −3.5033% −1.6277%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6546% +3.6305% +5.7464%]
                        Performance has improved.
span/with_kind          time:   [329.95 ns 331.03 ns 332.21 ns]
                        thrpt:  [3.0102 Melem/s 3.0208 Melem/s 3.0307 Melem/s]
                 change:
                        time:   [+1.8440% +2.4135% +2.9779%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8918% −2.3566% −1.8106%]
                        Performance has regressed.

context_store/create_context
                        time:   [96.981 µs 97.029 µs 97.076 µs]
                        thrpt:  [10.301 Kelem/s 10.306 Kelem/s 10.311 Kelem/s]
                 change:
                        time:   [−2.4602% −2.3888% −2.3174%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3724% +2.4473% +2.5223%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
