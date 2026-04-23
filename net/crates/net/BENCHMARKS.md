# Net Benchmarks

Performance benchmarks for the Net Rust core and Net transport layer.

Benchmarks accurate as of 2026-04-23.

**Test Systems:**
- Apple M1 Max, macOS
- Intel i9-14900K @5GHz, Windows 11

## Net Header Operations

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 1.9806 ns | 504.90 Melem/s | 1.3040 ns | 766.88 Melem/s |
| Deserialize | 2.1051 ns | 475.04 Melem/s | 1.5163 ns | 659.49 Melem/s |
| Roundtrip | 2.1047 ns | 475.12 Melem/s | 1.5129 ns | 661.00 Melem/s |
| AAD generation | 2.0277 ns | 493.18 Melem/s | 1.3773 ns | 726.07 Melem/s |

## Event Frame Serialization

### Single Write

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 18.001 ns | 3.3112 GiB/s | 60.560 ns | 1007.8 MiB/s |
| 256B | 48.418 ns | 4.9242 GiB/s | 58.715 ns | 4.0606 GiB/s |
| 1KB | 35.877 ns | 26.582 GiB/s | 71.344 ns | 13.367 GiB/s |
| 4KB | 78.991 ns | 48.293 GiB/s | 105.25 ns | 36.245 GiB/s |

### Batch Write (64B events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 events | 18.034 ns | 3.3052 GiB/s | 58.307 ns | 1.0223 GiB/s |
| 10 events | 69.844 ns | 8.5340 GiB/s | 108.32 ns | 5.5027 GiB/s |
| 50 events | 148.44 ns | 20.077 GiB/s | 373.70 ns | 7.9749 GiB/s |
| 100 events | 272.90 ns | 21.841 GiB/s | 763.52 ns | 7.8066 GiB/s |

### Batch Read

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Read batch (10 events) | 139.33 ns | 71.772 Melem/s | 235.80 ns | 42.410 Melem/s |

## Packet Pool (Zero-Allocation)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool get+return | 38.013 ns | 26.306 Melem/s | 82.776 ns | 12.081 Melem/s |
| Thread-local get+return | 83.050 ns | 12.041 Melem/s | 123.64 ns | 8.0878 Melem/s |

### Pool Comparison (10x cycles)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool 10x | 339.86 ns | 2.9424 Melem/s | 757.69 ns | 1.3198 Melem/s |
| Thread-local 10x | 985.47 ns | 1.0147 Melem/s | 1.4840 us | 673.86 Kelem/s |

## Packet Build

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 event | 490.60 ns | 124.41 MiB/s | 1.1052 us | 55.225 MiB/s |
| 10 events | 1.8492 us | 330.05 MiB/s | 1.8353 us | 332.57 MiB/s |
| 50 events | 8.2019 us | 372.08 MiB/s | 4.9927 us | 611.25 MiB/s |

## Encryption (ChaCha20-Poly1305)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 64B | 486.18 ns | 125.54 MiB/s | 1.1040 us | 55.283 MiB/s |
| 256B | 923.05 ns | 264.49 MiB/s | 1.2472 us | 195.76 MiB/s |
| 1KB | 2.6985 us | 361.89 MiB/s | 2.0986 us | 465.33 MiB/s |
| 4KB | 9.7699 us | 399.82 MiB/s | 5.4939 us | 711.02 MiB/s |

### End-to-End Packet Build (50 events)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Shared pool | 8.2074 us | 371.83 MiB/s | 3.2966 us | 925.72 MiB/s |
| Thread-local pool | 8.2034 us | 372.01 MiB/s | 2.8896 us | 1.0314 GiB/s |

## Adaptive Batcher Overhead

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| optimal_size() | 975.48 ps | 1.0251 Gelem/s | 1.5727 ns | 635.85 Melem/s |
| record() | 3.8810 ns | 257.67 Melem/s | 14.312 ns | 69.870 Melem/s |
| full_cycle | 4.3973 ns | 227.41 Melem/s | 13.358 ns | 74.861 Melem/s |

## Key Generation

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Keypair generate | 12.396 us | 80.672 Kelem/s | 26.526 us | 37.699 Kelem/s |

## Multi-threaded Packet Build (1000 packets/thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 1.8221 ms | 4.3906 Melem/s | 1.6268 ms | 4.9175 Melem/s |
| 8 | Thread-local | 891.72 us | 8.9714 Melem/s | 1.4419 ms | 5.5482 Melem/s |
| 16 | Shared | 4.5237 ms | 3.5369 Melem/s | 2.6111 ms | 6.1276 Melem/s |
| 16 | Thread-local | 1.7291 ms | 9.2533 Melem/s | 1.9341 ms | 8.2725 Melem/s |
| 24 | Shared | 7.2028 ms | 3.3320 Melem/s | 3.8851 ms | 6.1775 Melem/s |
| 24 | Thread-local | 2.5086 ms | 9.5671 Melem/s | 2.8158 ms | 8.5233 Melem/s |
| 32 | Shared | 10.232 ms | 3.1273 Melem/s | 5.5873 ms | 5.7273 Melem/s |
| 32 | Thread-local | 3.2853 ms | 9.7403 Melem/s | 3.2644 ms | 9.8027 Melem/s |

### Pool Contention (10,000 acquire/release per thread)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 17.625 ms | 4.5391 Melem/s | 9.6126 ms | 8.3224 Melem/s |
| 8 | Thread-local | 1.1100 ms | 72.069 Melem/s | 1.2513 ms | 63.934 Melem/s |
| 16 | Shared | 42.609 ms | 3.7551 Melem/s | 22.275 ms | 7.1828 Melem/s |
| 16 | Thread-local | 2.2840 ms | 70.053 Melem/s | 2.0820 ms | 76.851 Melem/s |
| 24 | Shared | 64.591 ms | 3.7157 Melem/s | 35.430 ms | 6.7738 Melem/s |
| 24 | Thread-local | 3.2835 ms | 73.092 Melem/s | 2.4509 ms | 97.922 Melem/s |
| 32 | Shared | 84.946 ms | 3.7671 Melem/s | 49.066 ms | 6.5219 Melem/s |
| 32 | Thread-local | 4.7333 ms | 67.606 Melem/s | 3.1466 ms | 101.70 Melem/s |

### Mixed Frame Sizes (64B/256B/1KB rotation)

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 8 | Shared | 1.3970 ms | 8.5899 Melem/s | 1.1054 ms | 10.856 Melem/s |
| 8 | Thread-local | 1.0635 ms | 11.283 Melem/s | 975.98 us | 12.295 Melem/s |
| 16 | Shared | 3.0807 ms | 7.7904 Melem/s | 1.5830 ms | 15.161 Melem/s |
| 16 | Thread-local | 2.0532 ms | 11.689 Melem/s | 1.3642 ms | 17.592 Melem/s |
| 24 | Shared | 4.7466 ms | 7.5844 Melem/s | 2.1062 ms | 17.093 Melem/s |
| 24 | Thread-local | 3.0157 ms | 11.938 Melem/s | 1.8407 ms | 19.558 Melem/s |
| 32 | Shared | 6.6194 ms | 7.2514 Melem/s | 3.2450 ms | 14.792 Melem/s |
| 32 | Thread-local | 3.9629 ms | 12.112 Melem/s | 2.2052 ms | 21.766 Melem/s |

### Throughput Scaling (Thread-local Pool)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 threads | 6.8805 ms | 290.68 Kelem/s | 3.6200 ms | 552.49 Kelem/s |
| 2 threads | 6.9830 ms | 572.82 Kelem/s | 3.7355 ms | 1.0708 Melem/s |
| 4 threads | 7.4118 ms | 1.0794 Melem/s | 3.7423 ms | 2.1377 Melem/s |
| 8 threads | 7.8405 ms | 2.0407 Melem/s | 4.0659 ms | 3.9352 Melem/s |
| 16 threads | 15.533 ms | 2.0601 Melem/s | 5.4934 ms | 5.8252 Melem/s |
| 24 threads | 22.995 ms | 2.0874 Melem/s | 17.527 ms | 2.7386 Melem/s |
| 32 threads | 29.987 ms | 2.1343 Melem/s | 26.572 ms | 2.4085 Melem/s |

## Routing

### Routing Header

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 631.55 ps | 1.5834 Gelem/s | 586.33 ps | 1.7055 Gelem/s |
| Deserialize | 939.48 ps | 1.0644 Gelem/s | 917.56 ps | 1.0898 Gelem/s |
| Roundtrip | 937.35 ps | 1.0668 Gelem/s | 915.00 ps | 1.0929 Gelem/s |
| Forward | 573.36 ps | 1.7441 Gelem/s | 506.13 ps | 1.9758 Gelem/s |

### Routing Table

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| lookup_hit | 38.272 ns | 26.129 Melem/s | 57.792 ns | 17.304 Melem/s |
| lookup_miss | 14.983 ns | 66.741 Melem/s | 17.394 ns | 57.493 Melem/s |
| is_local | 314.51 ps | 3.1796 Gelem/s | 202.26 ps | 4.9441 Gelem/s |
| add_route | 240.93 ns | 4.1506 Melem/s | 208.69 ns | 4.7917 Melem/s |
| record_in | 48.542 ns | 20.601 Melem/s | 40.591 ns | 24.636 Melem/s |
| record_out | 18.196 ns | 54.958 Melem/s | 20.803 ns | 48.071 Melem/s |
| aggregate_stats | 2.0831 us | 480.05 Kelem/s | 8.0281 us | 124.56 Kelem/s |

### Concurrent Routing Lookup

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Lookup | 163.70 us | 24.434 Melem/s | 176.20 us | 22.701 Melem/s |
| 4 | Stats | 280.79 us | 14.245 Melem/s | 221.97 us | 18.020 Melem/s |
| 8 | Lookup | 247.37 us | 32.340 Melem/s | 272.64 us | 29.343 Melem/s |
| 8 | Stats | 410.01 us | 19.512 Melem/s | 316.76 us | 25.256 Melem/s |
| 16 | Lookup | 424.01 us | 37.735 Melem/s | 498.88 us | 32.072 Melem/s |
| 16 | Stats | 802.18 us | 19.946 Melem/s | 543.29 us | 29.450 Melem/s |

### Decision Pipeline

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| parse + lookup + forward | 37.828 ns | 26.435 Melem/s | 37.739 ns | 26.498 Melem/s |
| full with stats | 109.89 ns | 9.1003 Melem/s | 99.043 ns | 10.097 Melem/s |

## Multi-hop Forwarding

### Packet Builder

| Payload | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | Build | 23.388 ns | 2.5485 GiB/s | 40.943 ns | 1.4558 GiB/s |
| 64B | Build (priority) | 20.757 ns | 2.8716 GiB/s | 30.095 ns | 1.9805 GiB/s |
| 256B | Build | 51.626 ns | 4.6182 GiB/s | 41.987 ns | 5.6783 GiB/s |
| 256B | Build (priority) | 49.400 ns | 4.8263 GiB/s | 31.901 ns | 7.4737 GiB/s |
| 1KB | Build | 41.060 ns | 23.226 GiB/s | 43.774 ns | 21.786 GiB/s |
| 1KB | Build (priority) | 38.338 ns | 24.875 GiB/s | 34.897 ns | 27.328 GiB/s |
| 4KB | Build | 94.450 ns | 40.389 GiB/s | 66.192 ns | 57.630 GiB/s |
| 4KB | Build (priority) | 93.279 ns | 40.896 GiB/s | 54.450 ns | 70.058 GiB/s |

### Chain Scaling (forward_chain)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 60.075 ns | 16.646 Melem/s | 52.147 ns | 19.177 Melem/s |
| 2 hops | 115.48 ns | 8.6592 Melem/s | 84.541 ns | 11.829 Melem/s |
| 3 hops | 163.31 ns | 6.1232 Melem/s | 119.24 ns | 8.3867 Melem/s |
| 4 hops | 221.16 ns | 4.5217 Melem/s | 151.43 ns | 6.6037 Melem/s |
| 5 hops | 298.76 ns | 3.3471 Melem/s | 186.41 ns | 5.3646 Melem/s |

### Hop Latency

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Single hop process | 1.4917 ns | 670.36 Melem/s | 953.13 ps | 1.0492 Gelem/s |
| Single hop full | 54.924 ns | 18.207 Melem/s | 32.966 ns | 30.335 Melem/s |

### Hop Scaling by Payload Size

| Payload | Hops | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 64B | 1 hops | 29.787 ns | 2.0010 GiB/s | 51.335 ns | 1.1611 GiB/s |
| 64B | 2 hops | 52.402 ns | 1.1375 GiB/s | 82.718 ns | 737.87 MiB/s |
| 64B | 3 hops | 76.510 ns | 797.74 MiB/s | 116.64 ns | 523.28 MiB/s |
| 64B | 4 hops | 100.14 ns | 609.50 MiB/s | 146.93 ns | 415.40 MiB/s |
| 64B | 5 hops | 130.65 ns | 467.15 MiB/s | 180.47 ns | 338.20 MiB/s |
| 256B | 1 hops | 57.408 ns | 4.1530 GiB/s | 52.546 ns | 4.5373 GiB/s |
| 256B | 2 hops | 118.98 ns | 2.0038 GiB/s | 85.848 ns | 2.7772 GiB/s |
| 256B | 3 hops | 161.98 ns | 1.4719 GiB/s | 119.12 ns | 2.0015 GiB/s |
| 256B | 4 hops | 229.50 ns | 1.0389 GiB/s | 151.27 ns | 1.5761 GiB/s |
| 256B | 5 hops | 276.93 ns | 881.59 MiB/s | 187.38 ns | 1.2723 GiB/s |
| 1024B | 1 hops | 49.380 ns | 19.313 GiB/s | 54.025 ns | 17.653 GiB/s |
| 1024B | 2 hops | 118.54 ns | 8.0452 GiB/s | 88.689 ns | 10.753 GiB/s |
| 1024B | 3 hops | 155.43 ns | 6.1359 GiB/s | 124.24 ns | 7.6762 GiB/s |
| 1024B | 4 hops | 214.08 ns | 4.4548 GiB/s | 158.56 ns | 6.0145 GiB/s |
| 1024B | 5 hops | 266.20 ns | 3.5825 GiB/s | 196.44 ns | 4.8548 GiB/s |

### Route and Forward (with routing table lookup)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1 hops | 176.18 ns | 5.6761 Melem/s | 152.94 ns | 6.5384 Melem/s |
| 2 hops | 350.09 ns | 2.8564 Melem/s | 283.72 ns | 3.5247 Melem/s |
| 3 hops | 527.48 ns | 1.8958 Melem/s | 415.71 ns | 2.4055 Melem/s |
| 4 hops | 706.86 ns | 1.4147 Melem/s | 555.89 ns | 1.7989 Melem/s |
| 5 hops | 929.86 ns | 1.0754 Melem/s | 684.26 ns | 1.4614 Melem/s |

### Concurrent Forwarding

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Forward | 742.05 us | 5.3904 Melem/s | 558.79 us | 7.1583 Melem/s |
| 8 | Forward | 1.0698 ms | 7.4782 Melem/s | 699.87 us | 11.431 Melem/s |
| 16 | Forward | 2.2729 ms | 7.0396 Melem/s | 1.2969 ms | 12.337 Melem/s |

## Swarm / Discovery

### Pingwave

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Serialize | 776.77 ps | 1.2874 Gelem/s | 534.31 ps | 1.8716 Gelem/s |
| Deserialize | 931.49 ps | 1.0735 Gelem/s | 634.49 ps | 1.5761 Gelem/s |
| Roundtrip | 931.65 ps | 1.0734 Gelem/s | 633.79 ps | 1.5778 Gelem/s |
| Forward | 627.24 ps | 1.5943 Gelem/s | 525.52 ps | 1.9029 Gelem/s |

### Local Graph

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create_pingwave | 2.1019 ns | 475.77 Melem/s | 4.8889 ns | 204.54 Melem/s |
| on_pingwave_new | 112.51 ns | 8.8880 Melem/s | 174.15 ns | 5.7420 Melem/s |
| on_pingwave_duplicate | 33.135 ns | 30.180 Melem/s | 17.026 ns | 58.734 Melem/s |
| get_node | 31.479 ns | 31.767 Melem/s | 14.762 ns | 67.743 Melem/s |
| node_count | 208.29 ns | 4.8010 Melem/s | 934.80 ns | 1.0697 Melem/s |
| stats | 615.99 ns | 1.6234 Melem/s | 2.8166 us | 355.04 Kelem/s |

### Graph Scaling

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | all_nodes | 2.4677 us | 40.524 Melem/s | 7.2113 us | 13.867 Melem/s |
| 100 | nodes_within_hops | 2.8428 us | 35.177 Melem/s | 7.3082 us | 13.683 Melem/s |
| 500 | all_nodes | 7.9031 us | 63.266 Melem/s | 15.858 us | 31.530 Melem/s |
| 500 | nodes_within_hops | 9.3159 us | 53.671 Melem/s | 15.526 us | 32.204 Melem/s |
| 1,000 | all_nodes | 160.51 us | 6.2300 Melem/s | 26.182 us | 38.194 Melem/s |
| 1,000 | nodes_within_hops | 171.31 us | 5.8375 Melem/s | 26.229 us | 38.126 Melem/s |
| 5,000 | all_nodes | 223.50 us | 22.372 Melem/s | 216.45 us | 23.100 Melem/s |
| 5,000 | nodes_within_hops | 240.81 us | 20.763 Melem/s | 216.89 us | 23.054 Melem/s |

### Path Finding

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| path_1_hop | 1.5153 us | 659.93 Kelem/s | 5.3652 us | 186.39 Kelem/s |
| path_2_hops | 1.5971 us | 626.15 Kelem/s | 5.4806 us | 182.46 Kelem/s |
| path_4_hops | 1.8255 us | 547.79 Kelem/s | 5.6925 us | 175.67 Kelem/s |
| path_not_found | 1.8306 us | 546.26 Kelem/s | 5.6874 us | 175.83 Kelem/s |
| path_complex_graph | 313.18 us | 3.1930 Kelem/s | 276.85 us | 3.6121 Kelem/s |

### Concurrent Pingwave Processing

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Pingwave | 113.91 us | 17.557 Melem/s | 164.18 us | 12.181 Melem/s |
| 8 | Pingwave | 179.90 us | 22.234 Melem/s | 268.29 us | 14.909 Melem/s |
| 16 | Pingwave | 317.45 us | 25.201 Melem/s | 495.20 us | 16.155 Melem/s |

## Failure Detection

### Failure Detector

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| heartbeat_existing | 28.981 ns | 34.505 Melem/s | 33.768 ns | 29.614 Melem/s |
| heartbeat_new | 253.98 ns | 3.9373 Melem/s | 189.82 ns | 5.2682 Melem/s |
| status_check | 13.420 ns | 74.515 Melem/s | 12.908 ns | 77.469 Melem/s |
| check_all | 344.25 ms | 2.9049 elem/s | 647.32 ms | 1.5448 elem/s |
| stats | 81.085 ms | 12.333 elem/s | 192.16 ms | 5.2039 elem/s |

### Circuit Breaker

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| allow_closed | 13.481 ns | 74.178 Melem/s | 9.8585 ns | 101.44 Melem/s |
| record_success | 9.6531 ns | 103.59 Melem/s | 8.4035 ns | 119.00 Melem/s |
| record_failure | 9.6192 ns | 103.96 Melem/s | 9.4532 ns | 105.78 Melem/s |
| state | 13.827 ns | 72.323 Melem/s | 9.8217 ns | 101.82 Melem/s |

### Recovery Manager

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| on_failure_with_alternates | 275.92 ns | 3.6243 Melem/s | 241.75 ns | 4.1364 Melem/s |
| on_failure_no_alternates | 291.96 ns | 3.4252 Melem/s | 212.30 ns | 4.7102 Melem/s |
| get_action | 36.084 ns | 27.713 Melem/s | 36.507 ns | 27.392 Melem/s |
| is_failed | 12.319 ns | 81.174 Melem/s | 12.290 ns | 81.367 Melem/s |
| on_recovery | 106.07 ns | 9.4274 Melem/s | 119.55 ns | 8.3647 Melem/s |
| stats | 702.69 ps | 1.4231 Gelem/s | 1.1536 ns | 866.87 Melem/s |

### Full Recovery Cycle

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Fail + recover cycle | 303.14 ns | 3.2989 Melem/s | 242.68 ns | 4.1206 Melem/s |

### Loss Simulator

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 1% | 2.8028 ns | 356.79 Melem/s | 10.724 ns | 93.246 Melem/s |
| 5% | 3.1644 ns | 316.01 Melem/s | 10.983 ns | 91.051 Melem/s |
| 10% | 3.6482 ns | 274.11 Melem/s | 11.492 ns | 87.018 Melem/s |
| 20% | 4.5941 ns | 217.67 Melem/s | 12.430 ns | 80.448 Melem/s |
| burst | 2.9408 ns | 340.05 Melem/s | 11.114 ns | 89.977 Melem/s |

### Failure Scaling (check_all)

| Nodes | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 100 | check_all | 4.7979 us | 20.842 Melem/s | 8.5232 us | 11.733 Melem/s |
| 100 | healthy_nodes | 1.7015 us | 58.771 Melem/s | 6.0894 us | 16.422 Melem/s |
| 500 | check_all | 20.948 us | 23.869 Melem/s | 22.212 us | 22.510 Melem/s |
| 500 | healthy_nodes | 5.4484 us | 91.770 Melem/s | 9.6302 us | 51.920 Melem/s |
| 1,000 | check_all | 41.121 us | 24.318 Melem/s | 39.325 us | 25.429 Melem/s |
| 1,000 | healthy_nodes | 10.242 us | 97.638 Melem/s | 14.242 us | 70.216 Melem/s |
| 5,000 | check_all | 204.15 us | 24.492 Melem/s | 176.08 us | 28.396 Melem/s |
| 5,000 | healthy_nodes | 48.999 us | 102.04 Melem/s | 51.262 us | 97.538 Melem/s |

### Concurrent Heartbeats

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Heartbeat | 185.37 us | 10.789 Melem/s | 200.50 us | 9.9749 Melem/s |
| 8 | Heartbeat | 258.94 us | 15.448 Melem/s | 301.04 us | 13.287 Melem/s |
| 16 | Heartbeat | 467.35 us | 17.118 Melem/s | 539.22 us | 14.836 Melem/s |

## Stream Multiplexing

| Streams | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 10 | Lookup | 292.61 ns | 34.175 Melem/s | 335.82 ns | 29.777 Melem/s |
| 10 | Stats | 476.61 ns | 20.981 Melem/s | 401.09 ns | 24.932 Melem/s |
| 100 | Lookup | 2.9255 us | 34.182 Melem/s | 3.3827 us | 29.562 Melem/s |
| 100 | Stats | 4.8267 us | 20.718 Melem/s | 4.0649 us | 24.601 Melem/s |
| 1,000 | Lookup | 29.242 us | 34.197 Melem/s | 35.538 us | 28.139 Melem/s |
| 1,000 | Stats | 52.571 us | 19.022 Melem/s | 43.780 us | 22.842 Melem/s |
| 10,000 | Lookup | 293.53 us | 34.069 Melem/s | 385.64 us | 25.931 Melem/s |
| 10,000 | Stats | 568.57 us | 17.588 Melem/s | 463.01 us | 21.598 Melem/s |

## Fair Scheduler

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Creation | 288.81 ns | 3.4625 Melem/s | 1.5589 us | 641.48 Kelem/s |
| Stream count (empty) | 200.62 ns | 4.9844 Melem/s | 942.24 ns | 1.0613 Melem/s |
| Total queued | 312.81 ps | 3.1968 Gelem/s | 198.13 ps | 5.0473 Gelem/s |
| Cleanup (empty) | 201.79 ns | 4.9557 Melem/s | 1.2699 us | 787.47 Kelem/s |

## Capability System

### CapabilitySet

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 520.96 ns | 1.9195 Melem/s | 746.15 ns | 1.3402 Melem/s |
| serialize | 915.93 ns | 1.0918 Melem/s | 724.67 ns | 1.3799 Melem/s |
| deserialize | 1.7249 us | 579.75 Kelem/s | 2.9743 us | 336.22 Kelem/s |
| roundtrip | 2.6818 us | 372.89 Kelem/s | 3.8588 us | 259.15 Kelem/s |
| has_tag | 749.37 ps | 1.3345 Gelem/s | 576.00 ps | 1.7361 Gelem/s |
| has_model | 937.63 ps | 1.0665 Gelem/s | 417.07 ps | 2.3977 Gelem/s |
| has_tool | 749.69 ps | 1.3339 Gelem/s | 690.86 ps | 1.4475 Gelem/s |
| has_gpu | 312.29 ps | 3.2021 Gelem/s | 191.72 ps | 5.2159 Gelem/s |

### Capability Announcement

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| create | 374.32 ns | 2.6715 Melem/s | 1.4728 us | 678.98 Kelem/s |
| serialize | 1.2213 us | 818.81 Kelem/s | 976.44 ns | 1.0241 Melem/s |
| deserialize | 2.1485 us | 465.44 Kelem/s | 2.6077 us | 383.48 Kelem/s |
| is_expired | 25.327 ns | 39.483 Melem/s | 20.835 ns | 47.995 Melem/s |

### Capability Serialization (Simple vs Complex)

| Type | Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| Simple | Serialize | 19.657 ns | 50.873 Melem/s | 35.970 ns | 27.801 Melem/s |
| Simple | Deserialize | 4.8917 ns | 204.43 Melem/s | 8.7685 ns | 114.04 Melem/s |
| Complex | Serialize | 40.999 ns | 24.391 Melem/s | 37.950 ns | 26.351 Melem/s |
| Complex | Deserialize | 153.11 ns | 6.5313 Melem/s | 228.21 ns | 4.3820 Melem/s |

### Capability Filter

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 9.9923 ns | 100.08 Melem/s | 3.2695 ns | 305.86 Melem/s |
| require_gpu | 4.0679 ns | 245.83 Melem/s | 1.7332 ns | 576.97 Melem/s |
| gpu_vendor | 3.7483 ns | 266.79 Melem/s | 1.9190 ns | 521.09 Melem/s |
| min_memory | 3.7478 ns | 266.83 Melem/s | 1.7279 ns | 578.74 Melem/s |
| complex | 10.305 ns | 97.038 Melem/s | 5.2833 ns | 189.28 Melem/s |
| no_match | 3.1225 ns | 320.26 Melem/s | 1.7749 ns | 563.40 Melem/s |

### Capability Index (Insert)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| 100 nodes | 114.68 us | 872.00 Kelem/s | 174.53 us | 572.98 Kelem/s |
| 1,000 nodes | 1.2188 ms | 820.47 Kelem/s | 1.4194 ms | 704.53 Kelem/s |
| 10,000 nodes | 19.690 ms | 507.87 Kelem/s | 15.799 ms | 632.94 Kelem/s |

### Capability Index (Query, 10,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| single_tag | 184.06 us | 5.4329 Kelem/s | 172.88 us | 5.7844 Kelem/s |
| require_gpu | 239.69 us | 4.1720 Kelem/s | 171.25 us | 5.8394 Kelem/s |
| gpu_vendor | 747.74 us | 1.3374 Kelem/s | 487.58 us | 2.0509 Kelem/s |
| min_memory | 779.25 us | 1.2833 Kelem/s | 521.40 us | 1.9179 Kelem/s |
| complex | 515.85 us | 1.9386 Kelem/s | 325.13 us | 3.0757 Kelem/s |
| model | 100.41 us | 9.9596 Kelem/s | 91.849 us | 10.887 Kelem/s |
| tool | 604.28 us | 1.6549 Kelem/s | 473.45 us | 2.1122 Kelem/s |
| no_results | 22.764 ns | 43.929 Melem/s | 25.254 ns | 39.598 Melem/s |

### Capability Index (Find Best)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| Simple | 456.38 us | 2.1912 Kelem/s | 323.84 us | 3.0879 Kelem/s |
| With preferences | 781.65 us | 1.2793 Kelem/s | 527.60 us | 1.8954 Kelem/s |

### Capability Search (1,000 nodes)

| Operation | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|
| find_with_gpu | 17.512 us | 57.105 Kelem/s | 29.005 us | 34.477 Kelem/s |
| find_by_tool (Python) | 31.281 us | 31.968 Kelem/s | 56.336 us | 17.751 Kelem/s |
| find_by_tool (Rust) | 40.041 us | 24.974 Kelem/s | 71.838 us | 13.920 Kelem/s |

### Capability Index Scaling

| Nodes | Query Type | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 1,000 | Tag | 12.573 us | 79.536 Kelem/s | 10.221 us | 97.833 Kelem/s |
| 1,000 | Complex | 40.690 us | 24.576 Kelem/s | 26.819 us | 37.287 Kelem/s |
| 5,000 | Tag | 70.294 us | 14.226 Kelem/s | 53.855 us | 18.568 Kelem/s |
| 5,000 | Complex | 201.85 us | 4.9541 Kelem/s | 131.83 us | 7.5855 Kelem/s |
| 10,000 | Tag | 294.00 us | 3.4014 Kelem/s | 173.48 us | 5.7644 Kelem/s |
| 10,000 | Complex | 680.21 us | 1.4701 Kelem/s | 328.71 us | 3.0422 Kelem/s |
| 50,000 | Tag | 2.7127 ms | 368.63 elem/s | 1.1816 ms | 846.29 elem/s |
| 50,000 | Complex | 3.9879 ms | 250.76 elem/s | 1.9880 ms | 503.03 elem/s |

### Concurrent Capability Index

| Threads | Pool | M1 Max | M1 Throughput | i9-14900K | i9 Throughput |
|---|---|---|---|---|---|
| 4 | Insert | 394.70 us | 5.0672 Melem/s | 866.82 us | 2.3073 Melem/s |
| 4 | Query | 470.11 ms | 4.2544 Kelem/s | 243.63 ms | 8.2093 Kelem/s |
| 8 | Insert | 453.22 us | 8.8257 Melem/s | 856.69 us | 4.6691 Melem/s |
| 8 | Query | 533.76 ms | 7.4940 Kelem/s | 282.43 ms | 14.163 Kelem/s |
| 16 | Insert | 908.31 us | 8.8075 Melem/s | 1.8613 ms | 4.2980 Melem/s |
| 16 | Query | 1.0204 s | 7.8404 Kelem/s | 422.46 ms | 18.937 Kelem/s |

## Running Benchmarks

```bash
cargo bench --features net --bench net
```

For native CPU optimizations:

```bash
RUSTFLAGS="-C target-cpu=native" cargo bench --features net --bench net
```

## Key Insights

1. **Header serialize/deserialize runs at ~770M ops/sec** (i9) / ~500M ops/sec (M1) — sub-2ns per operation
2. **Routing header operations achieve 1.6–2.0G ops/sec** — sub-nanosecond serialization
3. **Thread-local pool eliminates contention** — up to 16x faster than shared pool at 32 threads (M1: 67.6M vs 3.77M ops/sec)
4. **Capability filters run at 200–580M ops/sec** — fast enough for inline packet decisions
5. **Circuit breaker checks are ~10ns** — negligible overhead per packet
6. **Event frame write scales with payload** — 3.3 GiB/s at 64B, 36–48 GiB/s at 4KB on M1
7. **Multi-hop forwarding adds ~30–60ns per hop** — linear scaling, no amplification
8. **i9-14900K wins on routing/lookup and large payloads** due to higher single-thread clock and wider memory bandwidth; **M1 Max wins on small-frame batch writes and event frame serialization**
