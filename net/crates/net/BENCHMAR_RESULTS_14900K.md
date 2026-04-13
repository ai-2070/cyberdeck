     Running benches\bltp.rs (target\release\deps\bltp-ea728accfc4b0932.exe)
Gnuplot not found, using plotters backend
bltp_header/serialize   time:   [1.1926 ns 1.2450 ns 1.2948 ns]
                        thrpt:  [772.31 Melem/s 803.19 Melem/s 838.50 Melem/s]
                 change:
                        time:   [−4.2855% −0.2470% +4.3103%] (p = 0.91 > 0.05)
                        thrpt:  [−4.1322% +0.2476% +4.4774%]
                        No change in performance detected.
Benchmarking bltp_header/deserialize: Collecting 100 samples in estimated 5.0000 s (4.0B iterationsbltp_header/deserialize time:   [1.2419 ns 1.2502 ns 1.2597 ns]
                        thrpt:  [793.82 Melem/s 799.86 Melem/s 805.22 Melem/s]
                 change:
                        time:   [−6.1873% −4.5464% −2.9255%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0136% +4.7629% +6.5954%]
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
bltp_header/roundtrip   time:   [1.2364 ns 1.2432 ns 1.2510 ns]
                        thrpt:  [799.38 Melem/s 804.37 Melem/s 808.81 Melem/s]
                 change:
                        time:   [−1.6759% −0.7391% +0.1352%] (p = 0.13 > 0.05)
                        thrpt:  [−0.1350% +0.7446% +1.7045%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking bltp_event_frame/write_single/64: Collecting 100 samples in estimated 5.0002 s (137M ibltp_event_frame/write_single/64
                        time:   [36.441 ns 36.658 ns 36.878 ns]
                        thrpt:  [1.6163 GiB/s 1.6260 GiB/s 1.6356 GiB/s]
                 change:
                        time:   [−3.7639% −2.7974% −1.8440%] (p = 0.00 < 0.05)
                        thrpt:  [+1.8787% +2.8779% +3.9111%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking bltp_event_frame/write_single/256: Collecting 100 samples in estimated 5.0001 s (135M bltp_event_frame/write_single/256
                        time:   [36.619 ns 36.810 ns 37.017 ns]
                        thrpt:  [6.4408 GiB/s 6.4769 GiB/s 6.5109 GiB/s]
                 change:
                        time:   [−0.4148% +0.6536% +1.7773%] (p = 0.24 > 0.05)
                        thrpt:  [−1.7463% −0.6493% +0.4165%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking bltp_event_frame/write_single/1024: Collecting 100 samples in estimated 5.0001 s (133Mbltp_event_frame/write_single/1024
                        time:   [36.967 ns 37.230 ns 37.513 ns]
                        thrpt:  [25.422 GiB/s 25.616 GiB/s 25.798 GiB/s]
                 change:
                        time:   [−1.2545% −0.0178% +1.1893%] (p = 0.97 > 0.05)
                        thrpt:  [−1.1753% +0.0178% +1.2704%]
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking bltp_event_frame/write_single/4096: Collecting 100 samples in estimated 5.0002 s (87M bltp_event_frame/write_single/4096
                        time:   [55.778 ns 56.200 ns 56.661 ns]
                        thrpt:  [67.325 GiB/s 67.878 GiB/s 68.390 GiB/s]
                 change:
                        time:   [+2.3984% +3.7755% +5.0939%] (p = 0.00 < 0.05)
                        thrpt:  [−4.8470% −3.6381% −2.3422%]
                        Performance has regressed.
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking bltp_event_frame/write_batch/1: Collecting 100 samples in estimated 5.0000 s (175M itebltp_event_frame/write_batch/1
                        time:   [28.278 ns 28.567 ns 28.888 ns]
                        thrpt:  [2.0633 GiB/s 2.0865 GiB/s 2.1078 GiB/s]
                 change:
                        time:   [−1.7254% −0.5005% +0.6477%] (p = 0.42 > 0.05)
                        thrpt:  [−0.6435% +0.5031% +1.7557%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking bltp_event_frame/write_batch/10: Collecting 100 samples in estimated 5.0002 s (86M itebltp_event_frame/write_batch/10
                        time:   [57.730 ns 58.107 ns 58.548 ns]
                        thrpt:  [10.180 GiB/s 10.258 GiB/s 10.325 GiB/s]
                 change:
                        time:   [+1.2141% +2.9998% +5.6943%] (p = 0.00 < 0.05)
                        thrpt:  [−5.3875% −2.9124% −1.1995%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking bltp_event_frame/write_batch/50: Collecting 100 samples in estimated 5.0006 s (33M itebltp_event_frame/write_batch/50
                        time:   [148.73 ns 149.63 ns 150.58 ns]
                        thrpt:  [19.791 GiB/s 19.918 GiB/s 20.037 GiB/s]
                 change:
                        time:   [+3.5889% +5.3246% +7.0840%] (p = 0.00 < 0.05)
                        thrpt:  [−6.6153% −5.0554% −3.4646%]
                        Performance has regressed.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking bltp_event_frame/write_batch/100: Collecting 100 samples in estimated 5.0007 s (18M itbltp_event_frame/write_batch/100
                        time:   [280.49 ns 281.87 ns 283.38 ns]
                        thrpt:  [21.033 GiB/s 21.146 GiB/s 21.250 GiB/s]
                 change:
                        time:   [+3.7285% +4.9744% +6.2458%] (p = 0.00 < 0.05)
                        thrpt:  [−5.8787% −4.7387% −3.5945%]
                        Performance has regressed.
Found 10 outliers among 100 measurements (10.00%)
  3 (3.00%) high mild
  7 (7.00%) high severe
Benchmarking bltp_event_frame/read_batch_10: Collecting 100 samples in estimated 5.0008 s (29M iterbltp_event_frame/read_batch_10
                        time:   [168.63 ns 169.60 ns 170.63 ns]
                        thrpt:  [58.607 Melem/s 58.961 Melem/s 59.300 Melem/s]
                 change:
                        time:   [+0.9685% +2.2348% +3.7479%] (p = 0.00 < 0.05)
                        thrpt:  [−3.6125% −2.1859% −0.9592%]
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking bltp_packet_pool/get_return/16: Collecting 100 samples in estimated 5.0001 s (152M itebltp_packet_pool/get_return/16
                        time:   [33.093 ns 33.471 ns 33.895 ns]
                        thrpt:  [29.503 Melem/s 29.877 Melem/s 30.218 Melem/s]
                 change:
                        time:   [−1.7511% −0.4193% +1.0683%] (p = 0.56 > 0.05)
                        thrpt:  [−1.0570% +0.4210% +1.7823%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking bltp_packet_pool/get_return/64: Collecting 100 samples in estimated 5.0001 s (155M itebltp_packet_pool/get_return/64
                        time:   [32.776 ns 33.086 ns 33.423 ns]
                        thrpt:  [29.920 Melem/s 30.224 Melem/s 30.510 Melem/s]
                 change:
                        time:   [−2.0188% −0.7464% +0.6245%] (p = 0.29 > 0.05)
                        thrpt:  [−0.6206% +0.7520% +2.0604%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking bltp_packet_pool/get_return/256: Collecting 100 samples in estimated 5.0000 s (154M itbltp_packet_pool/get_return/256
                        time:   [33.014 ns 33.328 ns 33.665 ns]
                        thrpt:  [29.705 Melem/s 30.005 Melem/s 30.291 Melem/s]
                 change:
                        time:   [−0.3281% +0.8148% +2.0019%] (p = 0.18 > 0.05)
                        thrpt:  [−1.9626% −0.8082% +0.3292%]
                        No change in performance detected.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking bltp_packet_build/build_packet/1: Collecting 100 samples in estimated 5.0047 s (4.4M ibltp_packet_build/build_packet/1
                        time:   [1.1348 µs 1.1398 µs 1.1454 µs]
                        thrpt:  [53.288 MiB/s 53.550 MiB/s 53.785 MiB/s]
                 change:
                        time:   [−0.7868% −0.2418% +0.3092%] (p = 0.40 > 0.05)
                        thrpt:  [−0.3082% +0.2424% +0.7930%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking bltp_packet_build/build_packet/10: Collecting 100 samples in estimated 5.0032 s (3.3M bltp_packet_build/build_packet/10
                        time:   [1.4997 µs 1.5049 µs 1.5103 µs]
                        thrpt:  [404.12 MiB/s 405.58 MiB/s 406.97 MiB/s]
                 change:
                        time:   [−3.0685% −2.2134% −1.4230%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4435% +2.2635% +3.1657%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking bltp_packet_build/build_packet/50: Collecting 100 samples in estimated 5.0019 s (1.7M bltp_packet_build/build_packet/50
                        time:   [2.9518 µs 2.9616 µs 2.9718 µs]
                        thrpt:  [1.0029 GiB/s 1.0063 GiB/s 1.0096 GiB/s]
                 change:
                        time:   [−2.9259% −1.8586% −0.9424%] (p = 0.00 < 0.05)
                        thrpt:  [+0.9514% +1.8938% +3.0141%]
                        Change within noise threshold.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking bltp_encryption/encrypt/64: Collecting 100 samples in estimated 5.0040 s (4.4M iteratibltp_encryption/encrypt/64
                        time:   [1.1339 µs 1.1382 µs 1.1428 µs]
                        thrpt:  [53.407 MiB/s 53.623 MiB/s 53.828 MiB/s]
                 change:
                        time:   [−0.7660% −0.2963% +0.2317%] (p = 0.22 > 0.05)
                        thrpt:  [−0.2311% +0.2972% +0.7719%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking bltp_encryption/encrypt/256: Collecting 100 samples in estimated 5.0049 s (4.1M iteratbltp_encryption/encrypt/256
                        time:   [1.2058 µs 1.2106 µs 1.2161 µs]
                        thrpt:  [200.75 MiB/s 201.67 MiB/s 202.48 MiB/s]
                 change:
                        time:   [−3.4096% −2.3970% −1.4504%] (p = 0.00 < 0.05)
                        thrpt:  [+1.4718% +2.4559% +3.5300%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking bltp_encryption/encrypt/1024: Collecting 100 samples in estimated 5.0071 s (3.1M iterabltp_encryption/encrypt/1024
                        time:   [1.5938 µs 1.6028 µs 1.6121 µs]
                        thrpt:  [605.75 MiB/s 609.30 MiB/s 612.72 MiB/s]
                 change:
                        time:   [−1.8857% −1.2304% −0.5799%] (p = 0.00 < 0.05)
                        thrpt:  [+0.5832% +1.2457% +1.9219%]
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking bltp_encryption/encrypt/4096: Collecting 100 samples in estimated 5.0090 s (1.6M iterabltp_encryption/encrypt/4096
                        time:   [3.1643 µs 3.1795 µs 3.1971 µs]
                        thrpt:  [1.1932 GiB/s 1.1998 GiB/s 1.2055 GiB/s]
                 change:
                        time:   [−2.2792% −1.6805% −1.0930%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1050% +1.7092% +2.3324%]
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

bltp_keypair/generate   time:   [21.275 µs 21.389 µs 21.518 µs]
                        thrpt:  [46.473 Kelem/s 46.752 Kelem/s 47.004 Kelem/s]
                 change:
                        time:   [−0.3205% +0.1715% +0.7039%] (p = 0.50 > 0.05)
                        thrpt:  [−0.6990% −0.1712% +0.3215%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

bltp_aad/generate       time:   [1.0079 ns 1.0620 ns 1.1115 ns]
                        thrpt:  [899.66 Melem/s 941.64 Melem/s 992.18 Melem/s]
                 change:
                        time:   [−6.0796% −0.9945% +4.5965%] (p = 0.72 > 0.05)
                        thrpt:  [−4.3945% +1.0045% +6.4731%]
                        No change in performance detected.

Benchmarking pool_comparison/shared_pool_get_return: Collecting 100 samples in estimated 5.0001 s (pool_comparison/shared_pool_get_return
                        time:   [33.116 ns 33.465 ns 33.791 ns]
                        thrpt:  [29.593 Melem/s 29.882 Melem/s 30.197 Melem/s]
                 change:
                        time:   [−3.6883% −2.0325% −0.4981%] (p = 0.01 < 0.05)
                        thrpt:  [+0.5006% +2.0747% +3.8296%]
                        Change within noise threshold.
Benchmarking pool_comparison/thread_local_pool_get_return: Collecting 100 samples in estimated 5.00pool_comparison/thread_local_pool_get_return
                        time:   [40.000 ns 40.309 ns 40.659 ns]
                        thrpt:  [24.595 Melem/s 24.809 Melem/s 25.000 Melem/s]
                 change:
                        time:   [−0.6920% +0.2674% +1.1690%] (p = 0.59 > 0.05)
                        thrpt:  [−1.1554% −0.2667% +0.6968%]
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking pool_comparison/shared_pool_10x: Collecting 100 samples in estimated 5.0011 s (16M itepool_comparison/shared_pool_10x
                        time:   [317.65 ns 321.42 ns 325.41 ns]
                        thrpt:  [3.0730 Melem/s 3.1112 Melem/s 3.1481 Melem/s]
                 change:
                        time:   [−1.9876% −0.6140% +0.6839%] (p = 0.38 > 0.05)
                        thrpt:  [−0.6792% +0.6178% +2.0279%]
                        No change in performance detected.
Benchmarking pool_comparison/thread_local_pool_10x: Collecting 100 samples in estimated 5.0013 s (1pool_comparison/thread_local_pool_10x
                        time:   [480.82 ns 484.38 ns 487.86 ns]
                        thrpt:  [2.0497 Melem/s 2.0645 Melem/s 2.0798 Melem/s]
                 change:
                        time:   [−3.6083% −2.4376% −1.2316%] (p = 0.00 < 0.05)
                        thrpt:  [+1.2469% +2.4985% +3.7434%]
                        Performance has improved.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking cipher_comparison/shared_pool/64: Collecting 100 samples in estimated 5.0014 s (4.4M icipher_comparison/shared_pool/64
                        time:   [1.1321 µs 1.1376 µs 1.1441 µs]
                        thrpt:  [53.347 MiB/s 53.654 MiB/s 53.915 MiB/s]
                 change:
                        time:   [−2.5049% −1.4382% −0.3813%] (p = 0.01 < 0.05)
                        thrpt:  [+0.3827% +1.4592% +2.5693%]
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/64: Collecting 100 samples in estimated 5.0037 s (4.4Mcipher_comparison/fast_chacha20/64
                        time:   [1.1235 µs 1.1274 µs 1.1321 µs]
                        thrpt:  [53.913 MiB/s 54.136 MiB/s 54.326 MiB/s]
                 change:
                        time:   [−3.3078% −2.3432% −1.5089%] (p = 0.00 < 0.05)
                        thrpt:  [+1.5321% +2.3994% +3.4210%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/shared_pool/256: Collecting 100 samples in estimated 5.0042 s (4.1M cipher_comparison/shared_pool/256
                        time:   [1.2058 µs 1.2118 µs 1.2181 µs]
                        thrpt:  [200.42 MiB/s 201.48 MiB/s 202.47 MiB/s]
                 change:
                        time:   [−1.8740% −1.2946% −0.7133%] (p = 0.00 < 0.05)
                        thrpt:  [+0.7184% +1.3115% +1.9098%]
                        Change within noise threshold.
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking cipher_comparison/fast_chacha20/256: Collecting 100 samples in estimated 5.0032 s (4.1cipher_comparison/fast_chacha20/256
                        time:   [1.2024 µs 1.2082 µs 1.2143 µs]
                        thrpt:  [201.05 MiB/s 202.07 MiB/s 203.05 MiB/s]
                 change:
                        time:   [−2.4936% −1.8272% −1.1752%] (p = 0.00 < 0.05)
                        thrpt:  [+1.1892% +1.8612% +2.5573%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cipher_comparison/shared_pool/1024: Collecting 100 samples in estimated 5.0065 s (3.1Mcipher_comparison/shared_pool/1024
                        time:   [1.6026 µs 1.6121 µs 1.6219 µs]
                        thrpt:  [602.11 MiB/s 605.77 MiB/s 609.35 MiB/s]
                 change:
                        time:   [−3.8056% −2.7295% −1.6628%] (p = 0.00 < 0.05)
                        thrpt:  [+1.6909% +2.8061% +3.9562%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking cipher_comparison/fast_chacha20/1024: Collecting 100 samples in estimated 5.0020 s (3.cipher_comparison/fast_chacha20/1024
                        time:   [1.5708 µs 1.5764 µs 1.5824 µs]
                        thrpt:  [617.16 MiB/s 619.50 MiB/s 621.68 MiB/s]
                 change:
                        time:   [−5.2023% −3.9800% −2.7255%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8019% +4.1449% +5.4878%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking cipher_comparison/shared_pool/4096: Collecting 100 samples in estimated 5.0098 s (1.6Mcipher_comparison/shared_pool/4096
                        time:   [3.1711 µs 3.1838 µs 3.1962 µs]
                        thrpt:  [1.1935 GiB/s 1.1982 GiB/s 1.2029 GiB/s]
                 change:
                        time:   [−7.7796% −5.9469% −4.4923%] (p = 0.00 < 0.05)
                        thrpt:  [+4.7036% +6.3229% +8.4358%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking cipher_comparison/fast_chacha20/4096: Collecting 100 samples in estimated 5.0110 s (1.cipher_comparison/fast_chacha20/4096
                        time:   [3.1055 µs 3.1220 µs 3.1409 µs]
                        thrpt:  [1.2145 GiB/s 1.2219 GiB/s 1.2284 GiB/s]
                 change:
                        time:   [−5.4679% −4.6924% −3.9671%] (p = 0.00 < 0.05)
                        thrpt:  [+4.1310% +4.9234% +5.7841%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking adaptive_batcher/optimal_size: Collecting 100 samples in estimated 5.0000 s (6.0B iteradaptive_batcher/optimal_size
                        time:   [823.12 ps 828.74 ps 834.79 ps]
                        thrpt:  [1.1979 Gelem/s 1.2067 Gelem/s 1.2149 Gelem/s]
                 change:
                        time:   [−1.9983% −0.9823% +0.1530%] (p = 0.06 > 0.05)
                        thrpt:  [−0.1528% +0.9921% +2.0390%]
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking adaptive_batcher/record: Collecting 100 samples in estimated 5.0000 s (516M iterationsadaptive_batcher/record time:   [9.6692 ns 9.7081 ns 9.7526 ns]
                        thrpt:  [102.54 Melem/s 103.01 Melem/s 103.42 Melem/s]
                 change:
                        time:   [−1.1710% −0.5865% +0.0339%] (p = 0.06 > 0.05)
                        thrpt:  [−0.0339% +0.5900% +1.1849%]
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking adaptive_batcher/full_cycle: Collecting 100 samples in estimated 5.0000 s (607M iteratadaptive_batcher/full_cycle
                        time:   [8.1972 ns 8.2401 ns 8.2881 ns]
                        thrpt:  [120.66 Melem/s 121.36 Melem/s 121.99 Melem/s]
                 change:
                        time:   [−3.2787% −2.3458% −1.3412%] (p = 0.00 < 0.05)
                        thrpt:  [+1.3595% +2.4021% +3.3898%]
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking e2e_packet_build/shared_pool_50_events: Collecting 100 samples in estimated 5.0034 s (e2e_packet_build/shared_pool_50_events
                        time:   [2.9477 µs 2.9614 µs 2.9773 µs]
                        thrpt:  [1.0010 GiB/s 1.0063 GiB/s 1.0110 GiB/s]
                 change:
                        time:   [−5.7184% −4.7071% −3.8054%] (p = 0.00 < 0.05)
                        thrpt:  [+3.9560% +4.9396% +6.0653%]
                        Performance has improved.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking e2e_packet_build/fast_50_events: Collecting 100 samples in estimated 5.0117 s (1.7M ite2e_packet_build/fast_50_events
                        time:   [2.9027 µs 2.9145 µs 2.9266 µs]
                        thrpt:  [1.0183 GiB/s 1.0225 GiB/s 1.0267 GiB/s]
                 change:
                        time:   [−4.1950% −3.4527% −2.7335%] (p = 0.00 < 0.05)
                        thrpt:  [+2.8103% +3.5762% +4.3787%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking multithread_packet_build/shared_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/shared_pool/8: Collecting 100 samples in estimated 9.0761 s (multithread_packet_build/shared_pool/8
                        time:   [1.7699 ms 1.7858 ms 1.8017 ms]
                        thrpt:  [4.4403 Melem/s 4.4799 Melem/s 4.5201 Melem/s]
                 change:
                        time:   [−6.1314% −4.9851% −3.7998%] (p = 0.00 < 0.05)
                        thrpt:  [+3.9498% +5.2466% +6.5319%]
                        Performance has improved.
Benchmarking multithread_packet_build/thread_local_pool/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.0s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_packet_build/thread_local_pool/8: Collecting 100 samples in estimated 7.97multithread_packet_build/thread_local_pool/8
                        time:   [1.5380 ms 1.5508 ms 1.5637 ms]
                        thrpt:  [5.1160 Melem/s 5.1586 Melem/s 5.2015 Melem/s]
                 change:
                        time:   [−9.4864% −8.5046% −7.5091%] (p = 0.00 < 0.05)
                        thrpt:  [+8.1188% +9.2951% +10.481%]
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_packet_build/shared_pool/16: Collecting 100 samples in estimated 5.1496 s multithread_packet_build/shared_pool/16
                        time:   [2.7916 ms 2.8302 ms 2.8708 ms]
                        thrpt:  [5.5734 Melem/s 5.6533 Melem/s 5.7314 Melem/s]
                 change:
                        time:   [−12.287% −10.276% −8.2104%] (p = 0.00 < 0.05)
                        thrpt:  [+8.9448% +11.453% +14.008%]
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking multithread_packet_build/thread_local_pool/16: Collecting 100 samples in estimated 5.2multithread_packet_build/thread_local_pool/16
                        time:   [2.0796 ms 2.1174 ms 2.1574 ms]
                        thrpt:  [7.4163 Melem/s 7.5566 Melem/s 7.6938 Melem/s]
                 change:
                        time:   [−14.173% −11.839% −9.3834%] (p = 0.00 < 0.05)
                        thrpt:  [+10.355% +13.429% +16.514%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking multithread_packet_build/shared_pool/24: Collecting 100 samples in estimated 5.1466 s multithread_packet_build/shared_pool/24
                        time:   [4.2028 ms 4.3035 ms 4.4199 ms]
                        thrpt:  [5.4300 Melem/s 5.5769 Melem/s 5.7104 Melem/s]
                 change:
                        time:   [−15.638% −12.160% −8.6772%] (p = 0.00 < 0.05)
                        thrpt:  [+9.5017% +13.843% +18.537%]
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/24: Collecting 100 samples in estimated 5.2multithread_packet_build/thread_local_pool/24
                        time:   [3.0767 ms 3.1152 ms 3.1550 ms]
                        thrpt:  [7.6070 Melem/s 7.7042 Melem/s 7.8005 Melem/s]
                 change:
                        time:   [−7.7340% −6.1606% −4.5012%] (p = 0.00 < 0.05)
                        thrpt:  [+4.7134% +6.5650% +8.3822%]
                        Performance has improved.
Benchmarking multithread_packet_build/shared_pool/32: Collecting 100 samples in estimated 5.0422 s multithread_packet_build/shared_pool/32
                        time:   [5.4961 ms 5.6272 ms 5.7794 ms]
                        thrpt:  [5.5369 Melem/s 5.6867 Melem/s 5.8224 Melem/s]
                 change:
                        time:   [−6.2900% −2.2558% +1.9287%] (p = 0.30 > 0.05)
                        thrpt:  [−1.8922% +2.3079% +6.7123%]
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking multithread_packet_build/thread_local_pool/32: Collecting 100 samples in estimated 5.2multithread_packet_build/thread_local_pool/32
                        time:   [3.6457 ms 3.7177 ms 3.7938 ms]
                        thrpt:  [8.4347 Melem/s 8.6074 Melem/s 8.7776 Melem/s]
                 change:
                        time:   [+1.9987% +5.8090% +9.5471%] (p = 0.00 < 0.05)
                        thrpt:  [−8.7150% −5.4901% −1.9596%]
                        Performance has regressed.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking multithread_mixed_frames/shared_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.3s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/shared_mixed/8: Collecting 100 samples in estimated 6.3495 s multithread_mixed_frames/shared_mixed/8
                        time:   [1.2492 ms 1.2559 ms 1.2623 ms]
                        thrpt:  [9.5066 Melem/s 9.5547 Melem/s 9.6060 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) low mild
  2 (2.00%) high mild
Benchmarking multithread_mixed_frames/fast_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 5.7s, enable flat sampling, or reduce sample count to 60.
Benchmarking multithread_mixed_frames/fast_mixed/8: Collecting 100 samples in estimated 5.7490 s (5multithread_mixed_frames/fast_mixed/8
                        time:   [1.1331 ms 1.1409 ms 1.1483 ms]
                        thrpt:  [10.450 Melem/s 10.518 Melem/s 10.590 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/shared_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.1s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/shared_mixed/16: Collecting 100 samples in estimated 9.0909 smultithread_mixed_frames/shared_mixed/16
                        time:   [1.9078 ms 1.9486 ms 1.9889 ms]
                        thrpt:  [12.067 Melem/s 12.316 Melem/s 12.580 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/fast_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 8.7s, enable flat sampling, or reduce sample count to 50.
Benchmarking multithread_mixed_frames/fast_mixed/16: Collecting 100 samples in estimated 8.7038 s (multithread_mixed_frames/fast_mixed/16
                        time:   [1.7017 ms 1.7394 ms 1.7773 ms]
                        thrpt:  [13.504 Melem/s 13.798 Melem/s 14.104 Melem/s]
Benchmarking multithread_mixed_frames/shared_mixed/24: Collecting 100 samples in estimated 5.1445 smultithread_mixed_frames/shared_mixed/24
                        time:   [2.5465 ms 2.6323 ms 2.7327 ms]
                        thrpt:  [13.174 Melem/s 13.676 Melem/s 14.137 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/24: Collecting 100 samples in estimated 5.0242 s (multithread_mixed_frames/fast_mixed/24
                        time:   [2.2431 ms 2.2817 ms 2.3210 ms]
                        thrpt:  [15.511 Melem/s 15.778 Melem/s 16.049 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking multithread_mixed_frames/shared_mixed/32: Collecting 100 samples in estimated 5.2450 smultithread_mixed_frames/shared_mixed/32
                        time:   [3.3898 ms 3.5061 ms 3.6355 ms]
                        thrpt:  [13.203 Melem/s 13.690 Melem/s 14.160 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking multithread_mixed_frames/fast_mixed/32: Collecting 100 samples in estimated 5.2280 s (multithread_mixed_frames/fast_mixed/32
                        time:   [2.7624 ms 2.8195 ms 2.8785 ms]
                        thrpt:  [16.676 Melem/s 17.024 Melem/s 17.376 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking pool_contention/shared_acquire_release/8: Collecting 100 samples in estimated 5.2221 spool_contention/shared_acquire_release/8
                        time:   [8.6963 ms 8.7283 ms 8.7603 ms]
                        thrpt:  [9.1322 Melem/s 9.1656 Melem/s 9.1993 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking pool_contention/fast_acquire_release/8: Collecting 100 samples in estimated 9.6993 s (pool_contention/fast_acquire_release/8
                        time:   [959.78 µs 968.01 µs 975.70 µs]
                        thrpt:  [81.992 Melem/s 82.644 Melem/s 83.352 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) low mild
Benchmarking pool_contention/shared_acquire_release/16: Collecting 100 samples in estimated 6.1068 pool_contention/shared_acquire_release/16
                        time:   [20.286 ms 20.381 ms 20.477 ms]
                        thrpt:  [7.8136 Melem/s 7.8504 Melem/s 7.8872 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild
Benchmarking pool_contention/fast_acquire_release/16: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 6.7s, enable flat sampling, or reduce sample count to 60.
Benchmarking pool_contention/fast_acquire_release/16: Collecting 100 samples in estimated 6.7220 s pool_contention/fast_acquire_release/16
                        time:   [1.2990 ms 1.3143 ms 1.3300 ms]
                        thrpt:  [120.30 Melem/s 121.74 Melem/s 123.17 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking pool_contention/shared_acquire_release/24: Collecting 100 samples in estimated 5.9689 pool_contention/shared_acquire_release/24
                        time:   [30.241 ms 30.619 ms 31.034 ms]
                        thrpt:  [7.7334 Melem/s 7.8383 Melem/s 7.9362 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking pool_contention/fast_acquire_release/24: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.0s, enable flat sampling, or reduce sample count to 50.
Benchmarking pool_contention/fast_acquire_release/24: Collecting 100 samples in estimated 9.0446 s pool_contention/fast_acquire_release/24
                        time:   [1.7557 ms 1.7749 ms 1.7941 ms]
                        thrpt:  [133.77 Melem/s 135.22 Melem/s 136.70 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking pool_contention/shared_acquire_release/32: Collecting 100 samples in estimated 8.0681 pool_contention/shared_acquire_release/32
                        time:   [39.958 ms 40.292 ms 40.645 ms]
                        thrpt:  [7.8730 Melem/s 7.9421 Melem/s 8.0083 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high severe
Benchmarking pool_contention/fast_acquire_release/32: Collecting 100 samples in estimated 5.0427 s pool_contention/fast_acquire_release/32
                        time:   [2.2045 ms 2.2420 ms 2.2800 ms]
                        thrpt:  [140.35 Melem/s 142.73 Melem/s 145.15 Melem/s]

Benchmarking throughput_scaling/fast_pool_scaling/1: Collecting 20 samples in estimated 5.2884 s (1throughput_scaling/fast_pool_scaling/1
                        time:   [3.5723 ms 3.5843 ms 3.5985 ms]
                        thrpt:  [555.79 Kelem/s 558.00 Kelem/s 559.86 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/2: Collecting 20 samples in estimated 5.5412 s (1throughput_scaling/fast_pool_scaling/2
                        time:   [3.7191 ms 3.7487 ms 3.7769 ms]
                        thrpt:  [1.0591 Melem/s 1.0670 Melem/s 1.0755 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/4: Collecting 20 samples in estimated 5.1923 s (1throughput_scaling/fast_pool_scaling/4
                        time:   [3.9414 ms 3.9937 ms 4.0460 ms]
                        thrpt:  [1.9772 Melem/s 2.0032 Melem/s 2.0297 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) low mild
  1 (5.00%) high mild
Benchmarking throughput_scaling/fast_pool_scaling/8: Collecting 20 samples in estimated 5.1048 s (1throughput_scaling/fast_pool_scaling/8
                        time:   [4.8057 ms 4.8470 ms 4.8875 ms]
                        thrpt:  [3.2737 Melem/s 3.3010 Melem/s 3.3294 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/16: Collecting 20 samples in estimated 6.0960 s (throughput_scaling/fast_pool_scaling/16
                        time:   [5.7204 ms 5.8128 ms 5.9157 ms]
                        thrpt:  [5.4094 Melem/s 5.5051 Melem/s 5.5940 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/24: Collecting 20 samples in estimated 5.8786 s (throughput_scaling/fast_pool_scaling/24
                        time:   [8.9398 ms 9.0385 ms 9.1392 ms]
                        thrpt:  [5.2521 Melem/s 5.3106 Melem/s 5.3693 Melem/s]
Benchmarking throughput_scaling/fast_pool_scaling/32: Collecting 20 samples in estimated 6.6786 s (throughput_scaling/fast_pool_scaling/32
                        time:   [10.495 ms 10.621 ms 10.731 ms]
                        thrpt:  [5.9638 Melem/s 6.0257 Melem/s 6.0980 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

Benchmarking routing_header/serialize: Collecting 100 samples in estimated 5.0000 s (16B iterationsrouting_header/serialize
                        time:   [307.28 ps 309.02 ps 310.99 ps]
                        thrpt:  [3.2156 Gelem/s 3.2360 Gelem/s 3.2544 Gelem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking routing_header/deserialize: Collecting 100 samples in estimated 5.0000 s (6.1B iteratirouting_header/deserialize
                        time:   [749.98 ps 763.80 ps 776.93 ps]
                        thrpt:  [1.2871 Gelem/s 1.3093 Gelem/s 1.3334 Gelem/s]
Benchmarking routing_header/roundtrip: Collecting 100 samples in estimated 5.0000 s (6.1B iterationrouting_header/roundtrip
                        time:   [744.35 ps 757.92 ps 770.70 ps]
                        thrpt:  [1.2975 Gelem/s 1.3194 Gelem/s 1.3434 Gelem/s]
routing_header/forward  time:   [223.04 ps 237.39 ps 254.53 ps]
                        thrpt:  [3.9288 Gelem/s 4.2126 Gelem/s 4.4835 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe

Benchmarking routing_table/lookup_hit: Collecting 100 samples in estimated 5.0000 s (363M iterationrouting_table/lookup_hit
                        time:   [13.743 ns 13.840 ns 13.942 ns]
                        thrpt:  [71.725 Melem/s 72.253 Melem/s 72.766 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking routing_table/lookup_miss: Collecting 100 samples in estimated 5.0000 s (296M iteratiorouting_table/lookup_miss
                        time:   [16.871 ns 16.919 ns 16.971 ns]
                        thrpt:  [58.923 Melem/s 59.106 Melem/s 59.273 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
routing_table/is_local  time:   [201.93 ps 202.16 ps 202.41 ps]
                        thrpt:  [4.9405 Gelem/s 4.9466 Gelem/s 4.9523 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
routing_table/add_route time:   [237.53 ns 239.05 ns 240.65 ns]
                        thrpt:  [4.1554 Melem/s 4.1832 Melem/s 4.2100 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  5 (5.00%) low mild
  1 (1.00%) high mild
Benchmarking routing_table/record_in: Collecting 100 samples in estimated 5.0001 s (119M iterationsrouting_table/record_in time:   [41.758 ns 42.015 ns 42.285 ns]
                        thrpt:  [23.649 Melem/s 23.801 Melem/s 23.947 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking routing_table/record_out: Collecting 100 samples in estimated 5.0001 s (229M iterationrouting_table/record_out
                        time:   [21.361 ns 21.456 ns 21.570 ns]
                        thrpt:  [46.360 Melem/s 46.606 Melem/s 46.815 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  6 (6.00%) high mild
  5 (5.00%) high severe
Benchmarking routing_table/aggregate_stats: Collecting 100 samples in estimated 5.0267 s (596k iterrouting_table/aggregate_stats
                        time:   [8.1195 µs 8.1608 µs 8.2126 µs]
                        thrpt:  [121.76 Kelem/s 122.54 Kelem/s 123.16 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe

Benchmarking fair_scheduler/creation: Collecting 100 samples in estimated 5.0059 s (3.1M iterationsfair_scheduler/creation time:   [1.6394 µs 1.6471 µs 1.6561 µs]
                        thrpt:  [603.83 Kelem/s 607.13 Kelem/s 609.98 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking fair_scheduler/stream_count_empty: Collecting 100 samples in estimated 5.0018 s (5.1M fair_scheduler/stream_count_empty
                        time:   [973.00 ns 976.75 ns 981.00 ns]
                        thrpt:  [1.0194 Melem/s 1.0238 Melem/s 1.0278 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe
Benchmarking fair_scheduler/total_queued: Collecting 100 samples in estimated 5.0000 s (25B iteratifair_scheduler/total_queued
                        time:   [202.32 ps 202.86 ps 203.45 ps]
                        thrpt:  [4.9153 Gelem/s 4.9296 Gelem/s 4.9426 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking fair_scheduler/cleanup_empty: Collecting 100 samples in estimated 5.0020 s (3.9M iterafair_scheduler/cleanup_empty
                        time:   [1.2961 µs 1.2988 µs 1.3021 µs]
                        thrpt:  [768.02 Kelem/s 769.94 Kelem/s 771.55 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking routing_table_concurrent/concurrent_lookup/4: Collecting 100 samples in estimated 5.73routing_table_concurrent/concurrent_lookup/4
                        time:   [178.15 µs 182.45 µs 186.83 µs]
                        thrpt:  [21.410 Melem/s 21.924 Melem/s 22.453 Melem/s]
Benchmarking routing_table_concurrent/concurrent_stats/4: Collecting 100 samples in estimated 5.567routing_table_concurrent/concurrent_stats/4
                        time:   [262.95 µs 269.30 µs 275.76 µs]
                        thrpt:  [14.505 Melem/s 14.853 Melem/s 15.212 Melem/s]
Benchmarking routing_table_concurrent/concurrent_lookup/8: Collecting 100 samples in estimated 6.44routing_table_concurrent/concurrent_lookup/8
                        time:   [306.92 µs 314.17 µs 321.74 µs]
                        thrpt:  [24.864 Melem/s 25.464 Melem/s 26.066 Melem/s]
Benchmarking routing_table_concurrent/concurrent_stats/8: Collecting 100 samples in estimated 6.179routing_table_concurrent/concurrent_stats/8
                        time:   [400.96 µs 406.86 µs 412.68 µs]
                        thrpt:  [19.386 Melem/s 19.663 Melem/s 19.952 Melem/s]
Benchmarking routing_table_concurrent/concurrent_lookup/16: Collecting 100 samples in estimated 6.0routing_table_concurrent/concurrent_lookup/16
                        time:   [598.11 µs 611.00 µs 623.18 µs]
                        thrpt:  [25.675 Melem/s 26.187 Melem/s 26.751 Melem/s]
Benchmarking routing_table_concurrent/concurrent_stats/16: Collecting 100 samples in estimated 6.98routing_table_concurrent/concurrent_stats/16
                        time:   [677.58 µs 687.85 µs 697.52 µs]
                        thrpt:  [22.938 Melem/s 23.261 Melem/s 23.614 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) low mild
  1 (1.00%) high mild

Benchmarking routing_decision/parse_lookup_forward: Collecting 100 samples in estimated 5.0001 s (3routing_decision/parse_lookup_forward
                        time:   [14.292 ns 14.377 ns 14.478 ns]
                        thrpt:  [69.072 Melem/s 69.556 Melem/s 69.970 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking routing_decision/full_with_stats: Collecting 100 samples in estimated 5.0003 s (61M itrouting_decision/full_with_stats
                        time:   [81.492 ns 81.937 ns 82.409 ns]
                        thrpt:  [12.135 Melem/s 12.204 Melem/s 12.271 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  9 (9.00%) high mild
  1 (1.00%) high severe

Benchmarking stream_multiplexing/lookup_all/10: Collecting 100 samples in estimated 5.0001 s (34M istream_multiplexing/lookup_all/10
                        time:   [145.63 ns 146.25 ns 146.92 ns]
                        thrpt:  [68.064 Melem/s 68.378 Melem/s 68.668 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/10: Collecting 100 samples in estimated 5.0004 s (12M itstream_multiplexing/stats_all/10
                        time:   [417.80 ns 419.31 ns 420.98 ns]
                        thrpt:  [23.754 Melem/s 23.849 Melem/s 23.935 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking stream_multiplexing/lookup_all/100: Collecting 100 samples in estimated 5.0060 s (3.6Mstream_multiplexing/lookup_all/100
                        time:   [1.3935 µs 1.4008 µs 1.4088 µs]
                        thrpt:  [70.984 Melem/s 71.385 Melem/s 71.763 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking stream_multiplexing/stats_all/100: Collecting 100 samples in estimated 5.0091 s (1.2M stream_multiplexing/stats_all/100
                        time:   [4.1903 µs 4.2109 µs 4.2349 µs]
                        thrpt:  [23.613 Melem/s 23.748 Melem/s 23.865 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking stream_multiplexing/lookup_all/1000: Collecting 100 samples in estimated 5.0008 s (343stream_multiplexing/lookup_all/1000
                        time:   [14.485 µs 14.556 µs 14.632 µs]
                        thrpt:  [68.341 Melem/s 68.703 Melem/s 69.035 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking stream_multiplexing/stats_all/1000: Collecting 100 samples in estimated 5.0586 s (111kstream_multiplexing/stats_all/1000
                        time:   [45.328 µs 45.515 µs 45.716 µs]
                        thrpt:  [21.874 Melem/s 21.971 Melem/s 22.062 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking stream_multiplexing/lookup_all/10000: Collecting 100 samples in estimated 5.3111 s (35stream_multiplexing/lookup_all/10000
                        time:   [149.27 µs 150.80 µs 152.61 µs]
                        thrpt:  [65.527 Melem/s 66.311 Melem/s 66.992 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe
Benchmarking stream_multiplexing/stats_all/10000: Collecting 100 samples in estimated 7.3120 s (15kstream_multiplexing/stats_all/10000
                        time:   [480.40 µs 483.49 µs 486.85 µs]
                        thrpt:  [20.540 Melem/s 20.683 Melem/s 20.816 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

Benchmarking multihop_packet_builder/build/64: Collecting 100 samples in estimated 5.0001 s (105M imultihop_packet_builder/build/64
                        time:   [47.564 ns 47.740 ns 47.939 ns]
                        thrpt:  [1.2433 GiB/s 1.2485 GiB/s 1.2531 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking multihop_packet_builder/build_priority/64: Collecting 100 samples in estimated 5.0001 multihop_packet_builder/build_priority/64
                        time:   [31.043 ns 31.259 ns 31.495 ns]
                        thrpt:  [1.8925 GiB/s 1.9068 GiB/s 1.9201 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking multihop_packet_builder/build/256: Collecting 100 samples in estimated 5.0002 s (119M multihop_packet_builder/build/256
                        time:   [41.746 ns 41.914 ns 42.104 ns]
                        thrpt:  [5.6626 GiB/s 5.6882 GiB/s 5.7111 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_packet_builder/build_priority/256: Collecting 100 samples in estimated 5.0000multihop_packet_builder/build_priority/256
                        time:   [31.459 ns 31.705 ns 31.973 ns]
                        thrpt:  [7.4568 GiB/s 7.5198 GiB/s 7.5788 GiB/s]
Found 12 outliers among 100 measurements (12.00%)
  6 (6.00%) high mild
  6 (6.00%) high severe
Benchmarking multihop_packet_builder/build/1024: Collecting 100 samples in estimated 5.0001 s (109Mmultihop_packet_builder/build/1024
                        time:   [45.352 ns 45.548 ns 45.760 ns]
                        thrpt:  [20.841 GiB/s 20.938 GiB/s 21.028 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking multihop_packet_builder/build_priority/1024: Collecting 100 samples in estimated 5.000multihop_packet_builder/build_priority/1024
                        time:   [39.886 ns 40.098 ns 40.327 ns]
                        thrpt:  [23.649 GiB/s 23.783 GiB/s 23.910 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking multihop_packet_builder/build/4096: Collecting 100 samples in estimated 5.0002 s (77M multihop_packet_builder/build/4096
                        time:   [65.816 ns 66.203 ns 66.629 ns]
                        thrpt:  [57.253 GiB/s 57.621 GiB/s 57.960 GiB/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking multihop_packet_builder/build_priority/4096: Collecting 100 samples in estimated 5.000multihop_packet_builder/build_priority/4096
                        time:   [63.000 ns 63.393 ns 63.811 ns]
                        thrpt:  [59.781 GiB/s 60.176 GiB/s 60.551 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking multihop_chain/forward_chain/1: Collecting 100 samples in estimated 5.0003 s (95M itermultihop_chain/forward_chain/1
                        time:   [52.668 ns 52.972 ns 53.317 ns]
                        thrpt:  [18.756 Melem/s 18.878 Melem/s 18.987 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking multihop_chain/forward_chain/2: Collecting 100 samples in estimated 5.0001 s (59M itermultihop_chain/forward_chain/2
                        time:   [85.037 ns 85.618 ns 86.300 ns]
                        thrpt:  [11.588 Melem/s 11.680 Melem/s 11.760 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking multihop_chain/forward_chain/3: Collecting 100 samples in estimated 5.0002 s (42M itermultihop_chain/forward_chain/3
                        time:   [117.77 ns 118.60 ns 119.53 ns]
                        thrpt:  [8.3663 Melem/s 8.4315 Melem/s 8.4909 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking multihop_chain/forward_chain/4: Collecting 100 samples in estimated 5.0008 s (33M itermultihop_chain/forward_chain/4
                        time:   [151.30 ns 152.32 ns 153.41 ns]
                        thrpt:  [6.5183 Melem/s 6.5651 Melem/s 6.6095 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_chain/forward_chain/5: Collecting 100 samples in estimated 5.0009 s (27M itermultihop_chain/forward_chain/5
                        time:   [186.10 ns 187.23 ns 188.43 ns]
                        thrpt:  [5.3070 Melem/s 5.3409 Melem/s 5.3733 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking hop_latency/single_hop_process: Collecting 100 samples in estimated 5.0000 s (6.3B itehop_latency/single_hop_process
                        time:   [786.68 ps 792.30 ps 798.50 ps]
                        thrpt:  [1.2523 Gelem/s 1.2621 Gelem/s 1.2712 Gelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking hop_latency/single_hop_full: Collecting 100 samples in estimated 5.0002 s (134M iterathop_latency/single_hop_full
                        time:   [33.505 ns 33.774 ns 34.063 ns]
                        thrpt:  [29.357 Melem/s 29.609 Melem/s 29.846 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe

hop_scaling/64B_1hops   time:   [52.640 ns 52.892 ns 53.159 ns]
                        thrpt:  [1.1213 GiB/s 1.1269 GiB/s 1.1323 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
hop_scaling/64B_2hops   time:   [84.804 ns 85.319 ns 85.852 ns]
                        thrpt:  [710.93 MiB/s 715.38 MiB/s 719.72 MiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/64B_3hops   time:   [140.39 ns 141.14 ns 142.01 ns]
                        thrpt:  [429.80 MiB/s 432.45 MiB/s 434.75 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
hop_scaling/64B_4hops   time:   [150.82 ns 151.95 ns 153.16 ns]
                        thrpt:  [398.51 MiB/s 401.68 MiB/s 404.70 MiB/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
hop_scaling/64B_5hops   time:   [221.21 ns 222.34 ns 223.58 ns]
                        thrpt:  [272.99 MiB/s 274.52 MiB/s 275.91 MiB/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
hop_scaling/256B_1hops  time:   [52.764 ns 53.002 ns 53.259 ns]
                        thrpt:  [4.4766 GiB/s 4.4983 GiB/s 4.5186 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/256B_2hops  time:   [85.470 ns 86.032 ns 86.626 ns]
                        thrpt:  [2.7523 GiB/s 2.7713 GiB/s 2.7895 GiB/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
hop_scaling/256B_3hops  time:   [117.78 ns 118.77 ns 119.99 ns]
                        thrpt:  [1.9870 GiB/s 2.0074 GiB/s 2.0243 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
hop_scaling/256B_4hops  time:   [151.89 ns 152.93 ns 154.11 ns]
                        thrpt:  [1.5471 GiB/s 1.5590 GiB/s 1.5697 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
hop_scaling/256B_5hops  time:   [185.86 ns 186.82 ns 187.92 ns]
                        thrpt:  [1.2687 GiB/s 1.2762 GiB/s 1.2828 GiB/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
hop_scaling/1024B_1hops time:   [55.152 ns 55.427 ns 55.731 ns]
                        thrpt:  [17.112 GiB/s 17.206 GiB/s 17.292 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
hop_scaling/1024B_2hops time:   [92.175 ns 92.584 ns 93.024 ns]
                        thrpt:  [10.252 GiB/s 10.301 GiB/s 10.346 GiB/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_3hops time:   [132.13 ns 132.92 ns 133.77 ns]
                        thrpt:  [7.1293 GiB/s 7.1747 GiB/s 7.2177 GiB/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
hop_scaling/1024B_4hops time:   [173.07 ns 173.95 ns 174.89 ns]
                        thrpt:  [5.4530 GiB/s 5.4825 GiB/s 5.5104 GiB/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
hop_scaling/1024B_5hops time:   [217.12 ns 218.58 ns 220.30 ns]
                        thrpt:  [4.3290 GiB/s 4.3630 GiB/s 4.3924 GiB/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking multihop_with_routing/route_and_forward/1: Collecting 100 samples in estimated 5.0004 multihop_with_routing/route_and_forward/1
                        time:   [130.63 ns 131.37 ns 132.22 ns]
                        thrpt:  [7.5634 Melem/s 7.6120 Melem/s 7.6552 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/2: Collecting 100 samples in estimated 5.0007 multihop_with_routing/route_and_forward/2
                        time:   [240.93 ns 242.47 ns 244.27 ns]
                        thrpt:  [4.0939 Melem/s 4.1243 Melem/s 4.1507 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/3: Collecting 100 samples in estimated 5.0016 multihop_with_routing/route_and_forward/3
                        time:   [349.96 ns 351.30 ns 352.72 ns]
                        thrpt:  [2.8351 Melem/s 2.8466 Melem/s 2.8574 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/4: Collecting 100 samples in estimated 5.0004 multihop_with_routing/route_and_forward/4
                        time:   [463.76 ns 466.91 ns 470.34 ns]
                        thrpt:  [2.1261 Melem/s 2.1417 Melem/s 2.1563 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking multihop_with_routing/route_and_forward/5: Collecting 100 samples in estimated 5.0019 multihop_with_routing/route_and_forward/5
                        time:   [572.94 ns 576.02 ns 579.42 ns]
                        thrpt:  [1.7259 Melem/s 1.7361 Melem/s 1.7454 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Benchmarking multihop_concurrent/concurrent_forward/4: Collecting 20 samples in estimated 5.2166 s multihop_concurrent/concurrent_forward/4
                        time:   [1.1264 ms 1.1323 ms 1.1382 ms]
                        thrpt:  [3.5144 Melem/s 3.5325 Melem/s 3.5510 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) low mild
Benchmarking multihop_concurrent/concurrent_forward/8: Collecting 20 samples in estimated 5.1128 s multihop_concurrent/concurrent_forward/8
                        time:   [864.47 µs 877.68 µs 888.83 µs]
                        thrpt:  [9.0006 Melem/s 9.1150 Melem/s 9.2543 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking multihop_concurrent/concurrent_forward/16: Collecting 20 samples in estimated 5.1863 smultihop_concurrent/concurrent_forward/16
                        time:   [1.1660 ms 1.1791 ms 1.1942 ms]
                        thrpt:  [13.398 Melem/s 13.569 Melem/s 13.722 Melem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) high mild

pingwave/serialize      time:   [522.21 ps 529.78 ps 538.73 ps]
                        thrpt:  [1.8562 Gelem/s 1.8876 Gelem/s 1.9149 Gelem/s]
Found 18 outliers among 100 measurements (18.00%)
  1 (1.00%) high mild
  17 (17.00%) high severe
pingwave/deserialize    time:   [651.08 ps 679.87 ps 714.79 ps]
                        thrpt:  [1.3990 Gelem/s 1.4709 Gelem/s 1.5359 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  1 (1.00%) high mild
  18 (18.00%) high severe
pingwave/roundtrip      time:   [652.10 ps 680.37 ps 714.52 ps]
                        thrpt:  [1.3995 Gelem/s 1.4698 Gelem/s 1.5335 Gelem/s]
Found 17 outliers among 100 measurements (17.00%)
  17 (17.00%) high severe
pingwave/forward        time:   [529.50 ps 540.82 ps 553.14 ps]
                        thrpt:  [1.8078 Gelem/s 1.8490 Gelem/s 1.8886 Gelem/s]

Benchmarking capabilities/serialize_simple: Collecting 100 samples in estimated 5.0001 s (131M itercapabilities/serialize_simple
                        time:   [37.897 ns 38.090 ns 38.277 ns]
                        thrpt:  [26.125 Melem/s 26.254 Melem/s 26.388 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capabilities/deserialize_simple: Collecting 100 samples in estimated 5.0000 s (521M itcapabilities/deserialize_simple
                        time:   [9.5448 ns 9.7975 ns 10.036 ns]
                        thrpt:  [99.640 Melem/s 102.07 Melem/s 104.77 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  13 (13.00%) high mild
  3 (3.00%) high severe
Benchmarking capabilities/serialize_complex: Collecting 100 samples in estimated 5.0000 s (128M itecapabilities/serialize_complex
                        time:   [39.597 ns 39.913 ns 40.267 ns]
                        thrpt:  [24.834 Melem/s 25.055 Melem/s 25.255 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking capabilities/deserialize_complex: Collecting 100 samples in estimated 5.0005 s (21M itcapabilities/deserialize_complex
                        time:   [239.43 ns 241.75 ns 244.62 ns]
                        thrpt:  [4.0880 Melem/s 4.1365 Melem/s 4.1766 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe

Benchmarking local_graph/create_pingwave: Collecting 100 samples in estimated 5.0000 s (977M iteratlocal_graph/create_pingwave
                        time:   [5.1119 ns 5.1314 ns 5.1515 ns]
                        thrpt:  [194.12 Melem/s 194.88 Melem/s 195.62 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking local_graph/on_pingwave_new: Collecting 100 samples in estimated 5.0006 s (25M iteratilocal_graph/on_pingwave_new
                        time:   [194.00 ns 196.14 ns 198.45 ns]
                        thrpt:  [5.0390 Melem/s 5.0985 Melem/s 5.1546 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) low severe
  1 (1.00%) low mild
Benchmarking local_graph/on_pingwave_duplicate: Collecting 100 samples in estimated 5.0000 s (312M local_graph/on_pingwave_duplicate
                        time:   [15.849 ns 15.967 ns 16.100 ns]
                        thrpt:  [62.112 Melem/s 62.630 Melem/s 63.095 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
local_graph/get_node    time:   [15.470 ns 15.568 ns 15.677 ns]
                        thrpt:  [63.790 Melem/s 64.236 Melem/s 64.642 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
local_graph/node_count  time:   [974.40 ns 979.30 ns 985.06 ns]
                        thrpt:  [1.0152 Melem/s 1.0211 Melem/s 1.0263 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
local_graph/stats       time:   [2.9299 µs 2.9408 µs 2.9527 µs]
                        thrpt:  [338.67 Kelem/s 340.05 Kelem/s 341.31 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking graph_scaling/all_nodes/100: Collecting 100 samples in estimated 5.0236 s (641k iteratgraph_scaling/all_nodes/100
                        time:   [7.6650 µs 7.7085 µs 7.7590 µs]
                        thrpt:  [12.888 Melem/s 12.973 Melem/s 13.046 Melem/s]
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/100: Collecting 100 samples in estimated 5.0313 s (646graph_scaling/nodes_within_hops/100
                        time:   [7.7334 µs 7.7721 µs 7.8155 µs]
                        thrpt:  [12.795 Melem/s 12.867 Melem/s 12.931 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking graph_scaling/all_nodes/500: Collecting 100 samples in estimated 5.0752 s (288k iteratgraph_scaling/all_nodes/500
                        time:   [17.027 µs 17.100 µs 17.180 µs]
                        thrpt:  [29.103 Melem/s 29.240 Melem/s 29.365 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/500: Collecting 100 samples in estimated 5.0057 s (303graph_scaling/nodes_within_hops/500
                        time:   [16.546 µs 16.634 µs 16.730 µs]
                        thrpt:  [29.886 Melem/s 30.060 Melem/s 30.218 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking graph_scaling/all_nodes/1000: Collecting 100 samples in estimated 5.1256 s (182k iteragraph_scaling/all_nodes/1000
                        time:   [27.718 µs 27.881 µs 28.049 µs]
                        thrpt:  [35.651 Melem/s 35.866 Melem/s 36.078 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/1000: Collecting 100 samples in estimated 5.0945 s (18graph_scaling/nodes_within_hops/1000
                        time:   [27.838 µs 28.009 µs 28.199 µs]
                        thrpt:  [35.462 Melem/s 35.702 Melem/s 35.922 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking graph_scaling/all_nodes/5000: Collecting 100 samples in estimated 5.8699 s (25k iteratgraph_scaling/all_nodes/5000
                        time:   [228.04 µs 229.45 µs 230.93 µs]
                        thrpt:  [21.652 Melem/s 21.791 Melem/s 21.926 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking graph_scaling/nodes_within_hops/5000: Collecting 100 samples in estimated 5.6617 s (25graph_scaling/nodes_within_hops/5000
                        time:   [224.93 µs 226.25 µs 227.65 µs]
                        thrpt:  [21.964 Melem/s 22.100 Melem/s 22.229 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking capability_search/find_with_gpu: Collecting 100 samples in estimated 5.0083 s (157k itcapability_search/find_with_gpu
                        time:   [31.515 µs 31.728 µs 31.961 µs]
                        thrpt:  [31.288 Kelem/s 31.518 Kelem/s 31.731 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking capability_search/find_by_tool_python: Collecting 100 samples in estimated 5.1229 s (8capability_search/find_by_tool_python
                        time:   [62.511 µs 62.902 µs 63.331 µs]
                        thrpt:  [15.790 Kelem/s 15.898 Kelem/s 15.997 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_search/find_by_tool_rust: Collecting 100 samples in estimated 5.3190 s (71kcapability_search/find_by_tool_rust
                        time:   [74.597 µs 74.991 µs 75.423 µs]
                        thrpt:  [13.259 Kelem/s 13.335 Kelem/s 13.405 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

Benchmarking graph_concurrent/concurrent_pingwave/4: Collecting 20 samples in estimated 5.0019 s (2graph_concurrent/concurrent_pingwave/4
                        time:   [210.68 µs 219.60 µs 230.14 µs]
                        thrpt:  [8.6905 Melem/s 9.1075 Melem/s 9.4930 Melem/s]
Benchmarking graph_concurrent/concurrent_pingwave/8: Collecting 20 samples in estimated 5.0144 s (1graph_concurrent/concurrent_pingwave/8
                        time:   [359.44 µs 370.16 µs 378.99 µs]
                        thrpt:  [10.555 Melem/s 10.806 Melem/s 11.128 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking graph_concurrent/concurrent_pingwave/16: Collecting 20 samples in estimated 5.0615 s (graph_concurrent/concurrent_pingwave/16
                        time:   [651.19 µs 668.69 µs 684.36 µs]
                        thrpt:  [11.690 Melem/s 11.964 Melem/s 12.285 Melem/s]

Benchmarking path_finding/path_1_hop: Collecting 100 samples in estimated 5.0164 s (858k iterationspath_finding/path_1_hop time:   [5.8495 µs 5.8981 µs 5.9522 µs]
                        thrpt:  [168.01 Kelem/s 169.55 Kelem/s 170.95 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking path_finding/path_2_hops: Collecting 100 samples in estimated 5.0246 s (848k iterationpath_finding/path_2_hops
                        time:   [5.8283 µs 5.8644 µs 5.9033 µs]
                        thrpt:  [169.40 Kelem/s 170.52 Kelem/s 171.58 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking path_finding/path_4_hops: Collecting 100 samples in estimated 5.0160 s (803k iterationpath_finding/path_4_hops
                        time:   [6.1036 µs 6.1469 µs 6.1977 µs]
                        thrpt:  [161.35 Kelem/s 162.68 Kelem/s 163.84 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking path_finding/path_not_found: Collecting 100 samples in estimated 5.0172 s (813k iteratpath_finding/path_not_found
                        time:   [6.0794 µs 6.1103 µs 6.1433 µs]
                        thrpt:  [162.78 Kelem/s 163.66 Kelem/s 164.49 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking path_finding/path_complex_graph: Collecting 100 samples in estimated 6.2097 s (20k itepath_finding/path_complex_graph
                        time:   [303.19 µs 304.93 µs 307.07 µs]
                        thrpt:  [3.2566 Kelem/s 3.2795 Kelem/s 3.2982 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking failure_detector/heartbeat_existing: Collecting 100 samples in estimated 5.0001 s (141failure_detector/heartbeat_existing
                        time:   [35.639 ns 35.787 ns 35.941 ns]
                        thrpt:  [27.823 Melem/s 27.943 Melem/s 28.059 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking failure_detector/heartbeat_new: Collecting 100 samples in estimated 5.0003 s (26M iterfailure_detector/heartbeat_new
                        time:   [240.88 ns 243.16 ns 245.57 ns]
                        thrpt:  [4.0721 Melem/s 4.1125 Melem/s 4.1514 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) low mild
  3 (3.00%) high mild
Benchmarking failure_detector/status_check: Collecting 100 samples in estimated 5.0000 s (373M iterfailure_detector/status_check
                        time:   [13.370 ns 13.451 ns 13.538 ns]
                        thrpt:  [73.868 Melem/s 74.342 Melem/s 74.793 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_detector/check_all: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 34.5s, or reduce sample count to 10.
Benchmarking failure_detector/check_all: Collecting 100 samples in estimated 34.515 s (100 iteratiofailure_detector/check_all
                        time:   [338.36 ms 340.07 ms 341.89 ms]
                        thrpt:  [2.9249  elem/s 2.9405  elem/s 2.9554  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking failure_detector/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 10.2s, or reduce sample count to 40.
failure_detector/stats  time:   [102.73 ms 103.14 ms 103.58 ms]
                        thrpt:  [9.6547  elem/s 9.6957  elem/s 9.7342  elem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking loss_simulator/should_drop_1pct: Collecting 100 samples in estimated 5.0000 s (955M itloss_simulator/should_drop_1pct
                        time:   [5.1725 ns 5.1992 ns 5.2305 ns]
                        thrpt:  [191.19 Melem/s 192.34 Melem/s 193.33 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_5pct: Collecting 100 samples in estimated 5.0000 s (930M itloss_simulator/should_drop_5pct
                        time:   [5.3324 ns 5.3557 ns 5.3848 ns]
                        thrpt:  [185.71 Melem/s 186.72 Melem/s 187.53 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_10pct: Collecting 100 samples in estimated 5.0000 s (886M iloss_simulator/should_drop_10pct
                        time:   [5.5613 ns 5.5911 ns 5.6245 ns]
                        thrpt:  [177.79 Melem/s 178.85 Melem/s 179.81 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_20pct: Collecting 100 samples in estimated 5.0000 s (826M iloss_simulator/should_drop_20pct
                        time:   [6.0283 ns 6.0622 ns 6.0986 ns]
                        thrpt:  [163.97 Melem/s 164.96 Melem/s 165.88 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking loss_simulator/should_drop_burst: Collecting 100 samples in estimated 5.0000 s (797M iloss_simulator/should_drop_burst
                        time:   [6.2504 ns 6.2800 ns 6.3122 ns]
                        thrpt:  [158.42 Melem/s 159.23 Melem/s 159.99 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking circuit_breaker/allow_closed: Collecting 100 samples in estimated 5.0000 s (475M iteracircuit_breaker/allow_closed
                        time:   [10.542 ns 10.598 ns 10.653 ns]
                        thrpt:  [93.868 Melem/s 94.361 Melem/s 94.858 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking circuit_breaker/record_success: Collecting 100 samples in estimated 5.0000 s (474M itecircuit_breaker/record_success
                        time:   [10.505 ns 10.556 ns 10.617 ns]
                        thrpt:  [94.192 Melem/s 94.735 Melem/s 95.197 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking circuit_breaker/record_failure: Collecting 100 samples in estimated 5.0000 s (471M itecircuit_breaker/record_failure
                        time:   [10.498 ns 10.542 ns 10.591 ns]
                        thrpt:  [94.420 Melem/s 94.859 Melem/s 95.253 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
circuit_breaker/state   time:   [10.517 ns 10.564 ns 10.615 ns]
                        thrpt:  [94.207 Melem/s 94.661 Melem/s 95.084 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

Benchmarking recovery_manager/on_failure_with_alternates: Collecting 100 samples in estimated 5.000recovery_manager/on_failure_with_alternates
                        time:   [312.65 ns 314.80 ns 317.01 ns]
                        thrpt:  [3.1545 Melem/s 3.1766 Melem/s 3.1985 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high mild
Benchmarking recovery_manager/on_failure_no_alternates: Collecting 100 samples in estimated 5.0004 recovery_manager/on_failure_no_alternates
                        time:   [273.36 ns 286.82 ns 312.73 ns]
                        thrpt:  [3.1976 Melem/s 3.4866 Melem/s 3.6582 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) low severe
  2 (2.00%) low mild
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking recovery_manager/get_action: Collecting 100 samples in estimated 5.0001 s (125M iteratrecovery_manager/get_action
                        time:   [38.882 ns 39.079 ns 39.282 ns]
                        thrpt:  [25.457 Melem/s 25.589 Melem/s 25.719 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking recovery_manager/is_failed: Collecting 100 samples in estimated 5.0001 s (372M iteratirecovery_manager/is_failed
                        time:   [13.319 ns 13.421 ns 13.538 ns]
                        thrpt:  [73.867 Melem/s 74.512 Melem/s 75.080 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking recovery_manager/on_recovery: Collecting 100 samples in estimated 5.0003 s (37M iteratrecovery_manager/on_recovery
                        time:   [128.06 ns 128.67 ns 129.35 ns]
                        thrpt:  [7.7312 Melem/s 7.7721 Melem/s 7.8088 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
recovery_manager/stats  time:   [1.2226 ns 1.2261 ns 1.2300 ns]
                        thrpt:  [813.03 Melem/s 815.56 Melem/s 817.94 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

Benchmarking failure_scaling/check_all/100: Collecting 100 samples in estimated 5.0325 s (571k iterfailure_scaling/check_all/100
                        time:   [9.0431 µs 9.0881 µs 9.1387 µs]
                        thrpt:  [10.943 Melem/s 11.003 Melem/s 11.058 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking failure_scaling/healthy_nodes/100: Collecting 100 samples in estimated 5.0155 s (778k failure_scaling/healthy_nodes/100
                        time:   [6.4102 µs 6.4434 µs 6.4799 µs]
                        thrpt:  [15.432 Melem/s 15.520 Melem/s 15.600 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking failure_scaling/check_all/500: Collecting 100 samples in estimated 5.0741 s (232k iterfailure_scaling/check_all/500
                        time:   [23.517 µs 23.604 µs 23.699 µs]
                        thrpt:  [21.098 Melem/s 21.183 Melem/s 21.261 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking failure_scaling/healthy_nodes/500: Collecting 100 samples in estimated 5.0271 s (556k failure_scaling/healthy_nodes/500
                        time:   [9.1086 µs 9.1803 µs 9.2634 µs]
                        thrpt:  [53.976 Melem/s 54.465 Melem/s 54.893 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking failure_scaling/check_all/1000: Collecting 100 samples in estimated 5.1637 s (136k itefailure_scaling/check_all/1000
                        time:   [41.747 µs 41.914 µs 42.090 µs]
                        thrpt:  [23.759 Melem/s 23.858 Melem/s 23.954 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) low severe
  3 (3.00%) low mild
  3 (3.00%) high severe
Benchmarking failure_scaling/healthy_nodes/1000: Collecting 100 samples in estimated 5.0156 s (389kfailure_scaling/healthy_nodes/1000
                        time:   [12.854 µs 12.931 µs 13.018 µs]
                        thrpt:  [76.818 Melem/s 77.332 Melem/s 77.798 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking failure_scaling/check_all/5000: Collecting 100 samples in estimated 5.1058 s (30k iterfailure_scaling/check_all/5000
                        time:   [186.56 µs 187.33 µs 188.06 µs]
                        thrpt:  [26.587 Melem/s 26.691 Melem/s 26.801 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking failure_scaling/healthy_nodes/5000: Collecting 100 samples in estimated 5.0454 s (116kfailure_scaling/healthy_nodes/5000
                        time:   [42.783 µs 42.962 µs 43.158 µs]
                        thrpt:  [115.85 Melem/s 116.38 Melem/s 116.87 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking failure_concurrent/concurrent_heartbeat/4: Collecting 20 samples in estimated 5.0463 sfailure_concurrent/concurrent_heartbeat/4
                        time:   [255.40 µs 265.12 µs 274.60 µs]
                        thrpt:  [7.2833 Melem/s 7.5436 Melem/s 7.8309 Melem/s]
Benchmarking failure_concurrent/concurrent_heartbeat/8: Collecting 20 samples in estimated 5.0523 sfailure_concurrent/concurrent_heartbeat/8
                        time:   [401.18 µs 408.56 µs 416.93 µs]
                        thrpt:  [9.5940 Melem/s 9.7904 Melem/s 9.9706 Melem/s]
Benchmarking failure_concurrent/concurrent_heartbeat/16: Collecting 20 samples in estimated 5.0364 failure_concurrent/concurrent_heartbeat/16
                        time:   [699.91 µs 711.70 µs 723.00 µs]
                        thrpt:  [11.065 Melem/s 11.241 Melem/s 11.430 Melem/s]
Found 3 outliers among 20 measurements (15.00%)
  2 (10.00%) low mild
  1 (5.00%) high mild

Benchmarking failure_recovery_cycle/full_cycle: Collecting 100 samples in estimated 5.0007 s (16M ifailure_recovery_cycle/full_cycle
                        time:   [321.39 ns 323.40 ns 325.47 ns]
                        thrpt:  [3.0725 Melem/s 3.0922 Melem/s 3.1115 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) low severe
  1 (1.00%) low mild

capability_set/create   time:   [797.57 ns 806.28 ns 816.01 ns]
                        thrpt:  [1.2255 Melem/s 1.2403 Melem/s 1.2538 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_set/serialize: Collecting 100 samples in estimated 5.0005 s (7.0M iterationcapability_set/serialize
                        time:   [713.96 ns 721.65 ns 730.29 ns]
                        thrpt:  [1.3693 Melem/s 1.3857 Melem/s 1.4006 Melem/s]
Found 13 outliers among 100 measurements (13.00%)
  6 (6.00%) high mild
  7 (7.00%) high severe
Benchmarking capability_set/deserialize: Collecting 100 samples in estimated 5.0138 s (1.6M iteraticapability_set/deserialize
                        time:   [3.1273 µs 3.1463 µs 3.1662 µs]
                        thrpt:  [315.84 Kelem/s 317.84 Kelem/s 319.76 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_set/roundtrip: Collecting 100 samples in estimated 5.0125 s (1.2M iterationcapability_set/roundtrip
                        time:   [4.0243 µs 4.0612 µs 4.1065 µs]
                        thrpt:  [243.51 Kelem/s 246.23 Kelem/s 248.49 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
capability_set/has_tag  time:   [622.24 ps 626.71 ps 631.53 ps]
                        thrpt:  [1.5835 Gelem/s 1.5956 Gelem/s 1.6071 Gelem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high mild
Benchmarking capability_set/has_model: Collecting 100 samples in estimated 5.0000 s (11B iterationscapability_set/has_model
                        time:   [437.69 ps 441.79 ps 446.12 ps]
                        thrpt:  [2.2416 Gelem/s 2.2635 Gelem/s 2.2847 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild
Benchmarking capability_set/has_tool: Collecting 100 samples in estimated 5.0000 s (7.9B iterationscapability_set/has_tool time:   [629.35 ps 635.13 ps 641.40 ps]
                        thrpt:  [1.5591 Gelem/s 1.5745 Gelem/s 1.5889 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
capability_set/has_gpu  time:   [202.64 ps 203.16 ps 203.73 ps]
                        thrpt:  [4.9085 Gelem/s 4.9223 Gelem/s 4.9349 Gelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking capability_announcement/create: Collecting 100 samples in estimated 5.0018 s (3.2M itecapability_announcement/create
                        time:   [1.5566 µs 1.5682 µs 1.5810 µs]
                        thrpt:  [632.52 Kelem/s 637.66 Kelem/s 642.43 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_announcement/serialize: Collecting 100 samples in estimated 5.0015 s (6.0M capability_announcement/serialize
                        time:   [821.68 ns 832.23 ns 843.87 ns]
                        thrpt:  [1.1850 Melem/s 1.2016 Melem/s 1.2170 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_announcement/deserialize: Collecting 100 samples in estimated 5.0052 s (2.0capability_announcement/deserialize
                        time:   [2.4468 µs 2.4731 µs 2.5013 µs]
                        thrpt:  [399.79 Kelem/s 404.36 Kelem/s 408.69 Kelem/s]
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
Benchmarking capability_announcement/is_expired: Collecting 100 samples in estimated 5.0001 s (222Mcapability_announcement/is_expired
                        time:   [22.426 ns 22.463 ns 22.502 ns]
                        thrpt:  [44.441 Melem/s 44.519 Melem/s 44.592 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_filter/match_single_tag: Collecting 100 samples in estimated 5.0000 s (1.3Bcapability_filter/match_single_tag
                        time:   [3.6063 ns 3.6578 ns 3.7139 ns]
                        thrpt:  [269.26 Melem/s 273.39 Melem/s 277.29 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_filter/match_require_gpu: Collecting 100 samples in estimated 5.0000 s (2.6capability_filter/match_require_gpu
                        time:   [1.9136 ns 1.9383 ns 1.9664 ns]
                        thrpt:  [508.54 Melem/s 515.92 Melem/s 522.58 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_gpu_vendor: Collecting 100 samples in estimated 5.0000 s (2.3Bcapability_filter/match_gpu_vendor
                        time:   [2.1467 ns 2.1797 ns 2.2159 ns]
                        thrpt:  [451.29 Melem/s 458.78 Melem/s 465.83 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_min_memory: Collecting 100 samples in estimated 5.0000 s (2.6Bcapability_filter/match_min_memory
                        time:   [1.9163 ns 1.9416 ns 1.9691 ns]
                        thrpt:  [507.84 Melem/s 515.05 Melem/s 521.84 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_filter/match_complex: Collecting 100 samples in estimated 5.0000 s (811M itcapability_filter/match_complex
                        time:   [6.0869 ns 6.1297 ns 6.1743 ns]
                        thrpt:  [161.96 Melem/s 163.14 Melem/s 164.29 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_filter/match_no_match: Collecting 100 samples in estimated 5.0000 s (2.6B icapability_filter/match_no_match
                        time:   [1.8593 ns 1.8707 ns 1.8831 ns]
                        thrpt:  [531.04 Melem/s 534.55 Melem/s 537.84 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking capability_index_insert/index_nodes/100: Collecting 100 samples in estimated 5.8276 s capability_index_insert/index_nodes/100
                        time:   [190.18 µs 191.49 µs 192.93 µs]
                        thrpt:  [518.34 Kelem/s 522.21 Kelem/s 525.81 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_insert/index_nodes/1000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 7.9s, enable flat sampling, or reduce sample count to 50.
Benchmarking capability_index_insert/index_nodes/1000: Collecting 100 samples in estimated 7.9287 scapability_index_insert/index_nodes/1000
                        time:   [1.5637 ms 1.5759 ms 1.5887 ms]
                        thrpt:  [629.45 Kelem/s 634.54 Kelem/s 639.50 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_insert/index_nodes/10000: Collecting 100 samples in estimated 6.0096 capability_index_insert/index_nodes/10000
                        time:   [19.841 ms 20.038 ms 20.247 ms]
                        thrpt:  [493.90 Kelem/s 499.06 Kelem/s 504.01 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe

Benchmarking capability_index_query/query_single_tag: Collecting 100 samples in estimated 5.7518 s capability_index_query/query_single_tag
                        time:   [188.21 µs 190.15 µs 192.19 µs]
                        thrpt:  [5.2032 Kelem/s 5.2591 Kelem/s 5.3131 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_require_gpu: Collecting 100 samples in estimated 5.1631 scapability_index_query/query_require_gpu
                        time:   [207.13 µs 209.24 µs 211.38 µs]
                        thrpt:  [4.7309 Kelem/s 4.7791 Kelem/s 4.8278 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking capability_index_query/query_gpu_vendor: Collecting 100 samples in estimated 5.6225 s capability_index_query/query_gpu_vendor
                        time:   [551.90 µs 556.54 µs 561.16 µs]
                        thrpt:  [1.7820 Kelem/s 1.7968 Kelem/s 1.8119 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_min_memory: Collecting 100 samples in estimated 5.4275 s capability_index_query/query_min_memory
                        time:   [540.04 µs 544.70 µs 549.78 µs]
                        thrpt:  [1.8189 Kelem/s 1.8359 Kelem/s 1.8517 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking capability_index_query/query_complex: Collecting 100 samples in estimated 5.3925 s (15capability_index_query/query_complex
                        time:   [355.32 µs 358.56 µs 362.05 µs]
                        thrpt:  [2.7620 Kelem/s 2.7889 Kelem/s 2.8144 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_model: Collecting 100 samples in estimated 5.2745 s (50k capability_index_query/query_model
                        time:   [103.84 µs 104.98 µs 106.27 µs]
                        thrpt:  [9.4103 Kelem/s 9.5254 Kelem/s 9.6303 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_index_query/query_tool: Collecting 100 samples in estimated 5.4227 s (10k icapability_index_query/query_tool
                        time:   [530.27 µs 537.51 µs 545.42 µs]
                        thrpt:  [1.8334 Kelem/s 1.8604 Kelem/s 1.8858 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_query/query_no_results: Collecting 100 samples in estimated 5.0001 s capability_index_query/query_no_results
                        time:   [26.650 ns 26.752 ns 26.866 ns]
                        thrpt:  [37.222 Melem/s 37.381 Melem/s 37.523 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  5 (5.00%) high mild
  5 (5.00%) high severe

Benchmarking capability_index_find_best/find_best_simple: Collecting 100 samples in estimated 5.874capability_index_find_best/find_best_simple
                        time:   [373.46 µs 376.63 µs 380.04 µs]
                        thrpt:  [2.6313 Kelem/s 2.6551 Kelem/s 2.6777 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_find_best/find_best_with_prefs: Collecting 100 samples in estimated 5capability_index_find_best/find_best_with_prefs
                        time:   [578.04 µs 582.87 µs 588.14 µs]
                        thrpt:  [1.7003 Kelem/s 1.7157 Kelem/s 1.7300 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe

Benchmarking capability_index_scaling/query_tag/1000: Collecting 100 samples in estimated 5.0073 s capability_index_scaling/query_tag/1000
                        time:   [10.782 µs 10.841 µs 10.905 µs]
                        thrpt:  [91.702 Kelem/s 92.243 Kelem/s 92.748 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
Benchmarking capability_index_scaling/query_complex/1000: Collecting 100 samples in estimated 5.056capability_index_scaling/query_complex/1000
                        time:   [28.747 µs 28.963 µs 29.204 µs]
                        thrpt:  [34.242 Kelem/s 34.527 Kelem/s 34.786 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_scaling/query_tag/5000: Collecting 100 samples in estimated 5.1000 s capability_index_scaling/query_tag/5000
                        time:   [58.057 µs 58.625 µs 59.267 µs]
                        thrpt:  [16.873 Kelem/s 17.057 Kelem/s 17.225 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_index_scaling/query_complex/5000: Collecting 100 samples in estimated 5.188capability_index_scaling/query_complex/5000
                        time:   [143.38 µs 144.55 µs 145.74 µs]
                        thrpt:  [6.8618 Kelem/s 6.9178 Kelem/s 6.9746 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_tag/10000: Collecting 100 samples in estimated 5.7135 scapability_index_scaling/query_tag/10000
                        time:   [186.45 µs 188.14 µs 189.91 µs]
                        thrpt:  [5.2655 Kelem/s 5.3152 Kelem/s 5.3634 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_index_scaling/query_complex/10000: Collecting 100 samples in estimated 5.57capability_index_scaling/query_complex/10000
                        time:   [360.49 µs 363.66 µs 367.12 µs]
                        thrpt:  [2.7239 Kelem/s 2.7498 Kelem/s 2.7740 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking capability_index_scaling/query_tag/50000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 9.4s, enable flat sampling, or reduce sample count to 50.
Benchmarking capability_index_scaling/query_tag/50000: Collecting 100 samples in estimated 9.4395 scapability_index_scaling/query_tag/50000
                        time:   [1.7914 ms 1.8541 ms 1.9203 ms]
                        thrpt:  [520.74  elem/s 539.34  elem/s 558.23  elem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_scaling/query_complex/50000: Collecting 100 samples in estimated 5.25capability_index_scaling/query_complex/50000
                        time:   [3.0491 ms 3.1590 ms 3.2677 ms]
                        thrpt:  [306.03  elem/s 316.56  elem/s 327.97  elem/s]

Benchmarking capability_index_concurrent/concurrent_index/4: Collecting 20 samples in estimated 5.0capability_index_concurrent/concurrent_index/4
                        time:   [671.98 µs 694.33 µs 723.64 µs]
                        thrpt:  [2.7638 Melem/s 2.8805 Melem/s 2.9763 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.6s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/4: Collecting 20 samples in estimated 5.5capability_index_concurrent/concurrent_query/4
                        time:   [266.46 ms 270.68 ms 274.61 ms]
                        thrpt:  [7.2830 Kelem/s 7.3888 Kelem/s 7.5057 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 5.3capability_index_concurrent/concurrent_mixed/4
                        time:   [129.80 ms 131.88 ms 134.11 ms]
                        thrpt:  [14.913 Kelem/s 15.165 Kelem/s 15.409 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking capability_index_concurrent/concurrent_index/8: Collecting 20 samples in estimated 5.1capability_index_concurrent/concurrent_index/8
                        time:   [1.4220 ms 1.4484 ms 1.4797 ms]
                        thrpt:  [2.7032 Melem/s 2.7617 Melem/s 2.8129 Melem/s]
Benchmarking capability_index_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.1s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/8: Collecting 20 samples in estimated 7.0capability_index_concurrent/concurrent_query/8
                        time:   [343.69 ms 347.95 ms 352.29 ms]
                        thrpt:  [11.354 Kelem/s 11.496 Kelem/s 11.639 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 7.1capability_index_concurrent/concurrent_mixed/8
                        time:   [176.32 ms 179.22 ms 182.08 ms]
                        thrpt:  [21.969 Kelem/s 22.318 Kelem/s 22.686 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_index/16: Collecting 20 samples in estimated 5.capability_index_concurrent/concurrent_index/16
                        time:   [1.9287 ms 1.9625 ms 1.9954 ms]
                        thrpt:  [4.0092 Melem/s 4.0765 Melem/s 4.1478 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking capability_index_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.5s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_query/16: Collecting 20 samples in estimated 9.capability_index_concurrent/concurrent_query/16
                        time:   [462.16 ms 465.83 ms 469.31 ms]
                        thrpt:  [17.046 Kelem/s 17.174 Kelem/s 17.310 Kelem/s]
Benchmarking capability_index_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.2s, or reduce sample count to 10.
Benchmarking capability_index_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 5.capability_index_concurrent/concurrent_mixed/16
                        time:   [257.29 ms 259.24 ms 261.17 ms]
                        thrpt:  [30.631 Kelem/s 30.860 Kelem/s 31.093 Kelem/s]

Benchmarking capability_index_updates/update_higher_version: Collecting 100 samples in estimated 5.capability_index_updates/update_higher_version
                        time:   [837.84 ns 845.01 ns 852.78 ns]
                        thrpt:  [1.1726 Melem/s 1.1834 Melem/s 1.1935 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking capability_index_updates/update_same_version: Collecting 100 samples in estimated 5.00capability_index_updates/update_same_version
                        time:   [843.69 ns 848.83 ns 854.45 ns]
                        thrpt:  [1.1703 Melem/s 1.1781 Melem/s 1.1853 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_index_updates/remove_and_readd: Collecting 100 samples in estimated 5.0067 capability_index_updates/remove_and_readd
                        time:   [1.3846 µs 1.3984 µs 1.4138 µs]
                        thrpt:  [707.29 Kelem/s 715.10 Kelem/s 722.22 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

diff_op/create_add_tag  time:   [26.073 ns 26.275 ns 26.513 ns]
                        thrpt:  [37.717 Melem/s 38.059 Melem/s 38.354 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  7 (7.00%) high mild
  5 (5.00%) high severe
Benchmarking diff_op/create_remove_tag: Collecting 100 samples in estimated 5.0000 s (187M iteratiodiff_op/create_remove_tag
                        time:   [26.159 ns 26.449 ns 26.764 ns]
                        thrpt:  [37.364 Melem/s 37.809 Melem/s 38.228 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_op/create_add_model: Collecting 100 samples in estimated 5.0004 s (54M iterationsdiff_op/create_add_model
                        time:   [91.118 ns 91.777 ns 92.513 ns]
                        thrpt:  [10.809 Melem/s 10.896 Melem/s 10.975 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_op/create_update_model: Collecting 100 samples in estimated 5.0000 s (185M iteratdiff_op/create_update_model
                        time:   [26.205 ns 26.424 ns 26.646 ns]
                        thrpt:  [37.529 Melem/s 37.845 Melem/s 38.161 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
diff_op/estimated_size  time:   [1.0345 ns 1.0409 ns 1.0476 ns]
                        thrpt:  [954.57 Melem/s 960.72 Melem/s 966.66 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

capability_diff/create  time:   [88.090 ns 88.707 ns 89.384 ns]
                        thrpt:  [11.188 Melem/s 11.273 Melem/s 11.352 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking capability_diff/serialize: Collecting 100 samples in estimated 5.0001 s (35M iterationcapability_diff/serialize
                        time:   [141.17 ns 142.40 ns 143.75 ns]
                        thrpt:  [6.9564 Melem/s 7.0227 Melem/s 7.0837 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe
Benchmarking capability_diff/deserialize: Collecting 100 samples in estimated 5.0001 s (20M iteraticapability_diff/deserialize
                        time:   [247.30 ns 249.72 ns 252.36 ns]
                        thrpt:  [3.9627 Melem/s 4.0045 Melem/s 4.0437 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking capability_diff/estimated_size: Collecting 100 samples in estimated 5.0000 s (2.0B itecapability_diff/estimated_size
                        time:   [2.5010 ns 2.5228 ns 2.5492 ns]
                        thrpt:  [392.28 Melem/s 396.38 Melem/s 399.84 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe

Benchmarking diff_generation/no_changes: Collecting 100 samples in estimated 5.0008 s (17M iteratiodiff_generation/no_changes
                        time:   [303.91 ns 308.42 ns 313.91 ns]
                        thrpt:  [3.1856 Melem/s 3.2424 Melem/s 3.2905 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking diff_generation/add_one_tag: Collecting 100 samples in estimated 5.0016 s (12M iteratidiff_generation/add_one_tag
                        time:   [414.49 ns 419.54 ns 424.71 ns]
                        thrpt:  [2.3546 Melem/s 2.3835 Melem/s 2.4126 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_generation/multiple_tag_changes: Collecting 100 samples in estimated 5.0019 s (11diff_generation/multiple_tag_changes
                        time:   [466.54 ns 471.30 ns 476.88 ns]
                        thrpt:  [2.0969 Melem/s 2.1218 Melem/s 2.1435 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_generation/update_model_loaded: Collecting 100 samples in estimated 5.0018 s (14Mdiff_generation/update_model_loaded
                        time:   [357.94 ns 361.61 ns 365.74 ns]
                        thrpt:  [2.7342 Melem/s 2.7654 Melem/s 2.7938 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  4 (4.00%) high mild
  6 (6.00%) high severe
Benchmarking diff_generation/update_memory: Collecting 100 samples in estimated 5.0012 s (15M iteradiff_generation/update_memory
                        time:   [333.79 ns 337.83 ns 342.33 ns]
                        thrpt:  [2.9212 Melem/s 2.9600 Melem/s 2.9959 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_generation/add_model: Collecting 100 samples in estimated 5.0012 s (11M iterationdiff_generation/add_model
                        time:   [458.87 ns 463.43 ns 468.02 ns]
                        thrpt:  [2.1366 Melem/s 2.1578 Melem/s 2.1793 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_generation/complex_diff: Collecting 100 samples in estimated 5.0042 s (6.0M iteradiff_generation/complex_diff
                        time:   [822.51 ns 830.50 ns 839.25 ns]
                        thrpt:  [1.1915 Melem/s 1.2041 Melem/s 1.2158 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking diff_application/apply_single_op: Collecting 100 samples in estimated 5.0021 s (7.5M idiff_application/apply_single_op
                        time:   [651.10 ns 656.54 ns 662.66 ns]
                        thrpt:  [1.5091 Melem/s 1.5231 Melem/s 1.5359 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_application/apply_small_diff: Collecting 100 samples in estimated 5.0014 s (7.5M diff_application/apply_small_diff
                        time:   [661.62 ns 668.05 ns 674.90 ns]
                        thrpt:  [1.4817 Melem/s 1.4969 Melem/s 1.5114 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking diff_application/apply_medium_diff: Collecting 100 samples in estimated 5.0099 s (2.4Mdiff_application/apply_medium_diff
                        time:   [2.0882 µs 2.1020 µs 2.1168 µs]
                        thrpt:  [472.41 Kelem/s 475.73 Kelem/s 478.89 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking diff_application/apply_strict_mode: Collecting 100 samples in estimated 5.0008 s (2.1Mdiff_application/apply_strict_mode
                        time:   [2.3633 µs 2.3727 µs 2.3830 µs]
                        thrpt:  [419.64 Kelem/s 421.46 Kelem/s 423.14 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  6 (6.00%) high mild

Benchmarking diff_chain_validation/validate_chain_10: Collecting 100 samples in estimated 5.0000 s diff_chain_validation/validate_chain_10
                        time:   [4.4519 ns 4.4975 ns 4.5452 ns]
                        thrpt:  [220.01 Melem/s 222.34 Melem/s 224.62 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
Benchmarking diff_chain_validation/validate_chain_100: Collecting 100 samples in estimated 5.0002 sdiff_chain_validation/validate_chain_100
                        time:   [51.108 ns 51.453 ns 51.818 ns]
                        thrpt:  [19.298 Melem/s 19.435 Melem/s 19.566 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low severe
  1 (1.00%) high mild
  2 (2.00%) high severe

Benchmarking diff_compaction/compact_5_diffs: Collecting 100 samples in estimated 5.0331 s (631k itdiff_compaction/compact_5_diffs
                        time:   [7.9692 µs 8.0350 µs 8.1063 µs]
                        thrpt:  [123.36 Kelem/s 124.46 Kelem/s 125.48 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking diff_compaction/compact_20_diffs: Collecting 100 samples in estimated 5.1220 s (146k idiff_compaction/compact_20_diffs
                        time:   [34.156 µs 34.339 µs 34.548 µs]
                        thrpt:  [28.945 Kelem/s 29.122 Kelem/s 29.277 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking bandwidth_savings/calculate_small: Collecting 100 samples in estimated 5.0006 s (7.1M bandwidth_savings/calculate_small
                        time:   [702.93 ns 709.98 ns 717.88 ns]
                        thrpt:  [1.3930 Melem/s 1.4085 Melem/s 1.4226 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking bandwidth_savings/calculate_medium: Collecting 100 samples in estimated 5.0032 s (7.0Mbandwidth_savings/calculate_medium
                        time:   [700.27 ns 706.51 ns 713.62 ns]
                        thrpt:  [1.4013 Melem/s 1.4154 Melem/s 1.4280 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

Benchmarking diff_roundtrip/generate_apply_verify: Collecting 100 samples in estimated 5.0057 s (2.diff_roundtrip/generate_apply_verify
                        time:   [2.4495 µs 2.4680 µs 2.4871 µs]
                        thrpt:  [402.08 Kelem/s 405.19 Kelem/s 408.25 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe

location_info/create    time:   [58.652 ns 59.528 ns 60.435 ns]
                        thrpt:  [16.547 Melem/s 16.799 Melem/s 17.050 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking location_info/distance_to: Collecting 100 samples in estimated 5.0000 s (1.8B iteratiolocation_info/distance_to
                        time:   [2.7748 ns 2.8042 ns 2.8361 ns]
                        thrpt:  [352.60 Melem/s 356.61 Melem/s 360.39 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking location_info/same_continent: Collecting 100 samples in estimated 5.0000 s (1.9B iteralocation_info/same_continent
                        time:   [2.6589 ns 2.6785 ns 2.7012 ns]
                        thrpt:  [370.20 Melem/s 373.34 Melem/s 376.09 Melem/s]
Found 12 outliers among 100 measurements (12.00%)
  8 (8.00%) high mild
  4 (4.00%) high severe
Benchmarking location_info/same_continent_cross: Collecting 100 samples in estimated 5.0000 s (24B location_info/same_continent_cross
                        time:   [211.25 ps 212.79 ps 214.41 ps]
                        thrpt:  [4.6640 Gelem/s 4.6994 Gelem/s 4.7337 Gelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking location_info/same_region: Collecting 100 samples in estimated 5.0000 s (2.5B iteratiolocation_info/same_region
                        time:   [2.0306 ns 2.0362 ns 2.0423 ns]
                        thrpt:  [489.66 Melem/s 491.11 Melem/s 492.46 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

topology_hints/create   time:   [3.4928 ns 3.5337 ns 3.5777 ns]
                        thrpt:  [279.51 Melem/s 282.99 Melem/s 286.31 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  8 (8.00%) high mild
  1 (1.00%) high severe
Benchmarking topology_hints/connectivity_score: Collecting 100 samples in estimated 5.0000 s (25B itopology_hints/connectivity_score
                        time:   [201.81 ps 202.08 ps 202.41 ps]
                        thrpt:  [4.9404 Gelem/s 4.9485 Gelem/s 4.9552 Gelem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking topology_hints/average_latency_empty: Collecting 100 samples in estimated 5.0000 s (20topology_hints/average_latency_empty
                        time:   [265.37 ps 279.52 ps 296.38 ps]
                        thrpt:  [3.3741 Gelem/s 3.5776 Gelem/s 3.7684 Gelem/s]
Found 19 outliers among 100 measurements (19.00%)
  19 (19.00%) high severe
Benchmarking topology_hints/average_latency_100: Collecting 100 samples in estimated 5.0002 s (100Mtopology_hints/average_latency_100
                        time:   [49.566 ns 49.937 ns 50.341 ns]
                        thrpt:  [19.864 Melem/s 20.025 Melem/s 20.175 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

nat_type/difficulty     time:   [201.97 ps 202.34 ps 202.78 ps]
                        thrpt:  [4.9316 Gelem/s 4.9421 Gelem/s 4.9511 Gelem/s]
Found 11 outliers among 100 measurements (11.00%)
  4 (4.00%) high mild
  7 (7.00%) high severe
Benchmarking nat_type/can_connect_direct: Collecting 100 samples in estimated 5.0000 s (25B iteratinat_type/can_connect_direct
                        time:   [201.89 ps 202.24 ps 202.62 ps]
                        thrpt:  [4.9354 Gelem/s 4.9447 Gelem/s 4.9533 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking nat_type/can_connect_symmetric: Collecting 100 samples in estimated 5.0000 s (25B iternat_type/can_connect_symmetric
                        time:   [201.72 ps 201.90 ps 202.10 ps]
                        thrpt:  [4.9481 Gelem/s 4.9529 Gelem/s 4.9574 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

Benchmarking node_metadata/create_simple: Collecting 100 samples in estimated 5.0000 s (151M iteratnode_metadata/create_simple
                        time:   [32.878 ns 32.997 ns 33.130 ns]
                        thrpt:  [30.184 Melem/s 30.306 Melem/s 30.416 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  1 (1.00%) high mild
  4 (4.00%) high severe
Benchmarking node_metadata/create_full: Collecting 100 samples in estimated 5.0021 s (12M iterationnode_metadata/create_full
                        time:   [425.91 ns 429.26 ns 432.80 ns]
                        thrpt:  [2.3105 Melem/s 2.3296 Melem/s 2.3479 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  6 (6.00%) high mild
  4 (4.00%) high severe
Benchmarking node_metadata/routing_score: Collecting 100 samples in estimated 5.0000 s (25B iteratinode_metadata/routing_score
                        time:   [202.07 ps 202.43 ps 202.84 ps]
                        thrpt:  [4.9300 Gelem/s 4.9400 Gelem/s 4.9489 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
node_metadata/age       time:   [25.307 ns 25.348 ns 25.396 ns]
                        thrpt:  [39.377 Melem/s 39.451 Melem/s 39.514 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
node_metadata/is_stale  time:   [24.824 ns 24.876 ns 24.930 ns]
                        thrpt:  [40.112 Melem/s 40.200 Melem/s 40.283 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking node_metadata/serialize: Collecting 100 samples in estimated 5.0015 s (8.6M iterationsnode_metadata/serialize time:   [572.75 ns 577.34 ns 582.51 ns]
                        thrpt:  [1.7167 Melem/s 1.7321 Melem/s 1.7459 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking node_metadata/deserialize: Collecting 100 samples in estimated 5.0076 s (3.0M iterationode_metadata/deserialize
                        time:   [1.6336 µs 1.6474 µs 1.6624 µs]
                        thrpt:  [601.56 Kelem/s 607.03 Kelem/s 612.15 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  3 (3.00%) high mild
  4 (4.00%) high severe

Benchmarking metadata_query/match_status: Collecting 100 samples in estimated 5.0000 s (1.8B iteratmetadata_query/match_status
                        time:   [2.6819 ns 2.6979 ns 2.7155 ns]
                        thrpt:  [368.26 Melem/s 370.65 Melem/s 372.87 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_query/match_min_tier: Collecting 100 samples in estimated 5.0000 s (1.8B itermetadata_query/match_min_tier
                        time:   [2.6721 ns 2.6910 ns 2.7131 ns]
                        thrpt:  [368.58 Melem/s 371.61 Melem/s 374.23 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_query/match_continent: Collecting 100 samples in estimated 5.0000 s (1.0B itemetadata_query/match_continent
                        time:   [4.9484 ns 5.0187 ns 5.1001 ns]
                        thrpt:  [196.07 Melem/s 199.25 Melem/s 202.09 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking metadata_query/match_complex: Collecting 100 samples in estimated 5.0000 s (1.0B iterametadata_query/match_complex
                        time:   [4.8989 ns 4.9585 ns 5.0257 ns]
                        thrpt:  [198.98 Melem/s 201.67 Melem/s 204.13 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  1 (1.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_query/match_no_match: Collecting 100 samples in estimated 5.0000 s (3.1B itermetadata_query/match_no_match
                        time:   [1.5640 ns 1.5806 ns 1.5979 ns]
                        thrpt:  [625.84 Melem/s 632.66 Melem/s 639.37 Melem/s]

Benchmarking metadata_store_basic/create: Collecting 100 samples in estimated 5.0068 s (1.8M iteratmetadata_store_basic/create
                        time:   [2.7487 µs 2.7606 µs 2.7745 µs]
                        thrpt:  [360.42 Kelem/s 362.23 Kelem/s 363.81 Kelem/s]
Found 11 outliers among 100 measurements (11.00%)
  8 (8.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/upsert_new: Collecting 100 samples in estimated 5.0007 s (2.9M itmetadata_store_basic/upsert_new
                        time:   [2.0739 µs 2.0905 µs 2.1081 µs]
                        thrpt:  [474.36 Kelem/s 478.36 Kelem/s 482.17 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  2 (2.00%) low mild
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/upsert_existing: Collecting 100 samples in estimated 5.0000 s (4.metadata_store_basic/upsert_existing
                        time:   [1.0331 µs 1.0428 µs 1.0538 µs]
                        thrpt:  [948.92 Kelem/s 958.93 Kelem/s 967.94 Kelem/s]
Found 9 outliers among 100 measurements (9.00%)
  7 (7.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_basic/get: Collecting 100 samples in estimated 5.0000 s (206M iterationmetadata_store_basic/get
                        time:   [24.125 ns 24.300 ns 24.485 ns]
                        thrpt:  [40.841 Melem/s 41.152 Melem/s 41.450 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking metadata_store_basic/get_miss: Collecting 100 samples in estimated 5.0001 s (205M itermetadata_store_basic/get_miss
                        time:   [23.964 ns 24.066 ns 24.182 ns]
                        thrpt:  [41.354 Melem/s 41.552 Melem/s 41.730 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_basic/len: Collecting 100 samples in estimated 5.0041 s (5.1M iterationmetadata_store_basic/len
                        time:   [976.37 ns 982.61 ns 989.53 ns]
                        thrpt:  [1.0106 Melem/s 1.0177 Melem/s 1.0242 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 19.8s, or reduce sample count to 20.
Benchmarking metadata_store_basic/stats: Collecting 100 samples in estimated 19.752 s (100 iteratiometadata_store_basic/stats
                        time:   [196.76 ms 197.76 ms 198.79 ms]
                        thrpt:  [5.0304  elem/s 5.0567  elem/s 5.0824  elem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking metadata_store_query/query_by_status: Collecting 100 samples in estimated 6.2958 s (20metadata_store_query/query_by_status
                        time:   [310.32 µs 312.92 µs 315.64 µs]
                        thrpt:  [3.1682 Kelem/s 3.1957 Kelem/s 3.2224 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_query/query_by_continent: Collecting 100 samples in estimated 5.0711 s metadata_store_query/query_by_continent
                        time:   [163.54 µs 166.43 µs 169.25 µs]
                        thrpt:  [5.9084 Kelem/s 6.0085 Kelem/s 6.1148 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_query/query_by_tier: Collecting 100 samples in estimated 6.5001 s (15k metadata_store_query/query_by_tier
                        time:   [425.90 µs 429.06 µs 432.50 µs]
                        thrpt:  [2.3121 Kelem/s 2.3307 Kelem/s 2.3480 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_accepting_work: Collecting 100 samples in estimated 5.4009 metadata_store_query/query_accepting_work
                        time:   [523.84 µs 527.61 µs 531.58 µs]
                        thrpt:  [1.8812 Kelem/s 1.8953 Kelem/s 1.9090 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking metadata_store_query/query_with_limit: Collecting 100 samples in estimated 5.3135 s (1metadata_store_query/query_with_limit
                        time:   [510.19 µs 514.39 µs 519.26 µs]
                        thrpt:  [1.9258 Kelem/s 1.9440 Kelem/s 1.9601 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_query/query_complex: Collecting 100 samples in estimated 5.0927 s (15k metadata_store_query/query_complex
                        time:   [327.23 µs 330.75 µs 334.42 µs]
                        thrpt:  [2.9903 Kelem/s 3.0234 Kelem/s 3.0560 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking metadata_store_spatial/find_nearby_100km: Collecting 100 samples in estimated 5.1270 smetadata_store_spatial/find_nearby_100km
                        time:   [334.24 µs 337.15 µs 340.29 µs]
                        thrpt:  [2.9386 Kelem/s 2.9661 Kelem/s 2.9918 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_1000km: Collecting 100 samples in estimated 5.8638 metadata_store_spatial/find_nearby_1000km
                        time:   [384.43 µs 387.43 µs 390.50 µs]
                        thrpt:  [2.5608 Kelem/s 2.5811 Kelem/s 2.6013 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking metadata_store_spatial/find_nearby_5000km: Collecting 100 samples in estimated 5.0802 metadata_store_spatial/find_nearby_5000km
                        time:   [500.33 µs 505.89 µs 511.99 µs]
                        thrpt:  [1.9532 Kelem/s 1.9767 Kelem/s 1.9987 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_spatial/find_best_for_routing: Collecting 100 samples in estimated 5.16metadata_store_spatial/find_best_for_routing
                        time:   [340.47 µs 343.68 µs 347.25 µs]
                        thrpt:  [2.8797 Kelem/s 2.9097 Kelem/s 2.9371 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_spatial/find_relays: Collecting 100 samples in estimated 5.5866 s (10k metadata_store_spatial/find_relays
                        time:   [544.79 µs 549.53 µs 554.53 µs]
                        thrpt:  [1.8033 Kelem/s 1.8197 Kelem/s 1.8356 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe

Benchmarking metadata_store_scaling/query_status/1000: Collecting 100 samples in estimated 5.0119 smetadata_store_scaling/query_status/1000
                        time:   [22.656 µs 22.858 µs 23.076 µs]
                        thrpt:  [43.336 Kelem/s 43.749 Kelem/s 44.138 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/query_complex/1000: Collecting 100 samples in estimated 5.0274 metadata_store_scaling/query_complex/1000
                        time:   [21.635 µs 21.826 µs 22.040 µs]
                        thrpt:  [45.372 Kelem/s 45.817 Kelem/s 46.220 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/1000: Collecting 100 samples in estimated 5.0573 s metadata_store_scaling/find_nearby/1000
                        time:   [48.753 µs 49.005 µs 49.285 µs]
                        thrpt:  [20.290 Kelem/s 20.406 Kelem/s 20.512 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_status/5000: Collecting 100 samples in estimated 5.6080 smetadata_store_scaling/query_status/5000
                        time:   [123.14 µs 124.73 µs 126.32 µs]
                        thrpt:  [7.9164 Kelem/s 8.0174 Kelem/s 8.1206 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking metadata_store_scaling/query_complex/5000: Collecting 100 samples in estimated 5.0917 metadata_store_scaling/query_complex/5000
                        time:   [122.92 µs 124.44 µs 126.14 µs]
                        thrpt:  [7.9274 Kelem/s 8.0361 Kelem/s 8.1353 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/find_nearby/5000: Collecting 100 samples in estimated 5.9428 s metadata_store_scaling/find_nearby/5000
                        time:   [234.58 µs 236.49 µs 238.55 µs]
                        thrpt:  [4.1921 Kelem/s 4.2285 Kelem/s 4.2630 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  4 (4.00%) high mild
  4 (4.00%) high severe
Benchmarking metadata_store_scaling/query_status/10000: Collecting 100 samples in estimated 6.3485 metadata_store_scaling/query_status/10000
                        time:   [307.02 µs 309.87 µs 312.88 µs]
                        thrpt:  [3.1961 Kelem/s 3.2271 Kelem/s 3.2571 Kelem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
Benchmarking metadata_store_scaling/query_complex/10000: Collecting 100 samples in estimated 6.6541metadata_store_scaling/query_complex/10000
                        time:   [325.64 µs 328.66 µs 331.98 µs]
                        thrpt:  [3.0123 Kelem/s 3.0427 Kelem/s 3.0709 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking metadata_store_scaling/find_nearby/10000: Collecting 100 samples in estimated 5.1541 smetadata_store_scaling/find_nearby/10000
                        time:   [501.64 µs 504.66 µs 507.94 µs]
                        thrpt:  [1.9687 Kelem/s 1.9815 Kelem/s 1.9935 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  7 (7.00%) high severe
Benchmarking metadata_store_scaling/query_status/50000: Collecting 100 samples in estimated 5.2616 metadata_store_scaling/query_status/50000
                        time:   [3.1394 ms 3.2212 ms 3.3066 ms]
                        thrpt:  [302.43  elem/s 310.45  elem/s 318.54  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking metadata_store_scaling/query_complex/50000: Collecting 100 samples in estimated 5.3491metadata_store_scaling/query_complex/50000
                        time:   [3.3944 ms 3.4700 ms 3.5473 ms]
                        thrpt:  [281.91  elem/s 288.19  elem/s 294.60  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking metadata_store_scaling/find_nearby/50000: Collecting 100 samples in estimated 5.1512 smetadata_store_scaling/find_nearby/50000
                        time:   [3.2085 ms 3.2681 ms 3.3299 ms]
                        thrpt:  [300.31  elem/s 305.98  elem/s 311.67  elem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

Benchmarking metadata_store_concurrent/concurrent_upsert/4: Collecting 20 samples in estimated 5.25metadata_store_concurrent/concurrent_upsert/4
                        time:   [1.4363 ms 1.4708 ms 1.4996 ms]
                        thrpt:  [1.3337 Melem/s 1.3598 Melem/s 1.3925 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 5.4s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/4: Collecting 20 samples in estimated 5.393metadata_store_concurrent/concurrent_query/4
                        time:   [270.27 ms 274.61 ms 279.22 ms]
                        thrpt:  [7.1627 Kelem/s 7.2831 Kelem/s 7.4001 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 9.823metadata_store_concurrent/concurrent_mixed/4
                        time:   [239.76 ms 242.73 ms 245.83 ms]
                        thrpt:  [8.1356 Kelem/s 8.2395 Kelem/s 8.3417 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_upsert/8: Collecting 20 samples in estimated 5.02metadata_store_concurrent/concurrent_upsert/8
                        time:   [1.9815 ms 2.0046 ms 2.0255 ms]
                        thrpt:  [1.9748 Melem/s 1.9954 Melem/s 2.0186 Melem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking metadata_store_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.7s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/8: Collecting 20 samples in estimated 6.745metadata_store_concurrent/concurrent_query/8
                        time:   [339.17 ms 345.84 ms 352.67 ms]
                        thrpt:  [11.342 Kelem/s 11.566 Kelem/s 11.794 Kelem/s]
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 6.4s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 6.433metadata_store_concurrent/concurrent_mixed/8
                        time:   [315.35 ms 324.38 ms 333.95 ms]
                        thrpt:  [11.978 Kelem/s 12.331 Kelem/s 12.684 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking metadata_store_concurrent/concurrent_upsert/16: Collecting 20 samples in estimated 5.5metadata_store_concurrent/concurrent_upsert/16
                        time:   [3.6757 ms 3.7330 ms 3.7850 ms]
                        thrpt:  [2.1136 Melem/s 2.1430 Melem/s 2.1764 Melem/s]
Benchmarking metadata_store_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.8s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_query/16: Collecting 20 samples in estimated 9.77metadata_store_concurrent/concurrent_query/16
                        time:   [484.84 ms 489.81 ms 494.16 ms]
                        thrpt:  [16.189 Kelem/s 16.333 Kelem/s 16.500 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) low mild
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 10.7s, or reduce sample count to 10.
Benchmarking metadata_store_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 10.6metadata_store_concurrent/concurrent_mixed/16
                        time:   [539.46 ms 543.68 ms 547.26 ms]
                        thrpt:  [14.618 Kelem/s 14.715 Kelem/s 14.830 Kelem/s]
Found 2 outliers among 20 measurements (10.00%)
  2 (10.00%) low mild

Benchmarking metadata_store_versioning/update_versioned_success: Collecting 100 samples in estimatemetadata_store_versioning/update_versioned_success
                        time:   [230.12 ns 233.19 ns 236.74 ns]
                        thrpt:  [4.2240 Melem/s 4.2884 Melem/s 4.3456 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
Benchmarking metadata_store_versioning/update_versioned_conflict: Collecting 100 samples in estimatmetadata_store_versioning/update_versioned_conflict
                        time:   [235.25 ns 237.38 ns 239.64 ns]
                        thrpt:  [4.1729 Melem/s 4.2127 Melem/s 4.2507 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking schema_validation/validate_string: Collecting 100 samples in estimated 5.0000 s (2.0B schema_validation/validate_string
                        time:   [3.1279 ns 3.5306 ns 3.9723 ns]
                        thrpt:  [251.74 Melem/s 283.24 Melem/s 319.70 Melem/s]
Benchmarking schema_validation/validate_integer: Collecting 100 samples in estimated 5.0000 s (1.8Bschema_validation/validate_integer
                        time:   [3.2596 ns 3.6463 ns 4.0716 ns]
                        thrpt:  [245.60 Melem/s 274.25 Melem/s 306.79 Melem/s]
Benchmarking schema_validation/validate_object: Collecting 100 samples in estimated 5.0001 s (78M ischema_validation/validate_object
                        time:   [63.453 ns 63.851 ns 64.274 ns]
                        thrpt:  [15.558 Melem/s 15.661 Melem/s 15.760 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking schema_validation/validate_array_10: Collecting 100 samples in estimated 5.0001 s (137schema_validation/validate_array_10
                        time:   [36.121 ns 36.332 ns 36.558 ns]
                        thrpt:  [27.354 Melem/s 27.524 Melem/s 27.684 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
Benchmarking schema_validation/validate_complex: Collecting 100 samples in estimated 5.0005 s (31M schema_validation/validate_complex
                        time:   [158.44 ns 159.80 ns 161.27 ns]
                        thrpt:  [6.2008 Melem/s 6.2579 Melem/s 6.3115 Melem/s]
Found 16 outliers among 100 measurements (16.00%)
  16 (16.00%) high severe

Benchmarking endpoint_matching/match_success: Collecting 100 samples in estimated 5.0004 s (23M iteendpoint_matching/match_success
                        time:   [221.36 ns 223.98 ns 226.91 ns]
                        thrpt:  [4.4071 Melem/s 4.4648 Melem/s 4.5176 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking endpoint_matching/match_failure: Collecting 100 samples in estimated 5.0003 s (22M iteendpoint_matching/match_failure
                        time:   [219.66 ns 222.12 ns 224.79 ns]
                        thrpt:  [4.4486 Melem/s 4.5021 Melem/s 4.5524 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking endpoint_matching/match_multi_param: Collecting 100 samples in estimated 5.0011 s (9.9endpoint_matching/match_multi_param
                        time:   [493.82 ns 498.16 ns 502.70 ns]
                        thrpt:  [1.9893 Melem/s 2.0074 Melem/s 2.0250 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

Benchmarking api_version/is_compatible_with: Collecting 100 samples in estimated 5.0000 s (25B iterapi_version/is_compatible_with
                        time:   [201.98 ps 202.65 ps 203.75 ps]
                        thrpt:  [4.9080 Gelem/s 4.9346 Gelem/s 4.9510 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
api_version/parse       time:   [42.751 ns 43.197 ns 43.716 ns]
                        thrpt:  [22.875 Melem/s 23.150 Melem/s 23.391 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
api_version/to_string   time:   [59.468 ns 59.908 ns 60.388 ns]
                        thrpt:  [16.560 Melem/s 16.692 Melem/s 16.816 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

api_schema/create       time:   [4.1293 µs 4.1473 µs 4.1677 µs]
                        thrpt:  [239.94 Kelem/s 241.12 Kelem/s 242.17 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
api_schema/serialize    time:   [1.4508 µs 1.4613 µs 1.4731 µs]
                        thrpt:  [678.82 Kelem/s 684.33 Kelem/s 689.28 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
api_schema/deserialize  time:   [8.2973 µs 8.3576 µs 8.4283 µs]
                        thrpt:  [118.65 Kelem/s 119.65 Kelem/s 120.52 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) high mild
  2 (2.00%) high severe
Benchmarking api_schema/find_endpoint: Collecting 100 samples in estimated 5.0004 s (19M iterationsapi_schema/find_endpoint
                        time:   [255.77 ns 257.96 ns 260.44 ns]
                        thrpt:  [3.8396 Melem/s 3.8765 Melem/s 3.9098 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  3 (3.00%) high mild
  5 (5.00%) high severe
Benchmarking api_schema/endpoints_by_tag: Collecting 100 samples in estimated 5.0001 s (48M iteratiapi_schema/endpoints_by_tag
                        time:   [101.76 ns 102.64 ns 103.60 ns]
                        thrpt:  [9.6523 Melem/s 9.7424 Melem/s 9.8268 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe

Benchmarking request_validation/validate_full_request: Collecting 100 samples in estimated 5.0001 srequest_validation/validate_full_request
                        time:   [61.429 ns 61.793 ns 62.175 ns]
                        thrpt:  [16.084 Melem/s 16.183 Melem/s 16.279 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking request_validation/validate_path_only: Collecting 100 samples in estimated 5.0000 s (3request_validation/validate_path_only
                        time:   [15.764 ns 15.987 ns 16.234 ns]
                        thrpt:  [61.599 Melem/s 62.551 Melem/s 63.437 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

Benchmarking api_registry_basic/create: Collecting 100 samples in estimated 5.0026 s (3.9M iteratioapi_registry_basic/create
                        time:   [1.2755 µs 1.2844 µs 1.2940 µs]
                        thrpt:  [772.77 Kelem/s 778.58 Kelem/s 783.99 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_basic/register_new: Collecting 100 samples in estimated 5.0211 s (1.1M itapi_registry_basic/register_new
                        time:   [4.6996 µs 4.7635 µs 4.8377 µs]
                        thrpt:  [206.71 Kelem/s 209.93 Kelem/s 212.78 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
api_registry_basic/get  time:   [24.273 ns 24.433 ns 24.606 ns]
                        thrpt:  [40.640 Melem/s 40.929 Melem/s 41.198 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
api_registry_basic/len  time:   [976.53 ns 980.65 ns 985.32 ns]
                        thrpt:  [1.0149 Melem/s 1.0197 Melem/s 1.0240 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
Benchmarking api_registry_basic/stats: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 5.0s. You may wish to increase target time to 15.5s, or reduce sample count to 30.
Benchmarking api_registry_basic/stats: Collecting 100 samples in estimated 15.463 s (100 iterationsapi_registry_basic/stats
                        time:   [153.48 ms 154.37 ms 155.33 ms]
                        thrpt:  [6.4381  elem/s 6.4778  elem/s 6.5157  elem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking api_registry_query/query_by_name: Collecting 100 samples in estimated 5.0971 s (56k itapi_registry_query/query_by_name
                        time:   [90.564 µs 91.482 µs 92.472 µs]
                        thrpt:  [10.814 Kelem/s 10.931 Kelem/s 11.042 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_query/query_by_tag: Collecting 100 samples in estimated 8.5695 s (10k iteapi_registry_query/query_by_tag
                        time:   [830.12 µs 838.91 µs 848.49 µs]
                        thrpt:  [1.1786 Kelem/s 1.1920 Kelem/s 1.2046 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_query/query_with_version: Collecting 100 samples in estimated 5.0784 s (1api_registry_query/query_with_version
                        time:   [45.059 µs 45.829 µs 46.635 µs]
                        thrpt:  [21.443 Kelem/s 21.820 Kelem/s 22.193 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
Benchmarking api_registry_query/find_by_endpoint: Collecting 100 samples in estimated 5.2826 s (100api_registry_query/find_by_endpoint
                        time:   [5.1886 ms 5.2850 ms 5.3834 ms]
                        thrpt:  [185.76  elem/s 189.21  elem/s 192.73  elem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Benchmarking api_registry_query/find_compatible: Collecting 100 samples in estimated 5.2903 s (81k api_registry_query/find_compatible
                        time:   [64.526 µs 65.145 µs 65.755 µs]
                        thrpt:  [15.208 Kelem/s 15.350 Kelem/s 15.498 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe

Benchmarking api_registry_scaling/query_by_name/1000: Collecting 100 samples in estimated 5.0161 s api_registry_scaling/query_by_name/1000
                        time:   [7.8674 µs 7.9088 µs 7.9565 µs]
                        thrpt:  [125.68 Kelem/s 126.44 Kelem/s 127.11 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/1000: Collecting 100 samples in estimated 5.1370 s (api_registry_scaling/query_by_tag/1000
                        time:   [40.523 µs 40.741 µs 40.987 µs]
                        thrpt:  [24.398 Kelem/s 24.545 Kelem/s 24.678 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
Benchmarking api_registry_scaling/query_by_name/5000: Collecting 100 samples in estimated 5.0367 s api_registry_scaling/query_by_name/5000
                        time:   [39.306 µs 39.501 µs 39.727 µs]
                        thrpt:  [25.172 Kelem/s 25.316 Kelem/s 25.441 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
Benchmarking api_registry_scaling/query_by_tag/5000: Collecting 100 samples in estimated 6.4052 s (api_registry_scaling/query_by_tag/5000
                        time:   [308.68 µs 311.91 µs 315.29 µs]
                        thrpt:  [3.1717 Kelem/s 3.2061 Kelem/s 3.2396 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking api_registry_scaling/query_by_name/10000: Collecting 100 samples in estimated 5.1276 sapi_registry_scaling/query_by_name/10000
                        time:   [91.074 µs 91.763 µs 92.537 µs]
                        thrpt:  [10.806 Kelem/s 10.898 Kelem/s 10.980 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking api_registry_scaling/query_by_tag/10000: Collecting 100 samples in estimated 8.3710 s api_registry_scaling/query_by_tag/10000
                        time:   [802.27 µs 807.49 µs 813.28 µs]
                        thrpt:  [1.2296 Kelem/s 1.2384 Kelem/s 1.2465 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe

Benchmarking api_registry_concurrent/concurrent_query/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 14.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/4: Collecting 20 samples in estimated 14.689 api_registry_concurrent/concurrent_query/4
                        time:   [711.60 ms 723.64 ms 736.79 ms]
                        thrpt:  [2.7145 Kelem/s 2.7638 Kelem/s 2.8106 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/4: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 9.3s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/4: Collecting 20 samples in estimated 9.3079 api_registry_concurrent/concurrent_mixed/4
                        time:   [458.44 ms 461.78 ms 465.95 ms]
                        thrpt:  [4.2923 Kelem/s 4.3310 Kelem/s 4.3626 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
Benchmarking api_registry_concurrent/concurrent_query/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 17.1s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/8: Collecting 20 samples in estimated 17.063 api_registry_concurrent/concurrent_query/8
                        time:   [805.99 ms 815.45 ms 825.00 ms]
                        thrpt:  [4.8485 Kelem/s 4.9053 Kelem/s 4.9628 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/8: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 11.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/8: Collecting 20 samples in estimated 11.369 api_registry_concurrent/concurrent_mixed/8
                        time:   [565.20 ms 570.22 ms 576.03 ms]
                        thrpt:  [6.9441 Kelem/s 7.0148 Kelem/s 7.0772 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild
Benchmarking api_registry_concurrent/concurrent_query/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 21.7s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_query/16: Collecting 20 samples in estimated 21.708api_registry_concurrent/concurrent_query/16
                        time:   [1.0657 s 1.0832 s 1.1007 s]
                        thrpt:  [7.2683 Kelem/s 7.3858 Kelem/s 7.5071 Kelem/s]
Benchmarking api_registry_concurrent/concurrent_mixed/16: Warming up for 3.0000 s
Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 16.4s, or reduce sample count to 10.
Benchmarking api_registry_concurrent/concurrent_mixed/16: Collecting 20 samples in estimated 16.417api_registry_concurrent/concurrent_mixed/16
                        time:   [789.70 ms 796.81 ms 804.09 ms]
                        thrpt:  [9.9491 Kelem/s 10.040 Kelem/s 10.130 Kelem/s]
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high mild

compare_op/eq           time:   [1.4378 ns 1.4454 ns 1.4534 ns]
                        thrpt:  [688.05 Melem/s 691.84 Melem/s 695.51 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
compare_op/gt           time:   [924.71 ps 927.56 ps 930.72 ps]
                        thrpt:  [1.0744 Gelem/s 1.0781 Gelem/s 1.0814 Gelem/s]
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
Benchmarking compare_op/contains_string: Collecting 100 samples in estimated 5.0000 s (255M iteraticompare_op/contains_string
                        time:   [19.442 ns 19.504 ns 19.572 ns]
                        thrpt:  [51.092 Melem/s 51.273 Melem/s 51.435 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
compare_op/in_array     time:   [3.1275 ns 3.1485 ns 3.1719 ns]
                        thrpt:  [315.26 Melem/s 317.62 Melem/s 319.74 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild

condition/simple        time:   [46.874 ns 47.284 ns 47.726 ns]
                        thrpt:  [20.953 Melem/s 21.149 Melem/s 21.334 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  4 (4.00%) high mild
  2 (2.00%) high severe
condition/nested_field  time:   [620.09 ns 627.27 ns 635.29 ns]
                        thrpt:  [1.5741 Melem/s 1.5942 Melem/s 1.6127 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
condition/string_eq     time:   [75.052 ns 75.651 ns 76.322 ns]
                        thrpt:  [13.102 Melem/s 13.219 Melem/s 13.324 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe

condition_expr/single   time:   [46.865 ns 47.274 ns 47.730 ns]
                        thrpt:  [20.951 Melem/s 21.153 Melem/s 21.338 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
condition_expr/and_2    time:   [97.455 ns 98.150 ns 98.905 ns]
                        thrpt:  [10.111 Melem/s 10.188 Melem/s 10.261 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe
condition_expr/and_5    time:   [309.37 ns 311.95 ns 314.82 ns]
                        thrpt:  [3.1764 Melem/s 3.2057 Melem/s 3.2324 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  4 (4.00%) high mild
  5 (5.00%) high severe
condition_expr/or_3     time:   [188.64 ns 193.60 ns 199.00 ns]
                        thrpt:  [5.0251 Melem/s 5.1652 Melem/s 5.3011 Melem/s]
Found 10 outliers among 100 measurements (10.00%)
  8 (8.00%) high mild
  2 (2.00%) high severe
condition_expr/nested   time:   [131.76 ns 133.38 ns 135.29 ns]
                        thrpt:  [7.3916 Melem/s 7.4974 Melem/s 7.5897 Melem/s]
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

rule/create             time:   [412.11 ns 415.28 ns 418.81 ns]
                        thrpt:  [2.3877 Melem/s 2.4080 Melem/s 2.4265 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
rule/matches            time:   [106.97 ns 107.54 ns 108.17 ns]
                        thrpt:  [9.2449 Melem/s 9.2987 Melem/s 9.3486 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

rule_context/create     time:   [1.8524 µs 1.8636 µs 1.8760 µs]
                        thrpt:  [533.04 Kelem/s 536.60 Kelem/s 539.83 Kelem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_context/get_simple: Collecting 100 samples in estimated 5.0002 s (107M iterationsrule_context/get_simple time:   [46.405 ns 47.087 ns 47.865 ns]
                        thrpt:  [20.892 Melem/s 21.237 Melem/s 21.549 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  4 (4.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_context/get_nested: Collecting 100 samples in estimated 5.0027 s (8.1M iterationsrule_context/get_nested time:   [611.83 ns 617.66 ns 624.13 ns]
                        thrpt:  [1.6022 Melem/s 1.6190 Melem/s 1.6344 Melem/s]
Found 11 outliers among 100 measurements (11.00%)
  7 (7.00%) high mild
  4 (4.00%) high severe
Benchmarking rule_context/get_deep_nested: Collecting 100 samples in estimated 5.0004 s (8.1M iterarule_context/get_deep_nested
                        time:   [611.74 ns 615.75 ns 619.91 ns]
                        thrpt:  [1.6131 Melem/s 1.6240 Melem/s 1.6347 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe

Benchmarking rule_engine_basic/create: Collecting 100 samples in estimated 5.0001 s (402M iterationrule_engine_basic/create
                        time:   [12.086 ns 12.185 ns 12.285 ns]
                        thrpt:  [81.402 Melem/s 82.065 Melem/s 82.739 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_basic/add_rule: Collecting 100 samples in estimated 5.1051 s (106k iteratirule_engine_basic/add_rule
                        time:   [2.6903 µs 2.8189 µs 2.9271 µs]
                        thrpt:  [341.64 Kelem/s 354.75 Kelem/s 371.70 Kelem/s]
Benchmarking rule_engine_basic/get_rule: Collecting 100 samples in estimated 5.0000 s (301M iteratirule_engine_basic/get_rule
                        time:   [16.509 ns 16.613 ns 16.730 ns]
                        thrpt:  [59.773 Melem/s 60.192 Melem/s 60.571 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_basic/rules_by_tag: Collecting 100 samples in estimated 5.0019 s (4.9M iterule_engine_basic/rules_by_tag
                        time:   [1.0067 µs 1.0164 µs 1.0267 µs]
                        thrpt:  [974.04 Kelem/s 983.90 Kelem/s 993.37 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
Benchmarking rule_engine_basic/stats: Collecting 100 samples in estimated 5.0285 s (550k iterationsrule_engine_basic/stats time:   [9.0970 µs 9.1705 µs 9.2502 µs]
                        thrpt:  [108.11 Kelem/s 109.04 Kelem/s 109.93 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild

Benchmarking rule_engine_evaluate/evaluate_10_rules: Collecting 100 samples in estimated 5.0168 s (rule_engine_evaluate/evaluate_10_rules
                        time:   [3.8460 µs 3.8751 µs 3.9066 µs]
                        thrpt:  [255.98 Kelem/s 258.06 Kelem/s 260.01 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_10_rules: Collecting 100 samples in estimated 5.00rule_engine_evaluate/evaluate_first_10_rules
                        time:   [334.62 ns 338.94 ns 343.65 ns]
                        thrpt:  [2.9099 Melem/s 2.9504 Melem/s 2.9884 Melem/s]
Found 7 outliers among 100 measurements (7.00%)
  6 (6.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_100_rules: Collecting 100 samples in estimated 5.1515 s rule_engine_evaluate/evaluate_100_rules
                        time:   [34.633 µs 34.918 µs 35.233 µs]
                        thrpt:  [28.382 Kelem/s 28.639 Kelem/s 28.874 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
Benchmarking rule_engine_evaluate/evaluate_first_100_rules: Collecting 100 samples in estimated 5.0rule_engine_evaluate/evaluate_first_100_rules
                        time:   [331.94 ns 335.91 ns 340.29 ns]
                        thrpt:  [2.9387 Melem/s 2.9770 Melem/s 3.0126 Melem/s]
Found 9 outliers among 100 measurements (9.00%)
  6 (6.00%) high mild
  3 (3.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_matching_100_rules: Collecting 100 samples in estimated rule_engine_evaluate/evaluate_matching_100_rules
                        time:   [35.452 µs 35.692 µs 35.948 µs]
                        thrpt:  [27.818 Kelem/s 28.018 Kelem/s 28.207 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_1000_rules: Collecting 100 samples in estimated 6.4076 srule_engine_evaluate/evaluate_1000_rules
                        time:   [313.50 µs 316.33 µs 319.49 µs]
                        thrpt:  [3.1300 Kelem/s 3.1613 Kelem/s 3.1898 Kelem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high severe
Benchmarking rule_engine_evaluate/evaluate_first_1000_rules: Collecting 100 samples in estimated 5.rule_engine_evaluate/evaluate_first_1000_rules
                        time:   [334.70 ns 337.62 ns 340.86 ns]
                        thrpt:  [2.9338 Melem/s 2.9619 Melem/s 2.9878 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

Benchmarking rule_engine_scaling/evaluate/10: Collecting 100 samples in estimated 5.0040 s (1.3M itrule_engine_scaling/evaluate/10
                        time:   [3.8572 µs 3.8842 µs 3.9130 µs]
                        thrpt:  [255.56 Kelem/s 257.45 Kelem/s 259.25 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
rule_engine_scaling/evaluate_first/10
                        time:   [334.32 ns 339.16 ns 344.41 ns]
                        thrpt:  [2.9035 Melem/s 2.9484 Melem/s 2.9911 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
rule_engine_scaling/evaluate/50
                        time:   [20.535 µs 20.744 µs 20.977 µs]
                        thrpt:  [47.672 Kelem/s 48.206 Kelem/s 48.697 Kelem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe
rule_engine_scaling/evaluate_first/50
                        time:   [330.47 ns 333.50 ns 336.67 ns]
                        thrpt:  [2.9703 Melem/s 2.9985 Melem/s 3.0260 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
rule_engine_scaling/evaluate/100
                        time:   [34.985 µs 35.338 µs 35.702 µs]
                        thrpt:  [28.010 Kelem/s 28.298 Kelem/s 28.584 Kelem/s]
Found 8 outliers among 100 measurements (8.00%)
  5 (5.00%) high mild
  3 (3.00%) high severe
rule_engine_scaling/evaluate_first/100
                        time:   [347.26 ns 355.58 ns 364.79 ns]
                        thrpt:  [2.7413 Melem/s 2.8123 Melem/s 2.8797 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
rule_engine_scaling/evaluate/500
                        time:   [192.80 µs 197.32 µs 201.89 µs]
                        thrpt:  [4.9533 Kelem/s 5.0679 Kelem/s 5.1868 Kelem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
rule_engine_scaling/evaluate_first/500
                        time:   [360.00 ns 367.28 ns 374.77 ns]
                        thrpt:  [2.6683 Melem/s 2.7227 Melem/s 2.7778 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_scaling/evaluate/1000
                        time:   [378.58 µs 387.01 µs 395.30 µs]
                        thrpt:  [2.5297 Kelem/s 2.5839 Kelem/s 2.6415 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
rule_engine_scaling/evaluate_first/1000
                        time:   [365.11 ns 372.21 ns 379.81 ns]
                        thrpt:  [2.6329 Melem/s 2.6867 Melem/s 2.7389 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild

rule_set/create         time:   [7.0667 µs 7.1837 µs 7.3020 µs]
                        thrpt:  [136.95 Kelem/s 139.20 Kelem/s 141.51 Kelem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high mild
rule_set/load_into_engine
                        time:   [14.546 µs 14.801 µs 15.068 µs]
                        thrpt:  [66.368 Kelem/s 67.563 Kelem/s 68.747 Kelem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

trace_id/generate       time:   [53.039 ns 53.903 ns 54.778 ns]
                        thrpt:  [18.255 Melem/s 18.552 Melem/s 18.854 Melem/s]
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe
trace_id/to_hex         time:   [105.19 ns 107.54 ns 109.90 ns]
                        thrpt:  [9.0992 Melem/s 9.2990 Melem/s 9.5065 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
trace_id/from_hex       time:   [20.662 ns 21.074 ns 21.513 ns]
                        thrpt:  [46.484 Melem/s 47.452 Melem/s 48.397 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

context_operations/create
                        time:   [91.559 ns 93.145 ns 94.763 ns]
                        thrpt:  [10.553 Melem/s 10.736 Melem/s 10.922 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
context_operations/child
                        time:   [41.086 ns 41.705 ns 42.329 ns]
                        thrpt:  [23.624 Melem/s 23.978 Melem/s 24.339 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
context_operations/for_remote
                        time:   [41.080 ns 41.674 ns 42.257 ns]
                        thrpt:  [23.665 Melem/s 23.996 Melem/s 24.343 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
context_operations/to_traceparent
                        time:   [323.71 ns 329.59 ns 335.83 ns]
                        thrpt:  [2.9777 Melem/s 3.0340 Melem/s 3.0892 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
context_operations/from_traceparent
                        time:   [131.55 ns 134.03 ns 136.50 ns]
                        thrpt:  [7.3258 Melem/s 7.4608 Melem/s 7.6017 Melem/s]
Found 5 outliers among 100 measurements (5.00%)
  3 (3.00%) high mild
  2 (2.00%) high severe

baggage/create          time:   [4.6523 ns 4.6718 ns 4.6914 ns]
                        thrpt:  [213.15 Melem/s 214.05 Melem/s 214.95 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) low mild
baggage/get             time:   [9.5752 ns 9.7679 ns 9.9727 ns]
                        thrpt:  [100.27 Melem/s 102.38 Melem/s 104.44 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
baggage/set             time:   [73.567 ns 75.342 ns 77.093 ns]
                        thrpt:  [12.971 Melem/s 13.273 Melem/s 13.593 Melem/s]
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
baggage/merge           time:   [1.9281 µs 1.9562 µs 1.9856 µs]
                        thrpt:  [503.63 Kelem/s 511.20 Kelem/s 518.63 Kelem/s]

span/create             time:   [93.299 ns 94.760 ns 96.267 ns]
                        thrpt:  [10.388 Melem/s 10.553 Melem/s 10.718 Melem/s]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) high mild
  1 (1.00%) high severe
span/set_attribute      time:   [69.802 ns 71.222 ns 72.638 ns]
                        thrpt:  [13.767 Melem/s 14.041 Melem/s 14.326 Melem/s]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
span/add_event          time:   [77.060 ns 109.49 ns 180.55 ns]
                        thrpt:  [5.5387 Melem/s 9.1332 Melem/s 12.977 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  4 (4.00%) high severe
span/with_kind          time:   [94.204 ns 95.897 ns 97.610 ns]
                        thrpt:  [10.245 Melem/s 10.428 Melem/s 10.615 Melem/s]
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe

context_store/create_context
                        time:   [110.06 µs 111.78 µs 113.55 µs]
                        thrpt:  [8.8063 Kelem/s 8.9460 Kelem/s 9.0861 Kelem/s]
