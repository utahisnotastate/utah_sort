Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 6.75 | 6.87 | 1.99 | 7.61 | 3.02 | 0.80 |
| random | 1000000 | 84.3 | 82.6 | 24.2 | 97.2 | 39.2 | 9.50 |
| random | 10000000 | 1021 | 971 | 292 | 1170 | 471 | 108 |
| few unique (4 values) | 1000000 | 12.7 | 20.8 | 24.2 | 19.9 | 18.3 | 1.16 |
| sorted | 1000000 | 5.54 | 12.7 | 24.1 | 4.21 | 8.47 | 0.18 |
| reversed | 1000000 | 9.22 | 14.1 | 24.3 | 4.36 | 8.64 | 0.29 |
| nearly sorted (1% swapped) | 1000000 | 61.4 | 61.4 | 24.5 | 72.1 | 32.9 | 9.74 |
| sorted + 10 random at end | 1000000 | 6.64 | 15.1 | 24.5 | 16.3 | 21.4 | 8.28 |
