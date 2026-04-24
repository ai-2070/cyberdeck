     Running benches/auth_guard.rs (target/release/deps/auth_guard-48253259a03ece7b)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_threaBenchmarking auth_guard_check_fast_hit/single_threaBenchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (249Benchmarking auth_guard_check_fast_hit/single_threaauth_guard_check_fast_hit/single_thread
                        time:   [20.200 ns 20.270 ns 20.324 ns]
                        thrpt:  [49.203 Melem/s 49.334 Melem/s 49.504 Melem/s]
                 change:
                        time:   [+0.0688% +0.3648% +0.6813%] (p = 0.02 < 0.05)
                        thrpt:  [−0.6767% −0.3634% −0.0688%]
                        Change within noise threshold.

Benchmarking auth_guard_check_fast_miss/single_threBenchmarking auth_guard_check_fast_miss/single_threBenchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (1.2B iteratiauth_guard_check_fast_miss/single_thread
                        time:   [4.0890 ns 4.1006 ns 4.1151 ns]
                        thrpt:  [243.01 Melem/s 243.87 Melem/s 244.56 Melem/s]
                 change:
                        time:   [+1.0262% +1.3133% +1.6206%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5948% −1.2963% −1.0158%]
                        Performance has regressed.

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 s (16auth_guard_check_fast_contended/eight_threads
                        time:   [29.817 ns 29.973 ns 30.142 ns]
                        thrpt:  [33.176 Melem/s 33.363 Melem/s 33.538 Melem/s]
                 change:
                        time:   [−3.9869% −3.4716% −2.9327%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0213% +3.5965% +4.1525%]
                        Performance has improved.
Found 7 outliers among 50 measurements (14.00%)
  1 (2.00%) low mild
  2 (4.00%) high mild
  4 (8.00%) high severe

auth_guard_allow_channel/insert
                        time:   [192.32 ns 205.46 ns 217.89 ns]
                        thrpt:  [4.5895 Melem/s 4.8670 Melem/s 5.1995 Melem/s]
                 change:
                        time:   [−8.9061% −3.1287% +2.7545%] (p = 0.32 > 0.05)
                        thrpt:  [−2.6807% +3.2297% +9.7769%]
                        No change in performance detected.
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 7.6463 s (2550 iterauth_guard_hot_hit_ceiling/million_ops
                        time:   [2.9946 ms 2.9967 ms 2.9988 ms]
                        change: [−2.0753% −1.5350% −1.0356%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

     Running benches/cortex.rs (target/release/deps/cortex-1dcce40d61fc1588)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [260.93 ns 266.64 ns 272.98 ns]
                        thrpt:  [3.6632 Melem/s 3.7504 Melem/s 3.8325 Melem/s]
                 change:
                        time:   [−3.4709% +1.5054% +6.8382%] (p = 0.56 > 0.05)
                        thrpt:  [−6.4005% −1.4830% +3.5957%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
cortex_ingest/memories_store
                        time:   [405.73 ns 412.42 ns 419.56 ns]
                        thrpt:  [2.3834 Melem/s 2.4247 Melem/s 2.4647 Melem/s]
                 change:
                        time:   [+1.0199% +5.2706% +10.410%] (p = 0.03 < 0.05)
                        thrpt:  [−9.4282% −5.0067% −1.0096%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0059 s (874k cortex_fold_barrier/tasks_create_and_wait
                        time:   [5.6246 µs 5.6444 µs 5.6667 µs]
                        thrpt:  [176.47 Kelem/s 177.17 Kelem/s 177.79 Kelem/s]
                 change:
                        time:   [+0.1303% +0.9684% +1.7323%] (p = 0.02 < 0.05)
                        thrpt:  [−1.7028% −0.9591% −0.1302%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0247 s (838cortex_fold_barrier/memories_store_and_wait
                        time:   [5.7912 µs 5.8071 µs 5.8238 µs]
                        thrpt:  [171.71 Kelem/s 172.20 Kelem/s 172.68 Kelem/s]
                 change:
                        time:   [−0.1041% +0.7114% +1.6546%] (p = 0.11 > 0.05)
                        thrpt:  [−1.6277% −0.7063% +0.1042%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0040 s (5.1M iterationcortex_query/tasks_find_many/100
                        time:   [981.02 ns 982.64 ns 984.36 ns]
                        thrpt:  [101.59 Melem/s 101.77 Melem/s 101.93 Melem/s]
                 change:
                        time:   [−0.9101% −0.6639% −0.4225%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4243% +0.6684% +0.9185%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0005 s (33M iteratiocortex_query/tasks_count_where/100
                        time:   [151.94 ns 152.07 ns 152.20 ns]
                        thrpt:  [657.03 Melem/s 657.61 Melem/s 658.15 Melem/s]
                 change:
                        time:   [+0.3526% +0.5153% +0.6765%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6719% −0.5126% −0.3514%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (560M iteraticortex_query/tasks_find_unique/100
                        time:   [8.9485 ns 8.9652 ns 8.9820 ns]
                        thrpt:  [11.133 Gelem/s 11.154 Gelem/s 11.175 Gelem/s]
                 change:
                        time:   [+0.1016% +0.3339% +0.5660%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5629% −0.3328% −0.1015%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0125 s (1.1M itcortex_query/memories_find_many_tag/100
                        time:   [4.5511 µs 4.5550 µs 4.5595 µs]
                        thrpt:  [21.932 Melem/s 21.954 Melem/s 21.973 Melem/s]
                 change:
                        time:   [−3.9252% −3.6619% −3.4113%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5317% +3.8011% +4.0855%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0001 s (5.2M itercortex_query/memories_count_where/100
                        time:   [948.66 ns 951.18 ns 953.60 ns]
                        thrpt:  [104.87 Melem/s 105.13 Melem/s 105.41 Melem/s]
                 change:
                        time:   [+2.5228% +2.9981% +3.5103%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3913% −2.9108% −2.4608%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0065 s (656k iteratiocortex_query/tasks_find_many/1000
                        time:   [7.6008 µs 7.6160 µs 7.6320 µs]
                        thrpt:  [131.03 Melem/s 131.30 Melem/s 131.56 Melem/s]
                 change:
                        time:   [−5.0055% −4.6195% −4.2361%] (p = 0.00 < 0.05)
                        thrpt:  [+4.4235% +4.8433% +5.2692%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0022 s (3.2M iteratcortex_query/tasks_count_where/1000
                        time:   [1.5407 µs 1.5432 µs 1.5464 µs]
                        thrpt:  [646.66 Melem/s 647.99 Melem/s 649.04 Melem/s]
                 change:
                        time:   [+0.5422% +0.7535% +0.9949%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9851% −0.7478% −0.5393%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (560M iteratcortex_query/tasks_find_unique/1000
                        time:   [8.9498 ns 8.9600 ns 8.9705 ns]
                        thrpt:  [111.48 Gelem/s 111.61 Gelem/s 111.73 Gelem/s]
                 change:
                        time:   [−0.2009% +0.0249% +0.2403%] (p = 0.83 > 0.05)
                        thrpt:  [−0.2398% −0.0249% +0.2013%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.2431 s (106k icortex_query/memories_find_many_tag/1000
                        time:   [49.388 µs 49.449 µs 49.511 µs]
                        thrpt:  [20.197 Melem/s 20.223 Melem/s 20.248 Melem/s]
                 change:
                        time:   [+0.1899% +0.4128% +0.6390%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6349% −0.4111% −0.1895%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0378 s (495k itecortex_query/memories_count_where/1000
                        time:   [10.143 µs 10.160 µs 10.177 µs]
                        thrpt:  [98.257 Melem/s 98.425 Melem/s 98.593 Melem/s]
                 change:
                        time:   [+0.7552% +1.0194% +1.2741%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2581% −1.0091% −0.7495%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.0410 s (35k iteratiocortex_query/tasks_find_many/10000
                        time:   [138.41 µs 142.59 µs 146.76 µs]
                        thrpt:  [68.139 Melem/s 70.129 Melem/s 72.252 Melem/s]
                 change:
                        time:   [−20.557% −16.723% −12.702%] (p = 0.00 < 0.05)
                        thrpt:  [+14.550% +20.081% +25.877%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.1278 s (136k iteracortex_query/tasks_count_where/10000
                        time:   [36.984 µs 37.489 µs 38.010 µs]
                        thrpt:  [263.09 Melem/s 266.74 Melem/s 270.39 Melem/s]
                 change:
                        time:   [+58.710% +62.151% +65.832%] (p = 0.00 < 0.05)
                        thrpt:  [−39.698% −38.329% −36.992%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (556M iteracortex_query/tasks_find_unique/10000
                        time:   [8.9339 ns 8.9474 ns 8.9639 ns]
                        thrpt:  [1115.6 Gelem/s 1117.6 Gelem/s 1119.3 Gelem/s]
                 change:
                        time:   [−0.2960% +0.0312% +0.3417%] (p = 0.85 > 0.05)
                        thrpt:  [−0.3405% −0.0312% +0.2968%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 7.1206 s (10k icortex_query/memories_find_many_tag/10000
                        time:   [720.88 µs 725.41 µs 730.02 µs]
                        thrpt:  [13.698 Melem/s 13.785 Melem/s 13.872 Melem/s]
                 change:
                        time:   [+12.629% +16.334% +19.903%] (p = 0.00 < 0.05)
                        thrpt:  [−16.599% −14.040% −11.213%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.5797 s (40k itecortex_query/memories_count_where/10000
                        time:   [138.04 µs 138.20 µs 138.37 µs]
                        thrpt:  [72.271 Melem/s 72.357 Melem/s 72.443 Melem/s]
                 change:
                        time:   [+0.6390% +0.8919% +1.2274%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2126% −0.8840% −0.6350%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0086 s (1.6M iterationcortex_snapshot/tasks_encode/100
                        time:   [3.1353 µs 3.1392 µs 3.1432 µs]
                        thrpt:  [31.815 Melem/s 31.856 Melem/s 31.895 Melem/s]
                 change:
                        time:   [−0.4131% −0.1793% +0.0425%] (p = 0.14 > 0.05)
                        thrpt:  [−0.0425% +0.1797% +0.4148%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0207 s (934k iteratcortex_snapshot/memories_encode/100
                        time:   [5.3622 µs 5.3700 µs 5.3785 µs]
                        thrpt:  [18.593 Melem/s 18.622 Melem/s 18.649 Melem/s]
                 change:
                        time:   [−0.8272% −0.5932% −0.3479%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3491% +0.5967% +0.8341%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5.0066cortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.1376 µs 2.1398 µs 2.1422 µs]
                        thrpt:  [46.680 Melem/s 46.733 Melem/s 46.782 Melem/s]
                 change:
                        time:   [−0.5566% −0.3446% −0.1370%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1372% +0.3458% +0.5597%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0033 s (2.2M itcortex_snapshot/netdb_bundle_decode/100
                        time:   [2.2497 µs 2.2533 µs 2.2568 µs]
                        thrpt:  [44.311 Melem/s 44.380 Melem/s 44.450 Melem/s]
                 change:
                        time:   [−0.2730% −0.0265% +0.2253%] (p = 0.84 > 0.05)
                        thrpt:  [−0.2248% +0.0265% +0.2738%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.0565 s (162k iteratiocortex_snapshot/tasks_encode/1000
                        time:   [31.285 µs 31.337 µs 31.389 µs]
                        thrpt:  [31.858 Melem/s 31.911 Melem/s 31.964 Melem/s]
                 change:
                        time:   [+3.1672% +3.4101% +3.6514%] (p = 0.00 < 0.05)
                        thrpt:  [−3.5228% −3.2977% −3.0700%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.2399 s (91k iteratcortex_snapshot/memories_encode/1000
                        time:   [57.544 µs 57.598 µs 57.655 µs]
                        thrpt:  [17.345 Melem/s 17.362 Melem/s 17.378 Melem/s]
                 change:
                        time:   [+0.8321% +1.0532% +1.2604%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2448% −1.0423% −0.8253%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimated 5.03cortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [22.159 µs 22.182 µs 22.204 µs]
                        thrpt:  [45.037 Melem/s 45.082 Melem/s 45.129 Melem/s]
                 change:
                        time:   [−0.1908% −0.0034% +0.1705%] (p = 0.98 > 0.05)
                        thrpt:  [−0.1702% +0.0034% +0.1912%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0954 s (192k icortex_snapshot/netdb_bundle_decode/1000
                        time:   [26.508 µs 26.531 µs 26.556 µs]
                        thrpt:  [37.657 Melem/s 37.691 Melem/s 37.725 Melem/s]
                 change:
                        time:   [+0.0721% +0.2391% +0.4031%] (p = 0.01 < 0.05)
                        thrpt:  [−0.4014% −0.2385% −0.0721%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 5.0700 s (15k iteratiocortex_snapshot/tasks_encode/10000
                        time:   [335.38 µs 339.69 µs 343.88 µs]
                        thrpt:  [29.080 Melem/s 29.439 Melem/s 29.817 Melem/s]
                 change:
                        time:   [−9.5783% −7.3485% −4.9442%] (p = 0.00 < 0.05)
                        thrpt:  [+5.2014% +7.9313% +10.593%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 6.5989 s (10k iteracortex_snapshot/memories_encode/10000
                        time:   [658.54 µs 669.08 µs 679.79 µs]
                        thrpt:  [14.710 Melem/s 14.946 Melem/s 15.185 Melem/s]
                 change:
                        time:   [−2.2206% +0.2270% +2.8544%] (p = 0.87 > 0.05)
                        thrpt:  [−2.7752% −0.2264% +2.2710%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Collecting 100 samples in estimated 5.cortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [319.91 µs 327.21 µs 334.29 µs]
                        thrpt:  [29.915 Melem/s 30.561 Melem/s 31.259 Melem/s]
                 change:
                        time:   [+28.159% +32.214% +36.534%] (p = 0.00 < 0.05)
                        thrpt:  [−26.758% −24.365% −21.972%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 5.6842 s (20k icortex_snapshot/netdb_bundle_decode/10000
                        time:   [281.12 µs 281.63 µs 282.18 µs]
                        thrpt:  [35.438 Melem/s 35.507 Melem/s 35.572 Melem/s]
                 change:
                        time:   [−9.9864% −8.4622% −7.0210%] (p = 0.00 < 0.05)
                        thrpt:  [+7.5511% +9.2445% +11.094%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

     Running benches/ingestion.rs (target/release/deps/ingestion-0fee1add4fee7ff0)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [10.880 ns 10.892 ns 10.906 ns]
                        thrpt:  [91.697 Melem/s 91.807 Melem/s 91.915 Melem/s]
                 change:
                        time:   [−0.3765% −0.1433% +0.0901%] (p = 0.24 > 0.05)
                        thrpt:  [−0.0900% +0.1435% +0.3779%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
ring_buffer/push_pop/1024
                        time:   [10.434 ns 10.471 ns 10.508 ns]
                        thrpt:  [95.165 Melem/s 95.504 Melem/s 95.837 Melem/s]
                 change:
                        time:   [+0.2073% +0.4243% +0.6719%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6674% −0.4225% −0.2069%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
ring_buffer/push/8192   time:   [11.144 ns 11.156 ns 11.167 ns]
                        thrpt:  [89.552 Melem/s 89.640 Melem/s 89.731 Melem/s]
                 change:
                        time:   [+1.9247% +2.2788% +2.6350%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5673% −2.2280% −1.8884%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
ring_buffer/push_pop/8192
                        time:   [10.512 ns 10.543 ns 10.578 ns]
                        thrpt:  [94.532 Melem/s 94.847 Melem/s 95.127 Melem/s]
                 change:
                        time:   [+1.7997% +2.0319% +2.2808%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2300% −1.9915% −1.7679%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
ring_buffer/push/65536  time:   [10.843 ns 10.857 ns 10.871 ns]
                        thrpt:  [91.987 Melem/s 92.107 Melem/s 92.227 Melem/s]
                 change:
                        time:   [−0.9344% −0.1126% +0.6982%] (p = 0.80 > 0.05)
                        thrpt:  [−0.6934% +0.1127% +0.9432%]
                        No change in performance detected.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) low severe
  5 (5.00%) low mild
ring_buffer/push_pop/65536
                        time:   [10.425 ns 10.434 ns 10.443 ns]
                        thrpt:  [95.754 Melem/s 95.841 Melem/s 95.919 Melem/s]
                 change:
                        time:   [+0.0381% +0.2706% +0.4807%] (p = 0.02 < 0.05)
                        thrpt:  [−0.4784% −0.2699% −0.0381%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
ring_buffer/push/1048576
                        time:   [10.243 ns 10.301 ns 10.347 ns]
                        thrpt:  [96.649 Melem/s 97.078 Melem/s 97.624 Melem/s]
                 change:
                        time:   [−3.2917% −0.2060% +3.2340%] (p = 0.90 > 0.05)
                        thrpt:  [−3.1326% +0.2064% +3.4038%]
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  13 (13.00%) low mild
ring_buffer/push_pop/1048576
                        time:   [10.584 ns 10.614 ns 10.658 ns]
                        thrpt:  [93.826 Melem/s 94.212 Melem/s 94.478 Melem/s]
                 change:
                        time:   [−0.3013% +0.1667% +0.6401%] (p = 0.49 > 0.05)
                        thrpt:  [−0.6360% −0.1664% +0.3022%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

timestamp/next          time:   [7.5203 ns 7.5256 ns 7.5310 ns]
                        thrpt:  [132.79 Melem/s 132.88 Melem/s 132.97 Melem/s]
                 change:
                        time:   [+0.1062% +0.3029% +0.5208%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5181% −0.3020% −0.1061%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  6 (6.00%) high severe
timestamp/now_raw       time:   [624.98 ps 625.42 ps 625.88 ps]
                        thrpt:  [1.5977 Gelem/s 1.5989 Gelem/s 1.6001 Gelem/s]
                 change:
                        time:   [+0.0734% +0.2620% +0.4624%] (p = 0.01 < 0.05)
                        thrpt:  [−0.4602% −0.2613% −0.0734%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

event/internal_event_new
                        time:   [172.70 ns 172.85 ns 173.00 ns]
                        thrpt:  [5.7805 Melem/s 5.7855 Melem/s 5.7905 Melem/s]
                 change:
                        time:   [−0.1732% +0.0005% +0.1754%] (p = 1.00 > 0.05)
                        thrpt:  [−0.1751% −0.0005% +0.1735%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
event/json_creation     time:   [106.38 ns 106.74 ns 107.13 ns]
                        thrpt:  [9.3342 Melem/s 9.3685 Melem/s 9.4005 Melem/s]
                 change:
                        time:   [+0.9535% +1.3171% +1.6790%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6512% −1.3000% −0.9445%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

batch/pop_batch/100     time:   [779.23 ns 779.93 ns 780.63 ns]
                        thrpt:  [128.10 Melem/s 128.22 Melem/s 128.33 Melem/s]
                 change:
                        time:   [−0.0986% +0.2656% +0.5376%] (p = 0.10 > 0.05)
                        thrpt:  [−0.5348% −0.2649% +0.0987%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
batch/pop_batch/1000    time:   [43.997 µs 45.382 µs 46.887 µs]
                        thrpt:  [21.328 Melem/s 22.035 Melem/s 22.729 Melem/s]
                 change:
                        time:   [+7.7782% +13.485% +19.482%] (p = 0.00 < 0.05)
                        thrpt:  [−16.305% −11.883% −7.2169%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
batch/pop_batch/10000   time:   [850.14 µs 867.00 µs 884.97 µs]
                        thrpt:  [11.300 Melem/s 11.534 Melem/s 11.763 Melem/s]
                 change:
                        time:   [+21.685% +28.203% +35.345%] (p = 0.00 < 0.05)
                        thrpt:  [−26.115% −21.999% −17.820%]
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  8 (8.00%) high mild
  1 (1.00%) high severe

     Running benches/mesh.rs (target/release/deps/mesh-42e009289d1700c4)
Gnuplot not found, using plotters backend
mesh_reroute/triangle_failure
                        time:   [5.1262 µs 5.1994 µs 5.2760 µs]
                        thrpt:  [189.54 Kelem/s 192.33 Kelem/s 195.08 Kelem/s]
                 change:
                        time:   [−0.1509% +1.2521% +2.6518%] (p = 0.09 > 0.05)
                        thrpt:  [−2.5833% −1.2366% +0.1511%]
                        No change in performance detected.
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimated 5.0237 s (172k iterationsmesh_reroute/10_peers_10_routes
                        time:   [29.165 µs 29.580 µs 30.097 µs]
                        thrpt:  [33.226 Kelem/s 33.807 Kelem/s 34.287 Kelem/s]
                 change:
                        time:   [+1.1002% +2.1006% +3.2657%] (p = 0.00 < 0.05)
                        thrpt:  [−3.1625% −2.0574% −1.0882%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimated 6.5464 s (20k iterationsmesh_reroute/50_peers_100_routes
                        time:   [313.11 µs 317.50 µs 323.86 µs]
                        thrpt:  [3.0878 Kelem/s 3.1496 Kelem/s 3.1938 Kelem/s]
                 change:
                        time:   [+1.3466% +2.0126% +2.9676%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8821% −1.9729% −1.3287%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

mesh_proximity/on_pingwave_new
                        time:   [167.19 ns 170.79 ns 174.34 ns]
                        thrpt:  [5.7358 Melem/s 5.8551 Melem/s 5.9812 Melem/s]
                 change:
                        time:   [−4.2711% +1.1252% +6.6285%] (p = 0.68 > 0.05)
                        thrpt:  [−6.2164% −1.1127% +4.4617%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimated 5.0000 s (73M iterationsmesh_proximity/on_pingwave_dedup
                        time:   [68.075 ns 68.142 ns 68.210 ns]
                        thrpt:  [14.661 Melem/s 14.675 Melem/s 14.690 Melem/s]
                 change:
                        time:   [−0.0995% +0.1032% +0.3137%] (p = 0.35 > 0.05)
                        thrpt:  [−0.3127% −0.1031% +0.0996%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estimated 5.0000 s (2.5B iteratiomesh_proximity/pingwave_serialize
                        time:   [1.9770 ns 1.9803 ns 1.9847 ns]
                        thrpt:  [503.85 Melem/s 504.96 Melem/s 505.81 Melem/s]
                 change:
                        time:   [−0.2588% −0.0809% +0.1070%] (p = 0.39 > 0.05)
                        thrpt:  [−0.1069% +0.0810% +0.2594%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in estimated 5.0000 s (2.6B iteratmesh_proximity/pingwave_deserialize
                        time:   [1.9010 ns 1.9041 ns 1.9077 ns]
                        thrpt:  [524.19 Melem/s 525.17 Melem/s 526.03 Melem/s]
                 change:
                        time:   [+1.5175% +1.7169% +1.9100%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8742% −1.6879% −1.4948%]
                        Performance has regressed.
mesh_proximity/node_count
                        time:   [222.12 ns 251.31 ns 295.49 ns]
                        thrpt:  [3.3843 Melem/s 3.9791 Melem/s 4.5021 Melem/s]
                 change:
                        time:   [+5.9933% +13.893% +23.514%] (p = 0.00 < 0.05)
                        thrpt:  [−19.038% −12.198% −5.6544%]
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) high mild
  11 (11.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 52.1s, or reduce sample count to 10.
mesh_proximity/all_nodes_100
                        time:   [442.22 ms 445.20 ms 448.50 ms]
                        thrpt:  [2.2297  elem/s 2.2462  elem/s 2.2613  elem/s]
                 change:
                        time:   [+1.6785% +2.3551% +3.1461%] (p = 0.00 < 0.05)
                        thrpt:  [−3.0502% −2.3009% −1.6508%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe

mesh_dispatch/classify_direct
                        time:   [626.81 ps 627.63 ps 628.64 ps]
                        thrpt:  [1.5907 Gelem/s 1.5933 Gelem/s 1.5954 Gelem/s]
                 change:
                        time:   [+0.3560% +0.5233% +0.7082%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7032% −0.5206% −0.3548%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
mesh_dispatch/classify_routed
                        time:   [446.71 ps 447.05 ps 447.41 ps]
                        thrpt:  [2.2351 Gelem/s 2.2369 Gelem/s 2.2386 Gelem/s]
                 change:
                        time:   [−7.2241% −6.9806% −6.7095%] (p = 0.00 < 0.05)
                        thrpt:  [+7.1920% +7.5044% +7.7866%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
mesh_dispatch/classify_pingwave
                        time:   [314.37 ps 314.80 ps 315.41 ps]
                        thrpt:  [3.1705 Gelem/s 3.1766 Gelem/s 3.1810 Gelem/s]
                 change:
                        time:   [−0.0773% +0.2306% +0.5279%] (p = 0.13 > 0.05)
                        thrpt:  [−0.5251% −0.2301% +0.0773%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

mesh_routing/lookup_hit time:   [14.806 ns 14.868 ns 14.932 ns]
                        thrpt:  [66.968 Melem/s 67.258 Melem/s 67.539 Melem/s]
                 change:
                        time:   [−4.5603% −3.4069% −2.2210%] (p = 0.00 < 0.05)
                        thrpt:  [+2.2714% +3.5270% +4.7782%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
mesh_routing/lookup_miss
                        time:   [14.968 ns 15.075 ns 15.177 ns]
                        thrpt:  [65.889 Melem/s 66.335 Melem/s 66.811 Melem/s]
                 change:
                        time:   [+1.9290% +2.8871% +3.9378%] (p = 0.00 < 0.05)
                        thrpt:  [−3.7886% −2.8061% −1.8925%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
mesh_routing/is_local   time:   [323.52 ps 324.25 ps 325.02 ps]
                        thrpt:  [3.0767 Gelem/s 3.0840 Gelem/s 3.0910 Gelem/s]
                 change:
                        time:   [+2.7601% +3.1280% +3.4843%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3670% −3.0331% −2.6860%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
mesh_routing/all_routes/10
                        time:   [1.3722 µs 1.3769 µs 1.3817 µs]
                        thrpt:  [723.74 Kelem/s 726.28 Kelem/s 728.77 Kelem/s]
                 change:
                        time:   [+4.0340% +4.3747% +4.7070%] (p = 0.00 < 0.05)
                        thrpt:  [−4.4954% −4.1913% −3.8776%]
                        Performance has regressed.
mesh_routing/all_routes/100
                        time:   [2.2539 µs 2.2572 µs 2.2606 µs]
                        thrpt:  [442.36 Kelem/s 443.02 Kelem/s 443.67 Kelem/s]
                 change:
                        time:   [+1.7626% +2.1763% +2.5930%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5275% −2.1299% −1.7321%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  9 (9.00%) high severe
mesh_routing/all_routes/1000
                        time:   [11.831 µs 11.848 µs 11.866 µs]
                        thrpt:  [84.276 Kelem/s 84.400 Kelem/s 84.522 Kelem/s]
                 change:
                        time:   [+1.0026% +1.3354% +1.6858%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6579% −1.3178% −0.9927%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
mesh_routing/add_route  time:   [45.026 ns 45.215 ns 45.398 ns]
                        thrpt:  [22.027 Melem/s 22.117 Melem/s 22.209 Melem/s]
                 change:
                        time:   [+4.5132% +6.3092% +8.1552%] (p = 0.00 < 0.05)
                        thrpt:  [−7.5402% −5.9348% −4.3183%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild

     Running benches/net.rs (target/release/deps/net-826d8a1cebce9bac)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [2.0373 ns 2.0393 ns 2.0412 ns]
                        thrpt:  [489.92 Melem/s 490.37 Melem/s 490.84 Melem/s]
                 change:
                        time:   [+2.8074% +3.0228% +3.2199%] (p = 0.00 < 0.05)
                        thrpt:  [−3.1195% −2.9341% −2.7307%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  4 (4.00%) high severe
net_header/deserialize  time:   [2.1658 ns 2.1676 ns 2.1693 ns]
                        thrpt:  [460.99 Melem/s 461.34 Melem/s 461.71 Melem/s]
                 change:
                        time:   [+2.8972% +3.0350% +3.1695%] (p = 0.00 < 0.05)
                        thrpt:  [−3.0721% −2.9456% −2.8156%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
net_header/roundtrip    time:   [2.1244 ns 2.1268 ns 2.1297 ns]
                        thrpt:  [469.54 Melem/s 470.18 Melem/s 470.72 Melem/s]
                 change:
                        time:   [+1.4493% +1.7654% +2.0634%] (p = 0.00 < 0.05)
                        thrpt:  [−2.0217% −1.7348% −1.4286%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimated 5.0001 s (273M iterationsnet_event_frame/write_single/64
                        time:   [18.381 ns 18.400 ns 18.421 ns]
                        thrpt:  [3.2356 GiB/s 3.2393 GiB/s 3.2428 GiB/s]
                 change:
                        time:   [+1.8473% +2.0984% +2.3409%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2873% −2.0553% −1.8138%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estimated 5.0002 s (108M iterationnet_event_frame/write_single/256
                        time:   [46.167 ns 46.346 ns 46.542 ns]
                        thrpt:  [5.1227 GiB/s 5.1443 GiB/s 5.1643 GiB/s]
                 change:
                        time:   [−5.5585% −4.6343% −3.6931%] (p = 0.00 < 0.05)
                        thrpt:  [+3.8347% +4.8595% +5.8856%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0000 s (137M iterationet_event_frame/write_single/1024
                        time:   [36.196 ns 36.219 ns 36.244 ns]
                        thrpt:  [26.312 GiB/s 26.331 GiB/s 26.347 GiB/s]
                 change:
                        time:   [+0.9646% +1.1372% +1.3192%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3020% −1.1244% −0.9553%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0002 s (62M iterationnet_event_frame/write_single/4096
                        time:   [79.042 ns 79.589 ns 80.193 ns]
                        thrpt:  [47.569 GiB/s 47.930 GiB/s 48.262 GiB/s]
                 change:
                        time:   [−2.0307% +1.0425% +4.0132%] (p = 0.52 > 0.05)
                        thrpt:  [−3.8583% −1.0317% +2.0727%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
net_event_frame/write_batch/1
                        time:   [18.221 ns 18.249 ns 18.281 ns]
                        thrpt:  [3.2604 GiB/s 3.2661 GiB/s 3.2712 GiB/s]
                 change:
                        time:   [+0.5627% +0.8749% +1.1653%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1519% −0.8674% −0.5596%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
net_event_frame/write_batch/10
                        time:   [69.912 ns 70.012 ns 70.111 ns]
                        thrpt:  [8.5014 GiB/s 8.5135 GiB/s 8.5257 GiB/s]
                 change:
                        time:   [+0.1718% +0.3857% +0.6251%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6212% −0.3842% −0.1715%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
net_event_frame/write_batch/50
                        time:   [149.33 ns 149.92 ns 150.71 ns]
                        thrpt:  [19.774 GiB/s 19.879 GiB/s 19.957 GiB/s]
                 change:
                        time:   [+0.5553% +0.9593% +1.3671%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3486% −0.9502% −0.5522%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  7 (7.00%) high severe
net_event_frame/write_batch/100
                        time:   [274.69 ns 274.91 ns 275.14 ns]
                        thrpt:  [21.664 GiB/s 21.681 GiB/s 21.698 GiB/s]
                 change:
                        time:   [+0.6203% +0.8107% +1.0224%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0121% −0.8042% −0.6165%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
net_event_frame/read_batch_10
                        time:   [135.21 ns 135.96 ns 136.79 ns]
                        thrpt:  [73.106 Melem/s 73.549 Melem/s 73.961 Melem/s]
                 change:
                        time:   [−3.2253% −2.6185% −2.0034%] (p = 0.00 < 0.05)
                        thrpt:  [+2.0443% +2.6889% +3.3328%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

net_packet_pool/get_return/16
                        time:   [38.910 ns 39.005 ns 39.135 ns]
                        thrpt:  [25.552 Melem/s 25.637 Melem/s 25.701 Melem/s]
                 change:
                        time:   [+1.3071% +1.5491% +1.7940%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7623% −1.5255% −1.2903%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
net_packet_pool/get_return/64
                        time:   [38.557 ns 38.603 ns 38.648 ns]
                        thrpt:  [25.874 Melem/s 25.905 Melem/s 25.936 Melem/s]
                 change:
                        time:   [+1.2005% +1.4308% +1.6755%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6478% −1.4106% −1.1863%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
net_packet_pool/get_return/256
                        time:   [38.544 ns 38.603 ns 38.664 ns]
                        thrpt:  [25.864 Melem/s 25.905 Melem/s 25.944 Melem/s]
                 change:
                        time:   [+1.4693% +1.6626% +1.8733%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8389% −1.6355% −1.4481%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

net_packet_build/build_packet/1
                        time:   [486.44 ns 487.03 ns 487.84 ns]
                        thrpt:  [125.11 MiB/s 125.32 MiB/s 125.47 MiB/s]
                 change:
                        time:   [−1.2312% −0.2083% +0.4830%] (p = 0.72 > 0.05)
                        thrpt:  [−0.4807% +0.2087% +1.2466%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0007 s (2.7M iterationnet_packet_build/build_packet/10
                        time:   [1.8567 µs 1.8611 µs 1.8670 µs]
                        thrpt:  [326.91 MiB/s 327.95 MiB/s 328.72 MiB/s]
                 change:
                        time:   [+0.4917% +0.7822% +1.0914%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0796% −0.7761% −0.4893%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0188 s (601k iterationnet_packet_build/build_packet/50
                        time:   [8.2373 µs 8.2478 µs 8.2589 µs]
                        thrpt:  [369.51 MiB/s 370.01 MiB/s 370.48 MiB/s]
                 change:
                        time:   [+0.2377% +0.4924% +0.7393%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7339% −0.4900% −0.2371%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

net_encryption/encrypt/64
                        time:   [484.38 ns 484.82 ns 485.25 ns]
                        thrpt:  [125.78 MiB/s 125.89 MiB/s 126.01 MiB/s]
                 change:
                        time:   [−0.6562% −0.3810% −0.0981%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0982% +0.3825% +0.6605%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
net_encryption/encrypt/256
                        time:   [926.13 ns 928.72 ns 932.10 ns]
                        thrpt:  [261.93 MiB/s 262.88 MiB/s 263.61 MiB/s]
                 change:
                        time:   [+0.2277% +0.6242% +1.0265%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0161% −0.6203% −0.2272%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
net_encryption/encrypt/1024
                        time:   [2.7305 µs 2.7398 µs 2.7505 µs]
                        thrpt:  [355.05 MiB/s 356.43 MiB/s 357.65 MiB/s]
                 change:
                        time:   [+1.7573% +2.0584% +2.3618%] (p = 0.00 < 0.05)
                        thrpt:  [−2.3074% −2.0169% −1.7270%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
net_encryption/encrypt/4096
                        time:   [9.8097 µs 9.8193 µs 9.8290 µs]
                        thrpt:  [397.42 MiB/s 397.81 MiB/s 398.20 MiB/s]
                 change:
                        time:   [+0.3315% +0.4955% +0.6561%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6518% −0.4930% −0.3304%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

net_keypair/generate    time:   [12.767 µs 12.910 µs 13.068 µs]
                        thrpt:  [76.522 Kelem/s 77.457 Kelem/s 78.324 Kelem/s]
                 change:
                        time:   [+2.7386% +3.5080% +4.3845%] (p = 0.00 < 0.05)
                        thrpt:  [−4.2003% −3.3891% −2.6656%]
                        Performance has regressed.

net_aad/generate        time:   [2.0335 ns 2.0359 ns 2.0393 ns]
                        thrpt:  [490.36 Melem/s 491.19 Melem/s 491.77 Melem/s]
                 change:
                        time:   [+0.2426% +0.3985% +0.5534%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5503% −0.3969% −0.2420%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0000 s (129M itepool_comparison/shared_pool_get_return
                        time:   [38.424 ns 38.476 ns 38.528 ns]
                        thrpt:  [25.955 Melem/s 25.990 Melem/s 26.025 Melem/s]
                 change:
                        time:   [+1.0199% +1.2263% +1.4262%] (p = 0.00 < 0.05)
                        thrpt:  [−1.4061% −1.2114% −1.0096%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.0002 s (61pool_comparison/thread_local_pool_get_return
                        time:   [82.395 ns 82.483 ns 82.597 ns]
                        thrpt:  [12.107 Melem/s 12.124 Melem/s 12.137 Melem/s]
                 change:
                        time:   [−6.3378% −4.4012% −2.5355%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6015% +4.6038% +6.7666%]
                        Performance has improved.
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) high mild
  12 (12.00%) high severe
pool_comparison/shared_pool_10x
                        time:   [343.58 ns 344.22 ns 344.97 ns]
                        thrpt:  [2.8988 Melem/s 2.9051 Melem/s 2.9105 Melem/s]
                 change:
                        time:   [+1.1673% +1.3655% +1.5760%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5516% −1.3471% −1.1539%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0016 s (5.2M iterpool_comparison/thread_local_pool_10x
                        time:   [1.1021 µs 1.1033 µs 1.1049 µs]
                        thrpt:  [905.08 Kelem/s 906.35 Kelem/s 907.39 Kelem/s]
                 change:
                        time:   [+13.047% +13.852% +14.535%] (p = 0.00 < 0.05)
                        thrpt:  [−12.690% −12.167% −11.541%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0003 s (10M iterationscipher_comparison/shared_pool/64
                        time:   [485.21 ns 485.65 ns 486.14 ns]
                        thrpt:  [125.55 MiB/s 125.68 MiB/s 125.79 MiB/s]
                 change:
                        time:   [−2.8919% −2.5844% −2.2787%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3319% +2.6530% +2.9780%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0002 s (9.3M iteraticipher_comparison/fast_chacha20/64
                        time:   [532.12 ns 532.62 ns 533.17 ns]
                        thrpt:  [114.48 MiB/s 114.60 MiB/s 114.70 MiB/s]
                 change:
                        time:   [−1.5963% −1.3381% −1.0655%] (p = 0.00 < 0.05)
                        thrpt:  [+1.0770% +1.3562% +1.6222%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0021 s (5.2M iteratiocipher_comparison/shared_pool/256
                        time:   [924.53 ns 925.90 ns 927.36 ns]
                        thrpt:  [263.26 MiB/s 263.68 MiB/s 264.07 MiB/s]
                 change:
                        time:   [−0.1174% +0.2173% +0.5775%] (p = 0.23 > 0.05)
                        thrpt:  [−0.5742% −0.2168% +0.1175%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0001 s (5.2M iteratcipher_comparison/fast_chacha20/256
                        time:   [967.57 ns 969.30 ns 971.52 ns]
                        thrpt:  [251.30 MiB/s 251.87 MiB/s 252.32 MiB/s]
                 change:
                        time:   [+0.1906% +0.3709% +0.5832%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5798% −0.3695% −0.1903%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0000 s (1.8M iteraticipher_comparison/shared_pool/1024
                        time:   [2.7881 µs 2.8337 µs 2.8951 µs]
                        thrpt:  [337.32 MiB/s 344.63 MiB/s 350.26 MiB/s]
                 change:
                        time:   [+2.8448% +4.0601% +5.7008%] (p = 0.00 < 0.05)
                        thrpt:  [−5.3933% −3.9017% −2.7661%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0026 s (1.8M iteracipher_comparison/fast_chacha20/1024
                        time:   [2.7232 µs 2.7250 µs 2.7269 µs]
                        thrpt:  [358.12 MiB/s 358.37 MiB/s 358.61 MiB/s]
                 change:
                        time:   [−0.1954% +0.1964% +0.5023%] (p = 0.31 > 0.05)
                        thrpt:  [−0.4998% −0.1960% +0.1958%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0191 s (510k iteraticipher_comparison/shared_pool/4096
                        time:   [9.8458 µs 9.9068 µs 9.9842 µs]
                        thrpt:  [391.24 MiB/s 394.30 MiB/s 396.74 MiB/s]
                 change:
                        time:   [+0.6119% +0.9180% +1.2566%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2410% −0.9096% −0.6082%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0480 s (515k iteracipher_comparison/fast_chacha20/4096
                        time:   [9.7878 µs 9.7963 µs 9.8065 µs]
                        thrpt:  [398.33 MiB/s 398.75 MiB/s 399.09 MiB/s]
                 change:
                        time:   [+0.3890% +0.5519% +0.7352%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7299% −0.5489% −0.3875%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high severe

adaptive_batcher/optimal_size
                        time:   [979.17 ps 980.28 ps 981.36 ps]
                        thrpt:  [1.0190 Gelem/s 1.0201 Gelem/s 1.0213 Gelem/s]
                 change:
                        time:   [+0.1531% +0.3337% +0.5083%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5057% −0.3326% −0.1529%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
adaptive_batcher/record time:   [3.8878 ns 3.8910 ns 3.8943 ns]
                        thrpt:  [256.78 Melem/s 257.01 Melem/s 257.22 Melem/s]
                 change:
                        time:   [+0.1571% +0.3029% +0.4433%] (p = 0.00 < 0.05)
                        thrpt:  [−0.4414% −0.3019% −0.1569%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
adaptive_batcher/full_cycle
                        time:   [4.4080 ns 4.4137 ns 4.4208 ns]
                        thrpt:  [226.20 Melem/s 226.57 Melem/s 226.86 Melem/s]
                 change:
                        time:   [+0.2195% +0.4305% +0.6471%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6429% −0.4287% −0.2190%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0340 s (611k itee2e_packet_build/shared_pool_50_events
                        time:   [8.2473 µs 8.2717 µs 8.3008 µs]
                        thrpt:  [367.65 MiB/s 368.94 MiB/s 370.03 MiB/s]
                 change:
                        time:   [+0.4152% +0.6605% +0.9372%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9285% −0.6562% −0.4134%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0373 s (611k iterationse2e_packet_build/fast_50_events
                        time:   [8.2354 µs 8.2439 µs 8.2531 µs]
                        thrpt:  [369.77 MiB/s 370.18 MiB/s 370.57 MiB/s]
                 change:
                        time:   [+0.3120% +0.4769% +0.6405%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6364% −0.4747% −0.3111%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.8s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 9.7865 s (5050 itemultithread_packet_build/shared_pool/8
                        time:   [1.8493 ms 1.8533 ms 1.8577 ms]
                        thrpt:  [4.3064 Melem/s 4.3166 Melem/s 4.3260 Melem/s]
                 change:
                        time:   [+1.0600% +1.7927% +2.5108%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4493% −1.7611% −1.0489%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 8.8644 s (10multithread_packet_build/thread_local_pool/8
                        time:   [863.10 µs 865.13 µs 867.38 µs]
                        thrpt:  [9.2231 Melem/s 9.2471 Melem/s 9.2689 Melem/s]
                 change:
                        time:   [−3.0195% −1.8522% −0.6605%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6649% +1.8871% +3.1135%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.2294 s (1100 itmultithread_packet_build/shared_pool/16
                        time:   [4.6962 ms 4.8674 ms 5.0522 ms]
                        thrpt:  [3.1669 Melem/s 3.2872 Melem/s 3.4070 Melem/s]
                 change:
                        time:   [+3.2820% +7.5968% +12.352%] (p = 0.00 < 0.05)
                        thrpt:  [−10.994% −7.0604% −3.1777%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.6s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 8.6139 s (5multithread_packet_build/thread_local_pool/16
                        time:   [1.6862 ms 1.6915 ms 1.6982 ms]
                        thrpt:  [9.4220 Melem/s 9.4592 Melem/s 9.4889 Melem/s]
                 change:
                        time:   [−2.1098% −1.3978% −0.5810%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5844% +1.4176% +2.1552%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  6 (6.00%) high mild
  9 (9.00%) high severe
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.1159 s (700 itemultithread_packet_build/shared_pool/24
                        time:   [7.4484 ms 7.7064 ms 7.9848 ms]
                        thrpt:  [3.0057 Melem/s 3.1143 Melem/s 3.2222 Melem/s]
                 change:
                        time:   [+2.8740% +6.9924% +11.763%] (p = 0.00 < 0.05)
                        thrpt:  [−10.525% −6.5354% −2.7937%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.0648 s (2multithread_packet_build/thread_local_pool/24
                        time:   [2.5219 ms 2.5354 ms 2.5514 ms]
                        thrpt:  [9.4066 Melem/s 9.4659 Melem/s 9.5168 Melem/s]
                 change:
                        time:   [+0.4668% +1.0689% +1.7122%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6833% −1.0576% −0.4646%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.9667 s (600 itemultithread_packet_build/shared_pool/32
                        time:   [10.249 ms 10.676 ms 11.126 ms]
                        thrpt:  [2.8762 Melem/s 2.9972 Melem/s 3.1222 Melem/s]
                 change:
                        time:   [−1.9835% +4.3393% +10.898%] (p = 0.18 > 0.05)
                        thrpt:  [−9.8274% −4.1588% +2.0236%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.3000 s (1multithread_packet_build/thread_local_pool/32
                        time:   [3.3047 ms 3.3113 ms 3.3183 ms]
                        thrpt:  [9.6434 Melem/s 9.6638 Melem/s 9.6831 Melem/s]
                 change:
                        time:   [+0.5004% +0.7919% +1.1040%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0919% −0.7857% −0.4979%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.0s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 7.0373 s (5050 itmultithread_mixed_frames/shared_mixed/8
                        time:   [1.4018 ms 1.4377 ms 1.4854 ms]
                        thrpt:  [8.0787 Melem/s 8.3466 Melem/s 8.5602 Melem/s]
                 change:
                        time:   [−0.2042% +1.3150% +3.0628%] (p = 0.12 > 0.05)
                        thrpt:  [−2.9718% −1.2979% +0.2046%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.2s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 6.1945 s (5050 iterations)^C
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net % clear
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
    Finished `bench` profile [optimized] target(s) in 0.15s
     Running unittests src/lib.rs (target/release/deps/net-012c650a62ef09c2)

running 1110 tests
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
test adapter::net::crypto::tests::session_prefix_stable_for_same_id ... ignored
test adapter::net::crypto::tests::session_prefix_uses_high_bits_of_session_id ... ignored
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
test adapter::net::protocol::tests::test_event_frame_length_prefix_fits_u32 ... ignored
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
test consumer::merge::tests::test_poll_merger_filter_insertion_ts_truncates_after_sort ... ignored
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
test ffi::mesh::tests::alloc_bytes_round_trip_across_sizes ... ignored
test ffi::mesh::tests::hardware_from_json_saturates_overflow_cpu_fields ... ignored
test ffi::mesh::tests::net_free_bytes_null_and_zero_len_are_noops ... ignored
test ffi::mesh::tests::net_mesh_shutdown_runs_even_with_outstanding_arc_refs ... ignored
test ffi::mesh::tests::saturating_u16_cap_clamps_at_u16_max ... ignored
test ffi::tests::test_parse_config_empty ... ignored
test ffi::tests::test_parse_config_invalid_json ... ignored
test ffi::tests::test_parse_config_num_shards_max_valid ... ignored
test ffi::tests::test_parse_config_num_shards_overflow ... ignored
test ffi::tests::test_parse_config_valid ... ignored
test ffi::tests::test_parse_poll_request_empty_uses_default_limit ... ignored
test ffi::tests::test_parse_poll_request_limit_at_usize_max ... ignored
test ffi::tests::test_parse_poll_request_no_cursor_defaults_to_none ... ignored
test ffi::tests::test_parse_poll_request_null_fields_use_defaults ... ignored
test ffi::tests::test_parse_poll_request_preserves_cursor ... ignored
test ffi::tests::test_parse_poll_request_wrong_type_cursor_errors ... ignored
test ffi::tests::test_parse_poll_request_wrong_type_limit_errors ... ignored
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
test shard::tests::test_ingest_raw_batch_drop_accounting ... ignored
test shard::tests::test_ingest_raw_batch_empty ... ignored
test shard::tests::test_ingest_raw_batch_routes_and_preserves_order ... ignored
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

test result: ok. 0 passed; 0 failed; 1110 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running benches/auth_guard.rs (target/release/deps/auth_guard-48253259a03ece7b)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (246M iteauth_guard_check_fast_hit/single_thread
                        time:   [20.426 ns 20.468 ns 20.513 ns]
                        thrpt:  [48.750 Melem/s 48.856 Melem/s 48.956 Melem/s]
                 change:
                        time:   [+1.1551% +1.4882% +1.8096%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7774% −1.4664% −1.1419%]
                        Performance has regressed.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (1.2B itauth_guard_check_fast_miss/single_thread
                        time:   [4.1662 ns 4.1746 ns 4.1832 ns]
                        thrpt:  [239.05 Melem/s 239.54 Melem/s 240.03 Melem/s]
                 change:
                        time:   [+1.1132% +1.4426% +1.7885%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7570% −1.4221% −1.1009%]
                        Performance has regressed.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 s (16auth_guard_check_fast_contended/eight_threads
                        time:   [30.234 ns 30.622 ns 31.070 ns]
                        thrpt:  [32.185 Melem/s 32.656 Melem/s 33.075 Melem/s]
                 change:
                        time:   [+0.9523% +2.1349% +3.1857%] (p = 0.00 < 0.05)
                        thrpt:  [−3.0873% −2.0903% −0.9433%]
                        Change within noise threshold.
Found 2 outliers among 50 measurements (4.00%)
  1 (2.00%) high mild
  1 (2.00%) high severe

auth_guard_allow_channel/insert
                        time:   [196.95 ns 209.86 ns 221.63 ns]
                        thrpt:  [4.5121 Melem/s 4.7651 Melem/s 5.0775 Melem/s]
                 change:
                        time:   [−6.1926% +0.1265% +6.8637%] (p = 0.97 > 0.05)
                        thrpt:  [−6.4229% −0.1263% +6.6014%]
                        No change in performance detected.

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 7.8008 s (2550 iterauth_guard_hot_hit_ceiling/million_ops
                        time:   [3.0544 ms 3.0571 ms 3.0598 ms]
                        change: [+1.8511% +1.9772% +2.1015%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

     Running benches/cortex.rs (target/release/deps/cortex-1dcce40d61fc1588)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [289.75 ns 298.11 ns 307.00 ns]
                        thrpt:  [3.2573 Melem/s 3.3545 Melem/s 3.4513 Melem/s]
                 change:
                        time:   [+2.4382% +7.8594% +13.230%] (p = 0.00 < 0.05)
                        thrpt:  [−11.684% −7.2867% −2.3802%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
cortex_ingest/memories_store
                        time:   [425.54 ns 433.57 ns 442.51 ns]
                        thrpt:  [2.2598 Melem/s 2.3065 Melem/s 2.3499 Melem/s]
                 change:
                        time:   [−1.1294% +3.7438% +8.6890%] (p = 0.12 > 0.05)
                        thrpt:  [−7.9944% −3.6087% +1.1423%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0095 s (813k cortex_fold_barrier/tasks_create_and_wait
                        time:   [5.7804 µs 5.8081 µs 5.8400 µs]
                        thrpt:  [171.23 Kelem/s 172.17 Kelem/s 173.00 Kelem/s]
                 change:
                        time:   [+1.5127% +2.4071% +3.3029%] (p = 0.00 < 0.05)
                        thrpt:  [−3.1973% −2.3505% −1.4902%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0026 s (808k iterations)^C
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net %
lazlo@Lazlos-MBP net % clear
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
    Finished `bench` profile [optimized] target(s) in 0.21s
     Running unittests src/lib.rs (target/release/deps/net-012c650a62ef09c2)

running 1110 tests
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
test adapter::net::crypto::tests::session_prefix_stable_for_same_id ... ignored
test adapter::net::crypto::tests::session_prefix_uses_high_bits_of_session_id ... ignored
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
test adapter::net::protocol::tests::test_event_frame_length_prefix_fits_u32 ... ignored
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
test consumer::merge::tests::test_poll_merger_filter_insertion_ts_truncates_after_sort ... ignored
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
test ffi::mesh::tests::alloc_bytes_round_trip_across_sizes ... ignored
test ffi::mesh::tests::hardware_from_json_saturates_overflow_cpu_fields ... ignored
test ffi::mesh::tests::net_free_bytes_null_and_zero_len_are_noops ... ignored
test ffi::mesh::tests::net_mesh_shutdown_runs_even_with_outstanding_arc_refs ... ignored
test ffi::mesh::tests::saturating_u16_cap_clamps_at_u16_max ... ignored
test ffi::tests::test_parse_config_empty ... ignored
test ffi::tests::test_parse_config_invalid_json ... ignored
test ffi::tests::test_parse_config_num_shards_max_valid ... ignored
test ffi::tests::test_parse_config_num_shards_overflow ... ignored
test ffi::tests::test_parse_config_valid ... ignored
test ffi::tests::test_parse_poll_request_empty_uses_default_limit ... ignored
test ffi::tests::test_parse_poll_request_limit_at_usize_max ... ignored
test ffi::tests::test_parse_poll_request_no_cursor_defaults_to_none ... ignored
test ffi::tests::test_parse_poll_request_null_fields_use_defaults ... ignored
test ffi::tests::test_parse_poll_request_preserves_cursor ... ignored
test ffi::tests::test_parse_poll_request_wrong_type_cursor_errors ... ignored
test ffi::tests::test_parse_poll_request_wrong_type_limit_errors ... ignored
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
test shard::tests::test_ingest_raw_batch_drop_accounting ... ignored
test shard::tests::test_ingest_raw_batch_empty ... ignored
test shard::tests::test_ingest_raw_batch_routes_and_preserves_order ... ignored
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

test result: ok. 0 passed; 0 failed; 1110 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/auth_guard.rs (target/release/deps/auth_guard-48253259a03ece7b)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (248M iteauth_guard_check_fast_hit/single_thread
                        time:   [20.055 ns 20.087 ns 20.121 ns]
                        thrpt:  [49.698 Melem/s 49.784 Melem/s 49.863 Melem/s]
                 change:
                        time:   [−1.7482% −1.4654% −1.1707%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1845% +1.4872% +1.7793%]
                        Performance has improved.
Found 3 outliers among 50 measurements (6.00%)
  3 (6.00%) high mild

Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (1.2B itauth_guard_check_fast_miss/single_thread
                        time:   [4.0776 ns 4.0867 ns 4.0959 ns]
                        thrpt:  [244.14 Melem/s 244.70 Melem/s 245.24 Melem/s]
                 change:
                        time:   [−2.4810% −2.1878% −1.8904%] (p = 0.00 < 0.05)
                        thrpt:  [+1.9268% +2.2367% +2.5442%]
                        Performance has improved.
Found 2 outliers among 50 measurements (4.00%)
  1 (2.00%) low mild
  1 (2.00%) high mild

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 s (15auth_guard_check_fast_contended/eight_threads
                        time:   [32.431 ns 32.617 ns 32.815 ns]
                        thrpt:  [30.474 Melem/s 30.659 Melem/s 30.835 Melem/s]
                 change:
                        time:   [+6.2133% +7.5130% +8.8188%] (p = 0.00 < 0.05)
                        thrpt:  [−8.1041% −6.9880% −5.8499%]
                        Performance has regressed.
Found 2 outliers among 50 measurements (4.00%)
  2 (4.00%) high mild

auth_guard_allow_channel/insert
                        time:   [189.86 ns 201.26 ns 211.67 ns]
                        thrpt:  [4.7243 Melem/s 4.9686 Melem/s 5.2670 Melem/s]
                 change:
                        time:   [−7.6931% −1.3059% +5.6247%] (p = 0.70 > 0.05)
                        thrpt:  [−5.3252% +1.3232% +8.3343%]
                        No change in performance detected.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 7.6561 s (2550 iterauth_guard_hot_hit_ceiling/million_ops
                        time:   [2.9945 ms 2.9961 ms 2.9977 ms]
                        change: [−2.0722% −1.9192% −1.7404%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 50 measurements (10.00%)
  2 (4.00%) high mild
  3 (6.00%) high severe

     Running benches/cortex.rs (target/release/deps/cortex-1dcce40d61fc1588)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [258.71 ns 265.50 ns 273.19 ns]
                        thrpt:  [3.6605 Melem/s 3.7665 Melem/s 3.8653 Melem/s]
                 change:
                        time:   [−11.772% −7.1092% −0.9572%] (p = 0.01 < 0.05)
                        thrpt:  [+0.9665% +7.6533% +13.343%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
cortex_ingest/memories_store
                        time:   [397.27 ns 404.37 ns 412.24 ns]
                        thrpt:  [2.4258 Melem/s 2.4730 Melem/s 2.5172 Melem/s]
                 change:
                        time:   [−10.505% −7.0414% −3.1810%] (p = 0.00 < 0.05)
                        thrpt:  [+3.2855% +7.5747% +11.738%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0186 s (874k cortex_fold_barrier/tasks_create_and_wait
                        time:   [5.6048 µs 5.6178 µs 5.6317 µs]
                        thrpt:  [177.57 Kelem/s 178.01 Kelem/s 178.42 Kelem/s]
                 change:
                        time:   [−3.8461% −3.1358% −2.5147%] (p = 0.00 < 0.05)
                        thrpt:  [+2.5796% +3.2373% +3.9999%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0281 s (843cortex_fold_barrier/memories_store_and_wait
                        time:   [5.7687 µs 5.7895 µs 5.8112 µs]
                        thrpt:  [172.08 Kelem/s 172.73 Kelem/s 173.35 Kelem/s]
                 change:
                        time:   [−1.8637% −0.9753% −0.2309%] (p = 0.01 < 0.05)
                        thrpt:  [+0.2314% +0.9849% +1.8991%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0018 s (5.1M iterationcortex_query/tasks_find_many/100
                        time:   [979.19 ns 981.74 ns 985.26 ns]
                        thrpt:  [101.50 Melem/s 101.86 Melem/s 102.13 Melem/s]
                 change:
                        time:   [−0.5562% −0.2517% +0.0818%] (p = 0.12 > 0.05)
                        thrpt:  [−0.0817% +0.2524% +0.5593%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0002 s (33M iteratiocortex_query/tasks_count_where/100
                        time:   [153.12 ns 153.21 ns 153.30 ns]
                        thrpt:  [652.31 Melem/s 652.69 Melem/s 653.06 Melem/s]
                 change:
                        time:   [+0.6338% +0.8314% +1.0257%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0153% −0.8245% −0.6298%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (558M iteraticortex_query/tasks_find_unique/100
                        time:   [8.9476 ns 8.9804 ns 9.0206 ns]
                        thrpt:  [11.086 Gelem/s 11.135 Gelem/s 11.176 Gelem/s]
                 change:
                        time:   [−0.0157% +0.3152% +0.7078%] (p = 0.10 > 0.05)
                        thrpt:  [−0.7028% −0.3142% +0.0157%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0023 s (1.1M itcortex_query/memories_find_many_tag/100
                        time:   [4.6163 µs 4.6211 µs 4.6263 µs]
                        thrpt:  [21.615 Melem/s 21.640 Melem/s 21.662 Melem/s]
                 change:
                        time:   [+1.3522% +1.5605% +1.7643%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7337% −1.5365% −1.3342%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0036 s (5.6M itercortex_query/memories_count_where/100
                        time:   [890.98 ns 898.38 ns 905.71 ns]
                        thrpt:  [110.41 Melem/s 111.31 Melem/s 112.24 Melem/s]
                 change:
                        time:   [−6.0956% −5.5277% −4.9728%] (p = 0.00 < 0.05)
                        thrpt:  [+5.2330% +5.8511% +6.4913%]
                        Performance has improved.
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0120 s (667k iteratiocortex_query/tasks_find_many/1000
                        time:   [7.4867 µs 7.5067 µs 7.5276 µs]
                        thrpt:  [132.85 Melem/s 133.21 Melem/s 133.57 Melem/s]
                 change:
                        time:   [−1.6182% −1.3084% −0.9893%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9992% +1.3257% +1.6448%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0033 s (3.2M iteratcortex_query/tasks_count_where/1000
                        time:   [1.5393 µs 1.5454 µs 1.5519 µs]
                        thrpt:  [644.35 Melem/s 647.07 Melem/s 649.64 Melem/s]
                 change:
                        time:   [−0.1060% +0.2845% +0.6759%] (p = 0.16 > 0.05)
                        thrpt:  [−0.6713% −0.2837% +0.1061%]
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) high mild
  8 (8.00%) high severe
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (557M iteratcortex_query/tasks_find_unique/1000
                        time:   [8.9528 ns 8.9786 ns 9.0087 ns]
                        thrpt:  [111.00 Gelem/s 111.38 Gelem/s 111.70 Gelem/s]
                 change:
                        time:   [−0.0198% +0.3193% +0.6582%] (p = 0.07 > 0.05)
                        thrpt:  [−0.6539% −0.3183% +0.0198%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.1911 s (106k icortex_query/memories_find_many_tag/1000
                        time:   [48.911 µs 48.964 µs 49.017 µs]
                        thrpt:  [20.401 Melem/s 20.423 Melem/s 20.445 Melem/s]
                 change:
                        time:   [−1.3335% −1.0890% −0.8480%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8552% +1.1010% +1.3515%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0061 s (480k itecortex_query/memories_count_where/1000
                        time:   [10.415 µs 10.436 µs 10.456 µs]
                        thrpt:  [95.641 Melem/s 95.823 Melem/s 96.018 Melem/s]
                 change:
                        time:   [+2.0008% +2.3602% +2.6633%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5942% −2.3058% −1.9615%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.1626 s (45k iteratiocortex_query/tasks_find_many/10000
                        time:   [110.77 µs 112.81 µs 115.05 µs]
                        thrpt:  [86.921 Melem/s 88.645 Melem/s 90.277 Melem/s]
                 change:
                        time:   [−25.362% −22.597% −19.581%] (p = 0.00 < 0.05)
                        thrpt:  [+24.349% +29.194% +33.980%]
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.0535 s (131k iteracortex_query/tasks_count_where/10000
                        time:   [37.667 µs 38.191 µs 38.738 µs]
                        thrpt:  [258.15 Melem/s 261.84 Melem/s 265.48 Melem/s]
                 change:
                        time:   [−2.0339% −0.4604% +1.2262%] (p = 0.60 > 0.05)
                        thrpt:  [−1.2114% +0.4625% +2.0761%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (558M iteracortex_query/tasks_find_unique/10000
                        time:   [8.9989 ns 9.0283 ns 9.0607 ns]
                        thrpt:  [1103.7 Gelem/s 1107.6 Gelem/s 1111.2 Gelem/s]
                 change:
                        time:   [+0.1330% +0.5366% +0.9233%] (p = 0.01 < 0.05)
                        thrpt:  [−0.9148% −0.5338% −0.1328%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 5.6010 s (10k icortex_query/memories_find_many_tag/10000
                        time:   [549.82 µs 555.94 µs 562.36 µs]
                        thrpt:  [17.782 Melem/s 17.988 Melem/s 18.188 Melem/s]
                 change:
                        time:   [−23.696% −22.249% −20.884%] (p = 0.00 < 0.05)
                        thrpt:  [+26.397% +28.616% +31.054%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.6418 s (40k itecortex_query/memories_count_where/10000
                        time:   [139.10 µs 139.30 µs 139.49 µs]
                        thrpt:  [71.689 Melem/s 71.790 Melem/s 71.893 Melem/s]
                 change:
                        time:   [+0.3235% +0.6318% +0.9017%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8936% −0.6279% −0.3225%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0147 s (1.6M iterationcortex_snapshot/tasks_encode/100
                        time:   [3.1231 µs 3.1272 µs 3.1316 µs]
                        thrpt:  [31.932 Melem/s 31.977 Melem/s 32.019 Melem/s]
                 change:
                        time:   [−0.6878% −0.4718% −0.2659%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2666% +0.4741% +0.6926%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0189 s (924k iteratcortex_snapshot/memories_encode/100
                        time:   [5.4104 µs 5.4193 µs 5.4283 µs]
                        thrpt:  [18.422 Melem/s 18.452 Melem/s 18.483 Melem/s]
                 change:
                        time:   [+0.5617% +0.8663% +1.1684%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1549% −0.8589% −0.5585%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5.0003cortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [2.1440 µs 2.1485 µs 2.1533 µs]
                        thrpt:  [46.441 Melem/s 46.544 Melem/s 46.642 Melem/s]
                 change:
                        time:   [+0.2084% +0.4934% +0.7525%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7469% −0.4910% −0.2079%]
                        Change within noise threshold.
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0090 s (2.2M itcortex_snapshot/netdb_bundle_decode/100
                        time:   [2.2588 µs 2.2662 µs 2.2744 µs]
                        thrpt:  [43.967 Melem/s 44.128 Melem/s 44.270 Melem/s]
                 change:
                        time:   [+0.0484% +0.3329% +0.6122%] (p = 0.02 < 0.05)
                        thrpt:  [−0.6085% −0.3318% −0.0483%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.0317 s (162k iteratiocortex_snapshot/tasks_encode/1000
                        time:   [31.120 µs 31.162 µs 31.204 µs]
                        thrpt:  [32.047 Melem/s 32.090 Melem/s 32.134 Melem/s]
                 change:
                        time:   [−0.7840% −0.5461% −0.3256%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3267% +0.5491% +0.7902%]
                        Change within noise threshold.
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.2463 s (91k iteratcortex_snapshot/memories_encode/1000
                        time:   [57.594 µs 57.664 µs 57.734 µs]
                        thrpt:  [17.321 Melem/s 17.342 Melem/s 17.363 Melem/s]
                 change:
                        time:   [−0.1259% +0.1403% +0.4567%] (p = 0.37 > 0.05)
                        thrpt:  [−0.4547% −0.1401% +0.1261%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimated 5.03cortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [22.138 µs 22.176 µs 22.217 µs]
                        thrpt:  [45.011 Melem/s 45.093 Melem/s 45.171 Melem/s]
                 change:
                        time:   [−0.2352% −0.0336% +0.1764%] (p = 0.75 > 0.05)
                        thrpt:  [−0.1761% +0.0336% +0.2358%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0861 s (192k icortex_snapshot/netdb_bundle_decode/1000
                        time:   [26.470 µs 26.493 µs 26.518 µs]
                        thrpt:  [37.711 Melem/s 37.746 Melem/s 37.779 Melem/s]
                 change:
                        time:   [−0.4156% −0.2446% −0.0771%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0771% +0.2452% +0.4174%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 5.0578 s (15k iteratiocortex_snapshot/tasks_encode/10000
                        time:   [324.23 µs 329.49 µs 334.98 µs]
                        thrpt:  [29.852 Melem/s 30.350 Melem/s 30.842 Melem/s]
                 change:
                        time:   [−3.5737% −1.3020% +1.3595%] (p = 0.32 > 0.05)
                        thrpt:  [−1.3413% +1.3192% +3.7062%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 7.2735 s (10k iteracortex_snapshot/memories_encode/10000
                        time:   [703.08 µs 713.90 µs 724.89 µs]
                        thrpt:  [13.795 Melem/s 14.007 Melem/s 14.223 Melem/s]
                 change:
                        time:   [−0.0196% +2.7360% +5.4133%] (p = 0.04 < 0.05)
                        thrpt:  [−5.1353% −2.6631% +0.0196%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Collecting 100 samples in estimated 6.cortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [302.86 µs 310.40 µs 318.01 µs]
                        thrpt:  [31.445 Melem/s 32.216 Melem/s 33.019 Melem/s]
                 change:
                        time:   [−8.3716% −4.5479% −0.8072%] (p = 0.02 < 0.05)
                        thrpt:  [+0.8138% +4.7646% +9.1364%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 5.6758 s (20k icortex_snapshot/netdb_bundle_decode/10000
                        time:   [280.46 µs 280.80 µs 281.17 µs]
                        thrpt:  [35.566 Melem/s 35.612 Melem/s 35.655 Melem/s]
                 change:
                        time:   [−0.5689% −0.2911% −0.0050%] (p = 0.05 < 0.05)
                        thrpt:  [+0.0050% +0.2919% +0.5721%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

     Running benches/ingestion.rs (target/release/deps/ingestion-0fee1add4fee7ff0)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [10.849 ns 10.860 ns 10.872 ns]
                        thrpt:  [91.983 Melem/s 92.080 Melem/s 92.178 Melem/s]
                 change:
                        time:   [−0.5674% −0.2603% +0.0971%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0970% +0.2610% +0.5707%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
ring_buffer/push_pop/1024
                        time:   [10.390 ns 10.397 ns 10.405 ns]
                        thrpt:  [96.103 Melem/s 96.178 Melem/s 96.248 Melem/s]
                 change:
                        time:   [−0.5935% −0.3776% −0.1722%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1725% +0.3791% +0.5970%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
ring_buffer/push/8192   time:   [10.867 ns 10.882 ns 10.899 ns]
                        thrpt:  [91.755 Melem/s 91.893 Melem/s 92.023 Melem/s]
                 change:
                        time:   [−2.7313% −2.4053% −2.0527%] (p = 0.00 < 0.05)
                        thrpt:  [+2.0957% +2.4646% +2.8080%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
ring_buffer/push_pop/8192
                        time:   [10.392 ns 10.400 ns 10.408 ns]
                        thrpt:  [96.084 Melem/s 96.157 Melem/s 96.227 Melem/s]
                 change:
                        time:   [−2.0756% −1.8079% −1.5367%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5607% +1.8412% +2.1196%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
ring_buffer/push/65536  time:   [10.820 ns 10.834 ns 10.847 ns]
                        thrpt:  [92.188 Melem/s 92.300 Melem/s 92.423 Melem/s]
                 change:
                        time:   [−1.1101% −0.2117% +0.6930%] (p = 0.66 > 0.05)
                        thrpt:  [−0.6883% +0.2121% +1.1226%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low severe
  7 (7.00%) low mild
ring_buffer/push_pop/65536
                        time:   [10.439 ns 10.449 ns 10.458 ns]
                        thrpt:  [95.619 Melem/s 95.706 Melem/s 95.795 Melem/s]
                 change:
                        time:   [−0.1958% +0.0230% +0.2336%] (p = 0.83 > 0.05)
                        thrpt:  [−0.2330% −0.0230% +0.1962%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
ring_buffer/push/1048576
                        time:   [10.226 ns 10.278 ns 10.318 ns]
                        thrpt:  [96.915 Melem/s 97.298 Melem/s 97.794 Melem/s]
                 change:
                        time:   [−3.2134% −0.0680% +3.2898%] (p = 0.97 > 0.05)
                        thrpt:  [−3.1850% +0.0681% +3.3201%]
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  14 (14.00%) low mild
ring_buffer/push_pop/1048576
                        time:   [10.555 ns 10.573 ns 10.594 ns]
                        thrpt:  [94.391 Melem/s 94.578 Melem/s 94.741 Melem/s]
                 change:
                        time:   [−0.6438% −0.0432% +0.6832%] (p = 0.90 > 0.05)
                        thrpt:  [−0.6786% +0.0432% +0.6479%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

timestamp/next          time:   [7.5023 ns 7.5072 ns 7.5123 ns]
                        thrpt:  [133.12 Melem/s 133.21 Melem/s 133.29 Melem/s]
                 change:
                        time:   [−0.5197% −0.2602% −0.0164%] (p = 0.04 < 0.05)
                        thrpt:  [+0.0164% +0.2608% +0.5224%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
timestamp/now_raw       time:   [623.74 ps 624.27 ps 624.82 ps]
                        thrpt:  [1.6005 Gelem/s 1.6019 Gelem/s 1.6032 Gelem/s]
                 change:
                        time:   [−0.5007% −0.2768% −0.0770%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0770% +0.2776% +0.5032%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

event/internal_event_new
                        time:   [176.49 ns 177.09 ns 177.68 ns]
                        thrpt:  [5.6282 Melem/s 5.6469 Melem/s 5.6662 Melem/s]
                 change:
                        time:   [+2.2406% +2.5707% +2.9391%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8552% −2.5063% −2.1915%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
event/json_creation     time:   [105.34 ns 105.61 ns 105.87 ns]
                        thrpt:  [9.4455 Melem/s 9.4692 Melem/s 9.4933 Melem/s]
                 change:
                        time:   [−1.5775% −1.1612% −0.7505%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7562% +1.1749% +1.6028%]
                        Change within noise threshold.

batch/pop_batch/100     time:   [778.16 ns 778.76 ns 779.40 ns]
                        thrpt:  [128.30 Melem/s 128.41 Melem/s 128.51 Melem/s]
                 change:
                        time:   [−0.3069% −0.1359% +0.0258%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0258% +0.1361% +0.3078%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
batch/pop_batch/1000    time:   [44.333 µs 45.451 µs 46.580 µs]
                        thrpt:  [21.469 Melem/s 22.002 Melem/s 22.556 Melem/s]
                 change:
                        time:   [−4.0876% −0.9546% +2.5004%] (p = 0.59 > 0.05)
                        thrpt:  [−2.4394% +0.9638% +4.2619%]
                        No change in performance detected.
batch/pop_batch/10000   time:   [785.40 µs 805.41 µs 825.82 µs]
                        thrpt:  [12.109 Melem/s 12.416 Melem/s 12.732 Melem/s]
                 change:
                        time:   [−9.5469% −6.6449% −3.5594%] (p = 0.00 < 0.05)
                        thrpt:  [+3.6908% +7.1179% +10.555%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

     Running benches/mesh.rs (target/release/deps/mesh-42e009289d1700c4)
Gnuplot not found, using plotters backend
mesh_reroute/triangle_failure
                        time:   [5.0210 µs 5.0705 µs 5.1205 µs]
                        thrpt:  [195.29 Kelem/s 197.22 Kelem/s 199.16 Kelem/s]
                 change:
                        time:   [−3.5663% −2.3033% −0.8411%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8482% +2.3576% +3.6982%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimated 5.1387 s (177k iterationsmesh_reroute/10_peers_10_routes
                        time:   [28.238 µs 28.419 µs 28.619 µs]
                        thrpt:  [34.942 Kelem/s 35.187 Kelem/s 35.413 Kelem/s]
                 change:
                        time:   [−3.8655% −2.8063% −1.8323%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8665% +2.8873% +4.0210%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimated 6.3235 s (20k iterationsmesh_reroute/50_peers_100_routes
                        time:   [314.13 µs 314.81 µs 315.54 µs]
                        thrpt:  [3.1692 Kelem/s 3.1766 Kelem/s 3.1834 Kelem/s]
                 change:
                        time:   [−1.0098% −0.1290% +0.5183%] (p = 0.78 > 0.05)
                        thrpt:  [−0.5157% +0.1292% +1.0201%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild

mesh_proximity/on_pingwave_new
                        time:   [164.10 ns 168.70 ns 173.36 ns]
                        thrpt:  [5.7683 Melem/s 5.9276 Melem/s 6.0938 Melem/s]
                 change:
                        time:   [−8.2789% −2.9771% +2.1987%] (p = 0.26 > 0.05)
                        thrpt:  [−2.1514% +3.0684% +9.0261%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimated 5.0003 s (73M iterationsmesh_proximity/on_pingwave_dedup
                        time:   [68.379 ns 68.422 ns 68.467 ns]
                        thrpt:  [14.606 Melem/s 14.615 Melem/s 14.624 Melem/s]
                 change:
                        time:   [+0.0299% +0.2396% +0.4290%] (p = 0.02 < 0.05)
                        thrpt:  [−0.4272% −0.2391% −0.0299%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estimated 5.0000 s (2.5B iteratiomesh_proximity/pingwave_serialize
                        time:   [1.9804 ns 1.9822 ns 1.9844 ns]
                        thrpt:  [503.93 Melem/s 504.48 Melem/s 504.96 Melem/s]
                 change:
                        time:   [+0.0326% +0.2602% +0.5030%] (p = 0.02 < 0.05)
                        thrpt:  [−0.5005% −0.2595% −0.0326%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in estimated 5.0000 s (2.7B iteratmesh_proximity/pingwave_deserialize
                        time:   [1.8720 ns 1.8731 ns 1.8743 ns]
                        thrpt:  [533.52 Melem/s 533.86 Melem/s 534.18 Melem/s]
                 change:
                        time:   [−1.8463% −1.6649% −1.4785%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5007% +1.6931% +1.8811%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
mesh_proximity/node_count
                        time:   [200.44 ns 200.59 ns 200.75 ns]
                        thrpt:  [4.9813 Melem/s 4.9852 Melem/s 4.9890 Melem/s]
                 change:
                        time:   [−19.904% −12.195% −6.0651%] (p = 0.00 < 0.05)
                        thrpt:  [+6.4568% +13.889% +24.851%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 44.1s, or reduce sample count to 10.
mesh_proximity/all_nodes_100
                        time:   [428.54 ms 429.15 ms 429.79 ms]
                        thrpt:  [2.3267  elem/s 2.3302  elem/s 2.3335  elem/s]
                 change:
                        time:   [−4.3264% −3.6044% −2.9435%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0328% +3.7392% +4.5221%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

mesh_dispatch/classify_direct
                        time:   [624.15 ps 624.52 ps 624.91 ps]
                        thrpt:  [1.6002 Gelem/s 1.6012 Gelem/s 1.6022 Gelem/s]
                 change:
                        time:   [−0.5891% −0.3424% −0.0721%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0722% +0.3436% +0.5926%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
mesh_dispatch/classify_routed
                        time:   [444.73 ps 444.96 ps 445.20 ps]
                        thrpt:  [2.2462 Gelem/s 2.2474 Gelem/s 2.2486 Gelem/s]
                 change:
                        time:   [−0.7153% −0.3991% −0.0509%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0510% +0.4007% +0.7204%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
mesh_dispatch/classify_pingwave
                        time:   [314.70 ps 315.32 ps 315.96 ps]
                        thrpt:  [3.1649 Gelem/s 3.1714 Gelem/s 3.1776 Gelem/s]
                 change:
                        time:   [−0.1453% +0.1671% +0.4842%] (p = 0.31 > 0.05)
                        thrpt:  [−0.4819% −0.1668% +0.1455%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

mesh_routing/lookup_hit time:   [14.813 ns 14.862 ns 14.909 ns]
                        thrpt:  [67.072 Melem/s 67.286 Melem/s 67.510 Melem/s]
                 change:
                        time:   [−1.9342% −1.0157% −0.0977%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0978% +1.0261% +1.9724%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
mesh_routing/lookup_miss
                        time:   [14.833 ns 14.905 ns 14.968 ns]
                        thrpt:  [66.811 Melem/s 67.093 Melem/s 67.415 Melem/s]
                 change:
                        time:   [−2.5313% −1.6975% −0.8433%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8504% +1.7269% +2.5970%]
                        Change within noise threshold.
Found 21 outliers among 100 measurements (21.00%)
  7 (7.00%) low severe
  8 (8.00%) low mild
  6 (6.00%) high mild
mesh_routing/is_local   time:   [314.14 ps 314.72 ps 315.34 ps]
                        thrpt:  [3.1712 Gelem/s 3.1774 Gelem/s 3.1833 Gelem/s]
                 change:
                        time:   [−3.2682% −2.9281% −2.5690%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6367% +3.0164% +3.3786%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
mesh_routing/all_routes/10
                        time:   [1.3009 µs 1.3021 µs 1.3033 µs]
                        thrpt:  [767.28 Kelem/s 768.00 Kelem/s 768.67 Kelem/s]
                 change:
                        time:   [−6.2218% −5.9442% −5.6614%] (p = 0.00 < 0.05)
                        thrpt:  [+6.0012% +6.3199% +6.6346%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
mesh_routing/all_routes/100
                        time:   [2.2555 µs 2.2615 µs 2.2681 µs]
                        thrpt:  [440.91 Kelem/s 442.18 Kelem/s 443.36 Kelem/s]
                 change:
                        time:   [−0.7472% −0.3264% +0.1338%] (p = 0.14 > 0.05)
                        thrpt:  [−0.1336% +0.3275% +0.7528%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
mesh_routing/all_routes/1000
                        time:   [11.752 µs 11.788 µs 11.834 µs]
                        thrpt:  [84.501 Kelem/s 84.832 Kelem/s 85.095 Kelem/s]
                 change:
                        time:   [−0.9535% −0.5886% −0.2451%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2457% +0.5921% +0.9627%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) low mild
  2 (2.00%) high mild
  6 (6.00%) high severe
mesh_routing/add_route  time:   [42.280 ns 42.906 ns 43.471 ns]
                        thrpt:  [23.004 Melem/s 23.307 Melem/s 23.652 Melem/s]
                 change:
                        time:   [−7.1673% −5.2233% −3.5723%] (p = 0.00 < 0.05)
                        thrpt:  [+3.7047% +5.5112% +7.7207%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild

     Running benches/net.rs (target/release/deps/net-826d8a1cebce9bac)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [1.9794 ns 1.9810 ns 1.9827 ns]
                        thrpt:  [504.38 Melem/s 504.80 Melem/s 505.19 Melem/s]
                 change:
                        time:   [−3.1192% −2.9305% −2.7564%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8345% +3.0190% +3.2196%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
net_header/deserialize  time:   [2.1043 ns 2.1056 ns 2.1069 ns]
                        thrpt:  [474.62 Melem/s 474.93 Melem/s 475.21 Melem/s]
                 change:
                        time:   [−3.0375% −2.8944% −2.7432%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8206% +2.9806% +3.1326%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
net_header/roundtrip    time:   [2.1035 ns 2.1049 ns 2.1065 ns]
                        thrpt:  [474.73 Melem/s 475.08 Melem/s 475.40 Melem/s]
                 change:
                        time:   [−2.0243% −1.7350% −1.4658%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4876% +1.7656% +2.0661%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimated 5.0001 s (274M iterationsnet_event_frame/write_single/64
                        time:   [18.256 ns 18.281 ns 18.306 ns]
                        thrpt:  [3.2560 GiB/s 3.2606 GiB/s 3.2650 GiB/s]
                 change:
                        time:   [−0.6889% −0.4514% −0.2023%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2027% +0.4535% +0.6937%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estimated 5.0001 s (98M iterationsnet_event_frame/write_single/256
                        time:   [49.810 ns 50.357 ns 50.883 ns]
                        thrpt:  [4.6857 GiB/s 4.7346 GiB/s 4.7866 GiB/s]
                 change:
                        time:   [+6.5448% +7.7510% +8.8454%] (p = 0.00 < 0.05)
                        thrpt:  [−8.1266% −7.1934% −6.1428%]
                        Performance has regressed.
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0001 s (138M iterationet_event_frame/write_single/1024
                        time:   [36.076 ns 36.119 ns 36.166 ns]
                        thrpt:  [26.370 GiB/s 26.403 GiB/s 26.435 GiB/s]
                 change:
                        time:   [−0.6078% −0.3361% −0.0318%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0318% +0.3372% +0.6116%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0002 s (64M iterationnet_event_frame/write_single/4096
                        time:   [77.435 ns 77.877 ns 78.358 ns]
                        thrpt:  [48.683 GiB/s 48.983 GiB/s 49.263 GiB/s]
                 change:
                        time:   [−6.5341% −4.2841% −2.4204%] (p = 0.00 < 0.05)
                        thrpt:  [+2.4804% +4.4759% +6.9909%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
net_event_frame/write_batch/1
                        time:   [18.268 ns 18.311 ns 18.348 ns]
                        thrpt:  [3.2485 GiB/s 3.2551 GiB/s 3.2628 GiB/s]
                 change:
                        time:   [+0.2930% +0.5713% +0.8377%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8307% −0.5681% −0.2922%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low severe
  7 (7.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
net_event_frame/write_batch/10
                        time:   [71.987 ns 72.303 ns 72.637 ns]
                        thrpt:  [8.2058 GiB/s 8.2437 GiB/s 8.2799 GiB/s]
                 change:
                        time:   [+3.2162% +3.7887% +4.2820%] (p = 0.00 < 0.05)
                        thrpt:  [−4.1062% −3.6504% −3.1159%]
                        Performance has regressed.
net_event_frame/write_batch/50
                        time:   [147.90 ns 148.02 ns 148.13 ns]
                        thrpt:  [20.118 GiB/s 20.134 GiB/s 20.150 GiB/s]
                 change:
                        time:   [−1.5809% −1.1580% −0.7846%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7908% +1.1715% +1.6063%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
net_event_frame/write_batch/100
                        time:   [273.07 ns 273.29 ns 273.51 ns]
                        thrpt:  [21.793 GiB/s 21.810 GiB/s 21.828 GiB/s]
                 change:
                        time:   [−0.9270% −0.7499% −0.5840%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5874% +0.7555% +0.9357%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
net_event_frame/read_batch_10
                        time:   [138.91 ns 140.05 ns 141.29 ns]
                        thrpt:  [70.777 Melem/s 71.403 Melem/s 71.989 Melem/s]
                 change:
                        time:   [+2.6361% +3.5276% +4.4772%] (p = 0.00 < 0.05)
                        thrpt:  [−4.2854% −3.4074% −2.5684%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

net_packet_pool/get_return/16
                        time:   [38.647 ns 38.701 ns 38.756 ns]
                        thrpt:  [25.803 Melem/s 25.839 Melem/s 25.875 Melem/s]
                 change:
                        time:   [−0.9608% −0.7263% −0.5032%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5057% +0.7316% +0.9701%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
net_packet_pool/get_return/64
                        time:   [38.335 ns 38.383 ns 38.430 ns]
                        thrpt:  [26.022 Melem/s 26.053 Melem/s 26.086 Melem/s]
                 change:
                        time:   [−0.7946% −0.5798% −0.3687%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3700% +0.5832% +0.8009%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
net_packet_pool/get_return/256
                        time:   [38.236 ns 38.283 ns 38.329 ns]
                        thrpt:  [26.090 Melem/s 26.121 Melem/s 26.153 Melem/s]
                 change:
                        time:   [−0.9616% −0.7507% −0.5459%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5489% +0.7564% +0.9709%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

net_packet_build/build_packet/1
                        time:   [483.03 ns 483.41 ns 483.80 ns]
                        thrpt:  [126.16 MiB/s 126.26 MiB/s 126.36 MiB/s]
                 change:
                        time:   [−1.2840% −0.9422% −0.6459%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6501% +0.9512% +1.3007%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0083 s (2.7M iterationnet_packet_build/build_packet/10
                        time:   [1.8445 µs 1.8467 µs 1.8490 µs]
                        thrpt:  [330.10 MiB/s 330.51 MiB/s 330.91 MiB/s]
                 change:
                        time:   [−1.2126% −0.8993% −0.5905%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5940% +0.9075% +1.2275%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0204 s (611k iterationnet_packet_build/build_packet/50
                        time:   [8.2064 µs 8.2177 µs 8.2295 µs]
                        thrpt:  [370.83 MiB/s 371.36 MiB/s 371.87 MiB/s]
                 change:
                        time:   [−0.6263% −0.3989% −0.1759%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1762% +0.4005% +0.6303%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

net_encryption/encrypt/64
                        time:   [482.90 ns 483.38 ns 483.88 ns]
                        thrpt:  [126.14 MiB/s 126.27 MiB/s 126.39 MiB/s]
                 change:
                        time:   [−0.5047% −0.2758% −0.0447%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0447% +0.2765% +0.5073%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
net_encryption/encrypt/256
                        time:   [920.95 ns 922.20 ns 923.53 ns]
                        thrpt:  [264.36 MiB/s 264.74 MiB/s 265.10 MiB/s]
                 change:
                        time:   [−1.2511% −0.8506% −0.4673%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4695% +0.8579% +1.2669%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
net_encryption/encrypt/1024
                        time:   [2.6898 µs 2.6930 µs 2.6963 µs]
                        thrpt:  [362.19 MiB/s 362.64 MiB/s 363.06 MiB/s]
                 change:
                        time:   [−2.4734% −2.1234% −1.7907%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8233% +2.1695% +2.5361%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
net_encryption/encrypt/4096
                        time:   [9.7636 µs 9.7720 µs 9.7805 µs]
                        thrpt:  [399.39 MiB/s 399.74 MiB/s 400.08 MiB/s]
                 change:
                        time:   [−0.5843% −0.3583% −0.0876%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0877% +0.3596% +0.5877%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

net_keypair/generate    time:   [12.729 µs 12.870 µs 13.024 µs]
                        thrpt:  [76.779 Kelem/s 77.702 Kelem/s 78.562 Kelem/s]
                 change:
                        time:   [−1.5291% −0.4288% +0.7068%] (p = 0.46 > 0.05)
                        thrpt:  [−0.7018% +0.4307% +1.5528%]
                        No change in performance detected.

net_aad/generate        time:   [2.0299 ns 2.0312 ns 2.0325 ns]
                        thrpt:  [492.01 Melem/s 492.33 Melem/s 492.64 Melem/s]
                 change:
                        time:   [−0.3660% −0.2143% −0.0614%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0614% +0.2148% +0.3674%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0002 s (130M itepool_comparison/shared_pool_get_return
                        time:   [38.351 ns 38.402 ns 38.450 ns]
                        thrpt:  [26.008 Melem/s 26.041 Melem/s 26.075 Melem/s]
                 change:
                        time:   [−0.5184% −0.3334% −0.1367%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1369% +0.3345% +0.5211%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.0001 s (61pool_comparison/thread_local_pool_get_return
                        time:   [82.278 ns 82.337 ns 82.402 ns]
                        thrpt:  [12.136 Melem/s 12.145 Melem/s 12.154 Melem/s]
                 change:
                        time:   [−0.9474% −0.6131% −0.3058%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3067% +0.6169% +0.9565%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
pool_comparison/shared_pool_10x
                        time:   [341.04 ns 341.46 ns 341.89 ns]
                        thrpt:  [2.9249 Melem/s 2.9286 Melem/s 2.9322 Melem/s]
                 change:
                        time:   [−1.0360% −0.7825% −0.4642%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4664% +0.7887% +1.0469%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0009 s (5.2M iterpool_comparison/thread_local_pool_10x
                        time:   [1.0987 µs 1.0996 µs 1.1004 µs]
                        thrpt:  [908.75 Kelem/s 909.40 Kelem/s 910.13 Kelem/s]
                 change:
                        time:   [−0.7576% −0.4516% −0.1538%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1540% +0.4537% +0.7634%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0015 s (10M iterationscipher_comparison/shared_pool/64
                        time:   [483.33 ns 483.75 ns 484.19 ns]
                        thrpt:  [126.06 MiB/s 126.17 MiB/s 126.28 MiB/s]
                 change:
                        time:   [−0.6150% −0.3897% −0.1492%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1495% +0.3912% +0.6188%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0005 s (9.4M iteraticipher_comparison/fast_chacha20/64
                        time:   [530.34 ns 530.66 ns 530.98 ns]
                        thrpt:  [114.95 MiB/s 115.02 MiB/s 115.09 MiB/s]
                 change:
                        time:   [−0.5530% −0.4104% −0.2643%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2650% +0.4121% +0.5561%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0033 s (5.4M iteratiocipher_comparison/shared_pool/256
                        time:   [922.25 ns 923.32 ns 924.46 ns]
                        thrpt:  [264.09 MiB/s 264.42 MiB/s 264.72 MiB/s]
                 change:
                        time:   [−0.6353% −0.2935% +0.0507%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0507% +0.2943% +0.6394%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0007 s (5.2M iteratcipher_comparison/fast_chacha20/256
                        time:   [964.93 ns 965.59 ns 966.31 ns]
                        thrpt:  [252.65 MiB/s 252.84 MiB/s 253.01 MiB/s]
                 change:
                        time:   [−0.6030% −0.3726% −0.1548%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1550% +0.3740% +0.6067%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0095 s (1.9M iteraticipher_comparison/shared_pool/1024
                        time:   [2.6940 µs 2.6969 µs 2.7000 µs]
                        thrpt:  [361.69 MiB/s 362.11 MiB/s 362.50 MiB/s]
                 change:
                        time:   [−5.5328% −4.0534% −2.9931%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0855% +4.2247% +5.8568%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0019 s (1.8M iteracipher_comparison/fast_chacha20/1024
                        time:   [2.7191 µs 2.7211 µs 2.7235 µs]
                        thrpt:  [358.57 MiB/s 358.88 MiB/s 359.15 MiB/s]
                 change:
                        time:   [−0.5008% −0.3132% −0.1310%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1311% +0.3142% +0.5033%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0414 s (515k iteraticipher_comparison/shared_pool/4096
                        time:   [9.7761 µs 9.7835 µs 9.7914 µs]
                        thrpt:  [398.95 MiB/s 399.27 MiB/s 399.57 MiB/s]
                 change:
                        time:   [−0.9624% −0.5625% −0.1688%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1691% +0.5657% +0.9717%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0239 s (515k iteracipher_comparison/fast_chacha20/4096
                        time:   [9.7464 µs 9.7517 µs 9.7572 µs]
                        thrpt:  [400.34 MiB/s 400.57 MiB/s 400.79 MiB/s]
                 change:
                        time:   [−0.6453% −0.4138% −0.1385%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1387% +0.4155% +0.6495%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe

adaptive_batcher/optimal_size
                        time:   [976.29 ps 977.83 ps 980.10 ps]
                        thrpt:  [1.0203 Gelem/s 1.0227 Gelem/s 1.0243 Gelem/s]
                 change:
                        time:   [−0.3472% −0.1589% +0.0288%] (p = 0.12 > 0.05)
                        thrpt:  [−0.0288% +0.1592% +0.3484%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
adaptive_batcher/record time:   [3.8828 ns 3.8846 ns 3.8864 ns]
                        thrpt:  [257.30 Melem/s 257.42 Melem/s 257.55 Melem/s]
                 change:
                        time:   [−0.3705% −0.2267% −0.0843%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0844% +0.2272% +0.3719%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
adaptive_batcher/full_cycle
                        time:   [4.3992 ns 4.4019 ns 4.4047 ns]
                        thrpt:  [227.03 Melem/s 227.17 Melem/s 227.31 Melem/s]
                 change:
                        time:   [−0.5794% −0.3596% −0.1506%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1508% +0.3609% +0.5828%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0290 s (611k itee2e_packet_build/shared_pool_50_events
                        time:   [8.2158 µs 8.2247 µs 8.2343 µs]
                        thrpt:  [370.61 MiB/s 371.05 MiB/s 371.45 MiB/s]
                 change:
                        time:   [−0.7285% −0.4562% −0.1832%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1835% +0.4583% +0.7339%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0247 s (611k iterationse2e_packet_build/fast_50_events
                        time:   [8.2110 µs 8.2156 µs 8.2204 µs]
                        thrpt:  [371.24 MiB/s 371.46 MiB/s 371.67 MiB/s]
                 change:
                        time:   [−0.5043% −0.3573% −0.2074%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2079% +0.3586% +0.5068%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.3s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 9.3358 s (5050 itemultithread_packet_build/shared_pool/8
                        time:   [1.8439 ms 1.8479 ms 1.8519 ms]
                        thrpt:  [4.3199 Melem/s 4.3293 Melem/s 4.3385 Melem/s]
                 change:
                        time:   [−1.1181% −0.2489% +0.7766%] (p = 0.61 > 0.05)
                        thrpt:  [−0.7706% +0.2495% +1.1308%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 8.9994 s (10multithread_packet_build/thread_local_pool/8
                        time:   [887.06 µs 891.61 µs 896.38 µs]
                        thrpt:  [8.9248 Melem/s 8.9725 Melem/s 9.0185 Melem/s]
                 change:
                        time:   [+0.3606% +1.7419% +2.8795%] (p = 0.00 < 0.05)
                        thrpt:  [−2.7989% −1.7121% −0.3593%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.3754 s (1200 itmultithread_packet_build/shared_pool/16
                        time:   [4.4041 ms 4.4759 ms 4.5533 ms]
                        thrpt:  [3.5140 Melem/s 3.5747 Melem/s 3.6330 Melem/s]
                 change:
                        time:   [−11.785% −8.0425% −4.3705%] (p = 0.00 < 0.05)
                        thrpt:  [+4.5702% +8.7458% +13.359%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 8.6716 s (5multithread_packet_build/thread_local_pool/16
                        time:   [1.7160 ms 1.7187 ms 1.7214 ms]
                        thrpt:  [9.2946 Melem/s 9.3093 Melem/s 9.3239 Melem/s]
                 change:
                        time:   [−0.0898% +0.7623% +1.5481%] (p = 0.07 > 0.05)
                        thrpt:  [−1.5245% −0.7565% +0.0899%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.6406 s (800 itemultithread_packet_build/shared_pool/24
                        time:   [7.2675 ms 7.4488 ms 7.6464 ms]
                        thrpt:  [3.1387 Melem/s 3.2220 Melem/s 3.3024 Melem/s]
                 change:
                        time:   [−7.5560% −3.3430% +0.7444%] (p = 0.13 > 0.05)
                        thrpt:  [−0.7389% +3.4586% +8.1736%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.0267 s (1multithread_packet_build/thread_local_pool/24
                        time:   [2.7827 ms 2.7912 ms 2.7997 ms]
                        thrpt:  [8.5723 Melem/s 8.5985 Melem/s 8.6247 Melem/s]
                 change:
                        time:   [+9.3212% +10.089% +10.788%] (p = 0.00 < 0.05)
                        thrpt:  [−9.7377% −9.1640% −8.5265%]
                        Performance has regressed.
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.1512 s (500 itemultithread_packet_build/shared_pool/32
                        time:   [9.5837 ms 9.9502 ms 10.354 ms]
                        thrpt:  [3.0907 Melem/s 3.2160 Melem/s 3.3390 Melem/s]
                 change:
                        time:   [−12.037% −6.8027% −1.4199%] (p = 0.02 < 0.05)
                        thrpt:  [+1.4404% +7.2992% +13.684%]
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  10 (10.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.2804 s (1multithread_packet_build/thread_local_pool/32
                        time:   [3.2834 ms 3.2905 ms 3.2979 ms]
                        thrpt:  [9.7032 Melem/s 9.7249 Melem/s 9.7461 Melem/s]
                 change:
                        time:   [−0.9188% −0.6278% −0.3276%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3287% +0.6317% +0.9273%]
                        Change within noise threshold.

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 7.0876 s (5050 itmultithread_mixed_frames/shared_mixed/8
                        time:   [1.3985 ms 1.4045 ms 1.4108 ms]
                        thrpt:  [8.5061 Melem/s 8.5439 Melem/s 8.5803 Melem/s]
                 change:
                        time:   [−2.4446% −0.7763% +0.8399%] (p = 0.37 > 0.05)
                        thrpt:  [−0.8329% +0.7824% +2.5059%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.4s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 5.4414 s (5050 itermultithread_mixed_frames/fast_mixed/8
                        time:   [1.0757 ms 1.0831 ms 1.0925 ms]
                        thrpt:  [10.984 Melem/s 11.079 Melem/s 11.156 Melem/s]
                 change:
                        time:   [+0.3864% +1.6978% +2.9068%] (p = 0.01 < 0.05)
                        thrpt:  [−2.8247% −1.6695% −0.3849%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samples in estimated 5.1949 s (1700 imultithread_mixed_frames/shared_mixed/16
                        time:   [3.0026 ms 3.0489 ms 3.0998 ms]
                        thrpt:  [7.7425 Melem/s 7.8717 Melem/s 7.9930 Melem/s]
                 change:
                        time:   [−3.3912% −1.0322% +1.2973%] (p = 0.40 > 0.05)
                        thrpt:  [−1.2807% +1.0429% +3.5102%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples in estimated 5.1648 s (2500 itemultithread_mixed_frames/fast_mixed/16
                        time:   [2.0532 ms 2.0598 ms 2.0668 ms]
                        thrpt:  [11.612 Melem/s 11.651 Melem/s 11.689 Melem/s]
                 change:
                        time:   [−0.0880% +0.3230% +0.7328%] (p = 0.14 > 0.05)
                        thrpt:  [−0.7275% −0.3219% +0.0880%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samples in estimated 5.3093 s (1100 imultithread_mixed_frames/shared_mixed/24
                        time:   [4.5688 ms 4.7076 ms 4.8612 ms]
                        thrpt:  [7.4056 Melem/s 7.6471 Melem/s 7.8795 Melem/s]
                 change:
                        time:   [−4.7368% −0.8199% +3.9302%] (p = 0.71 > 0.05)
                        thrpt:  [−3.7816% +0.8267% +4.9724%]
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  9 (9.00%) high mild
  5 (5.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples in estimated 5.1529 s (1700 itemultithread_mixed_frames/fast_mixed/24
                        time:   [3.0096 ms 3.0188 ms 3.0282 ms]
                        thrpt:  [11.888 Melem/s 11.925 Melem/s 11.962 Melem/s]
                 change:
                        time:   [−0.3006% +0.1043% +0.4977%] (p = 0.63 > 0.05)
                        thrpt:  [−0.4952% −0.1042% +0.3015%]
                        No change in performance detected.
Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samples in estimated 5.2846 s (800 itmultithread_mixed_frames/shared_mixed/32
                        time:   [5.9663 ms 6.1237 ms 6.2958 ms]
                        thrpt:  [7.6241 Melem/s 7.8384 Melem/s 8.0452 Melem/s]
                 change:
                        time:   [−12.383% −7.4881% −2.5037%] (p = 0.01 < 0.05)
                        thrpt:  [+2.5680% +8.0942% +14.132%]
                        Performance has improved.
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples in estimated 5.1551 s (1300 itemultithread_mixed_frames/fast_mixed/32
                        time:   [3.9508 ms 3.9612 ms 3.9721 ms]
                        thrpt:  [12.084 Melem/s 12.117 Melem/s 12.149 Melem/s]
                 change:
                        time:   [−0.3841% −0.0430% +0.3412%] (p = 0.81 > 0.05)
                        thrpt:  [−0.3400% +0.0431% +0.3855%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samples in estimated 5.3161 s (300 itpool_contention/shared_acquire_release/8
                        time:   [17.651 ms 17.686 ms 17.721 ms]
                        thrpt:  [4.5143 Melem/s 4.5234 Melem/s 4.5323 Melem/s]
                 change:
                        time:   [+0.0674% +0.3473% +0.6352%] (p = 0.02 < 0.05)
                        thrpt:  [−0.6312% −0.3461% −0.0674%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.7s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples in estimated 5.6907 s (5050 itepool_contention/fast_acquire_release/8
                        time:   [1.1204 ms 1.1269 ms 1.1337 ms]
                        thrpt:  [70.567 Melem/s 70.989 Melem/s 71.401 Melem/s]
                 change:
                        time:   [+0.4925% +1.7307% +2.9665%] (p = 0.01 < 0.05)
                        thrpt:  [−2.8810% −1.7013% −0.4901%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samples in estimated 8.4736 s (200 ipool_contention/shared_acquire_release/16
                        time:   [41.308 ms 41.722 ms 42.151 ms]
                        thrpt:  [3.7959 Melem/s 3.8349 Melem/s 3.8734 Melem/s]
                 change:
                        time:   [−3.4856% −2.0812% −0.7571%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7629% +2.1254% +3.6114%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples in estimated 5.0694 s (2200 itpool_contention/fast_acquire_release/16
                        time:   [2.2923 ms 2.3029 ms 2.3137 ms]
                        thrpt:  [69.154 Melem/s 69.477 Melem/s 69.798 Melem/s]
                 change:
                        time:   [+0.1869% +0.8297% +1.5063%] (p = 0.02 < 0.05)
                        thrpt:  [−1.4840% −0.8229% −0.1865%]
                        Change within noise threshold.
Benchmarking pool_contention/shared_acquire_release/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.4s, or reduce sample count to 70.
Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samples in estimated 6.3881 s (100 ipool_contention/shared_acquire_release/24
                        time:   [61.050 ms 61.914 ms 62.831 ms]
                        thrpt:  [3.8198 Melem/s 3.8763 Melem/s 3.9312 Melem/s]
                 change:
                        time:   [−6.1666% −4.1441% −1.8628%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8981% +4.3232% +6.5718%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples in estimated 5.2888 s (1600 itpool_contention/fast_acquire_release/24
                        time:   [3.2951 ms 3.3096 ms 3.3249 ms]
                        thrpt:  [72.182 Melem/s 72.516 Melem/s 72.835 Melem/s]
                 change:
                        time:   [+0.2375% +0.7948% +1.3774%] (p = 0.01 < 0.05)
                        thrpt:  [−1.3586% −0.7886% −0.2369%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking pool_contention/shared_acquire_release/32: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.0s, or reduce sample count to 50.
Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samples in estimated 8.9606 s (100 ipool_contention/shared_acquire_release/32
                        time:   [86.021 ms 87.756 ms 89.664 ms]
                        thrpt:  [3.5689 Melem/s 3.6465 Melem/s 3.7200 Melem/s]
                 change:
                        time:   [+0.6069% +3.3080% +5.9959%] (p = 0.02 < 0.05)
                        thrpt:  [−5.6567% −3.2021% −0.6032%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples in estimated 5.0958 s (1200 itpool_contention/fast_acquire_release/32
                        time:   [4.1998 ms 4.2160 ms 4.2324 ms]
                        thrpt:  [75.606 Melem/s 75.901 Melem/s 76.194 Melem/s]
                 change:
                        time:   [−11.377% −10.928% −10.487%] (p = 0.00 < 0.05)
                        thrpt:  [+11.716% +12.269% +12.837%]
                        Performance has improved.

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples in estimated 5.6911 s (840 iterathroughput_scaling/fast_pool_scaling/1
                        time:   [6.7208 ms 6.7284 ms 6.7415 ms]
                        thrpt:  [296.67 Kelem/s 297.25 Kelem/s 297.58 Kelem/s]
                 change:
                        time:   [−2.1869% −1.9456% −1.6877%] (p = 0.00 < 0.05)
                        thrpt:  [+1.7167% +1.9842% +2.2358%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples in estimated 5.8777 s (840 iterathroughput_scaling/fast_pool_scaling/2
                        time:   [6.9861 ms 6.9914 ms 6.9975 ms]
                        thrpt:  [571.63 Kelem/s 572.13 Kelem/s 572.56 Kelem/s]
                 change:
                        time:   [−0.0544% +0.1533% +0.3645%] (p = 0.16 > 0.05)
                        thrpt:  [−0.3632% −0.1530% +0.0545%]
                        No change in performance detected.
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples in estimated 6.2343 s (840 iterathroughput_scaling/fast_pool_scaling/4
                        time:   [7.4240 ms 7.4301 ms 7.4367 ms]
                        thrpt:  [1.0757 Melem/s 1.0767 Melem/s 1.0776 Melem/s]
                 change:
                        time:   [+0.1178% +0.2508% +0.3841%] (p = 0.00 < 0.05)
                        thrpt:  [−0.3826% −0.2502% −0.1177%]
                        Change within noise threshold.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples in estimated 6.5945 s (840 iterathroughput_scaling/fast_pool_scaling/8
                        time:   [7.8322 ms 7.8672 ms 7.9050 ms]
                        thrpt:  [2.0240 Melem/s 2.0338 Melem/s 2.0429 Melem/s]
                 change:
                        time:   [−0.3027% +0.7795% +1.7518%] (p = 0.16 > 0.05)
                        thrpt:  [−1.7216% −0.7735% +0.3036%]
                        No change in performance detected.
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples in estimated 6.5660 s (420 iterthroughput_scaling/fast_pool_scaling/16
                        time:   [15.513 ms 15.564 ms 15.615 ms]
                        thrpt:  [2.0493 Melem/s 2.0561 Melem/s 2.0628 Melem/s]
                 change:
                        time:   [−1.1731% −0.0254% +0.9399%] (p = 0.97 > 0.05)
                        thrpt:  [−0.9312% +0.0254% +1.1870%]
                        No change in performance detected.
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples in estimated 9.6616 s (420 iterthroughput_scaling/fast_pool_scaling/24
                        time:   [23.032 ms 23.101 ms 23.170 ms]
                        thrpt:  [2.0716 Melem/s 2.0779 Melem/s 2.0840 Melem/s]
                 change:
                        time:   [−1.5976% −0.3539% +0.6856%] (p = 0.58 > 0.05)
                        thrpt:  [−0.6810% +0.3551% +1.6236%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking throughput_scaling/fast_pool_scaling/32: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.3s, enable flat sampling, or reduce sample count to 10.
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples in estimated 6.3006 s (210 iterthroughput_scaling/fast_pool_scaling/32
                        time:   [29.900 ms 29.997 ms 30.096 ms]
                        thrpt:  [2.1265 Melem/s 2.1335 Melem/s 2.1405 Melem/s]
                 change:
                        time:   [−1.1611% −0.4091% +0.2993%] (p = 0.30 > 0.05)
                        thrpt:  [−0.2984% +0.4108% +1.1747%]
                        No change in performance detected.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  2 (10.00%) high mild

routing_header/serialize
                        time:   [627.63 ps 628.55 ps 629.72 ps]
                        thrpt:  [1.5880 Gelem/s 1.5910 Gelem/s 1.5933 Gelem/s]
                 change:
                        time:   [−0.8841% −0.5873% −0.2840%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2848% +0.5907% +0.8920%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
routing_header/deserialize
                        time:   [937.67 ps 941.97 ps 950.87 ps]
                        thrpt:  [1.0517 Gelem/s 1.0616 Gelem/s 1.0665 Gelem/s]
                 change:
                        time:   [−0.2401% +0.3564% +1.3809%] (p = 0.53 > 0.05)
                        thrpt:  [−1.3621% −0.3551% +0.2407%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
routing_header/roundtrip
                        time:   [937.05 ps 938.40 ps 940.49 ps]
                        thrpt:  [1.0633 Gelem/s 1.0656 Gelem/s 1.0672 Gelem/s]
                 change:
                        time:   [−0.1542% +0.0566% +0.3017%] (p = 0.69 > 0.05)
                        thrpt:  [−0.3008% −0.0566% +0.1545%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
routing_header/forward  time:   [572.43 ps 573.68 ps 574.96 ps]
                        thrpt:  [1.7392 Gelem/s 1.7431 Gelem/s 1.7469 Gelem/s]
                 change:
                        time:   [−0.4939% −0.0502% +0.3618%] (p = 0.81 > 0.05)
                        thrpt:  [−0.3605% +0.0502% +0.4964%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild

routing_table/lookup_hit
                        time:   [37.251 ns 37.772 ns 38.321 ns]
                        thrpt:  [26.095 Melem/s 26.474 Melem/s 26.845 Melem/s]
                 change:
                        time:   [−2.5958% −0.5396% +1.5175%] (p = 0.61 > 0.05)
                        thrpt:  [−1.4948% +0.5425% +2.6650%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
routing_table/lookup_miss
                        time:   [14.827 ns 14.917 ns 14.997 ns]
                        thrpt:  [66.679 Melem/s 67.036 Melem/s 67.443 Melem/s]
                 change:
                        time:   [−0.2537% +0.4960% +1.2860%] (p = 0.21 > 0.05)
                        thrpt:  [−1.2696% −0.4936% +0.2544%]
                        No change in performance detected.
Found 17 outliers among 100 measurements (17.00%)
  7 (7.00%) low severe
  8 (8.00%) low mild
  2 (2.00%) high severe
routing_table/is_local  time:   [313.99 ps 314.83 ps 315.84 ps]
                        thrpt:  [3.1662 Gelem/s 3.1763 Gelem/s 3.1848 Gelem/s]
                 change:
                        time:   [−0.2401% +0.0208% +0.2854%] (p = 0.88 > 0.05)
                        thrpt:  [−0.2846% −0.0208% +0.2407%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
routing_table/add_route time:   [237.68 ns 242.95 ns 247.50 ns]
                        thrpt:  [4.0403 Melem/s 4.1160 Melem/s 4.2073 Melem/s]
                 change:
                        time:   [−7.4885% −2.5157% +2.8329%] (p = 0.33 > 0.05)
                        thrpt:  [−2.7549% +2.5806% +8.0947%]
                        No change in performance detected.
routing_table/record_in time:   [49.391 ns 50.047 ns 50.655 ns]
                        thrpt:  [19.742 Melem/s 19.981 Melem/s 20.246 Melem/s]
                 change:
                        time:   [−0.8790% +0.9345% +2.7648%] (p = 0.33 > 0.05)
                        thrpt:  [−2.6904% −0.9259% +0.8868%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) low mild
routing_table/record_out
                        time:   [19.735 ns 20.175 ns 20.619 ns]
                        thrpt:  [48.499 Melem/s 49.566 Melem/s 50.672 Melem/s]
                 change:
                        time:   [+6.2238% +9.8301% +13.274%] (p = 0.00 < 0.05)
                        thrpt:  [−11.719% −8.9503% −5.8591%]
                        Performance has regressed.
routing_table/aggregate_stats
                        time:   [2.0758 µs 2.0789 µs 2.0827 µs]
                        thrpt:  [480.14 Kelem/s 481.03 Kelem/s 481.73 Kelem/s]
                 change:
                        time:   [−0.4031% −0.1524% +0.1015%] (p = 0.24 > 0.05)
                        thrpt:  [−0.1014% +0.1526% +0.4047%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe

fair_scheduler/creation time:   [286.70 ns 286.97 ns 287.23 ns]
                        thrpt:  [3.4815 Melem/s 3.4847 Melem/s 3.4879 Melem/s]
                 change:
                        time:   [−0.9286% −0.6171% −0.3455%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3467% +0.6209% +0.9373%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estimated 5.0001 s (25M iterationfair_scheduler/stream_count_empty
                        time:   [200.53 ns 200.63 ns 200.74 ns]
                        thrpt:  [4.9815 Melem/s 4.9842 Melem/s 4.9867 Melem/s]
                 change:
                        time:   [−0.1260% +0.0194% +0.1554%] (p = 0.79 > 0.05)
                        thrpt:  [−0.1552% −0.0194% +0.1261%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
fair_scheduler/total_queued
                        time:   [312.46 ps 312.66 ps 312.86 ps]
                        thrpt:  [3.1963 Gelem/s 3.1984 Gelem/s 3.2004 Gelem/s]
                 change:
                        time:   [−0.2703% −0.0608% +0.1236%] (p = 0.55 > 0.05)
                        thrpt:  [−0.1235% +0.0608% +0.2711%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
fair_scheduler/cleanup_empty
                        time:   [201.46 ns 201.57 ns 201.68 ns]
                        thrpt:  [4.9584 Melem/s 4.9612 Melem/s 4.9639 Melem/s]
                 change:
                        time:   [−0.2437% +0.0496% +0.4206%] (p = 0.78 > 0.05)
                        thrpt:  [−0.4189% −0.0495% +0.2443%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samples in estimated 5.6840 s (35routing_table_concurrent/concurrent_lookup/4
                        time:   [155.68 µs 159.50 µs 163.01 µs]
                        thrpt:  [24.538 Melem/s 25.078 Melem/s 25.693 Melem/s]
                 change:
                        time:   [−3.5304% −1.7619% −0.0078%] (p = 0.06 > 0.05)
                        thrpt:  [+0.0078% +1.7935% +3.6596%]
                        No change in performance detected.
Found 17 outliers among 100 measurements (17.00%)
  10 (10.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samples in estimated 5.9397 s (20krouting_table_concurrent/concurrent_stats/4
                        time:   [293.33 µs 293.89 µs 294.44 µs]
                        thrpt:  [13.585 Melem/s 13.611 Melem/s 13.636 Melem/s]
                 change:
                        time:   [+4.2578% +4.6185% +4.9688%] (p = 0.00 < 0.05)
                        thrpt:  [−4.7336% −4.4146% −4.0839%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samples in estimated 6.2088 s (25routing_table_concurrent/concurrent_lookup/8
                        time:   [243.65 µs 245.30 µs 246.47 µs]
                        thrpt:  [32.458 Melem/s 32.613 Melem/s 32.834 Melem/s]
                 change:
                        time:   [−1.9401% −1.2416% −0.6086%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6123% +1.2572% +1.9784%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  7 (7.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samples in estimated 6.3290 s (15krouting_table_concurrent/concurrent_stats/8
                        time:   [417.10 µs 418.22 µs 419.38 µs]
                        thrpt:  [19.076 Melem/s 19.129 Melem/s 19.180 Melem/s]
                 change:
                        time:   [+0.4503% +1.7999% +2.8940%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8126% −1.7681% −0.4482%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 samples in estimated 6.4840 s (1routing_table_concurrent/concurrent_lookup/16
                        time:   [427.49 µs 428.47 µs 429.31 µs]
                        thrpt:  [37.269 Melem/s 37.342 Melem/s 37.428 Melem/s]
                 change:
                        time:   [+0.0453% +0.6284% +1.1158%] (p = 0.02 < 0.05)
                        thrpt:  [−1.1035% −0.6245% −0.0453%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low severe
  3 (3.00%) high mild
Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samples in estimated 8.3467 s (10routing_table_concurrent/concurrent_stats/16
                        time:   [824.61 µs 826.00 µs 827.57 µs]
                        thrpt:  [19.334 Melem/s 19.371 Melem/s 19.403 Melem/s]
                 change:
                        time:   [+2.5333% +2.9407% +3.3490%] (p = 0.00 < 0.05)
                        thrpt:  [−3.2405% −2.8566% −2.4707%]
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples in estimated 5.0000 s (133M iterrouting_decision/parse_lookup_forward
                        time:   [39.070 ns 39.590 ns 40.102 ns]
                        thrpt:  [24.936 Melem/s 25.259 Melem/s 25.595 Melem/s]
                 change:
                        time:   [−0.2655% +1.1495% +2.4199%] (p = 0.11 > 0.05)
                        thrpt:  [−2.3627% −1.1365% +0.2662%]
                        No change in performance detected.
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estimated 5.0005 s (45M iterationsrouting_decision/full_with_stats
                        time:   [109.79 ns 109.97 ns 110.20 ns]
                        thrpt:  [9.0744 Melem/s 9.0935 Melem/s 9.1087 Melem/s]
                 change:
                        time:   [−0.2631% +0.0748% +0.3649%] (p = 0.66 > 0.05)
                        thrpt:  [−0.3636% −0.0748% +0.2638%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in estimated 5.0007 s (17M iterationstream_multiplexing/lookup_all/10
                        time:   [292.37 ns 292.53 ns 292.69 ns]
                        thrpt:  [34.166 Melem/s 34.184 Melem/s 34.203 Melem/s]
                 change:
                        time:   [−0.5082% −0.1545% +0.1086%] (p = 0.38 > 0.05)
                        thrpt:  [−0.1085% +0.1548% +0.5108%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estimated 5.0023 s (10M iterationsstream_multiplexing/stats_all/10
                        time:   [481.79 ns 487.82 ns 493.67 ns]
                        thrpt:  [20.256 Melem/s 20.500 Melem/s 20.756 Melem/s]
                 change:
                        time:   [+1.1646% +2.5919% +4.1054%] (p = 0.00 < 0.05)
                        thrpt:  [−3.9435% −2.5264% −1.1512%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in estimated 5.0077 s (1.7M iteratistream_multiplexing/lookup_all/100
                        time:   [2.9241 µs 2.9266 µs 2.9292 µs]
                        thrpt:  [34.139 Melem/s 34.170 Melem/s 34.199 Melem/s]
                 change:
                        time:   [−0.0770% +0.0802% +0.2347%] (p = 0.33 > 0.05)
                        thrpt:  [−0.2342% −0.0802% +0.0770%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in estimated 5.0009 s (1.0M iteratiostream_multiplexing/stats_all/100
                        time:   [4.9061 µs 4.9594 µs 5.0097 µs]
                        thrpt:  [19.961 Melem/s 20.164 Melem/s 20.383 Melem/s]
                 change:
                        time:   [−0.6209% +1.1619% +2.7986%] (p = 0.17 > 0.05)
                        thrpt:  [−2.7225% −1.1485% +0.6248%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low severe
  6 (6.00%) low mild
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estimated 5.1467 s (172k iteratstream_multiplexing/lookup_all/1000
                        time:   [29.956 µs 29.979 µs 30.002 µs]
                        thrpt:  [33.331 Melem/s 33.357 Melem/s 33.383 Melem/s]
                 change:
                        time:   [+2.1477% +2.3667% +2.5585%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4947% −2.3120% −2.1026%]
                        Performance has regressed.
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in estimated 5.1471 s (96k iteratiostream_multiplexing/stats_all/1000
                        time:   [53.505 µs 53.619 µs 53.746 µs]
                        thrpt:  [18.606 Melem/s 18.650 Melem/s 18.690 Melem/s]
                 change:
                        time:   [+1.4209% +1.7614% +2.0679%] (p = 0.00 < 0.05)
                        thrpt:  [−2.0260% −1.7309% −1.4010%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in estimated 5.9770 s (20k iteratstream_multiplexing/lookup_all/10000
                        time:   [293.08 µs 293.32 µs 293.57 µs]
                        thrpt:  [34.064 Melem/s 34.093 Melem/s 34.120 Melem/s]
                 change:
                        time:   [−0.2673% −0.0579% +0.1561%] (p = 0.60 > 0.05)
                        thrpt:  [−0.1558% +0.0579% +0.2680%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estimated 5.7497 s (10k iteratistream_multiplexing/stats_all/10000
                        time:   [568.10 µs 569.72 µs 571.47 µs]
                        thrpt:  [17.499 Melem/s 17.553 Melem/s 17.603 Melem/s]
                 change:
                        time:   [−0.1817% +0.5076% +1.4433%] (p = 0.23 > 0.05)
                        thrpt:  [−1.4228% −0.5050% +0.1820%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimated 5.0001 s (212M iterationmultihop_packet_builder/build/64
                        time:   [23.208 ns 23.306 ns 23.412 ns]
                        thrpt:  [2.5459 GiB/s 2.5574 GiB/s 2.5682 GiB/s]
                 change:
                        time:   [−1.6291% −0.6088% +0.2526%] (p = 0.23 > 0.05)
                        thrpt:  [−0.2520% +0.6126% +1.6561%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samples in estimated 5.0000 s (237M multihop_packet_builder/build_priority/64
                        time:   [21.165 ns 21.188 ns 21.211 ns]
                        thrpt:  [2.8101 GiB/s 2.8132 GiB/s 2.8162 GiB/s]
                 change:
                        time:   [+0.6630% +1.5554% +2.2414%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1923% −1.5315% −0.6587%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estimated 5.0000 s (92M iterationmultihop_packet_builder/build/256
                        time:   [54.398 ns 54.754 ns 55.104 ns]
                        thrpt:  [4.3267 GiB/s 4.3544 GiB/s 4.3828 GiB/s]
                 change:
                        time:   [+4.9178% +5.7113% +6.4732%] (p = 0.00 < 0.05)
                        thrpt:  [−6.0797% −5.4027% −4.6873%]
                        Performance has regressed.
Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samples in estimated 5.0000 s (95M multihop_packet_builder/build_priority/256
                        time:   [51.642 ns 52.233 ns 52.792 ns]
                        thrpt:  [4.5162 GiB/s 4.5645 GiB/s 4.6167 GiB/s]
                 change:
                        time:   [+4.8454% +6.0615% +7.2050%] (p = 0.00 < 0.05)
                        thrpt:  [−6.7208% −5.7151% −4.6215%]
                        Performance has regressed.
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in estimated 5.0001 s (123M iteratimultihop_packet_builder/build/1024
                        time:   [42.266 ns 42.852 ns 44.085 ns]
                        thrpt:  [21.633 GiB/s 22.255 GiB/s 22.564 GiB/s]
                 change:
                        time:   [+3.1394% +15.057% +48.168%] (p = 0.18 > 0.05)
                        thrpt:  [−32.509% −13.087% −3.0438%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 samples in estimated 5.0001 s (127multihop_packet_builder/build_priority/1024
                        time:   [39.252 ns 39.308 ns 39.401 ns]
                        thrpt:  [24.204 GiB/s 24.262 GiB/s 24.296 GiB/s]
                 change:
                        time:   [+2.2854% +2.4804% +2.6877%] (p = 0.00 < 0.05)
                        thrpt:  [−2.6173% −2.4204% −2.2343%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  7 (7.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in estimated 5.0000 s (58M iteratiomultihop_packet_builder/build/4096
                        time:   [82.256 ns 82.808 ns 83.496 ns]
                        thrpt:  [45.687 GiB/s 46.067 GiB/s 46.376 GiB/s]
                 change:
                        time:   [−12.223% −9.9659% −7.4634%] (p = 0.00 < 0.05)
                        thrpt:  [+8.0654% +11.069% +13.926%]
                        Performance has improved.
Found 18 outliers among 100 measurements (18.00%)
  3 (3.00%) high mild
  15 (15.00%) high severe
Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 samples in estimated 5.0002 s (62Mmultihop_packet_builder/build_priority/4096
                        time:   [80.548 ns 80.646 ns 80.750 ns]
                        thrpt:  [47.241 GiB/s 47.302 GiB/s 47.359 GiB/s]
                 change:
                        time:   [−14.044% −13.050% −12.138%] (p = 0.00 < 0.05)
                        thrpt:  [+13.815% +15.009% +16.338%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

multihop_chain/forward_chain/1
                        time:   [59.158 ns 59.971 ns 60.849 ns]
                        thrpt:  [16.434 Melem/s 16.675 Melem/s 16.904 Melem/s]
                 change:
                        time:   [−1.0974% −0.0536% +1.0078%] (p = 0.92 > 0.05)
                        thrpt:  [−0.9977% +0.0536% +1.1096%]
                        No change in performance detected.
multihop_chain/forward_chain/2
                        time:   [113.88 ns 114.06 ns 114.23 ns]
                        thrpt:  [8.7541 Melem/s 8.7677 Melem/s 8.7812 Melem/s]
                 change:
                        time:   [−1.6581% −1.3858% −1.1273%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1401% +1.4053% +1.6860%]
                        Performance has improved.
multihop_chain/forward_chain/3
                        time:   [159.55 ns 159.79 ns 160.02 ns]
                        thrpt:  [6.2493 Melem/s 6.2582 Melem/s 6.2675 Melem/s]
                 change:
                        time:   [−2.8087% −2.2961% −1.8020%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8351% +2.3501% +2.8899%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
multihop_chain/forward_chain/4
                        time:   [220.10 ns 238.17 ns 276.64 ns]
                        thrpt:  [3.6149 Melem/s 4.1987 Melem/s 4.5434 Melem/s]
                 change:
                        time:   [−2.1353% +3.1948% +13.250%] (p = 0.68 > 0.05)
                        thrpt:  [−11.700% −3.0959% +2.1819%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
multihop_chain/forward_chain/5
                        time:   [275.44 ns 275.93 ns 276.47 ns]
                        thrpt:  [3.6170 Melem/s 3.6241 Melem/s 3.6305 Melem/s]
                 change:
                        time:   [−9.5669% −4.4671% −1.4329%] (p = 0.05 < 0.05)
                        thrpt:  [+1.4538% +4.6760% +10.579%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

hop_latency/single_hop_process
                        time:   [1.4886 ns 1.4891 ns 1.4896 ns]
                        thrpt:  [671.31 Melem/s 671.54 Melem/s 671.76 Melem/s]
                 change:
                        time:   [−0.3037% −0.2162% −0.1425%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1427% +0.2166% +0.3046%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  3 (3.00%) high mild
  3 (3.00%) high severe
hop_latency/single_hop_full
                        time:   [57.991 ns 59.194 ns 60.469 ns]
                        thrpt:  [16.538 Melem/s 16.894 Melem/s 17.244 Melem/s]
                 change:
                        time:   [+9.6547% +11.398% +13.191%] (p = 0.00 < 0.05)
                        thrpt:  [−11.654% −10.232% −8.8046%]
                        Performance has regressed.

hop_scaling/64B_1hops   time:   [29.558 ns 29.600 ns 29.647 ns]
                        thrpt:  [2.0105 GiB/s 2.0137 GiB/s 2.0165 GiB/s]
                 change:
                        time:   [−0.2807% −0.0804% +0.1501%] (p = 0.45 > 0.05)
                        thrpt:  [−0.1499% +0.0805% +0.2815%]
                        No change in performance detected.
hop_scaling/64B_2hops   time:   [52.557 ns 52.720 ns 52.909 ns]
                        thrpt:  [1.1266 GiB/s 1.1306 GiB/s 1.1341 GiB/s]
                 change:
                        time:   [+0.2030% +0.4781% +0.7600%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7543% −0.4758% −0.2026%]
                        Change within noise threshold.
Found 17 outliers among 100 measurements (17.00%)
  8 (8.00%) high mild
  9 (9.00%) high severe
hop_scaling/64B_3hops   time:   [76.347 ns 76.539 ns 76.726 ns]
                        thrpt:  [795.49 MiB/s 797.44 MiB/s 799.44 MiB/s]
                 change:
                        time:   [−0.8574% −0.5247% −0.2169%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2173% +0.5274% +0.8648%]
                        Change within noise threshold.
hop_scaling/64B_4hops   time:   [99.275 ns 99.501 ns 99.731 ns]
                        thrpt:  [612.00 MiB/s 613.42 MiB/s 614.81 MiB/s]
                 change:
                        time:   [−0.9117% −0.6353% −0.3590%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3603% +0.6394% +0.9200%]
                        Change within noise threshold.
Found 31 outliers among 100 measurements (31.00%)
  12 (12.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  12 (12.00%) high severe
hop_scaling/64B_5hops   time:   [129.84 ns 130.29 ns 130.70 ns]
                        thrpt:  [466.99 MiB/s 468.47 MiB/s 470.09 MiB/s]
                 change:
                        time:   [−1.9459% −1.3306% −0.7335%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7389% +1.3485% +1.9845%]
                        Change within noise threshold.
hop_scaling/256B_1hops  time:   [61.444 ns 62.276 ns 63.048 ns]
                        thrpt:  [3.7815 GiB/s 3.8284 GiB/s 3.8802 GiB/s]
                 change:
                        time:   [+5.5959% +7.0782% +8.5244%] (p = 0.00 < 0.05)
                        thrpt:  [−7.8548% −6.6103% −5.2994%]
                        Performance has regressed.
hop_scaling/256B_2hops  time:   [119.07 ns 119.58 ns 120.07 ns]
                        thrpt:  [1.9857 GiB/s 1.9937 GiB/s 2.0023 GiB/s]
                 change:
                        time:   [−2.4373% −1.4161% −0.3425%] (p = 0.01 < 0.05)
                        thrpt:  [+0.3437% +1.4364% +2.4982%]
                        Change within noise threshold.
hop_scaling/256B_3hops  time:   [171.00 ns 173.04 ns 175.23 ns]
                        thrpt:  [1.3606 GiB/s 1.3778 GiB/s 1.3943 GiB/s]
                 change:
                        time:   [+3.3366% +4.9854% +6.5516%] (p = 0.00 < 0.05)
                        thrpt:  [−6.1488% −4.7487% −3.2289%]
                        Performance has regressed.
hop_scaling/256B_4hops  time:   [222.68 ns 225.45 ns 228.01 ns]
                        thrpt:  [1.0456 GiB/s 1.0575 GiB/s 1.0707 GiB/s]
                 change:
                        time:   [−5.8700% −4.9201% −3.9507%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1132% +5.1747% +6.2361%]
                        Performance has improved.
Found 20 outliers among 100 measurements (20.00%)
  8 (8.00%) high mild
  12 (12.00%) high severe
hop_scaling/256B_5hops  time:   [284.82 ns 288.26 ns 291.90 ns]
                        thrpt:  [836.40 MiB/s 846.96 MiB/s 857.17 MiB/s]
                 change:
                        time:   [+1.7351% +3.0267% +4.3129%] (p = 0.00 < 0.05)
                        thrpt:  [−4.1346% −2.9378% −1.7055%]
                        Performance has regressed.
hop_scaling/1024B_1hops time:   [47.933 ns 48.009 ns 48.110 ns]
                        thrpt:  [19.823 GiB/s 19.864 GiB/s 19.896 GiB/s]
                 change:
                        time:   [−3.2063% −3.0085% −2.8243%] (p = 0.00 < 0.05)
                        thrpt:  [+2.9064% +3.1018% +3.3125%]
                        Performance has improved.
Found 20 outliers among 100 measurements (20.00%)
  10 (10.00%) low severe
  1 (1.00%) low mild
  1 (1.00%) high mild
  8 (8.00%) high severe
hop_scaling/1024B_2hops time:   [112.54 ns 112.77 ns 113.02 ns]
                        thrpt:  [8.4382 GiB/s 8.4571 GiB/s 8.4745 GiB/s]
                 change:
                        time:   [−5.7091% −4.2955% −3.1730%] (p = 0.00 < 0.05)
                        thrpt:  [+3.2770% +4.4883% +6.0547%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe
hop_scaling/1024B_3hops time:   [161.26 ns 162.73 ns 164.03 ns]
                        thrpt:  [5.8142 GiB/s 5.8604 GiB/s 5.9140 GiB/s]
                 change:
                        time:   [+4.2053% +5.2392% +6.2612%] (p = 0.00 < 0.05)
                        thrpt:  [−5.8923% −4.9784% −4.0356%]
                        Performance has regressed.
Found 21 outliers among 100 measurements (21.00%)
  18 (18.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high severe
hop_scaling/1024B_4hops time:   [220.00 ns 239.02 ns 280.02 ns]
                        thrpt:  [3.4058 GiB/s 3.9899 GiB/s 4.3349 GiB/s]
                 change:
                        time:   [+1.3148% +6.7766% +17.101%] (p = 0.17 > 0.05)
                        thrpt:  [−14.603% −6.3465% −1.2977%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
hop_scaling/1024B_5hops time:   [261.94 ns 264.29 ns 266.88 ns]
                        thrpt:  [3.5735 GiB/s 3.6085 GiB/s 3.6408 GiB/s]
                 change:
                        time:   [+1.0038% +1.8728% +2.7165%] (p = 0.00 < 0.05)
                        thrpt:  [−2.6447% −1.8384% −0.9939%]
                        Performance has regressed.
Found 27 outliers among 100 measurements (27.00%)
  3 (3.00%) low severe
  8 (8.00%) low mild
  16 (16.00%) high mild

Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samples in estimated 5.0004 s (27M imultihop_with_routing/route_and_forward/1
                        time:   [182.23 ns 183.21 ns 184.29 ns]
                        thrpt:  [5.4263 Melem/s 5.4583 Melem/s 5.4877 Melem/s]
                 change:
                        time:   [+3.9037% +4.5115% +5.0904%] (p = 0.00 < 0.05)
                        thrpt:  [−4.8438% −4.3167% −3.7570%]
                        Performance has regressed.
Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samples in estimated 5.0002 s (14M imultihop_with_routing/route_and_forward/2
                        time:   [348.41 ns 349.64 ns 350.79 ns]
                        thrpt:  [2.8507 Melem/s 2.8601 Melem/s 2.8702 Melem/s]
                 change:
                        time:   [−0.7174% −0.3617% +0.0357%] (p = 0.06 > 0.05)
                        thrpt:  [−0.0357% +0.3630% +0.7225%]
                        No change in performance detected.
Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samples in estimated 5.0021 s (9.5M multihop_with_routing/route_and_forward/3
                        time:   [516.10 ns 518.40 ns 521.13 ns]
                        thrpt:  [1.9189 Melem/s 1.9290 Melem/s 1.9376 Melem/s]
                 change:
                        time:   [−1.8045% −1.2578% −0.7132%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7184% +1.2738% +1.8376%]
                        Change within noise threshold.
Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samples in estimated 5.0026 s (7.1M multihop_with_routing/route_and_forward/4
                        time:   [701.77 ns 703.95 ns 706.50 ns]
                        thrpt:  [1.4154 Melem/s 1.4206 Melem/s 1.4250 Melem/s]
                 change:
                        time:   [−0.6066% −0.2619% +0.0964%] (p = 0.17 > 0.05)
                        thrpt:  [−0.0963% +0.2626% +0.6103%]
                        No change in performance detected.
Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samples in estimated 5.0006 s (5.7M multihop_with_routing/route_and_forward/5
                        time:   [879.66 ns 884.50 ns 889.44 ns]
                        thrpt:  [1.1243 Melem/s 1.1306 Melem/s 1.1368 Melem/s]
                 change:
                        time:   [−22.553% −10.253% −2.1292%] (p = 0.17 > 0.05)
                        thrpt:  [+2.1755% +11.425% +29.120%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples in estimated 5.0492 s (7350 itmultihop_concurrent/concurrent_forward/4
                        time:   [685.49 µs 687.43 µs 689.44 µs]
                        thrpt:  [5.8018 Melem/s 5.8187 Melem/s 5.8352 Melem/s]
                 change:
                        time:   [−10.273% −7.2655% −4.8870%] (p = 0.00 < 0.05)
                        thrpt:  [+5.1381% +7.8348% +11.450%]
                        Performance has improved.
Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples in estimated 5.1788 s (4620 itmultihop_concurrent/concurrent_forward/8
                        time:   [953.95 µs 983.59 µs 1.0113 ms]
                        thrpt:  [7.9103 Melem/s 8.1335 Melem/s 8.3862 Melem/s]
                 change:
                        time:   [−21.035% −17.160% −12.072%] (p = 0.00 < 0.05)
                        thrpt:  [+13.729% +20.715% +26.639%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samples in estimated 5.3544 s (2520 imultihop_concurrent/concurrent_forward/16
                        time:   [2.0513 ms 2.0707 ms 2.0944 ms]
                        thrpt:  [7.6393 Melem/s 7.7270 Melem/s 7.7999 Melem/s]
                 change:
                        time:   [−8.5606% −7.3644% −6.2534%] (p = 0.00 < 0.05)
                        thrpt:  [+6.6705% +7.9498% +9.3620%]
                        Performance has improved.

pingwave/serialize      time:   [786.22 ps 787.06 ps 787.93 ps]
                        thrpt:  [1.2691 Gelem/s 1.2706 Gelem/s 1.2719 Gelem/s]
                 change:
                        time:   [+0.6530% +0.9483% +1.2082%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1938% −0.9393% −0.6488%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
pingwave/deserialize    time:   [953.00 ps 954.16 ps 955.37 ps]
                        thrpt:  [1.0467 Gelem/s 1.0480 Gelem/s 1.0493 Gelem/s]
                 change:
                        time:   [+2.2142% +2.3368% +2.4966%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4358% −2.2834% −2.1662%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
pingwave/roundtrip      time:   [954.35 ps 956.03 ps 957.71 ps]
                        thrpt:  [1.0442 Gelem/s 1.0460 Gelem/s 1.0478 Gelem/s]
                 change:
                        time:   [+2.4295% +2.5621% +2.7001%] (p = 0.00 < 0.05)
                        thrpt:  [−2.6291% −2.4981% −2.3718%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
pingwave/forward        time:   [632.87 ps 634.22 ps 635.69 ps]
                        thrpt:  [1.5731 Gelem/s 1.5767 Gelem/s 1.5801 Gelem/s]
                 change:
                        time:   [+1.2507% +1.5324% +1.8166%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7842% −1.5093% −1.2353%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

capabilities/serialize_simple
                        time:   [19.151 ns 19.169 ns 19.187 ns]
                        thrpt:  [52.118 Melem/s 52.167 Melem/s 52.217 Melem/s]
                 change:
                        time:   [−2.5936% −2.4401% −2.2764%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3294% +2.5011% +2.6626%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimated 5.0000 s (1.0B iterationscapabilities/deserialize_simple
                        time:   [4.7486 ns 4.7556 ns 4.7626 ns]
                        thrpt:  [209.97 Melem/s 210.28 Melem/s 210.59 Melem/s]
                 change:
                        time:   [−2.7168% −2.5315% −2.3253%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3807% +2.5973% +2.7927%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
capabilities/serialize_complex
                        time:   [41.187 ns 41.215 ns 41.244 ns]
                        thrpt:  [24.246 Melem/s 24.263 Melem/s 24.280 Melem/s]
                 change:
                        time:   [+0.4675% +0.5664% +0.6703%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6658% −0.5632% −0.4653%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in estimated 5.0001 s (33M iterationscapabilities/deserialize_complex
                        time:   [153.65 ns 153.80 ns 153.96 ns]
                        thrpt:  [6.4953 Melem/s 6.5018 Melem/s 6.5082 Melem/s]
                 change:
                        time:   [+0.3405% +0.5009% +0.6475%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6433% −0.4984% −0.3394%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

local_graph/create_pingwave
                        time:   [2.1043 ns 2.1087 ns 2.1132 ns]
                        thrpt:  [473.22 Melem/s 474.23 Melem/s 475.23 Melem/s]
                 change:
                        time:   [+0.0108% +0.3890% +0.7602%] (p = 0.04 < 0.05)
                        thrpt:  [−0.7545% −0.3875% −0.0108%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
local_graph/on_pingwave_new
                        time:   [119.78 ns 122.80 ns 125.59 ns]
                        thrpt:  [7.9622 Melem/s 8.1433 Melem/s 8.3489 Melem/s]
                 change:
                        time:   [+4.3925% +9.0382% +13.867%] (p = 0.00 < 0.05)
                        thrpt:  [−12.178% −8.2890% −4.2077%]
                        Performance has regressed.
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estimated 5.0000 s (145M iteratiolocal_graph/on_pingwave_duplicate
                        time:   [34.281 ns 34.336 ns 34.396 ns]
                        thrpt:  [29.073 Melem/s 29.124 Melem/s 29.170 Melem/s]
                 change:
                        time:   [+3.1337% +3.3864% +3.6277%] (p = 0.00 < 0.05)
                        thrpt:  [−3.5007% −3.2755% −3.0385%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
local_graph/get_node    time:   [26.037 ns 26.057 ns 26.079 ns]
                        thrpt:  [38.345 Melem/s 38.377 Melem/s 38.407 Melem/s]
                 change:
                        time:   [−18.977% −7.5230% −0.3570%] (p = 0.21 > 0.05)
                        thrpt:  [+0.3583% +8.1350% +23.422%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
local_graph/node_count  time:   [200.34 ns 200.50 ns 200.66 ns]
                        thrpt:  [4.9835 Melem/s 4.9876 Melem/s 4.9915 Melem/s]
                 change:
                        time:   [−4.6362% −3.5620% −2.7333%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8102% +3.6936% +4.8616%]
                        Performance has improved.
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
local_graph/stats       time:   [600.23 ns 600.73 ns 601.24 ns]
                        thrpt:  [1.6632 Melem/s 1.6647 Melem/s 1.6660 Melem/s]
                 change:
                        time:   [−2.6044% −2.4585% −2.3136%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3684% +2.5205% +2.6740%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

graph_scaling/all_nodes/100
                        time:   [2.4431 µs 2.4481 µs 2.4532 µs]
                        thrpt:  [40.763 Melem/s 40.848 Melem/s 40.932 Melem/s]
                 change:
                        time:   [−1.5460% −1.1351% −0.7518%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7575% +1.1482% +1.5702%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in estimated 5.0025 s (1.7M iteratgraph_scaling/nodes_within_hops/100
                        time:   [2.8661 µs 2.8775 µs 2.8882 µs]
                        thrpt:  [34.623 Melem/s 34.752 Melem/s 34.890 Melem/s]
                 change:
                        time:   [+0.7196% +1.1911% +1.6208%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5950% −1.1771% −0.7145%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
graph_scaling/all_nodes/500
                        time:   [8.1528 µs 8.1750 µs 8.1983 µs]
                        thrpt:  [60.988 Melem/s 61.162 Melem/s 61.329 Melem/s]
                 change:
                        time:   [+3.1540% +3.6376% +4.0912%] (p = 0.00 < 0.05)
                        thrpt:  [−3.9304% −3.5100% −3.0576%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in estimated 5.0377 s (520k iteratgraph_scaling/nodes_within_hops/500
                        time:   [9.6663 µs 9.6884 µs 9.7101 µs]
                        thrpt:  [51.493 Melem/s 51.608 Melem/s 51.726 Melem/s]
                 change:
                        time:   [+3.4821% +3.8564% +4.2645%] (p = 0.00 < 0.05)
                        thrpt:  [−4.0900% −3.7132% −3.3649%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
graph_scaling/all_nodes/1000
                        time:   [188.96 µs 199.99 µs 209.25 µs]
                        thrpt:  [4.7789 Melem/s 5.0001 Melem/s 5.2921 Melem/s]
                 change:
                        time:   [+17.043% +22.843% +28.969%] (p = 0.00 < 0.05)
                        thrpt:  [−22.462% −18.595% −14.561%]
                        Performance has regressed.
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in estimated 5.0705 s (25k iteratgraph_scaling/nodes_within_hops/1000
                        time:   [193.34 µs 203.97 µs 212.72 µs]
                        thrpt:  [4.7011 Melem/s 4.9028 Melem/s 5.1723 Melem/s]
                 change:
                        time:   [+16.256% +22.135% +28.414%] (p = 0.00 < 0.05)
                        thrpt:  [−22.127% −18.124% −13.983%]
                        Performance has regressed.
Found 15 outliers among 100 measurements (15.00%)
  10 (10.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
graph_scaling/all_nodes/5000
                        time:   [258.06 µs 264.12 µs 269.99 µs]
                        thrpt:  [18.519 Melem/s 18.931 Melem/s 19.375 Melem/s]
                 change:
                        time:   [+15.916% +20.665% +25.845%] (p = 0.00 < 0.05)
                        thrpt:  [−20.537% −17.126% −13.731%]
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  7 (7.00%) low mild
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in estimated 5.5561 s (20k iteratgraph_scaling/nodes_within_hops/5000
                        time:   [271.18 µs 277.80 µs 283.72 µs]
                        thrpt:  [17.623 Melem/s 17.999 Melem/s 18.438 Melem/s]
                 change:
                        time:   [+13.715% +18.115% +23.000%] (p = 0.00 < 0.05)
                        thrpt:  [−18.699% −15.337% −12.061%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimated 5.0268 s (293k iterationscapability_search/find_with_gpu
                        time:   [17.148 µs 17.189 µs 17.239 µs]
                        thrpt:  [58.008 Kelem/s 58.175 Kelem/s 58.316 Kelem/s]
                 change:
                        time:   [−2.1292% −1.9069% −1.7205%] (p = 0.00 < 0.05)
                        thrpt:  [+1.7507% +1.9440% +2.1755%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples in estimated 5.1105 s (167k itercapability_search/find_by_tool_python
                        time:   [30.575 µs 30.640 µs 30.705 µs]
                        thrpt:  [32.568 Kelem/s 32.637 Kelem/s 32.706 Kelem/s]
                 change:
                        time:   [−2.1464% −1.7913% −1.4272%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4479% +1.8240% +2.1934%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in estimated 5.0042 s (126k iteratcapability_search/find_by_tool_rust
                        time:   [39.540 µs 39.641 µs 39.744 µs]
                        thrpt:  [25.161 Kelem/s 25.227 Kelem/s 25.291 Kelem/s]
                 change:
                        time:   [−1.4580% −1.1534% −0.8550%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8623% +1.1669% +1.4796%]
                        Change within noise threshold.

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples in estimated 5.0093 s (45k iteragraph_concurrent/concurrent_pingwave/4
                        time:   [111.12 µs 112.21 µs 113.45 µs]
                        thrpt:  [17.629 Melem/s 17.824 Melem/s 17.998 Melem/s]
                 change:
                        time:   [−3.8698% −1.9258% +0.0132%] (p = 0.07 > 0.05)
                        thrpt:  [−0.0132% +1.9636% +4.0256%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples in estimated 5.0133 s (28k iteragraph_concurrent/concurrent_pingwave/8
                        time:   [178.65 µs 180.32 µs 182.17 µs]
                        thrpt:  [21.958 Melem/s 22.183 Melem/s 22.390 Melem/s]
                 change:
                        time:   [−0.9792% +0.6823% +2.3874%] (p = 0.44 > 0.05)
                        thrpt:  [−2.3317% −0.6777% +0.9889%]
                        No change in performance detected.
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples in estimated 5.0039 s (16k itergraph_concurrent/concurrent_pingwave/16
                        time:   [314.48 µs 316.51 µs 318.49 µs]
                        thrpt:  [25.118 Melem/s 25.275 Melem/s 25.439 Melem/s]
                 change:
                        time:   [−0.5222% +0.0797% +0.7142%] (p = 0.81 > 0.05)
                        thrpt:  [−0.7092% −0.0796% +0.5250%]
                        No change in performance detected.

path_finding/path_1_hop time:   [1.5588 µs 1.5606 µs 1.5624 µs]
                        thrpt:  [640.06 Kelem/s 640.78 Kelem/s 641.52 Kelem/s]
                 change:
                        time:   [+2.7257% +2.9079% +3.0876%] (p = 0.00 < 0.05)
                        thrpt:  [−2.9952% −2.8258% −2.6533%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
path_finding/path_2_hops
                        time:   [1.6020 µs 1.6042 µs 1.6065 µs]
                        thrpt:  [622.48 Kelem/s 623.37 Kelem/s 624.20 Kelem/s]
                 change:
                        time:   [+0.2966% +0.4808% +0.6894%] (p = 0.00 < 0.05)
                        thrpt:  [−0.6846% −0.4785% −0.2958%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
path_finding/path_4_hops
                        time:   [1.8534 µs 1.8562 µs 1.8592 µs]
                        thrpt:  [537.87 Kelem/s 538.73 Kelem/s 539.54 Kelem/s]
                 change:
                        time:   [+1.3117% +1.5125% +1.7309%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7014% −1.4900% −1.2947%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
path_finding/path_not_found
                        time:   [1.7504 µs 1.7535 µs 1.7565 µs]
                        thrpt:  [569.31 Kelem/s 570.28 Kelem/s 571.29 Kelem/s]
                 change:
                        time:   [−4.6839% −4.4466% −4.2192%] (p = 0.00 < 0.05)
                        thrpt:  [+4.4050% +4.6535% +4.9140%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
path_finding/path_complex_graph
                        time:   [214.32 µs 215.84 µs 217.39 µs]
                        thrpt:  [4.6001 Kelem/s 4.6330 Kelem/s 4.6658 Kelem/s]
                 change:
                        time:   [−33.034% −32.064% −31.166%] (p = 0.00 < 0.05)
                        thrpt:  [+45.278% +47.198% +49.329%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in estimated 5.0001 s (169M iteratfailure_detector/heartbeat_existing
                        time:   [28.972 ns 29.328 ns 29.776 ns]
                        thrpt:  [33.584 Melem/s 34.097 Melem/s 34.517 Melem/s]
                 change:
                        time:   [+0.1604% +1.1214% +2.3116%] (p = 0.04 < 0.05)
                        thrpt:  [−2.2594% −1.1089% −0.1601%]
                        Change within noise threshold.
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) high mild
  11 (11.00%) high severe
failure_detector/heartbeat_new
                        time:   [252.35 ns 257.27 ns 261.79 ns]
                        thrpt:  [3.8198 Melem/s 3.8870 Melem/s 3.9627 Melem/s]
                 change:
                        time:   [−4.8298% −0.1548% +4.7744%] (p = 0.95 > 0.05)
                        thrpt:  [−4.5568% +0.1551% +5.0749%]
                        No change in performance detected.
failure_detector/status_check
                        time:   [13.897 ns 14.099 ns 14.309 ns]
                        thrpt:  [69.884 Melem/s 70.925 Melem/s 71.961 Melem/s]
                 change:
                        time:   [+3.0761% +5.0209% +7.0140%] (p = 0.00 < 0.05)
                        thrpt:  [−6.5543% −4.7808% −2.9843%]
                        Performance has regressed.
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 35.0s, or reduce sample count to 10.
failure_detector/check_all
                        time:   [343.22 ms 343.39 ms 343.61 ms]
                        thrpt:  [2.9103  elem/s 2.9122  elem/s 2.9136  elem/s]
                 change:
                        time:   [−0.3466% −0.2510% −0.1525%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1527% +0.2517% +0.3478%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.0s, or reduce sample count to 60.
failure_detector/stats  time:   [80.519 ms 80.558 ms 80.599 ms]
                        thrpt:  [12.407  elem/s 12.413  elem/s 12.419  elem/s]
                 change:
                        time:   [−0.7191% −0.6495% −0.5784%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5818% +0.6537% +0.7243%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimated 5.0000 s (1.8B iterationsloss_simulator/should_drop_1pct
                        time:   [2.7984 ns 2.8007 ns 2.8033 ns]
                        thrpt:  [356.73 Melem/s 357.05 Melem/s 357.35 Melem/s]
                 change:
                        time:   [−0.1200% +0.0489% +0.2213%] (p = 0.60 > 0.05)
                        thrpt:  [−0.2208% −0.0489% +0.1201%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimated 5.0000 s (1.6B iterationsloss_simulator/should_drop_5pct
                        time:   [3.1603 ns 3.1638 ns 3.1673 ns]
                        thrpt:  [315.73 Melem/s 316.08 Melem/s 316.42 Melem/s]
                 change:
                        time:   [−0.3438% −0.1499% +0.0569%] (p = 0.15 > 0.05)
                        thrpt:  [−0.0568% +0.1501% +0.3450%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estimated 5.0000 s (1.4B iterationloss_simulator/should_drop_10pct
                        time:   [3.6300 ns 3.6334 ns 3.6369 ns]
                        thrpt:  [274.96 Melem/s 275.22 Melem/s 275.48 Melem/s]
                 change:
                        time:   [−0.6159% −0.4400% −0.2744%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2752% +0.4419% +0.6197%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estimated 5.0000 s (1.1B iterationloss_simulator/should_drop_20pct
                        time:   [4.5920 ns 4.5966 ns 4.6019 ns]
                        thrpt:  [217.30 Melem/s 217.55 Melem/s 217.77 Melem/s]
                 change:
                        time:   [−0.1755% +0.0331% +0.2352%] (p = 0.75 > 0.05)
                        thrpt:  [−0.2346% −0.0331% +0.1758%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estimated 5.0000 s (1.7B iterationloss_simulator/should_drop_burst
                        time:   [2.9403 ns 2.9429 ns 2.9456 ns]
                        thrpt:  [339.49 Melem/s 339.80 Melem/s 340.10 Melem/s]
                 change:
                        time:   [−0.1686% +0.0321% +0.2507%] (p = 0.77 > 0.05)
                        thrpt:  [−0.2500% −0.0321% +0.1689%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

circuit_breaker/allow_closed
                        time:   [13.471 ns 13.481 ns 13.492 ns]
                        thrpt:  [74.118 Melem/s 74.177 Melem/s 74.235 Melem/s]
                 change:
                        time:   [−0.2606% −0.0843% +0.0919%] (p = 0.36 > 0.05)
                        thrpt:  [−0.0918% +0.0844% +0.2613%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
circuit_breaker/record_success
                        time:   [9.6557 ns 9.6668 ns 9.6804 ns]
                        thrpt:  [103.30 Melem/s 103.45 Melem/s 103.57 Melem/s]
                 change:
                        time:   [−0.1813% +0.0283% +0.2223%] (p = 0.79 > 0.05)
                        thrpt:  [−0.2218% −0.0283% +0.1816%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
circuit_breaker/record_failure
                        time:   [9.6117 ns 9.6195 ns 9.6278 ns]
                        thrpt:  [103.87 Melem/s 103.96 Melem/s 104.04 Melem/s]
                 change:
                        time:   [−0.3887% −0.0368% +0.2845%] (p = 0.84 > 0.05)
                        thrpt:  [−0.2837% +0.0368% +0.3902%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
circuit_breaker/state   time:   [13.471 ns 13.482 ns 13.494 ns]
                        thrpt:  [74.108 Melem/s 74.173 Melem/s 74.234 Melem/s]
                 change:
                        time:   [−2.6106% −2.4408% −2.2609%] (p = 0.00 < 0.05)
                        thrpt:  [+2.3131% +2.5019% +2.6805%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samples in estimated 5.0006 s (20Mrecovery_manager/on_failure_with_alternates
                        time:   [264.46 ns 268.97 ns 272.77 ns]
                        thrpt:  [3.6661 Melem/s 3.7179 Melem/s 3.7812 Melem/s]
                 change:
                        time:   [−7.5335% −3.1874% +1.5877%] (p = 0.19 > 0.05)
                        thrpt:  [−1.5629% +3.2923% +8.1473%]
                        No change in performance detected.
Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samples in estimated 5.0008 s (17M irecovery_manager/on_failure_no_alternates
                        time:   [290.58 ns 296.47 ns 301.63 ns]
                        thrpt:  [3.3153 Melem/s 3.3731 Melem/s 3.4413 Melem/s]
                 change:
                        time:   [−6.0321% −1.3229% +3.4548%] (p = 0.60 > 0.05)
                        thrpt:  [−3.3395% +1.3407% +6.4193%]
                        No change in performance detected.
recovery_manager/get_action
                        time:   [37.908 ns 38.066 ns 38.226 ns]
                        thrpt:  [26.160 Melem/s 26.270 Melem/s 26.379 Melem/s]
                 change:
                        time:   [+5.2349% +5.8944% +6.5564%] (p = 0.00 < 0.05)
                        thrpt:  [−6.1530% −5.5663% −4.9745%]
                        Performance has regressed.
recovery_manager/is_failed
                        time:   [12.796 ns 13.021 ns 13.252 ns]
                        thrpt:  [75.461 Melem/s 76.798 Melem/s 78.152 Melem/s]
                 change:
                        time:   [+4.5446% +7.2910% +9.8643%] (p = 0.00 < 0.05)
                        thrpt:  [−8.9786% −6.7955% −4.3470%]
                        Performance has regressed.
recovery_manager/on_recovery
                        time:   [106.37 ns 106.60 ns 106.91 ns]
                        thrpt:  [9.3540 Melem/s 9.3808 Melem/s 9.4016 Melem/s]
                 change:
                        time:   [−0.5227% +0.2297% +0.8792%] (p = 0.54 > 0.05)
                        thrpt:  [−0.8715% −0.2292% +0.5254%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
recovery_manager/stats  time:   [702.53 ps 703.02 ps 703.55 ps]
                        thrpt:  [1.4214 Gelem/s 1.4224 Gelem/s 1.4234 Gelem/s]
                 change:
                        time:   [−0.0304% +0.1400% +0.2960%] (p = 0.10 > 0.05)
                        thrpt:  [−0.2951% −0.1398% +0.0304%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe

failure_scaling/check_all/100
                        time:   [4.7959 µs 4.8177 µs 4.8355 µs]
                        thrpt:  [20.680 Melem/s 20.757 Melem/s 20.851 Melem/s]
                 change:
                        time:   [−1.4537% +0.4149% +2.3491%] (p = 0.67 > 0.05)
                        thrpt:  [−2.2951% −0.4132% +1.4752%]
                        No change in performance detected.
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estimated 5.0078 s (2.9M iteratiofailure_scaling/healthy_nodes/100
                        time:   [1.7228 µs 1.7240 µs 1.7251 µs]
                        thrpt:  [57.968 Melem/s 58.006 Melem/s 58.044 Melem/s]
                 change:
                        time:   [+1.2235% +1.4244% +1.6079%] (p = 0.00 < 0.05)
                        thrpt:  [−1.5824% −1.4044% −1.2087%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
failure_scaling/check_all/500
                        time:   [20.869 µs 20.939 µs 20.996 µs]
                        thrpt:  [23.814 Melem/s 23.878 Melem/s 23.959 Melem/s]
                 change:
                        time:   [−2.2453% −0.2245% +1.8157%] (p = 0.83 > 0.05)
                        thrpt:  [−1.7833% +0.2251% +2.2969%]
                        No change in performance detected.
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estimated 5.0095 s (919k iteratiofailure_scaling/healthy_nodes/500
                        time:   [5.4491 µs 5.4519 µs 5.4550 µs]
                        thrpt:  [91.660 Melem/s 91.711 Melem/s 91.759 Melem/s]
                 change:
                        time:   [−0.1083% +0.0662% +0.2460%] (p = 0.48 > 0.05)
                        thrpt:  [−0.2454% −0.0661% +0.1084%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
failure_scaling/check_all/1000
                        time:   [40.910 µs 41.076 µs 41.207 µs]
                        thrpt:  [24.268 Melem/s 24.345 Melem/s 24.444 Melem/s]
                 change:
                        time:   [−2.1778% −0.0462% +2.0759%] (p = 0.96 > 0.05)
                        thrpt:  [−2.0337% +0.0463% +2.2263%]
                        No change in performance detected.
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in estimated 5.0188 s (490k iteratifailure_scaling/healthy_nodes/1000
                        time:   [10.240 µs 10.292 µs 10.395 µs]
                        thrpt:  [96.204 Melem/s 97.159 Melem/s 97.659 Melem/s]
                 change:
                        time:   [−0.0308% +0.2499% +0.6720%] (p = 0.19 > 0.05)
                        thrpt:  [−0.6675% −0.2493% +0.0308%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
failure_scaling/check_all/5000
                        time:   [203.76 µs 204.17 µs 204.52 µs]
                        thrpt:  [24.448 Melem/s 24.489 Melem/s 24.539 Melem/s]
                 change:
                        time:   [−0.2871% −0.0182% +0.2631%] (p = 0.90 > 0.05)
                        thrpt:  [−0.2624% +0.0182% +0.2880%]
                        No change in performance detected.
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in estimated 5.0563 s (101k iteratifailure_scaling/healthy_nodes/5000
                        time:   [50.129 µs 50.173 µs 50.215 µs]
                        thrpt:  [99.572 Melem/s 99.655 Melem/s 99.743 Melem/s]
                 change:
                        time:   [+2.1125% +2.3226% +2.4866%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4263% −2.2699% −2.0688%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samples in estimated 5.0237 s (28k itfailure_concurrent/concurrent_heartbeat/4
                        time:   [182.83 µs 183.40 µs 184.09 µs]
                        thrpt:  [10.864 Melem/s 10.905 Melem/s 10.939 Melem/s]
                 change:
                        time:   [−1.7211% −1.1992% −0.6721%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6766% +1.2137% +1.7513%]
                        Change within noise threshold.
Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samples in estimated 5.0546 s (18k itfailure_concurrent/concurrent_heartbeat/8
                        time:   [259.42 µs 260.27 µs 261.55 µs]
                        thrpt:  [15.294 Melem/s 15.369 Melem/s 15.419 Melem/s]
                 change:
                        time:   [−1.1732% +0.1545% +1.2767%] (p = 0.82 > 0.05)
                        thrpt:  [−1.2606% −0.1543% +1.1871%]
                        No change in performance detected.
Found 5 outliers among 20 measurements (25.00%)
  2 (10.00%) low severe
  3 (15.00%) high severe
Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samples in estimated 5.0489 s (11k ifailure_concurrent/concurrent_heartbeat/16
                        time:   [467.05 µs 468.32 µs 469.50 µs]
                        thrpt:  [17.040 Melem/s 17.082 Melem/s 17.129 Melem/s]
                 change:
                        time:   [−0.2113% +0.1187% +0.4490%] (p = 0.50 > 0.05)
                        thrpt:  [−0.4470% −0.1186% +0.2118%]
                        No change in performance detected.
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) low mild

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estimated 5.0002 s (16M iterationfailure_recovery_cycle/full_cycle
                        time:   [300.03 ns 306.54 ns 311.96 ns]
                        thrpt:  [3.2055 Melem/s 3.2622 Melem/s 3.3331 Melem/s]
                 change:
                        time:   [−3.7412% +1.0422% +6.1838%] (p = 0.68 > 0.05)
                        thrpt:  [−5.8237% −1.0314% +3.8866%]
                        No change in performance detected.

capability_set/create   time:   [517.70 ns 518.55 ns 519.38 ns]
                        thrpt:  [1.9254 Melem/s 1.9285 Melem/s 1.9316 Melem/s]
                 change:
                        time:   [−1.8351% −1.2941% −0.7673%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7732% +1.3111% +1.8694%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  2 (2.00%) high severe
capability_set/serialize
                        time:   [921.54 ns 922.25 ns 922.99 ns]
                        thrpt:  [1.0834 Melem/s 1.0843 Melem/s 1.0851 Melem/s]
                 change:
                        time:   [+0.4222% +0.6792% +0.9742%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9648% −0.6746% −0.4205%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
capability_set/deserialize
                        time:   [1.7170 µs 1.7187 µs 1.7204 µs]
                        thrpt:  [581.24 Kelem/s 581.84 Kelem/s 582.41 Kelem/s]
                 change:
                        time:   [−0.1950% −0.0180% +0.1502%] (p = 0.84 > 0.05)
                        thrpt:  [−0.1499% +0.0180% +0.1954%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
capability_set/roundtrip
                        time:   [2.6768 µs 2.6815 µs 2.6880 µs]
                        thrpt:  [372.02 Kelem/s 372.93 Kelem/s 373.57 Kelem/s]
                 change:
                        time:   [−0.1891% −0.0077% +0.1650%] (p = 0.93 > 0.05)
                        thrpt:  [−0.1648% +0.0077% +0.1895%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
capability_set/has_tag  time:   [749.61 ps 750.07 ps 750.55 ps]
                        thrpt:  [1.3324 Gelem/s 1.3332 Gelem/s 1.3340 Gelem/s]
                 change:
                        time:   [−0.0423% +0.1101% +0.2872%] (p = 0.18 > 0.05)
                        thrpt:  [−0.2864% −0.1099% +0.0423%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
capability_set/has_model
                        time:   [936.30 ps 937.00 ps 937.70 ps]
                        thrpt:  [1.0664 Gelem/s 1.0672 Gelem/s 1.0680 Gelem/s]
                 change:
                        time:   [−0.8014% −0.3861% −0.0065%] (p = 0.06 > 0.05)
                        thrpt:  [+0.0065% +0.3876% +0.8079%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
capability_set/has_tool time:   [749.54 ps 750.08 ps 750.64 ps]
                        thrpt:  [1.3322 Gelem/s 1.3332 Gelem/s 1.3342 Gelem/s]
                 change:
                        time:   [−0.1662% +0.0191% +0.1960%] (p = 0.84 > 0.05)
                        thrpt:  [−0.1956% −0.0191% +0.1664%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
capability_set/has_gpu  time:   [312.32 ps 312.52 ps 312.72 ps]
                        thrpt:  [3.1978 Gelem/s 3.1998 Gelem/s 3.2018 Gelem/s]
                 change:
                        time:   [−0.3065% −0.0140% +0.2331%] (p = 0.93 > 0.05)
                        thrpt:  [−0.2326% +0.0140% +0.3075%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

capability_announcement/create
                        time:   [376.05 ns 376.79 ns 377.57 ns]
                        thrpt:  [2.6485 Melem/s 2.6540 Melem/s 2.6592 Melem/s]
                 change:
                        time:   [+0.5920% +0.8364% +1.0904%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0787% −0.8295% −0.5885%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_announcement/serialize: Collecting 100 samples in estimated 5.0000 s (4.1M iteratiocapability_announcement/serialize
                        time:   [1.2216 µs 1.2230 µs 1.2244 µs]
                        thrpt:  [816.70 Kelem/s 817.68 Kelem/s 818.63 Kelem/s]
                 change:
                        time:   [+0.0631% +0.2969% +0.5231%] (p = 0.01 < 0.05)
                        thrpt:  [−0.5204% −0.2960% −0.0631%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in estimated 5.0078 s (2.4M iteratcapability_announcement/deserialize
                        time:   [2.0960 µs 2.1078 µs 2.1216 µs]
                        thrpt:  [471.34 Kelem/s 474.44 Kelem/s 477.09 Kelem/s]
                 change:
                        time:   [−2.1375% −1.3523% −0.5969%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6005% +1.3709% +2.1841%]
                        Change within noise threshold.
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) high mild
  20 (20.00%) high severe
Benchmarking capability_announcement/is_expired: Collecting 100 samples in estimated 5.0000 s (197M iteraticapability_announcement/is_expired
                        time:   [25.317 ns 25.353 ns 25.394 ns]
                        thrpt:  [39.379 Melem/s 39.443 Melem/s 39.499 Melem/s]
                 change:
                        time:   [−0.2721% −0.0115% +0.2391%] (p = 0.93 > 0.05)
                        thrpt:  [−0.2385% +0.0115% +0.2729%]
                        No change in performance detected.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in estimated 5.0000 s (500M iteraticapability_filter/match_single_tag
                        time:   [9.9836 ns 9.9901 ns 9.9969 ns]
                        thrpt:  [100.03 Melem/s 100.10 Melem/s 100.16 Melem/s]
                 change:
                        time:   [−0.2269% −0.0645% +0.0897%] (p = 0.44 > 0.05)
                        thrpt:  [−0.0896% +0.0645% +0.2274%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in estimated 5.0000 s (1.2B iteratcapability_filter/match_require_gpu
                        time:   [4.0573 ns 4.0606 ns 4.0642 ns]
                        thrpt:  [246.05 Melem/s 246.27 Melem/s 246.47 Melem/s]
                 change:
                        time:   [−1.6922% −0.6118% +0.1326%] (p = 0.24 > 0.05)
                        thrpt:  [−0.1324% +0.6156% +1.7214%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in estimated 5.0000 s (1.3B iteraticapability_filter/match_gpu_vendor
                        time:   [3.7466 ns 3.7491 ns 3.7519 ns]
                        thrpt:  [266.53 Melem/s 266.73 Melem/s 266.91 Melem/s]
                 change:
                        time:   [−0.1569% +0.0218% +0.2289%] (p = 0.83 > 0.05)
                        thrpt:  [−0.2284% −0.0218% +0.1571%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in estimated 5.0000 s (1.3B iteraticapability_filter/match_min_memory
                        time:   [3.7465 ns 3.7492 ns 3.7520 ns]
                        thrpt:  [266.53 Melem/s 266.72 Melem/s 266.91 Melem/s]
                 change:
                        time:   [−0.1219% +0.0615% +0.2838%] (p = 0.55 > 0.05)
                        thrpt:  [−0.2830% −0.0615% +0.1220%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimated 5.0000 s (470M iterationscapability_filter/match_complex
                        time:   [10.617 ns 10.624 ns 10.631 ns]
                        thrpt:  [94.067 Melem/s 94.129 Melem/s 94.188 Melem/s]
                 change:
                        time:   [+2.9011% +3.1294% +3.4440%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3294% −3.0345% −2.8193%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_no_match: Collecting 100 samples in estimated 5.0000 s (1.6B iterationcapability_filter/match_no_match
                        time:   [3.1227 ns 3.1247 ns 3.1268 ns]
                        thrpt:  [319.81 Melem/s 320.03 Melem/s 320.23 Melem/s]
                 change:
                        time:   [−0.0518% +0.1299% +0.3128%] (p = 0.17 > 0.05)
                        thrpt:  [−0.3118% −0.1297% +0.0518%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples in estimated 5.1975 s (45k itecapability_index_insert/index_nodes/100
                        time:   [121.21 µs 124.56 µs 127.63 µs]
                        thrpt:  [783.50 Kelem/s 802.80 Kelem/s 824.99 Kelem/s]
                 change:
                        time:   [+1.9986% +3.5235% +5.1707%] (p = 0.00 < 0.05)
                        thrpt:  [−4.9165% −3.4036% −1.9594%]
                        Performance has regressed.
Found 20 outliers among 100 measurements (20.00%)
  4 (4.00%) high mild
  16 (16.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.1s, enable flat sampling, or reduce sample count to 60.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samples in estimated 6.1230 s (5050 icapability_index_insert/index_nodes/1000
                        time:   [1.3370 ms 1.3891 ms 1.4369 ms]
                        thrpt:  [695.93 Kelem/s 719.87 Kelem/s 747.93 Kelem/s]
                 change:
                        time:   [+3.1493% +5.7600% +7.9655%] (p = 0.00 < 0.05)
                        thrpt:  [−7.3778% −5.4463% −3.0532%]
                        Performance has regressed.
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samples in estimated 5.3734 s (300 icapability_index_insert/index_nodes/10000
                        time:   [18.319 ms 18.595 ms 18.886 ms]
                        thrpt:  [529.49 Kelem/s 537.79 Kelem/s 545.88 Kelem/s]
                 change:
                        time:   [−7.2989% −5.5634% −3.7525%] (p = 0.00 < 0.05)
                        thrpt:  [+3.8989% +5.8912% +7.8736%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples in estimated 5.5988 s (30k itecapability_index_query/query_single_tag
                        time:   [180.96 µs 187.86 µs 195.19 µs]
                        thrpt:  [5.1233 Kelem/s 5.3231 Kelem/s 5.5260 Kelem/s]
                 change:
                        time:   [−4.6493% +1.3764% +7.3801%] (p = 0.66 > 0.05)
                        thrpt:  [−6.8729% −1.3577% +4.8760%]
                        No change in performance detected.
Benchmarking capability_index_query/query_require_gpu: Collecting 100 samples in estimated 5.8081 s (25k itcapability_index_query/query_require_gpu
                        time:   [221.77 µs 231.82 µs 241.82 µs]
                        thrpt:  [4.1353 Kelem/s 4.3137 Kelem/s 4.5092 Kelem/s]
                 change:
                        time:   [−4.8630% +2.7134% +10.325%] (p = 0.48 > 0.05)
                        thrpt:  [−9.3589% −2.6417% +5.1115%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples in estimated 7.3610 s (10k itecapability_index_query/query_gpu_vendor
                        time:   [722.11 µs 734.28 µs 746.66 µs]
                        thrpt:  [1.3393 Kelem/s 1.3619 Kelem/s 1.3848 Kelem/s]
                 change:
                        time:   [−4.4023% −1.4944% +1.3446%] (p = 0.32 > 0.05)
                        thrpt:  [−1.3268% +1.5171% +4.6051%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples in estimated 7.8053 s (10k itecapability_index_query/query_min_memory
                        time:   [763.66 µs 773.99 µs 784.94 µs]
                        thrpt:  [1.2740 Kelem/s 1.2920 Kelem/s 1.3095 Kelem/s]
                 change:
                        time:   [−4.7996% −2.5379% −0.3654%] (p = 0.02 < 0.05)
                        thrpt:  [+0.3668% +2.6040% +5.0416%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_query/query_complex: Collecting 100 samples in estimated 7.4637 s (15k iteratcapability_index_query/query_complex
                        time:   [476.40 µs 491.11 µs 505.80 µs]
                        thrpt:  [1.9771 Kelem/s 2.0362 Kelem/s 2.0991 Kelem/s]
                 change:
                        time:   [−11.324% −7.2646% −3.0273%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1218% +7.8336% +12.770%]
                        Performance has improved.
Benchmarking capability_index_query/query_model: Collecting 100 samples in estimated 5.0535 s (50k iteratiocapability_index_query/query_model
                        time:   [99.833 µs 100.25 µs 100.80 µs]
                        thrpt:  [9.9205 Kelem/s 9.9749 Kelem/s 10.017 Kelem/s]
                 change:
                        time:   [−0.8286% −0.4097% +0.0501%] (p = 0.08 > 0.05)
                        thrpt:  [−0.0501% +0.4114% +0.8355%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estimated 6.0347 s (10k iterationcapability_index_query/query_tool
                        time:   [606.60 µs 628.35 µs 650.08 µs]
                        thrpt:  [1.5383 Kelem/s 1.5915 Kelem/s 1.6485 Kelem/s]
                 change:
                        time:   [+2.6809% +7.5246% +12.366%] (p = 0.00 < 0.05)
                        thrpt:  [−11.005% −6.9980% −2.6109%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capability_index_query/query_no_results: Collecting 100 samples in estimated 5.0000 s (220M itcapability_index_query/query_no_results
                        time:   [22.582 ns 22.669 ns 22.725 ns]
                        thrpt:  [44.005 Melem/s 44.113 Melem/s 44.283 Melem/s]
                 change:
                        time:   [−0.6406% −0.3995% −0.1652%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1655% +0.4011% +0.6448%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  4 (4.00%) high mild

Benchmarking capability_index_find_best/find_best_simple: Collecting 100 samples in estimated 5.9220 s (15kcapability_index_find_best/find_best_simple
                        time:   [381.98 µs 393.68 µs 405.59 µs]
                        thrpt:  [2.4656 Kelem/s 2.5401 Kelem/s 2.6179 Kelem/s]
                 change:
                        time:   [−16.976% −13.584% −10.097%] (p = 0.00 < 0.05)
                        thrpt:  [+11.231% +15.719% +20.447%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 7.1212 s capability_index_find_best/find_best_with_prefs
                        time:   [700.94 µs 714.53 µs 728.11 µs]
                        thrpt:  [1.3734 Kelem/s 1.3995 Kelem/s 1.4267 Kelem/s]
                 change:
                        time:   [−13.569% −10.838% −7.9811%] (p = 0.00 < 0.05)
                        thrpt:  [+8.6733% +12.155% +15.699%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples in estimated 5.0014 s (399k itcapability_index_scaling/query_tag/1000
                        time:   [12.529 µs 12.538 µs 12.548 µs]
                        thrpt:  [79.692 Kelem/s 79.755 Kelem/s 79.817 Kelem/s]
                 change:
                        time:   [−0.3979% −0.2339% −0.0728%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0729% +0.2345% +0.3995%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 samples in estimated 5.1151 s (126capability_index_scaling/query_complex/1000
                        time:   [40.244 µs 40.438 µs 40.630 µs]
                        thrpt:  [24.612 Kelem/s 24.729 Kelem/s 24.849 Kelem/s]
                 change:
                        time:   [−0.8022% −0.2189% +0.4371%] (p = 0.49 > 0.05)
                        thrpt:  [−0.4352% +0.2194% +0.8087%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples in estimated 5.3456 s (76k itecapability_index_scaling/query_tag/5000
                        time:   [70.274 µs 70.401 µs 70.527 µs]
                        thrpt:  [14.179 Kelem/s 14.204 Kelem/s 14.230 Kelem/s]
                 change:
                        time:   [−0.1172% +0.2647% +0.7316%] (p = 0.22 > 0.05)
                        thrpt:  [−0.7263% −0.2640% +0.1173%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 samples in estimated 5.1688 s (25kcapability_index_scaling/query_complex/5000
                        time:   [204.28 µs 205.55 µs 206.77 µs]
                        thrpt:  [4.8363 Kelem/s 4.8651 Kelem/s 4.8952 Kelem/s]
                 change:
                        time:   [+0.0474% +1.5402% +2.9103%] (p = 0.03 < 0.05)
                        thrpt:  [−2.8280% −1.5169% −0.0474%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samples in estimated 5.1598 s (30k itcapability_index_scaling/query_tag/10000
                        time:   [169.85 µs 175.94 µs 182.65 µs]
                        thrpt:  [5.4748 Kelem/s 5.6837 Kelem/s 5.8874 Kelem/s]
                 change:
                        time:   [−44.569% −40.790% −36.866%] (p = 0.00 < 0.05)
                        thrpt:  [+58.392% +68.891% +80.404%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samples in estimated 6.9836 s (15capability_index_scaling/query_complex/10000
                        time:   [452.51 µs 459.95 µs 467.47 µs]
                        thrpt:  [2.1392 Kelem/s 2.1742 Kelem/s 2.2099 Kelem/s]
                 change:
                        time:   [−34.310% −32.652% −31.139%] (p = 0.00 < 0.05)
                        thrpt:  [+45.221% +48.482% +52.230%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samples in estimated 5.0902 s (2100 icapability_index_scaling/query_tag/50000
                        time:   [2.3999 ms 2.4207 ms 2.4417 ms]
                        thrpt:  [409.55  elem/s 413.11  elem/s 416.69  elem/s]
                 change:
                        time:   [−11.760% −10.766% −9.7365%] (p = 0.00 < 0.05)
                        thrpt:  [+10.787% +12.065% +13.327%]
                        Performance has improved.
Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samples in estimated 5.1149 s (14capability_index_scaling/query_complex/50000
                        time:   [3.6212 ms 3.6456 ms 3.6705 ms]
                        thrpt:  [272.44  elem/s 274.30  elem/s 276.16  elem/s]
                 change:
                        time:   [−9.4930% −8.5825% −7.6009%] (p = 0.00 < 0.05)
                        thrpt:  [+8.2262% +9.3883% +10.489%]
                        Performance has improved.

Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 samples in estimated 5.0674 s (1capability_index_concurrent/concurrent_index/4
                        time:   [394.13 µs 394.61 µs 395.06 µs]
                        thrpt:  [5.0625 Melem/s 5.0683 Melem/s 5.0744 Melem/s]
                 change:
                        time:   [−0.1961% +0.0143% +0.2193%] (p = 0.90 > 0.05)
                        thrpt:  [−0.2188% −0.0143% +0.1965%]
                        No change in performance detected.
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.0s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 samples in estimated 8.0401 s (2capability_index_concurrent/concurrent_query/4
                        time:   [406.00 ms 408.90 ms 411.90 ms]
                        thrpt:  [4.8556 Kelem/s 4.8911 Kelem/s 4.9261 Kelem/s]
                 change:
                        time:   [−13.966% −13.019% −12.054%] (p = 0.00 < 0.05)
                        thrpt:  [+13.706% +14.968% +16.234%]
                        Performance has improved.
Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 7.3497 s (4capability_index_concurrent/concurrent_mixed/4
                        time:   [177.31 ms 181.82 ms 186.13 ms]
                        thrpt:  [10.745 Kelem/s 11.000 Kelem/s 11.280 Kelem/s]
                 change:
                        time:   [−2.5495% +0.7164% +4.1826%] (p = 0.69 > 0.05)
                        thrpt:  [−4.0146% −0.7113% +2.6162%]
                        No change in performance detected.
Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 samples in estimated 5.0352 s (1capability_index_concurrent/concurrent_index/8
                        time:   [468.38 µs 469.90 µs 471.16 µs]
                        thrpt:  [8.4897 Melem/s 8.5125 Melem/s 8.5400 Melem/s]
                 change:
                        time:   [+2.5429% +3.3905% +4.2558%] (p = 0.00 < 0.05)
                        thrpt:  [−4.0821% −3.2793% −2.4798%]
                        Performance has regressed.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  2 (10.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.4s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 samples in estimated 10.436 s (2capability_index_concurrent/concurrent_query/8
                        time:   [521.29 ms 524.92 ms 528.38 ms]
                        thrpt:  [7.5703 Kelem/s 7.6203 Kelem/s 7.6733 Kelem/s]
                 change:
                        time:   [−2.4746% −1.6574% −0.9147%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9231% +1.6853% +2.5374%]
                        Change within noise threshold.
Benchmarking capability_index_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.2s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 5.1785 s (2capability_index_concurrent/concurrent_mixed/8
                        time:   [255.92 ms 259.64 ms 263.89 ms]
                        thrpt:  [15.158 Kelem/s 15.406 Kelem/s 15.630 Kelem/s]
                 change:
                        time:   [−0.9562% +0.4210% +2.1813%] (p = 0.63 > 0.05)
                        thrpt:  [−2.1348% −0.4192% +0.9654%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.0026 s (capability_index_concurrent/concurrent_index/16
                        time:   [950.27 µs 951.61 µs 952.65 µs]
                        thrpt:  [8.3976 Melem/s 8.4068 Melem/s 8.4186 Melem/s]
                 change:
                        time:   [+3.5896% +4.4852% +5.0989%] (p = 0.00 < 0.05)
                        thrpt:  [−4.8515% −4.2927% −3.4652%]
                        Performance has regressed.
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) low severe
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 20.5s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 samples in estimated 20.517 s (capability_index_concurrent/concurrent_query/16
                        time:   [1.0401 s 1.0455 s 1.0511 s]
                        thrpt:  [7.6110 Kelem/s 7.6522 Kelem/s 7.6916 Kelem/s]
                 change:
                        time:   [+1.6845% +2.4597% +3.1470%] (p = 0.00 < 0.05)
                        thrpt:  [−3.0510% −2.4007% −1.6566%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.8s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 10.795 s (capability_index_concurrent/concurrent_mixed/16
                        time:   [548.29 ms 550.21 ms 552.05 ms]
                        thrpt:  [14.491 Kelem/s 14.540 Kelem/s 14.591 Kelem/s]
                 change:
                        time:   [+2.3142% +2.7009% +3.0689%] (p = 0.00 < 0.05)
                        thrpt:  [−2.9775% −2.6298% −2.2618%]
                        Performance has regressed.

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.0008 s (capability_index_updates/update_higher_version
                        time:   [560.67 ns 561.63 ns 562.61 ns]
                        thrpt:  [1.7774 Melem/s 1.7805 Melem/s 1.7836 Melem/s]
                 change:
                        time:   [−0.2334% +0.2072% +0.7407%] (p = 0.44 > 0.05)
                        thrpt:  [−0.7353% −0.2068% +0.2340%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  5 (5.00%) high severe
Benchmarking capability_index_updates/update_same_version: Collecting 100 samples in estimated 5.0016 s (9.capability_index_updates/update_same_version
                        time:   [558.43 ns 559.47 ns 560.51 ns]
                        thrpt:  [1.7841 Melem/s 1.7874 Melem/s 1.7907 Melem/s]
                 change:
                        time:   [−0.3609% −0.1081% +0.1396%] (p = 0.40 > 0.05)
                        thrpt:  [−0.1394% +0.1082% +0.3622%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samples in estimated 5.0050 s (3.6M capability_index_updates/remove_and_readd
                        time:   [1.3896 µs 1.3995 µs 1.4097 µs]
                        thrpt:  [709.39 Kelem/s 714.53 Kelem/s 719.62 Kelem/s]
                 change:
                        time:   [+0.3196% +0.9831% +1.6901%] (p = 0.01 < 0.05)
                        thrpt:  [−1.6620% −0.9736% −0.3186%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

diff_op/create_add_tag  time:   [14.844 ns 14.913 ns 14.988 ns]
                        thrpt:  [66.722 Melem/s 67.055 Melem/s 67.367 Melem/s]
                 change:
                        time:   [+3.1484% +3.8982% +4.5860%] (p = 0.00 < 0.05)
                        thrpt:  [−4.3849% −3.7519% −3.0523%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
diff_op/create_remove_tag
                        time:   [14.941 ns 15.041 ns 15.149 ns]
                        thrpt:  [66.013 Melem/s 66.487 Melem/s 66.930 Melem/s]
                 change:
                        time:   [+1.9054% +2.7044% +3.4489%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3339% −2.6332% −1.8698%]
                        Performance has regressed.
diff_op/create_add_model
                        time:   [48.715 ns 48.797 ns 48.886 ns]
                        thrpt:  [20.456 Melem/s 20.493 Melem/s 20.527 Melem/s]
                 change:
                        time:   [+0.6654% +0.8807% +1.0992%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0873% −0.8731% −0.6610%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
diff_op/create_update_model
                        time:   [15.075 ns 15.167 ns 15.264 ns]
                        thrpt:  [65.513 Melem/s 65.931 Melem/s 66.337 Melem/s]
                 change:
                        time:   [−1.0552% −0.3329% +0.3945%] (p = 0.37 > 0.05)
                        thrpt:  [−0.3929% +0.3340% +1.0664%]
                        No change in performance detected.
diff_op/estimated_size  time:   [1.9221 ns 1.9235 ns 1.9248 ns]
                        thrpt:  [519.53 Melem/s 519.89 Melem/s 520.27 Melem/s]
                 change:
                        time:   [−0.2821% −0.0177% +0.2156%] (p = 0.90 > 0.05)
                        thrpt:  [−0.2151% +0.0177% +0.2829%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe

capability_diff/create  time:   [103.10 ns 103.54 ns 103.97 ns]
                        thrpt:  [9.6177 Melem/s 9.6580 Melem/s 9.6995 Melem/s]
                 change:
                        time:   [+0.7160% +1.3673% +2.0282%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9879% −1.3489% −0.7109%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
capability_diff/serialize
                        time:   [152.49 ns 152.70 ns 152.92 ns]
                        thrpt:  [6.5394 Melem/s 6.5488 Melem/s 6.5577 Melem/s]
                 change:
                        time:   [+0.2106% +0.7610% +1.2145%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1999% −0.7553% −0.2101%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_diff/deserialize
                        time:   [243.65 ns 244.36 ns 245.18 ns]
                        thrpt:  [4.0787 Melem/s 4.0924 Melem/s 4.1042 Melem/s]
                 change:
                        time:   [−1.7515% −1.4272% −1.1104%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1229% +1.4479% +1.7827%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
capability_diff/estimated_size
                        time:   [5.3092 ns 5.3131 ns 5.3173 ns]
                        thrpt:  [188.07 Melem/s 188.21 Melem/s 188.35 Melem/s]
                 change:
                        time:   [−0.1462% +0.0633% +0.2684%] (p = 0.56 > 0.05)
                        thrpt:  [−0.2677% −0.0633% +0.1465%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

diff_generation/no_changes
                        time:   [346.26 ns 346.77 ns 347.42 ns]
                        thrpt:  [2.8783 Melem/s 2.8838 Melem/s 2.8880 Melem/s]
                 change:
                        time:   [−0.2247% +0.0708% +0.3544%] (p = 0.64 > 0.05)
                        thrpt:  [−0.3531% −0.0707% +0.2252%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
diff_generation/add_one_tag
                        time:   [453.28 ns 453.80 ns 454.45 ns]
                        thrpt:  [2.2005 Melem/s 2.2036 Melem/s 2.2061 Melem/s]
                 change:
                        time:   [−1.7379% −1.2369% −0.7466%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7523% +1.2523% +1.7687%]
                        Change within noise threshold.
Found 19 outliers among 100 measurements (19.00%)
  6 (6.00%) high mild
  13 (13.00%) high severe
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in estimated 5.0022 s (9.6M iteradiff_generation/multiple_tag_changes
                        time:   [515.01 ns 515.96 ns 516.93 ns]
                        thrpt:  [1.9345 Melem/s 1.9381 Melem/s 1.9417 Melem/s]
                 change:
                        time:   [+0.5944% +0.9345% +1.2863%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2700% −0.9258% −0.5909%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in estimated 5.0003 s (12M iteratidiff_generation/update_model_loaded
                        time:   [405.86 ns 406.26 ns 406.72 ns]
                        thrpt:  [2.4587 Melem/s 2.4615 Melem/s 2.4639 Melem/s]
                 change:
                        time:   [−0.7038% −0.3817% −0.0323%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0323% +0.3831% +0.7088%]
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) high mild
  3 (3.00%) high severe
diff_generation/update_memory
                        time:   [394.54 ns 395.01 ns 395.50 ns]
                        thrpt:  [2.5285 Melem/s 2.5316 Melem/s 2.5346 Melem/s]
                 change:
                        time:   [−0.6965% −0.3072% +0.0975%] (p = 0.13 > 0.05)
                        thrpt:  [−0.0974% +0.3081% +0.7014%]
                        No change in performance detected.
Found 17 outliers among 100 measurements (17.00%)
  9 (9.00%) high mild
  8 (8.00%) high severe
diff_generation/add_model
                        time:   [486.16 ns 486.58 ns 487.05 ns]
                        thrpt:  [2.0532 Melem/s 2.0552 Melem/s 2.0569 Melem/s]
                 change:
                        time:   [−0.8984% −0.6142% −0.3429%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3441% +0.6180% +0.9065%]
                        Change within noise threshold.
Found 19 outliers among 100 measurements (19.00%)
  15 (15.00%) high mild
  4 (4.00%) high severe
diff_generation/complex_diff
                        time:   [781.59 ns 782.50 ns 783.62 ns]
                        thrpt:  [1.2761 Melem/s 1.2780 Melem/s 1.2794 Melem/s]
                 change:
                        time:   [+0.0899% +0.4655% +0.8528%] (p = 0.01 < 0.05)
                        thrpt:  [−0.8456% −0.4633% −0.0898%]
                        Change within noise threshold.
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estimated 5.0012 s (14M iterationsdiff_application/apply_single_op
                        time:   [363.49 ns 363.90 ns 364.39 ns]
                        thrpt:  [2.7443 Melem/s 2.7480 Melem/s 2.7511 Melem/s]
                 change:
                        time:   [+0.4467% +0.6733% +0.9226%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9141% −0.6688% −0.4447%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estimated 5.0006 s (14M iterationdiff_application/apply_small_diff
                        time:   [367.48 ns 367.75 ns 368.03 ns]
                        thrpt:  [2.7171 Melem/s 2.7193 Melem/s 2.7212 Melem/s]
                 change:
                        time:   [+0.5544% +0.7594% +0.9633%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9541% −0.7536% −0.5514%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in estimated 5.0035 s (5.9M iteratidiff_application/apply_medium_diff
                        time:   [842.54 ns 843.39 ns 844.28 ns]
                        thrpt:  [1.1844 Melem/s 1.1857 Melem/s 1.1869 Melem/s]
                 change:
                        time:   [+0.1516% +0.3349% +0.5129%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5103% −0.3338% −0.1514%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in estimated 5.0000 s (14M iteratiodiff_application/apply_strict_mode
                        time:   [363.70 ns 364.05 ns 364.42 ns]
                        thrpt:  [2.7441 Melem/s 2.7469 Melem/s 2.7495 Melem/s]
                 change:
                        time:   [−0.1305% +0.1075% +0.3199%] (p = 0.36 > 0.05)
                        thrpt:  [−0.3189% −0.1074% +0.1307%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples in estimated 5.0000 s (825M itdiff_chain_validation/validate_chain_10
                        time:   [6.0527 ns 6.0567 ns 6.0610 ns]
                        thrpt:  [164.99 Melem/s 165.11 Melem/s 165.22 Melem/s]
                 change:
                        time:   [−0.2031% −0.0330% +0.1261%] (p = 0.70 > 0.05)
                        thrpt:  [−0.1259% +0.0330% +0.2036%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samples in estimated 5.0003 s (74M itdiff_chain_validation/validate_chain_100
                        time:   [67.557 ns 67.622 ns 67.691 ns]
                        thrpt:  [14.773 Melem/s 14.788 Melem/s 14.802 Melem/s]
                 change:
                        time:   [−0.5567% −0.3216% −0.0983%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0984% +0.3227% +0.5599%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimated 5.0049 s (1.5M iterationsdiff_compaction/compact_5_diffs
                        time:   [3.2640 µs 3.2668 µs 3.2698 µs]
                        thrpt:  [305.83 Kelem/s 306.11 Kelem/s 306.37 Kelem/s]
                 change:
                        time:   [−0.0028% +0.2392% +0.4629%] (p = 0.04 < 0.05)
                        thrpt:  [−0.4608% −0.2387% +0.0028%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estimated 5.0012 s (333k iterationdiff_compaction/compact_20_diffs
                        time:   [14.892 µs 14.915 µs 14.938 µs]
                        thrpt:  [66.943 Kelem/s 67.048 Kelem/s 67.151 Kelem/s]
                 change:
                        time:   [−0.7376% −0.4278% −0.1596%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1599% +0.4296% +0.7431%]
                        Change within noise threshold.

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estimated 5.0029 s (5.5M iteratiobandwidth_savings/calculate_small
                        time:   [909.45 ns 910.37 ns 911.35 ns]
                        thrpt:  [1.0973 Melem/s 1.0985 Melem/s 1.0996 Melem/s]
                 change:
                        time:   [+0.7889% +1.0113% +1.2637%] (p = 0.00 < 0.05)
                        thrpt:  [−1.2479% −1.0012% −0.7827%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in estimated 5.0020 s (5.5M iteratibandwidth_savings/calculate_medium
                        time:   [915.20 ns 915.91 ns 916.67 ns]
                        thrpt:  [1.0909 Melem/s 1.0918 Melem/s 1.0927 Melem/s]
                 change:
                        time:   [+0.6720% +0.8037% +0.9424%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9336% −0.7973% −0.6675%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in estimated 5.0007 s (3.7M iteradiff_roundtrip/generate_apply_verify
                        time:   [1.3473 µs 1.3486 µs 1.3499 µs]
                        thrpt:  [740.79 Kelem/s 741.49 Kelem/s 742.20 Kelem/s]
                 change:
                        time:   [−0.4044% −0.0366% +0.2551%] (p = 0.84 > 0.05)
                        thrpt:  [−0.2545% +0.0366% +0.4060%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

location_info/create    time:   [28.387 ns 28.448 ns 28.511 ns]
                        thrpt:  [35.074 Melem/s 35.152 Melem/s 35.228 Melem/s]
                 change:
                        time:   [+0.3641% +0.6506% +0.9364%] (p = 0.00 < 0.05)
                        thrpt:  [−0.9277% −0.6464% −0.3628%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
location_info/distance_to
                        time:   [3.9450 ns 3.9493 ns 3.9540 ns]
                        thrpt:  [252.91 Melem/s 253.21 Melem/s 253.48 Melem/s]
                 change:
                        time:   [−0.0288% +0.2014% +0.4026%] (p = 0.08 > 0.05)
                        thrpt:  [−0.4010% −0.2010% +0.0288%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
location_info/same_continent
                        time:   [7.1844 ns 7.1918 ns 7.2006 ns]
                        thrpt:  [138.88 Melem/s 139.05 Melem/s 139.19 Melem/s]
                 change:
                        time:   [+0.0552% +0.3085% +0.6513%] (p = 0.03 < 0.05)
                        thrpt:  [−0.6471% −0.3075% −0.0552%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in estimated 5.0000 s (16B iteratiolocation_info/same_continent_cross
                        time:   [312.33 ps 312.53 ps 312.74 ps]
                        thrpt:  [3.1975 Gelem/s 3.1997 Gelem/s 3.2017 Gelem/s]
                 change:
                        time:   [−0.0437% +0.1161% +0.2834%] (p = 0.16 > 0.05)
                        thrpt:  [−0.2826% −0.1160% +0.0437%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
location_info/same_region
                        time:   [4.0682 ns 4.0740 ns 4.0809 ns]
                        thrpt:  [245.05 Melem/s 245.46 Melem/s 245.81 Melem/s]
                 change:
                        time:   [+0.6663% +1.0095% +1.3366%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3190% −0.9994% −0.6619%]
                        Change within noise threshold.

topology_hints/create   time:   [3.1707 ns 3.1808 ns 3.1916 ns]
                        thrpt:  [313.33 Melem/s 314.38 Melem/s 315.39 Melem/s]
                 change:
                        time:   [−0.4690% −0.0147% +0.4176%] (p = 0.95 > 0.05)
                        thrpt:  [−0.4158% +0.0147% +0.4712%]
                        No change in performance detected.
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in estimated 5.0000 s (16B iterationtopology_hints/connectivity_score
                        time:   [312.31 ps 312.52 ps 312.73 ps]
                        thrpt:  [3.1976 Gelem/s 3.1998 Gelem/s 3.2020 Gelem/s]
                 change:
                        time:   [−0.2313% −0.0566% +0.1198%] (p = 0.53 > 0.05)
                        thrpt:  [−0.1196% +0.0567% +0.2318%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in estimated 5.0000 s (8.0B iteratopology_hints/average_latency_empty
                        time:   [624.80 ps 625.25 ps 625.72 ps]
                        thrpt:  [1.5982 Gelem/s 1.5994 Gelem/s 1.6005 Gelem/s]
                 change:
                        time:   [−1.9142% −0.7075% +0.0658%] (p = 0.20 > 0.05)
                        thrpt:  [−0.0658% +0.7125% +1.9516%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estimated 5.0001 s (70M iteratiotopology_hints/average_latency_100
                        time:   [70.952 ns 71.111 ns 71.315 ns]
                        thrpt:  [14.022 Melem/s 14.063 Melem/s 14.094 Melem/s]
                 change:
                        time:   [+0.1284% +0.3774% +0.7180%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7129% −0.3760% −0.1282%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

nat_type/difficulty     time:   [312.22 ps 312.48 ps 312.77 ps]
                        thrpt:  [3.1973 Gelem/s 3.2002 Gelem/s 3.2028 Gelem/s]
                 change:
                        time:   [−1.5368% −0.5451% +0.0376%] (p = 0.24 > 0.05)
                        thrpt:  [−0.0376% +0.5481% +1.5608%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
nat_type/can_connect_direct
                        time:   [312.38 ps 312.64 ps 312.92 ps]
                        thrpt:  [3.1957 Gelem/s 3.1986 Gelem/s 3.2012 Gelem/s]
                 change:
                        time:   [−0.0709% +0.0972% +0.2686%] (p = 0.28 > 0.05)
                        thrpt:  [−0.2679% −0.0971% +0.0709%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
nat_type/can_connect_symmetric
                        time:   [312.29 ps 312.54 ps 312.80 ps]
                        thrpt:  [3.1969 Gelem/s 3.1996 Gelem/s 3.2022 Gelem/s]
                 change:
                        time:   [−1.0597% −0.3219% +0.1499%] (p = 0.39 > 0.05)
                        thrpt:  [−0.1497% +0.3229% +1.0710%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

node_metadata/create_simple
                        time:   [44.879 ns 44.959 ns 45.036 ns]
                        thrpt:  [22.204 Melem/s 22.243 Melem/s 22.282 Melem/s]
                 change:
                        time:   [−2.1376% −1.7987% −1.5077%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5308% +1.8317% +2.1843%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
node_metadata/create_full
                        time:   [357.78 ns 360.49 ns 363.61 ns]
                        thrpt:  [2.7502 Melem/s 2.7740 Melem/s 2.7950 Melem/s]
                 change:
                        time:   [−0.8433% −0.0580% +0.7099%] (p = 0.89 > 0.05)
                        thrpt:  [−0.7049% +0.0581% +0.8505%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
node_metadata/routing_score
                        time:   [2.8764 ns 2.8802 ns 2.8850 ns]
                        thrpt:  [346.62 Melem/s 347.20 Melem/s 347.66 Melem/s]
                 change:
                        time:   [−0.0350% +0.2396% +0.5509%] (p = 0.12 > 0.05)
                        thrpt:  [−0.5478% −0.2390% +0.0350%]
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  1 (1.00%) low mild
  8 (8.00%) high mild
  7 (7.00%) high severe
node_metadata/age       time:   [27.529 ns 27.546 ns 27.564 ns]
                        thrpt:  [36.280 Melem/s 36.303 Melem/s 36.326 Melem/s]
                 change:
                        time:   [−0.1861% +0.0210% +0.3029%] (p = 0.89 > 0.05)
                        thrpt:  [−0.3019% −0.0210% +0.1865%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
node_metadata/is_stale  time:   [25.876 ns 25.892 ns 25.909 ns]
                        thrpt:  [38.597 Melem/s 38.622 Melem/s 38.646 Melem/s]
                 change:
                        time:   [−1.9338% −0.7997% +0.0140%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0140% +0.8061% +1.9719%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
node_metadata/serialize time:   [746.81 ns 747.47 ns 748.11 ns]
                        thrpt:  [1.3367 Melem/s 1.3379 Melem/s 1.3390 Melem/s]
                 change:
                        time:   [+0.2799% +0.4257% +0.5704%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5672% −0.4239% −0.2791%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
node_metadata/deserialize
                        time:   [1.4071 µs 1.4082 µs 1.4093 µs]
                        thrpt:  [709.55 Kelem/s 710.15 Kelem/s 710.70 Kelem/s]
                 change:
                        time:   [−1.3975% −0.6626% −0.1912%] (p = 0.02 < 0.05)
                        thrpt:  [+0.1916% +0.6670% +1.4173%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe

metadata_query/match_status
                        time:   [3.4386 ns 3.4404 ns 3.4421 ns]
                        thrpt:  [290.52 Melem/s 290.67 Melem/s 290.81 Melem/s]
                 change:
                        time:   [−0.1532% +0.0198% +0.1906%] (p = 0.82 > 0.05)
                        thrpt:  [−0.1902% −0.0198% +0.1535%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
metadata_query/match_min_tier
                        time:   [3.4391 ns 3.4426 ns 3.4466 ns]
                        thrpt:  [290.14 Melem/s 290.47 Melem/s 290.77 Melem/s]
                 change:
                        time:   [−0.1818% +0.0074% +0.1898%] (p = 0.93 > 0.05)
                        thrpt:  [−0.1895% −0.0074% +0.1821%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
metadata_query/match_continent
                        time:   [11.359 ns 11.392 ns 11.428 ns]
                        thrpt:  [87.502 Melem/s 87.778 Melem/s 88.039 Melem/s]
                 change:
                        time:   [+1.0074% +1.3504% +1.7766%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7456% −1.3324% −0.9974%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
metadata_query/match_complex
                        time:   [10.717 ns 10.751 ns 10.789 ns]
                        thrpt:  [92.686 Melem/s 93.017 Melem/s 93.308 Melem/s]
                 change:
                        time:   [+0.5677% +1.1556% +1.6865%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6585% −1.1424% −0.5645%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
metadata_query/match_no_match
                        time:   [3.4345 ns 3.4365 ns 3.4385 ns]
                        thrpt:  [290.82 Melem/s 290.99 Melem/s 291.16 Melem/s]
                 change:
                        time:   [−1.1404% −0.4124% +0.1049%] (p = 0.25 > 0.05)
                        thrpt:  [−0.1048% +0.4141% +1.1535%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

metadata_store_basic/create
                        time:   [763.32 ns 764.73 ns 766.07 ns]
                        thrpt:  [1.3054 Melem/s 1.3076 Melem/s 1.3101 Melem/s]
                 change:
                        time:   [−0.2187% +0.0291% +0.2802%] (p = 0.82 > 0.05)
                        thrpt:  [−0.2795% −0.0291% +0.2192%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimated 5.0031 s (2.5M iterationsmetadata_store_basic/upsert_new
                        time:   [2.1792 µs 2.2028 µs 2.2225 µs]
                        thrpt:  [449.94 Kelem/s 453.97 Kelem/s 458.88 Kelem/s]
                 change:
                        time:   [−12.825% −10.198% −7.4345%] (p = 0.00 < 0.05)
                        thrpt:  [+8.0317% +11.356% +14.712%]
                        Performance has improved.
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in estimated 5.0024 s (4.2M iterametadata_store_basic/upsert_existing
                        time:   [1.1489 µs 1.1509 µs 1.1529 µs]
                        thrpt:  [867.40 Kelem/s 868.87 Kelem/s 870.39 Kelem/s]
                 change:
                        time:   [−0.2437% +0.0419% +0.3082%] (p = 0.77 > 0.05)
                        thrpt:  [−0.3073% −0.0419% +0.2443%]
                        No change in performance detected.
metadata_store_basic/get
                        time:   [24.702 ns 25.168 ns 25.646 ns]
                        thrpt:  [38.992 Melem/s 39.733 Melem/s 40.483 Melem/s]
                 change:
                        time:   [−8.1885% −5.7768% −3.1402%] (p = 0.00 < 0.05)
                        thrpt:  [+3.2420% +6.1309% +8.9188%]
                        Performance has improved.
metadata_store_basic/get_miss
                        time:   [25.161 ns 25.679 ns 26.205 ns]
                        thrpt:  [38.161 Melem/s 38.943 Melem/s 39.745 Melem/s]
                 change:
                        time:   [−4.1528% −1.6044% +1.1206%] (p = 0.25 > 0.05)
                        thrpt:  [−1.1082% +1.6306% +4.3327%]
                        No change in performance detected.
metadata_store_basic/len
                        time:   [200.60 ns 200.72 ns 200.85 ns]
                        thrpt:  [4.9788 Melem/s 4.9820 Melem/s 4.9851 Melem/s]
                 change:
                        time:   [−0.3158% −0.1561% +0.0069%] (p = 0.06 > 0.05)
                        thrpt:  [−0.0069% +0.1563% +0.3168%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 20.8s, or reduce sample count to 20.
metadata_store_basic/stats
                        time:   [206.10 ms 206.84 ms 207.52 ms]
                        thrpt:  [4.8189  elem/s 4.8346  elem/s 4.8520  elem/s]
                 change:
                        time:   [−5.8660% −5.4448% −5.0981%] (p = 0.00 < 0.05)
                        thrpt:  [+5.3720% +5.7583% +6.2316%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  7 (7.00%) low mild

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in estimated 6.5248 s (20k iteratmetadata_store_query/query_by_status
                        time:   [307.80 µs 323.46 µs 339.42 µs]
                        thrpt:  [2.9462 Kelem/s 3.0916 Kelem/s 3.2489 Kelem/s]
                 change:
                        time:   [+44.675% +51.494% +59.413%] (p = 0.00 < 0.05)
                        thrpt:  [−37.270% −33.991% −30.880%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples in estimated 5.2339 s (35k itemetadata_store_query/query_by_continent
                        time:   [147.63 µs 147.97 µs 148.32 µs]
                        thrpt:  [6.7420 Kelem/s 6.7582 Kelem/s 6.7737 Kelem/s]
                 change:
                        time:   [−0.1051% +0.3235% +0.7954%] (p = 0.16 > 0.05)
                        thrpt:  [−0.7891% −0.3224% +0.1052%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in estimated 5.3934 s (10k iteratiometadata_store_query/query_by_tier
                        time:   [480.56 µs 499.57 µs 518.92 µs]
                        thrpt:  [1.9271 Kelem/s 2.0017 Kelem/s 2.0809 Kelem/s]
                 change:
                        time:   [+29.031% +34.651% +40.166%] (p = 0.00 < 0.05)
                        thrpt:  [−28.656% −25.734% −22.499%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samples in estimated 5.9140 s (10k imetadata_store_query/query_accepting_work
                        time:   [569.29 µs 588.24 µs 606.30 µs]
                        thrpt:  [1.6494 Kelem/s 1.7000 Kelem/s 1.7566 Kelem/s]
                 change:
                        time:   [+28.373% +34.933% +41.377%] (p = 0.00 < 0.05)
                        thrpt:  [−29.267% −25.889% −22.102%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples in estimated 5.2134 s (10k iterametadata_store_query/query_with_limit
                        time:   [497.01 µs 520.23 µs 542.85 µs]
                        thrpt:  [1.8421 Kelem/s 1.9222 Kelem/s 2.0120 Kelem/s]
                 change:
                        time:   [+27.511% +34.101% +40.652%] (p = 0.00 < 0.05)
                        thrpt:  [−28.903% −25.429% −21.575%]
                        Performance has regressed.
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in estimated 5.7052 s (20k iteratiometadata_store_query/query_complex
                        time:   [281.73 µs 282.50 µs 283.32 µs]
                        thrpt:  [3.5296 Kelem/s 3.5398 Kelem/s 3.5495 Kelem/s]
                 change:
                        time:   [+0.4815% +1.1500% +1.7688%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7380% −1.1369% −0.4792%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samples in estimated 6.6641 s (20k itmetadata_store_spatial/find_nearby_100km
                        time:   [329.78 µs 331.16 µs 332.71 µs]
                        thrpt:  [3.0056 Kelem/s 3.0197 Kelem/s 3.0324 Kelem/s]
                 change:
                        time:   [−0.8225% −0.1572% +0.4908%] (p = 0.64 > 0.05)
                        thrpt:  [−0.4884% +0.1574% +0.8293%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samples in estimated 6.1418 s (15k imetadata_store_spatial/find_nearby_1000km
                        time:   [399.68 µs 401.17 µs 402.95 µs]
                        thrpt:  [2.4817 Kelem/s 2.4927 Kelem/s 2.5020 Kelem/s]
                 change:
                        time:   [−0.7154% +0.1424% +1.0493%] (p = 0.76 > 0.05)
                        thrpt:  [−1.0384% −0.1422% +0.7206%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samples in estimated 5.0056 s (10k imetadata_store_spatial/find_nearby_5000km
                        time:   [472.44 µs 491.35 µs 511.11 µs]
                        thrpt:  [1.9565 Kelem/s 2.0352 Kelem/s 2.1167 Kelem/s]
                 change:
                        time:   [+5.3291% +9.8914% +14.805%] (p = 0.00 < 0.05)
                        thrpt:  [−12.896% −9.0011% −5.0595%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 samples in estimated 5.3282 s (25metadata_store_spatial/find_best_for_routing
                        time:   [208.72 µs 209.30 µs 209.92 µs]
                        thrpt:  [4.7638 Kelem/s 4.7778 Kelem/s 4.7911 Kelem/s]
                 change:
                        time:   [−15.551% −13.762% −12.044%] (p = 0.00 < 0.05)
                        thrpt:  [+13.694% +15.959% +18.415%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in estimated 5.3614 s (10k iteratiometadata_store_spatial/find_relays
                        time:   [526.12 µs 576.84 µs 652.32 µs]
                        thrpt:  [1.5330 Kelem/s 1.7336 Kelem/s 1.9007 Kelem/s]
                 change:
                        time:   [+7.2247% +14.225% +23.933%] (p = 0.00 < 0.05)
                        thrpt:  [−19.311% −12.454% −6.7379%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samples in estimated 5.0023 s (263k imetadata_store_scaling/query_status/1000
                        time:   [19.022 µs 19.037 µs 19.053 µs]
                        thrpt:  [52.486 Kelem/s 52.530 Kelem/s 52.572 Kelem/s]
                 change:
                        time:   [+1.9176% +2.1078% +2.3143%] (p = 0.00 < 0.05)
                        thrpt:  [−2.2620% −2.0643% −1.8815%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0176 s (232k metadata_store_scaling/query_complex/1000
                        time:   [21.274 µs 21.363 µs 21.459 µs]
                        thrpt:  [46.601 Kelem/s 46.809 Kelem/s 47.005 Kelem/s]
                 change:
                        time:   [+0.8459% +1.1289% +1.4042%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3847% −1.1163% −0.8388%]
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  11 (11.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples in estimated 5.1324 s (96k itemetadata_store_scaling/find_nearby/1000
                        time:   [53.440 µs 53.555 µs 53.693 µs]
                        thrpt:  [18.624 Kelem/s 18.673 Kelem/s 18.712 Kelem/s]
                 change:
                        time:   [+0.1645% +0.6394% +1.0784%] (p = 0.00 < 0.05)
                        thrpt:  [−1.0669% −0.6354% −0.1642%]
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samples in estimated 5.4294 s (56k itmetadata_store_scaling/query_status/5000
                        time:   [97.796 µs 97.882 µs 97.976 µs]
                        thrpt:  [10.207 Kelem/s 10.216 Kelem/s 10.225 Kelem/s]
                 change:
                        time:   [−3.6215% −2.4221% −1.5839%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6094% +2.4822% +3.7576%]
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samples in estimated 5.4163 s (45k imetadata_store_scaling/query_complex/5000
                        time:   [119.13 µs 119.22 µs 119.33 µs]
                        thrpt:  [8.3802 Kelem/s 8.3877 Kelem/s 8.3942 Kelem/s]
                 change:
                        time:   [−1.4559% −1.1530% −0.8726%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8802% +1.1664% +1.4774%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples in estimated 6.1209 s (20k itemetadata_store_scaling/find_nearby/5000
                        time:   [385.35 µs 409.65 µs 432.76 µs]
                        thrpt:  [2.3107 Kelem/s 2.4411 Kelem/s 2.5950 Kelem/s]
                 change:
                        time:   [+43.409% +48.754% +53.514%] (p = 0.00 < 0.05)
                        thrpt:  [−34.860% −32.775% −30.269%]
                        Performance has regressed.
Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samples in estimated 5.5601 s (20k imetadata_store_scaling/query_status/10000
                        time:   [330.63 µs 367.80 µs 417.26 µs]
                        thrpt:  [2.3966 Kelem/s 2.7188 Kelem/s 3.0245 Kelem/s]
                 change:
                        time:   [+27.439% +39.456% +53.325%] (p = 0.00 < 0.05)
                        thrpt:  [−34.779% −28.293% −21.531%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samples in estimated 6.7973 s (15k metadata_store_scaling/query_complex/10000
                        time:   [356.87 µs 380.30 µs 403.62 µs]
                        thrpt:  [2.4776 Kelem/s 2.6295 Kelem/s 2.8021 Kelem/s]
                 change:
                        time:   [+27.625% +33.736% +39.797%] (p = 0.00 < 0.05)
                        thrpt:  [−28.468% −25.226% −21.645%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samples in estimated 5.8883 s (10k itmetadata_store_scaling/find_nearby/10000
                        time:   [556.68 µs 576.41 µs 598.66 µs]
                        thrpt:  [1.6704 Kelem/s 1.7349 Kelem/s 1.7964 Kelem/s]
                 change:
                        time:   [+0.6220% +3.5339% +6.6783%] (p = 0.03 < 0.05)
                        thrpt:  [−6.2602% −3.4133% −0.6181%]
                        Change within noise threshold.
Found 31 outliers among 100 measurements (31.00%)
  8 (8.00%) low severe
  1 (1.00%) low mild
  22 (22.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samples in estimated 5.1127 s (2200 metadata_store_scaling/query_status/50000
                        time:   [2.3248 ms 2.3437 ms 2.3640 ms]
                        thrpt:  [423.02  elem/s 426.68  elem/s 430.15  elem/s]
                 change:
                        time:   [−30.083% −28.455% −26.762%] (p = 0.00 < 0.05)
                        thrpt:  [+36.541% +39.772% +43.026%]
                        Performance has improved.
Found 25 outliers among 100 measurements (25.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  23 (23.00%) high severe
Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samples in estimated 5.0168 s (1900metadata_store_scaling/query_complex/50000
                        time:   [2.6560 ms 2.6917 ms 2.7275 ms]
                        thrpt:  [366.64  elem/s 371.51  elem/s 376.51  elem/s]
                 change:
                        time:   [−22.560% −20.573% −18.549%] (p = 0.00 < 0.05)
                        thrpt:  [+22.773% +25.901% +29.133%]
                        Performance has improved.
Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samples in estimated 5.1504 s (1500 imetadata_store_scaling/find_nearby/50000
                        time:   [3.2898 ms 3.3191 ms 3.3495 ms]
                        thrpt:  [298.56  elem/s 301.29  elem/s 303.97  elem/s]
                 change:
                        time:   [−9.3049% −7.1045% −4.9191%] (p = 0.00 < 0.05)
                        thrpt:  [+5.1736% +7.6479% +10.260%]
                        Performance has improved.

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 samples in estimated 5.2959 s (31metadata_store_concurrent/concurrent_upsert/4
                        time:   [1.6803 ms 1.7055 ms 1.7317 ms]
                        thrpt:  [1.1550 Melem/s 1.1727 Melem/s 1.1902 Melem/s]
                 change:
                        time:   [−3.7208% −0.9513% +1.2012%] (p = 0.52 > 0.05)
                        thrpt:  [−1.1869% +0.9604% +3.8646%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low severe
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.1s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 samples in estimated 6.0980 s (20 metadata_store_concurrent/concurrent_query/4
                        time:   [299.19 ms 308.62 ms 317.81 ms]
                        thrpt:  [6.2931 Kelem/s 6.4804 Kelem/s 6.6846 Kelem/s]
                 change:
                        time:   [−24.326% −20.379% −15.910%] (p = 0.00 < 0.05)
                        thrpt:  [+18.920% +25.596% +32.146%]
                        Performance has improved.
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.3s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 10.267 s (20 metadata_store_concurrent/concurrent_mixed/4
                        time:   [618.29 ms 708.24 ms 780.94 ms]
                        thrpt:  [2.5610 Kelem/s 2.8239 Kelem/s 3.2347 Kelem/s]
                 change:
                        time:   [+79.723% +105.72% +127.49%] (p = 0.00 < 0.05)
                        thrpt:  [−56.042% −51.389% −44.359%]
                        Performance has regressed.
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) low severe
  1 (5.00%) low mild
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 samples in estimated 5.0855 s (14metadata_store_concurrent/concurrent_upsert/8
                        time:   [3.0379 ms 3.0537 ms 3.0839 ms]
                        thrpt:  [1.2971 Melem/s 1.3099 Melem/s 1.3167 Melem/s]
                 change:
                        time:   [−1.8053% +1.2190% +4.3892%] (p = 0.48 > 0.05)
                        thrpt:  [−4.2046% −1.2043% +1.8385%]
                        No change in performance detected.
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.0s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 samples in estimated 15.977 s (20 metadata_store_concurrent/concurrent_query/8
                        time:   [788.96 ms 796.16 ms 803.80 ms]
                        thrpt:  [4.9764 Kelem/s 5.0241 Kelem/s 5.0700 Kelem/s]
                 change:
                        time:   [−1.9886% −0.8868% +0.3412%] (p = 0.14 > 0.05)
                        thrpt:  [−0.3400% +0.8947% +2.0290%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 19.3s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 19.336 s (20 metadata_store_concurrent/concurrent_mixed/8
                        time:   [951.94 ms 984.62 ms 1.0292 s]
                        thrpt:  [3.8865 Kelem/s 4.0625 Kelem/s 4.2020 Kelem/s]
                 change:
                        time:   [+7.6644% +11.529% +16.158%] (p = 0.00 < 0.05)
                        thrpt:  [−13.910% −10.337% −7.1188%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 samples in estimated 6.4983 s (8metadata_store_concurrent/concurrent_upsert/16
                        time:   [7.7239 ms 7.7819 ms 7.8502 ms]
                        thrpt:  [1.0191 Melem/s 1.0280 Melem/s 1.0358 Melem/s]
                 change:
                        time:   [+10.477% +11.451% +12.359%] (p = 0.00 < 0.05)
                        thrpt:  [−11.000% −10.275% −9.4831%]
                        Performance has regressed.
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 31.5s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 samples in estimated 31.509 s (20metadata_store_concurrent/concurrent_query/16
                        time:   [1.5039 s 1.5269 s 1.5526 s]
                        thrpt:  [5.1527 Kelem/s 5.2394 Kelem/s 5.3196 Kelem/s]
                 change:
                        time:   [+4.8226% +6.5096% +8.5025%] (p = 0.00 < 0.05)
                        thrpt:  [−7.8362% −6.1118% −4.6007%]
                        Performance has regressed.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 34.6s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 34.612 s (20metadata_store_concurrent/concurrent_mixed/16
                        time:   [1.7474 s 1.7517 s 1.7559 s]
                        thrpt:  [4.5562 Kelem/s 4.5670 Kelem/s 4.5781 Kelem/s]
                 change:
                        time:   [+2.1009% +2.5272% +2.8859%] (p = 0.00 < 0.05)
                        thrpt:  [−2.8050% −2.4649% −2.0577%]
                        Performance has regressed.
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) low mild
  1 (5.00%) high mild

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimated 5.0003metadata_store_versioning/update_versioned_success
                        time:   [176.60 ns 179.04 ns 181.93 ns]
                        thrpt:  [5.4967 Melem/s 5.5852 Melem/s 5.6625 Melem/s]
                 change:
                        time:   [−1.3301% −0.7331% −0.0334%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0334% +0.7385% +1.3480%]
                        Change within noise threshold.
Found 19 outliers among 100 measurements (19.00%)
  19 (19.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimated 5.000metadata_store_versioning/update_versioned_conflict
                        time:   [176.27 ns 178.78 ns 181.81 ns]
                        thrpt:  [5.5002 Melem/s 5.5934 Melem/s 5.6731 Melem/s]
                 change:
                        time:   [−0.7566% −0.1778% +0.4451%] (p = 0.59 > 0.05)
                        thrpt:  [−0.4431% +0.1781% +0.7624%]
                        No change in performance detected.
Found 37 outliers among 100 measurements (37.00%)
  18 (18.00%) low mild
  19 (19.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estimated 5.0000 s (1.5B iteratioschema_validation/validate_string
                        time:   [3.4197 ns 3.4212 ns 3.4230 ns]
                        thrpt:  [292.14 Melem/s 292.29 Melem/s 292.43 Melem/s]
                 change:
                        time:   [−2.1262% −1.9403% −1.7881%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8206% +1.9787% +2.1723%]
                        Performance has improved.
Benchmarking schema_validation/validate_integer: Collecting 100 samples in estimated 5.0000 s (1.5B iteratischema_validation/validate_integer
                        time:   [3.5286 ns 3.5844 ns 3.7073 ns]
                        thrpt:  [269.74 Melem/s 278.99 Melem/s 283.40 Melem/s]
                 change:
                        time:   [+0.4846% +9.5831% +26.898%] (p = 0.22 > 0.05)
                        thrpt:  [−21.196% −8.7450% −0.4823%]
                        No change in performance detected.
Found 33 outliers among 100 measurements (33.00%)
  16 (16.00%) low severe
  3 (3.00%) low mild
  8 (8.00%) high mild
  6 (6.00%) high severe
Benchmarking schema_validation/validate_object: Collecting 100 samples in estimated 5.0002 s (63M iterationschema_validation/validate_object
                        time:   [78.885 ns 78.978 ns 79.077 ns]
                        thrpt:  [12.646 Melem/s 12.662 Melem/s 12.677 Melem/s]
                 change:
                        time:   [+2.8751% +3.1230% +3.3622%] (p = 0.00 < 0.05)
                        thrpt:  [−3.2529% −3.0285% −2.7947%]
                        Performance has regressed.
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in estimated 5.0000 s (136M iteratschema_validation/validate_array_10
                        time:   [35.864 ns 35.899 ns 35.945 ns]
                        thrpt:  [27.820 Melem/s 27.856 Melem/s 27.883 Melem/s]
                 change:
                        time:   [−0.7842% −0.5288% −0.2669%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2676% +0.5316% +0.7904%]
                        Change within noise threshold.
Found 20 outliers among 100 measurements (20.00%)
  20 (20.00%) high severe
Benchmarking schema_validation/validate_complex: Collecting 100 samples in estimated 5.0008 s (24M iteratioschema_validation/validate_complex
                        time:   [205.16 ns 206.31 ns 207.81 ns]
                        thrpt:  [4.8120 Melem/s 4.8471 Melem/s 4.8742 Melem/s]
                 change:
                        time:   [+3.9624% +5.9818% +8.2908%] (p = 0.00 < 0.05)
                        thrpt:  [−7.6561% −5.6442% −3.8114%]
                        Performance has regressed.

endpoint_matching/match_success
                        time:   [169.15 ns 169.29 ns 169.49 ns]
                        thrpt:  [5.9001 Melem/s 5.9071 Melem/s 5.9121 Melem/s]
                 change:
                        time:   [−3.9855% −2.4952% −1.4731%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4951% +2.5590% +4.1510%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
endpoint_matching/match_failure
                        time:   [170.91 ns 170.97 ns 171.03 ns]
                        thrpt:  [5.8469 Melem/s 5.8490 Melem/s 5.8509 Melem/s]
                 change:
                        time:   [−0.4883% −0.3358% −0.2021%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2025% +0.3370% +0.4907%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in estimated 5.0006 s (13M iteratiendpoint_matching/match_multi_param
                        time:   [372.20 ns 372.35 ns 372.53 ns]
                        thrpt:  [2.6844 Melem/s 2.6857 Melem/s 2.6867 Melem/s]
                 change:
                        time:   [+0.2188% +0.4939% +0.8073%] (p = 0.00 < 0.05)
                        thrpt:  [−0.8008% −0.4915% −0.2183%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high severe

api_version/is_compatible_with
                        time:   [316.92 ps 343.91 ps 403.58 ps]
                        thrpt:  [2.4778 Gelem/s 2.9078 Gelem/s 3.1554 Gelem/s]
                 change:
                        time:   [+0.3908% +6.2741% +17.716%] (p = 0.20 > 0.05)
                        thrpt:  [−15.050% −5.9037% −0.3892%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
api_version/parse       time:   [36.424 ns 36.435 ns 36.445 ns]
                        thrpt:  [27.438 Melem/s 27.446 Melem/s 27.455 Melem/s]
                 change:
                        time:   [+2.2251% +2.4070% +2.5645%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5004% −2.3504% −2.1766%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
api_version/to_string   time:   [45.121 ns 45.206 ns 45.265 ns]
                        thrpt:  [22.092 Melem/s 22.121 Melem/s 22.163 Melem/s]
                 change:
                        time:   [+3.6877% +3.8559% +4.0024%] (p = 0.00 < 0.05)
                        thrpt:  [−3.8483% −3.7127% −3.5566%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild

api_schema/create       time:   [1.2295 µs 1.2309 µs 1.2323 µs]
                        thrpt:  [811.52 Kelem/s 812.40 Kelem/s 813.36 Kelem/s]
                 change:
                        time:   [−0.1220% +0.0900% +0.2834%] (p = 0.39 > 0.05)
                        thrpt:  [−0.2826% −0.0899% +0.1221%]
                        No change in performance detected.
api_schema/serialize    time:   [1.8915 µs 1.8966 µs 1.9025 µs]
                        thrpt:  [525.61 Kelem/s 527.25 Kelem/s 528.67 Kelem/s]
                 change:
                        time:   [−1.3081% −0.6259% −0.1340%] (p = 0.02 < 0.05)
                        thrpt:  [+0.1341% +0.6298% +1.3255%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
api_schema/deserialize  time:   [5.1199 µs 5.1445 µs 5.1715 µs]
                        thrpt:  [193.37 Kelem/s 194.38 Kelem/s 195.32 Kelem/s]
                 change:
                        time:   [−0.7094% −0.1028% +0.4546%] (p = 0.74 > 0.05)
                        thrpt:  [−0.4525% +0.1029% +0.7145%]
                        No change in performance detected.
api_schema/find_endpoint
                        time:   [197.82 ns 198.24 ns 198.95 ns]
                        thrpt:  [5.0263 Melem/s 5.0443 Melem/s 5.0550 Melem/s]
                 change:
                        time:   [+0.1352% +0.3206% +0.5318%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5290% −0.3195% −0.1350%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
api_schema/endpoints_by_tag
                        time:   [57.347 ns 57.460 ns 57.565 ns]
                        thrpt:  [17.372 Melem/s 17.403 Melem/s 17.438 Melem/s]
                 change:
                        time:   [+0.1203% +0.3206% +0.5052%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5027% −0.3196% −0.1202%]
                        Change within noise threshold.

Benchmarking request_validation/validate_full_request: Collecting 100 samples in estimated 5.0000 s (65M itrequest_validation/validate_full_request
                        time:   [74.387 ns 74.581 ns 74.799 ns]
                        thrpt:  [13.369 Melem/s 13.408 Melem/s 13.443 Melem/s]
                 change:
                        time:   [+4.5094% +4.8331% +5.1420%] (p = 0.00 < 0.05)
                        thrpt:  [−4.8906% −4.6103% −4.3148%]
                        Performance has regressed.
Benchmarking request_validation/validate_path_only: Collecting 100 samples in estimated 5.0000 s (227M iterrequest_validation/validate_path_only
                        time:   [22.284 ns 22.363 ns 22.432 ns]
                        thrpt:  [44.579 Melem/s 44.716 Melem/s 44.875 Melem/s]
                 change:
                        time:   [+3.9291% +4.3399% +4.7407%] (p = 0.00 < 0.05)
                        thrpt:  [−4.5261% −4.1593% −3.7806%]
                        Performance has regressed.
Found 18 outliers among 100 measurements (18.00%)
  10 (10.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe

api_registry_basic/create
                        time:   [424.77 ns 426.88 ns 428.93 ns]
                        thrpt:  [2.3314 Melem/s 2.3426 Melem/s 2.3542 Melem/s]
                 change:
                        time:   [+2.9507% +3.3739% +3.7640%] (p = 0.00 < 0.05)
                        thrpt:  [−3.6275% −3.2638% −2.8662%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  14 (14.00%) low severe
  3 (3.00%) low mild
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimated 5.0102 s (1.3M iterationsapi_registry_basic/register_new
                        time:   [3.7637 µs 3.8187 µs 3.8793 µs]
                        thrpt:  [257.78 Kelem/s 261.87 Kelem/s 265.70 Kelem/s]
                 change:
                        time:   [−15.556% −13.987% −12.309%] (p = 0.00 < 0.05)
                        thrpt:  [+14.036% +16.262% +18.421%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
api_registry_basic/get  time:   [27.883 ns 28.809 ns 29.714 ns]
                        thrpt:  [33.654 Melem/s 34.712 Melem/s 35.865 Melem/s]
                 change:
                        time:   [+2.5201% +5.8901% +9.4152%] (p = 0.00 < 0.05)
                        thrpt:  [−8.6050% −5.5625% −2.4582%]
                        Performance has regressed.
api_registry_basic/len  time:   [199.31 ns 199.34 ns 199.37 ns]
                        thrpt:  [5.0157 Melem/s 5.0166 Melem/s 5.0174 Melem/s]
                 change:
                        time:   [−0.4818% −0.3632% −0.2642%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2649% +0.3645% +0.4841%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 25.9s, or reduce sample count to 10.
api_registry_basic/stats
                        time:   [241.36 ms 248.70 ms 258.99 ms]
                        thrpt:  [3.8612  elem/s 4.0209  elem/s 4.1432  elem/s]
                 change:
                        time:   [+0.0105% +3.1407% +7.7078%] (p = 0.10 > 0.05)
                        thrpt:  [−7.1562% −3.0450% −0.0105%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estimated 5.0398 s (45k iterationsapi_registry_query/query_by_name
                        time:   [109.45 µs 110.27 µs 111.07 µs]
                        thrpt:  [9.0031 Kelem/s 9.0685 Kelem/s 9.1367 Kelem/s]
                 change:
                        time:   [+5.6481% +6.3560% +7.1005%] (p = 0.00 < 0.05)
                        thrpt:  [−6.6298% −5.9762% −5.3461%]
                        Performance has regressed.
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low severe
  6 (6.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
api_registry_query/query_by_tag
                        time:   [652.84 µs 696.19 µs 740.22 µs]
                        thrpt:  [1.3509 Kelem/s 1.4364 Kelem/s 1.5318 Kelem/s]
                 change:
                        time:   [+9.8343% +15.849% +22.395%] (p = 0.00 < 0.05)
                        thrpt:  [−18.297% −13.681% −8.9537%]
                        Performance has regressed.
Benchmarking api_registry_query/query_with_version: Collecting 100 samples in estimated 5.0473 s (86k iteraapi_registry_query/query_with_version
                        time:   [58.606 µs 58.627 µs 58.649 µs]
                        thrpt:  [17.050 Kelem/s 17.057 Kelem/s 17.063 Kelem/s]
                 change:
                        time:   [−0.9749% −0.8517% −0.7525%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7582% +0.8590% +0.9845%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in estimated 5.1346 s (1600 iteratapi_registry_query/find_by_endpoint
                        time:   [3.1690 ms 3.2279 ms 3.2857 ms]
                        thrpt:  [304.35  elem/s 309.80  elem/s 315.56  elem/s]
                 change:
                        time:   [+1.8469% +4.1011% +6.3244%] (p = 0.00 < 0.05)
                        thrpt:  [−5.9482% −3.9396% −1.8134%]
                        Performance has regressed.
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in estimated 5.2037 s (81k iteratioapi_registry_query/find_compatible
                        time:   [64.676 µs 65.020 µs 65.377 µs]
                        thrpt:  [15.296 Kelem/s 15.380 Kelem/s 15.462 Kelem/s]
                 change:
                        time:   [−5.2135% −4.8507% −4.5001%] (p = 0.00 < 0.05)
                        thrpt:  [+4.7121% +5.0980% +5.5002%]
                        Performance has improved.
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) high mild
  10 (10.00%) high severe

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples in estimated 5.0184 s (656k itapi_registry_scaling/query_by_name/1000
                        time:   [7.6635 µs 7.6719 µs 7.6800 µs]
                        thrpt:  [130.21 Kelem/s 130.35 Kelem/s 130.49 Kelem/s]
                 change:
                        time:   [−4.5388% −4.2989% −4.0647%] (p = 0.00 < 0.05)
                        thrpt:  [+4.2369% +4.4920% +4.7546%]
                        Performance has improved.
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples in estimated 5.1591 s (106k iteapi_registry_scaling/query_by_tag/1000
                        time:   [48.403 µs 48.651 µs 48.885 µs]
                        thrpt:  [20.456 Kelem/s 20.555 Kelem/s 20.660 Kelem/s]
                 change:
                        time:   [−4.2390% −3.7508% −3.2492%] (p = 0.00 < 0.05)
                        thrpt:  [+3.3583% +3.8970% +4.4266%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples in estimated 5.1134 s (106k itapi_registry_scaling/query_by_name/5000
                        time:   [48.409 µs 48.533 µs 48.647 µs]
                        thrpt:  [20.556 Kelem/s 20.605 Kelem/s 20.657 Kelem/s]
                 change:
                        time:   [−0.7731% −0.3523% +0.0284%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0284% +0.3535% +0.7791%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples in estimated 6.9159 s (15k iterapi_registry_scaling/query_by_tag/5000
                        time:   [381.58 µs 409.45 µs 438.33 µs]
                        thrpt:  [2.2814 Kelem/s 2.4423 Kelem/s 2.6207 Kelem/s]
                 change:
                        time:   [+63.644% +68.838% +73.755%] (p = 0.00 < 0.05)
                        thrpt:  [−42.448% −40.772% −38.892%]
                        Performance has regressed.
Found 25 outliers among 100 measurements (25.00%)
  16 (16.00%) low severe
  1 (1.00%) low mild
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samples in estimated 5.2469 s (50k itapi_registry_scaling/query_by_name/10000
                        time:   [93.694 µs 94.173 µs 94.705 µs]
                        thrpt:  [10.559 Kelem/s 10.619 Kelem/s 10.673 Kelem/s]
                 change:
                        time:   [−8.3903% −7.8303% −7.3018%] (p = 0.00 < 0.05)
                        thrpt:  [+7.8770% +8.4955% +9.1587%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples in estimated 6.2204 s (15k iteapi_registry_scaling/query_by_tag/10000
                        time:   [389.25 µs 400.87 µs 414.46 µs]
                        thrpt:  [2.4128 Kelem/s 2.4946 Kelem/s 2.5690 Kelem/s]
                 change:
                        time:   [−27.223% −24.856% −22.198%] (p = 0.00 < 0.05)
                        thrpt:  [+28.531% +33.078% +37.405%]
                        Performance has improved.

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 20.0s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 samples in estimated 19.970 s (20 itapi_registry_concurrent/concurrent_query/4
                        time:   [759.53 ms 803.03 ms 856.69 ms]
                        thrpt:  [2.3346 Kelem/s 2.4906 Kelem/s 2.6332 Kelem/s]
                 change:
                        time:   [−6.6871% −1.1016% +5.7525%] (p = 0.73 > 0.05)
                        thrpt:  [−5.4396% +1.1138% +7.1663%]
                        No change in performance detected.
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) high severe
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.2s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 9.2406 s (20 itapi_registry_concurrent/concurrent_mixed/4
                        time:   [487.07 ms 502.99 ms 516.14 ms]
                        thrpt:  [3.8749 Kelem/s 3.9762 Kelem/s 4.1062 Kelem/s]
                 change:
                        time:   [−4.3350% −1.2425% +1.8328%] (p = 0.46 > 0.05)
                        thrpt:  [−1.7999% +1.2581% +4.5314%]
                        No change in performance detected.
Found 4 outliers among 20 measurements (20.00%)
  4 (20.00%) low severe
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 22.2s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 samples in estimated 22.237 s (20 itapi_registry_concurrent/concurrent_query/8
                        time:   [1.1144 s 1.1196 s 1.1252 s]
                        thrpt:  [3.5549 Kelem/s 3.5728 Kelem/s 3.5895 Kelem/s]
                 change:
                        time:   [+1.6955% +2.2821% +2.8726%] (p = 0.00 < 0.05)
                        thrpt:  [−2.7924% −2.2312% −1.6672%]
                        Performance has regressed.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 19.1s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 19.098 s (20 itapi_registry_concurrent/concurrent_mixed/8
                        time:   [959.50 ms 971.16 ms 983.50 ms]
                        thrpt:  [4.0671 Kelem/s 4.1188 Kelem/s 4.1688 Kelem/s]
                 change:
                        time:   [+1.7597% +3.8118% +6.1857%] (p = 0.00 < 0.05)
                        thrpt:  [−5.8254% −3.6719% −1.7292%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 37.3s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samples in estimated 37.256 s (20 iapi_registry_concurrent/concurrent_query/16
                        time:   [1.8596 s 1.8655 s 1.8729 s]
                        thrpt:  [4.2715 Kelem/s 4.2883 Kelem/s 4.3019 Kelem/s]
                 change:
                        time:   [+3.6474% +4.0596% +4.5374%] (p = 0.00 < 0.05)
                        thrpt:  [−4.3404% −3.9012% −3.5190%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 34.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 34.356 s (20 iapi_registry_concurrent/concurrent_mixed/16
                        time:   [1.7502 s 1.7551 s 1.7605 s]
                        thrpt:  [4.5441 Kelem/s 4.5581 Kelem/s 4.5710 Kelem/s]
                 change:
                        time:   [+6.4777% +7.0149% +7.5744%] (p = 0.00 < 0.05)
                        thrpt:  [−7.0411% −6.5551% −6.0836%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

compare_op/eq           time:   [1.9387 ns 1.9394 ns 1.9402 ns]
                        thrpt:  [515.40 Melem/s 515.63 Melem/s 515.81 Melem/s]
                 change:
                        time:   [−2.2591% −2.1547% −2.0641%] (p = 0.00 < 0.05)
                        thrpt:  [+2.1076% +2.2022% +2.3113%]
                        Performance has improved.
Found 14 outliers among 100 measurements (14.00%)
  6 (6.00%) high mild
  8 (8.00%) high severe
compare_op/gt           time:   [1.2408 ns 1.2411 ns 1.2415 ns]
                        thrpt:  [805.46 Melem/s 805.72 Melem/s 805.95 Melem/s]
                 change:
                        time:   [−1.1244% −0.7984% −0.5416%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5446% +0.8048% +1.1372%]
                        Change within noise threshold.
Found 14 outliers among 100 measurements (14.00%)
  4 (4.00%) high mild
  10 (10.00%) high severe
compare_op/contains_string
                        time:   [25.127 ns 25.141 ns 25.161 ns]
                        thrpt:  [39.744 Melem/s 39.775 Melem/s 39.798 Melem/s]
                 change:
                        time:   [−0.5519% −0.4218% −0.3113%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3123% +0.4236% +0.5549%]
                        Change within noise threshold.
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  10 (10.00%) high severe
compare_op/in_array     time:   [6.8268 ns 6.8295 ns 6.8325 ns]
                        thrpt:  [146.36 Melem/s 146.42 Melem/s 146.48 Melem/s]
                 change:
                        time:   [−0.5716% −0.4524% −0.3496%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3508% +0.4545% +0.5749%]
                        Change within noise threshold.
Found 20 outliers among 100 measurements (20.00%)
  2 (2.00%) low mild
  4 (4.00%) high mild
  14 (14.00%) high severe

condition/simple        time:   [50.244 ns 50.289 ns 50.340 ns]
                        thrpt:  [19.865 Melem/s 19.885 Melem/s 19.903 Melem/s]
                 change:
                        time:   [−1.9993% −1.7911% −1.6000%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6260% +1.8237% +2.0401%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
condition/nested_field  time:   [557.97 ns 559.23 ns 560.39 ns]
                        thrpt:  [1.7845 Melem/s 1.7882 Melem/s 1.7922 Melem/s]
                 change:
                        time:   [−0.6141% −0.3268% −0.0399%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0400% +0.3279% +0.6179%]
                        Change within noise threshold.
condition/string_eq     time:   [70.164 ns 70.287 ns 70.424 ns]
                        thrpt:  [14.200 Melem/s 14.227 Melem/s 14.252 Melem/s]
                 change:
                        time:   [−0.2470% −0.1037% +0.0410%] (p = 0.17 > 0.05)
                        thrpt:  [−0.0409% +0.1038% +0.2476%]
                        No change in performance detected.
Found 15 outliers among 100 measurements (15.00%)
  15 (15.00%) low mild

condition_expr/single   time:   [50.906 ns 50.988 ns 51.083 ns]
                        thrpt:  [19.576 Melem/s 19.613 Melem/s 19.644 Melem/s]
                 change:
                        time:   [−0.5067% −0.1866% +0.1160%] (p = 0.25 > 0.05)
                        thrpt:  [−0.1159% +0.1869% +0.5092%]
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  5 (5.00%) low mild
  6 (6.00%) high mild
condition_expr/and_2    time:   [103.74 ns 103.90 ns 104.07 ns]
                        thrpt:  [9.6085 Melem/s 9.6242 Melem/s 9.6397 Melem/s]
                 change:
                        time:   [−1.0006% −0.7386% −0.4870%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4894% +0.7441% +1.0107%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
condition_expr/and_5    time:   [306.33 ns 306.87 ns 307.42 ns]
                        thrpt:  [3.2529 Melem/s 3.2587 Melem/s 3.2644 Melem/s]
                 change:
                        time:   [−0.8805% −0.6019% −0.3412%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3423% +0.6056% +0.8883%]
                        Change within noise threshold.
condition_expr/or_3     time:   [174.27 ns 174.79 ns 175.36 ns]
                        thrpt:  [5.7026 Melem/s 5.7211 Melem/s 5.7382 Melem/s]
                 change:
                        time:   [+1.2802% +1.5240% +1.7552%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7249% −1.5011% −1.2640%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) low mild
condition_expr/nested   time:   [123.02 ns 123.41 ns 123.81 ns]
                        thrpt:  [8.0769 Melem/s 8.1032 Melem/s 8.1288 Melem/s]
                 change:
                        time:   [−1.8168% −1.5038% −1.2132%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2281% +1.5267% +1.8504%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild

rule/create             time:   [278.40 ns 279.29 ns 280.29 ns]
                        thrpt:  [3.5678 Melem/s 3.5805 Melem/s 3.5920 Melem/s]
                 change:
                        time:   [−0.7711% −0.4731% −0.1798%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1802% +0.4754% +0.7771%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule/matches            time:   [103.74 ns 104.06 ns 104.41 ns]
                        thrpt:  [9.5776 Melem/s 9.6095 Melem/s 9.6398 Melem/s]
                 change:
                        time:   [−0.6046% −0.2708% +0.0307%] (p = 0.10 > 0.05)
                        thrpt:  [−0.0307% +0.2715% +0.6083%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

rule_context/create     time:   [840.64 ns 843.86 ns 847.04 ns]
                        thrpt:  [1.1806 Melem/s 1.1850 Melem/s 1.1896 Melem/s]
                 change:
                        time:   [−0.1129% +0.1422% +0.4051%] (p = 0.27 > 0.05)
                        thrpt:  [−0.4034% −0.1420% +0.1130%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
rule_context/get_simple time:   [49.074 ns 49.134 ns 49.191 ns]
                        thrpt:  [20.329 Melem/s 20.353 Melem/s 20.377 Melem/s]
                 change:
                        time:   [−1.2660% −0.9377% −0.6121%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6159% +0.9466% +1.2822%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  6 (6.00%) high severe
rule_context/get_nested time:   [559.36 ns 560.84 ns 562.28 ns]
                        thrpt:  [1.7785 Melem/s 1.7830 Melem/s 1.7877 Melem/s]
                 change:
                        time:   [−0.7808% −0.5643% −0.3571%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3584% +0.5675% +0.7870%]
                        Change within noise threshold.
Found 16 outliers among 100 measurements (16.00%)
  3 (3.00%) low severe
  10 (10.00%) low mild
  3 (3.00%) high mild
rule_context/get_deep_nested
                        time:   [567.22 ns 568.63 ns 570.02 ns]
                        thrpt:  [1.7543 Melem/s 1.7586 Melem/s 1.7630 Melem/s]
                 change:
                        time:   [−1.0275% −0.7786% −0.5303%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5332% +0.7847% +1.0381%]
                        Change within noise threshold.
Found 27 outliers among 100 measurements (27.00%)
  2 (2.00%) low severe
  16 (16.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe

rule_engine_basic/create
                        time:   [8.1850 ns 8.2119 ns 8.2413 ns]
                        thrpt:  [121.34 Melem/s 121.77 Melem/s 122.17 Melem/s]
                 change:
                        time:   [+0.0429% +0.2919% +0.5530%] (p = 0.03 < 0.05)
                        thrpt:  [−0.5499% −0.2911% −0.0429%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  12 (12.00%) high mild
  1 (1.00%) high severe
rule_engine_basic/add_rule
                        time:   [2.7939 µs 2.9503 µs 3.0817 µs]
                        thrpt:  [324.50 Kelem/s 338.95 Kelem/s 357.93 Kelem/s]
                 change:
                        time:   [−12.761% −1.0254% +11.684%] (p = 0.87 > 0.05)
                        thrpt:  [−10.461% +1.0360% +14.628%]
                        No change in performance detected.
rule_engine_basic/get_rule
                        time:   [21.948 ns 22.940 ns 23.869 ns]
                        thrpt:  [41.895 Melem/s 43.591 Melem/s 45.561 Melem/s]
                 change:
                        time:   [+25.885% +31.595% +36.837%] (p = 0.00 < 0.05)
                        thrpt:  [−26.920% −24.009% −20.563%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) low mild
rule_engine_basic/rules_by_tag
                        time:   [1.1248 µs 1.1254 µs 1.1261 µs]
                        thrpt:  [888.04 Kelem/s 888.59 Kelem/s 889.07 Kelem/s]
                 change:
                        time:   [−0.1139% +0.0416% +0.1901%] (p = 0.60 > 0.05)
                        thrpt:  [−0.1898% −0.0416% +0.1140%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
rule_engine_basic/stats time:   [6.9190 µs 6.9228 µs 6.9267 µs]
                        thrpt:  [144.37 Kelem/s 144.45 Kelem/s 144.53 Kelem/s]
                 change:
                        time:   [−1.4682% −1.1459% −0.8310%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8379% +1.1592% +1.4901%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples in estimated 5.0108 s (2.3M iterule_engine_evaluate/evaluate_10_rules
                        time:   [2.1746 µs 2.1764 µs 2.1782 µs]
                        thrpt:  [459.09 Kelem/s 459.48 Kelem/s 459.85 Kelem/s]
                 change:
                        time:   [+0.2441% +0.3943% +0.5324%] (p = 0.00 < 0.05)
                        thrpt:  [−0.5295% −0.3928% −0.2435%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samples in estimated 5.0010 s (21rule_engine_evaluate/evaluate_first_10_rules
                        time:   [233.16 ns 233.37 ns 233.58 ns]
                        thrpt:  [4.2812 Melem/s 4.2851 Melem/s 4.2888 Melem/s]
                 change:
                        time:   [−0.4752% −0.2716% −0.0653%] (p = 0.01 < 0.05)
                        thrpt:  [+0.0653% +0.2723% +0.4775%]
                        Change within noise threshold.
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples in estimated 5.0814 s (232k itrule_engine_evaluate/evaluate_100_rules
                        time:   [21.785 µs 21.810 µs 21.833 µs]
                        thrpt:  [45.803 Kelem/s 45.852 Kelem/s 45.903 Kelem/s]
                 change:
                        time:   [−0.0189% +0.1577% +0.3314%] (p = 0.08 > 0.05)
                        thrpt:  [−0.3303% −0.1574% +0.0189%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 samples in estimated 5.0008 s (2rule_engine_evaluate/evaluate_first_100_rules
                        time:   [232.65 ns 232.94 ns 233.27 ns]
                        thrpt:  [4.2869 Melem/s 4.2929 Melem/s 4.2983 Melem/s]
                 change:
                        time:   [−0.7583% −0.4613% −0.1771%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1774% +0.4634% +0.7641%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated 5.0505 srule_engine_evaluate/evaluate_matching_100_rules
                        time:   [22.256 µs 22.267 µs 22.277 µs]
                        thrpt:  [44.889 Kelem/s 44.909 Kelem/s 44.932 Kelem/s]
                 change:
                        time:   [−0.2224% −0.0633% +0.0813%] (p = 0.43 > 0.05)
                        thrpt:  [−0.0812% +0.0634% +0.2229%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samples in estimated 6.1484 s (25k itrule_engine_evaluate/evaluate_1000_rules
                        time:   [214.89 µs 216.46 µs 218.65 µs]
                        thrpt:  [4.5736 Kelem/s 4.6198 Kelem/s 4.6535 Kelem/s]
                 change:
                        time:   [−0.4273% +0.0389% +0.6805%] (p = 0.90 > 0.05)
                        thrpt:  [−0.6759% −0.0389% +0.4292%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 samples in estimated 5.0008 s (rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [233.13 ns 233.79 ns 234.57 ns]
                        thrpt:  [4.2632 Melem/s 4.2774 Melem/s 4.2894 Melem/s]
                 change:
                        time:   [−0.6570% −0.3479% −0.0661%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0661% +0.3491% +0.6613%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimated 5.0020 s (2.3M iterationsrule_engine_scaling/evaluate/10
                        time:   [2.1755 µs 2.1810 µs 2.1865 µs]
                        thrpt:  [457.36 Kelem/s 458.50 Kelem/s 459.66 Kelem/s]
                 change:
                        time:   [−0.7867% −0.5943% −0.3897%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3913% +0.5978% +0.7930%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/10: Collecting 100 samples in estimated 5.0003 s (21M iterarule_engine_scaling/evaluate_first/10
                        time:   [234.13 ns 234.59 ns 235.18 ns]
                        thrpt:  [4.2520 Melem/s 4.2628 Melem/s 4.2711 Melem/s]
                 change:
                        time:   [−0.2364% +0.0511% +0.3305%] (p = 0.73 > 0.05)
                        thrpt:  [−0.3294% −0.0511% +0.2369%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high severe
Benchmarking rule_engine_scaling/evaluate/50: Collecting 100 samples in estimated 5.0424 s (460k iterationsrule_engine_scaling/evaluate/50
                        time:   [10.969 µs 10.996 µs 11.024 µs]
                        thrpt:  [90.715 Kelem/s 90.945 Kelem/s 91.169 Kelem/s]
                 change:
                        time:   [+0.8329% +1.0772% +1.3240%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3067% −1.0658% −0.8260%]
                        Change within noise threshold.
Benchmarking rule_engine_scaling/evaluate_first/50: Collecting 100 samples in estimated 5.0010 s (21M iterarule_engine_scaling/evaluate_first/50
                        time:   [233.95 ns 234.83 ns 235.80 ns]
                        thrpt:  [4.2409 Melem/s 4.2583 Melem/s 4.2744 Melem/s]
                 change:
                        time:   [−0.8868% −0.5158% −0.1588%] (p = 0.01 < 0.05)
                        thrpt:  [+0.1590% +0.5184% +0.8947%]
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_scaling/evaluate/100: Collecting 100 samples in estimated 5.0906 s (232k iterationrule_engine_scaling/evaluate/100
                        time:   [21.993 µs 22.029 µs 22.064 µs]
                        thrpt:  [45.323 Kelem/s 45.395 Kelem/s 45.469 Kelem/s]
                 change:
                        time:   [−0.0954% +0.1104% +0.2930%] (p = 0.28 > 0.05)
                        thrpt:  [−0.2922% −0.1103% +0.0955%]
                        No change in performance detected.
Benchmarking rule_engine_scaling/evaluate_first/100: Collecting 100 samples in estimated 5.0003 s (21M iterrule_engine_scaling/evaluate_first/100
                        time:   [233.03 ns 233.43 ns 233.86 ns]
                        thrpt:  [4.2761 Melem/s 4.2839 Melem/s 4.2912 Melem/s]
                 change:
                        time:   [−0.7415% −0.4679% −0.2147%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2152% +0.4701% +0.7470%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking rule_engine_scaling/evaluate/500: Collecting 100 samples in estimated 5.7424 s (30k iterationsrule_engine_scaling/evaluate/500
                        time:   [161.35 µs 184.36 µs 209.49 µs]
                        thrpt:  [4.7734 Kelem/s 5.4243 Kelem/s 6.1976 Kelem/s]
                 change:
                        time:   [+71.382% +89.007% +106.39%] (p = 0.00 < 0.05)
                        thrpt:  [−51.548% −47.092% −41.651%]
                        Performance has regressed.
Benchmarking rule_engine_scaling/evaluate_first/500: Collecting 100 samples in estimated 5.0000 s (21M iterrule_engine_scaling/evaluate_first/500
                        time:   [236.81 ns 237.82 ns 238.72 ns]
                        thrpt:  [4.1890 Melem/s 4.2049 Melem/s 4.2228 Melem/s]
                 change:
                        time:   [−0.3056% +0.0974% +0.5234%] (p = 0.64 > 0.05)
                        thrpt:  [−0.5207% −0.0973% +0.3066%]
                        No change in performance detected.
Benchmarking rule_engine_scaling/evaluate/1000: Collecting 100 samples in estimated 5.4334 s (15k iterationrule_engine_scaling/evaluate/1000
                        time:   [251.66 µs 273.45 µs 297.02 µs]
                        thrpt:  [3.3668 Kelem/s 3.6570 Kelem/s 3.9735 Kelem/s]
                 change:
                        time:   [+35.942% +44.368% +52.692%] (p = 0.00 < 0.05)
                        thrpt:  [−34.509% −30.733% −26.439%]
                        Performance has regressed.
Benchmarking rule_engine_scaling/evaluate_first/1000: Collecting 100 samples in estimated 5.0001 s (22M iterule_engine_scaling/evaluate_first/1000
                        time:   [234.48 ns 235.94 ns 237.31 ns]
                        thrpt:  [4.2139 Melem/s 4.2384 Melem/s 4.2647 Melem/s]
                 change:
                        time:   [−1.2555% −0.7671% −0.2927%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2936% +0.7731% +1.2715%]
                        Change within noise threshold.

rule_set/create         time:   [2.7967 µs 2.8021 µs 2.8076 µs]
                        thrpt:  [356.18 Kelem/s 356.88 Kelem/s 357.56 Kelem/s]
                 change:
                        time:   [+0.0751% +0.3038% +0.5624%] (p = 0.02 < 0.05)
                        thrpt:  [−0.5592% −0.3029% −0.0751%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
rule_set/load_into_engine
                        time:   [5.5154 µs 5.5185 µs 5.5214 µs]
                        thrpt:  [181.11 Kelem/s 181.21 Kelem/s 181.31 Kelem/s]
                 change:
                        time:   [−0.4427% −0.2866% −0.1434%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1436% +0.2875% +0.4447%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) low mild

trace_id/generate       time:   [540.98 ns 542.69 ns 544.48 ns]
                        thrpt:  [1.8366 Melem/s 1.8427 Melem/s 1.8485 Melem/s]
                 change:
                        time:   [−0.2438% +0.0058% +0.2304%] (p = 0.96 > 0.05)
                        thrpt:  [−0.2298% −0.0058% +0.2444%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
trace_id/to_hex         time:   [76.021 ns 76.202 ns 76.396 ns]
                        thrpt:  [13.090 Melem/s 13.123 Melem/s 13.154 Melem/s]
                 change:
                        time:   [−0.9674% −0.7688% −0.5729%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5762% +0.7747% +0.9768%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high mild
trace_id/from_hex       time:   [23.173 ns 23.178 ns 23.184 ns]
                        thrpt:  [43.133 Melem/s 43.144 Melem/s 43.153 Melem/s]
                 change:
                        time:   [−0.6213% −0.4484% −0.3079%] (p = 0.00 < 0.05)
                        thrpt:  [+0.3089% +0.4504% +0.6252%]
                        Change within noise threshold.
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) high mild
  5 (5.00%) high severe

context_operations/create
                        time:   [822.47 ns 823.56 ns 824.64 ns]
                        thrpt:  [1.2126 Melem/s 1.2142 Melem/s 1.2159 Melem/s]
                 change:
                        time:   [−0.4564% −0.2030% +0.0102%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0102% +0.2034% +0.4584%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
context_operations/child
                        time:   [284.66 ns 285.03 ns 285.38 ns]
                        thrpt:  [3.5041 Melem/s 3.5085 Melem/s 3.5129 Melem/s]
                 change:
                        time:   [−0.2857% −0.0917% +0.0953%] (p = 0.34 > 0.05)
                        thrpt:  [−0.0952% +0.0918% +0.2865%]
                        No change in performance detected.
context_operations/for_remote
                        time:   [293.92 ns 294.27 ns 294.61 ns]
                        thrpt:  [3.3943 Melem/s 3.3983 Melem/s 3.4023 Melem/s]
                 change:
                        time:   [+3.4492% +3.9864% +4.5532%] (p = 0.00 < 0.05)
                        thrpt:  [−4.3549% −3.8336% −3.3342%]
                        Performance has regressed.
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) high mild
  13 (13.00%) high severe
Benchmarking context_operations/to_traceparent: Collecting 100 samples in estimated 5.0003 s (22M iterationcontext_operations/to_traceparent
                        time:   [234.34 ns 235.10 ns 235.87 ns]
                        thrpt:  [4.2397 Melem/s 4.2534 Melem/s 4.2674 Melem/s]
                 change:
                        time:   [+4.1407% +4.5985% +5.0463%] (p = 0.00 < 0.05)
                        thrpt:  [−4.8039% −4.3963% −3.9760%]
                        Performance has regressed.
Benchmarking context_operations/from_traceparent: Collecting 100 samples in estimated 5.0011 s (13M iteraticontext_operations/from_traceparent
                        time:   [374.26 ns 374.87 ns 375.55 ns]
                        thrpt:  [2.6628 Melem/s 2.6676 Melem/s 2.6719 Melem/s]
                 change:
                        time:   [+0.1729% +0.3269% +0.4843%] (p = 0.00 < 0.05)
                        thrpt:  [−0.4819% −0.3259% −0.1726%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

baggage/create          time:   [2.0362 ns 2.0368 ns 2.0375 ns]
                        thrpt:  [490.80 Melem/s 490.96 Melem/s 491.10 Melem/s]
                 change:
                        time:   [−0.4826% −0.3366% −0.2086%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2090% +0.3377% +0.4850%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
baggage/get             time:   [16.856 ns 17.696 ns 18.484 ns]
                        thrpt:  [54.101 Melem/s 56.509 Melem/s 59.326 Melem/s]
                 change:
                        time:   [−15.369% −10.780% −6.2245%] (p = 0.00 < 0.05)
                        thrpt:  [+6.6377% +12.082% +18.160%]
                        Performance has improved.
baggage/set             time:   [48.985 ns 49.004 ns 49.025 ns]
                        thrpt:  [20.398 Melem/s 20.406 Melem/s 20.414 Melem/s]
                 change:
                        time:   [−0.4560% −0.2901% −0.1427%] (p = 0.00 < 0.05)
                        thrpt:  [+0.1429% +0.2909% +0.4581%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
baggage/merge           time:   [861.43 ns 861.98 ns 862.72 ns]
                        thrpt:  [1.1591 Melem/s 1.1601 Melem/s 1.1609 Melem/s]
                 change:
                        time:   [−0.4576% −0.2976% −0.0807%] (p = 0.00 < 0.05)
                        thrpt:  [+0.0808% +0.2985% +0.4597%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

span/create             time:   [324.32 ns 324.81 ns 325.32 ns]
                        thrpt:  [3.0739 Melem/s 3.0788 Melem/s 3.0834 Melem/s]
                 change:
                        time:   [−0.0016% +0.2616% +0.4775%] (p = 0.03 < 0.05)
                        thrpt:  [−0.4752% −0.2609% +0.0016%]
                        Change within noise threshold.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
span/set_attribute      time:   [47.986 ns 48.114 ns 48.239 ns]
                        thrpt:  [20.730 Melem/s 20.784 Melem/s 20.839 Melem/s]
                 change:
                        time:   [−0.3769% +0.0794% +0.4950%] (p = 0.73 > 0.05)
                        thrpt:  [−0.4926% −0.0793% +0.3783%]
                        No change in performance detected.
span/add_event          time:   [51.051 ns 51.478 ns 51.844 ns]
                        thrpt:  [19.289 Melem/s 19.426 Melem/s 19.588 Melem/s]
                 change:
                        time:   [−2.6292% −0.9306% +0.8409%] (p = 0.31 > 0.05)
                        thrpt:  [−0.8339% +0.9393% +2.7002%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild
span/with_kind          time:   [323.61 ns 324.84 ns 326.31 ns]
                        thrpt:  [3.0646 Melem/s 3.0784 Melem/s 3.0901 Melem/s]
                 change:
                        time:   [+0.3356% +0.7189% +1.2016%] (p = 0.00 < 0.05)
                        thrpt:  [−1.1873% −0.7138% −0.3345%]
                        Change within noise threshold.
Found 18 outliers among 100 measurements (18.00%)
  3 (3.00%) high mild
  15 (15.00%) high severe

context_store/create_context
                        time:   [99.428 µs 99.449 µs 99.472 µs]
                        thrpt:  [10.053 Kelem/s 10.055 Kelem/s 10.058 Kelem/s]
                 change:
                        time:   [+2.3427% +2.5171% +2.6562%] (p = 0.00 < 0.05)
                        thrpt:  [−2.5875% −2.4553% −2.2891%]
                        Performance has regressed.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

thread 'main' (281220636) panicked at benches/net.rs:4552:45:
called `Result::unwrap()` on an `Err` value: CapacityExceeded
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

error: bench failed, to rerun pass `--bench net`
lazlo@Lazlos-MBP net %
