# Blackstream Benchmarks

Performance benchmarks for the Blackstream Rust core and BLTP transport layer.

**Test Systems:**
- Apple M1 Max, macOS
- Intel i9-14900K @5GHz, Windows 11

## BLTP Header Operations

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 2.0539 ns | 486.87 Melem/s | 1.2450 ns | 803.19 Melem/s |
| Deserialize | 2.1828 ns | 458.12 Melem/s | 1.2502 ns | 799.86 Melem/s |
| Roundtrip | 2.1811 ns | 458.49 Melem/s | 1.2432 ns | 804.37 Melem/s |
| AAD generation | 2.0996 ns | 476.27 Melem/s | 1.0620 ns | 941.64 Melem/s |

## Event Frame Serialization

### Single Write

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 19.312 ns | 3.0864 GiB/s | 36.658 ns | 1.6260 GiB/s |
| 256B | 50.368 ns | 4.7335 GiB/s | 36.810 ns | 6.4769 GiB/s |
| 1KB | 39.321 ns | 24.254 GiB/s | 37.230 ns | 25.616 GiB/s |
| 4KB | 88.656 ns | 43.028 GiB/s | 56.200 ns | 67.878 GiB/s |

### Batch Write (64B events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 events | 18.988 ns | 3.1390 GiB/s | 28.567 ns | 2.0865 GiB/s |
| 10 events | 73.368 ns | 8.1240 GiB/s | 58.107 ns | 10.258 GiB/s |
| 50 events | 156.61 ns | 19.030 GiB/s | 149.63 ns | 19.918 GiB/s |
| 100 events | 285.39 ns | 20.886 GiB/s | 281.87 ns | 21.146 GiB/s |

### Batch Read

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Read batch (10 events) | 148.38 ns | 67.393 Melem/s | 169.60 ns | 58.961 Melem/s |

## Packet Pool (Zero-Allocation)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool get+return | 34.152 ns | 29.281 Melem/s | 33.465 ns | 29.882 Melem/s |
| Thread-local get+return | 45.752 ns | 21.857 Melem/s | 40.309 ns | 24.809 Melem/s |

### Pool Comparison (10x cycles)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool 10x | 319.20 ns | 3.1328 Melem/s | 321.42 ns | 3.1112 Melem/s |
| Thread-local 10x | 528.45 ns | 1.8923 Melem/s | 484.38 ns | 2.0645 Melem/s |

## Packet Build

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 event | 739.56 ns | 82.529 MiB/s | 1.1398 us | 53.550 MiB/s |
| 10 events | 2.5199 us | 242.21 MiB/s | 1.5049 us | 405.58 MiB/s |
| 50 events | 10.820 us | 282.06 MiB/s | 2.9616 us | 1.0063 GiB/s |

## Encryption (ChaCha20-Poly1305)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 742.76 ns | 82.173 MiB/s | 1.1382 us | 53.623 MiB/s |
| 256B | 1.3187 us | 185.14 MiB/s | 1.2106 us | 201.67 MiB/s |
| 1KB | 3.6486 us | 267.66 MiB/s | 1.6028 us | 609.30 MiB/s |
| 4KB | 12.944 us | 301.77 MiB/s | 3.1795 us | 1.1998 GiB/s |

### End-to-End Packet Build (50 events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool | 10.869 us | 280.79 MiB/s | 2.9614 us | 1.0063 GiB/s |
| Thread-local pool | 10.787 us | 282.90 MiB/s | 2.9145 us | 1.0225 GiB/s |

## Adaptive Batcher Overhead

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| optimal_size() | 1.0156 ns | 984.68 Melem/s | 828.74 ps | 1.2067 Gelem/s |
| record() | 2.6674 ns | 374.90 Melem/s | 9.7081 ns | 103.01 Melem/s |
| full_cycle | 2.9280 ns | 341.53 Melem/s | 8.2401 ns | 121.36 Melem/s |

## Key Generation

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Keypair generate | 34.651 us | 28.859 Kelem/s | 21.389 us | 46.752 Kelem/s |

## Multi-threaded Packet Build (1000 packets/thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 2.5449 ms | 3.1435 Melem/s | 1.7858 ms | 4.4799 Melem/s |
| 8 | Thread-local | 1.3421 ms | 5.9609 Melem/s | 1.5508 ms | 5.1586 Melem/s |
| 16 | Shared | 5.0687 ms | 3.1566 Melem/s | 2.8302 ms | 5.6533 Melem/s |
| 16 | Thread-local | 2.1354 ms | 7.4929 Melem/s | 2.1174 ms | 7.5566 Melem/s |
| 24 | Shared | 7.7371 ms | 3.1019 Melem/s | 4.3035 ms | 5.5769 Melem/s |
| 24 | Thread-local | 3.0211 ms | 7.9441 Melem/s | 3.1152 ms | 7.7042 Melem/s |
| 32 | Shared | 10.527 ms | 3.0398 Melem/s | 5.6272 ms | 5.6867 Melem/s |
| 32 | Thread-local | 3.8275 ms | 8.3605 Melem/s | 3.7177 ms | 8.6074 Melem/s |

### Pool Contention (10,000 acquire/release per thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 14.503 ms | 5.5161 Melem/s | 8.7283 ms | 9.1656 Melem/s |
| 8 | Thread-local | 985.95 us | 81.140 Melem/s | 968.01 us | 82.644 Melem/s |
| 16 | Shared | 30.569 ms | 5.2341 Melem/s | 20.381 ms | 7.8504 Melem/s |
| 16 | Thread-local | 1.5572 ms | 102.75 Melem/s | 1.3143 ms | 121.74 Melem/s |
| 24 | Shared | 47.616 ms | 5.0403 Melem/s | 30.619 ms | 7.8383 Melem/s |
| 24 | Thread-local | 2.1800 ms | 110.09 Melem/s | 1.7749 ms | 135.22 Melem/s |
| 32 | Shared | 64.580 ms | 4.9551 Melem/s | 40.292 ms | 7.9421 Melem/s |
| 32 | Thread-local | 2.8017 ms | 114.22 Melem/s | 2.2420 ms | 142.73 Melem/s |

### Mixed Frame Sizes (64B/256B/1KB rotation)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 2.2018 ms | 5.4502 Melem/s | 1.2559 ms | 9.5547 Melem/s |
| 8 | Thread-local | 1.6787 ms | 7.1484 Melem/s | 1.1409 ms | 10.518 Melem/s |
| 16 | Shared | 4.0314 ms | 5.9532 Melem/s | 1.9486 ms | 12.316 Melem/s |
| 16 | Thread-local | 2.8845 ms | 8.3203 Melem/s | 1.7394 ms | 13.798 Melem/s |
| 24 | Shared | 5.8829 ms | 6.1194 Melem/s | 2.6323 ms | 13.676 Melem/s |
| 24 | Thread-local | 4.0865 ms | 8.8096 Melem/s | 2.2817 ms | 15.778 Melem/s |
| 32 | Shared | 7.3288 ms | 6.5495 Melem/s | 3.5061 ms | 13.690 Melem/s |
| 32 | Thread-local | 5.2124 ms | 9.2087 Melem/s | 2.8195 ms | 17.024 Melem/s |

### Throughput Scaling (Thread-local Pool)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 threads | 8.9148 ms | 224.35 Kelem/s | 3.5843 ms | 558.00 Kelem/s |
| 2 threads | 9.0292 ms | 443.01 Kelem/s | 3.7487 ms | 1.0670 Melem/s |
| 4 threads | 9.3492 ms | 855.69 Kelem/s | 3.9937 ms | 2.0032 Melem/s |
| 8 threads | 12.241 ms | 1.3071 Melem/s | 4.8470 ms | 3.3010 Melem/s |
| 16 threads | 24.331 ms | 1.3152 Melem/s | 5.8128 ms | 5.5051 Melem/s |
| 24 threads | 30.225 ms | 1.5881 Melem/s | 9.0385 ms | 5.3106 Melem/s |
| 32 threads | 39.469 ms | 1.6215 Melem/s | 10.621 ms | 6.0257 Melem/s |

## Routing

### Routing Header

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 584.17 ps | 1.7118 Gelem/s | 309.02 ps | 3.2360 Gelem/s |
| Deserialize | 987.82 ps | 1.0123 Gelem/s | 763.80 ps | 1.3093 Gelem/s |
| Roundtrip | 975.83 ps | 1.0248 Gelem/s | 757.92 ps | 1.3194 Gelem/s |
| Forward | 580.35 ps | 1.7231 Gelem/s | 237.39 ps | 4.2126 Gelem/s |

### Routing Table

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| lookup_hit | 20.194 ns | 49.519 Melem/s | 13.840 ns | 72.253 Melem/s |
| lookup_miss | 12.146 ns | 82.334 Melem/s | 16.919 ns | 59.106 Melem/s |
| is_local | 324.84 ps | 3.0784 Gelem/s | 202.16 ps | 4.9466 Gelem/s |
| add_route | 302.00 ns | 3.3112 Melem/s | 239.05 ns | 4.1832 Melem/s |
| record_in | 52.627 ns | 19.002 Melem/s | 42.015 ns | 23.801 Melem/s |
| record_out | 19.720 ns | 50.710 Melem/s | 21.456 ns | 46.606 Melem/s |
| aggregate_stats | 2.1794 us | 458.83 Kelem/s | 8.1608 us | 122.54 Kelem/s |

### Concurrent Routing Lookup

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Lookup | 127.30 us | 31.423 Melem/s | 182.45 us | 21.924 Melem/s |
| 4 | Stats | 308.32 us | 12.974 Melem/s | 269.30 us | 14.853 Melem/s |
| 8 | Lookup | 209.67 us | 38.155 Melem/s | 314.17 us | 25.464 Melem/s |
| 8 | Stats | 506.51 us | 15.794 Melem/s | 406.86 us | 19.663 Melem/s |
| 16 | Lookup | 383.71 us | 41.698 Melem/s | 611.00 us | 26.187 Melem/s |
| 16 | Stats | 1.0359 ms | 15.445 Melem/s | 687.85 us | 23.261 Melem/s |

### Decision Pipeline

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| parse + lookup + forward | 17.958 ns | 55.685 Melem/s | 14.377 ns | 69.556 Melem/s |
| full with stats | 75.326 ns | 13.276 Melem/s | 81.937 ns | 12.204 Melem/s |

## Multi-hop Forwarding

### Packet Builder

| Payload | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | Build | 25.366 ns | 2.3498 GiB/s | 47.740 ns | 1.2485 GiB/s |
| 64B | Build (priority) | 26.921 ns | 2.2140 GiB/s | 31.259 ns | 1.9068 GiB/s |
| 256B | Build | 56.812 ns | 4.1966 GiB/s | 41.914 ns | 5.6882 GiB/s |
| 256B | Build (priority) | 55.882 ns | 4.2665 GiB/s | 31.705 ns | 7.5198 GiB/s |
| 1KB | Build | 43.533 ns | 21.907 GiB/s | 45.548 ns | 20.938 GiB/s |
| 1KB | Build (priority) | 43.410 ns | 21.969 GiB/s | 40.098 ns | 23.783 GiB/s |
| 4KB | Build | 97.036 ns | 39.312 GiB/s | 66.203 ns | 57.621 GiB/s |
| 4KB | Build (priority) | 95.018 ns | 40.147 GiB/s | 63.393 ns | 60.176 GiB/s |

### Chain Scaling (forward_chain)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 59.628 ns | 16.771 Melem/s | 52.972 ns | 18.878 Melem/s |
| 2 hops | 118.82 ns | 8.4160 Melem/s | 85.618 ns | 11.680 Melem/s |
| 3 hops | 168.94 ns | 5.9194 Melem/s | 118.60 ns | 8.4315 Melem/s |
| 4 hops | 231.17 ns | 4.3258 Melem/s | 152.32 ns | 6.5651 Melem/s |
| 5 hops | 290.57 ns | 3.4415 Melem/s | 187.23 ns | 5.3409 Melem/s |

### Hop Latency

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Single hop process | 1.3413 ns | 745.55 Melem/s | 792.30 ps | 1.2621 Gelem/s |
| Single hop full | 57.879 ns | 17.277 Melem/s | 33.774 ns | 29.609 Melem/s |

### Hop Scaling by Payload Size

| Payload | Hops | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | 1 hops | 30.398 ns | 1.9608 GiB/s | 52.892 ns | 1.1269 GiB/s |
| 64B | 2 hops | 52.968 ns | 1.1253 GiB/s | 85.319 ns | 715.38 MiB/s |
| 64B | 3 hops | 77.216 ns | 790.45 MiB/s | 141.14 ns | 432.45 MiB/s |
| 64B | 4 hops | 101.03 ns | 604.10 MiB/s | 151.95 ns | 401.68 MiB/s |
| 64B | 5 hops | 132.75 ns | 459.77 MiB/s | 222.34 ns | 274.52 MiB/s |
| 256B | 1 hops | 63.826 ns | 3.7354 GiB/s | 53.002 ns | 4.4983 GiB/s |
| 256B | 2 hops | 122.66 ns | 1.9438 GiB/s | 86.032 ns | 2.7713 GiB/s |
| 256B | 3 hops | 174.27 ns | 1.3681 GiB/s | 118.77 ns | 2.0074 GiB/s |
| 256B | 4 hops | 233.61 ns | 1.0206 GiB/s | 152.93 ns | 1.5590 GiB/s |
| 256B | 5 hops | 285.09 ns | 856.35 MiB/s | 186.82 ns | 1.2762 GiB/s |
| 1024B | 1 hops | 51.031 ns | 18.688 GiB/s | 55.427 ns | 17.206 GiB/s |
| 1024B | 2 hops | 117.28 ns | 8.1319 GiB/s | 92.584 ns | 10.301 GiB/s |
| 1024B | 3 hops | 163.45 ns | 5.8348 GiB/s | 132.92 ns | 7.1747 GiB/s |
| 1024B | 4 hops | 220.52 ns | 4.3247 GiB/s | 173.95 ns | 5.4825 GiB/s |
| 1024B | 5 hops | 261.15 ns | 3.6518 GiB/s | 218.58 ns | 4.3630 GiB/s |

### Route and Forward (with routing table lookup)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 154.32 ns | 6.4801 Melem/s | 131.37 ns | 7.6120 Melem/s |
| 2 hops | 298.07 ns | 3.3549 Melem/s | 242.47 ns | 4.1243 Melem/s |
| 3 hops | 445.34 ns | 2.2455 Melem/s | 351.30 ns | 2.8466 Melem/s |
| 4 hops | 609.33 ns | 1.6412 Melem/s | 466.91 ns | 2.1417 Melem/s |
| 5 hops | 756.10 ns | 1.3226 Melem/s | 576.02 ns | 1.7361 Melem/s |

### Concurrent Forwarding

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Forward | 656.13 us | 6.0964 Melem/s | 1.1323 ms | 3.5325 Melem/s |
| 8 | Forward | 1.6468 ms | 4.8578 Melem/s | 877.68 us | 9.1150 Melem/s |
| 16 | Forward | 2.0272 ms | 7.8926 Melem/s | 1.1791 ms | 13.569 Melem/s |

## Swarm / Discovery

### Pingwave

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 816.83 ps | 1.2242 Gelem/s | 529.78 ps | 1.8876 Gelem/s |
| Deserialize | 976.47 ps | 1.0241 Gelem/s | 679.87 ps | 1.4709 Gelem/s |
| Roundtrip | 976.05 ps | 1.0245 Gelem/s | 680.37 ps | 1.4698 Gelem/s |
| Forward | 657.00 ps | 1.5221 Gelem/s | 540.82 ps | 1.8490 Gelem/s |

### Local Graph

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create_pingwave | 2.2123 ns | 452.01 Melem/s | 5.1314 ns | 194.88 Melem/s |
| on_pingwave_new | 153.17 ns | 6.5286 Melem/s | 196.14 ns | 5.0985 Melem/s |
| on_pingwave_duplicate | 23.160 ns | 43.178 Melem/s | 15.967 ns | 62.630 Melem/s |
| get_node | 27.952 ns | 35.776 Melem/s | 15.568 ns | 64.236 Melem/s |
| node_count | 209.67 ns | 4.7695 Melem/s | 979.30 ns | 1.0211 Melem/s |
| stats | 628.82 ns | 1.5903 Melem/s | 2.9408 us | 340.05 Kelem/s |

### Graph Scaling

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | all_nodes | 2.4210 us | 41.304 Melem/s | 7.7085 us | 12.973 Melem/s |
| 100 | nodes_within_hops | 2.9285 us | 34.147 Melem/s | 7.7721 us | 12.867 Melem/s |
| 500 | all_nodes | 7.6763 us | 65.135 Melem/s | 17.100 us | 29.240 Melem/s |
| 500 | nodes_within_hops | 9.8546 us | 50.738 Melem/s | 16.634 us | 30.060 Melem/s |
| 1,000 | all_nodes | 150.46 us | 6.6463 Melem/s | 27.881 us | 35.866 Melem/s |
| 1,000 | nodes_within_hops | 159.10 us | 6.2853 Melem/s | 28.009 us | 35.702 Melem/s |
| 5,000 | all_nodes | 125.45 us | 39.855 Melem/s | 229.45 us | 21.791 Melem/s |
| 5,000 | nodes_within_hops | 179.69 us | 27.825 Melem/s | 226.25 us | 22.100 Melem/s |

### Path Finding

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| path_1_hop | 1.6254 us | 615.22 Kelem/s | 5.8981 us | 169.55 Kelem/s |
| path_2_hops | 1.7114 us | 584.32 Kelem/s | 5.8644 us | 170.52 Kelem/s |
| path_4_hops | 1.9694 us | 507.77 Kelem/s | 6.1469 us | 162.68 Kelem/s |
| path_not_found | 1.9593 us | 510.39 Kelem/s | 6.1103 us | 163.66 Kelem/s |
| path_complex_graph | 352.00 us | 2.8409 Kelem/s | 304.93 us | 3.2795 Kelem/s |

### Concurrent Pingwave Processing

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Pingwave | 149.76 us | 13.354 Melem/s | 219.60 us | 9.1075 Melem/s |
| 8 | Pingwave | 228.53 us | 17.503 Melem/s | 370.16 us | 10.806 Melem/s |
| 16 | Pingwave | 417.55 us | 19.159 Melem/s | 668.69 us | 11.964 Melem/s |

## Failure Detection

### Failure Detector

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| heartbeat_existing | 32.413 ns | 30.852 Melem/s | 35.787 ns | 27.943 Melem/s |
| heartbeat_new | 310.87 ns | 3.2167 Melem/s | 243.16 ns | 4.1125 Melem/s |
| status_check | 13.278 ns | 75.311 Melem/s | 13.451 ns | 74.342 Melem/s |
| check_all | 358.09 ms | 2.7926 elem/s | 340.07 ms | 2.9405 elem/s |
| stats | 87.183 ms | 11.470 elem/s | 103.14 ms | 9.6957 elem/s |

### Circuit Breaker

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| allow_closed | 14.040 ns | 71.226 Melem/s | 10.598 ns | 94.361 Melem/s |
| record_success | 14.018 ns | 71.336 Melem/s | 10.556 ns | 94.735 Melem/s |
| record_failure | 14.014 ns | 71.356 Melem/s | 10.542 ns | 94.859 Melem/s |
| state | 13.982 ns | 71.522 Melem/s | 10.564 ns | 94.661 Melem/s |

### Recovery Manager

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| on_failure_with_alternates | 321.61 ns | 3.1094 Melem/s | 314.80 ns | 3.1766 Melem/s |
| on_failure_no_alternates | 302.44 ns | 3.3065 Melem/s | 286.82 ns | 3.4866 Melem/s |
| get_action | 37.991 ns | 26.322 Melem/s | 39.079 ns | 25.589 Melem/s |
| is_failed | 11.182 ns | 89.433 Melem/s | 13.421 ns | 74.512 Melem/s |
| on_recovery | 105.52 ns | 9.4770 Melem/s | 128.67 ns | 7.7721 Melem/s |
| stats | 733.78 ps | 1.3628 Gelem/s | 1.2261 ns | 815.56 Melem/s |

### Full Recovery Cycle

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Fail + recover cycle | 362.09 ns | 2.7618 Melem/s | 323.40 ns | 3.0922 Melem/s |

### Loss Simulator

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1% | 2.1761 ns | 459.55 Melem/s | 5.1992 ns | 192.34 Melem/s |
| 5% | 2.2410 ns | 446.22 Melem/s | 5.3557 ns | 186.72 Melem/s |
| 10% | 2.3430 ns | 426.80 Melem/s | 5.5911 ns | 178.85 Melem/s |
| 20% | 3.2354 ns | 309.08 Melem/s | 6.0622 ns | 164.96 Melem/s |
| burst | 2.6500 ns | 377.36 Melem/s | 6.2800 ns | 159.23 Melem/s |

### Failure Scaling (check_all)

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | check_all | 4.9581 us | 20.169 Melem/s | 9.0881 us | 11.003 Melem/s |
| 100 | healthy_nodes | 1.7634 us | 56.709 Melem/s | 6.4434 us | 15.520 Melem/s |
| 500 | check_all | 21.533 us | 23.220 Melem/s | 23.604 us | 21.183 Melem/s |
| 500 | healthy_nodes | 7.2929 us | 68.560 Melem/s | 9.1803 us | 54.465 Melem/s |
| 1,000 | check_all | 42.246 us | 23.671 Melem/s | 41.914 us | 23.858 Melem/s |
| 1,000 | healthy_nodes | 10.715 us | 93.326 Melem/s | 12.931 us | 77.332 Melem/s |
| 5,000 | check_all | 210.66 us | 23.735 Melem/s | 187.33 us | 26.691 Melem/s |
| 5,000 | healthy_nodes | 51.587 us | 96.923 Melem/s | 42.962 us | 116.38 Melem/s |

### Concurrent Heartbeats

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Heartbeat | 214.98 us | 9.3031 Melem/s | 265.12 us | 7.5436 Melem/s |
| 8 | Heartbeat | 331.04 us | 12.083 Melem/s | 408.56 us | 9.7904 Melem/s |
| 16 | Heartbeat | 639.84 us | 12.503 Melem/s | 711.70 us | 11.241 Melem/s |

## Stream Multiplexing

| Streams | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 10 | Lookup | 127.34 ns | 78.529 Melem/s | 146.25 ns | 68.378 Melem/s |
| 10 | Stats | 525.86 ns | 19.016 Melem/s | 419.31 ns | 23.849 Melem/s |
| 100 | Lookup | 1.2897 us | 77.540 Melem/s | 1.4008 us | 71.385 Melem/s |
| 100 | Stats | 5.3736 us | 18.609 Melem/s | 4.2109 us | 23.748 Melem/s |
| 1,000 | Lookup | 13.122 us | 76.210 Melem/s | 14.556 us | 68.703 Melem/s |
| 1,000 | Stats | 54.913 us | 18.211 Melem/s | 45.515 us | 21.971 Melem/s |
| 10,000 | Lookup | 141.96 us | 70.440 Melem/s | 150.80 us | 66.311 Melem/s |
| 10,000 | Stats | 617.02 us | 16.207 Melem/s | 483.49 us | 20.683 Melem/s |

## Fair Scheduler

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Creation | 302.08 ns | 3.3104 Melem/s | 1.6471 us | 607.13 Kelem/s |
| Stream count (empty) | 211.23 ns | 4.7341 Melem/s | 976.75 ns | 1.0238 Melem/s |
| Total queued | 324.82 ps | 3.0786 Gelem/s | 202.86 ps | 4.9296 Gelem/s |
| Cleanup (empty) | 208.74 ns | 4.7907 Melem/s | 1.2988 us | 769.94 Kelem/s |

## Capability System

### CapabilitySet

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 540.39 ns | 1.8505 Melem/s | 806.28 ns | 1.2403 Melem/s |
| serialize | 962.64 ns | 1.0388 Melem/s | 721.65 ns | 1.3857 Melem/s |
| deserialize | 1.8434 us | 542.49 Kelem/s | 3.1463 us | 317.84 Kelem/s |
| roundtrip | 2.8648 us | 349.06 Kelem/s | 4.0612 us | 246.23 Kelem/s |
| has_tag | 782.12 ps | 1.2786 Gelem/s | 626.71 ps | 1.5956 Gelem/s |
| has_model | 976.58 ps | 1.0240 Gelem/s | 441.79 ps | 2.2635 Gelem/s |
| has_tool | 781.72 ps | 1.2792 Gelem/s | 635.13 ps | 1.5745 Gelem/s |
| has_gpu | 325.46 ps | 3.0726 Gelem/s | 203.16 ps | 4.9223 Gelem/s |

### Capability Announcement

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 407.16 ns | 2.4560 Melem/s | 1.5682 us | 637.66 Kelem/s |
| serialize | 1.1187 us | 893.93 Kelem/s | 832.23 ns | 1.2016 Melem/s |
| deserialize | 1.9950 us | 501.25 Kelem/s | 2.4731 us | 404.36 Kelem/s |
| is_expired | 27.053 ns | 36.964 Melem/s | 22.463 ns | 44.519 Melem/s |

### Capability Serialization (Simple vs Complex)

| Type | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| Simple | Serialize | 19.917 ns | 50.208 Melem/s | 38.090 ns | 26.254 Melem/s |
| Simple | Deserialize | 5.0161 ns | 199.36 Melem/s | 9.7975 ns | 102.07 Melem/s |
| Complex | Serialize | 42.121 ns | 23.741 Melem/s | 39.913 ns | 25.055 Melem/s |
| Complex | Deserialize | 165.62 ns | 6.0380 Melem/s | 241.75 ns | 4.1365 Melem/s |

### Capability Filter

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 10.474 ns | 95.476 Melem/s | 3.6578 ns | 273.39 Melem/s |
| require_gpu | 4.2293 ns | 236.45 Melem/s | 1.9383 ns | 515.92 Melem/s |
| gpu_vendor | 3.9099 ns | 255.76 Melem/s | 2.1797 ns | 458.78 Melem/s |
| min_memory | 3.9248 ns | 254.79 Melem/s | 1.9416 ns | 515.05 Melem/s |
| complex | 10.754 ns | 92.990 Melem/s | 6.1297 ns | 163.14 Melem/s |
| no_match | 3.2559 ns | 307.14 Melem/s | 1.8707 ns | 534.55 Melem/s |

### Capability Index (Insert)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 100 nodes | 124.08 us | 805.93 Kelem/s | 191.49 us | 522.21 Kelem/s |
| 1,000 nodes | 1.3514 ms | 739.97 Kelem/s | 1.5759 ms | 634.54 Kelem/s |
| 10,000 nodes | 22.069 ms | 453.12 Kelem/s | 20.038 ms | 499.06 Kelem/s |

### Capability Index (Query, 10,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 260.79 us | 3.8345 Kelem/s | 190.15 us | 5.2591 Kelem/s |
| require_gpu | 408.14 us | 2.4501 Kelem/s | 209.24 us | 4.7791 Kelem/s |
| gpu_vendor | 1.0347 ms | 966.42 elem/s | 556.54 us | 1.7968 Kelem/s |
| min_memory | 997.65 us | 1.0024 Kelem/s | 544.70 us | 1.8359 Kelem/s |
| complex | 744.50 us | 1.3432 Kelem/s | 358.56 us | 2.7889 Kelem/s |
| model | 109.38 us | 9.1425 Kelem/s | 104.98 us | 9.5254 Kelem/s |
| tool | 1.0299 ms | 970.95 elem/s | 537.51 us | 1.8604 Kelem/s |
| no_results | 23.791 ns | 42.033 Melem/s | 26.752 ns | 37.381 Melem/s |

### Capability Index (Find Best)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Simple | 477.66 us | 2.0935 Kelem/s | 376.63 us | 2.6551 Kelem/s |
| With preferences | 870.64 us | 1.1486 Kelem/s | 582.87 us | 1.7157 Kelem/s |

### Capability Search (1,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| find_with_gpu | 18.423 us | 54.279 Kelem/s | 31.728 us | 31.518 Kelem/s |
| find_by_tool (Python) | 33.361 us | 29.975 Kelem/s | 62.902 us | 15.898 Kelem/s |
| find_by_tool (Rust) | 42.520 us | 23.518 Kelem/s | 74.991 us | 13.335 Kelem/s |

### Capability Index Scaling

| Nodes | Query Type | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 1,000 | Tag | 13.227 us | 75.604 Kelem/s | 10.841 us | 92.243 Kelem/s |
| 1,000 | Complex | 42.524 us | 23.516 Kelem/s | 28.963 us | 34.527 Kelem/s |
| 5,000 | Tag | 76.989 us | 12.989 Kelem/s | 58.625 us | 17.057 Kelem/s |
| 5,000 | Complex | 287.24 us | 3.4814 Kelem/s | 144.55 us | 6.9178 Kelem/s |
| 10,000 | Tag | 248.77 us | 4.0198 Kelem/s | 188.14 us | 5.3152 Kelem/s |
| 10,000 | Complex | 676.70 us | 1.4778 Kelem/s | 363.66 us | 2.7498 Kelem/s |
| 50,000 | Tag | 3.2346 ms | 309.15 elem/s | 1.8541 ms | 539.34 elem/s |
| 50,000 | Complex | 4.9312 ms | 202.79 elem/s | 3.1590 ms | 316.56 elem/s |

### Concurrent Capability Index

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Insert | 437.42 us | 4.5723 Melem/s | 694.33 us | 2.8805 Melem/s |
| 4 | Query | 474.96 ms | 4.2109 Kelem/s | 270.68 ms | 7.3888 Kelem/s |
| 8 | Insert | 686.91 us | 5.8232 Melem/s | 1.4484 ms | 2.7617 Melem/s |
| 8 | Query | 604.45 ms | 6.6176 Kelem/s | 347.95 ms | 11.496 Kelem/s |
| 16 | Insert | 1.0935 ms | 7.3163 Melem/s | 1.9625 ms | 4.0765 Melem/s |
| 16 | Query | 1.1437 s | 6.9951 Kelem/s | 465.83 ms | 17.174 Kelem/s |

## Running Benchmarks

```bash
cargo bench --features bltp --bench bltp
```

For native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo bench --features bltp --bench bltp
```

## Key Insights

1. **Header serialize/deserialize runs at ~800M ops/sec** (i9) / ~490M ops/sec (M1) — sub-2ns per operation
2. **Routing header operations achieve 1.7–4.2G ops/sec** — sub-nanosecond serialization
3. **Thread-local pool eliminates contention** — up to 18x faster than shared pool at 32 threads
4. **Capability filters run at 200–500M ops/sec** — fast enough for inline packet decisions
5. **Circuit breaker checks are ~10ns** — negligible overhead per packet
6. **Event frame write scales with payload** — 3 GiB/s at 64B, 43–68 GiB/s at 4KB
7. **Multi-hop forwarding adds ~30–60ns per hop** — linear scaling, no amplification
8. **i9-14900K is 1.5–2x faster on serialization/routing** due to higher single-thread clock; **M1 Max matches or wins on encryption and small-payload framing**
