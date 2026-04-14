     Running benches/bltp.rs (target/release/deps/bltp-0fbb49fa179aeb47)
Gnuplot not found, using plotters backend
bltp_header/serialize   time:   [2.0441 ns 2.0539 ns 2.0639 ns]
                        thrpt:  [484.51 Melem/s 486.87 Melem/s 489.22 Melem/s]
                 change:
                        time:   [−1.2306% −0.5974% +0.0851%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0850% +0.6010% +1.2459%]
                        No change in performance detected.
bltp_header/deserialize time:   [2.1705 ns 2.1828 ns 2.1951 ns]
                        thrpt:  [455.56 Melem/s 458.12 Melem/s 460.72 Melem/s]
                 change:
                        time:   [−0.7953% −0.1980% +0.3300%] (p = 0.49 > 0.05)
                        thrpt:  [−0.3289% +0.1984% +0.8017%]
                        No change in performance detected.
bltp_header/roundtrip   time:   [2.1696 ns 2.1811 ns 2.1923 ns]
                        thrpt:  [456.14 Melem/s 458.49 Melem/s 460.91 Melem/s]
                 change:
                        time:   [−1.2486% −0.6808% −0.0906%] (p = 0.02 < 0.05)
                        thrpt:  [+0.0906% +0.6854% +1.2644%]
                        Change within noise threshold.

bltp_event_frame/write_single/64
                        time:   [19.247 ns 19.312 ns 19.378 ns]
                        thrpt:  [3.0758 GiB/s 3.0864 GiB/s 3.0968 GiB/s]
                 change:
                        time:   [−0.9384% −0.4102% +0.0999%] (p = 0.13 > 0.05)
                        thrpt:  [−0.0998% +0.4119% +0.9473%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
bltp_event_frame/write_single/256
                        time:   [50.079 ns 50.368 ns 50.667 ns]
                        thrpt:  [4.7056 GiB/s 4.7335 GiB/s 4.7608 GiB/s]
                 change:
                        time:   [−0.8299% +0.0298% +0.8927%] (p = 0.94 > 0.05)
                        thrpt:  [−0.8848% −0.0298% +0.8369%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) low mild
  6 (6.00%) high mild
bltp_event_frame/write_single/1024
                        time:   [39.179 ns 39.321 ns 39.463 ns]
                        thrpt:  [24.166 GiB/s 24.254 GiB/s 24.342 GiB/s]
                 change:
                        time:   [−1.5126% −0.8928% −0.2987%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2996% +0.9008% +1.5358%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
bltp_event_frame/write_single/4096
                        time:   [87.296 ns 88.656 ns 90.054 ns]
                        thrpt:  [42.360 GiB/s 43.028 GiB/s 43.698 GiB/s]
                 change:
                        time:   [−3.5717% −1.6543% +0.1391%] (p = 0.08 > 0.05)
                        thrpt:  [−0.1389% +1.6822% +3.7040%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
bltp_event_frame/write_batch/1
                        time:   [18.894 ns 18.988 ns 19.084 ns]
                        thrpt:  [3.1234 GiB/s 3.1390 GiB/s 3.1546 GiB/s]
                 change:
                        time:   [−0.6903% −0.1731% +0.3602%] (p = 0.53 > 0.05)
                        thrpt:  [−0.3589% +0.1734% +0.6951%]
                        No change in performance detected.
bltp_event_frame/write_batch/10
                        time:   [73.161 ns 73.368 ns 73.580 ns]
                        thrpt:  [8.1006 GiB/s 8.1240 GiB/s 8.1471 GiB/s]
                 change:
                        time:   [+0.7658% +1.3071% +1.8681%] (p = 0.00 < 0.05)
                        thrpt:  [−1.8338% −1.2903% −0.7600%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
bltp_event_frame/write_batch/50
                        time:   [155.71 ns 156.61 ns 158.18 ns]
                        thrpt:  [18.840 GiB/s 19.030 GiB/s 19.140 GiB/s]
                 change:
                        time:   [−0.3500% +0.2841% +0.9563%] (p = 0.43 > 0.05)
                        thrpt:  [−0.9472% −0.2833% +0.3513%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
bltp_event_frame/write_batch/100
                        time:   [284.32 ns 285.39 ns 286.46 ns]
                        thrpt:  [20.807 GiB/s 20.886 GiB/s 20.964 GiB/s]
                 change:
                        time:   [−0.5411% −0.0060% +0.5477%] (p = 0.98 > 0.05)
                        thrpt:  [−0.5447% +0.0060% +0.5441%]
                        No change in performance detected.
bltp_event_frame/read_batch_10
                        time:   [146.88 ns 148.38 ns 150.24 ns]
                        thrpt:  [66.560 Melem/s 67.393 Melem/s 68.081 Melem/s]
                 change:
                        time:   [+2.2649% +4.2572% +6.7692%] (p = 0.00 < 0.05)
                        thrpt:  [−6.3400% −4.0834% −2.2148%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe

bltp_packet_pool/get_return/16
                        time:   [34.199 ns 34.317 ns 34.434 ns]
                        thrpt:  [29.041 Melem/s 29.140 Melem/s 29.241 Melem/s]
                 change:
                        time:   [−0.8716% −0.3890% +0.0881%] (p = 0.10 > 0.05)
                        thrpt:  [−0.0881% +0.3905% +0.8792%]
                        No change in performance detected.
bltp_packet_pool/get_return/64
                        time:   [34.110 ns 34.230 ns 34.350 ns]
                        thrpt:  [29.112 Melem/s 29.214 Melem/s 29.317 Melem/s]
                 change:
                        time:   [−0.4646% +0.0642% +0.5914%] (p = 0.81 > 0.05)
                        thrpt:  [−0.5879% −0.0642% +0.4668%]
                        No change in performance detected.
bltp_packet_pool/get_return/256
                        time:   [34.137 ns 34.283 ns 34.425 ns]
                        thrpt:  [29.049 Melem/s 29.169 Melem/s 29.294 Melem/s]
                 change:
                        time:   [−0.5189% −0.0090% +0.5087%] (p = 0.98 > 0.05)
                        thrpt:  [−0.5061% +0.0090% +0.5216%]
                        No change in performance detected.

bltp_packet_build/build_packet/1
                        time:   [735.36 ns 739.56 ns 743.72 ns]
                        thrpt:  [82.068 MiB/s 82.529 MiB/s 83.000 MiB/s]
                 change:
                        time:   [−1.0949% −0.4360% +0.2181%] (p = 0.19 > 0.05)
                        thrpt:  [−0.2176% +0.4379% +1.1071%]
                        No change in performance detected.
bltp_packet_build/build_packet/10
                        time:   [2.5036 µs 2.5199 µs 2.5362 µs]
                        thrpt:  [240.65 MiB/s 242.21 MiB/s 243.79 MiB/s]
                 change:
                        time:   [−1.3916% −0.6690% +0.0554%] (p = 0.08 > 0.05)
                        thrpt:  [−0.0554% +0.6735% +1.4113%]
                        No change in performance detected.
bltp_packet_build/build_packet/50
                        time:   [10.760 µs 10.820 µs 10.879 µs]
                        thrpt:  [280.52 MiB/s 282.06 MiB/s 283.63 MiB/s]
                 change:
                        time:   [−1.9722% −1.4031% −0.8521%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8595% +1.4231% +2.0119%]
                        Change within noise threshold.

bltp_encryption/encrypt/64
                        time:   [736.38 ns 742.76 ns 751.26 ns]
                        thrpt:  [81.244 MiB/s 82.173 MiB/s 82.886 MiB/s]
                 change:
                        time:   [−2.0478% −1.5100% −0.8950%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9031% +1.5331% +2.0906%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
bltp_encryption/encrypt/256
                        time:   [1.3119 µs 1.3187 µs 1.3254 µs]
                        thrpt:  [184.20 MiB/s 185.14 MiB/s 186.10 MiB/s]
                 change:
                        time:   [−0.5763% +0.1196% +0.8115%] (p = 0.75 > 0.05)
                        thrpt:  [−0.8050% −0.1194% +0.5796%]
                        No change in performance detected.
bltp_encryption/encrypt/1024
                        time:   [3.6303 µs 3.6486 µs 3.6669 µs]
                        thrpt:  [266.32 MiB/s 267.66 MiB/s 269.01 MiB/s]
                 change:
                        time:   [−1.0135% −0.3325% +0.3732%] (p = 0.35 > 0.05)
                        thrpt:  [−0.3718% +0.3336% +1.0239%]
                        No change in performance detected.
bltp_encryption/encrypt/4096
                        time:   [12.830 µs 12.944 µs 13.121 µs]
                        thrpt:  [297.72 MiB/s 301.77 MiB/s 304.45 MiB/s]
                 change:
                        time:   [−1.0247% −0.2523% +0.8015%] (p = 0.61 > 0.05)
                        thrpt:  [−0.7951% +0.2529% +1.0353%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

bltp_keypair/generate   time:   [34.481 µs 34.651 µs 34.825 µs]
                        thrpt:  [28.715 Kelem/s 28.859 Kelem/s 29.001 Kelem/s]
                 change:
                        time:   [−0.5193% +0.1247% +0.7883%] (p = 0.71 > 0.05)
                        thrpt:  [−0.7821% −0.1246% +0.5220%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

bltp_aad/generate       time:   [2.0885 ns 2.0996 ns 2.1108 ns]
                        thrpt:  [473.76 Melem/s 476.27 Melem/s 478.80 Melem/s]
                 change:
                        time:   [−0.5291% +0.1100% +0.7688%] (p = 0.75 > 0.05)
                        thrpt:  [−0.7629% −0.1099% +0.5319%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

pool_comparison/shared_pool_get_return
                        time:   [33.999 ns 34.152 ns 34.306 ns]
                        thrpt:  [29.150 Melem/s 29.281 Melem/s 29.413 Melem/s]
                 change:
                        time:   [−1.0438% −0.4925% +0.0847%] (p = 0.09 > 0.05)
                        thrpt:  [−0.0846% +0.4949% +1.0549%]
                        No change in performance detected.
pool_comparison/thread_local_pool_get_return
                        time:   [45.579 ns 45.752 ns 45.925 ns]
                        thrpt:  [21.775 Melem/s 21.857 Melem/s 21.940 Melem/s]
                 change:
                        time:   [−0.4284% +0.1768% +0.8813%] (p = 0.59 > 0.05)
                        thrpt:  [−0.8736% −0.1765% +0.4303%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
pool_comparison/shared_pool_10x
                        time:   [318.00 ns 319.20 ns 320.41 ns]
                        thrpt:  [3.1210 Melem/s 3.1328 Melem/s 3.1447 Melem/s]
                 change:
                        time:   [−0.4064% +0.2243% +0.8586%] (p = 0.48 > 0.05)
                        thrpt:  [−0.8513% −0.2238% +0.4080%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
pool_comparison/thread_local_pool_10x
                        time:   [526.12 ns 528.45 ns 530.76 ns]
                        thrpt:  [1.8841 Melem/s 1.8923 Melem/s 1.9007 Melem/s]
                 change:
                        time:   [−0.6894% −0.0747% +0.5033%] (p = 0.80 > 0.05)
                        thrpt:  [−0.5008% +0.0748% +0.6942%]
                        No change in performance detected.

cipher_comparison/shared_pool/64
                        time:   [739.83 ns 750.90 ns 765.57 ns]
                        thrpt:  [79.725 MiB/s 81.283 MiB/s 82.498 MiB/s]
                 change:
                        time:   [−0.5100% +0.4413% +1.4230%] (p = 0.40 > 0.05)
                        thrpt:  [−1.4031% −0.4394% +0.5126%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
cipher_comparison/fast_chacha20/64
                        time:   [743.12 ns 749.02 ns 756.18 ns]
                        thrpt:  [80.715 MiB/s 81.486 MiB/s 82.134 MiB/s]
                 change:
                        time:   [−0.5820% +0.1807% +0.9166%] (p = 0.64 > 0.05)
                        thrpt:  [−0.9082% −0.1804% +0.5854%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
cipher_comparison/shared_pool/256
                        time:   [1.3112 µs 1.3188 µs 1.3264 µs]
                        thrpt:  [184.06 MiB/s 185.12 MiB/s 186.19 MiB/s]
                 change:
                        time:   [−1.4507% −0.8142% −0.1045%] (p = 0.02 < 0.05)
                        thrpt:  [+0.1046% +0.8209% +1.4720%]
                        Change within noise threshold.
cipher_comparison/fast_chacha20/256
                        time:   [1.3088 µs 1.3147 µs 1.3207 µs]
                        thrpt:  [184.86 MiB/s 185.70 MiB/s 186.54 MiB/s]
                 change:
                        time:   [−0.0282% +0.6198% +1.2469%] (p = 0.06 > 0.05)
                        thrpt:  [−1.2315% −0.6160% +0.0282%]
                        No change in performance detected.
cipher_comparison/shared_pool/1024
                        time:   [3.6339 µs 3.6566 µs 3.6838 µs]
                        thrpt:  [265.10 MiB/s 267.07 MiB/s 268.74 MiB/s]
                 change:
                        time:   [−0.1550% +0.7718% +1.8793%] (p = 0.16 > 0.05)
                        thrpt:  [−1.8446% −0.7659% +0.1553%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
cipher_comparison/fast_chacha20/1024
                        time:   [3.5914 µs 3.6104 µs 3.6291 µs]
                        thrpt:  [269.09 MiB/s 270.49 MiB/s 271.92 MiB/s]
                 change:
                        time:   [−0.8956% −0.2234% +0.5394%] (p = 0.54 > 0.05)
                        thrpt:  [−0.5365% +0.2239% +0.9037%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
cipher_comparison/shared_pool/4096
                        time:   [13.007 µs 13.053 µs 13.099 µs]
                        thrpt:  [298.21 MiB/s 299.26 MiB/s 300.32 MiB/s]
                 change:
                        time:   [+0.9340% +1.4773% +2.0369%] (p = 0.00 < 0.05)
                        thrpt:  [−1.9962% −1.4557% −0.9254%]
                        Change within noise threshold.
cipher_comparison/fast_chacha20/4096
                        time:   [12.729 µs 12.824 µs 12.949 µs]
                        thrpt:  [301.68 MiB/s 304.61 MiB/s 306.89 MiB/s]
                 change:
                        time:   [−0.2722% +0.4931% +1.4095%] (p = 0.27 > 0.05)
                        thrpt:  [−1.3899% −0.4906% +0.2730%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

adaptive_batcher/optimal_size
                        time:   [1.0111 ns 1.0156 ns 1.0200 ns]
                        thrpt:  [980.40 Melem/s 984.68 Melem/s 989.04 Melem/s]
                 change:
                        time:   [−0.7440% −0.1307% +0.4548%] (p = 0.67 > 0.05)
                        thrpt:  [−0.4527% +0.1309% +0.7496%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
adaptive_batcher/record time:   [2.6563 ns 2.6674 ns 2.6784 ns]
                        thrpt:  [373.36 Melem/s 374.90 Melem/s 376.47 Melem/s]
                 change:
                        time:   [−1.4135% −0.8606% −0.2884%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2892% +0.8680% +1.4338%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
adaptive_batcher/full_cycle
                        time:   [2.9126 ns 2.9280 ns 2.9484 ns]
                        thrpt:  [339.16 Melem/s 341.53 Melem/s 343.34 Melem/s]
                 change:
                        time:   [−0.4010% +1.0322% +3.1639%] (p = 0.32 > 0.05)
                        thrpt:  [−3.0669% −1.0217% +0.4026%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

e2e_packet_build/shared_pool_50_events
                        time:   [10.820 µs 10.869 µs 10.916 µs]
                        thrpt:  [279.56 MiB/s 280.79 MiB/s 282.05 MiB/s]
                 change:
                        time:   [−0.0090% +0.6599% +1.3174%] (p = 0.05 < 0.05)
                        thrpt:  [−1.3003% −0.6555% +0.0090%]
                        Change within noise threshold.
e2e_packet_build/fast_50_events
                        time:   [10.733 µs 10.787 µs 10.841 µs]
                        thrpt:  [281.51 MiB/s 282.90 MiB/s 284.32 MiB/s]
                 change:
                        time:   [−0.7313% −0.0480% +0.6411%] (p = 0.90 > 0.05)
                        thrpt:  [−0.6370% +0.0480% +0.7367%]
                        No change in performance detected.

multithread_packet_build/shared_pool/8
                        time:   [2.5253 ms 2.5449 ms 2.5649 ms]
                        thrpt:  [3.1191 Melem/s 3.1435 Melem/s 3.1680 Melem/s]
                 change:
                        time:   [+0.8529% +3.0714% +5.3858%] (p = 0.01 < 0.05)
                        thrpt:  [−5.1106% −2.9798% −0.8457%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.8s, enable flat sampling, or reduce sample count to 60.
multithread_packet_build/thread_local_pool/8
                        time:   [1.3355 ms 1.3421 ms 1.3487 ms]
                        thrpt:  [5.9315 Melem/s 5.9609 Melem/s 5.9903 Melem/s]
                 change:
                        time:   [−0.2563% +1.2654% +2.8039%] (p = 0.11 > 0.05)
                        thrpt:  [−2.7274% −1.2496% +0.2570%]
                        No change in performance detected.
Found 10 outliers among 100 measurements (10.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  4 (4.00%) high mild
  3 (3.00%) high severe
multithread_packet_build/shared_pool/16
                        time:   [4.9551 ms 5.0687 ms 5.1984 ms]
                        thrpt:  [3.0779 Melem/s 3.1566 Melem/s 3.2290 Melem/s]
                 change:
                        time:   [−2.3561% +1.0388% +4.9155%] (p = 0.57 > 0.05)
                        thrpt:  [−4.6852% −1.0281% +2.4129%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
multithread_packet_build/thread_local_pool/16
                        time:   [2.0825 ms 2.1354 ms 2.1878 ms]
                        thrpt:  [7.3134 Melem/s 7.4929 Melem/s 7.6832 Melem/s]
                 change:
                        time:   [−2.8830% +0.6147% +4.3757%] (p = 0.74 > 0.05)
                        thrpt:  [−4.1923% −0.6110% +2.9686%]
                        No change in performance detected.
multithread_packet_build/shared_pool/24
                        time:   [7.5323 ms 7.7371 ms 7.9640 ms]
                        thrpt:  [3.0136 Melem/s 3.1019 Melem/s 3.1863 Melem/s]
                 change:
                        time:   [−3.0279% +0.6372% +4.5453%] (p = 0.74 > 0.05)
                        thrpt:  [−4.3477% −0.6332% +3.1224%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
multithread_packet_build/thread_local_pool/24
                        time:   [2.9568 ms 3.0211 ms 3.0870 ms]
                        thrpt:  [7.7746 Melem/s 7.9441 Melem/s 8.1170 Melem/s]
                 change:
                        time:   [−1.8643% +1.0028% +3.9206%] (p = 0.50 > 0.05)
                        thrpt:  [−3.7727% −0.9929% +1.8997%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
multithread_packet_build/shared_pool/32
                        time:   [10.161 ms 10.527 ms 10.921 ms]
                        thrpt:  [2.9301 Melem/s 3.0398 Melem/s 3.1492 Melem/s]
                 change:
                        time:   [−0.1268% +4.8475% +9.9290%] (p = 0.05 > 0.05)
                        thrpt:  [−9.0322% −4.6234% +0.1269%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
multithread_packet_build/thread_local_pool/32
                        time:   [3.7659 ms 3.8275 ms 3.8875 ms]
                        thrpt:  [8.2314 Melem/s 8.3605 Melem/s 8.4974 Melem/s]
                 change:
                        time:   [−2.1717% +0.0344% +2.4129%] (p = 0.97 > 0.05)
                        thrpt:  [−2.3561% −0.0344% +2.2199%]
                        No change in performance detected.

multithread_mixed_frames/shared_mixed/8
                        time:   [2.1620 ms 2.2018 ms 2.2445 ms]
                        thrpt:  [5.3465 Melem/s 5.4502 Melem/s 5.5504 Melem/s]
                 change:
                        time:   [−3.4232% −1.6292% +0.5039%] (p = 0.12 > 0.05)
                        thrpt:  [−0.5014% +1.6562% +3.5445%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.5s, enable flat sampling, or reduce sample count to 50.
multithread_mixed_frames/fast_mixed/8
                        time:   [1.6425 ms 1.6787 ms 1.7140 ms]
                        thrpt:  [7.0010 Melem/s 7.1484 Melem/s 7.3058 Melem/s]
                 change:
                        time:   [−6.9309% −4.0126% −0.7856%] (p = 0.01 < 0.05)
                        thrpt:  [+0.7918% +4.1804% +7.4471%]
                        Change within noise threshold.
multithread_mixed_frames/shared_mixed/16
                        time:   [3.9562 ms 4.0314 ms 4.1089 ms]
                        thrpt:  [5.8409 Melem/s 5.9532 Melem/s 6.0665 Melem/s]
                 change:
                        time:   [−2.4851% −0.0956% +2.2705%] (p = 0.93 > 0.05)
                        thrpt:  [−2.2201% +0.0957% +2.5485%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
multithread_mixed_frames/fast_mixed/16
                        time:   [2.8194 ms 2.8845 ms 2.9484 ms]
                        thrpt:  [8.1399 Melem/s 8.3203 Melem/s 8.5124 Melem/s]
                 change:
                        time:   [−19.193% −8.0255% −0.1357%] (p = 0.17 > 0.05)
                        thrpt:  [+0.1359% +8.7258% +23.751%]
                        No change in performance detected.
multithread_mixed_frames/shared_mixed/24
                        time:   [5.7640 ms 5.8829 ms 6.0163 ms]
                        thrpt:  [5.9838 Melem/s 6.1194 Melem/s 6.2457 Melem/s]
                 change:
                        time:   [−5.2637% −2.5156% +0.2882%] (p = 0.09 > 0.05)
                        thrpt:  [−0.2874% +2.5805% +5.5561%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
multithread_mixed_frames/fast_mixed/24
                        time:   [4.0098 ms 4.0865 ms 4.1709 ms]
                        thrpt:  [8.6311 Melem/s 8.8096 Melem/s 8.9779 Melem/s]
                 change:
                        time:   [+1.0487% +3.3509% +6.0899%] (p = 0.01 < 0.05)
                        thrpt:  [−5.7403% −3.2423% −1.0378%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
multithread_mixed_frames/shared_mixed/32
                        time:   [7.2264 ms 7.3288 ms 7.4388 ms]
                        thrpt:  [6.4526 Melem/s 6.5495 Melem/s 6.6423 Melem/s]
                 change:
                        time:   [−5.7764% −3.5864% −1.2700%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2864% +3.7198% +6.1306%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
multithread_mixed_frames/fast_mixed/32
                        time:   [5.1390 ms 5.2124 ms 5.2844 ms]
                        thrpt:  [9.0833 Melem/s 9.2087 Melem/s 9.3403 Melem/s]
                 change:
                        time:   [+8.0987% +9.7744% +11.449%] (p = 0.00 < 0.05)
                        thrpt:  [−10.273% −8.9041% −7.4919%]
                        Performance has regressed.

pool_contention/shared_acquire_release/8
                        time:   [14.013 ms 14.503 ms 14.987 ms]
                        thrpt:  [5.3379 Melem/s 5.5161 Melem/s 5.7090 Melem/s]
                 change:
                        time:   [−2.7184% +2.0020% +7.2711%] (p = 0.44 > 0.05)
                        thrpt:  [−6.7782% −1.9627% +2.7944%]
                        No change in performance detected.
pool_contention/fast_acquire_release/8
                        time:   [979.21 µs 985.95 µs 992.14 µs]
                        thrpt:  [80.634 Melem/s 81.140 Melem/s 81.698 Melem/s]
                 change:
                        time:   [+1.7654% +4.3324% +6.7444%] (p = 0.00 < 0.05)
                        thrpt:  [−6.3183% −4.1525% −1.7348%]
                        Performance has regressed.
Found 17 outliers among 100 measurements (17.00%)
  5 (5.00%) low severe
  8 (8.00%) low mild
  4 (4.00%) high mild
pool_contention/shared_acquire_release/16
                        time:   [29.392 ms 30.569 ms 31.717 ms]
                        thrpt:  [5.0446 Melem/s 5.2341 Melem/s 5.4437 Melem/s]
                 change:
                        time:   [−10.772% −1.5575% +6.4598%] (p = 0.76 > 0.05)
                        thrpt:  [−6.0679% +1.5821% +12.072%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
Benchmarking pool_contention/fast_acquire_release/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.1s, enable flat sampling, or reduce sample count to 50.
pool_contention/fast_acquire_release/16
                        time:   [1.5191 ms 1.5572 ms 1.5954 ms]
                        thrpt:  [100.29 Melem/s 102.75 Melem/s 105.32 Melem/s]
                 change:
                        time:   [−4.7861% −2.1447% +0.6071%] (p = 0.11 > 0.05)
                        thrpt:  [−0.6034% +2.1917% +5.0267%]
                        No change in performance detected.
pool_contention/shared_acquire_release/24
                        time:   [46.306 ms 47.616 ms 48.940 ms]
                        thrpt:  [4.9040 Melem/s 5.0403 Melem/s 5.1829 Melem/s]
                 change:
                        time:   [+0.3025% +4.2544% +8.5415%] (p = 0.04 < 0.05)
                        thrpt:  [−7.8693% −4.0808% −0.3016%]
                        Change within noise threshold.
pool_contention/fast_acquire_release/24
                        time:   [2.1314 ms 2.1800 ms 2.2281 ms]
                        thrpt:  [107.71 Melem/s 110.09 Melem/s 112.60 Melem/s]
                 change:
                        time:   [+5.7499% +8.4945% +11.235%] (p = 0.00 < 0.05)
                        thrpt:  [−10.100% −7.8294% −5.4372%]
                        Performance has regressed.
Benchmarking pool_contention/shared_acquire_release/32: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.8s, or reduce sample count to 70.
pool_contention/shared_acquire_release/32
                        time:   [62.306 ms 64.580 ms 66.882 ms]
                        thrpt:  [4.7845 Melem/s 4.9551 Melem/s 5.1360 Melem/s]
                 change:
                        time:   [+0.0587% +4.9421% +10.332%] (p = 0.05 > 0.05)
                        thrpt:  [−9.3641% −4.7094% −0.0587%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
pool_contention/fast_acquire_release/32
                        time:   [2.7401 ms 2.8017 ms 2.8640 ms]
                        thrpt:  [111.73 Melem/s 114.22 Melem/s 116.78 Melem/s]
                 change:
                        time:   [+7.8731% +10.546% +13.105%] (p = 0.00 < 0.05)
                        thrpt:  [−11.587% −9.5401% −7.2985%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

throughput_scaling/fast_pool_scaling/1
                        time:   [8.8694 ms 8.9148 ms 8.9950 ms]
                        thrpt:  [222.35 Kelem/s 224.35 Kelem/s 225.49 Kelem/s]
                 change:
                        time:   [+1.5538% +2.2663% +3.0218%] (p = 0.00 < 0.05)
                        thrpt:  [−2.9332% −2.2161% −1.5300%]
                        Performance has regressed.
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low severe
  1 (5.00%) high mild
  2 (10.00%) high severe
throughput_scaling/fast_pool_scaling/2
                        time:   [9.0184 ms 9.0292 ms 9.0396 ms]
                        thrpt:  [442.50 Kelem/s 443.01 Kelem/s 443.54 Kelem/s]
                 change:
                        time:   [−4.2072% −2.3041% −0.9498%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9589% +2.3584% +4.3919%]
                        Change within noise threshold.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low severe
  1 (5.00%) high severe
throughput_scaling/fast_pool_scaling/4
                        time:   [9.3245 ms 9.3492 ms 9.3738 ms]
                        thrpt:  [853.44 Kelem/s 855.69 Kelem/s 857.95 Kelem/s]
                 change:
                        time:   [−2.3980% −1.0681% +0.0933%] (p = 0.11 > 0.05)
                        thrpt:  [−0.0932% +1.0797% +2.4569%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
throughput_scaling/fast_pool_scaling/8
                        time:   [11.822 ms 12.241 ms 12.758 ms]
                        thrpt:  [1.2541 Melem/s 1.3071 Melem/s 1.3534 Melem/s]
                 change:
                        time:   [−15.745% −11.787% −7.1510%] (p = 0.00 < 0.05)
                        thrpt:  [+7.7017% +13.362% +18.687%]
                        Performance has improved.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.1s, enable flat sampling, or reduce sample count to 10.
throughput_scaling/fast_pool_scaling/16
                        time:   [24.127 ms 24.331 ms 24.550 ms]
                        thrpt:  [1.3035 Melem/s 1.3152 Melem/s 1.3263 Melem/s]
                 change:
                        time:   [+19.793% +22.152% +24.592%] (p = 0.00 < 0.05)
                        thrpt:  [−19.738% −18.135% −16.523%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high severe
Benchmarking throughput_scaling/fast_pool_scaling/24: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.5s, enable flat sampling, or reduce sample count to 10.
throughput_scaling/fast_pool_scaling/24
                        time:   [30.006 ms 30.225 ms 30.477 ms]
                        thrpt:  [1.5750 Melem/s 1.5881 Melem/s 1.5997 Melem/s]
                 change:
                        time:   [+4.3598% +6.1658% +8.7128%] (p = 0.00 < 0.05)
                        thrpt:  [−8.0145% −5.8077% −4.1776%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe
Benchmarking throughput_scaling/fast_pool_scaling/32: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 8.3s, enable flat sampling, or reduce sample count to 10.
throughput_scaling/fast_pool_scaling/32
                        time:   [39.057 ms 39.469 ms 40.019 ms]
                        thrpt:  [1.5992 Melem/s 1.6215 Melem/s 1.6386 Melem/s]
                 change:
                        time:   [+3.5504% +4.8249% +6.2264%] (p = 0.00 < 0.05)
                        thrpt:  [−5.8614% −4.6028% −3.4287%]
                        Performance has regressed.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe

routing_header/serialize
                        time:   [579.19 ps 584.17 ps 593.32 ps]
                        thrpt:  [1.6854 Gelem/s 1.7118 Gelem/s 1.7265 Gelem/s]
                 change:
                        time:   [−6.1412% −1.0886% +2.0006%] (p = 0.79 > 0.05)
                        thrpt:  [−1.9614% +1.1006% +6.5430%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
routing_header/deserialize
                        time:   [984.43 ps 987.82 ps 991.00 ps]
                        thrpt:  [1.0091 Gelem/s 1.0123 Gelem/s 1.0158 Gelem/s]
                 change:
                        time:   [+0.4599% +0.9131% +1.3552%] (p = 0.00 < 0.05)
                        thrpt:  [−1.3370% −0.9048% −0.4578%]
                        Change within noise threshold.
routing_header/roundtrip
                        time:   [972.43 ps 975.83 ps 979.22 ps]
                        thrpt:  [1.0212 Gelem/s 1.0248 Gelem/s 1.0284 Gelem/s]
                 change:
                        time:   [−1.6065% +0.2482% +1.6539%] (p = 0.80 > 0.05)
                        thrpt:  [−1.6270% −0.2476% +1.6328%]
                        No change in performance detected.
routing_header/forward  time:   [577.87 ps 580.35 ps 584.40 ps]
                        thrpt:  [1.7111 Gelem/s 1.7231 Gelem/s 1.7305 Gelem/s]
                 change:
                        time:   [+0.6899% +1.1401% +1.6493%] (p = 0.00 < 0.05)
                        thrpt:  [−1.6226% −1.1272% −0.6851%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe

routing_table/lookup_hit
                        time:   [19.188 ns 20.194 ns 21.212 ns]
                        thrpt:  [47.143 Melem/s 49.519 Melem/s 52.115 Melem/s]
                 change:
                        time:   [−14.158% −5.9013% +1.2827%] (p = 0.19 > 0.05)
                        thrpt:  [−1.2665% +6.2714% +16.493%]
                        No change in performance detected.
routing_table/lookup_miss
                        time:   [11.803 ns 12.146 ns 12.502 ns]
                        thrpt:  [79.986 Melem/s 82.334 Melem/s 84.727 Melem/s]
                 change:
                        time:   [−10.262% −6.8721% −3.4720%] (p = 0.00 < 0.05)
                        thrpt:  [+3.5969% +7.3792% +11.436%]
                        Performance has improved.
routing_table/is_local  time:   [323.67 ps 324.84 ps 326.00 ps]
                        thrpt:  [3.0675 Gelem/s 3.0784 Gelem/s 3.0896 Gelem/s]
                 change:
                        time:   [−0.2429% +0.1702% +0.5837%] (p = 0.43 > 0.05)
                        thrpt:  [−0.5803% −0.1699% +0.2435%]
                        No change in performance detected.
routing_table/add_route time:   [298.12 ns 302.00 ns 305.52 ns]
                        thrpt:  [3.2731 Melem/s 3.3112 Melem/s 3.3543 Melem/s]
                 change:
                        time:   [+5.9082% +12.993% +20.659%] (p = 0.00 < 0.05)
                        thrpt:  [−17.122% −11.499% −5.5786%]
                        Performance has regressed.
Found 14 outliers among 100 measurements (14.00%)
  13 (13.00%) low mild
  1 (1.00%) high mild
routing_table/record_in time:   [52.092 ns 52.627 ns 53.159 ns]
                        thrpt:  [18.811 Melem/s 19.002 Melem/s 19.197 Melem/s]
                 change:
                        time:   [−0.5781% +0.8812% +2.3494%] (p = 0.23 > 0.05)
                        thrpt:  [−2.2954% −0.8735% +0.5814%]
                        No change in performance detected.
routing_table/record_out
                        time:   [19.346 ns 19.720 ns 20.085 ns]
                        thrpt:  [49.789 Melem/s 50.710 Melem/s 51.692 Melem/s]
                 change:
                        time:   [−0.7274% +2.1406% +4.7852%] (p = 0.13 > 0.05)
                        thrpt:  [−4.5667% −2.0958% +0.7327%]
                        No change in performance detected.
routing_table/aggregate_stats
                        time:   [2.1708 µs 2.1794 µs 2.1881 µs]
                        thrpt:  [457.01 Kelem/s 458.83 Kelem/s 460.67 Kelem/s]
                 change:
                        time:   [−0.4535% +0.2131% +0.8448%] (p = 0.53 > 0.05)
                        thrpt:  [−0.8378% −0.2127% +0.4556%]
                        No change in performance detected.

fair_scheduler/creation time:   [300.75 ns 302.08 ns 303.41 ns]
                        thrpt:  [3.2959 Melem/s 3.3104 Melem/s 3.3251 Melem/s]
                 change:
                        time:   [−0.1754% +0.4075% +0.9748%] (p = 0.17 > 0.05)
                        thrpt:  [−0.9654% −0.4058% +0.1758%]
                        No change in performance detected.
fair_scheduler/stream_count_empty
                        time:   [209.96 ns 211.23 ns 212.73 ns]
                        thrpt:  [4.7008 Melem/s 4.7341 Melem/s 4.7629 Melem/s]
                 change:
                        time:   [+0.2971% +1.0957% +1.9024%] (p = 0.01 < 0.05)
                        thrpt:  [−1.8669% −1.0839% −0.2962%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
fair_scheduler/total_queued
                        time:   [323.82 ps 324.82 ps 325.79 ps]
                        thrpt:  [3.0695 Gelem/s 3.0786 Gelem/s 3.0881 Gelem/s]
                 change:
                        time:   [−0.1131% +0.3975% +0.8872%] (p = 0.14 > 0.05)
                        thrpt:  [−0.8794% −0.3959% +0.1133%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
fair_scheduler/cleanup_empty
                        time:   [207.81 ns 208.74 ns 209.67 ns]
                        thrpt:  [4.7694 Melem/s 4.7907 Melem/s 4.8121 Melem/s]
                 change:
                        time:   [+0.2040% +0.8789% +1.5614%] (p = 0.01 < 0.05)
                        thrpt:  [−1.5373% −0.8713% −0.2036%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

routing_table_concurrent/concurrent_lookup/4
                        time:   [123.01 µs 127.30 µs 131.27 µs]
                        thrpt:  [30.471 Melem/s 31.423 Melem/s 32.518 Melem/s]
                 change:
                        time:   [−2.5469% +0.9753% +4.9132%] (p = 0.61 > 0.05)
                        thrpt:  [−4.6831% −0.9659% +2.6135%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  8 (8.00%) low mild
routing_table_concurrent/concurrent_stats/4
                        time:   [302.44 µs 308.32 µs 314.15 µs]
                        thrpt:  [12.733 Melem/s 12.974 Melem/s 13.226 Melem/s]
                 change:
                        time:   [−5.7943% −3.2824% −0.7331%] (p = 0.01 < 0.05)
                        thrpt:  [+0.7385% +3.3938% +6.1507%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
routing_table_concurrent/concurrent_lookup/8
                        time:   [203.31 µs 209.67 µs 215.79 µs]
                        thrpt:  [37.073 Melem/s 38.155 Melem/s 39.348 Melem/s]
                 change:
                        time:   [−10.507% −7.7959% −4.9286%] (p = 0.00 < 0.05)
                        thrpt:  [+5.1841% +8.4551% +11.740%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
routing_table_concurrent/concurrent_stats/8
                        time:   [487.15 µs 506.51 µs 526.21 µs]
                        thrpt:  [15.203 Melem/s 15.794 Melem/s 16.422 Melem/s]
                 change:
                        time:   [−9.5824% −5.3189% −0.8074%] (p = 0.02 < 0.05)
                        thrpt:  [+0.8140% +5.6176% +10.598%]
                        Change within noise threshold.
routing_table_concurrent/concurrent_lookup/16
                        time:   [369.50 µs 383.71 µs 397.57 µs]
                        thrpt:  [40.245 Melem/s 41.698 Melem/s 43.302 Melem/s]
                 change:
                        time:   [−6.3718% −1.8280% +2.8395%] (p = 0.44 > 0.05)
                        thrpt:  [−2.7611% +1.8621% +6.8055%]
                        No change in performance detected.
Benchmarking routing_table_concurrent/concurrent_stats/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.3s, enable flat sampling, or reduce sample count to 60.
routing_table_concurrent/concurrent_stats/16
                        time:   [1.0059 ms 1.0359 ms 1.0668 ms]
                        thrpt:  [14.998 Melem/s 15.445 Melem/s 15.906 Melem/s]
                 change:
                        time:   [+8.5843% +12.814% +17.365%] (p = 0.00 < 0.05)
                        thrpt:  [−14.796% −11.358% −7.9057%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

routing_decision/parse_lookup_forward
                        time:   [17.472 ns 17.958 ns 18.469 ns]
                        thrpt:  [54.144 Melem/s 55.685 Melem/s 57.235 Melem/s]
                 change:
                        time:   [−3.1342% +0.7751% +4.5935%] (p = 0.71 > 0.05)
                        thrpt:  [−4.3918% −0.7692% +3.2356%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
routing_decision/full_with_stats
                        time:   [74.999 ns 75.326 ns 75.660 ns]
                        thrpt:  [13.217 Melem/s 13.276 Melem/s 13.333 Melem/s]
                 change:
                        time:   [−0.1121% +0.5945% +1.3352%] (p = 0.12 > 0.05)
                        thrpt:  [−1.3176% −0.5910% +0.1123%]
                        No change in performance detected.

stream_multiplexing/lookup_all/10
                        time:   [126.41 ns 127.34 ns 128.35 ns]
                        thrpt:  [77.911 Melem/s 78.529 Melem/s 79.106 Melem/s]
                 change:
                        time:   [−1.3310% −0.2877% +0.9026%] (p = 0.60 > 0.05)
                        thrpt:  [−0.8945% +0.2885% +1.3489%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
stream_multiplexing/stats_all/10
                        time:   [520.59 ns 525.86 ns 532.74 ns]
                        thrpt:  [18.771 Melem/s 19.016 Melem/s 19.209 Melem/s]
                 change:
                        time:   [+0.7756% +2.4493% +4.2152%] (p = 0.00 < 0.05)
                        thrpt:  [−4.0447% −2.3907% −0.7697%]
                        Change within noise threshold.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low severe
  6 (6.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
stream_multiplexing/lookup_all/100
                        time:   [1.2795 µs 1.2897 µs 1.3002 µs]
                        thrpt:  [76.911 Melem/s 77.540 Melem/s 78.154 Melem/s]
                 change:
                        time:   [−3.5398% −2.0769% −0.5458%] (p = 0.01 < 0.05)
                        thrpt:  [+0.5488% +2.1210% +3.6697%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
stream_multiplexing/stats_all/100
                        time:   [5.3232 µs 5.3736 µs 5.4265 µs]
                        thrpt:  [18.428 Melem/s 18.609 Melem/s 18.786 Melem/s]
                 change:
                        time:   [−0.8461% +0.7757% +2.6586%] (p = 0.43 > 0.05)
                        thrpt:  [−2.5898% −0.7697% +0.8533%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) low severe
  3 (3.00%) low mild
  1 (1.00%) high severe
stream_multiplexing/lookup_all/1000
                        time:   [12.999 µs 13.122 µs 13.250 µs]
                        thrpt:  [75.472 Melem/s 76.210 Melem/s 76.930 Melem/s]
                 change:
                        time:   [−2.3151% −0.8180% +0.7026%] (p = 0.30 > 0.05)
                        thrpt:  [−0.6977% +0.8247% +2.3700%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
stream_multiplexing/stats_all/1000
                        time:   [54.591 µs 54.913 µs 55.235 µs]
                        thrpt:  [18.105 Melem/s 18.211 Melem/s 18.318 Melem/s]
                 change:
                        time:   [−1.4964% −0.8504% −0.2212%] (p = 0.02 < 0.05)
                        thrpt:  [+0.2217% +0.8577% +1.5191%]
                        Change within noise threshold.
stream_multiplexing/lookup_all/10000
                        time:   [140.41 µs 141.96 µs 143.54 µs]
                        thrpt:  [69.665 Melem/s 70.440 Melem/s 71.218 Melem/s]
                 change:
                        time:   [−2.8164% −1.1103% +0.5533%] (p = 0.20 > 0.05)
                        thrpt:  [−0.5503% +1.1227% +2.8980%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
stream_multiplexing/stats_all/10000
                        time:   [611.27 µs 617.02 µs 622.62 µs]
                        thrpt:  [16.061 Melem/s 16.207 Melem/s 16.359 Melem/s]
                 change:
                        time:   [−3.9751% −2.2564% −0.6008%] (p = 0.01 < 0.05)
                        thrpt:  [+0.6045% +2.3085% +4.1396%]
                        Change within noise threshold.

multihop_packet_builder/build/64
                        time:   [25.249 ns 25.366 ns 25.482 ns]
                        thrpt:  [2.3391 GiB/s 2.3498 GiB/s 2.3606 GiB/s]
                 change:
                        time:   [−1.3742% −0.8197% −0.2654%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2661% +0.8265% +1.3933%]
                        Change within noise threshold.
multihop_packet_builder/build_priority/64
                        time:   [26.775 ns 26.921 ns 27.067 ns]
                        thrpt:  [2.2021 GiB/s 2.2140 GiB/s 2.2262 GiB/s]
                 change:
                        time:   [−0.5154% +0.1505% +0.7840%] (p = 0.65 > 0.05)
                        thrpt:  [−0.7779% −0.1503% +0.5180%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
multihop_packet_builder/build/256
                        time:   [56.408 ns 56.812 ns 57.216 ns]
                        thrpt:  [4.1670 GiB/s 4.1966 GiB/s 4.2267 GiB/s]
                 change:
                        time:   [+2.7556% +3.6569% +4.5759%] (p = 0.00 < 0.05)
                        thrpt:  [−4.3757% −3.5279% −2.6817%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
multihop_packet_builder/build_priority/256
                        time:   [55.475 ns 55.882 ns 56.286 ns]
                        thrpt:  [4.2358 GiB/s 4.2665 GiB/s 4.2977 GiB/s]
                 change:
                        time:   [+1.9850% +3.0005% +4.0014%] (p = 0.00 < 0.05)
                        thrpt:  [−3.8475% −2.9131% −1.9464%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
multihop_packet_builder/build/1024
                        time:   [43.398 ns 43.533 ns 43.668 ns]
                        thrpt:  [21.839 GiB/s 21.907 GiB/s 21.975 GiB/s]
                 change:
                        time:   [−1.6450% −1.1718% −0.6899%] (p = 0.00 < 0.05)
                        thrpt:  [+0.6947% +1.1857% +1.6725%]
                        Change within noise threshold.
multihop_packet_builder/build_priority/1024
                        time:   [43.263 ns 43.410 ns 43.556 ns]
                        thrpt:  [21.895 GiB/s 21.969 GiB/s 22.044 GiB/s]
                 change:
                        time:   [−2.5742% −1.2839% −0.3421%] (p = 0.02 < 0.05)
                        thrpt:  [+0.3432% +1.3006% +2.6423%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
multihop_packet_builder/build/4096
                        time:   [94.753 ns 97.036 ns 99.179 ns]
                        thrpt:  [38.463 GiB/s 39.312 GiB/s 40.259 GiB/s]
                 change:
                        time:   [−27.658% −15.121% −4.0398%] (p = 0.02 < 0.05)
                        thrpt:  [+4.2099% +17.815% +38.232%]
                        Performance has improved.
multihop_packet_builder/build_priority/4096
                        time:   [93.130 ns 95.018 ns 97.060 ns]
                        thrpt:  [39.302 GiB/s 40.147 GiB/s 40.961 GiB/s]
                 change:
                        time:   [−5.1633% −1.5517% +1.9580%] (p = 0.41 > 0.05)
                        thrpt:  [−1.9204% +1.5761% +5.4445%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

multihop_chain/forward_chain/1
                        time:   [59.242 ns 59.628 ns 60.036 ns]
                        thrpt:  [16.657 Melem/s 16.771 Melem/s 16.880 Melem/s]
                 change:
                        time:   [−0.9506% −0.2050% +0.5371%] (p = 0.59 > 0.05)
                        thrpt:  [−0.5343% +0.2054% +0.9598%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
multihop_chain/forward_chain/2
                        time:   [117.73 ns 118.82 ns 120.71 ns]
                        thrpt:  [8.2845 Melem/s 8.4160 Melem/s 8.4942 Melem/s]
                 change:
                        time:   [+0.5080% +1.4405% +2.6899%] (p = 0.01 < 0.05)
                        thrpt:  [−2.6195% −1.4200% −0.5054%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
multihop_chain/forward_chain/3
                        time:   [167.84 ns 168.94 ns 169.98 ns]
                        thrpt:  [5.8831 Melem/s 5.9194 Melem/s 5.9579 Melem/s]
                 change:
                        time:   [+1.6631% +2.6040% +3.4754%] (p = 0.00 < 0.05)
                        thrpt:  [−3.3587% −2.5379% −1.6358%]
                        Performance has regressed.
multihop_chain/forward_chain/4
                        time:   [229.29 ns 231.17 ns 233.01 ns]
                        thrpt:  [4.2917 Melem/s 4.3258 Melem/s 4.3613 Melem/s]
                 change:
                        time:   [+2.7822% +3.7114% +4.7647%] (p = 0.00 < 0.05)
                        thrpt:  [−4.5480% −3.5786% −2.7068%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
multihop_chain/forward_chain/5
                        time:   [288.97 ns 290.57 ns 292.10 ns]
                        thrpt:  [3.4234 Melem/s 3.4415 Melem/s 3.4605 Melem/s]
                 change:
                        time:   [+4.5584% +5.2655% +5.9979%] (p = 0.00 < 0.05)
                        thrpt:  [−5.6585% −5.0021% −4.3597%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild

hop_latency/single_hop_process
                        time:   [1.3326 ns 1.3413 ns 1.3535 ns]
                        thrpt:  [738.81 Melem/s 745.55 Melem/s 750.43 Melem/s]
                 change:
                        time:   [−0.2725% +0.3615% +1.1017%] (p = 0.31 > 0.05)
                        thrpt:  [−1.0897% −0.3602% +0.2733%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
hop_latency/single_hop_full
                        time:   [57.495 ns 57.879 ns 58.257 ns]
                        thrpt:  [17.165 Melem/s 17.277 Melem/s 17.393 Melem/s]
                 change:
                        time:   [+0.6819% +1.7100% +2.7404%] (p = 0.00 < 0.05)
                        thrpt:  [−2.6673% −1.6813% −0.6773%]
                        Change within noise threshold.

hop_scaling/64B_1hops   time:   [30.256 ns 30.398 ns 30.541 ns]
                        thrpt:  [1.9517 GiB/s 1.9608 GiB/s 1.9700 GiB/s]
                 change:
                        time:   [−0.8293% −0.1293% +0.5712%] (p = 0.72 > 0.05)
                        thrpt:  [−0.5679% +0.1295% +0.8363%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
hop_scaling/64B_2hops   time:   [52.717 ns 52.968 ns 53.221 ns]
                        thrpt:  [1.1199 GiB/s 1.1253 GiB/s 1.1307 GiB/s]
                 change:
                        time:   [−0.8070% −0.2312% +0.3703%] (p = 0.45 > 0.05)
                        thrpt:  [−0.3689% +0.2318% +0.8136%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/64B_3hops   time:   [76.909 ns 77.216 ns 77.515 ns]
                        thrpt:  [787.40 MiB/s 790.45 MiB/s 793.61 MiB/s]
                 change:
                        time:   [−0.2292% +0.2914% +0.8445%] (p = 0.31 > 0.05)
                        thrpt:  [−0.8374% −0.2905% +0.2297%]
                        No change in performance detected.
hop_scaling/64B_4hops   time:   [100.62 ns 101.03 ns 101.45 ns]
                        thrpt:  [601.65 MiB/s 604.10 MiB/s 606.59 MiB/s]
                 change:
                        time:   [−0.6079% −0.0625% +0.4554%] (p = 0.82 > 0.05)
                        thrpt:  [−0.4534% +0.0625% +0.6117%]
                        No change in performance detected.
hop_scaling/64B_5hops   time:   [132.13 ns 132.75 ns 133.37 ns]
                        thrpt:  [457.63 MiB/s 459.77 MiB/s 461.92 MiB/s]
                 change:
                        time:   [−0.6572% −0.0611% +0.5552%] (p = 0.84 > 0.05)
                        thrpt:  [−0.5522% +0.0612% +0.6616%]
                        No change in performance detected.
hop_scaling/256B_1hops  time:   [63.520 ns 63.826 ns 64.139 ns]
                        thrpt:  [3.7172 GiB/s 3.7354 GiB/s 3.7535 GiB/s]
                 change:
                        time:   [+3.7992% +4.8610% +5.9231%] (p = 0.00 < 0.05)
                        thrpt:  [−5.5919% −4.6357% −3.6602%]
                        Performance has regressed.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
hop_scaling/256B_2hops  time:   [121.48 ns 122.66 ns 124.20 ns]
                        thrpt:  [1.9196 GiB/s 1.9438 GiB/s 1.9626 GiB/s]
                 change:
                        time:   [+3.6732% +4.5588% +5.6814%] (p = 0.00 < 0.05)
                        thrpt:  [−5.3760% −4.3600% −3.5430%]
                        Performance has regressed.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
  1 (1.00%) high severe
hop_scaling/256B_3hops  time:   [172.31 ns 174.27 ns 176.95 ns]
                        thrpt:  [1.3474 GiB/s 1.3681 GiB/s 1.3836 GiB/s]
                 change:
                        time:   [+5.2083% +6.5659% +8.1128%] (p = 0.00 < 0.05)
                        thrpt:  [−7.5041% −6.1614% −4.9505%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
hop_scaling/256B_4hops  time:   [232.01 ns 233.61 ns 235.16 ns]
                        thrpt:  [1.0138 GiB/s 1.0206 GiB/s 1.0276 GiB/s]
                 change:
                        time:   [+1.2001% +2.0954% +3.0227%] (p = 0.00 < 0.05)
                        thrpt:  [−2.9340% −2.0524% −1.1858%]
                        Performance has regressed.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
hop_scaling/256B_5hops  time:   [283.48 ns 285.09 ns 286.71 ns]
                        thrpt:  [851.52 MiB/s 856.35 MiB/s 861.24 MiB/s]
                 change:
                        time:   [−1.4505% −0.5215% +0.3804%] (p = 0.25 > 0.05)
                        thrpt:  [−0.3789% +0.5242% +1.4719%]
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) low mild
  2 (2.00%) high mild
hop_scaling/1024B_1hops time:   [50.217 ns 51.031 ns 52.021 ns]
                        thrpt:  [18.333 GiB/s 18.688 GiB/s 18.991 GiB/s]
                 change:
                        time:   [+1.3326% +3.0430% +5.3887%] (p = 0.00 < 0.05)
                        thrpt:  [−5.1132% −2.9531% −1.3151%]
                        Performance has regressed.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
hop_scaling/1024B_2hops time:   [116.79 ns 117.28 ns 117.78 ns]
                        thrpt:  [8.0973 GiB/s 8.1319 GiB/s 8.1657 GiB/s]
                 change:
                        time:   [−0.8270% −0.2570% +0.3357%] (p = 0.39 > 0.05)
                        thrpt:  [−0.3346% +0.2577% +0.8339%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
hop_scaling/1024B_3hops time:   [162.58 ns 163.45 ns 164.35 ns]
                        thrpt:  [5.8028 GiB/s 5.8348 GiB/s 5.8659 GiB/s]
                 change:
                        time:   [+0.6411% +1.6226% +2.5531%] (p = 0.00 < 0.05)
                        thrpt:  [−2.4895% −1.5967% −0.6370%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
hop_scaling/1024B_4hops time:   [219.60 ns 220.52 ns 221.43 ns]
                        thrpt:  [4.3070 GiB/s 4.3247 GiB/s 4.3428 GiB/s]
                 change:
                        time:   [−1.2795% −0.7754% −0.2989%] (p = 0.00 < 0.05)
                        thrpt:  [+0.2998% +0.7815% +1.2961%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
hop_scaling/1024B_5hops time:   [259.81 ns 261.15 ns 262.50 ns]
                        thrpt:  [3.6331 GiB/s 3.6518 GiB/s 3.6707 GiB/s]
                 change:
                        time:   [−1.8526% −1.1491% −0.4748%] (p = 0.00 < 0.05)
                        thrpt:  [+0.4770% +1.1624% +1.8876%]
                        Change within noise threshold.

multihop_with_routing/route_and_forward/1
                        time:   [153.81 ns 154.32 ns 154.83 ns]
                        thrpt:  [6.4588 Melem/s 6.4801 Melem/s 6.5014 Melem/s]
                 change:
                        time:   [−0.5260% +0.0013% +0.5148%] (p = 0.99 > 0.05)
                        thrpt:  [−0.5122% −0.0013% +0.5287%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
multihop_with_routing/route_and_forward/2
                        time:   [295.56 ns 298.07 ns 302.13 ns]
                        thrpt:  [3.3099 Melem/s 3.3549 Melem/s 3.3834 Melem/s]
                 change:
                        time:   [+0.4825% +1.2942% +2.2085%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1608% −1.2777% −0.4802%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
multihop_with_routing/route_and_forward/3
                        time:   [442.21 ns 445.34 ns 449.97 ns]
                        thrpt:  [2.2224 Melem/s 2.2455 Melem/s 2.2613 Melem/s]
                 change:
                        time:   [+0.0759% +0.9781% +2.1413%] (p = 0.05 < 0.05)
                        thrpt:  [−2.0964% −0.9687% −0.0759%]
                        Change within noise threshold.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
multihop_with_routing/route_and_forward/4
                        time:   [606.47 ns 609.33 ns 612.25 ns]
                        thrpt:  [1.6333 Melem/s 1.6412 Melem/s 1.6489 Melem/s]
                 change:
                        time:   [+0.0072% +0.5234% +1.0920%] (p = 0.06 > 0.05)
                        thrpt:  [−1.0802% −0.5207% −0.0072%]
                        No change in performance detected.
multihop_with_routing/route_and_forward/5
                        time:   [752.72 ns 756.10 ns 759.49 ns]
                        thrpt:  [1.3167 Melem/s 1.3226 Melem/s 1.3285 Melem/s]
                 change:
                        time:   [+0.9917% +1.5826% +2.2030%] (p = 0.00 < 0.05)
                        thrpt:  [−2.1555% −1.5579% −0.9820%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

multihop_concurrent/concurrent_forward/4
                        time:   [645.38 µs 656.13 µs 670.17 µs]
                        thrpt:  [5.9686 Melem/s 6.0964 Melem/s 6.1979 Melem/s]
                 change:
                        time:   [+0.0588% +3.4079% +7.4713%] (p = 0.08 > 0.05)
                        thrpt:  [−6.9519% −3.2956% −0.0588%]
                        No change in performance detected.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) high mild
  1 (5.00%) high severe
multihop_concurrent/concurrent_forward/8
                        time:   [1.6255 ms 1.6468 ms 1.6620 ms]
                        thrpt:  [4.8134 Melem/s 4.8578 Melem/s 4.9216 Melem/s]
                 change:
                        time:   [+1.1115% +3.6553% +5.6328%] (p = 0.00 < 0.05)
                        thrpt:  [−5.3324% −3.5264% −1.0992%]
                        Performance has regressed.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
multihop_concurrent/concurrent_forward/16
                        time:   [2.0080 ms 2.0272 ms 2.0511 ms]
                        thrpt:  [7.8008 Melem/s 7.8926 Melem/s 7.9682 Melem/s]
                 change:
                        time:   [+9.3904% +12.877% +17.001%] (p = 0.00 < 0.05)
                        thrpt:  [−14.531% −11.408% −8.5843%]
                        Performance has regressed.
Found 5 outliers among 20 measurements (25.00%)
  3 (15.00%) low mild
  1 (5.00%) high mild
  1 (5.00%) high severe

pingwave/serialize      time:   [812.18 ps 816.83 ps 823.11 ps]
                        thrpt:  [1.2149 Gelem/s 1.2242 Gelem/s 1.2312 Gelem/s]
                 change:
                        time:   [−1.3991% +0.6182% +2.8166%] (p = 0.63 > 0.05)
                        thrpt:  [−2.7394% −0.6144% +1.4190%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
pingwave/deserialize    time:   [972.43 ps 976.47 ps 980.48 ps]
                        thrpt:  [1.0199 Gelem/s 1.0241 Gelem/s 1.0283 Gelem/s]
                 change:
                        time:   [−0.2321% +0.3689% +0.9972%] (p = 0.24 > 0.05)
                        thrpt:  [−0.9874% −0.3675% +0.2326%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
pingwave/roundtrip      time:   [972.20 ps 976.05 ps 979.92 ps]
                        thrpt:  [1.0205 Gelem/s 1.0245 Gelem/s 1.0286 Gelem/s]
                 change:
                        time:   [−1.1644% −0.1968% +0.5693%] (p = 0.70 > 0.05)
                        thrpt:  [−0.5661% +0.1972% +1.1781%]
                        No change in performance detected.
pingwave/forward        time:   [652.48 ps 657.00 ps 663.09 ps]
                        thrpt:  [1.5081 Gelem/s 1.5221 Gelem/s 1.5326 Gelem/s]
                 change:
                        time:   [+0.0001% +0.6644% +1.4492%] (p = 0.09 > 0.05)
                        thrpt:  [−1.4285% −0.6600% −0.0001%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

capabilities/serialize_simple
                        time:   [19.820 ns 19.917 ns 20.017 ns]
                        thrpt:  [49.958 Melem/s 50.208 Melem/s 50.455 Melem/s]
                 change:
                        time:   [−1.2919% −0.6690% −0.0625%] (p = 0.03 < 0.05)
                        thrpt:  [+0.0625% +0.6735% +1.3088%]
                        Change within noise threshold.
capabilities/deserialize_simple
                        time:   [4.9750 ns 5.0161 ns 5.0730 ns]
                        thrpt:  [197.12 Melem/s 199.36 Melem/s 201.01 Melem/s]
                 change:
                        time:   [−0.4521% +0.2395% +0.9571%] (p = 0.50 > 0.05)
                        thrpt:  [−0.9480% −0.2390% +0.4542%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capabilities/serialize_complex
                        time:   [41.893 ns 42.121 ns 42.384 ns]
                        thrpt:  [23.594 Melem/s 23.741 Melem/s 23.870 Melem/s]
                 change:
                        time:   [+0.0660% +0.6846% +1.2949%] (p = 0.03 < 0.05)
                        thrpt:  [−1.2783% −0.6799% −0.0660%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capabilities/deserialize_complex
                        time:   [164.83 ns 165.62 ns 166.41 ns]
                        thrpt:  [6.0093 Melem/s 6.0380 Melem/s 6.0669 Melem/s]
                 change:
                        time:   [−0.2559% +0.2810% +0.8235%] (p = 0.32 > 0.05)
                        thrpt:  [−0.8167% −0.2802% +0.2566%]
                        No change in performance detected.

local_graph/create_pingwave
                        time:   [2.1980 ns 2.2123 ns 2.2287 ns]
                        thrpt:  [448.70 Melem/s 452.01 Melem/s 454.96 Melem/s]
                 change:
                        time:   [−0.8116% +0.4504% +1.4768%] (p = 0.46 > 0.05)
                        thrpt:  [−1.4553% −0.4484% +0.8182%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
local_graph/on_pingwave_new
                        time:   [149.77 ns 153.17 ns 157.07 ns]
                        thrpt:  [6.3667 Melem/s 6.5286 Melem/s 6.6769 Melem/s]
                 change:
                        time:   [−1.4355% +4.1139% +10.095%] (p = 0.15 > 0.05)
                        thrpt:  [−9.1692% −3.9514% +1.4564%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low mild
  1 (1.00%) high mild
local_graph/on_pingwave_duplicate
                        time:   [22.936 ns 23.160 ns 23.482 ns]
                        thrpt:  [42.586 Melem/s 43.178 Melem/s 43.600 Melem/s]
                 change:
                        time:   [−6.9581% −6.1749% −5.1166%] (p = 0.00 < 0.05)
                        thrpt:  [+5.3925% +6.5813% +7.4784%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
local_graph/get_node    time:   [27.742 ns 27.952 ns 28.295 ns]
                        thrpt:  [35.342 Melem/s 35.776 Melem/s 36.046 Melem/s]
                 change:
                        time:   [+0.0462% +0.9019% +1.7921%] (p = 0.04 < 0.05)
                        thrpt:  [−1.7606% −0.8939% −0.0462%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
local_graph/node_count  time:   [208.12 ns 209.67 ns 211.97 ns]
                        thrpt:  [4.7176 Melem/s 4.7695 Melem/s 4.8050 Melem/s]
                 change:
                        time:   [+0.1969% +1.0380% +1.9738%] (p = 0.02 < 0.05)
                        thrpt:  [−1.9356% −1.0273% −0.1965%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
local_graph/stats       time:   [623.61 ns 628.82 ns 636.57 ns]
                        thrpt:  [1.5709 Melem/s 1.5903 Melem/s 1.6036 Melem/s]
                 change:
                        time:   [+0.5041% +1.7942% +3.6231%] (p = 0.01 < 0.05)
                        thrpt:  [−3.4964% −1.7626% −0.5015%]
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

graph_scaling/all_nodes/100
                        time:   [2.3997 µs 2.4210 µs 2.4494 µs]
                        thrpt:  [40.827 Melem/s 41.304 Melem/s 41.671 Melem/s]
                 change:
                        time:   [−0.1728% +1.1548% +2.7742%] (p = 0.14 > 0.05)
                        thrpt:  [−2.6993% −1.1416% +0.1731%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
graph_scaling/nodes_within_hops/100
                        time:   [2.9117 µs 2.9285 µs 2.9457 µs]
                        thrpt:  [33.948 Melem/s 34.147 Melem/s 34.344 Melem/s]
                 change:
                        time:   [−0.3802% +0.3391% +1.0543%] (p = 0.36 > 0.05)
                        thrpt:  [−1.0433% −0.3380% +0.3816%]
                        No change in performance detected.
graph_scaling/all_nodes/500
                        time:   [7.6311 µs 7.6763 µs 7.7213 µs]
                        thrpt:  [64.756 Melem/s 65.135 Melem/s 65.521 Melem/s]
                 change:
                        time:   [−1.0144% −0.2873% +0.4725%] (p = 0.47 > 0.05)
                        thrpt:  [−0.4703% +0.2881% +1.0248%]
                        No change in performance detected.
graph_scaling/nodes_within_hops/500
                        time:   [9.7906 µs 9.8546 µs 9.9489 µs]
                        thrpt:  [50.257 Melem/s 50.738 Melem/s 51.070 Melem/s]
                 change:
                        time:   [−0.3929% +0.4289% +1.3421%] (p = 0.34 > 0.05)
                        thrpt:  [−1.3243% −0.4271% +0.3944%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
graph_scaling/all_nodes/1000
                        time:   [141.61 µs 150.46 µs 159.38 µs]
                        thrpt:  [6.2742 Melem/s 6.6463 Melem/s 7.0614 Melem/s]
                 change:
                        time:   [+14.413% +24.329% +35.631%] (p = 0.00 < 0.05)
                        thrpt:  [−26.270% −19.568% −12.597%]
                        Performance has regressed.
graph_scaling/nodes_within_hops/1000
                        time:   [150.71 µs 159.10 µs 167.27 µs]
                        thrpt:  [5.9782 Melem/s 6.2853 Melem/s 6.6351 Melem/s]
                 change:
                        time:   [+12.833% +23.927% +35.736%] (p = 0.00 < 0.05)
                        thrpt:  [−26.328% −19.307% −11.374%]
                        Performance has regressed.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
graph_scaling/all_nodes/5000
                        time:   [115.72 µs 125.45 µs 135.40 µs]
                        thrpt:  [36.929 Melem/s 39.855 Melem/s 43.207 Melem/s]
                 change:
                        time:   [−37.842% −31.677% −24.903%] (p = 0.00 < 0.05)
                        thrpt:  [+33.162% +46.364% +60.880%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
graph_scaling/nodes_within_hops/5000
                        time:   [166.90 µs 179.69 µs 191.90 µs]
                        thrpt:  [26.056 Melem/s 27.825 Melem/s 29.958 Melem/s]
                 change:
                        time:   [−27.701% −21.788% −15.320%] (p = 0.00 < 0.05)
                        thrpt:  [+18.092% +27.858% +38.315%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

capability_search/find_with_gpu
                        time:   [18.284 µs 18.423 µs 18.589 µs]
                        thrpt:  [53.794 Kelem/s 54.279 Kelem/s 54.693 Kelem/s]
                 change:
                        time:   [−0.3921% +0.2621% +0.9805%] (p = 0.49 > 0.05)
                        thrpt:  [−0.9709% −0.2614% +0.3937%]
                        No change in performance detected.
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_search/find_by_tool_python
                        time:   [33.243 µs 33.361 µs 33.480 µs]
                        thrpt:  [29.869 Kelem/s 29.975 Kelem/s 30.081 Kelem/s]
                 change:
                        time:   [−0.7240% +0.1796% +0.9262%] (p = 0.70 > 0.05)
                        thrpt:  [−0.9177% −0.1792% +0.7293%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_search/find_by_tool_rust
                        time:   [42.337 µs 42.520 µs 42.700 µs]
                        thrpt:  [23.419 Kelem/s 23.518 Kelem/s 23.620 Kelem/s]
                 change:
                        time:   [−0.2322% +0.3875% +1.0068%] (p = 0.23 > 0.05)
                        thrpt:  [−0.9967% −0.3860% +0.2327%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

graph_concurrent/concurrent_pingwave/4
                        time:   [147.25 µs 149.76 µs 152.22 µs]
                        thrpt:  [13.139 Melem/s 13.354 Melem/s 13.583 Melem/s]
                 change:
                        time:   [−2.4491% +1.7205% +6.7116%] (p = 0.49 > 0.05)
                        thrpt:  [−6.2895% −1.6914% +2.5106%]
                        No change in performance detected.
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
graph_concurrent/concurrent_pingwave/8
                        time:   [226.49 µs 228.53 µs 231.08 µs]
                        thrpt:  [17.310 Melem/s 17.503 Melem/s 17.661 Melem/s]
                 change:
                        time:   [−3.5886% −1.4862% +1.0112%] (p = 0.23 > 0.05)
                        thrpt:  [−1.0011% +1.5086% +3.7222%]
                        No change in performance detected.
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  2 (10.00%) high mild
graph_concurrent/concurrent_pingwave/16
                        time:   [404.49 µs 417.55 µs 426.60 µs]
                        thrpt:  [18.753 Melem/s 19.159 Melem/s 19.778 Melem/s]
                 change:
                        time:   [−2.9050% +2.2203% +7.2780%] (p = 0.41 > 0.05)
                        thrpt:  [−6.7843% −2.1721% +2.9920%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

path_finding/path_1_hop time:   [1.6193 µs 1.6254 µs 1.6320 µs]
                        thrpt:  [612.73 Kelem/s 615.22 Kelem/s 617.54 Kelem/s]
                 change:
                        time:   [−0.8801% −0.3183% +0.2413%] (p = 0.26 > 0.05)
                        thrpt:  [−0.2407% +0.3194% +0.8879%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
path_finding/path_2_hops
                        time:   [1.7052 µs 1.7114 µs 1.7177 µs]
                        thrpt:  [582.18 Kelem/s 584.32 Kelem/s 586.44 Kelem/s]
                 change:
                        time:   [−1.7877% −0.5276% +0.4208%] (p = 0.41 > 0.05)
                        thrpt:  [−0.4190% +0.5304% +1.8203%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
path_finding/path_4_hops
                        time:   [1.9501 µs 1.9694 µs 2.0023 µs]
                        thrpt:  [499.42 Kelem/s 507.77 Kelem/s 512.79 Kelem/s]
                 change:
                        time:   [−1.3347% −0.6056% +0.2937%] (p = 0.14 > 0.05)
                        thrpt:  [−0.2928% +0.6093% +1.3527%]
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
path_finding/path_not_found
                        time:   [1.9540 µs 1.9593 µs 1.9644 µs]
                        thrpt:  [509.05 Kelem/s 510.39 Kelem/s 511.77 Kelem/s]
                 change:
                        time:   [−2.3013% −1.5356% −0.8692%] (p = 0.00 < 0.05)
                        thrpt:  [+0.8768% +1.5596% +2.3555%]
                        Change within noise threshold.
path_finding/path_complex_graph
                        time:   [344.47 µs 352.00 µs 361.07 µs]
                        thrpt:  [2.7696 Kelem/s 2.8409 Kelem/s 2.9030 Kelem/s]
                 change:
                        time:   [+3.3359% +5.3340% +7.5311%] (p = 0.00 < 0.05)
                        thrpt:  [−7.0037% −5.0639% −3.2282%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
  3 (3.00%) high severe

failure_detector/heartbeat_existing
                        time:   [31.884 ns 32.413 ns 32.941 ns]
                        thrpt:  [30.357 Melem/s 30.852 Melem/s 31.364 Melem/s]
                 change:
                        time:   [−4.1853% −2.2720% −0.4071%] (p = 0.03 < 0.05)
                        thrpt:  [+0.4088% +2.3248% +4.3682%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
failure_detector/heartbeat_new
                        time:   [307.54 ns 310.87 ns 313.89 ns]
                        thrpt:  [3.1858 Melem/s 3.2167 Melem/s 3.2516 Melem/s]
                 change:
                        time:   [−2.6825% +2.6493% +8.2490%] (p = 0.34 > 0.05)
                        thrpt:  [−7.6204% −2.5809% +2.7564%]
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) low severe
  8 (8.00%) low mild
failure_detector/status_check
                        time:   [13.083 ns 13.278 ns 13.474 ns]
                        thrpt:  [74.215 Melem/s 75.311 Melem/s 76.434 Melem/s]
                 change:
                        time:   [−0.7110% +1.4423% +3.8559%] (p = 0.21 > 0.05)
                        thrpt:  [−3.7127% −1.4218% +0.7161%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 36.6s, or reduce sample count to 10.
failure_detector/check_all
                        time:   [357.58 ms 358.09 ms 358.72 ms]
                        thrpt:  [2.7877  elem/s 2.7926  elem/s 2.7966  elem/s]
                 change:
                        time:   [+0.2926% +0.5037% +0.7211%] (p = 0.00 < 0.05)
                        thrpt:  [−0.7160% −0.5012% −0.2917%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.6s, or reduce sample count to 50.
failure_detector/stats  time:   [86.502 ms 87.183 ms 87.964 ms]
                        thrpt:  [11.368  elem/s 11.470  elem/s 11.560  elem/s]
                 change:
                        time:   [−2.9890% −1.7411% −0.4340%] (p = 0.01 < 0.05)
                        thrpt:  [+0.4359% +1.7720% +3.0811%]
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

loss_simulator/should_drop_1pct
                        time:   [2.1679 ns 2.1761 ns 2.1844 ns]
                        thrpt:  [457.80 Melem/s 459.55 Melem/s 461.28 Melem/s]
                 change:
                        time:   [−0.2787% +0.2932% +0.8858%] (p = 0.33 > 0.05)
                        thrpt:  [−0.8780% −0.2923% +0.2795%]
                        No change in performance detected.
loss_simulator/should_drop_5pct
                        time:   [2.2213 ns 2.2410 ns 2.2719 ns]
                        thrpt:  [440.16 Melem/s 446.22 Melem/s 450.19 Melem/s]
                 change:
                        time:   [−0.1028% +0.6085% +1.3648%] (p = 0.12 > 0.05)
                        thrpt:  [−1.3465% −0.6048% +0.1029%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
loss_simulator/should_drop_10pct
                        time:   [2.3329 ns 2.3430 ns 2.3533 ns]
                        thrpt:  [424.93 Melem/s 426.80 Melem/s 428.64 Melem/s]
                 change:
                        time:   [−0.8025% −0.2368% +0.3184%] (p = 0.42 > 0.05)
                        thrpt:  [−0.3174% +0.2373% +0.8089%]
                        No change in performance detected.
loss_simulator/should_drop_20pct
                        time:   [3.2231 ns 3.2354 ns 3.2479 ns]
                        thrpt:  [307.89 Melem/s 309.08 Melem/s 310.26 Melem/s]
                 change:
                        time:   [−0.8478% −0.3155% +0.2433%] (p = 0.27 > 0.05)
                        thrpt:  [−0.2427% +0.3165% +0.8551%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
loss_simulator/should_drop_burst
                        time:   [2.6386 ns 2.6500 ns 2.6615 ns]
                        thrpt:  [375.73 Melem/s 377.36 Melem/s 378.98 Melem/s]
                 change:
                        time:   [−0.3327% +0.2796% +0.8916%] (p = 0.37 > 0.05)
                        thrpt:  [−0.8838% −0.2788% +0.3338%]
                        No change in performance detected.

circuit_breaker/allow_closed
                        time:   [13.976 ns 14.040 ns 14.106 ns]
                        thrpt:  [70.893 Melem/s 71.226 Melem/s 71.550 Melem/s]
                 change:
                        time:   [−0.2583% +0.3988% +1.0751%] (p = 0.24 > 0.05)
                        thrpt:  [−1.0637% −0.3972% +0.2590%]
                        No change in performance detected.
circuit_breaker/record_success
                        time:   [13.953 ns 14.018 ns 14.085 ns]
                        thrpt:  [71.000 Melem/s 71.336 Melem/s 71.671 Melem/s]
                 change:
                        time:   [−0.4427% +0.1944% +0.8422%] (p = 0.55 > 0.05)
                        thrpt:  [−0.8351% −0.1941% +0.4447%]
                        No change in performance detected.
circuit_breaker/record_failure
                        time:   [13.942 ns 14.014 ns 14.087 ns]
                        thrpt:  [70.989 Melem/s 71.356 Melem/s 71.728 Melem/s]
                 change:
                        time:   [−0.5656% +0.0480% +0.6837%] (p = 0.89 > 0.05)
                        thrpt:  [−0.6791% −0.0480% +0.5688%]
                        No change in performance detected.
circuit_breaker/state   time:   [13.908 ns 13.982 ns 14.057 ns]
                        thrpt:  [71.140 Melem/s 71.522 Melem/s 71.901 Melem/s]
                 change:
                        time:   [−0.5368% +0.0651% +0.6441%] (p = 0.83 > 0.05)
                        thrpt:  [−0.6400% −0.0651% +0.5397%]
                        No change in performance detected.

recovery_manager/on_failure_with_alternates
                        time:   [316.07 ns 321.61 ns 327.95 ns]
                        thrpt:  [3.0493 Melem/s 3.1094 Melem/s 3.1639 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) low mild
  1 (1.00%) high mild
recovery_manager/on_failure_no_alternates
                        time:   [297.62 ns 302.44 ns 307.07 ns]
                        thrpt:  [3.2566 Melem/s 3.3065 Melem/s 3.3599 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) low mild
recovery_manager/get_action
                        time:   [37.813 ns 37.991 ns 38.178 ns]
                        thrpt:  [26.193 Melem/s 26.322 Melem/s 26.446 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
recovery_manager/is_failed
                        time:   [11.099 ns 11.182 ns 11.273 ns]
                        thrpt:  [88.707 Melem/s 89.433 Melem/s 90.098 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
recovery_manager/on_recovery
                        time:   [105.06 ns 105.52 ns 105.97 ns]
                        thrpt:  [9.4368 Melem/s 9.4770 Melem/s 9.5183 Melem/s]
recovery_manager/stats  time:   [730.98 ps 733.78 ps 736.60 ps]
                        thrpt:  [1.3576 Gelem/s 1.3628 Gelem/s 1.3680 Gelem/s]

failure_scaling/check_all/100
                        time:   [4.9252 µs 4.9581 µs 4.9882 µs]
                        thrpt:  [20.047 Melem/s 20.169 Melem/s 20.304 Melem/s]
failure_scaling/healthy_nodes/100
                        time:   [1.7551 µs 1.7634 µs 1.7718 µs]
                        thrpt:  [56.441 Melem/s 56.709 Melem/s 56.976 Melem/s]
failure_scaling/check_all/500
                        time:   [21.362 µs 21.533 µs 21.687 µs]
                        thrpt:  [23.055 Melem/s 23.220 Melem/s 23.406 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  14 (14.00%) low mild
failure_scaling/healthy_nodes/500
                        time:   [6.1384 µs 7.2929 µs 9.1778 µs]
                        thrpt:  [54.479 Melem/s 68.560 Melem/s 81.454 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  3 (3.00%) high mild
  9 (9.00%) high severe
failure_scaling/check_all/1000
                        time:   [41.898 µs 42.246 µs 42.566 µs]
                        thrpt:  [23.493 Melem/s 23.671 Melem/s 23.867 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  12 (12.00%) low mild
failure_scaling/healthy_nodes/1000
                        time:   [10.679 µs 10.715 µs 10.751 µs]
                        thrpt:  [93.015 Melem/s 93.326 Melem/s 93.646 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
failure_scaling/check_all/5000
                        time:   [209.38 µs 210.66 µs 211.73 µs]
                        thrpt:  [23.615 Melem/s 23.735 Melem/s 23.880 Melem/s]
failure_scaling/healthy_nodes/5000
                        time:   [50.998 µs 51.587 µs 52.494 µs]
                        thrpt:  [95.248 Melem/s 96.923 Melem/s 98.043 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

failure_concurrent/concurrent_heartbeat/4
                        time:   [213.14 µs 214.98 µs 217.14 µs]
                        thrpt:  [9.2105 Melem/s 9.3031 Melem/s 9.3834 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
failure_concurrent/concurrent_heartbeat/8
                        time:   [328.00 µs 331.04 µs 334.72 µs]
                        thrpt:  [11.950 Melem/s 12.083 Melem/s 12.195 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) high mild
failure_concurrent/concurrent_heartbeat/16
                        time:   [632.25 µs 639.84 µs 649.66 µs]
                        thrpt:  [12.314 Melem/s 12.503 Melem/s 12.653 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

failure_recovery_cycle/full_cycle
                        time:   [359.39 ns 362.09 ns 364.64 ns]
                        thrpt:  [2.7424 Melem/s 2.7618 Melem/s 2.7825 Melem/s]
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) low severe
  12 (12.00%) low mild

capability_set/create   time:   [538.22 ns 540.39 ns 542.55 ns]
                        thrpt:  [1.8432 Melem/s 1.8505 Melem/s 1.8580 Melem/s]
capability_set/serialize
                        time:   [959.19 ns 962.64 ns 966.19 ns]
                        thrpt:  [1.0350 Melem/s 1.0388 Melem/s 1.0425 Melem/s]
capability_set/deserialize
                        time:   [1.8294 µs 1.8434 µs 1.8574 µs]
                        thrpt:  [538.39 Kelem/s 542.49 Kelem/s 546.64 Kelem/s]
capability_set/roundtrip
                        time:   [2.8359 µs 2.8648 µs 2.9048 µs]
                        thrpt:  [344.26 Kelem/s 349.06 Kelem/s 352.62 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_set/has_tag  time:   [779.23 ps 782.12 ps 785.03 ps]
                        thrpt:  [1.2738 Gelem/s 1.2786 Gelem/s 1.2833 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_set/has_model
                        time:   [972.13 ps 976.58 ps 981.04 ps]
                        thrpt:  [1.0193 Gelem/s 1.0240 Gelem/s 1.0287 Gelem/s]
capability_set/has_tool time:   [778.51 ps 781.72 ps 784.95 ps]
                        thrpt:  [1.2740 Gelem/s 1.2792 Gelem/s 1.2845 Gelem/s]
capability_set/has_gpu  time:   [324.13 ps 325.46 ps 326.81 ps]
                        thrpt:  [3.0599 Gelem/s 3.0726 Gelem/s 3.0852 Gelem/s]

capability_announcement/create
                        time:   [404.77 ns 407.16 ns 409.55 ns]
                        thrpt:  [2.4417 Melem/s 2.4560 Melem/s 2.4706 Melem/s]
capability_announcement/serialize
                        time:   [1.1137 µs 1.1187 µs 1.1238 µs]
                        thrpt:  [889.87 Kelem/s 893.93 Kelem/s 897.91 Kelem/s]
capability_announcement/deserialize
                        time:   [1.9839 µs 1.9950 µs 2.0061 µs]
                        thrpt:  [498.49 Kelem/s 501.25 Kelem/s 504.06 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_announcement/is_expired
                        time:   [26.923 ns 27.053 ns 27.185 ns]
                        thrpt:  [36.785 Melem/s 36.964 Melem/s 37.143 Melem/s]

capability_filter/match_single_tag
                        time:   [10.379 ns 10.474 ns 10.613 ns]
                        thrpt:  [94.223 Melem/s 95.476 Melem/s 96.345 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_filter/match_require_gpu
                        time:   [4.2162 ns 4.2293 ns 4.2424 ns]
                        thrpt:  [235.71 Melem/s 236.45 Melem/s 237.18 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_filter/match_gpu_vendor
                        time:   [3.8911 ns 3.9099 ns 3.9288 ns]
                        thrpt:  [254.53 Melem/s 255.76 Melem/s 257.00 Melem/s]
capability_filter/match_min_memory
                        time:   [3.8974 ns 3.9248 ns 3.9640 ns]
                        thrpt:  [252.27 Melem/s 254.79 Melem/s 256.58 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
capability_filter/match_complex
                        time:   [10.712 ns 10.754 ns 10.796 ns]
                        thrpt:  [92.628 Melem/s 92.990 Melem/s 93.349 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
capability_filter/match_no_match
                        time:   [3.2431 ns 3.2559 ns 3.2688 ns]
                        thrpt:  [305.92 Melem/s 307.14 Melem/s 308.34 Melem/s]

capability_index_insert/index_nodes/100
                        time:   [123.18 µs 124.08 µs 124.98 µs]
                        thrpt:  [800.14 Kelem/s 805.93 Kelem/s 811.82 Kelem/s]
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.8s, enable flat sampling, or reduce sample count to 60.
capability_index_insert/index_nodes/1000
                        time:   [1.3398 ms 1.3514 ms 1.3628 ms]
                        thrpt:  [733.78 Kelem/s 739.97 Kelem/s 746.37 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_index_insert/index_nodes/10000
                        time:   [21.698 ms 22.069 ms 22.435 ms]
                        thrpt:  [445.72 Kelem/s 453.12 Kelem/s 460.87 Kelem/s]

capability_index_query/query_single_tag
                        time:   [248.63 µs 260.79 µs 273.62 µs]
                        thrpt:  [3.6547 Kelem/s 3.8345 Kelem/s 4.0221 Kelem/s]
capability_index_query/query_require_gpu
                        time:   [392.73 µs 408.14 µs 423.78 µs]
                        thrpt:  [2.3597 Kelem/s 2.4501 Kelem/s 2.5463 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_gpu_vendor: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.3s, enable flat sampling, or reduce sample count to 60.
capability_index_query/query_gpu_vendor
                        time:   [1.0103 ms 1.0347 ms 1.0584 ms]
                        thrpt:  [944.83  elem/s 966.42  elem/s 989.77  elem/s]
Benchmarking capability_index_query/query_min_memory: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.2s, enable flat sampling, or reduce sample count to 60.
capability_index_query/query_min_memory
                        time:   [972.45 µs 997.65 µs 1.0230 ms]
                        thrpt:  [977.50  elem/s 1.0024 Kelem/s 1.0283 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
capability_index_query/query_complex
                        time:   [726.15 µs 744.50 µs 763.17 µs]
                        thrpt:  [1.3103 Kelem/s 1.3432 Kelem/s 1.3771 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
capability_index_query/query_model
                        time:   [107.93 µs 109.38 µs 111.27 µs]
                        thrpt:  [8.9873 Kelem/s 9.1425 Kelem/s 9.2651 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking capability_index_query/query_tool: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.2s, enable flat sampling, or reduce sample count to 60.
capability_index_query/query_tool
                        time:   [996.64 µs 1.0299 ms 1.0638 ms]
                        thrpt:  [940.00  elem/s 970.95  elem/s 1.0034 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
capability_index_query/query_no_results
                        time:   [23.682 ns 23.791 ns 23.899 ns]
                        thrpt:  [41.842 Melem/s 42.033 Melem/s 42.225 Melem/s]

capability_index_find_best/find_best_simple
                        time:   [461.22 µs 477.66 µs 493.60 µs]
                        thrpt:  [2.0260 Kelem/s 2.0935 Kelem/s 2.1682 Kelem/s]
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 8.9265 s (10k iterationscapability_index_find_best/find_best_with_prefs
                        time:   [851.08 µs 870.64 µs 890.08 µs]
                        thrpt:  [1.1235 Kelem/s 1.1486 Kelem/s 1.1750 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

capability_index_scaling/query_tag/1000
                        time:   [13.160 µs 13.227 µs 13.293 µs]
                        thrpt:  [75.227 Kelem/s 75.604 Kelem/s 75.987 Kelem/s]
capability_index_scaling/query_complex/1000
                        time:   [42.279 µs 42.524 µs 42.761 µs]
                        thrpt:  [23.386 Kelem/s 23.516 Kelem/s 23.652 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
capability_index_scaling/query_tag/5000
                        time:   [76.409 µs 76.989 µs 77.577 µs]
                        thrpt:  [12.890 Kelem/s 12.989 Kelem/s 13.087 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_index_scaling/query_complex/5000
                        time:   [276.63 µs 287.24 µs 298.23 µs]
                        thrpt:  [3.3531 Kelem/s 3.4814 Kelem/s 3.6149 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
capability_index_scaling/query_tag/10000
                        time:   [238.59 µs 248.77 µs 259.37 µs]
                        thrpt:  [3.8555 Kelem/s 4.0198 Kelem/s 4.1914 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
capability_index_scaling/query_complex/10000
                        time:   [659.10 µs 676.70 µs 694.88 µs]
                        thrpt:  [1.4391 Kelem/s 1.4778 Kelem/s 1.5172 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) low mild
  3 (3.00%) high mild
  1 (1.00%) high severe
capability_index_scaling/query_tag/50000
                        time:   [3.1608 ms 3.2346 ms 3.3107 ms]
                        thrpt:  [302.05  elem/s 309.15  elem/s 316.38  elem/s]
capability_index_scaling/query_complex/50000
                        time:   [4.8113 ms 4.9312 ms 5.0531 ms]
                        thrpt:  [197.90  elem/s 202.79  elem/s 207.85  elem/s]

capability_index_concurrent/concurrent_index/4
                        time:   [434.12 µs 437.42 µs 441.88 µs]
                        thrpt:  [4.5261 Melem/s 4.5723 Melem/s 4.6070 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.3s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/4
                        time:   [465.77 ms 474.96 ms 485.11 ms]
                        thrpt:  [4.1227 Kelem/s 4.2109 Kelem/s 4.2940 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
capability_index_concurrent/concurrent_mixed/4
                        time:   [177.11 ms 181.89 ms 186.46 ms]
                        thrpt:  [10.726 Kelem/s 10.995 Kelem/s 11.293 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
capability_index_concurrent/concurrent_index/8
                        time:   [674.30 µs 686.91 µs 700.00 µs]
                        thrpt:  [5.7143 Melem/s 5.8232 Melem/s 5.9320 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  1 (5.00%) low severe
  2 (10.00%) low mild
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 13.7s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/8
                        time:   [590.63 ms 604.45 ms 618.49 ms]
                        thrpt:  [6.4674 Kelem/s 6.6176 Kelem/s 6.7724 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.4s, or reduce sample count to 10.
capability_index_concurrent/concurrent_mixed/8
                        time:   [272.06 ms 276.20 ms 281.43 ms]
                        thrpt:  [14.213 Kelem/s 14.482 Kelem/s 14.702 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.1216 s (4620 iterationscapability_index_concurrent/concurrent_index/16
                        time:   [1.0764 ms 1.0935 ms 1.1091 ms]
                        thrpt:  [7.2132 Melem/s 7.3163 Melem/s 7.4324 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  3 (15.00%) low mild
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 24.6s, or reduce sample count to 10.
capability_index_concurrent/concurrent_query/16
                        time:   [1.1365 s 1.1437 s 1.1522 s]
                        thrpt:  [6.9434 Kelem/s 6.9951 Kelem/s 7.0391 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 11.8s, or reduce sample count to 10.
capability_index_concurrent/concurrent_mixed/16
                        time:   [583.93 ms 587.17 ms 590.38 ms]
                        thrpt:  [13.551 Kelem/s 13.625 Kelem/s 13.700 Kelem/s]

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.0020 s (5.4M iterationscapability_index_updates/update_higher_version
                        time:   [572.19 ns 574.67 ns 577.08 ns]
                        thrpt:  [1.7328 Melem/s 1.7401 Melem/s 1.7477 Melem/s]
capability_index_updates/update_same_version
                        time:   [573.38 ns 577.77 ns 585.09 ns]
                        thrpt:  [1.7092 Melem/s 1.7308 Melem/s 1.7440 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
capability_index_updates/remove_and_readd
                        time:   [1.4329 µs 1.4443 µs 1.4557 µs]
                        thrpt:  [686.94 Kelem/s 692.37 Kelem/s 697.88 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

diff_op/create_add_tag  time:   [15.168 ns 15.229 ns 15.292 ns]
                        thrpt:  [65.394 Melem/s 65.663 Melem/s 65.927 Melem/s]
diff_op/create_remove_tag
                        time:   [15.544 ns 15.654 ns 15.778 ns]
                        thrpt:  [63.381 Melem/s 63.881 Melem/s 64.335 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
diff_op/create_add_model
                        time:   [52.299 ns 52.474 ns 52.652 ns]
                        thrpt:  [18.993 Melem/s 19.057 Melem/s 19.121 Melem/s]
diff_op/create_update_model
                        time:   [15.685 ns 15.751 ns 15.814 ns]
                        thrpt:  [63.236 Melem/s 63.489 Melem/s 63.756 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
diff_op/estimated_size  time:   [1.9444 ns 1.9513 ns 1.9582 ns]
                        thrpt:  [510.66 Melem/s 512.48 Melem/s 514.29 Melem/s]

capability_diff/create  time:   [89.941 ns 90.341 ns 90.731 ns]
                        thrpt:  [11.022 Melem/s 11.069 Melem/s 11.118 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
capability_diff/serialize
                        time:   [158.57 ns 159.26 ns 159.95 ns]
                        thrpt:  [6.2518 Melem/s 6.2789 Melem/s 6.3062 Melem/s]
capability_diff/deserialize
                        time:   [249.97 ns 251.24 ns 252.51 ns]
                        thrpt:  [3.9602 Melem/s 3.9802 Melem/s 4.0005 Melem/s]
capability_diff/estimated_size
                        time:   [5.5178 ns 5.5389 ns 5.5603 ns]
                        thrpt:  [179.85 Melem/s 180.54 Melem/s 181.23 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

diff_generation/no_changes
                        time:   [360.40 ns 366.38 ns 374.26 ns]
                        thrpt:  [2.6719 Melem/s 2.7294 Melem/s 2.7747 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
diff_generation/add_one_tag
                        time:   [471.92 ns 473.92 ns 475.87 ns]
                        thrpt:  [2.1014 Melem/s 2.1101 Melem/s 2.1190 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
diff_generation/multiple_tag_changes
                        time:   [535.91 ns 537.86 ns 539.78 ns]
                        thrpt:  [1.8526 Melem/s 1.8592 Melem/s 1.8660 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
diff_generation/update_model_loaded
                        time:   [423.02 ns 424.59 ns 426.18 ns]
                        thrpt:  [2.3464 Melem/s 2.3552 Melem/s 2.3639 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  11 (11.00%) high mild
  3 (3.00%) high severe
diff_generation/update_memory
                        time:   [404.85 ns 406.50 ns 408.15 ns]
                        thrpt:  [2.4501 Melem/s 2.4601 Melem/s 2.4700 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  13 (13.00%) high mild
  1 (1.00%) high severe
diff_generation/add_model
                        time:   [511.33 ns 514.45 ns 518.78 ns]
                        thrpt:  [1.9276 Melem/s 1.9438 Melem/s 1.9557 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  10 (10.00%) high mild
  2 (2.00%) high severe
diff_generation/complex_diff
                        time:   [824.25 ns 831.71 ns 843.85 ns]
                        thrpt:  [1.1850 Melem/s 1.2023 Melem/s 1.2132 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe

diff_application/apply_single_op
                        time:   [386.66 ns 389.79 ns 394.86 ns]
                        thrpt:  [2.5326 Melem/s 2.5655 Melem/s 2.5863 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
diff_application/apply_small_diff
                        time:   [390.27 ns 393.32 ns 398.18 ns]
                        thrpt:  [2.5114 Melem/s 2.5425 Melem/s 2.5623 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
diff_application/apply_medium_diff
                        time:   [895.37 ns 902.62 ns 913.89 ns]
                        thrpt:  [1.0942 Melem/s 1.1079 Melem/s 1.1169 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
diff_application/apply_strict_mode
                        time:   [387.40 ns 389.99 ns 393.84 ns]
                        thrpt:  [2.5391 Melem/s 2.5642 Melem/s 2.5813 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

diff_chain_validation/validate_chain_10
                        time:   [6.3452 ns 6.3728 ns 6.4021 ns]
                        thrpt:  [156.20 Melem/s 156.92 Melem/s 157.60 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
diff_chain_validation/validate_chain_100
                        time:   [69.468 ns 69.754 ns 70.041 ns]
                        thrpt:  [14.277 Melem/s 14.336 Melem/s 14.395 Melem/s]

diff_compaction/compact_5_diffs
                        time:   [3.4812 µs 3.4981 µs 3.5155 µs]
                        thrpt:  [284.45 Kelem/s 285.87 Kelem/s 287.25 Kelem/s]
diff_compaction/compact_20_diffs
                        time:   [15.981 µs 16.173 µs 16.422 µs]
                        thrpt:  [60.896 Kelem/s 61.832 Kelem/s 62.575 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

bandwidth_savings/calculate_small
                        time:   [948.43 ns 956.79 ns 971.36 ns]
                        thrpt:  [1.0295 Melem/s 1.0452 Melem/s 1.0544 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
bandwidth_savings/calculate_medium
                        time:   [954.51 ns 961.86 ns 973.85 ns]
                        thrpt:  [1.0268 Melem/s 1.0396 Melem/s 1.0477 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

diff_roundtrip/generate_apply_verify
                        time:   [1.4152 µs 1.4292 µs 1.4522 µs]
                        thrpt:  [688.63 Kelem/s 699.71 Kelem/s 706.62 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

location_info/create    time:   [30.256 ns 30.372 ns 30.487 ns]
                        thrpt:  [32.801 Melem/s 32.925 Melem/s 33.051 Melem/s]
location_info/distance_to
                        time:   [4.4112 ns 4.4289 ns 4.4469 ns]
                        thrpt:  [224.87 Melem/s 225.79 Melem/s 226.70 Melem/s]
location_info/same_continent
                        time:   [7.4615 ns 7.5031 ns 7.5604 ns]
                        thrpt:  [132.27 Melem/s 133.28 Melem/s 134.02 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
location_info/same_continent_cross
                        time:   [325.88 ps 327.32 ps 329.00 ps]
                        thrpt:  [3.0395 Gelem/s 3.0551 Gelem/s 3.0686 Gelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
location_info/same_region
                        time:   [4.2089 ns 4.2229 ns 4.2370 ns]
                        thrpt:  [236.01 Melem/s 236.80 Melem/s 237.59 Melem/s]

topology_hints/create   time:   [4.5483 ns 4.5641 ns 4.5807 ns]
                        thrpt:  [218.31 Melem/s 219.10 Melem/s 219.86 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
topology_hints/connectivity_score
                        time:   [323.50 ps 324.76 ps 326.01 ps]
                        thrpt:  [3.0674 Gelem/s 3.0792 Gelem/s 3.0912 Gelem/s]
topology_hints/average_latency_empty
                        time:   [648.43 ps 653.13 ps 660.72 ps]
                        thrpt:  [1.5135 Gelem/s 1.5311 Gelem/s 1.5422 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
topology_hints/average_latency_100
                        time:   [73.405 ns 73.692 ns 73.978 ns]
                        thrpt:  [13.517 Melem/s 13.570 Melem/s 13.623 Melem/s]

nat_type/difficulty     time:   [323.07 ps 324.48 ps 325.89 ps]
                        thrpt:  [3.0685 Gelem/s 3.0818 Gelem/s 3.0953 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
nat_type/can_connect_direct
                        time:   [323.24 ps 324.53 ps 325.82 ps]
                        thrpt:  [3.0692 Gelem/s 3.0814 Gelem/s 3.0936 Gelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
nat_type/can_connect_symmetric
                        time:   [323.04 ps 324.39 ps 325.73 ps]
                        thrpt:  [3.0700 Gelem/s 3.0827 Gelem/s 3.0956 Gelem/s]

node_metadata/create_simple
                        time:   [46.162 ns 46.623 ns 47.308 ns]
                        thrpt:  [21.138 Melem/s 21.449 Melem/s 21.663 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
node_metadata/create_full
                        time:   [413.45 ns 415.36 ns 417.30 ns]
                        thrpt:  [2.3963 Melem/s 2.4076 Melem/s 2.4186 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
node_metadata/routing_score
                        time:   [3.0841 ns 3.0951 ns 3.1061 ns]
                        thrpt:  [321.95 Melem/s 323.09 Melem/s 324.24 Melem/s]
node_metadata/age       time:   [28.688 ns 28.835 ns 29.035 ns]
                        thrpt:  [34.441 Melem/s 34.680 Melem/s 34.858 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
node_metadata/is_stale  time:   [26.952 ns 27.075 ns 27.199 ns]
                        thrpt:  [36.765 Melem/s 36.934 Melem/s 37.104 Melem/s]
node_metadata/serialize time:   [777.52 ns 780.64 ns 783.84 ns]
                        thrpt:  [1.2758 Melem/s 1.2810 Melem/s 1.2861 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
node_metadata/deserialize
                        time:   [1.5263 µs 1.5344 µs 1.5448 µs]
                        thrpt:  [647.33 Kelem/s 651.71 Kelem/s 655.20 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

metadata_query/match_status
                        time:   [3.5644 ns 3.5767 ns 3.5889 ns]
                        thrpt:  [278.64 Melem/s 279.59 Melem/s 280.56 Melem/s]
metadata_query/match_min_tier
                        time:   [3.5949 ns 3.6059 ns 3.6170 ns]
                        thrpt:  [276.48 Melem/s 277.33 Melem/s 278.17 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_query/match_continent
                        time:   [11.678 ns 11.727 ns 11.776 ns]
                        thrpt:  [84.922 Melem/s 85.274 Melem/s 85.632 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
metadata_query/match_complex
                        time:   [11.050 ns 11.095 ns 11.145 ns]
                        thrpt:  [89.726 Melem/s 90.127 Melem/s 90.500 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
metadata_query/match_no_match
                        time:   [3.5719 ns 3.5880 ns 3.6043 ns]
                        thrpt:  [277.44 Melem/s 278.70 Melem/s 279.96 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

metadata_store_basic/create
                        time:   [805.36 ns 812.99 ns 824.26 ns]
                        thrpt:  [1.2132 Melem/s 1.2300 Melem/s 1.2417 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
metadata_store_basic/upsert_new
                        time:   [2.4034 µs 2.4204 µs 2.4362 µs]
                        thrpt:  [410.48 Kelem/s 413.15 Kelem/s 416.08 Kelem/s]
Found 13 outliers among 100 measurements (13.00%)
  5 (5.00%) low severe
  8 (8.00%) low mild
metadata_store_basic/upsert_existing
                        time:   [1.2481 µs 1.2555 µs 1.2630 µs]
                        thrpt:  [791.76 Kelem/s 796.49 Kelem/s 801.19 Kelem/s]
metadata_store_basic/get
                        time:   [26.013 ns 26.331 ns 26.658 ns]
                        thrpt:  [37.512 Melem/s 37.978 Melem/s 38.443 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
metadata_store_basic/get_miss
                        time:   [26.380 ns 26.693 ns 27.001 ns]
                        thrpt:  [37.036 Melem/s 37.463 Melem/s 37.907 Melem/s]
metadata_store_basic/len
                        time:   [208.83 ns 209.84 ns 210.82 ns]
                        thrpt:  [4.7433 Melem/s 4.7656 Melem/s 4.7885 Melem/s]
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 21.7s, or reduce sample count to 20.
metadata_store_basic/stats
                        time:   [215.65 ms 215.98 ms 216.27 ms]
                        thrpt:  [4.6238  elem/s 4.6302  elem/s 4.6370  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild

metadata_store_query/query_by_status
                        time:   [233.74 µs 237.14 µs 240.48 µs]
                        thrpt:  [4.1583 Kelem/s 4.2169 Kelem/s 4.2782 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
metadata_store_query/query_by_continent
                        time:   [158.86 µs 160.11 µs 161.38 µs]
                        thrpt:  [6.1967 Kelem/s 6.2456 Kelem/s 6.2950 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
metadata_store_query/query_by_tier
                        time:   [419.40 µs 426.14 µs 433.69 µs]
                        thrpt:  [2.3058 Kelem/s 2.3466 Kelem/s 2.3843 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
metadata_store_query/query_accepting_work
                        time:   [461.95 µs 467.34 µs 473.55 µs]
                        thrpt:  [2.1117 Kelem/s 2.1398 Kelem/s 2.1647 Kelem/s]
Found 17 outliers among 100 measurements (17.00%)
  3 (3.00%) high mild
  14 (14.00%) high severe
metadata_store_query/query_with_limit
                        time:   [405.67 µs 412.43 µs 420.00 µs]
                        thrpt:  [2.3810 Kelem/s 2.4247 Kelem/s 2.4651 Kelem/s]
Found 17 outliers among 100 measurements (17.00%)
  4 (4.00%) high mild
  13 (13.00%) high severe
metadata_store_query/query_complex
                        time:   [315.52 µs 319.88 µs 324.25 µs]
                        thrpt:  [3.0840 Kelem/s 3.1262 Kelem/s 3.1693 Kelem/s]

metadata_store_spatial/find_nearby_100km
                        time:   [367.17 µs 370.81 µs 374.36 µs]
                        thrpt:  [2.6712 Kelem/s 2.6968 Kelem/s 2.7236 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
metadata_store_spatial/find_nearby_1000km
                        time:   [447.53 µs 452.09 µs 456.73 µs]
                        thrpt:  [2.1895 Kelem/s 2.2119 Kelem/s 2.2345 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_spatial/find_nearby_5000km
                        time:   [479.09 µs 484.18 µs 489.38 µs]
                        thrpt:  [2.0434 Kelem/s 2.0654 Kelem/s 2.0873 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_spatial/find_best_for_routing
                        time:   [249.17 µs 255.81 µs 263.37 µs]
                        thrpt:  [3.7970 Kelem/s 3.9092 Kelem/s 4.0133 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_spatial/find_relays
                        time:   [529.69 µs 539.07 µs 548.38 µs]
                        thrpt:  [1.8235 Kelem/s 1.8550 Kelem/s 1.8879 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

metadata_store_scaling/query_status/1000
                        time:   [19.743 µs 19.865 µs 19.988 µs]
                        thrpt:  [50.031 Kelem/s 50.340 Kelem/s 50.652 Kelem/s]
metadata_store_scaling/query_complex/1000
                        time:   [21.643 µs 21.763 µs 21.887 µs]
                        thrpt:  [45.690 Kelem/s 45.949 Kelem/s 46.205 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
metadata_store_scaling/find_nearby/1000
                        time:   [57.385 µs 57.643 µs 57.903 µs]
                        thrpt:  [17.270 Kelem/s 17.348 Kelem/s 17.426 Kelem/s]
metadata_store_scaling/query_status/5000
                        time:   [106.63 µs 107.63 µs 108.63 µs]
                        thrpt:  [9.2059 Kelem/s 9.2915 Kelem/s 9.3779 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
metadata_store_scaling/query_complex/5000
                        time:   [127.09 µs 128.14 µs 129.17 µs]
                        thrpt:  [7.7415 Kelem/s 7.8042 Kelem/s 7.8685 Kelem/s]
metadata_store_scaling/find_nearby/5000
                        time:   [287.60 µs 291.37 µs 296.90 µs]
                        thrpt:  [3.3682 Kelem/s 3.4321 Kelem/s 3.4770 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
metadata_store_scaling/query_status/10000
                        time:   [233.76 µs 237.35 µs 241.05 µs]
                        thrpt:  [4.1484 Kelem/s 4.2131 Kelem/s 4.2779 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
metadata_store_scaling/query_complex/10000
                        time:   [284.85 µs 289.30 µs 293.84 µs]
                        thrpt:  [3.4032 Kelem/s 3.4566 Kelem/s 3.5106 Kelem/s]
metadata_store_scaling/find_nearby/10000
                        time:   [587.30 µs 592.72 µs 598.27 µs]
                        thrpt:  [1.6715 Kelem/s 1.6871 Kelem/s 1.7027 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
metadata_store_scaling/query_status/50000
                        time:   [2.6681 ms 2.7401 ms 2.8101 ms]
                        thrpt:  [355.86  elem/s 364.95  elem/s 374.79  elem/s]
metadata_store_scaling/query_complex/50000
                        time:   [3.1275 ms 3.1969 ms 3.2697 ms]
                        thrpt:  [305.83  elem/s 312.80  elem/s 319.75  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
metadata_store_scaling/find_nearby/50000
                        time:   [3.5514 ms 3.6045 ms 3.6570 ms]
                        thrpt:  [273.45  elem/s 277.43  elem/s 281.58  elem/s]

metadata_store_concurrent/concurrent_upsert/4
                        time:   [2.1739 ms 2.2255 ms 2.2842 ms]
                        thrpt:  [875.58 Kelem/s 898.68 Kelem/s 920.01 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.9s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/4
                        time:   [376.94 ms 385.73 ms 394.08 ms]
                        thrpt:  [5.0751 Kelem/s 5.1849 Kelem/s 5.3058 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.3s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/4
                        time:   [437.28 ms 463.13 ms 489.99 ms]
                        thrpt:  [4.0817 Kelem/s 4.3184 Kelem/s 4.5737 Kelem/s]
metadata_store_concurrent/concurrent_upsert/8
                        time:   [3.8422 ms 3.8934 ms 3.9590 ms]
                        thrpt:  [1.0104 Melem/s 1.0274 Melem/s 1.0411 Melem/s]
Found 4 outliers among 20 measurements (20.00%)
  1 (5.00%) low mild
  3 (15.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.4s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/8
                        time:   [800.47 ms 811.56 ms 822.26 ms]
                        thrpt:  [4.8646 Kelem/s 4.9288 Kelem/s 4.9971 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 19.4s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/8
                        time:   [961.64 ms 971.74 ms 981.40 ms]
                        thrpt:  [4.0758 Kelem/s 4.1163 Kelem/s 4.1596 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
metadata_store_concurrent/concurrent_upsert/16
                        time:   [8.3780 ms 8.4853 ms 8.5933 ms]
                        thrpt:  [930.96 Kelem/s 942.81 Kelem/s 954.88 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 30.8s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_query/16
                        time:   [1.5151 s 1.5322 s 1.5508 s]
                        thrpt:  [5.1585 Kelem/s 5.2214 Kelem/s 5.2802 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 39.4s, or reduce sample count to 10.
metadata_store_concurrent/concurrent_mixed/16
                        time:   [1.9820 s 1.9947 s 2.0082 s]
                        thrpt:  [3.9836 Kelem/s 4.0107 Kelem/s 4.0364 Kelem/s]

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimated 5.0021 s (9.3M iteratmetadata_store_versioning/update_versioned_success
                        time:   [196.23 ns 197.50 ns 198.83 ns]
                        thrpt:  [5.0294 Melem/s 5.0634 Melem/s 5.0960 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimated 5.0003 s (23M iteratmetadata_store_versioning/update_versioned_conflict
                        time:   [196.37 ns 198.16 ns 200.72 ns]
                        thrpt:  [4.9820 Melem/s 5.0463 Melem/s 5.0924 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

schema_validation/validate_string
                        time:   [3.5739 ns 3.5881 ns 3.6023 ns]
                        thrpt:  [277.60 Melem/s 278.70 Melem/s 279.81 Melem/s]
schema_validation/validate_integer
                        time:   [3.5768 ns 3.5923 ns 3.6077 ns]
                        thrpt:  [277.18 Melem/s 278.37 Melem/s 279.58 Melem/s]
schema_validation/validate_object
                        time:   [76.136 ns 76.470 ns 76.807 ns]
                        thrpt:  [13.020 Melem/s 13.077 Melem/s 13.134 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
schema_validation/validate_array_10
                        time:   [37.818 ns 38.236 ns 38.839 ns]
                        thrpt:  [25.748 Melem/s 26.153 Melem/s 26.442 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
schema_validation/validate_complex
                        time:   [267.08 ns 268.45 ns 269.92 ns]
                        thrpt:  [3.7048 Melem/s 3.7250 Melem/s 3.7442 Melem/s]

endpoint_matching/match_success
                        time:   [178.54 ns 179.16 ns 179.79 ns]
                        thrpt:  [5.5620 Melem/s 5.5815 Melem/s 5.6011 Melem/s]
endpoint_matching/match_failure
                        time:   [180.14 ns 181.67 ns 184.21 ns]
                        thrpt:  [5.4285 Melem/s 5.5046 Melem/s 5.5513 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
endpoint_matching/match_multi_param
                        time:   [391.62 ns 393.22 ns 394.82 ns]
                        thrpt:  [2.5328 Melem/s 2.5431 Melem/s 2.5535 Melem/s]

api_version/is_compatible_with
                        time:   [324.13 ps 325.36 ps 326.61 ps]
                        thrpt:  [3.0618 Gelem/s 3.0735 Gelem/s 3.0851 Gelem/s]
api_version/parse       time:   [35.604 ns 35.746 ns 35.889 ns]
                        thrpt:  [27.864 Melem/s 27.975 Melem/s 28.086 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
api_version/to_string   time:   [43.656 ns 43.820 ns 43.990 ns]
                        thrpt:  [22.732 Melem/s 22.821 Melem/s 22.907 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

api_schema/create       time:   [1.2822 µs 1.2900 µs 1.2988 µs]
                        thrpt:  [769.97 Kelem/s 775.16 Kelem/s 779.92 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
api_schema/serialize    time:   [1.9750 µs 1.9850 µs 1.9949 µs]
                        thrpt:  [501.27 Kelem/s 503.77 Kelem/s 506.32 Kelem/s]
api_schema/deserialize  time:   [5.1949 µs 5.2187 µs 5.2432 µs]
                        thrpt:  [190.72 Kelem/s 191.62 Kelem/s 192.50 Kelem/s]
api_schema/find_endpoint
                        time:   [210.66 ns 211.22 ns 211.77 ns]
                        thrpt:  [4.7220 Melem/s 4.7345 Melem/s 4.7471 Melem/s]
api_schema/endpoints_by_tag
                        time:   [60.791 ns 61.084 ns 61.381 ns]
                        thrpt:  [16.292 Melem/s 16.371 Melem/s 16.450 Melem/s]

request_validation/validate_full_request
                        time:   [72.858 ns 73.350 ns 74.018 ns]
                        thrpt:  [13.510 Melem/s 13.633 Melem/s 13.725 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
request_validation/validate_path_only
                        time:   [22.327 ns 22.434 ns 22.538 ns]
                        thrpt:  [44.370 Melem/s 44.576 Melem/s 44.790 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

api_registry_basic/create
                        time:   [438.28 ns 440.74 ns 443.18 ns]
                        thrpt:  [2.2564 Melem/s 2.2689 Melem/s 2.2816 Melem/s]
api_registry_basic/register_new
                        time:   [4.7006 µs 4.7250 µs 4.7532 µs]
                        thrpt:  [210.39 Kelem/s 211.64 Kelem/s 212.74 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) low severe
  5 (5.00%) low mild
  3 (3.00%) high severe
api_registry_basic/get  time:   [26.174 ns 26.475 ns 26.768 ns]
                        thrpt:  [37.357 Melem/s 37.771 Melem/s 38.206 Melem/s]
api_registry_basic/len  time:   [208.65 ns 209.45 ns 210.24 ns]
                        thrpt:  [4.7565 Melem/s 4.7743 Melem/s 4.7928 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 23.4s, or reduce sample count to 20.
api_registry_basic/stats
                        time:   [232.39 ms 232.75 ms 233.20 ms]
                        thrpt:  [4.2882  elem/s 4.2965  elem/s 4.3030  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

api_registry_query/query_by_name
                        time:   [113.34 µs 114.41 µs 115.46 µs]
                        thrpt:  [8.6608 Kelem/s 8.7403 Kelem/s 8.8228 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
api_registry_query/query_by_tag
                        time:   [744.30 µs 763.51 µs 782.49 µs]
                        thrpt:  [1.2780 Kelem/s 1.3097 Kelem/s 1.3435 Kelem/s]
api_registry_query/query_with_version
                        time:   [63.168 µs 63.511 µs 63.857 µs]
                        thrpt:  [15.660 Kelem/s 15.745 Kelem/s 15.831 Kelem/s]
api_registry_query/find_by_endpoint
                        time:   [3.6468 ms 3.7523 ms 3.8583 ms]
                        thrpt:  [259.18  elem/s 266.51  elem/s 274.21  elem/s]
api_registry_query/find_compatible
                        time:   [68.786 µs 69.192 µs 69.597 µs]
                        thrpt:  [14.369 Kelem/s 14.452 Kelem/s 14.538 Kelem/s]

api_registry_scaling/query_by_name/1000
                        time:   [8.1402 µs 8.2251 µs 8.3493 µs]
                        thrpt:  [119.77 Kelem/s 121.58 Kelem/s 122.85 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
api_registry_scaling/query_by_tag/1000
                        time:   [52.573 µs 52.887 µs 53.202 µs]
                        thrpt:  [18.796 Kelem/s 18.908 Kelem/s 19.021 Kelem/s]
api_registry_scaling/query_by_name/5000
                        time:   [50.405 µs 50.748 µs 51.090 µs]
                        thrpt:  [19.573 Kelem/s 19.705 Kelem/s 19.839 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
api_registry_scaling/query_by_tag/5000
                        time:   [294.89 µs 299.49 µs 304.62 µs]
                        thrpt:  [3.2828 Kelem/s 3.3390 Kelem/s 3.3911 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
api_registry_scaling/query_by_name/10000
                        time:   [112.92 µs 114.00 µs 115.08 µs]
                        thrpt:  [8.6893 Kelem/s 8.7717 Kelem/s 8.8558 Kelem/s]
api_registry_scaling/query_by_tag/10000
                        time:   [760.60 µs 781.63 µs 802.14 µs]
                        thrpt:  [1.2467 Kelem/s 1.2794 Kelem/s 1.3147 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 18.3s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/4
                        time:   [917.79 ms 926.42 ms 934.90 ms]
                        thrpt:  [2.1393 Kelem/s 2.1589 Kelem/s 2.1791 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  1 (5.00%) low mild
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 12.3s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/4
                        time:   [631.56 ms 644.06 ms 656.07 ms]
                        thrpt:  [3.0484 Kelem/s 3.1053 Kelem/s 3.1667 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 25.0s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/8
                        time:   [1.2635 s 1.2818 s 1.3026 s]
                        thrpt:  [3.0708 Kelem/s 3.1206 Kelem/s 3.1659 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 21.3s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/8
                        time:   [1.0357 s 1.0438 s 1.0516 s]
                        thrpt:  [3.8036 Kelem/s 3.8322 Kelem/s 3.8621 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 49.4s, or reduce sample count to 10.
api_registry_concurrent/concurrent_query/16
                        time:   [2.5370 s 2.5922 s 2.6459 s]
                        thrpt:  [3.0236 Kelem/s 3.0861 Kelem/s 3.1533 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 43.4s, or reduce sample count to 10.
api_registry_concurrent/concurrent_mixed/16
                        time:   [2.1395 s 2.1540 s 2.1695 s]
                        thrpt:  [3.6875 Kelem/s 3.7139 Kelem/s 3.7391 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

compare_op/eq           time:   [2.0268 ns 2.0344 ns 2.0418 ns]
                        thrpt:  [489.76 Melem/s 491.55 Melem/s 493.39 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
compare_op/gt           time:   [1.2971 ns 1.3061 ns 1.3195 ns]
                        thrpt:  [757.87 Melem/s 765.63 Melem/s 770.94 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
compare_op/contains_string
                        time:   [27.006 ns 27.124 ns 27.240 ns]
                        thrpt:  [36.711 Melem/s 36.868 Melem/s 37.029 Melem/s]
compare_op/in_array     time:   [7.1257 ns 7.1535 ns 7.1810 ns]
                        thrpt:  [139.26 Melem/s 139.79 Melem/s 140.34 Melem/s]

condition/simple        time:   [53.913 ns 54.409 ns 55.196 ns]
                        thrpt:  [18.117 Melem/s 18.379 Melem/s 18.549 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
condition/nested_field  time:   [594.01 ns 596.44 ns 598.87 ns]
                        thrpt:  [1.6698 Melem/s 1.6766 Melem/s 1.6835 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
condition/string_eq     time:   [73.896 ns 74.187 ns 74.481 ns]
                        thrpt:  [13.426 Melem/s 13.479 Melem/s 13.533 Melem/s]

condition_expr/single   time:   [54.501 ns 54.814 ns 55.164 ns]
                        thrpt:  [18.128 Melem/s 18.243 Melem/s 18.348 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
condition_expr/and_2    time:   [109.19 ns 110.25 ns 111.83 ns]
                        thrpt:  [8.9418 Melem/s 9.0704 Melem/s 9.1586 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
condition_expr/and_5    time:   [323.06 ns 324.69 ns 326.36 ns]
                        thrpt:  [3.0641 Melem/s 3.0798 Melem/s 3.0954 Melem/s]
condition_expr/or_3     time:   [181.79 ns 182.75 ns 183.74 ns]
                        thrpt:  [5.4425 Melem/s 5.4719 Melem/s 5.5009 Melem/s]
condition_expr/nested   time:   [129.91 ns 130.85 ns 132.09 ns]
                        thrpt:  [7.5708 Melem/s 7.6425 Melem/s 7.6978 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

rule/create             time:   [294.87 ns 296.83 ns 298.82 ns]
                        thrpt:  [3.3465 Melem/s 3.3690 Melem/s 3.3914 Melem/s]
rule/matches            time:   [108.75 ns 109.18 ns 109.61 ns]
                        thrpt:  [9.1233 Melem/s 9.1594 Melem/s 9.1956 Melem/s]

rule_context/create     time:   [872.98 ns 877.18 ns 881.98 ns]
                        thrpt:  [1.1338 Melem/s 1.1400 Melem/s 1.1455 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
rule_context/get_simple time:   [52.770 ns 53.056 ns 53.337 ns]
                        thrpt:  [18.749 Melem/s 18.848 Melem/s 18.950 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_context/get_nested time:   [594.52 ns 597.00 ns 599.49 ns]
                        thrpt:  [1.6681 Melem/s 1.6750 Melem/s 1.6820 Melem/s]
rule_context/get_deep_nested
                        time:   [602.59 ns 605.11 ns 607.62 ns]
                        thrpt:  [1.6458 Melem/s 1.6526 Melem/s 1.6595 Melem/s]

rule_engine_basic/create
                        time:   [8.5473 ns 8.5815 ns 8.6167 ns]
                        thrpt:  [116.05 Melem/s 116.53 Melem/s 117.00 Melem/s]
rule_engine_basic/add_rule
                        time:   [2.7950 µs 2.9558 µs 3.0936 µs]
                        thrpt:  [323.25 Kelem/s 338.32 Kelem/s 357.78 Kelem/s]
rule_engine_basic/get_rule
                        time:   [18.230 ns 18.601 ns 18.993 ns]
                        thrpt:  [52.650 Melem/s 53.761 Melem/s 54.854 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
rule_engine_basic/rules_by_tag
                        time:   [1.1671 µs 1.1713 µs 1.1756 µs]
                        thrpt:  [850.60 Kelem/s 853.74 Kelem/s 856.82 Kelem/s]
rule_engine_basic/stats time:   [7.2751 µs 7.3063 µs 7.3369 µs]
                        thrpt:  [136.30 Kelem/s 136.87 Kelem/s 137.45 Kelem/s]

rule_engine_evaluate/evaluate_10_rules
                        time:   [2.3754 µs 2.3846 µs 2.3939 µs]
                        thrpt:  [417.73 Kelem/s 419.35 Kelem/s 420.98 Kelem/s]
rule_engine_evaluate/evaluate_first_10_rules
                        time:   [243.07 ns 244.41 ns 245.73 ns]
                        thrpt:  [4.0696 Melem/s 4.0914 Melem/s 4.1141 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_evaluate/evaluate_100_rules
                        time:   [23.477 µs 23.612 µs 23.748 µs]
                        thrpt:  [42.109 Kelem/s 42.351 Kelem/s 42.595 Kelem/s]
rule_engine_evaluate/evaluate_first_100_rules
                        time:   [241.78 ns 243.33 ns 244.90 ns]
                        thrpt:  [4.0833 Melem/s 4.1097 Melem/s 4.1360 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated 5.1147 s (212k iteratiorule_engine_evaluate/evaluate_matching_100_rules
                        time:   [23.910 µs 24.028 µs 24.145 µs]
                        thrpt:  [41.417 Kelem/s 41.619 Kelem/s 41.823 Kelem/s]
rule_engine_evaluate/evaluate_1000_rules
                        time:   [233.84 µs 235.10 µs 236.35 µs]
                        thrpt:  [4.2310 Kelem/s 4.2536 Kelem/s 4.2764 Kelem/s]
rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [243.22 ns 244.41 ns 245.58 ns]
                        thrpt:  [4.0719 Melem/s 4.0916 Melem/s 4.1115 Melem/s]

rule_engine_scaling/evaluate/10
                        time:   [2.3773 µs 2.3898 µs 2.4025 µs]
                        thrpt:  [416.24 Kelem/s 418.45 Kelem/s 420.64 Kelem/s]
rule_engine_scaling/evaluate_first/10
                        time:   [241.08 ns 242.57 ns 244.07 ns]
                        thrpt:  [4.0973 Melem/s 4.1225 Melem/s 4.1481 Melem/s]
rule_engine_scaling/evaluate/50
                        time:   [11.932 µs 12.057 µs 12.229 µs]
                        thrpt:  [81.773 Kelem/s 82.938 Kelem/s 83.811 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
rule_engine_scaling/evaluate_first/50
                        time:   [246.16 ns 248.97 ns 253.64 ns]
                        thrpt:  [3.9426 Melem/s 4.0166 Melem/s 4.0624 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate/100
                        time:   [23.711 µs 24.100 µs 24.613 µs]
                        thrpt:  [40.629 Kelem/s 41.493 Kelem/s 42.174 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
rule_engine_scaling/evaluate_first/100
                        time:   [245.42 ns 246.87 ns 248.43 ns]
                        thrpt:  [4.0252 Melem/s 4.0508 Melem/s 4.0746 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate/500
                        time:   [117.79 µs 118.85 µs 120.23 µs]
                        thrpt:  [8.3177 Kelem/s 8.4142 Kelem/s 8.4895 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate_first/500
                        time:   [248.01 ns 248.88 ns 249.75 ns]
                        thrpt:  [4.0041 Melem/s 4.0180 Melem/s 4.0320 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
rule_engine_scaling/evaluate/1000
                        time:   [232.16 µs 233.39 µs 234.65 µs]
                        thrpt:  [4.2616 Kelem/s 4.2846 Kelem/s 4.3074 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_scaling/evaluate_first/1000
                        time:   [241.80 ns 244.22 ns 247.70 ns]
                        thrpt:  [4.0371 Melem/s 4.0946 Melem/s 4.1356 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe

rule_set/create         time:   [2.9645 µs 2.9953 µs 3.0430 µs]
                        thrpt:  [328.62 Kelem/s 333.86 Kelem/s 337.32 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
rule_set/load_into_engine
                        time:   [5.8260 µs 5.8747 µs 5.9385 µs]
                        thrpt:  [168.39 Kelem/s 170.22 Kelem/s 171.65 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe

trace_id/generate       time:   [597.41 ns 605.44 ns 614.16 ns]
                        thrpt:  [1.6282 Melem/s 1.6517 Melem/s 1.6739 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
trace_id/to_hex         time:   [81.916 ns 82.229 ns 82.541 ns]
                        thrpt:  [12.115 Melem/s 12.161 Melem/s 12.208 Melem/s]
trace_id/from_hex       time:   [24.139 ns 24.231 ns 24.324 ns]
                        thrpt:  [41.112 Melem/s 41.270 Melem/s 41.427 Melem/s]

context_operations/create
                        time:   [911.45 ns 925.76 ns 941.85 ns]
                        thrpt:  [1.0617 Melem/s 1.0802 Melem/s 1.0972 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
context_operations/child
                        time:   [311.77 ns 316.45 ns 321.79 ns]
                        thrpt:  [3.1077 Melem/s 3.1601 Melem/s 3.2075 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
context_operations/for_remote
                        time:   [312.16 ns 315.35 ns 318.59 ns]
                        thrpt:  [3.1388 Melem/s 3.1711 Melem/s 3.2035 Melem/s]
context_operations/to_traceparent
                        time:   [237.27 ns 238.09 ns 238.92 ns]
                        thrpt:  [4.1855 Melem/s 4.2001 Melem/s 4.2146 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
context_operations/from_traceparent
                        time:   [407.46 ns 411.45 ns 415.43 ns]
                        thrpt:  [2.4071 Melem/s 2.4305 Melem/s 2.4543 Melem/s]

baggage/create          time:   [2.1332 ns 2.1472 ns 2.1660 ns]
                        thrpt:  [461.68 Melem/s 465.72 Melem/s 468.79 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
baggage/get             time:   [19.137 ns 19.492 ns 19.868 ns]
                        thrpt:  [50.332 Melem/s 51.303 Melem/s 52.254 Melem/s]
baggage/set             time:   [51.706 ns 51.905 ns 52.100 ns]
                        thrpt:  [19.194 Melem/s 19.266 Melem/s 19.340 Melem/s]
baggage/merge           time:   [910.38 ns 914.01 ns 917.55 ns]
                        thrpt:  [1.0899 Melem/s 1.0941 Melem/s 1.0984 Melem/s]

span/create             time:   [362.27 ns 366.40 ns 370.54 ns]
                        thrpt:  [2.6988 Melem/s 2.7292 Melem/s 2.7603 Melem/s]
span/set_attribute      time:   [48.229 ns 48.639 ns 49.326 ns]
                        thrpt:  [20.273 Melem/s 20.560 Melem/s 20.734 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
span/add_event          time:   [53.046 ns 53.770 ns 54.428 ns]
                        thrpt:  [18.373 Melem/s 18.598 Melem/s 18.851 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low mild
  1 (1.00%) high severe
span/with_kind          time:   [362.12 ns 366.21 ns 370.40 ns]
                        thrpt:  [2.6998 Melem/s 2.7307 Melem/s 2.7615 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

context_store/create_context
                        time:   [101.77 µs 102.61 µs 103.70 µs]
                        thrpt:  [9.6436 Kelem/s 9.7458 Kelem/s 9.8262 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
