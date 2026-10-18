Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 6.47 | 6.73 | 2.17 | 7.77 | 3.24 | 0.80 |
| random | 1000000 | 81.4 | 81.4 | 26.6 | 97.6 | 42.4 | 9.40 |
| random | 10000000 | 1005 | 984 | 318 | 1183 | 518 | 109 |
| few unique (4 values) | 1000000 | 12.8 | 20.2 | 26.5 | 19.4 | 21.4 | 1.16 |
| sorted | 1000000 | 5.59 | 12.9 | 26.4 | 4.23 | 11.5 | 0.18 |
| reversed | 1000000 | 10.4 | 15.0 | 26.7 | 3.98 | 11.2 | 0.22 |
| nearly sorted (1% swapped) | 1000000 | 58.7 | 61.7 | 26.3 | 71.2 | 36.2 | 9.59 |
| sorted + 10 random at end | 1000000 | 8.21 | 13.5 | 26.7 | 16.5 | 24.2 | 8.19 |
