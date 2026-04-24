# Net Benchmarks

Performance benchmarks for the Net Rust core and Net transport layer.

Benchmarks accurate as of 2026-04-24.

**Test Systems:**
- Apple M1 Max, macOS
- Intel i9-14900K @5GHz, Windows 11

The latest 14900K run is a subset of the full bench suite — where a benchmark did not re-run on Windows, the i9 column retains the 2026-04-23 value and is so marked by the absence of change.

## Net Header Operations

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 1.9810 ns | 504.80 Melem/s | 1.3040 ns | 766.88 Melem/s |
| Deserialize | 2.1056 ns | 474.93 Melem/s | 1.5163 ns | 659.49 Melem/s |
| Roundtrip | 2.1049 ns | 475.08 Melem/s | 1.5129 ns | 661.00 Melem/s |
| AAD generation | 2.0312 ns | 492.33 Melem/s | 1.3773 ns | 726.07 Melem/s |

## Event Frame Serialization

### Single Write

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 18.281 ns | 3.2606 GiB/s | 35.381 ns | 1.6847 GiB/s |
| 256B | 50.357 ns | 4.7346 GiB/s | 35.402 ns | 6.7346 GiB/s |
| 1KB | 36.119 ns | 26.403 GiB/s | 35.436 ns | 26.912 GiB/s |
| 4KB | 77.877 ns | 48.983 GiB/s | 50.736 ns | 75.187 GiB/s |

### Batch Write (64B events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 events | 18.311 ns | 3.2551 GiB/s | 27.900 ns | 2.1364 GiB/s |
| 10 events | 72.303 ns | 8.2437 GiB/s | 55.337 ns | 10.771 GiB/s |
| 50 events | 148.02 ns | 20.134 GiB/s | 145.63 ns | 20.464 GiB/s |
| 100 events | 273.29 ns | 21.810 GiB/s | 255.28 ns | 23.348 GiB/s |

### Batch Read

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Read batch (10 events) | 140.05 ns | 71.403 Melem/s | 163.76 ns | 61.064 Melem/s |

## Packet Pool (Zero-Allocation)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool get+return | 38.402 ns | 26.041 Melem/s | 53.122 ns | 18.825 Melem/s |
| Thread-local get+return | 82.337 ns | 12.145 Melem/s | 64.089 ns | 15.603 Melem/s |

### Pool Comparison (10x cycles)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool 10x | 341.46 ns | 2.9286 Melem/s | 511.15 ns | 1.9564 Melem/s |
| Thread-local 10x | 1.0996 us | 909.40 Kelem/s | 808.74 ns | 1.2365 Melem/s |

## Packet Build

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 event | 483.41 ns | 126.26 MiB/s | 1.1343 us | 53.809 MiB/s |
| 10 events | 1.8467 us | 330.51 MiB/s | 1.5098 us | 404.27 MiB/s |
| 50 events | 8.2177 us | 371.36 MiB/s | 2.9381 us | 1.0144 GiB/s |

## Encryption (ChaCha20-Poly1305)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 483.38 ns | 126.27 MiB/s | 1.1370 us | 53.682 MiB/s |
| 256B | 922.20 ns | 264.74 MiB/s | 1.2018 us | 203.15 MiB/s |
| 1KB | 2.6930 us | 362.64 MiB/s | 1.5765 us | 619.45 MiB/s |
| 4KB | 9.7720 us | 399.74 MiB/s | 3.1348 us | 1.2169 GiB/s |

### End-to-End Packet Build (50 events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool | 8.2247 us | 371.05 MiB/s | 2.9299 us | 1.0172 GiB/s |
| Thread-local pool | 8.2156 us | 371.46 MiB/s | 2.8743 us | 1.0369 GiB/s |

## Adaptive Batcher Overhead

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| optimal_size() | 977.83 ps | 1.0227 Gelem/s | 804.54 ps | 1.2429 Gelem/s |
| record() | 3.8846 ns | 257.42 Melem/s | 14.312 ns | 69.870 Melem/s |
| full_cycle | 4.4019 ns | 227.17 Melem/s | 8.0530 ns | 124.18 Melem/s |

## Key Generation

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Keypair generate | 12.870 us | 77.702 Kelem/s | 26.526 us | 37.699 Kelem/s |

## Multi-threaded Packet Build (1000 packets/thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 1.8479 ms | 4.3293 Melem/s | 1.6198 ms | 4.9390 Melem/s |
| 8 | Thread-local | 891.61 us | 8.9725 Melem/s | 1.4381 ms | 5.5628 Melem/s |
| 16 | Shared | 4.4759 ms | 3.5747 Melem/s | 2.5714 ms | 6.2222 Melem/s |
| 16 | Thread-local | 1.7187 ms | 9.3093 Melem/s | 1.8659 ms | 8.5751 Melem/s |
| 24 | Shared | 7.4488 ms | 3.2220 Melem/s | 3.8969 ms | 6.1588 Melem/s |
| 24 | Thread-local | 2.7912 ms | 8.5985 Melem/s | 2.4499 ms | 9.7963 Melem/s |
| 32 | Shared | 9.9502 ms | 3.2160 Melem/s | 5.3719 ms | 5.9569 Melem/s |
| 32 | Thread-local | 3.2905 ms | 9.7249 Melem/s | 3.0901 ms | 10.356 Melem/s |

### Pool Contention (10,000 acquire/release per thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 17.686 ms | 4.5234 Melem/s | 9.6604 ms | 8.2813 Melem/s |
| 8 | Thread-local | 1.1269 ms | 70.989 Melem/s | 1.1538 ms | 69.335 Melem/s |
| 16 | Shared | 41.722 ms | 3.8349 Melem/s | 20.305 ms | 7.8797 Melem/s |
| 16 | Thread-local | 2.3029 ms | 69.477 Melem/s | 1.8001 ms | 88.885 Melem/s |
| 24 | Shared | 61.914 ms | 3.8763 Melem/s | 35.825 ms | 6.6992 Melem/s |
| 24 | Thread-local | 3.3096 ms | 72.516 Melem/s | 2.1117 ms | 113.65 Melem/s |
| 32 | Shared | 87.756 ms | 3.6465 Melem/s | 46.578 ms | 6.8702 Melem/s |
| 32 | Thread-local | 4.2160 ms | 75.901 Melem/s | 2.8109 ms | 113.84 Melem/s |

### Mixed Frame Sizes (64B/256B/1KB rotation)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 1.4045 ms | 8.5439 Melem/s | 1.0543 ms | 11.382 Melem/s |
| 8 | Thread-local | 1.0831 ms | 11.079 Melem/s | 949.77 us | 12.635 Melem/s |
| 16 | Shared | 3.0489 ms | 7.8717 Melem/s | 1.5601 ms | 15.384 Melem/s |
| 16 | Thread-local | 2.0598 ms | 11.651 Melem/s | 1.3516 ms | 17.756 Melem/s |
| 24 | Shared | 4.7076 ms | 7.6471 Melem/s | 2.4229 ms | 14.858 Melem/s |
| 24 | Thread-local | 3.0188 ms | 11.925 Melem/s | 1.7637 ms | 20.411 Melem/s |
| 32 | Shared | 6.1237 ms | 7.8384 Melem/s | 3.2392 ms | 14.818 Melem/s |
| 32 | Thread-local | 3.9612 ms | 12.117 Melem/s | 2.2101 ms | 21.718 Melem/s |

### Throughput Scaling (Thread-local Pool)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 threads | 6.7284 ms | 297.25 Kelem/s | 3.6150 ms | 553.24 Kelem/s |
| 2 threads | 6.9914 ms | 572.13 Kelem/s | 3.6904 ms | 1.0839 Melem/s |
| 4 threads | 7.4301 ms | 1.0767 Melem/s | 3.7270 ms | 2.1465 Melem/s |
| 8 threads | 7.8672 ms | 2.0338 Melem/s | 4.4588 ms | 3.5884 Melem/s |
| 16 threads | 15.564 ms | 2.0561 Melem/s | 5.4653 ms | 5.8551 Melem/s |
| 24 threads | 23.101 ms | 2.0779 Melem/s | 7.3116 ms | 6.5649 Melem/s |
| 32 threads | 29.997 ms | 2.1335 Melem/s | 10.059 ms | 6.3627 Melem/s |

## Routing

### Routing Header

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 628.55 ps | 1.5910 Gelem/s | 466.02 ps | 2.1458 Gelem/s |
| Deserialize | 941.97 ps | 1.0616 Gelem/s | 721.06 ps | 1.3868 Gelem/s |
| Roundtrip | 938.40 ps | 1.0656 Gelem/s | 720.67 ps | 1.3876 Gelem/s |
| Forward | 573.68 ps | 1.7431 Gelem/s | 506.13 ps | 1.9758 Gelem/s |

### Routing Table

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| lookup_hit | 37.772 ns | 26.474 Melem/s | 38.556 ns | 25.936 Melem/s |
| lookup_miss | 14.917 ns | 67.036 Melem/s | 17.821 ns | 56.112 Melem/s |
| is_local | 314.83 ps | 3.1763 Gelem/s | 202.26 ps | 4.9441 Gelem/s |
| add_route | 242.95 ns | 4.1160 Melem/s | 208.69 ns | 4.7917 Melem/s |
| record_in | 50.047 ns | 19.981 Melem/s | 40.591 ns | 24.636 Melem/s |
| record_out | 20.175 ns | 49.566 Melem/s | 21.115 ns | 47.359 Melem/s |
| aggregate_stats | 2.0789 us | 481.03 Kelem/s | 8.0750 us | 123.84 Kelem/s |

### Concurrent Routing Lookup

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Lookup | 159.50 us | 25.078 Melem/s | 176.99 us | 22.600 Melem/s |
| 4 | Stats | 293.89 us | 13.611 Melem/s | 224.60 us | 17.809 Melem/s |
| 8 | Lookup | 245.30 us | 32.613 Melem/s | 279.97 us | 28.575 Melem/s |
| 8 | Stats | 418.22 us | 19.129 Melem/s | 328.53 us | 24.351 Melem/s |
| 16 | Lookup | 428.47 us | 37.342 Melem/s | 504.12 us | 31.738 Melem/s |
| 16 | Stats | 826.00 us | 19.371 Melem/s | 566.68 us | 28.235 Melem/s |

### Decision Pipeline

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| parse + lookup + forward | 39.590 ns | 25.259 Melem/s | 38.730 ns | 25.820 Melem/s |
| full with stats | 109.97 ns | 9.0935 Melem/s | 100.90 ns | 9.9105 Melem/s |

## Multi-hop Forwarding

### Packet Builder

| Payload | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | Build | 23.306 ns | 2.5574 GiB/s | 41.064 ns | 1.4515 GiB/s |
| 64B | Build (priority) | 21.188 ns | 2.8132 GiB/s | 30.060 ns | 1.9829 GiB/s |
| 256B | Build | 54.754 ns | 4.3544 GiB/s | 42.797 ns | 5.5709 GiB/s |
| 256B | Build (priority) | 52.233 ns | 4.5645 GiB/s | 31.894 ns | 7.4754 GiB/s |
| 1KB | Build | 42.852 ns | 22.255 GiB/s | 44.150 ns | 21.601 GiB/s |
| 1KB | Build (priority) | 39.308 ns | 24.262 GiB/s | 35.236 ns | 27.065 GiB/s |
| 4KB | Build | 82.808 ns | 46.067 GiB/s | 80.072 ns | 47.641 GiB/s |
| 4KB | Build (priority) | 80.646 ns | 47.302 GiB/s | 69.770 ns | 54.676 GiB/s |

### Chain Scaling (forward_chain)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 59.971 ns | 16.675 Melem/s | 53.321 ns | 18.754 Melem/s |
| 2 hops | 114.06 ns | 8.7677 Melem/s | 87.654 ns | 11.408 Melem/s |
| 3 hops | 159.79 ns | 6.2582 Melem/s | 121.82 ns | 8.2085 Melem/s |
| 4 hops | 238.17 ns | 4.1987 Melem/s | 156.27 ns | 6.3991 Melem/s |
| 5 hops | 275.93 ns | 3.6241 Melem/s | 190.77 ns | 5.2419 Melem/s |

### Hop Latency

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Single hop process | 1.4891 ns | 671.54 Melem/s | 936.23 ps | 1.0681 Gelem/s |
| Single hop full | 59.194 ns | 16.894 Melem/s | 33.514 ns | 29.839 Melem/s |

### Hop Scaling by Payload Size

| Payload | Hops | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | 1 hops | 29.600 ns | 2.0137 GiB/s | 51.335 ns | 1.1611 GiB/s |
| 64B | 2 hops | 52.720 ns | 1.1306 GiB/s | 82.718 ns | 737.87 MiB/s |
| 64B | 3 hops | 76.539 ns | 797.44 MiB/s | 116.64 ns | 523.28 MiB/s |
| 64B | 4 hops | 99.501 ns | 613.42 MiB/s | 146.93 ns | 415.40 MiB/s |
| 64B | 5 hops | 130.29 ns | 468.47 MiB/s | 180.47 ns | 338.20 MiB/s |
| 256B | 1 hops | 62.276 ns | 3.8284 GiB/s | 52.546 ns | 4.5373 GiB/s |
| 256B | 2 hops | 119.58 ns | 1.9937 GiB/s | 85.848 ns | 2.7772 GiB/s |
| 256B | 3 hops | 173.04 ns | 1.3778 GiB/s | 119.12 ns | 2.0015 GiB/s |
| 256B | 4 hops | 225.45 ns | 1.0575 GiB/s | 151.27 ns | 1.5761 GiB/s |
| 256B | 5 hops | 288.26 ns | 846.96 MiB/s | 187.38 ns | 1.2723 GiB/s |
| 1024B | 1 hops | 48.009 ns | 19.864 GiB/s | 54.025 ns | 17.653 GiB/s |
| 1024B | 2 hops | 112.77 ns | 8.4571 GiB/s | 88.689 ns | 10.753 GiB/s |
| 1024B | 3 hops | 162.73 ns | 5.8604 GiB/s | 124.24 ns | 7.6762 GiB/s |
| 1024B | 4 hops | 239.02 ns | 3.9899 GiB/s | 158.56 ns | 6.0145 GiB/s |
| 1024B | 5 hops | 264.29 ns | 3.6085 GiB/s | 196.44 ns | 4.8548 GiB/s |

### Route and Forward (with routing table lookup)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 183.21 ns | 5.4583 Melem/s | 154.06 ns | 6.4912 Melem/s |
| 2 hops | 349.64 ns | 2.8601 Melem/s | 288.89 ns | 3.4615 Melem/s |
| 3 hops | 518.40 ns | 1.9290 Melem/s | 422.21 ns | 2.3685 Melem/s |
| 4 hops | 703.95 ns | 1.4206 Melem/s | 558.40 ns | 1.7908 Melem/s |
| 5 hops | 884.50 ns | 1.1306 Melem/s | 693.01 ns | 1.4430 Melem/s |

### Concurrent Forwarding

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Forward | 687.43 us | 5.8187 Melem/s | 563.87 us | 7.0938 Melem/s |
| 8 | Forward | 983.59 us | 8.1335 Melem/s | 750.29 us | 10.663 Melem/s |
| 16 | Forward | 2.0707 ms | 7.7270 Melem/s | 1.3628 ms | 11.740 Melem/s |

## Swarm / Discovery

### Pingwave

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 787.06 ps | 1.2706 Gelem/s | 534.31 ps | 1.8716 Gelem/s |
| Deserialize | 954.16 ps | 1.0480 Gelem/s | 634.49 ps | 1.5761 Gelem/s |
| Roundtrip | 956.03 ps | 1.0460 Gelem/s | 633.79 ps | 1.5778 Gelem/s |
| Forward | 634.22 ps | 1.5767 Gelem/s | 525.52 ps | 1.9029 Gelem/s |

### Local Graph

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create_pingwave | 2.1087 ns | 474.23 Melem/s | 5.0295 ns | 198.83 Melem/s |
| on_pingwave_new | 122.80 ns | 8.1433 Melem/s | 179.72 ns | 5.5641 Melem/s |
| on_pingwave_duplicate | 34.336 ns | 29.124 Melem/s | 16.687 ns | 59.929 Melem/s |
| get_node | 26.057 ns | 38.377 Melem/s | 14.762 ns | 67.743 Melem/s |
| node_count | 200.50 ns | 4.9876 Melem/s | 934.80 ns | 1.0697 Melem/s |
| stats | 600.73 ns | 1.6647 Melem/s | 2.8166 us | 355.04 Kelem/s |

### Graph Scaling

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | all_nodes | 2.4481 us | 40.848 Melem/s | 7.5661 us | 13.217 Melem/s |
| 100 | nodes_within_hops | 2.8775 us | 34.752 Melem/s | 7.5908 us | 13.174 Melem/s |
| 500 | all_nodes | 8.1750 us | 61.162 Melem/s | 16.336 us | 30.607 Melem/s |
| 500 | nodes_within_hops | 9.6884 us | 51.608 Melem/s | 16.575 us | 30.165 Melem/s |
| 1,000 | all_nodes | 199.99 us | 5.0001 Melem/s | 27.246 us | 36.702 Melem/s |
| 1,000 | nodes_within_hops | 203.97 us | 4.9028 Melem/s | 27.232 us | 36.721 Melem/s |
| 5,000 | all_nodes | 264.12 us | 18.931 Melem/s | 230.49 us | 21.693 Melem/s |
| 5,000 | nodes_within_hops | 277.80 us | 17.999 Melem/s | 230.87 us | 21.657 Melem/s |

### Path Finding

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| path_1_hop | 1.5606 us | 640.78 Kelem/s | 5.3652 us | 186.39 Kelem/s |
| path_2_hops | 1.6042 us | 623.37 Kelem/s | 5.7191 us | 174.85 Kelem/s |
| path_4_hops | 1.8562 us | 538.73 Kelem/s | 5.8860 us | 169.90 Kelem/s |
| path_not_found | 1.7535 us | 570.28 Kelem/s | 5.7897 us | 172.72 Kelem/s |
| path_complex_graph | 215.84 us | 4.6330 Kelem/s | 181.59 us | 5.5069 Kelem/s |

### Concurrent Pingwave Processing

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Pingwave | 112.21 us | 17.824 Melem/s | 168.66 us | 11.858 Melem/s |
| 8 | Pingwave | 180.32 us | 22.183 Melem/s | 272.86 us | 14.659 Melem/s |
| 16 | Pingwave | 316.51 us | 25.275 Melem/s | 512.19 us | 15.619 Melem/s |

## Failure Detection

### Failure Detector

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| heartbeat_existing | 29.328 ns | 34.097 Melem/s | 35.349 ns | 28.289 Melem/s |
| heartbeat_new | 257.27 ns | 3.8870 Melem/s | 207.59 ns | 4.8172 Melem/s |
| status_check | 14.099 ns | 70.925 Melem/s | 13.525 ns | 73.939 Melem/s |
| check_all | 343.39 ms | 2.9122 elem/s | 664.73 ms | 1.5044 elem/s |
| stats | 80.558 ms | 12.413 elem/s | 192.16 ms | 5.2039 elem/s |

### Circuit Breaker

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| allow_closed | 13.481 ns | 74.177 Melem/s | 10.300 ns | 97.085 Melem/s |
| record_success | 9.6668 ns | 103.45 Melem/s | 8.8206 ns | 113.37 Melem/s |
| record_failure | 9.6195 ns | 103.96 Melem/s | 9.9178 ns | 100.83 Melem/s |
| state | 13.482 ns | 74.173 Melem/s | 9.8217 ns | 101.82 Melem/s |

### Recovery Manager

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| on_failure_with_alternates | 268.97 ns | 3.7179 Melem/s | 264.70 ns | 3.7779 Melem/s |
| on_failure_no_alternates | 296.47 ns | 3.3731 Melem/s | 229.99 ns | 4.3480 Melem/s |
| get_action | 38.066 ns | 26.270 Melem/s | 37.769 ns | 26.477 Melem/s |
| is_failed | 13.021 ns | 76.798 Melem/s | 12.850 ns | 77.820 Melem/s |
| on_recovery | 106.60 ns | 9.3808 Melem/s | 124.49 ns | 8.0325 Melem/s |
| stats | 703.02 ps | 1.4224 Gelem/s | 1.1536 ns | 866.87 Melem/s |

### Full Recovery Cycle

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Fail + recover cycle | 306.54 ns | 3.2622 Melem/s | 260.79 ns | 3.8345 Melem/s |

### Loss Simulator

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1% | 2.8007 ns | 357.05 Melem/s | 11.071 ns | 90.323 Melem/s |
| 5% | 3.1638 ns | 316.08 Melem/s | 11.475 ns | 87.143 Melem/s |
| 10% | 3.6334 ns | 275.22 Melem/s | 11.991 ns | 83.397 Melem/s |
| 20% | 4.5966 ns | 217.55 Melem/s | 13.033 ns | 76.730 Melem/s |
| burst | 2.9429 ns | 339.80 Melem/s | 11.578 ns | 86.372 Melem/s |

### Failure Scaling (check_all)

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | check_all | 4.8177 us | 20.757 Melem/s | 8.9082 us | 11.226 Melem/s |
| 100 | healthy_nodes | 1.7240 us | 58.006 Melem/s | 6.3261 us | 15.807 Melem/s |
| 500 | check_all | 20.939 us | 23.878 Melem/s | 23.251 us | 21.504 Melem/s |
| 500 | healthy_nodes | 5.4519 us | 91.711 Melem/s | 10.023 us | 49.884 Melem/s |
| 1,000 | check_all | 41.076 us | 24.345 Melem/s | 41.233 us | 24.253 Melem/s |
| 1,000 | healthy_nodes | 10.292 us | 97.159 Melem/s | 14.875 us | 67.226 Melem/s |
| 5,000 | check_all | 204.17 us | 24.489 Melem/s | 185.63 us | 26.935 Melem/s |
| 5,000 | healthy_nodes | 50.173 us | 99.655 Melem/s | 53.617 us | 93.255 Melem/s |

### Concurrent Heartbeats

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Heartbeat | 183.40 us | 10.905 Melem/s | 204.32 us | 9.7888 Melem/s |
| 8 | Heartbeat | 260.27 us | 15.369 Melem/s | 315.26 us | 12.688 Melem/s |
| 16 | Heartbeat | 468.32 us | 17.082 Melem/s | 563.77 us | 14.190 Melem/s |

## Stream Multiplexing

| Streams | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 10 | Lookup | 292.53 ns | 34.184 Melem/s | 348.55 ns | 28.690 Melem/s |
| 10 | Stats | 487.82 ns | 20.500 Melem/s | 410.06 ns | 24.387 Melem/s |
| 100 | Lookup | 2.9266 us | 34.170 Melem/s | 3.4441 us | 29.035 Melem/s |
| 100 | Stats | 4.9594 us | 20.164 Melem/s | 4.1210 us | 24.266 Melem/s |
| 1,000 | Lookup | 29.979 us | 33.357 Melem/s | 36.136 us | 27.673 Melem/s |
| 1,000 | Stats | 53.619 us | 18.650 Melem/s | 44.654 us | 22.395 Melem/s |
| 10,000 | Lookup | 293.32 us | 34.093 Melem/s | 391.95 us | 25.513 Melem/s |
| 10,000 | Stats | 569.72 us | 17.553 Melem/s | 471.39 us | 21.214 Melem/s |

## Fair Scheduler

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Creation | 286.97 ns | 3.4847 Melem/s | 1.5589 us | 641.48 Kelem/s |
| Stream count (empty) | 200.63 ns | 4.9842 Melem/s | 960.19 ns | 1.0415 Melem/s |
| Total queued | 312.66 ps | 3.1984 Gelem/s | 200.85 ps | 4.9788 Gelem/s |
| Cleanup (empty) | 201.57 ns | 4.9612 Melem/s | 1.2886 us | 776.03 Kelem/s |

## Capability System

### CapabilitySet

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 518.55 ns | 1.9285 Melem/s | 746.15 ns | 1.3402 Melem/s |
| serialize | 922.25 ns | 1.0843 Melem/s | 668.08 ns | 1.4968 Melem/s |
| deserialize | 1.7187 us | 581.84 Kelem/s | 3.0060 us | 332.67 Kelem/s |
| roundtrip | 2.6815 us | 372.93 Kelem/s | 4.5812 us | 218.28 Kelem/s |
| has_tag | 750.07 ps | 1.3332 Gelem/s | 576.00 ps | 1.7361 Gelem/s |
| has_model | 937.00 ps | 1.0672 Gelem/s | 445.58 ps | 2.2443 Gelem/s |
| has_tool | 750.08 ps | 1.3332 Gelem/s | 690.86 ps | 1.4475 Gelem/s |
| has_gpu | 312.52 ps | 3.1998 Gelem/s | 191.72 ps | 5.2159 Gelem/s |

### Capability Announcement

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 376.79 ns | 2.6540 Melem/s | 1.5135 us | 660.71 Kelem/s |
| serialize | 1.2230 us | 817.68 Kelem/s | 1.6651 us | 600.58 Kelem/s |
| deserialize | 2.1078 us | 474.44 Kelem/s | 2.6587 us | 376.13 Kelem/s |
| is_expired | 25.353 ns | 39.443 Melem/s | 21.838 ns | 45.792 Melem/s |

### Capability Serialization (Simple vs Complex)

| Type | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| Simple | Serialize | 19.169 ns | 52.167 Melem/s | 37.370 ns | 26.759 Melem/s |
| Simple | Deserialize | 4.7556 ns | 210.28 Melem/s | 9.0720 ns | 110.23 Melem/s |
| Complex | Serialize | 41.215 ns | 24.263 Melem/s | 38.043 ns | 26.286 Melem/s |
| Complex | Deserialize | 153.80 ns | 6.5018 Melem/s | 233.99 ns | 4.2736 Melem/s |

### Capability Filter

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 9.9901 ns | 100.10 Melem/s | 3.4359 ns | 291.04 Melem/s |
| require_gpu | 4.0606 ns | 246.27 Melem/s | 1.8113 ns | 552.08 Melem/s |
| gpu_vendor | 3.7491 ns | 266.73 Melem/s | 2.0124 ns | 496.91 Melem/s |
| min_memory | 3.7492 ns | 266.72 Melem/s | 1.8126 ns | 551.69 Melem/s |
| complex | 10.624 ns | 94.129 Melem/s | 5.8556 ns | 170.78 Melem/s |
| no_match | 3.1247 ns | 320.03 Melem/s | 1.9699 ns | 507.64 Melem/s |

### Capability Index (Insert)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 100 nodes | 124.56 us | 802.80 Kelem/s | 189.33 us | 528.18 Kelem/s |
| 1,000 nodes | 1.3891 ms | 719.87 Kelem/s | 1.4997 ms | 666.82 Kelem/s |
| 10,000 nodes | 18.595 ms | 537.79 Kelem/s | 16.834 ms | 594.03 Kelem/s |

### Capability Index (Query, 10,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 187.86 us | 5.3231 Kelem/s | 179.32 us | 5.5765 Kelem/s |
| require_gpu | 231.82 us | 4.3137 Kelem/s | 188.52 us | 5.3044 Kelem/s |
| gpu_vendor | 734.28 us | 1.3619 Kelem/s | 518.78 us | 1.9276 Kelem/s |
| min_memory | 773.99 us | 1.2920 Kelem/s | 539.60 us | 1.8532 Kelem/s |
| complex | 491.11 us | 2.0362 Kelem/s | 342.95 us | 2.9159 Kelem/s |
| model | 100.25 us | 9.9749 Kelem/s | 95.210 us | 10.503 Kelem/s |
| tool | 628.35 us | 1.5915 Kelem/s | 493.70 us | 2.0255 Kelem/s |
| no_results | 22.669 ns | 44.113 Melem/s | 26.413 ns | 37.860 Melem/s |

### Capability Index (Find Best)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Simple | 393.68 us | 2.5401 Kelem/s | 343.25 us | 2.9134 Kelem/s |
| With preferences | 714.53 us | 1.3995 Kelem/s | 546.49 us | 1.8299 Kelem/s |

### Capability Search (1,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| find_with_gpu | 17.189 us | 58.175 Kelem/s | 30.636 us | 32.641 Kelem/s |
| find_by_tool (Python) | 30.640 us | 32.637 Kelem/s | 60.779 us | 16.453 Kelem/s |
| find_by_tool (Rust) | 39.641 us | 25.227 Kelem/s | 81.270 us | 12.305 Kelem/s |

### Capability Index Scaling

| Nodes | Query Type | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 1,000 | Tag | 12.538 us | 79.755 Kelem/s | 10.593 us | 94.398 Kelem/s |
| 1,000 | Complex | 40.438 us | 24.729 Kelem/s | 27.860 us | 35.893 Kelem/s |
| 5,000 | Tag | 70.401 us | 14.204 Kelem/s | 55.177 us | 18.123 Kelem/s |
| 5,000 | Complex | 205.55 us | 4.8651 Kelem/s | 136.36 us | 7.3334 Kelem/s |
| 10,000 | Tag | 175.94 us | 5.6837 Kelem/s | 173.09 us | 5.7774 Kelem/s |
| 10,000 | Complex | 459.95 us | 2.1742 Kelem/s | 346.43 us | 2.8866 Kelem/s |
| 50,000 | Tag | 2.4207 ms | 413.11 elem/s | 1.2576 ms | 795.15 elem/s |
| 50,000 | Complex | 3.6456 ms | 274.30 elem/s | 2.2123 ms | 452.02 elem/s |

### Concurrent Capability Index

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Insert | 394.61 us | 5.0683 Melem/s | 644.18 us | 3.1047 Melem/s |
| 4 | Query | 408.90 ms | 4.8911 Kelem/s | 246.76 ms | 8.1051 Kelem/s |
| 8 | Insert | 469.90 us | 8.5125 Melem/s | 883.34 us | 4.5283 Melem/s |
| 8 | Query | 524.92 ms | 7.6203 Kelem/s | 298.07 ms | 13.420 Kelem/s |
| 16 | Insert | 951.61 us | 8.4068 Melem/s | 1.6118 ms | 4.9633 Melem/s |
| 16 | Query | 1.0455 s | 7.6522 Kelem/s | 437.50 ms | 18.286 Kelem/s |

## Running Benchmarks

```bash
cargo bench --features net --bench net
```

For native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo bench --features net --bench net
```

To re-parse raw criterion output into structured markdown, use the helper script in `benchmarks/`:

```bash
./benchmarks/parse_criterion.py BENCHMARK_RESULTS_M1_MAX.md /tmp/m1_max_parsed.md
```

## Key Insights

1. **Header serialize/deserialize runs at ~660–770M ops/sec** (i9) / ~475–505M ops/sec (M1) — sub-2ns per operation
2. **Routing header operations achieve ~1.06–2.15G ops/sec** — sub-nanosecond serialization
3. **Thread-local pool eliminates contention** — up to ~21x faster than shared pool at 32 threads (M1: 75.9M vs 3.65M ops/sec; i9: 113.8M vs 6.87M ops/sec)
4. **Capability filters run at ~94–580M ops/sec** — fast enough for inline packet decisions
5. **Circuit breaker checks are ~10ns** — negligible overhead per packet
6. **Event frame write scales with payload** — ~1.7–3.3 GiB/s at 64B, ~49 GiB/s on M1 / ~75 GiB/s on i9 at 4KB
7. **Multi-hop forwarding adds ~30–60ns per hop** — linear scaling, no amplification
8. **i9-14900K wins on large-payload write and batch throughput** (+55% at 4KB single-write); **M1 Max wins on small-frame single-write and routing-table lookup_hit**. Routing-table lookup_hit is now essentially tied (~38 ns on both platforms) where the i9 used to trail.
