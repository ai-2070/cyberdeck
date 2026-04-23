     Running benches\auth_guard.rs (target\release\deps\auth_guard-a5a677cd9b1bf7dd.exe)
Gnuplot not found, using plotters backend
Benchmarking auth_guard_check_fast_hit/single_thread: Collecting 50 samples in estimated 5.0000 s (211Mauth_guard_check_fast_hit/single_thread
                        time:   [23.463 ns 23.529 ns 23.605 ns]
                        thrpt:  [42.364 Melem/s 42.501 Melem/s 42.621 Melem/s]
                 change:
                        time:   [−4.7455% −4.2957% −3.8846%] (p = 0.00 < 0.05)
                        thrpt:  [+4.0416% +4.4885% +4.9819%]
                        Performance has improved.
Found 4 outliers among 50 measurements (8.00%)
  4 (8.00%) high severe

Benchmarking auth_guard_check_fast_miss/single_thread: Collecting 50 samples in estimated 5.0000 s (505auth_guard_check_fast_miss/single_thread
                        time:   [9.8567 ns 9.8952 ns 9.9367 ns]
                        thrpt:  [100.64 Melem/s 101.06 Melem/s 101.45 Melem/s]
                 change:
                        time:   [−3.9061% −3.5008% −3.0589%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1555% +3.6279% +4.0649%]
                        Performance has improved.
Found 5 outliers among 50 measurements (10.00%)
  4 (8.00%) high mild
  1 (2.00%) high severe

Benchmarking auth_guard_check_fast_contended/eight_threads: Collecting 50 samples in estimated 5.0000 sauth_guard_check_fast_contended/eight_threads
                        time:   [25.501 ns 25.605 ns 25.725 ns]
                        thrpt:  [38.873 Melem/s 39.055 Melem/s 39.214 Melem/s]
                 change:
                        time:   [−4.4784% −2.0169% +0.5205%] (p = 0.13 > 0.05)
                        thrpt:  [−0.5178% +2.0584% +4.6884%]
                        No change in performance detected.
Found 5 outliers among 50 measurements (10.00%)
  5 (10.00%) high severe

Benchmarking auth_guard_allow_channel/insert: Collecting 50 samples in estimated 5.0000 s (18M iteratioauth_guard_allow_channel/insert
                        time:   [155.52 ns 160.50 ns 165.62 ns]
                        thrpt:  [6.0379 Melem/s 6.2307 Melem/s 6.4298 Melem/s]
                 change:
                        time:   [−8.2786% −4.4123% −0.1470%] (p = 0.04 < 0.05)
                        thrpt:  [+0.1472% +4.6160% +9.0258%]
                        Change within noise threshold.

Benchmarking auth_guard_hot_hit_ceiling/million_ops: Collecting 50 samples in estimated 5.1990 s (550 iauth_guard_hot_hit_ceiling/million_ops
                        time:   [9.3555 ms 9.3848 ms 9.4162 ms]
                        change: [−4.1179% −3.8143% −3.4857%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 50 measurements (2.00%)
  1 (2.00%) high mild

     Running benches\cortex.rs (target\release\deps\cortex-95b4f1707bca548c.exe)
Gnuplot not found, using plotters backend
cortex_ingest/tasks_create
                        time:   [481.51 ns 536.71 ns 610.46 ns]
                        thrpt:  [1.6381 Melem/s 1.8632 Melem/s 2.0768 Melem/s]
                 change:
                        time:   [−8.1784% +4.6445% +19.973%] (p = 0.52 > 0.05)
                        thrpt:  [−16.648% −4.4384% +8.9069%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  6 (6.00%) high severe
Benchmarking cortex_ingest/memories_store: Collecting 100 samples in estimated 5.0026 s (6.9M iterationcortex_ingest/memories_store
                        time:   [766.61 ns 854.29 ns 955.94 ns]
                        thrpt:  [1.0461 Melem/s 1.1706 Melem/s 1.3045 Melem/s]
                 change:
                        time:   [+19.243% +31.014% +43.361%] (p = 0.00 < 0.05)
                        thrpt:  [−30.246% −23.673% −16.137%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low mild
  16 (16.00%) high severe

Benchmarking cortex_fold_barrier/tasks_create_and_wait: Collecting 100 samples in estimated 5.0026 s (2cortex_fold_barrier/tasks_create_and_wait
                        time:   [1.9480 µs 2.0155 µs 2.0968 µs]
                        thrpt:  [476.91 Kelem/s 496.16 Kelem/s 513.36 Kelem/s]
                 change:
                        time:   [−8.6395% −5.0845% −1.3942%] (p = 0.01 < 0.05)
                        thrpt:  [+1.4139% +5.3568% +9.4565%]
                        Performance has improved.
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high severe
Benchmarking cortex_fold_barrier/memories_store_and_wait: Collecting 100 samples in estimated 5.0001 s cortex_fold_barrier/memories_store_and_wait
                        time:   [2.5839 µs 2.9267 µs 3.3906 µs]
                        thrpt:  [294.93 Kelem/s 341.68 Kelem/s 387.01 Kelem/s]
                 change:
                        time:   [−14.663% −3.5518% +8.5790%] (p = 0.57 > 0.05)
                        thrpt:  [−7.9012% +3.6826% +17.182%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe

Benchmarking cortex_query/tasks_find_many/100: Collecting 100 samples in estimated 5.0023 s (2.3M iteracortex_query/tasks_find_many/100
                        time:   [2.1593 µs 2.1638 µs 2.1688 µs]
                        thrpt:  [46.108 Melem/s 46.215 Melem/s 46.312 Melem/s]
                 change:
                        time:   [+2.6714% +2.9951% +3.3173%] (p = 0.00 < 0.05)
                        thrpt:  [−3.2108% −2.9080% −2.6019%]
                        Performance has regressed.
Benchmarking cortex_query/tasks_count_where/100: Collecting 100 samples in estimated 5.0008 s (30M itercortex_query/tasks_count_where/100
                        time:   [162.62 ns 163.92 ns 165.23 ns]
                        thrpt:  [605.22 Melem/s 610.07 Melem/s 614.92 Melem/s]
                 change:
                        time:   [−2.8426% −1.4566% −0.1340%] (p = 0.04 < 0.05)
                        thrpt:  [+0.1341% +1.4781% +2.9257%]
                        Change within noise threshold.
Benchmarking cortex_query/tasks_find_unique/100: Collecting 100 samples in estimated 5.0000 s (769M itecortex_query/tasks_find_unique/100
                        time:   [6.4836 ns 6.5063 ns 6.5312 ns]
                        thrpt:  [15.311 Gelem/s 15.370 Gelem/s 15.424 Gelem/s]
                 change:
                        time:   [−3.8460% −3.3845% −2.9115%] (p = 0.00 < 0.05)
                        thrpt:  [+2.9988% +3.5030% +3.9999%]
                        Performance has improved.
Benchmarking cortex_query/memories_find_many_tag/100: Collecting 100 samples in estimated 5.0341 s (424cortex_query/memories_find_many_tag/100
                        time:   [11.828 µs 11.857 µs 11.888 µs]
                        thrpt:  [8.4118 Melem/s 8.4341 Melem/s 8.4543 Melem/s]
                 change:
                        time:   [+1.1089% +1.4183% +1.7442%] (p = 0.00 < 0.05)
                        thrpt:  [−1.7143% −1.3984% −1.0967%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking cortex_query/memories_count_where/100: Collecting 100 samples in estimated 5.0005 s (8.6M cortex_query/memories_count_where/100
                        time:   [578.51 ns 580.78 ns 583.10 ns]
                        thrpt:  [171.50 Melem/s 172.18 Melem/s 172.86 Melem/s]
                 change:
                        time:   [+18.665% +19.179% +19.700%] (p = 0.00 < 0.05)
                        thrpt:  [−16.458% −16.093% −15.729%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_many/1000: Collecting 100 samples in estimated 5.0627 s (278k itercortex_query/tasks_find_many/1000
                        time:   [18.367 µs 18.417 µs 18.472 µs]
                        thrpt:  [54.137 Melem/s 54.299 Melem/s 54.446 Melem/s]
                 change:
                        time:   [−3.6440% −3.3357% −3.0220%] (p = 0.00 < 0.05)
                        thrpt:  [+3.1162% +3.4508% +3.7818%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/1000: Collecting 100 samples in estimated 5.0022 s (3.4M itcortex_query/tasks_count_where/1000
                        time:   [1.4796 µs 1.4863 µs 1.4935 µs]
                        thrpt:  [669.56 Melem/s 672.79 Melem/s 675.85 Melem/s]
                 change:
                        time:   [−10.333% −9.8385% −9.2879%] (p = 0.00 < 0.05)
                        thrpt:  [+10.239% +10.912% +11.524%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking cortex_query/tasks_find_unique/1000: Collecting 100 samples in estimated 5.0000 s (765M itcortex_query/tasks_find_unique/1000
                        time:   [6.4693 ns 6.4899 ns 6.5119 ns]
                        thrpt:  [153.57 Gelem/s 154.09 Gelem/s 154.58 Gelem/s]
                 change:
                        time:   [−3.9509% −3.6134% −3.2349%] (p = 0.00 < 0.05)
                        thrpt:  [+3.3431% +3.7488% +4.1135%]
                        Performance has improved.
Benchmarking cortex_query/memories_find_many_tag/1000: Collecting 100 samples in estimated 5.3030 s (56cortex_query/memories_find_many_tag/1000
                        time:   [94.996 µs 95.233 µs 95.501 µs]
                        thrpt:  [10.471 Melem/s 10.501 Melem/s 10.527 Melem/s]
                 change:
                        time:   [+40.279% +40.777% +41.319%] (p = 0.00 < 0.05)
                        thrpt:  [−29.238% −28.966% −28.713%]
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_count_where/1000: Collecting 100 samples in estimated 5.0015 s (823kcortex_query/memories_count_where/1000
                        time:   [5.9569 µs 5.9892 µs 6.0215 µs]
                        thrpt:  [166.07 Melem/s 166.97 Melem/s 167.87 Melem/s]
                 change:
                        time:   [+24.221% +24.986% +25.717%] (p = 0.00 < 0.05)
                        thrpt:  [−20.456% −19.991% −19.499%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cortex_query/tasks_find_many/10000: Collecting 100 samples in estimated 5.3119 s (30k itercortex_query/tasks_find_many/10000
                        time:   [173.05 µs 173.69 µs 174.39 µs]
                        thrpt:  [57.343 Melem/s 57.572 Melem/s 57.787 Melem/s]
                 change:
                        time:   [−4.0922% −3.6242% −3.1345%] (p = 0.00 < 0.05)
                        thrpt:  [+3.2359% +3.7605% +4.2668%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_query/tasks_count_where/10000: Collecting 100 samples in estimated 5.0738 s (136k icortex_query/tasks_count_where/10000
                        time:   [37.086 µs 37.233 µs 37.385 µs]
                        thrpt:  [267.49 Melem/s 268.58 Melem/s 269.65 Melem/s]
                 change:
                        time:   [+16.915% +18.236% +19.564%] (p = 0.00 < 0.05)
                        thrpt:  [−16.363% −15.424% −14.467%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/tasks_find_unique/10000: Collecting 100 samples in estimated 5.0000 s (722M icortex_query/tasks_find_unique/10000
                        time:   [6.8647 ns 6.8864 ns 6.9099 ns]
                        thrpt:  [1447.2 Gelem/s 1452.1 Gelem/s 1456.7 Gelem/s]
                 change:
                        time:   [−4.5816% −4.0403% −3.4044%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5243% +4.2104% +4.8016%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_query/memories_find_many_tag/10000: Collecting 100 samples in estimated 8.2485 s (1cortex_query/memories_find_many_tag/10000
                        time:   [812.76 µs 814.23 µs 815.86 µs]
                        thrpt:  [12.257 Melem/s 12.282 Melem/s 12.304 Melem/s]
                 change:
                        time:   [−7.1350% −6.7125% −6.2904%] (p = 0.00 < 0.05)
                        thrpt:  [+6.7126% +7.1955% +7.6831%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_query/memories_count_where/10000: Collecting 100 samples in estimated 5.0246 s (45kcortex_query/memories_count_where/10000
                        time:   [109.98 µs 110.32 µs 110.70 µs]
                        thrpt:  [90.336 Melem/s 90.645 Melem/s 90.929 Melem/s]
                 change:
                        time:   [+3.2233% +3.8076% +4.4113%] (p = 0.00 < 0.05)
                        thrpt:  [−4.2250% −3.6679% −3.1226%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

Benchmarking cortex_snapshot/tasks_encode/100: Collecting 100 samples in estimated 5.0122 s (1.7M iteracortex_snapshot/tasks_encode/100
                        time:   [2.9555 µs 2.9637 µs 2.9730 µs]
                        thrpt:  [33.637 Melem/s 33.742 Melem/s 33.836 Melem/s]
                 change:
                        time:   [−3.3710% −3.0222% −2.6142%] (p = 0.00 < 0.05)
                        thrpt:  [+2.6844% +3.1164% +3.4886%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cortex_snapshot/memories_encode/100: Collecting 100 samples in estimated 5.0226 s (970k itcortex_snapshot/memories_encode/100
                        time:   [5.5269 µs 5.6383 µs 5.7827 µs]
                        thrpt:  [17.293 Melem/s 17.736 Melem/s 18.093 Melem/s]
                 change:
                        time:   [+2.5933% +4.4064% +6.3173%] (p = 0.00 < 0.05)
                        thrpt:  [−5.9420% −4.2204% −2.5278%]
                        Performance has regressed.
Found 19 outliers among 100 measurements (19.00%)
  8 (8.00%) high mild
  11 (11.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_3939/100: Collecting 100 samples in estimated 5.cortex_snapshot/netdb_bundle_encode_bytes_3939/100
                        time:   [3.3194 µs 3.3243 µs 3.3299 µs]
                        thrpt:  [30.031 Melem/s 30.081 Melem/s 30.126 Melem/s]
                 change:
                        time:   [+63.850% +64.203% +64.543%] (p = 0.00 < 0.05)
                        thrpt:  [−39.226% −39.100% −38.968%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/100: Collecting 100 samples in estimated 5.0010 s (1.4cortex_snapshot/netdb_bundle_decode/100
                        time:   [3.6089 µs 3.6297 µs 3.6675 µs]
                        thrpt:  [27.267 Melem/s 27.551 Melem/s 27.709 Melem/s]
                 change:
                        time:   [+42.670% +43.241% +43.822%] (p = 0.00 < 0.05)
                        thrpt:  [−30.470% −30.188% −29.908%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  3 (3.00%) high mild
  14 (14.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/1000: Collecting 100 samples in estimated 5.1567 s (96k iteracortex_snapshot/tasks_encode/1000
                        time:   [51.033 µs 52.024 µs 53.204 µs]
                        thrpt:  [18.796 Melem/s 19.222 Melem/s 19.595 Melem/s]
                 change:
                        time:   [+62.056% +64.428% +67.384%] (p = 0.00 < 0.05)
                        thrpt:  [−40.257% −39.183% −38.293%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking cortex_snapshot/memories_encode/1000: Collecting 100 samples in estimated 5.4080 s (56k itcortex_snapshot/memories_encode/1000
                        time:   [94.070 µs 96.783 µs 99.662 µs]
                        thrpt:  [10.034 Melem/s 10.332 Melem/s 10.630 Melem/s]
                 change:
                        time:   [+73.069% +75.969% +79.262%] (p = 0.00 < 0.05)
                        thrpt:  [−44.216% −43.172% −42.220%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_48274/1000: Collecting 100 samples in estimated cortex_snapshot/netdb_bundle_encode_bytes_48274/1000
                        time:   [37.047 µs 37.694 µs 38.489 µs]
                        thrpt:  [25.981 Melem/s 26.529 Melem/s 26.993 Melem/s]
                 change:
                        time:   [+52.001% +54.214% +56.620%] (p = 0.00 < 0.05)
                        thrpt:  [−36.151% −35.155% −34.211%]
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/1000: Collecting 100 samples in estimated 5.0887 s (11cortex_snapshot/netdb_bundle_decode/1000
                        time:   [43.486 µs 43.645 µs 43.834 µs]
                        thrpt:  [22.813 Melem/s 22.912 Melem/s 22.996 Melem/s]
                 change:
                        time:   [+34.183% +34.664% +35.238%] (p = 0.00 < 0.05)
                        thrpt:  [−26.056% −25.741% −25.475%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking cortex_snapshot/tasks_encode/10000: Collecting 100 samples in estimated 5.2702 s (10k itercortex_snapshot/tasks_encode/10000
                        time:   [495.98 µs 508.08 µs 522.46 µs]
                        thrpt:  [19.140 Melem/s 19.682 Melem/s 20.162 Melem/s]
                 change:
                        time:   [+47.370% +50.047% +53.164%] (p = 0.00 < 0.05)
                        thrpt:  [−34.711% −33.354% −32.144%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking cortex_snapshot/memories_encode/10000: Collecting 100 samples in estimated 9.6692 s (10k icortex_snapshot/memories_encode/10000
                        time:   [938.09 µs 953.78 µs 971.32 µs]
                        thrpt:  [10.295 Melem/s 10.485 Melem/s 10.660 Melem/s]
                 change:
                        time:   [+59.521% +62.692% +66.288%] (p = 0.00 < 0.05)
                        thrpt:  [−39.863% −38.534% −37.312%]
                        Performance has regressed.
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_encode_bytes_511774/10000: Collecting 100 samples in estimatecortex_snapshot/netdb_bundle_encode_bytes_511774/10000
                        time:   [352.98 µs 353.46 µs 354.04 µs]
                        thrpt:  [28.245 Melem/s 28.292 Melem/s 28.331 Melem/s]
                 change:
                        time:   [+35.613% +35.996% +36.395%] (p = 0.00 < 0.05)
                        thrpt:  [−26.684% −26.469% −26.261%]
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
Benchmarking cortex_snapshot/netdb_bundle_decode/10000: Collecting 100 samples in estimated 6.9010 s (1cortex_snapshot/netdb_bundle_decode/10000
                        time:   [455.12 µs 455.43 µs 455.77 µs]
                        thrpt:  [21.941 Melem/s 21.957 Melem/s 21.972 Melem/s]
                 change:
                        time:   [+30.903% +31.266% +31.615%] (p = 0.00 < 0.05)
                        thrpt:  [−24.021% −23.819% −23.608%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe

     Running benches\ingestion.rs (target\release\deps\ingestion-5302a75653e49025.exe)
Gnuplot not found, using plotters backend
ring_buffer/push/1024   time:   [1.8093 ns 2.0055 ns 2.1696 ns]
                        thrpt:  [460.92 Melem/s 498.64 Melem/s 552.71 Melem/s]
Found 25 outliers among 100 measurements (25.00%)
  1 (1.00%) high mild
  24 (24.00%) high severe
ring_buffer/push_pop/1024
                        time:   [2.1305 ns 2.1370 ns 2.1452 ns]
                        thrpt:  [466.16 Melem/s 467.95 Melem/s 469.37 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
ring_buffer/push/8192   time:   [2.6287 ns 2.6324 ns 2.6371 ns]
                        thrpt:  [379.20 Melem/s 379.88 Melem/s 380.41 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
  4 (4.00%) high mild
  12 (12.00%) high severe
ring_buffer/push_pop/8192
                        time:   [2.1274 ns 2.1282 ns 2.1292 ns]
                        thrpt:  [469.65 Melem/s 469.87 Melem/s 470.06 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) high mild
  8 (8.00%) high severe
ring_buffer/push/65536  time:   [2.6271 ns 2.6286 ns 2.6302 ns]
                        thrpt:  [380.19 Melem/s 380.43 Melem/s 380.65 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
ring_buffer/push_pop/65536
                        time:   [2.1308 ns 2.1344 ns 2.1387 ns]
                        thrpt:  [467.57 Melem/s 468.52 Melem/s 469.30 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
ring_buffer/push/1048576
                        time:   [2.6065 ns 2.6131 ns 2.6203 ns]
                        thrpt:  [381.64 Melem/s 382.69 Melem/s 383.66 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
Benchmarking ring_buffer/push_pop/1048576: Collecting 100 samples in estimated 5.0000 s (2.3B iterationring_buffer/push_pop/1048576
                        time:   [2.1564 ns 2.1613 ns 2.1673 ns]
                        thrpt:  [461.41 Melem/s 462.68 Melem/s 463.73 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe

timestamp/next          time:   [23.786 ns 23.862 ns 23.946 ns]
                        thrpt:  [41.761 Melem/s 41.908 Melem/s 42.041 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
timestamp/now_raw       time:   [8.0316 ns 8.0604 ns 8.0990 ns]
                        thrpt:  [123.47 Melem/s 124.06 Melem/s 124.51 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe

event/internal_event_new
                        time:   [389.07 ns 394.19 ns 400.14 ns]
                        thrpt:  [2.4991 Melem/s 2.5369 Melem/s 2.5702 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) high mild
  8 (8.00%) high severe
event/json_creation     time:   [234.50 ns 235.88 ns 237.53 ns]
                        thrpt:  [4.2101 Melem/s 4.2394 Melem/s 4.2644 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe

batch/pop_batch/100     time:   [282.28 ns 282.93 ns 283.74 ns]
                        thrpt:  [352.44 Melem/s 353.45 Melem/s 354.26 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) high mild
  13 (13.00%) high severe
batch/pop_batch/1000    time:   [2.3342 µs 2.3374 µs 2.3413 µs]
                        thrpt:  [427.11 Melem/s 427.82 Melem/s 428.41 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
batch/pop_batch/10000   time:   [23.366 µs 23.409 µs 23.459 µs]
                        thrpt:  [426.27 Melem/s 427.18 Melem/s 427.97 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

     Running benches\mesh.rs (target\release\deps\mesh-7b9a2fdff86fc868.exe)
Gnuplot not found, using plotters backend
Benchmarking mesh_reroute/triangle_failure: Collecting 100 samples in estimated 5.0122 s (212k iteratiomesh_reroute/triangle_failure
                        time:   [23.496 µs 24.710 µs 26.201 µs]
                        thrpt:  [38.166 Kelem/s 40.470 Kelem/s 42.560 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking mesh_reroute/10_peers_10_routes: Collecting 100 samples in estimated 5.3958 s (25k iteratimesh_reroute/10_peers_10_routes
                        time:   [216.48 µs 218.56 µs 220.84 µs]
                        thrpt:  [4.5281 Kelem/s 4.5753 Kelem/s 4.6193 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking mesh_reroute/50_peers_100_routes: Collecting 100 samples in estimated 5.1597 s (2500 iteramesh_reroute/50_peers_100_routes
                        time:   [2.0573 ms 2.0616 ms 2.0662 ms]
                        thrpt:  [483.99  elem/s 485.07  elem/s 486.08  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking mesh_proximity/on_pingwave_new: Collecting 100 samples in estimated 5.0028 s (3.0M iteratimesh_proximity/on_pingwave_new
                        time:   [163.06 ns 165.85 ns 169.34 ns]
                        thrpt:  [5.9053 Melem/s 6.0296 Melem/s 6.1328 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking mesh_proximity/on_pingwave_dedup: Collecting 100 samples in estimated 5.0004 s (59M iteratmesh_proximity/on_pingwave_dedup
                        time:   [84.380 ns 84.443 ns 84.510 ns]
                        thrpt:  [11.833 Melem/s 11.842 Melem/s 11.851 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking mesh_proximity/pingwave_serialize: Collecting 100 samples in estimated 5.0000 s (2.9B itermesh_proximity/pingwave_serialize
                        time:   [1.4961 ns 1.5315 ns 1.5671 ns]
                        thrpt:  [638.14 Melem/s 652.94 Melem/s 668.40 Melem/s]
Benchmarking mesh_proximity/pingwave_deserialize: Collecting 100 samples in estimated 5.0000 s (2.9B itmesh_proximity/pingwave_deserialize
                        time:   [1.5927 ns 1.6363 ns 1.6743 ns]
                        thrpt:  [597.26 Melem/s 611.15 Melem/s 627.86 Melem/s]
mesh_proximity/node_count
                        time:   [1.3140 µs 1.3160 µs 1.3179 µs]
                        thrpt:  [758.80 Kelem/s 759.90 Kelem/s 761.06 Kelem/s]
Benchmarking mesh_proximity/all_nodes_100: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 42.9s, or reduce sample count to 10.
Benchmarking mesh_proximity/all_nodes_100: Collecting 100 samples in estimated 42.895 s (100 iterationsmesh_proximity/all_nodes_100
                        time:   [429.84 ms 436.96 ms 444.14 ms]
                        thrpt:  [2.2515  elem/s 2.2885  elem/s 2.3265  elem/s]

Benchmarking mesh_dispatch/classify_direct: Collecting 100 samples in estimated 5.0000 s (7.9B iteratiomesh_dispatch/classify_direct
                        time:   [627.06 ps 628.45 ps 630.13 ps]
                        thrpt:  [1.5870 Gelem/s 1.5912 Gelem/s 1.5948 Gelem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking mesh_dispatch/classify_routed: Collecting 100 samples in estimated 5.0000 s (9.8B iteratiomesh_dispatch/classify_routed
                        time:   [504.91 ps 506.31 ps 507.87 ps]
                        thrpt:  [1.9690 Gelem/s 1.9751 Gelem/s 1.9805 Gelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking mesh_dispatch/classify_pingwave: Collecting 100 samples in estimated 5.0000 s (16B iteratimesh_dispatch/classify_pingwave
                        time:   [319.01 ps 319.68 ps 320.45 ps]
                        thrpt:  [3.1206 Gelem/s 3.1282 Gelem/s 3.1347 Gelem/s]
Found 13 outliers among 100 measurements (13.00%)
  3 (3.00%) high mild
  10 (10.00%) high severe

mesh_routing/lookup_hit time:   [25.984 ns 26.249 ns 26.559 ns]
                        thrpt:  [37.652 Melem/s 38.097 Melem/s 38.485 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  17 (17.00%) high mild
  4 (4.00%) high severe
mesh_routing/lookup_miss
                        time:   [25.958 ns 26.199 ns 26.482 ns]
                        thrpt:  [37.761 Melem/s 38.170 Melem/s 38.523 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  1 (1.00%) high mild
  21 (21.00%) high severe
mesh_routing/is_local   time:   [509.18 ps 511.02 ps 512.99 ps]
                        thrpt:  [1.9494 Gelem/s 1.9569 Gelem/s 1.9639 Gelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
mesh_routing/all_routes/10
                        time:   [9.6197 µs 9.6550 µs 9.6939 µs]
                        thrpt:  [103.16 Kelem/s 103.57 Kelem/s 103.95 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking mesh_routing/all_routes/100: Collecting 100 samples in estimated 5.0107 s (429k iterationsmesh_routing/all_routes/100
                        time:   [11.669 µs 11.703 µs 11.741 µs]
                        thrpt:  [85.171 Kelem/s 85.447 Kelem/s 85.697 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  11 (11.00%) high severe
Benchmarking mesh_routing/all_routes/1000: Collecting 100 samples in estimated 5.1054 s (162k iterationmesh_routing/all_routes/1000
                        time:   [31.362 µs 31.465 µs 31.570 µs]
                        thrpt:  [31.676 Kelem/s 31.781 Kelem/s 31.886 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
mesh_routing/add_route  time:   [67.238 ns 67.610 ns 68.123 ns]
                        thrpt:  [14.679 Melem/s 14.791 Melem/s 14.872 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

     Running benches\net.rs (target\release\deps\net-ae108cd6536abf8c.exe)
Gnuplot not found, using plotters backend
net_header/serialize    time:   [1.2672 ns 1.3040 ns 1.3355 ns]
                        thrpt:  [748.77 Melem/s 766.88 Melem/s 789.15 Melem/s]
net_header/deserialize  time:   [1.5120 ns 1.5163 ns 1.5210 ns]
                        thrpt:  [657.45 Melem/s 659.49 Melem/s 661.40 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
net_header/roundtrip    time:   [1.5095 ns 1.5129 ns 1.5171 ns]
                        thrpt:  [659.15 Melem/s 661.00 Melem/s 662.47 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe

Benchmarking net_event_frame/write_single/64: Collecting 100 samples in estimated 5.0003 s (83M iteratinet_event_frame/write_single/64
                        time:   [59.452 ns 60.560 ns 61.816 ns]
                        thrpt:  [987.37 MiB/s 1007.8 MiB/s 1.0026 GiB/s]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  17 (17.00%) high severe
Benchmarking net_event_frame/write_single/256: Collecting 100 samples in estimated 5.0002 s (85M iteratnet_event_frame/write_single/256
                        time:   [58.272 ns 58.715 ns 59.361 ns]
                        thrpt:  [4.0164 GiB/s 4.0606 GiB/s 4.0914 GiB/s]
Found 11 outliers among 100 measurements (11.00%)
  2 (2.00%) high mild
  9 (9.00%) high severe
Benchmarking net_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0001 s (71M iteranet_event_frame/write_single/1024
                        time:   [69.341 ns 71.344 ns 73.280 ns]
                        thrpt:  [13.014 GiB/s 13.367 GiB/s 13.753 GiB/s]
Found 18 outliers among 100 measurements (18.00%)
  3 (3.00%) high mild
  15 (15.00%) high severe
Benchmarking net_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0005 s (47M iteranet_event_frame/write_single/4096
                        time:   [104.51 ns 105.25 ns 106.17 ns]
                        thrpt:  [35.930 GiB/s 36.245 GiB/s 36.499 GiB/s]
Benchmarking net_event_frame/write_batch/1: Collecting 100 samples in estimated 5.0001 s (86M iterationnet_event_frame/write_batch/1
                        time:   [56.869 ns 58.307 ns 60.009 ns]
                        thrpt:  [1017.1 MiB/s 1.0223 GiB/s 1.0481 GiB/s]
Found 14 outliers among 100 measurements (14.00%)
  3 (3.00%) high mild
  11 (11.00%) high severe
Benchmarking net_event_frame/write_batch/10: Collecting 100 samples in estimated 5.0000 s (44M iterationet_event_frame/write_batch/10
                        time:   [106.99 ns 108.32 ns 109.76 ns]
                        thrpt:  [5.4305 GiB/s 5.5027 GiB/s 5.5711 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking net_event_frame/write_batch/50: Collecting 100 samples in estimated 5.0013 s (14M iterationet_event_frame/write_batch/50
                        time:   [358.03 ns 373.70 ns 392.86 ns]
                        thrpt:  [7.5861 GiB/s 7.9749 GiB/s 8.3239 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking net_event_frame/write_batch/100: Collecting 100 samples in estimated 5.0019 s (7.3M iteratnet_event_frame/write_batch/100
                        time:   [716.45 ns 763.52 ns 815.18 ns]
                        thrpt:  [7.3118 GiB/s 7.8066 GiB/s 8.3195 GiB/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  11 (11.00%) high severe
Benchmarking net_event_frame/read_batch_10: Collecting 100 samples in estimated 5.0011 s (21M iterationnet_event_frame/read_batch_10
                        time:   [235.49 ns 235.80 ns 236.14 ns]
                        thrpt:  [42.347 Melem/s 42.410 Melem/s 42.465 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking net_packet_pool/get_return/16: Collecting 100 samples in estimated 5.0003 s (65M iterationnet_packet_pool/get_return/16
                        time:   [77.184 ns 78.513 ns 80.148 ns]
                        thrpt:  [12.477 Melem/s 12.737 Melem/s 12.956 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking net_packet_pool/get_return/64: Collecting 100 samples in estimated 5.0001 s (66M iterationnet_packet_pool/get_return/64
                        time:   [76.000 ns 76.186 ns 76.393 ns]
                        thrpt:  [13.090 Melem/s 13.126 Melem/s 13.158 Melem/s]
Benchmarking net_packet_pool/get_return/256: Collecting 100 samples in estimated 5.0002 s (65M iterationet_packet_pool/get_return/256
                        time:   [77.433 ns 78.025 ns 78.755 ns]
                        thrpt:  [12.698 Melem/s 12.816 Melem/s 12.914 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

Benchmarking net_packet_build/build_packet/1: Collecting 100 samples in estimated 5.0037 s (4.5M iteratnet_packet_build/build_packet/1
                        time:   [1.1007 µs 1.1052 µs 1.1103 µs]
                        thrpt:  [54.969 MiB/s 55.225 MiB/s 55.452 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  7 (7.00%) high mild
  8 (8.00%) high severe
Benchmarking net_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0030 s (2.7M iteranet_packet_build/build_packet/10
                        time:   [1.8281 µs 1.8353 µs 1.8436 µs]
                        thrpt:  [331.06 MiB/s 332.57 MiB/s 333.88 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  5 (5.00%) high mild
  10 (10.00%) high severe
Benchmarking net_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0046 s (1000k iternet_packet_build/build_packet/50
                        time:   [4.9824 µs 4.9927 µs 5.0050 µs]
                        thrpt:  [609.74 MiB/s 611.25 MiB/s 612.51 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe

net_encryption/encrypt/64
                        time:   [1.0998 µs 1.1040 µs 1.1092 µs]
                        thrpt:  [55.028 MiB/s 55.283 MiB/s 55.495 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
net_encryption/encrypt/256
                        time:   [1.2455 µs 1.2472 µs 1.2491 µs]
                        thrpt:  [195.45 MiB/s 195.76 MiB/s 196.01 MiB/s]
Found 15 outliers among 100 measurements (15.00%)
  15 (15.00%) high mild
Benchmarking net_encryption/encrypt/1024: Collecting 100 samples in estimated 5.0062 s (2.4M iterationsnet_encryption/encrypt/1024
                        time:   [2.0869 µs 2.0986 µs 2.1194 µs]
                        thrpt:  [460.77 MiB/s 465.33 MiB/s 467.95 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking net_encryption/encrypt/4096: Collecting 100 samples in estimated 5.0234 s (914k iterationsnet_encryption/encrypt/4096
                        time:   [5.4800 µs 5.4939 µs 5.5122 µs]
                        thrpt:  [708.65 MiB/s 711.02 MiB/s 712.82 MiB/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

net_keypair/generate    time:   [26.442 µs 26.526 µs 26.620 µs]
                        thrpt:  [37.566 Kelem/s 37.699 Kelem/s 37.818 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe

net_aad/generate        time:   [1.3219 ns 1.3773 ns 1.4277 ns]
                        thrpt:  [700.43 Melem/s 726.07 Melem/s 756.49 Melem/s]

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0001 s (61M pool_comparison/shared_pool_get_return
                        time:   [82.069 ns 82.776 ns 83.548 ns]
                        thrpt:  [11.969 Melem/s 12.081 Melem/s 12.185 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.0001 spool_comparison/thread_local_pool_get_return
                        time:   [122.98 ns 123.64 ns 124.40 ns]
                        thrpt:  [8.0386 Melem/s 8.0878 Melem/s 8.1316 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking pool_comparison/shared_pool_10x: Collecting 100 samples in estimated 5.0041 s (6.0M iteratpool_comparison/shared_pool_10x
                        time:   [743.35 ns 757.69 ns 773.39 ns]
                        thrpt:  [1.2930 Melem/s 1.3198 Melem/s 1.3453 Melem/s]
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0027 s (3.5M pool_comparison/thread_local_pool_10x
                        time:   [1.4682 µs 1.4840 µs 1.5019 µs]
                        thrpt:  [665.81 Kelem/s 673.86 Kelem/s 681.11 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0039 s (4.5M iteracipher_comparison/shared_pool/64
                        time:   [1.1012 µs 1.1047 µs 1.1085 µs]
                        thrpt:  [55.061 MiB/s 55.250 MiB/s 55.428 MiB/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0033 s (4.2M itecipher_comparison/fast_chacha20/64
                        time:   [1.1376 µs 1.1414 µs 1.1456 µs]
                        thrpt:  [53.276 MiB/s 53.473 MiB/s 53.652 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0050 s (4.0M itercipher_comparison/shared_pool/256
                        time:   [1.2447 µs 1.2485 µs 1.2529 µs]
                        thrpt:  [194.86 MiB/s 195.55 MiB/s 196.14 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0025 s (3.8M itcipher_comparison/fast_chacha20/256
                        time:   [1.2962 µs 1.3019 µs 1.3090 µs]
                        thrpt:  [186.51 MiB/s 187.52 MiB/s 188.36 MiB/s]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high severe
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0071 s (2.4M itecipher_comparison/shared_pool/1024
                        time:   [2.0885 µs 2.0937 µs 2.0999 µs]
                        thrpt:  [465.04 MiB/s 466.42 MiB/s 467.59 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0050 s (2.3M icipher_comparison/fast_chacha20/1024
                        time:   [2.1053 µs 2.1115 µs 2.1185 µs]
                        thrpt:  [460.97 MiB/s 462.50 MiB/s 463.86 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0128 s (909k itecipher_comparison/shared_pool/4096
                        time:   [5.4854 µs 5.4989 µs 5.5151 µs]
                        thrpt:  [708.28 MiB/s 710.37 MiB/s 712.12 MiB/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0019 s (919k icipher_comparison/fast_chacha20/4096
                        time:   [5.4046 µs 5.4300 µs 5.4624 µs]
                        thrpt:  [715.12 MiB/s 719.39 MiB/s 722.76 MiB/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe

Benchmarking adaptive_batcher/optimal_size: Collecting 100 samples in estimated 5.0000 s (3.2B iteratioadaptive_batcher/optimal_size
                        time:   [1.5686 ns 1.5727 ns 1.5775 ns]
                        thrpt:  [633.93 Melem/s 635.85 Melem/s 637.50 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high severe
adaptive_batcher/record time:   [14.248 ns 14.312 ns 14.372 ns]
                        thrpt:  [69.580 Melem/s 69.870 Melem/s 70.185 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) low severe
  7 (7.00%) low mild
Benchmarking adaptive_batcher/full_cycle: Collecting 100 samples in estimated 5.0001 s (375M iterationsadaptive_batcher/full_cycle
                        time:   [13.324 ns 13.358 ns 13.394 ns]
                        thrpt:  [74.662 Melem/s 74.861 Melem/s 75.051 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0037 s (1000e2e_packet_build/shared_pool_50_events
                        time:   [3.1728 µs 3.2966 µs 3.4597 µs]
                        thrpt:  [882.08 MiB/s 925.72 MiB/s 961.84 MiB/s]
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0042 s (1.7M iterate2e_packet_build/fast_50_events
                        time:   [2.8847 µs 2.8896 µs 2.8963 µs]
                        thrpt:  [1.0290 GiB/s 1.0314 GiB/s 1.0331 GiB/s]
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) high mild
  13 (13.00%) high severe

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 8.0861 s (5050multithread_packet_build/shared_pool/8
                        time:   [1.6164 ms 1.6268 ms 1.6385 ms]
                        thrpt:  [4.8825 Melem/s 4.9175 Melem/s 4.9493 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.5s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 7.5293 smultithread_packet_build/thread_local_pool/8
                        time:   [1.4389 ms 1.4419 ms 1.4452 ms]
                        thrpt:  [5.5358 Melem/s 5.5482 Melem/s 5.5597 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.2272 s (200multithread_packet_build/shared_pool/16
                        time:   [2.6072 ms 2.6111 ms 2.6154 ms]
                        thrpt:  [6.1176 Melem/s 6.1276 Melem/s 6.1369 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.7s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 9.6665 multithread_packet_build/thread_local_pool/16
                        time:   [1.9218 ms 1.9341 ms 1.9491 ms]
                        thrpt:  [8.2091 Melem/s 8.2725 Melem/s 8.3256 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.0372 s (130multithread_packet_build/shared_pool/24
                        time:   [3.8703 ms 3.8851 ms 3.9032 ms]
                        thrpt:  [6.1489 Melem/s 6.1775 Melem/s 6.2011 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  4 (4.00%) low mild
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.1153 multithread_packet_build/thread_local_pool/24
                        time:   [2.8068 ms 2.8158 ms 2.8255 ms]
                        thrpt:  [8.4939 Melem/s 8.5233 Melem/s 8.5506 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.0487 s (900multithread_packet_build/shared_pool/32
                        time:   [5.2953 ms 5.5873 ms 5.9073 ms]
                        thrpt:  [5.4170 Melem/s 5.7273 Melem/s 6.0431 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  3 (3.00%) high mild
  21 (21.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.0975 multithread_packet_build/thread_local_pool/32
                        time:   [3.2353 ms 3.2644 ms 3.3002 ms]
                        thrpt:  [9.6965 Melem/s 9.8027 Melem/s 9.8909 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.6s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 5.6151 s (505multithread_mixed_frames/shared_mixed/8
                        time:   [1.0927 ms 1.1054 ms 1.1198 ms]
                        thrpt:  [10.716 Melem/s 10.856 Melem/s 10.982 Melem/s]
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.3s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 5.2785 s (5050 multithread_mixed_frames/fast_mixed/8
                        time:   [970.21 µs 975.98 µs 982.39 µs]
                        thrpt:  [12.215 Melem/s 12.295 Melem/s 12.368 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.0s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samples in estimated 7.9785 s (50multithread_mixed_frames/shared_mixed/16
                        time:   [1.5744 ms 1.5830 ms 1.5935 ms]
                        thrpt:  [15.061 Melem/s 15.161 Melem/s 15.244 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking multithread_mixed_frames/fast_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.9s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples in estimated 6.9161 s (5050multithread_mixed_frames/fast_mixed/16
                        time:   [1.3612 ms 1.3642 ms 1.3678 ms]
                        thrpt:  [17.547 Melem/s 17.592 Melem/s 17.631 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  6 (6.00%) low mild
  6 (6.00%) high mild
  6 (6.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samples in estimated 5.0553 s (24multithread_mixed_frames/shared_mixed/24
                        time:   [2.0898 ms 2.1062 ms 2.1350 ms]
                        thrpt:  [16.861 Melem/s 17.093 Melem/s 17.227 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.4s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples in estimated 9.4307 s (5050multithread_mixed_frames/fast_mixed/24
                        time:   [1.8223 ms 1.8407 ms 1.8639 ms]
                        thrpt:  [19.314 Melem/s 19.558 Melem/s 19.755 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samples in estimated 5.2330 s (16multithread_mixed_frames/shared_mixed/32
                        time:   [3.0679 ms 3.2450 ms 3.4342 ms]
                        thrpt:  [13.977 Melem/s 14.792 Melem/s 15.646 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples in estimated 5.0646 s (2300multithread_mixed_frames/fast_mixed/32
                        time:   [2.1967 ms 2.2052 ms 2.2141 ms]
                        thrpt:  [21.679 Melem/s 21.766 Melem/s 21.851 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samples in estimated 5.7640 s (60pool_contention/shared_acquire_release/8
                        time:   [9.5958 ms 9.6126 ms 9.6291 ms]
                        thrpt:  [8.3082 Melem/s 8.3224 Melem/s 8.3369 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking pool_contention/fast_acquire_release/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.7s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples in estimated 6.6718 s (5050pool_contention/fast_acquire_release/8
                        time:   [1.2353 ms 1.2513 ms 1.2682 ms]
                        thrpt:  [63.083 Melem/s 63.934 Melem/s 64.762 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samples in estimated 6.6835 s (3pool_contention/shared_acquire_release/16
                        time:   [22.224 ms 22.275 ms 22.325 ms]
                        thrpt:  [7.1667 Melem/s 7.1828 Melem/s 7.1993 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples in estimated 5.1279 s (250pool_contention/fast_acquire_release/16
                        time:   [2.0722 ms 2.0820 ms 2.0914 ms]
                        thrpt:  [76.504 Melem/s 76.851 Melem/s 77.213 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samples in estimated 7.0931 s (2pool_contention/shared_acquire_release/24
                        time:   [35.357 ms 35.430 ms 35.501 ms]
                        thrpt:  [6.7604 Melem/s 6.7738 Melem/s 6.7879 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples in estimated 5.1241 s (210pool_contention/fast_acquire_release/24
                        time:   [2.4364 ms 2.4509 ms 2.4660 ms]
                        thrpt:  [97.322 Melem/s 97.922 Melem/s 98.504 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samples in estimated 9.8481 s (2pool_contention/shared_acquire_release/32
                        time:   [48.969 ms 49.066 ms 49.162 ms]
                        thrpt:  [6.5091 Melem/s 6.5219 Melem/s 6.5348 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples in estimated 5.0579 s (160pool_contention/fast_acquire_release/32
                        time:   [3.1295 ms 3.1466 ms 3.1639 ms]
                        thrpt:  [101.14 Melem/s 101.70 Melem/s 102.25 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples in estimated 5.3510 s (1470 throughput_scaling/fast_pool_scaling/1
                        time:   [3.6089 ms 3.6200 ms 3.6321 ms]
                        thrpt:  [550.64 Kelem/s 552.49 Kelem/s 554.19 Kelem/s]
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples in estimated 5.5094 s (1470 throughput_scaling/fast_pool_scaling/2
                        time:   [3.7271 ms 3.7355 ms 3.7450 ms]
                        thrpt:  [1.0681 Melem/s 1.0708 Melem/s 1.0732 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples in estimated 5.5057 s (1470 throughput_scaling/fast_pool_scaling/4
                        time:   [3.7326 ms 3.7423 ms 3.7515 ms]
                        thrpt:  [2.1325 Melem/s 2.1377 Melem/s 2.1433 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples in estimated 5.1457 s (1260 throughput_scaling/fast_pool_scaling/8
                        time:   [4.0081 ms 4.0659 ms 4.1223 ms]
                        thrpt:  [3.8813 Melem/s 3.9352 Melem/s 3.9919 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples in estimated 5.7634 s (1050throughput_scaling/fast_pool_scaling/16
                        time:   [5.4869 ms 5.4934 ms 5.4997 ms]
                        thrpt:  [5.8185 Melem/s 5.8252 Melem/s 5.8321 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples in estimated 5.2202 s (630 throughput_scaling/fast_pool_scaling/24
                        time:   [13.971 ms 17.527 ms 19.638 ms]
                        thrpt:  [2.4443 Melem/s 2.7386 Melem/s 3.4356 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/32: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.7s, enable flat sampling, or reduce sample count to 10.
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples in estimated 5.7128 s (210 throughput_scaling/fast_pool_scaling/32
                        time:   [26.395 ms 26.572 ms 26.846 ms]
                        thrpt:  [2.3840 Melem/s 2.4085 Melem/s 2.4247 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high severe

routing_header/serialize
                        time:   [550.96 ps 586.33 ps 629.06 ps]
                        thrpt:  [1.5897 Gelem/s 1.7055 Gelem/s 1.8150 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe
routing_header/deserialize
                        time:   [907.09 ps 917.56 ps 929.89 ps]
                        thrpt:  [1.0754 Gelem/s 1.0898 Gelem/s 1.1024 Gelem/s]
Found 21 outliers among 100 measurements (21.00%)
  1 (1.00%) high mild
  20 (20.00%) high severe
routing_header/roundtrip
                        time:   [906.53 ps 915.00 ps 924.73 ps]
                        thrpt:  [1.0814 Gelem/s 1.0929 Gelem/s 1.1031 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe
routing_header/forward  time:   [505.07 ps 506.13 ps 507.25 ps]
                        thrpt:  [1.9714 Gelem/s 1.9758 Gelem/s 1.9799 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

routing_table/lookup_hit
                        time:   [53.053 ns 57.792 ns 63.151 ns]
                        thrpt:  [15.835 Melem/s 17.304 Melem/s 18.849 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  18 (18.00%) low severe
routing_table/lookup_miss
                        time:   [17.287 ns 17.394 ns 17.497 ns]
                        thrpt:  [57.151 Melem/s 57.493 Melem/s 57.848 Melem/s]
routing_table/is_local  time:   [199.01 ps 202.26 ps 206.08 ps]
                        thrpt:  [4.8525 Gelem/s 4.9441 Gelem/s 5.0248 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
routing_table/add_route time:   [206.53 ns 208.69 ns 210.89 ns]
                        thrpt:  [4.7419 Melem/s 4.7917 Melem/s 4.8420 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  7 (7.00%) low severe
  10 (10.00%) low mild
  2 (2.00%) high mild
routing_table/record_in time:   [40.412 ns 40.591 ns 40.759 ns]
                        thrpt:  [24.534 Melem/s 24.636 Melem/s 24.745 Melem/s]
routing_table/record_out
                        time:   [20.702 ns 20.803 ns 20.897 ns]
                        thrpt:  [47.855 Melem/s 48.071 Melem/s 48.306 Melem/s]
Found 21 outliers among 100 measurements (21.00%)
  19 (19.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking routing_table/aggregate_stats: Collecting 100 samples in estimated 5.0361 s (631k iteratiorouting_table/aggregate_stats
                        time:   [7.9966 µs 8.0281 µs 8.0571 µs]
                        thrpt:  [124.11 Kelem/s 124.56 Kelem/s 125.05 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) low mild
  1 (1.00%) high mild

fair_scheduler/creation time:   [1.5524 µs 1.5589 µs 1.5650 µs]
                        thrpt:  [638.99 Kelem/s 641.48 Kelem/s 644.18 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) low mild
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estimated 5.0035 s (5.3M iterfair_scheduler/stream_count_empty
                        time:   [937.77 ns 942.24 ns 946.47 ns]
                        thrpt:  [1.0566 Melem/s 1.0613 Melem/s 1.0664 Melem/s]
fair_scheduler/total_queued
                        time:   [197.19 ps 198.13 ps 198.98 ps]
                        thrpt:  [5.0256 Gelem/s 5.0473 Gelem/s 5.0713 Gelem/s]
Benchmarking fair_scheduler/cleanup_empty: Collecting 100 samples in estimated 5.0047 s (4.0M iterationfair_scheduler/cleanup_empty
                        time:   [1.2637 µs 1.2699 µs 1.2757 µs]
                        thrpt:  [783.91 Kelem/s 787.47 Kelem/s 791.32 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  12 (12.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samples in estimated 5.3143 srouting_table_concurrent/concurrent_lookup/4
                        time:   [175.82 µs 176.20 µs 176.62 µs]
                        thrpt:  [22.648 Melem/s 22.701 Melem/s 22.751 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samples in estimated 5.6090 s routing_table_concurrent/concurrent_stats/4
                        time:   [221.58 µs 221.97 µs 222.44 µs]
                        thrpt:  [17.982 Melem/s 18.020 Melem/s 18.053 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samples in estimated 5.5318 srouting_table_concurrent/concurrent_lookup/8
                        time:   [271.32 µs 272.64 µs 274.21 µs]
                        thrpt:  [29.175 Melem/s 29.343 Melem/s 29.486 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  2 (2.00%) high mild
  11 (11.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samples in estimated 6.4019 s routing_table_concurrent/concurrent_stats/8
                        time:   [315.73 µs 316.76 µs 317.99 µs]
                        thrpt:  [25.158 Melem/s 25.256 Melem/s 25.338 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  4 (4.00%) high mild
  11 (11.00%) high severe
Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 samples in estimated 5.0061 routing_table_concurrent/concurrent_lookup/16
                        time:   [495.41 µs 498.88 µs 502.78 µs]
                        thrpt:  [31.823 Melem/s 32.072 Melem/s 32.296 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samples in estimated 5.4988 srouting_table_concurrent/concurrent_stats/16
                        time:   [540.89 µs 543.29 µs 546.08 µs]
                        thrpt:  [29.300 Melem/s 29.450 Melem/s 29.581 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples in estimated 5.0002 s (132M routing_decision/parse_lookup_forward
                        time:   [37.545 ns 37.739 ns 37.935 ns]
                        thrpt:  [26.361 Melem/s 26.498 Melem/s 26.635 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estimated 5.0000 s (50M iteratrouting_decision/full_with_stats
                        time:   [98.565 ns 99.043 ns 99.478 ns]
                        thrpt:  [10.052 Melem/s 10.097 Melem/s 10.146 Melem/s]

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in estimated 5.0009 s (15M iterastream_multiplexing/lookup_all/10
                        time:   [334.18 ns 335.82 ns 337.47 ns]
                        thrpt:  [29.632 Melem/s 29.777 Melem/s 29.924 Melem/s]
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estimated 5.0020 s (12M iteratstream_multiplexing/stats_all/10
                        time:   [399.19 ns 401.09 ns 403.11 ns]
                        thrpt:  [24.807 Melem/s 24.932 Melem/s 25.051 Melem/s]
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in estimated 5.0061 s (1.5M itestream_multiplexing/lookup_all/100
                        time:   [3.3646 µs 3.3827 µs 3.4000 µs]
                        thrpt:  [29.412 Melem/s 29.562 Melem/s 29.722 Melem/s]
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in estimated 5.0044 s (1.2M iterstream_multiplexing/stats_all/100
                        time:   [4.0430 µs 4.0649 µs 4.0857 µs]
                        thrpt:  [24.476 Melem/s 24.601 Melem/s 24.734 Melem/s]
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estimated 5.0211 s (141k itstream_multiplexing/lookup_all/1000
                        time:   [35.365 µs 35.538 µs 35.712 µs]
                        thrpt:  [28.002 Melem/s 28.139 Melem/s 28.276 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in estimated 5.1056 s (116k itestream_multiplexing/stats_all/1000
                        time:   [43.538 µs 43.780 µs 43.990 µs]
                        thrpt:  [22.732 Melem/s 22.842 Melem/s 22.968 Melem/s]
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in estimated 5.8567 s (15k itstream_multiplexing/lookup_all/10000
                        time:   [383.80 µs 385.64 µs 387.36 µs]
                        thrpt:  [25.815 Melem/s 25.931 Melem/s 26.055 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estimated 7.0387 s (15k itestream_multiplexing/stats_all/10000
                        time:   [460.99 µs 463.01 µs 464.98 µs]
                        thrpt:  [21.506 Melem/s 21.598 Melem/s 21.692 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimated 5.0001 s (122M iteramultihop_packet_builder/build/64
                        time:   [40.775 ns 40.943 ns 41.101 ns]
                        thrpt:  [1.4502 GiB/s 1.4558 GiB/s 1.4618 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) low mild
  1 (1.00%) high mild
Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samples in estimated 5.0001 s (1multihop_packet_builder/build_priority/64
                        time:   [29.908 ns 30.095 ns 30.292 ns]
                        thrpt:  [1.9677 GiB/s 1.9805 GiB/s 1.9929 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estimated 5.0002 s (119M itermultihop_packet_builder/build/256
                        time:   [41.755 ns 41.987 ns 42.217 ns]
                        thrpt:  [5.6474 GiB/s 5.6783 GiB/s 5.7099 GiB/s]
Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samples in estimated 5.0001 s (multihop_packet_builder/build_priority/256
                        time:   [31.522 ns 31.901 ns 32.243 ns]
                        thrpt:  [7.3943 GiB/s 7.4737 GiB/s 7.5636 GiB/s]
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in estimated 5.0000 s (114M itemultihop_packet_builder/build/1024
                        time:   [43.576 ns 43.774 ns 43.968 ns]
                        thrpt:  [21.690 GiB/s 21.786 GiB/s 21.885 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 samples in estimated 5.0000 s multihop_packet_builder/build_priority/1024
                        time:   [34.714 ns 34.897 ns 35.072 ns]
                        thrpt:  [27.192 GiB/s 27.328 GiB/s 27.472 GiB/s]
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in estimated 5.0003 s (76M itermultihop_packet_builder/build/4096
                        time:   [65.881 ns 66.192 ns 66.504 ns]
                        thrpt:  [57.360 GiB/s 57.630 GiB/s 57.903 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 samples in estimated 5.0000 s multihop_packet_builder/build_priority/4096
                        time:   [54.157 ns 54.450 ns 54.751 ns]
                        thrpt:  [69.674 GiB/s 70.058 GiB/s 70.438 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking multihop_chain/forward_chain/1: Collecting 100 samples in estimated 5.0002 s (93M iteratiomultihop_chain/forward_chain/1
                        time:   [51.878 ns 52.147 ns 52.428 ns]
                        thrpt:  [19.074 Melem/s 19.177 Melem/s 19.276 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multihop_chain/forward_chain/2: Collecting 100 samples in estimated 5.0000 s (59M iteratiomultihop_chain/forward_chain/2
                        time:   [84.181 ns 84.541 ns 84.919 ns]
                        thrpt:  [11.776 Melem/s 11.829 Melem/s 11.879 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking multihop_chain/forward_chain/3: Collecting 100 samples in estimated 5.0004 s (42M iteratiomultihop_chain/forward_chain/3
                        time:   [118.72 ns 119.24 ns 119.75 ns]
                        thrpt:  [8.3510 Melem/s 8.3867 Melem/s 8.4232 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multihop_chain/forward_chain/4: Collecting 100 samples in estimated 5.0007 s (33M iteratiomultihop_chain/forward_chain/4
                        time:   [150.61 ns 151.43 ns 152.28 ns]
                        thrpt:  [6.5669 Melem/s 6.6037 Melem/s 6.6396 Melem/s]
Benchmarking multihop_chain/forward_chain/5: Collecting 100 samples in estimated 5.0005 s (27M iteratiomultihop_chain/forward_chain/5
                        time:   [185.46 ns 186.41 ns 187.35 ns]
                        thrpt:  [5.3376 Melem/s 5.3646 Melem/s 5.3919 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking hop_latency/single_hop_process: Collecting 100 samples in estimated 5.0000 s (5.0B iteratihop_latency/single_hop_process
                        time:   [942.11 ps 953.13 ps 963.50 ps]
                        thrpt:  [1.0379 Gelem/s 1.0492 Gelem/s 1.0615 Gelem/s]
Benchmarking hop_latency/single_hop_full: Collecting 100 samples in estimated 5.0001 s (153M iterationshop_latency/single_hop_full
                        time:   [32.716 ns 32.966 ns 33.233 ns]
                        thrpt:  [30.091 Melem/s 30.335 Melem/s 30.566 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

hop_scaling/64B_1hops   time:   [51.054 ns 51.335 ns 51.600 ns]
                        thrpt:  [1.1551 GiB/s 1.1611 GiB/s 1.1675 GiB/s]
hop_scaling/64B_2hops   time:   [82.222 ns 82.718 ns 83.195 ns]
                        thrpt:  [733.64 MiB/s 737.87 MiB/s 742.32 MiB/s]
hop_scaling/64B_3hops   time:   [116.11 ns 116.64 ns 117.16 ns]
                        thrpt:  [520.94 MiB/s 523.28 MiB/s 525.65 MiB/s]
hop_scaling/64B_4hops   time:   [146.25 ns 146.93 ns 147.62 ns]
                        thrpt:  [413.47 MiB/s 415.40 MiB/s 417.34 MiB/s]
hop_scaling/64B_5hops   time:   [179.47 ns 180.47 ns 181.45 ns]
                        thrpt:  [336.37 MiB/s 338.20 MiB/s 340.08 MiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/256B_1hops  time:   [52.256 ns 52.546 ns 52.876 ns]
                        thrpt:  [4.5090 GiB/s 4.5373 GiB/s 4.5625 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/256B_2hops  time:   [85.222 ns 85.848 ns 86.532 ns]
                        thrpt:  [2.7553 GiB/s 2.7772 GiB/s 2.7976 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/256B_3hops  time:   [118.37 ns 119.12 ns 119.92 ns]
                        thrpt:  [1.9881 GiB/s 2.0015 GiB/s 2.0141 GiB/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
hop_scaling/256B_4hops  time:   [150.53 ns 151.27 ns 152.02 ns]
                        thrpt:  [1.5684 GiB/s 1.5761 GiB/s 1.5839 GiB/s]
hop_scaling/256B_5hops  time:   [186.27 ns 187.38 ns 188.58 ns]
                        thrpt:  [1.2643 GiB/s 1.2723 GiB/s 1.2800 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
hop_scaling/1024B_1hops time:   [53.773 ns 54.025 ns 54.258 ns]
                        thrpt:  [17.577 GiB/s 17.653 GiB/s 17.735 GiB/s]
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) low severe
  8 (8.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_2hops time:   [88.243 ns 88.689 ns 89.159 ns]
                        thrpt:  [10.696 GiB/s 10.753 GiB/s 10.807 GiB/s]
hop_scaling/1024B_3hops time:   [123.79 ns 124.24 ns 124.73 ns]
                        thrpt:  [7.6459 GiB/s 7.6762 GiB/s 7.7040 GiB/s]
hop_scaling/1024B_4hops time:   [157.77 ns 158.56 ns 159.34 ns]
                        thrpt:  [5.9853 GiB/s 6.0145 GiB/s 6.0449 GiB/s]
hop_scaling/1024B_5hops time:   [195.62 ns 196.44 ns 197.33 ns]
                        thrpt:  [4.8330 GiB/s 4.8548 GiB/s 4.8752 GiB/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samples in estimated 5.0003 s (3multihop_with_routing/route_and_forward/1
                        time:   [152.19 ns 152.94 ns 153.65 ns]
                        thrpt:  [6.5082 Melem/s 6.5384 Melem/s 6.5707 Melem/s]
Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samples in estimated 5.0013 s (1multihop_with_routing/route_and_forward/2
                        time:   [281.97 ns 283.72 ns 285.53 ns]
                        thrpt:  [3.5022 Melem/s 3.5247 Melem/s 3.5465 Melem/s]
Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samples in estimated 5.0011 s (1multihop_with_routing/route_and_forward/3
                        time:   [413.06 ns 415.71 ns 418.49 ns]
                        thrpt:  [2.3895 Melem/s 2.4055 Melem/s 2.4210 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samples in estimated 5.0027 s (9multihop_with_routing/route_and_forward/4
                        time:   [552.65 ns 555.89 ns 559.03 ns]
                        thrpt:  [1.7888 Melem/s 1.7989 Melem/s 1.8094 Melem/s]
Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samples in estimated 5.0017 s (7multihop_with_routing/route_and_forward/5
                        time:   [680.25 ns 684.26 ns 688.37 ns]
                        thrpt:  [1.4527 Melem/s 1.4614 Melem/s 1.4700 Melem/s]

Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples in estimated 5.0550 s (903multihop_concurrent/concurrent_forward/4
                        time:   [558.20 µs 558.79 µs 559.30 µs]
                        thrpt:  [7.1518 Melem/s 7.1583 Melem/s 7.1659 Melem/s]
Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples in estimated 5.1454 s (735multihop_concurrent/concurrent_forward/8
                        time:   [696.14 µs 699.87 µs 703.52 µs]
                        thrpt:  [11.371 Melem/s 11.431 Melem/s 11.492 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samples in estimated 5.2255 s (39multihop_concurrent/concurrent_forward/16
                        time:   [1.2905 ms 1.2969 ms 1.3044 ms]
                        thrpt:  [12.266 Melem/s 12.337 Melem/s 12.398 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

pingwave/serialize      time:   [521.44 ps 534.31 ps 546.70 ps]
                        thrpt:  [1.8292 Gelem/s 1.8716 Gelem/s 1.9178 Gelem/s]
pingwave/deserialize    time:   [613.20 ps 634.49 ps 658.06 ps]
                        thrpt:  [1.5196 Gelem/s 1.5761 Gelem/s 1.6308 Gelem/s]
pingwave/roundtrip      time:   [612.39 ps 633.79 ps 657.06 ps]
                        thrpt:  [1.5219 Gelem/s 1.5778 Gelem/s 1.6329 Gelem/s]
pingwave/forward        time:   [512.45 ps 525.52 ps 537.96 ps]
                        thrpt:  [1.8589 Gelem/s 1.9029 Gelem/s 1.9514 Gelem/s]

Benchmarking capabilities/serialize_simple: Collecting 100 samples in estimated 5.0001 s (138M iteratiocapabilities/serialize_simple
                        time:   [35.835 ns 35.970 ns 36.121 ns]
                        thrpt:  [27.685 Melem/s 27.801 Melem/s 27.905 Melem/s]
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimated 5.0000 s (597M iteratcapabilities/deserialize_simple
                        time:   [8.6740 ns 8.7685 ns 8.8680 ns]
                        thrpt:  [112.77 Melem/s 114.04 Melem/s 115.29 Melem/s]
Benchmarking capabilities/serialize_complex: Collecting 100 samples in estimated 5.0001 s (131M iteraticapabilities/serialize_complex
                        time:   [37.775 ns 37.950 ns 38.139 ns]
                        thrpt:  [26.220 Melem/s 26.351 Melem/s 26.473 Melem/s]
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in estimated 5.0002 s (22M iteratcapabilities/deserialize_complex
                        time:   [227.24 ns 228.21 ns 229.22 ns]
                        thrpt:  [4.3626 Melem/s 4.3820 Melem/s 4.4006 Melem/s]

Benchmarking local_graph/create_pingwave: Collecting 100 samples in estimated 5.0000 s (1.0B iterationslocal_graph/create_pingwave
                        time:   [4.8639 ns 4.8889 ns 4.9143 ns]
                        thrpt:  [203.49 Melem/s 204.54 Melem/s 205.60 Melem/s]
local_graph/on_pingwave_new
                        time:   [172.51 ns 174.15 ns 175.68 ns]
                        thrpt:  [5.6921 Melem/s 5.7420 Melem/s 5.7966 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) low mild
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estimated 5.0000 s (317M iterlocal_graph/on_pingwave_duplicate
                        time:   [16.666 ns 17.026 ns 17.421 ns]
                        thrpt:  [57.403 Melem/s 58.734 Melem/s 60.004 Melem/s]
local_graph/get_node    time:   [14.621 ns 14.762 ns 14.921 ns]
                        thrpt:  [67.022 Melem/s 67.743 Melem/s 68.396 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
local_graph/node_count  time:   [930.05 ns 934.80 ns 939.41 ns]
                        thrpt:  [1.0645 Melem/s 1.0697 Melem/s 1.0752 Melem/s]
local_graph/stats       time:   [2.8016 µs 2.8166 µs 2.8309 µs]
                        thrpt:  [353.24 Kelem/s 355.04 Kelem/s 356.93 Kelem/s]

Benchmarking graph_scaling/all_nodes/100: Collecting 100 samples in estimated 5.0237 s (692k iterationsgraph_scaling/all_nodes/100
                        time:   [7.1788 µs 7.2113 µs 7.2461 µs]
                        thrpt:  [13.800 Melem/s 13.867 Melem/s 13.930 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in estimated 5.0231 s (687k itgraph_scaling/nodes_within_hops/100
                        time:   [7.2694 µs 7.3082 µs 7.3506 µs]
                        thrpt:  [13.604 Melem/s 13.683 Melem/s 13.756 Melem/s]
Benchmarking graph_scaling/all_nodes/500: Collecting 100 samples in estimated 5.0517 s (323k iterationsgraph_scaling/all_nodes/500
                        time:   [15.761 µs 15.858 µs 15.956 µs]
                        thrpt:  [31.335 Melem/s 31.530 Melem/s 31.724 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in estimated 5.0037 s (323k itgraph_scaling/nodes_within_hops/500
                        time:   [15.448 µs 15.526 µs 15.610 µs]
                        thrpt:  [32.032 Melem/s 32.204 Melem/s 32.367 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking graph_scaling/all_nodes/1000: Collecting 100 samples in estimated 5.0678 s (197k iterationgraph_scaling/all_nodes/1000
                        time:   [26.044 µs 26.182 µs 26.327 µs]
                        thrpt:  [37.983 Melem/s 38.194 Melem/s 38.396 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in estimated 5.0288 s (192k igraph_scaling/nodes_within_hops/1000
                        time:   [26.075 µs 26.229 µs 26.393 µs]
                        thrpt:  [37.889 Melem/s 38.126 Melem/s 38.351 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking graph_scaling/all_nodes/5000: Collecting 100 samples in estimated 5.4650 s (25k iterationsgraph_scaling/all_nodes/5000
                        time:   [213.35 µs 216.45 µs 220.15 µs]
                        thrpt:  [22.712 Melem/s 23.100 Melem/s 23.436 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in estimated 5.4554 s (25k itgraph_scaling/nodes_within_hops/5000
                        time:   [214.30 µs 216.89 µs 219.93 µs]
                        thrpt:  [22.735 Melem/s 23.054 Melem/s 23.331 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimated 5.1261 s (177k iteratcapability_search/find_with_gpu
                        time:   [28.870 µs 29.005 µs 29.155 µs]
                        thrpt:  [34.300 Kelem/s 34.477 Kelem/s 34.638 Kelem/s]
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples in estimated 5.1403 s (91k icapability_search/find_by_tool_python
                        time:   [56.080 µs 56.336 µs 56.607 µs]
                        thrpt:  [17.666 Kelem/s 17.751 Kelem/s 17.832 Kelem/s]
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in estimated 5.0552 s (71k itecapability_search/find_by_tool_rust
                        time:   [71.488 µs 71.838 µs 72.177 µs]
                        thrpt:  [13.855 Kelem/s 13.920 Kelem/s 13.988 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples in estimated 5.0070 s (30k igraph_concurrent/concurrent_pingwave/4
                        time:   [163.92 µs 164.18 µs 164.44 µs]
                        thrpt:  [12.163 Melem/s 12.181 Melem/s 12.201 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples in estimated 5.0034 s (19k igraph_concurrent/concurrent_pingwave/8
                        time:   [267.48 µs 268.29 µs 269.15 µs]
                        thrpt:  [14.862 Melem/s 14.909 Melem/s 14.954 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples in estimated 5.0789 s (10k graph_concurrent/concurrent_pingwave/16
                        time:   [492.31 µs 495.20 µs 497.93 µs]
                        thrpt:  [16.066 Melem/s 16.155 Melem/s 16.250 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild

path_finding/path_1_hop time:   [5.3505 µs 5.3652 µs 5.3820 µs]
                        thrpt:  [185.81 Kelem/s 186.39 Kelem/s 186.90 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
path_finding/path_2_hops
                        time:   [5.4674 µs 5.4806 µs 5.4957 µs]
                        thrpt:  [181.96 Kelem/s 182.46 Kelem/s 182.90 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
path_finding/path_4_hops
                        time:   [5.6765 µs 5.6925 µs 5.7111 µs]
                        thrpt:  [175.10 Kelem/s 175.67 Kelem/s 176.17 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking path_finding/path_not_found: Collecting 100 samples in estimated 5.0228 s (884k iterationspath_finding/path_not_found
                        time:   [5.6690 µs 5.6874 µs 5.7082 µs]
                        thrpt:  [175.19 Kelem/s 175.83 Kelem/s 176.40 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  7 (7.00%) high mild
  8 (8.00%) high severe
Benchmarking path_finding/path_complex_graph: Collecting 100 samples in estimated 5.6906 s (20k iteratipath_finding/path_complex_graph
                        time:   [276.11 µs 276.85 µs 277.67 µs]
                        thrpt:  [3.6015 Kelem/s 3.6121 Kelem/s 3.6217 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in estimated 5.0001 s (147M itfailure_detector/heartbeat_existing
                        time:   [33.707 ns 33.768 ns 33.837 ns]
                        thrpt:  [29.553 Melem/s 29.614 Melem/s 29.667 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high mild
Benchmarking failure_detector/heartbeat_new: Collecting 100 samples in estimated 5.0002 s (30M iteratiofailure_detector/heartbeat_new
                        time:   [188.08 ns 189.82 ns 191.55 ns]
                        thrpt:  [5.2207 Melem/s 5.2682 Melem/s 5.3169 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low severe
  8 (8.00%) low mild
Benchmarking failure_detector/status_check: Collecting 100 samples in estimated 5.0000 s (387M iteratiofailure_detector/status_check
                        time:   [12.874 ns 12.908 ns 12.945 ns]
                        thrpt:  [77.247 Melem/s 77.469 Melem/s 77.673 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 66.9s, or reduce sample count to 10.
failure_detector/check_all
                        time:   [645.86 ms 647.32 ms 648.79 ms]
                        thrpt:  [1.5413  elem/s 1.5448  elem/s 1.5483  elem/s]
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 19.2s, or reduce sample count to 20.
failure_detector/stats  time:   [191.92 ms 192.16 ms 192.42 ms]
                        thrpt:  [5.1971  elem/s 5.2039  elem/s 5.2104  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimated 5.0000 s (472M iteratloss_simulator/should_drop_1pct
                        time:   [10.682 ns 10.724 ns 10.771 ns]
                        thrpt:  [92.845 Melem/s 93.246 Melem/s 93.614 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimated 5.0001 s (455M iteratloss_simulator/should_drop_5pct
                        time:   [10.951 ns 10.983 ns 11.018 ns]
                        thrpt:  [90.757 Melem/s 91.051 Melem/s 91.317 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estimated 5.0000 s (435M iteraloss_simulator/should_drop_10pct
                        time:   [11.454 ns 11.492 ns 11.533 ns]
                        thrpt:  [86.706 Melem/s 87.018 Melem/s 87.308 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estimated 5.0000 s (401M iteraloss_simulator/should_drop_20pct
                        time:   [12.398 ns 12.430 ns 12.466 ns]
                        thrpt:  [80.221 Melem/s 80.448 Melem/s 80.656 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estimated 5.0000 s (450M iteraloss_simulator/should_drop_burst
                        time:   [11.077 ns 11.114 ns 11.155 ns]
                        thrpt:  [89.648 Melem/s 89.977 Melem/s 90.279 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe

Benchmarking circuit_breaker/allow_closed: Collecting 100 samples in estimated 5.0000 s (507M iterationcircuit_breaker/allow_closed
                        time:   [9.8255 ns 9.8585 ns 9.8955 ns]
                        thrpt:  [101.06 Melem/s 101.44 Melem/s 101.78 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
Benchmarking circuit_breaker/record_success: Collecting 100 samples in estimated 5.0000 s (593M iteraticircuit_breaker/record_success
                        time:   [8.3861 ns 8.4035 ns 8.4228 ns]
                        thrpt:  [118.73 Melem/s 119.00 Melem/s 119.25 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking circuit_breaker/record_failure: Collecting 100 samples in estimated 5.0000 s (527M iteraticircuit_breaker/record_failure
                        time:   [9.4292 ns 9.4532 ns 9.4813 ns]
                        thrpt:  [105.47 Melem/s 105.78 Melem/s 106.05 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) high mild
  8 (8.00%) high severe
circuit_breaker/state   time:   [9.7938 ns 9.8217 ns 9.8536 ns]
                        thrpt:  [101.49 Melem/s 101.82 Melem/s 102.11 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samples in estimated 5.0008 s recovery_manager/on_failure_with_alternates
                        time:   [240.39 ns 241.75 ns 243.02 ns]
                        thrpt:  [4.1149 Melem/s 4.1364 Melem/s 4.1599 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  4 (4.00%) low severe
  7 (7.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samples in estimated 5.0000 s (2recovery_manager/on_failure_no_alternates
                        time:   [204.05 ns 212.30 ns 227.92 ns]
                        thrpt:  [4.3876 Melem/s 4.7102 Melem/s 4.9007 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  9 (9.00%) low severe
  6 (6.00%) low mild
  2 (2.00%) high severe
Benchmarking recovery_manager/get_action: Collecting 100 samples in estimated 5.0002 s (137M iterationsrecovery_manager/get_action
                        time:   [36.341 ns 36.507 ns 36.685 ns]
                        thrpt:  [27.259 Melem/s 27.392 Melem/s 27.517 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
recovery_manager/is_failed
                        time:   [12.261 ns 12.290 ns 12.322 ns]
                        thrpt:  [81.158 Melem/s 81.367 Melem/s 81.558 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking recovery_manager/on_recovery: Collecting 100 samples in estimated 5.0004 s (40M iterationsrecovery_manager/on_recovery
                        time:   [119.23 ns 119.55 ns 119.89 ns]
                        thrpt:  [8.3407 Melem/s 8.3647 Melem/s 8.3871 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
recovery_manager/stats  time:   [1.1502 ns 1.1536 ns 1.1574 ns]
                        thrpt:  [863.98 Melem/s 866.87 Melem/s 869.44 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  11 (11.00%) high mild
  1 (1.00%) high severe

Benchmarking failure_scaling/check_all/100: Collecting 100 samples in estimated 5.0029 s (611k iteratiofailure_scaling/check_all/100
                        time:   [8.5035 µs 8.5232 µs 8.5421 µs]
                        thrpt:  [11.707 Melem/s 11.733 Melem/s 11.760 Melem/s]
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estimated 5.0148 s (823k iterfailure_scaling/healthy_nodes/100
                        time:   [6.0698 µs 6.0894 µs 6.1111 µs]
                        thrpt:  [16.364 Melem/s 16.422 Melem/s 16.475 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking failure_scaling/check_all/500: Collecting 100 samples in estimated 5.0329 s (247k iteratiofailure_scaling/check_all/500
                        time:   [22.154 µs 22.212 µs 22.275 µs]
                        thrpt:  [22.446 Melem/s 22.510 Melem/s 22.570 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estimated 5.0210 s (520k iterfailure_scaling/healthy_nodes/500
                        time:   [9.6043 µs 9.6302 µs 9.6585 µs]
                        thrpt:  [51.768 Melem/s 51.920 Melem/s 52.060 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/check_all/1000: Collecting 100 samples in estimated 5.0224 s (141k iteratifailure_scaling/check_all/1000
                        time:   [39.222 µs 39.325 µs 39.428 µs]
                        thrpt:  [25.363 Melem/s 25.429 Melem/s 25.496 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) low mild
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in estimated 5.0451 s (354k itefailure_scaling/healthy_nodes/1000
                        time:   [14.201 µs 14.242 µs 14.290 µs]
                        thrpt:  [69.978 Melem/s 70.216 Melem/s 70.417 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) high mild
  11 (11.00%) high severe
Benchmarking failure_scaling/check_all/5000: Collecting 100 samples in estimated 5.6047 s (35k iteratiofailure_scaling/check_all/5000
                        time:   [175.68 µs 176.08 µs 176.53 µs]
                        thrpt:  [28.324 Melem/s 28.396 Melem/s 28.460 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in estimated 5.1028 s (101k itefailure_scaling/healthy_nodes/5000
                        time:   [51.114 µs 51.262 µs 51.426 µs]
                        thrpt:  [97.227 Melem/s 97.538 Melem/s 97.820 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild

Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samples in estimated 5.0409 s (25failure_concurrent/concurrent_heartbeat/4
                        time:   [200.18 µs 200.50 µs 200.82 µs]
                        thrpt:  [9.9591 Melem/s 9.9749 Melem/s 9.9912 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samples in estimated 5.0283 s (17failure_concurrent/concurrent_heartbeat/8
                        time:   [300.43 µs 301.04 µs 301.70 µs]
                        thrpt:  [13.258 Melem/s 13.287 Melem/s 13.314 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild
Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samples in estimated 5.0879 s (9failure_concurrent/concurrent_heartbeat/16
                        time:   [537.44 µs 539.22 µs 540.77 µs]
                        thrpt:  [14.794 Melem/s 14.836 Melem/s 14.885 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estimated 5.0004 s (21M iterafailure_recovery_cycle/full_cycle
                        time:   [239.25 ns 242.68 ns 246.39 ns]
                        thrpt:  [4.0586 Melem/s 4.1206 Melem/s 4.1797 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) low mild
  2 (2.00%) high mild

capability_set/create   time:   [743.94 ns 746.15 ns 748.60 ns]
                        thrpt:  [1.3358 Melem/s 1.3402 Melem/s 1.3442 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_set/serialize
                        time:   [722.88 ns 724.67 ns 726.59 ns]
                        thrpt:  [1.3763 Melem/s 1.3799 Melem/s 1.3834 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
capability_set/deserialize
                        time:   [2.9673 µs 2.9743 µs 2.9819 µs]
                        thrpt:  [335.36 Kelem/s 336.22 Kelem/s 337.01 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
capability_set/roundtrip
                        time:   [3.8466 µs 3.8588 µs 3.8735 µs]
                        thrpt:  [258.16 Kelem/s 259.15 Kelem/s 259.97 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
capability_set/has_tag  time:   [574.70 ps 576.00 ps 577.40 ps]
                        thrpt:  [1.7319 Gelem/s 1.7361 Gelem/s 1.7400 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
capability_set/has_model
                        time:   [413.02 ps 417.07 ps 421.55 ps]
                        thrpt:  [2.3722 Gelem/s 2.3977 Gelem/s 2.4212 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
capability_set/has_tool time:   [667.53 ps 690.86 ps 715.75 ps]
                        thrpt:  [1.3971 Gelem/s 1.4475 Gelem/s 1.4981 Gelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
capability_set/has_gpu  time:   [191.33 ps 191.72 ps 192.16 ps]
                        thrpt:  [5.2040 Gelem/s 5.2159 Gelem/s 5.2267 Gelem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_announcement/create: Collecting 100 samples in estimated 5.0066 s (3.4M iteraticapability_announcement/create
                        time:   [1.4702 µs 1.4728 µs 1.4757 µs]
                        thrpt:  [677.64 Kelem/s 678.98 Kelem/s 680.20 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_announcement/serialize: Collecting 100 samples in estimated 5.0037 s (5.1M itercapability_announcement/serialize
                        time:   [973.34 ns 976.44 ns 979.99 ns]
                        thrpt:  [1.0204 Melem/s 1.0241 Melem/s 1.0274 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  8 (8.00%) high mild
  8 (8.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in estimated 5.0091 s (1.9M itcapability_announcement/deserialize
                        time:   [2.5993 µs 2.6077 µs 2.6166 µs]
                        thrpt:  [382.18 Kelem/s 383.48 Kelem/s 384.71 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capability_announcement/is_expired: Collecting 100 samples in estimated 5.0000 s (239M itecapability_announcement/is_expired
                        time:   [20.771 ns 20.835 ns 20.910 ns]
                        thrpt:  [47.824 Melem/s 47.995 Melem/s 48.144 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  9 (9.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in estimated 5.0000 s (1.5B itecapability_filter/match_single_tag
                        time:   [3.2599 ns 3.2695 ns 3.2809 ns]
                        thrpt:  [304.80 Melem/s 305.86 Melem/s 306.76 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in estimated 5.0000 s (2.9B itcapability_filter/match_require_gpu
                        time:   [1.7273 ns 1.7332 ns 1.7396 ns]
                        thrpt:  [574.84 Melem/s 576.97 Melem/s 578.95 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in estimated 5.0000 s (2.6B itecapability_filter/match_gpu_vendor
                        time:   [1.9149 ns 1.9190 ns 1.9236 ns]
                        thrpt:  [519.87 Melem/s 521.09 Melem/s 522.22 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  10 (10.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in estimated 5.0000 s (2.9B itecapability_filter/match_min_memory
                        time:   [1.7238 ns 1.7279 ns 1.7324 ns]
                        thrpt:  [577.22 Melem/s 578.74 Melem/s 580.10 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimated 5.0000 s (912M iteratcapability_filter/match_complex
                        time:   [5.2497 ns 5.2833 ns 5.3186 ns]
                        thrpt:  [188.02 Melem/s 189.28 Melem/s 190.49 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_filter/match_no_match: Collecting 100 samples in estimated 5.0000 s (2.8B iteracapability_filter/match_no_match
                        time:   [1.7586 ns 1.7749 ns 1.7936 ns]
                        thrpt:  [557.52 Melem/s 563.40 Melem/s 568.62 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples in estimated 5.3106 s (30kcapability_index_insert/index_nodes/100
                        time:   [174.07 µs 174.53 µs 175.07 µs]
                        thrpt:  [571.19 Kelem/s 572.98 Kelem/s 574.49 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.3s, enable flat sampling, or reduce sample count to 50.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samples in estimated 7.3072 s (50capability_index_insert/index_nodes/1000
                        time:   [1.4161 ms 1.4194 ms 1.4230 ms]
                        thrpt:  [702.75 Kelem/s 704.53 Kelem/s 706.16 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samples in estimated 6.3789 s (4capability_index_insert/index_nodes/10000
                        time:   [15.663 ms 15.799 ms 15.950 ms]
                        thrpt:  [626.96 Kelem/s 632.94 Kelem/s 638.47 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples in estimated 5.2083 s (30kcapability_index_query/query_single_tag
                        time:   [172.23 µs 172.88 µs 173.59 µs]
                        thrpt:  [5.7607 Kelem/s 5.7844 Kelem/s 5.8062 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_query/query_require_gpu: Collecting 100 samples in estimated 5.2076 s (30capability_index_query/query_require_gpu
                        time:   [170.80 µs 171.25 µs 171.77 µs]
                        thrpt:  [5.8218 Kelem/s 5.8394 Kelem/s 5.8547 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples in estimated 7.4105 s (15kcapability_index_query/query_gpu_vendor
                        time:   [486.40 µs 487.58 µs 488.89 µs]
                        thrpt:  [2.0454 Kelem/s 2.0509 Kelem/s 2.0559 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples in estimated 5.2549 s (10kcapability_index_query/query_min_memory
                        time:   [519.75 µs 521.40 µs 523.40 µs]
                        thrpt:  [1.9106 Kelem/s 1.9179 Kelem/s 1.9240 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_complex: Collecting 100 samples in estimated 6.5910 s (20k itcapability_index_query/query_complex
                        time:   [324.17 µs 325.13 µs 326.26 µs]
                        thrpt:  [3.0650 Kelem/s 3.0757 Kelem/s 3.0848 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_model: Collecting 100 samples in estimated 5.1520 s (56k itercapability_index_query/query_model
                        time:   [91.579 µs 91.849 µs 92.152 µs]
                        thrpt:  [10.852 Kelem/s 10.887 Kelem/s 10.920 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estimated 7.1853 s (15k iteracapability_index_query/query_tool
                        time:   [472.34 µs 473.45 µs 474.64 µs]
                        thrpt:  [2.1068 Kelem/s 2.1122 Kelem/s 2.1171 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_query/query_no_results: Collecting 100 samples in estimated 5.0001 s (198capability_index_query/query_no_results
                        time:   [25.191 ns 25.254 ns 25.326 ns]
                        thrpt:  [39.486 Melem/s 39.598 Melem/s 39.697 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_index_find_best/find_best_simple: Collecting 100 samples in estimated 6.5489 s capability_index_find_best/find_best_simple
                        time:   [322.53 µs 323.84 µs 325.30 µs]
                        thrpt:  [3.0741 Kelem/s 3.0879 Kelem/s 3.1005 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 5.385capability_index_find_best/find_best_with_prefs
                        time:   [526.27 µs 527.60 µs 529.04 µs]
                        thrpt:  [1.8902 Kelem/s 1.8954 Kelem/s 1.9002 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples in estimated 5.0149 s (490capability_index_scaling/query_tag/1000
                        time:   [10.198 µs 10.221 µs 10.246 µs]
                        thrpt:  [97.597 Kelem/s 97.833 Kelem/s 98.055 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  9 (9.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0381 s capability_index_scaling/query_complex/1000
                        time:   [26.764 µs 26.819 µs 26.879 µs]
                        thrpt:  [37.204 Kelem/s 37.287 Kelem/s 37.364 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples in estimated 5.1812 s (96kcapability_index_scaling/query_tag/5000
                        time:   [53.683 µs 53.855 µs 54.046 µs]
                        thrpt:  [18.503 Kelem/s 18.568 Kelem/s 18.628 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 samples in estimated 5.3414 s capability_index_scaling/query_complex/5000
                        time:   [131.47 µs 131.83 µs 132.23 µs]
                        thrpt:  [7.5625 Kelem/s 7.5855 Kelem/s 7.6066 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  11 (11.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samples in estimated 5.3004 s (30capability_index_scaling/query_tag/10000
                        time:   [172.76 µs 173.48 µs 174.31 µs]
                        thrpt:  [5.7370 Kelem/s 5.7644 Kelem/s 5.7883 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samples in estimated 5.0148 scapability_index_scaling/query_complex/10000
                        time:   [327.91 µs 328.71 µs 329.61 µs]
                        thrpt:  [3.0339 Kelem/s 3.0422 Kelem/s 3.0496 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_scaling/query_tag/50000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.0s, enable flat sampling, or reduce sample count to 60.
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samples in estimated 6.0215 s (50capability_index_scaling/query_tag/50000
                        time:   [1.1778 ms 1.1816 ms 1.1855 ms]
                        thrpt:  [843.53  elem/s 846.29  elem/s 849.06  elem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samples in estimated 5.0368 scapability_index_scaling/query_complex/50000
                        time:   [1.9795 ms 1.9880 ms 1.9966 ms]
                        thrpt:  [500.84  elem/s 503.03  elem/s 505.18  elem/s]

Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 samples in estimated 5.0738 capability_index_concurrent/concurrent_index/4
                        time:   [857.44 µs 866.82 µs 875.64 µs]
                        thrpt:  [2.2840 Melem/s 2.3073 Melem/s 2.3325 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 samples in estimated 9.7779 capability_index_concurrent/concurrent_query/4
                        time:   [242.44 ms 243.63 ms 244.82 ms]
                        thrpt:  [8.1693 Kelem/s 8.2093 Kelem/s 8.2494 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 7.2279 capability_index_concurrent/concurrent_mixed/4
                        time:   [119.45 ms 120.18 ms 120.93 ms]
                        thrpt:  [16.539 Kelem/s 16.641 Kelem/s 16.743 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 samples in estimated 5.0565 capability_index_concurrent/concurrent_index/8
                        time:   [853.15 µs 856.69 µs 860.41 µs]
                        thrpt:  [4.6489 Melem/s 4.6691 Melem/s 4.6885 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.6s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 samples in estimated 5.5854 capability_index_concurrent/concurrent_query/8
                        time:   [279.77 ms 282.43 ms 285.62 ms]
                        thrpt:  [14.005 Kelem/s 14.163 Kelem/s 14.297 Kelem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) high mild
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 5.9227 capability_index_concurrent/concurrent_mixed/8
                        time:   [148.49 ms 149.77 ms 151.08 ms]
                        thrpt:  [26.477 Kelem/s 26.707 Kelem/s 26.938 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.0302capability_index_concurrent/concurrent_index/16
                        time:   [1.8322 ms 1.8613 ms 1.8818 ms]
                        thrpt:  [4.2512 Melem/s 4.2980 Melem/s 4.3663 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  3 (15.00%) low mild
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.4s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 samples in estimated 8.4440capability_index_concurrent/concurrent_query/16
                        time:   [420.53 ms 422.46 ms 424.39 ms]
                        thrpt:  [18.851 Kelem/s 18.937 Kelem/s 19.024 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 9.1146capability_index_concurrent/concurrent_mixed/16
                        time:   [227.81 ms 228.71 ms 229.72 ms]
                        thrpt:  [34.825 Kelem/s 34.979 Kelem/s 35.116 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.0013capability_index_updates/update_higher_version
                        time:   [782.65 ns 785.61 ns 788.82 ns]
                        thrpt:  [1.2677 Melem/s 1.2729 Melem/s 1.2777 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_updates/update_same_version: Collecting 100 samples in estimated 5.0028 scapability_index_updates/update_same_version
                        time:   [783.55 ns 786.76 ns 790.06 ns]
                        thrpt:  [1.2657 Melem/s 1.2710 Melem/s 1.2762 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samples in estimated 5.0061 s (2capability_index_updates/remove_and_readd
                        time:   [1.9684 µs 1.9721 µs 1.9760 µs]
                        thrpt:  [506.07 Kelem/s 507.08 Kelem/s 508.03 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

diff_op/create_add_tag  time:   [23.885 ns 23.976 ns 24.071 ns]
                        thrpt:  [41.544 Melem/s 41.708 Melem/s 41.867 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild
diff_op/create_remove_tag
                        time:   [23.467 ns 23.521 ns 23.578 ns]
                        thrpt:  [42.412 Melem/s 42.515 Melem/s 42.614 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
diff_op/create_add_model
                        time:   [84.020 ns 84.278 ns 84.557 ns]
                        thrpt:  [11.826 Melem/s 11.865 Melem/s 11.902 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild
Benchmarking diff_op/create_update_model: Collecting 100 samples in estimated 5.0001 s (211M iterationsdiff_op/create_update_model
                        time:   [23.574 ns 23.639 ns 23.709 ns]
                        thrpt:  [42.178 Melem/s 42.304 Melem/s 42.420 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
diff_op/estimated_size  time:   [958.51 ps 960.54 ps 962.79 ps]
                        thrpt:  [1.0386 Gelem/s 1.0411 Gelem/s 1.0433 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

capability_diff/create  time:   [82.950 ns 83.217 ns 83.505 ns]
                        thrpt:  [11.975 Melem/s 12.017 Melem/s 12.055 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
capability_diff/serialize
                        time:   [141.56 ns 142.39 ns 143.26 ns]
                        thrpt:  [6.9803 Melem/s 7.0229 Melem/s 7.0642 Melem/s]
capability_diff/deserialize
                        time:   [226.93 ns 227.93 ns 229.15 ns]
                        thrpt:  [4.3639 Melem/s 4.3873 Melem/s 4.4067 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking capability_diff/estimated_size: Collecting 100 samples in estimated 5.0000 s (2.2B iteraticapability_diff/estimated_size
                        time:   [2.3053 ns 2.3137 ns 2.3231 ns]
                        thrpt:  [430.47 Melem/s 432.21 Melem/s 433.79 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe

diff_generation/no_changes
                        time:   [265.38 ns 266.47 ns 267.68 ns]
                        thrpt:  [3.7357 Melem/s 3.7528 Melem/s 3.7682 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
diff_generation/add_one_tag
                        time:   [369.81 ns 370.82 ns 371.87 ns]
                        thrpt:  [2.6891 Melem/s 2.6967 Melem/s 2.7041 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high mild
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in estimated 5.0004 s (12M itdiff_generation/multiple_tag_changes
                        time:   [421.87 ns 422.98 ns 424.18 ns]
                        thrpt:  [2.3575 Melem/s 2.3642 Melem/s 2.3704 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in estimated 5.0008 s (15M itediff_generation/update_model_loaded
                        time:   [324.51 ns 325.62 ns 326.83 ns]
                        thrpt:  [3.0597 Melem/s 3.0711 Melem/s 3.0816 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  9 (9.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_generation/update_memory: Collecting 100 samples in estimated 5.0013 s (17M iterationdiff_generation/update_memory
                        time:   [297.33 ns 297.95 ns 298.59 ns]
                        thrpt:  [3.3491 Melem/s 3.3563 Melem/s 3.3633 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  9 (9.00%) high mild
diff_generation/add_model
                        time:   [411.24 ns 412.53 ns 413.95 ns]
                        thrpt:  [2.4158 Melem/s 2.4241 Melem/s 2.4317 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/complex_diff: Collecting 100 samples in estimated 5.0003 s (6.6M iterationdiff_generation/complex_diff
                        time:   [751.69 ns 753.63 ns 755.88 ns]
                        thrpt:  [1.3230 Melem/s 1.3269 Melem/s 1.3303 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estimated 5.0011 s (8.4M iteradiff_application/apply_single_op
                        time:   [593.45 ns 595.23 ns 597.10 ns]
                        thrpt:  [1.6748 Melem/s 1.6800 Melem/s 1.6850 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estimated 5.0056 s (3.4M iterdiff_application/apply_small_diff
                        time:   [1.4748 µs 1.4775 µs 1.4806 µs]
                        thrpt:  [675.40 Kelem/s 676.80 Kelem/s 678.04 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in estimated 5.0009 s (2.5M itediff_application/apply_medium_diff
                        time:   [1.9779 µs 1.9827 µs 1.9881 µs]
                        thrpt:  [502.99 Kelem/s 504.35 Kelem/s 505.60 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in estimated 5.0023 s (8.4M itediff_application/apply_strict_mode
                        time:   [589.71 ns 591.30 ns 593.02 ns]
                        thrpt:  [1.6863 Melem/s 1.6912 Melem/s 1.6957 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples in estimated 5.0000 s (1.3diff_chain_validation/validate_chain_10
                        time:   [3.8136 ns 3.8484 ns 3.8843 ns]
                        thrpt:  [257.45 Melem/s 259.85 Melem/s 262.22 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samples in estimated 5.0001 s (10diff_chain_validation/validate_chain_100
                        time:   [46.352 ns 46.620 ns 46.872 ns]
                        thrpt:  [21.335 Melem/s 21.450 Melem/s 21.574 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimated 5.0034 s (864k iteratdiff_compaction/compact_5_diffs
                        time:   [5.7451 µs 5.7613 µs 5.7794 µs]
                        thrpt:  [173.03 Kelem/s 173.57 Kelem/s 174.06 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estimated 5.0100 s (172k iteradiff_compaction/compact_20_diffs
                        time:   [28.990 µs 29.053 µs 29.119 µs]
                        thrpt:  [34.342 Kelem/s 34.420 Kelem/s 34.494 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estimated 5.0032 s (7.0M iterbandwidth_savings/calculate_small
                        time:   [714.88 ns 717.03 ns 719.37 ns]
                        thrpt:  [1.3901 Melem/s 1.3946 Melem/s 1.3988 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in estimated 5.0014 s (6.9M itebandwidth_savings/calculate_medium
                        time:   [722.56 ns 725.25 ns 728.25 ns]
                        thrpt:  [1.3732 Melem/s 1.3788 Melem/s 1.3840 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in estimated 5.0009 s (1.1M idiff_roundtrip/generate_apply_verify
                        time:   [4.4717 µs 4.4800 µs 4.4914 µs]
                        thrpt:  [222.65 Kelem/s 223.21 Kelem/s 223.63 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe

location_info/create    time:   [54.090 ns 55.120 ns 56.216 ns]
                        thrpt:  [17.788 Melem/s 18.142 Melem/s 18.488 Melem/s]
Found 19 outliers among 100 measurements (19.00%)
  15 (15.00%) high mild
  4 (4.00%) high severe
location_info/distance_to
                        time:   [2.5313 ns 2.5370 ns 2.5435 ns]
                        thrpt:  [393.17 Melem/s 394.16 Melem/s 395.05 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking location_info/same_continent: Collecting 100 samples in estimated 5.0000 s (2.0B iterationlocation_info/same_continent
                        time:   [2.5006 ns 2.5123 ns 2.5258 ns]
                        thrpt:  [395.92 Melem/s 398.04 Melem/s 399.91 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in estimated 5.0000 s (26B iterlocation_info/same_continent_cross
                        time:   [191.51 ps 191.97 ps 192.49 ps]
                        thrpt:  [5.1951 Gelem/s 5.2091 Gelem/s 5.2218 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
location_info/same_region
                        time:   [1.9184 ns 1.9249 ns 1.9323 ns]
                        thrpt:  [517.51 Melem/s 519.51 Melem/s 521.28 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  12 (12.00%) high mild
  1 (1.00%) high severe

topology_hints/create   time:   [3.4136 ns 3.4338 ns 3.4522 ns]
                        thrpt:  [289.67 Melem/s 291.22 Melem/s 292.95 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  2 (2.00%) low severe
  13 (13.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in estimated 5.0000 s (26B iteratopology_hints/connectivity_score
                        time:   [191.83 ps 192.31 ps 192.87 ps]
                        thrpt:  [5.1850 Gelem/s 5.1998 Gelem/s 5.2130 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in estimated 5.0000 s (20B ittopology_hints/average_latency_empty
                        time:   [252.22 ps 255.42 ps 258.59 ps]
                        thrpt:  [3.8672 Gelem/s 3.9151 Gelem/s 3.9649 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estimated 5.0001 s (108M itetopology_hints/average_latency_100
                        time:   [45.655 ns 45.764 ns 45.883 ns]
                        thrpt:  [21.795 Melem/s 21.851 Melem/s 21.903 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe

nat_type/difficulty     time:   [191.89 ps 192.44 ps 193.03 ps]
                        thrpt:  [5.1806 Gelem/s 5.1965 Gelem/s 5.2112 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
nat_type/can_connect_direct
                        time:   [191.86 ps 192.32 ps 192.91 ps]
                        thrpt:  [5.1838 Gelem/s 5.1996 Gelem/s 5.2122 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking nat_type/can_connect_symmetric: Collecting 100 samples in estimated 5.0000 s (26B iterationat_type/can_connect_symmetric
                        time:   [191.94 ps 192.49 ps 193.12 ps]
                        thrpt:  [5.1781 Gelem/s 5.1950 Gelem/s 5.2100 Gelem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe

Benchmarking node_metadata/create_simple: Collecting 100 samples in estimated 5.0000 s (115M iterationsnode_metadata/create_simple
                        time:   [29.985 ns 30.084 ns 30.192 ns]
                        thrpt:  [33.121 Melem/s 33.240 Melem/s 33.350 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  11 (11.00%) high mild
node_metadata/create_full
                        time:   [418.05 ns 419.00 ns 420.02 ns]
                        thrpt:  [2.3808 Melem/s 2.3867 Melem/s 2.3920 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
node_metadata/routing_score
                        time:   [191.63 ps 192.23 ps 192.92 ps]
                        thrpt:  [5.1834 Gelem/s 5.2020 Gelem/s 5.2184 Gelem/s]
Found 14 outliers among 100 measurements (14.00%)
  12 (12.00%) high mild
  2 (2.00%) high severe
node_metadata/age       time:   [23.874 ns 23.933 ns 23.997 ns]
                        thrpt:  [41.672 Melem/s 41.783 Melem/s 41.886 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
node_metadata/is_stale  time:   [23.277 ns 23.345 ns 23.422 ns]
                        thrpt:  [42.694 Melem/s 42.836 Melem/s 42.961 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
node_metadata/serialize time:   [538.60 ns 540.71 ns 543.57 ns]
                        thrpt:  [1.8397 Melem/s 1.8494 Melem/s 1.8567 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
node_metadata/deserialize
                        time:   [1.5828 µs 1.5876 µs 1.5932 µs]
                        thrpt:  [627.65 Kelem/s 629.90 Kelem/s 631.77 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high severe

Benchmarking metadata_query/match_status: Collecting 100 samples in estimated 5.0000 s (2.2B iterationsmetadata_query/match_status
                        time:   [2.3058 ns 2.3132 ns 2.3209 ns]
                        thrpt:  [430.87 Melem/s 432.30 Melem/s 433.69 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_min_tier: Collecting 100 samples in estimated 5.0000 s (2.2B iteratiometadata_query/match_min_tier
                        time:   [2.3004 ns 2.3056 ns 2.3112 ns]
                        thrpt:  [432.67 Melem/s 433.73 Melem/s 434.70 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_query/match_continent: Collecting 100 samples in estimated 5.0000 s (1.1B iteratimetadata_query/match_continent
                        time:   [4.4184 ns 4.4323 ns 4.4472 ns]
                        thrpt:  [224.86 Melem/s 225.62 Melem/s 226.32 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_complex: Collecting 100 samples in estimated 5.0000 s (1.1B iterationmetadata_query/match_complex
                        time:   [4.4099 ns 4.4216 ns 4.4350 ns]
                        thrpt:  [225.48 Melem/s 226.16 Melem/s 226.76 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_no_match: Collecting 100 samples in estimated 5.0000 s (3.2B iteratiometadata_query/match_no_match
                        time:   [1.4321 ns 1.4560 ns 1.4787 ns]
                        thrpt:  [676.27 Melem/s 686.79 Melem/s 698.27 Melem/s]

Benchmarking metadata_store_basic/create: Collecting 100 samples in estimated 5.0087 s (2.6M iterationsmetadata_store_basic/create
                        time:   [1.9231 µs 1.9287 µs 1.9349 µs]
                        thrpt:  [516.82 Kelem/s 518.49 Kelem/s 519.98 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimated 5.0017 s (3.4M iteratmetadata_store_basic/upsert_new
                        time:   [1.7006 µs 1.7125 µs 1.7256 µs]
                        thrpt:  [579.51 Kelem/s 583.94 Kelem/s 588.02 Kelem/s]
Found 18 outliers among 100 measurements (18.00%)
  8 (8.00%) low severe
  6 (6.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in estimated 5.0037 s (5.3M imetadata_store_basic/upsert_existing
                        time:   [943.04 ns 945.66 ns 948.61 ns]
                        thrpt:  [1.0542 Melem/s 1.0575 Melem/s 1.0604 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  11 (11.00%) high mild
  1 (1.00%) high severe
metadata_store_basic/get
                        time:   [22.481 ns 22.561 ns 22.656 ns]
                        thrpt:  [44.137 Melem/s 44.324 Melem/s 44.482 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/get_miss: Collecting 100 samples in estimated 5.0001 s (222M iteratiometadata_store_basic/get_miss
                        time:   [22.438 ns 22.512 ns 22.601 ns]
                        thrpt:  [44.246 Melem/s 44.421 Melem/s 44.567 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
metadata_store_basic/len
                        time:   [914.62 ns 916.79 ns 919.16 ns]
                        thrpt:  [1.0879 Melem/s 1.0908 Melem/s 1.0933 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 16.3s, or reduce sample count to 30.
metadata_store_basic/stats
                        time:   [162.21 ms 162.73 ms 163.33 ms]
                        thrpt:  [6.1225  elem/s 6.1452  elem/s 6.1650  elem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in estimated 5.5993 s (20k itmetadata_store_query/query_by_status
                        time:   [275.01 µs 275.91 µs 276.96 µs]
                        thrpt:  [3.6107 Kelem/s 3.6244 Kelem/s 3.6363 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples in estimated 5.4293 s (40kmetadata_store_query/query_by_continent
                        time:   [133.65 µs 134.17 µs 134.75 µs]
                        thrpt:  [7.4211 Kelem/s 7.4534 Kelem/s 7.4820 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in estimated 5.8016 s (15k itermetadata_store_query/query_by_tier
                        time:   [381.19 µs 382.20 µs 383.30 µs]
                        thrpt:  [2.6089 Kelem/s 2.6164 Kelem/s 2.6234 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samples in estimated 7.0883 s (1metadata_store_query/query_accepting_work
                        time:   [465.16 µs 466.07 µs 467.03 µs]
                        thrpt:  [2.1412 Kelem/s 2.1456 Kelem/s 2.1498 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples in estimated 6.9607 s (15k imetadata_store_query/query_with_limit
                        time:   [458.26 µs 459.68 µs 461.24 µs]
                        thrpt:  [2.1681 Kelem/s 2.1754 Kelem/s 2.1822 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in estimated 5.9498 s (20k itermetadata_store_query/query_complex
                        time:   [294.49 µs 295.41 µs 296.41 µs]
                        thrpt:  [3.3737 Kelem/s 3.3851 Kelem/s 3.3957 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe

Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samples in estimated 6.5071 s (20metadata_store_spatial/find_nearby_100km
                        time:   [322.99 µs 323.87 µs 324.80 µs]
                        thrpt:  [3.0788 Kelem/s 3.0876 Kelem/s 3.0961 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samples in estimated 5.3516 s (1metadata_store_spatial/find_nearby_1000km
                        time:   [351.29 µs 352.07 µs 352.90 µs]
                        thrpt:  [2.8337 Kelem/s 2.8403 Kelem/s 2.8466 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samples in estimated 6.7236 s (1metadata_store_spatial/find_nearby_5000km
                        time:   [444.61 µs 448.15 µs 451.92 µs]
                        thrpt:  [2.2128 Kelem/s 2.2314 Kelem/s 2.2491 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 samples in estimated 6.3744 smetadata_store_spatial/find_best_for_routing
                        time:   [314.81 µs 315.66 µs 316.95 µs]
                        thrpt:  [3.1551 Kelem/s 3.1679 Kelem/s 3.1766 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in estimated 7.5120 s (10k itermetadata_store_spatial/find_relays
                        time:   [775.97 µs 783.12 µs 790.58 µs]
                        thrpt:  [1.2649 Kelem/s 1.2769 Kelem/s 1.2887 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samples in estimated 5.0448 s (12metadata_store_scaling/query_status/1000
                        time:   [41.747 µs 41.840 µs 41.944 µs]
                        thrpt:  [23.841 Kelem/s 23.900 Kelem/s 23.954 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0161 s (2metadata_store_scaling/query_complex/1000
                        time:   [21.062 µs 21.143 µs 21.225 µs]
                        thrpt:  [47.114 Kelem/s 47.297 Kelem/s 47.478 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples in estimated 5.0561 s (101metadata_store_scaling/find_nearby/1000
                        time:   [50.327 µs 50.421 µs 50.509 µs]
                        thrpt:  [19.798 Kelem/s 19.833 Kelem/s 19.870 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samples in estimated 5.1620 s (45metadata_store_scaling/query_status/5000
                        time:   [112.78 µs 113.10 µs 113.43 µs]
                        thrpt:  [8.8162 Kelem/s 8.8419 Kelem/s 8.8669 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samples in estimated 5.3339 s (4metadata_store_scaling/query_complex/5000
                        time:   [113.97 µs 114.23 µs 114.50 µs]
                        thrpt:  [8.7336 Kelem/s 8.7540 Kelem/s 8.7739 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples in estimated 5.6159 s (25kmetadata_store_scaling/find_nearby/5000
                        time:   [225.34 µs 225.98 µs 226.63 µs]
                        thrpt:  [4.4125 Kelem/s 4.4252 Kelem/s 4.4377 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samples in estimated 5.9325 s (2metadata_store_scaling/query_status/10000
                        time:   [467.91 µs 471.23 µs 475.83 µs]
                        thrpt:  [2.1016 Kelem/s 2.1221 Kelem/s 2.1372 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) low severe
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samples in estimated 7.1400 s (metadata_store_scaling/query_complex/10000
                        time:   [467.46 µs 470.24 µs 473.56 µs]
                        thrpt:  [2.1117 Kelem/s 2.1266 Kelem/s 2.1392 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samples in estimated 7.5283 s (10metadata_store_scaling/find_nearby/10000
                        time:   [734.83 µs 738.31 µs 742.25 µs]
                        thrpt:  [1.3473 Kelem/s 1.3544 Kelem/s 1.3609 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samples in estimated 5.3442 s (1metadata_store_scaling/query_status/50000
                        time:   [4.6349 ms 4.7758 ms 4.9258 ms]
                        thrpt:  [203.01  elem/s 209.39  elem/s 215.76  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samples in estimated 5.4176 s (metadata_store_scaling/query_complex/50000
                        time:   [2.9734 ms 3.1772 ms 3.3802 ms]
                        thrpt:  [295.84  elem/s 314.75  elem/s 336.31  elem/s]
Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samples in estimated 5.0292 s (20metadata_store_scaling/find_nearby/50000
                        time:   [2.5470 ms 2.5583 ms 2.5724 ms]
                        thrpt:  [388.74  elem/s 390.88  elem/s 392.62  elem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 samples in estimated 5.1230 smetadata_store_concurrent/concurrent_upsert/4
                        time:   [1.2143 ms 1.2209 ms 1.2265 ms]
                        thrpt:  [1.6306 Melem/s 1.6382 Melem/s 1.6470 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 samples in estimated 9.0923 s metadata_store_concurrent/concurrent_query/4
                        time:   [225.83 ms 226.52 ms 227.22 ms]
                        thrpt:  [8.8020 Kelem/s 8.8293 Kelem/s 8.8563 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 8.5132 s metadata_store_concurrent/concurrent_mixed/4
                        time:   [205.19 ms 205.95 ms 206.75 ms]
                        thrpt:  [9.6734 Kelem/s 9.7112 Kelem/s 9.7471 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 samples in estimated 5.0413 smetadata_store_concurrent/concurrent_upsert/8
                        time:   [1.6967 ms 1.7058 ms 1.7154 ms]
                        thrpt:  [2.3318 Melem/s 2.3450 Melem/s 2.3575 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.2s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 samples in estimated 5.2007 s metadata_store_concurrent/concurrent_query/8
                        time:   [256.85 ms 258.80 ms 260.69 ms]
                        thrpt:  [15.344 Kelem/s 15.456 Kelem/s 15.573 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.0s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 5.0266 s metadata_store_concurrent/concurrent_mixed/8
                        time:   [247.89 ms 249.67 ms 251.54 ms]
                        thrpt:  [15.902 Kelem/s 16.021 Kelem/s 16.136 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 samples in estimated 5.7064 metadata_store_concurrent/concurrent_upsert/16
                        time:   [3.3779 ms 3.3851 ms 3.3936 ms]
                        thrpt:  [2.3574 Melem/s 2.3633 Melem/s 2.3683 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.3s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 samples in estimated 8.2629 smetadata_store_concurrent/concurrent_query/16
                        time:   [410.93 ms 412.88 ms 414.85 ms]
                        thrpt:  [19.284 Kelem/s 19.376 Kelem/s 19.468 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.9s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 8.9238 smetadata_store_concurrent/concurrent_mixed/16
                        time:   [447.93 ms 450.14 ms 452.44 ms]
                        thrpt:  [17.682 Kelem/s 17.772 Kelem/s 17.860 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimated 5.metadata_store_versioning/update_versioned_success
                        time:   [218.35 ns 219.07 ns 219.89 ns]
                        thrpt:  [4.5476 Melem/s 4.5647 Melem/s 4.5798 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimated 5metadata_store_versioning/update_versioned_conflict
                        time:   [217.80 ns 218.49 ns 219.12 ns]
                        thrpt:  [4.5636 Melem/s 4.5769 Melem/s 4.5914 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  10 (10.00%) low severe
  5 (5.00%) low mild
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estimated 5.0000 s (2.1B iterschema_validation/validate_string
                        time:   [2.7139 ns 2.9767 ns 3.2962 ns]
                        thrpt:  [303.38 Melem/s 335.95 Melem/s 368.48 Melem/s]
Found 36 outliers among 100 measurements (36.00%)
  12 (12.00%) low severe
  6 (6.00%) low mild
  18 (18.00%) high severe
Benchmarking schema_validation/validate_integer: Collecting 100 samples in estimated 5.0000 s (1.9B iteschema_validation/validate_integer
                        time:   [2.9262 ns 3.1776 ns 3.4829 ns]
                        thrpt:  [287.12 Melem/s 314.70 Melem/s 341.75 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  2 (2.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  19 (19.00%) high severe
Benchmarking schema_validation/validate_object: Collecting 100 samples in estimated 5.0002 s (88M iteraschema_validation/validate_object
                        time:   [61.013 ns 64.359 ns 68.000 ns]
                        thrpt:  [14.706 Melem/s 15.538 Melem/s 16.390 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in estimated 5.0000 s (152M itschema_validation/validate_array_10
                        time:   [33.056 ns 33.290 ns 33.519 ns]
                        thrpt:  [29.833 Melem/s 30.039 Melem/s 30.252 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking schema_validation/validate_complex: Collecting 100 samples in estimated 5.0007 s (35M iterschema_validation/validate_complex
                        time:   [147.18 ns 149.52 ns 152.05 ns]
                        thrpt:  [6.5769 Melem/s 6.6882 Melem/s 6.7943 Melem/s]
Found 31 outliers among 100 measurements (31.00%)
  11 (11.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  16 (16.00%) high severe

Benchmarking endpoint_matching/match_success: Collecting 100 samples in estimated 5.0005 s (24M iteratiendpoint_matching/match_success
                        time:   [209.15 ns 209.54 ns 209.92 ns]
                        thrpt:  [4.7638 Melem/s 4.7724 Melem/s 4.7812 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low severe
  4 (4.00%) low mild
  4 (4.00%) high mild
Benchmarking endpoint_matching/match_failure: Collecting 100 samples in estimated 5.0003 s (24M iteratiendpoint_matching/match_failure
                        time:   [206.35 ns 206.68 ns 207.04 ns]
                        thrpt:  [4.8300 Melem/s 4.8384 Melem/s 4.8461 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in estimated 5.0022 s (11M iteendpoint_matching/match_multi_param
                        time:   [466.07 ns 467.06 ns 468.12 ns]
                        thrpt:  [2.1362 Melem/s 2.1411 Melem/s 2.1456 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking api_version/is_compatible_with: Collecting 100 samples in estimated 5.0000 s (25B iteratioapi_version/is_compatible_with
                        time:   [200.96 ps 203.01 ps 205.79 ps]
                        thrpt:  [4.8593 Gelem/s 4.9258 Gelem/s 4.9762 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  10 (10.00%) low severe
  2 (2.00%) high mild
  7 (7.00%) high severe
api_version/parse       time:   [39.849 ns 39.921 ns 39.992 ns]
                        thrpt:  [25.005 Melem/s 25.049 Melem/s 25.095 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  3 (3.00%) low mild
  4 (4.00%) high mild
  2 (2.00%) high severe
api_version/to_string   time:   [53.353 ns 53.543 ns 53.758 ns]
                        thrpt:  [18.602 Melem/s 18.677 Melem/s 18.743 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe

api_schema/create       time:   [4.0566 µs 4.0644 µs 4.0721 µs]
                        thrpt:  [245.57 Kelem/s 246.04 Kelem/s 246.51 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  8 (8.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
api_schema/serialize    time:   [1.5663 µs 1.5732 µs 1.5796 µs]
                        thrpt:  [633.06 Kelem/s 635.64 Kelem/s 638.45 Kelem/s]
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low severe
  10 (10.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
api_schema/deserialize  time:   [8.2545 µs 8.2820 µs 8.3072 µs]
                        thrpt:  [120.38 Kelem/s 120.74 Kelem/s 121.15 Kelem/s]
Found 25 outliers among 100 measurements (25.00%)
  11 (11.00%) low severe
  4 (4.00%) low mild
  5 (5.00%) high mild
  5 (5.00%) high severe
api_schema/find_endpoint
                        time:   [233.64 ns 234.50 ns 235.28 ns]
                        thrpt:  [4.2503 Melem/s 4.2645 Melem/s 4.2801 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking api_schema/endpoints_by_tag: Collecting 100 samples in estimated 5.0015 s (5.9M iterationsapi_schema/endpoints_by_tag
                        time:   [844.08 ns 845.57 ns 847.31 ns]
                        thrpt:  [1.1802 Melem/s 1.1826 Melem/s 1.1847 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  6 (6.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  8 (8.00%) high severe

Benchmarking request_validation/validate_full_request: Collecting 100 samples in estimated 5.0002 s (91request_validation/validate_full_request
                        time:   [54.896 ns 55.095 ns 55.295 ns]
                        thrpt:  [18.085 Melem/s 18.150 Melem/s 18.216 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking request_validation/validate_path_only: Collecting 100 samples in estimated 5.0000 s (315M request_validation/validate_path_only
                        time:   [14.481 ns 14.729 ns 14.967 ns]
                        thrpt:  [66.815 Melem/s 67.892 Melem/s 69.057 Melem/s]

api_registry_basic/create
                        time:   [1.4991 µs 1.5030 µs 1.5065 µs]
                        thrpt:  [663.78 Kelem/s 665.32 Kelem/s 667.06 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimated 5.0216 s (1.1M iteratapi_registry_basic/register_new
                        time:   [4.1422 µs 4.1510 µs 4.1596 µs]
                        thrpt:  [240.41 Kelem/s 240.90 Kelem/s 241.42 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  9 (9.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high severe
api_registry_basic/get  time:   [23.510 ns 23.548 ns 23.583 ns]
                        thrpt:  [42.404 Melem/s 42.466 Melem/s 42.535 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high severe
api_registry_basic/len  time:   [954.14 ns 956.44 ns 958.33 ns]
                        thrpt:  [1.0435 Melem/s 1.0455 Melem/s 1.0481 Melem/s]
Found 18 outliers among 100 measurements (18.00%)
  8 (8.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 13.5s, or reduce sample count to 30.
api_registry_basic/stats
                        time:   [135.42 ms 135.89 ms 136.44 ms]
                        thrpt:  [7.3291  elem/s 7.3588  elem/s 7.3845  elem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estimated 5.1833 s (61k iteratapi_registry_query/query_by_name
                        time:   [84.799 µs 85.134 µs 85.421 µs]
                        thrpt:  [11.707 Kelem/s 11.746 Kelem/s 11.793 Kelem/s]
Found 18 outliers among 100 measurements (18.00%)
  8 (8.00%) low severe
  1 (1.00%) low mild
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_query/query_by_tag: Collecting 100 samples in estimated 7.4972 s (10k iteratiapi_registry_query/query_by_tag
                        time:   [745.40 µs 747.46 µs 749.26 µs]
                        thrpt:  [1.3347 Kelem/s 1.3379 Kelem/s 1.3416 Kelem/s]
Found 22 outliers among 100 measurements (22.00%)
  12 (12.00%) low severe
  5 (5.00%) low mild
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_query/query_with_version: Collecting 100 samples in estimated 5.0565 s (126k api_registry_query/query_with_version
                        time:   [40.537 µs 40.693 µs 40.845 µs]
                        thrpt:  [24.483 Kelem/s 24.574 Kelem/s 24.669 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
  10 (10.00%) high severe
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in estimated 5.2474 s (1500 itapi_registry_query/find_by_endpoint
                        time:   [3.4974 ms 3.5251 ms 3.5588 ms]
                        thrpt:  [280.99  elem/s 283.68  elem/s 285.93  elem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in estimated 5.1870 s (86k iterapi_registry_query/find_compatible
                        time:   [59.808 µs 60.110 µs 60.393 µs]
                        thrpt:  [16.558 Kelem/s 16.636 Kelem/s 16.720 Kelem/s]
Found 24 outliers among 100 measurements (24.00%)
  9 (9.00%) low severe
  4 (4.00%) low mild
  6 (6.00%) high mild
  5 (5.00%) high severe

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples in estimated 5.0313 s (672api_registry_scaling/query_by_name/1000
                        time:   [7.4523 µs 7.4889 µs 7.5239 µs]
                        thrpt:  [132.91 Kelem/s 133.53 Kelem/s 134.19 Kelem/s]
Found 22 outliers among 100 measurements (22.00%)
  17 (17.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples in estimated 5.0522 s (131kapi_registry_scaling/query_by_tag/1000
                        time:   [38.392 µs 38.530 µs 38.652 µs]
                        thrpt:  [25.872 Kelem/s 25.954 Kelem/s 26.047 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples in estimated 5.1253 s (136api_registry_scaling/query_by_name/5000
                        time:   [37.556 µs 37.669 µs 37.757 µs]
                        thrpt:  [26.485 Kelem/s 26.547 Kelem/s 26.627 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples in estimated 5.5017 s (20k api_registry_scaling/query_by_tag/5000
                        time:   [272.67 µs 273.27 µs 273.82 µs]
                        thrpt:  [3.6520 Kelem/s 3.6593 Kelem/s 3.6674 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samples in estimated 5.3844 s (66api_registry_scaling/query_by_name/10000
                        time:   [81.427 µs 81.739 µs 82.066 µs]
                        thrpt:  [12.185 Kelem/s 12.234 Kelem/s 12.281 Kelem/s]
Found 24 outliers among 100 measurements (24.00%)
  10 (10.00%) low severe
  5 (5.00%) low mild
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples in estimated 7.4138 s (10kapi_registry_scaling/query_by_tag/10000
                        time:   [732.33 µs 734.99 µs 737.35 µs]
                        thrpt:  [1.3562 Kelem/s 1.3606 Kelem/s 1.3655 Kelem/s]
Found 14 outliers among 100 measurements (14.00%)
  7 (7.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.0s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 samples in estimated 10.048 s (2api_registry_concurrent/concurrent_query/4
                        time:   [495.45 ms 499.86 ms 505.62 ms]
                        thrpt:  [3.9556 Kelem/s 4.0011 Kelem/s 4.0367 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 8.4047 s (2api_registry_concurrent/concurrent_mixed/4
                        time:   [417.09 ms 418.70 ms 420.33 ms]
                        thrpt:  [4.7582 Kelem/s 4.7767 Kelem/s 4.7951 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 11.1s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 samples in estimated 11.063 s (2api_registry_concurrent/concurrent_query/8
                        time:   [548.93 ms 553.34 ms 559.46 ms]
                        thrpt:  [7.1497 Kelem/s 7.2288 Kelem/s 7.2870 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high severe
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 9.4183 s (2api_registry_concurrent/concurrent_mixed/8
                        time:   [469.20 ms 470.44 ms 471.79 ms]
                        thrpt:  [8.4784 Kelem/s 8.5027 Kelem/s 8.5251 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 17.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samples in estimated 17.382 s (api_registry_concurrent/concurrent_query/16
                        time:   [869.46 ms 871.25 ms 873.09 ms]
                        thrpt:  [9.1628 Kelem/s 9.1822 Kelem/s 9.2011 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 14.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 14.366 s (api_registry_concurrent/concurrent_mixed/16
                        time:   [704.14 ms 708.01 ms 711.89 ms]
                        thrpt:  [11.238 Kelem/s 11.299 Kelem/s 11.361 Kelem/s]
Found 4 outliers among 20 measurements (20.00%)
  3 (15.00%) low mild
  1 (5.00%) high mild

compare_op/eq           time:   [1.3802 ns 1.3877 ns 1.3947 ns]
                        thrpt:  [717.00 Melem/s 720.63 Melem/s 724.55 Melem/s]
Found 29 outliers among 100 measurements (29.00%)
  21 (21.00%) low severe
  5 (5.00%) high mild
  3 (3.00%) high severe
compare_op/gt           time:   [909.11 ps 910.98 ps 913.11 ps]
                        thrpt:  [1.0952 Gelem/s 1.0977 Gelem/s 1.1000 Gelem/s]
Found 26 outliers among 100 measurements (26.00%)
  10 (10.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  11 (11.00%) high severe
compare_op/contains_string
                        time:   [19.245 ns 19.323 ns 19.394 ns]
                        thrpt:  [51.563 Melem/s 51.751 Melem/s 51.962 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high severe
compare_op/in_array     time:   [3.0058 ns 3.0229 ns 3.0403 ns]
                        thrpt:  [328.91 Melem/s 330.81 Melem/s 332.69 Melem/s]
Found 22 outliers among 100 measurements (22.00%)
  12 (12.00%) low severe
  3 (3.00%) low mild
  2 (2.00%) high mild
  5 (5.00%) high severe

condition/simple        time:   [46.142 ns 46.420 ns 46.705 ns]
                        thrpt:  [21.411 Melem/s 21.542 Melem/s 21.672 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
condition/nested_field  time:   [587.68 ns 590.49 ns 593.17 ns]
                        thrpt:  [1.6859 Melem/s 1.6935 Melem/s 1.7016 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  13 (13.00%) low mild
  2 (2.00%) high mild
condition/string_eq     time:   [73.194 ns 73.389 ns 73.601 ns]
                        thrpt:  [13.587 Melem/s 13.626 Melem/s 13.662 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe

condition_expr/single   time:   [49.623 ns 49.708 ns 49.800 ns]
                        thrpt:  [20.080 Melem/s 20.118 Melem/s 20.152 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
condition_expr/and_2    time:   [102.80 ns 103.07 ns 103.34 ns]
                        thrpt:  [9.6772 Melem/s 9.7022 Melem/s 9.7281 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  2 (2.00%) high severe
condition_expr/and_5    time:   [304.13 ns 305.85 ns 307.58 ns]
                        thrpt:  [3.2512 Melem/s 3.2696 Melem/s 3.2881 Melem/s]
condition_expr/or_3     time:   [175.49 ns 176.08 ns 176.55 ns]
                        thrpt:  [5.6642 Melem/s 5.6792 Melem/s 5.6983 Melem/s]
Found 17 outliers among 100 measurements (17.00%)
  1 (1.00%) low severe
  14 (14.00%) low mild
  2 (2.00%) high mild
condition_expr/nested   time:   [129.03 ns 129.37 ns 129.67 ns]
                        thrpt:  [7.7119 Melem/s 7.7298 Melem/s 7.7499 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  8 (8.00%) low severe
  4 (4.00%) low mild
  1 (1.00%) high severe

rule/create             time:   [391.25 ns 391.79 ns 392.33 ns]
                        thrpt:  [2.5489 Melem/s 2.5524 Melem/s 2.5559 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
rule/matches            time:   [101.42 ns 101.81 ns 102.17 ns]
                        thrpt:  [9.7872 Melem/s 9.8224 Melem/s 9.8597 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild

rule_context/create     time:   [1.7675 µs 1.7711 µs 1.7744 µs]
                        thrpt:  [563.58 Kelem/s 564.63 Kelem/s 565.78 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  4 (4.00%) high severe
rule_context/get_simple time:   [43.187 ns 43.386 ns 43.572 ns]
                        thrpt:  [22.951 Melem/s 23.049 Melem/s 23.155 Melem/s]
rule_context/get_nested time:   [583.69 ns 585.02 ns 586.25 ns]
                        thrpt:  [1.7058 Melem/s 1.7093 Melem/s 1.7132 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  2 (2.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking rule_context/get_deep_nested: Collecting 100 samples in estimated 5.0010 s (8.5M iterationrule_context/get_deep_nested
                        time:   [588.28 ns 589.22 ns 590.17 ns]
                        thrpt:  [1.6944 Melem/s 1.6972 Melem/s 1.6999 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

rule_engine_basic/create
                        time:   [11.736 ns 11.805 ns 11.877 ns]
                        thrpt:  [84.198 Melem/s 84.711 Melem/s 85.205 Melem/s]
rule_engine_basic/add_rule
                        time:   [1.6236 µs 1.6764 µs 1.7200 µs]
                        thrpt:  [581.40 Kelem/s 596.52 Kelem/s 615.91 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_basic/get_rule
                        time:   [16.033 ns 16.093 ns 16.145 ns]
                        thrpt:  [61.938 Melem/s 62.137 Melem/s 62.372 Melem/s]
Benchmarking rule_engine_basic/rules_by_tag: Collecting 100 samples in estimated 5.0026 s (5.1M iteratirule_engine_basic/rules_by_tag
                        time:   [966.70 ns 971.15 ns 975.04 ns]
                        thrpt:  [1.0256 Melem/s 1.0297 Melem/s 1.0344 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  18 (18.00%) low severe
  5 (5.00%) high mild
  1 (1.00%) high severe
rule_engine_basic/stats time:   [7.7948 µs 7.8044 µs 7.8131 µs]
                        thrpt:  [127.99 Kelem/s 128.13 Kelem/s 128.29 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples in estimated 5.0017 s (1.8Mrule_engine_evaluate/evaluate_10_rules
                        time:   [2.8259 µs 2.8396 µs 2.8530 µs]
                        thrpt:  [350.50 Kelem/s 352.17 Kelem/s 353.87 Kelem/s]
Found 16 outliers among 100 measurements (16.00%)
  7 (7.00%) low severe
  5 (5.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samples in estimated 5.0004 srule_engine_evaluate/evaluate_first_10_rules
                        time:   [324.37 ns 325.84 ns 327.34 ns]
                        thrpt:  [3.0549 Melem/s 3.0689 Melem/s 3.0829 Melem/s]
Found 34 outliers among 100 measurements (34.00%)
  13 (13.00%) low severe
  6 (6.00%) low mild
  5 (5.00%) high mild
  10 (10.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples in estimated 5.0212 s (152rule_engine_evaluate/evaluate_100_rules
                        time:   [32.828 µs 32.987 µs 33.144 µs]
                        thrpt:  [30.171 Kelem/s 30.315 Kelem/s 30.462 Kelem/s]
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 samples in estimated 5.0004 rule_engine_evaluate/evaluate_first_100_rules
                        time:   [320.87 ns 321.97 ns 323.13 ns]
                        thrpt:  [3.0947 Melem/s 3.1059 Melem/s 3.1165 Melem/s]
Found 20 outliers among 100 measurements (20.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  13 (13.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated 5.09rule_engine_evaluate/evaluate_matching_100_rules
                        time:   [38.831 µs 38.993 µs 39.142 µs]
                        thrpt:  [25.548 Kelem/s 25.646 Kelem/s 25.752 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) low mild
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samples in estimated 6.0633 s (20rule_engine_evaluate/evaluate_1000_rules
                        time:   [299.27 µs 300.46 µs 301.58 µs]
                        thrpt:  [3.3159 Kelem/s 3.3282 Kelem/s 3.3415 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) low severe
  2 (2.00%) low mild
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 samples in estimated 5.0003rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [321.28 ns 322.49 ns 323.79 ns]
                        thrpt:  [3.0884 Melem/s 3.1009 Melem/s 3.1125 Melem/s]
Found 23 outliers among 100 measurements (23.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild
  2 (2.00%) high mild
  16 (16.00%) high severe

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimated 5.0128 s (1.7M iteratrule_engine_scaling/evaluate/10
                        time:   [2.8967 µs 2.9024 µs 2.9090 µs]
                        thrpt:  [343.76 Kelem/s 344.54 Kelem/s 345.22 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/10: Collecting 100 samples in estimated 5.0014 s (16M irule_engine_scaling/evaluate_first/10
                        time:   [319.16 ns 320.07 ns 320.85 ns]
                        thrpt:  [3.1167 Melem/s 3.1244 Melem/s 3.1332 Melem/s]
Found 24 outliers among 100 measurements (24.00%)
  17 (17.00%) low severe
  4 (4.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate/50: Collecting 100 samples in estimated 5.0578 s (258k iteratrule_engine_scaling/evaluate/50
                        time:   [19.649 µs 19.686 µs 19.725 µs]
                        thrpt:  [50.696 Kelem/s 50.797 Kelem/s 50.892 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/50: Collecting 100 samples in estimated 5.0007 s (16M irule_engine_scaling/evaluate_first/50
                        time:   [322.44 ns 323.36 ns 324.19 ns]
                        thrpt:  [3.0846 Melem/s 3.0926 Melem/s 3.1014 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate/100: Collecting 100 samples in estimated 5.1436 s (152k iterarule_engine_scaling/evaluate/100
                        time:   [33.505 µs 33.641 µs 33.780 µs]
                        thrpt:  [29.603 Kelem/s 29.725 Kelem/s 29.846 Kelem/s]
Found 20 outliers among 100 measurements (20.00%)
  11 (11.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/100: Collecting 100 samples in estimated 5.0001 s (16M rule_engine_scaling/evaluate_first/100
                        time:   [309.31 ns 310.89 ns 312.64 ns]
                        thrpt:  [3.1986 Melem/s 3.2166 Melem/s 3.2330 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate/500: Collecting 100 samples in estimated 5.2423 s (35k iteratrule_engine_scaling/evaluate/500
                        time:   [146.55 µs 147.08 µs 147.70 µs]
                        thrpt:  [6.7706 Kelem/s 6.7991 Kelem/s 6.8238 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/500: Collecting 100 samples in estimated 5.0010 s (16M rule_engine_scaling/evaluate_first/500
                        time:   [306.43 ns 307.28 ns 308.27 ns]
                        thrpt:  [3.2439 Melem/s 3.2544 Melem/s 3.2634 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
Benchmarking rule_engine_scaling/evaluate/1000: Collecting 100 samples in estimated 5.8983 s (20k iterarule_engine_scaling/evaluate/1000
                        time:   [290.06 µs 291.23 µs 292.60 µs]
                        thrpt:  [3.4176 Kelem/s 3.4337 Kelem/s 3.4476 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  10 (10.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_scaling/evaluate_first/1000: Collecting 100 samples in estimated 5.0011 s (16Mrule_engine_scaling/evaluate_first/1000
                        time:   [305.76 ns 306.71 ns 307.74 ns]
                        thrpt:  [3.2495 Melem/s 3.2604 Melem/s 3.2705 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe

rule_set/create         time:   [4.9893 µs 5.0022 µs 5.0158 µs]
                        thrpt:  [199.37 Kelem/s 199.91 Kelem/s 200.43 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
rule_set/load_into_engine
                        time:   [12.187 µs 12.215 µs 12.247 µs]
                        thrpt:  [81.655 Kelem/s 81.870 Kelem/s 82.056 Kelem/s]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) high mild

trace_id/generate       time:   [45.229 ns 45.373 ns 45.532 ns]
                        thrpt:  [21.963 Melem/s 22.040 Melem/s 22.110 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
trace_id/to_hex         time:   [85.783 ns 86.097 ns 86.456 ns]
                        thrpt:  [11.567 Melem/s 11.615 Melem/s 11.657 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  10 (10.00%) high mild
trace_id/from_hex       time:   [17.310 ns 17.347 ns 17.388 ns]
                        thrpt:  [57.511 Melem/s 57.648 Melem/s 57.771 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe

context_operations/create
                        time:   [77.197 ns 77.462 ns 77.747 ns]
                        thrpt:  [12.862 Melem/s 12.910 Melem/s 12.954 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
context_operations/child
                        time:   [34.859 ns 35.011 ns 35.181 ns]
                        thrpt:  [28.425 Melem/s 28.562 Melem/s 28.687 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking context_operations/for_remote: Collecting 100 samples in estimated 5.0001 s (144M iteratiocontext_operations/for_remote
                        time:   [34.925 ns 35.077 ns 35.244 ns]
                        thrpt:  [28.373 Melem/s 28.509 Melem/s 28.633 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking context_operations/to_traceparent: Collecting 100 samples in estimated 5.0005 s (19M iteracontext_operations/to_traceparent
                        time:   [251.74 ns 252.41 ns 253.16 ns]
                        thrpt:  [3.9500 Melem/s 3.9618 Melem/s 3.9724 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking context_operations/from_traceparent: Collecting 100 samples in estimated 5.0003 s (47M itecontext_operations/from_traceparent
                        time:   [106.54 ns 106.86 ns 107.21 ns]
                        thrpt:  [9.3275 Melem/s 9.3580 Melem/s 9.3858 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

baggage/create          time:   [4.5805 ns 4.5984 ns 4.6167 ns]
                        thrpt:  [216.60 Melem/s 217.47 Melem/s 218.32 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  6 (6.00%) high mild
  2 (2.00%) high severe
baggage/get             time:   [8.1604 ns 8.1792 ns 8.2010 ns]
                        thrpt:  [121.94 Melem/s 122.26 Melem/s 122.54 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
baggage/set             time:   [58.444 ns 58.660 ns 58.903 ns]
                        thrpt:  [16.977 Melem/s 17.047 Melem/s 17.111 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild
baggage/merge           time:   [1.6760 µs 1.6792 µs 1.6831 µs]
                        thrpt:  [594.13 Kelem/s 595.51 Kelem/s 596.67 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  11 (11.00%) high mild
  2 (2.00%) high severe

span/create             time:   [79.971 ns 80.274 ns 80.592 ns]
                        thrpt:  [12.408 Melem/s 12.457 Melem/s 12.505 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
span/set_attribute      time:   [55.228 ns 55.553 ns 55.920 ns]
                        thrpt:  [17.883 Melem/s 18.001 Melem/s 18.107 Melem/s]
span/add_event          time:   [61.760 ns 62.981 ns 64.371 ns]
                        thrpt:  [15.535 Melem/s 15.878 Melem/s 16.192 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
span/with_kind          time:   [79.510 ns 79.680 ns 79.862 ns]
                        thrpt:  [12.522 Melem/s 12.550 Melem/s 12.577 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) high mild

Benchmarking context_store/create_context: Collecting 100 samples in estimated 5.1550 s (66k iterationscontext_store/create_context
                        time:   [91.090 µs 91.376 µs 91.665 µs]
                        thrpt:  [10.909 Kelem/s 10.944 Kelem/s 10.978 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
