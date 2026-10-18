Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 6.51 | 6.72 | 1.98 | 7.84 | 3.02 | 0.68 |
| random | 1000000 | 82.2 | 81.7 | 24.3 | 96.9 | 38.7 | 7.93 |
| random | 10000000 | 1011 | 973 | 288 | 1173 | 472 | 90.6 |
| few unique (4 values) | 1000000 | 15.3 | 19.8 | 23.8 | 19.4 | 18.8 | 1.04 |
| sorted | 1000000 | 8.28 | 12.8 | 24.6 | 4.36 | 8.47 | 0.18 |
| reversed | 1000000 | 10.3 | 14.0 | 24.0 | 4.25 | 8.50 | 0.22 |
| nearly sorted (1% swapped) | 1000000 | 60.4 | 61.6 | 24.4 | 72.0 | 32.5 | 8.06 |
| sorted + 10 random at end | 1000000 | 9.06 | 15.0 | 24.0 | 15.0 | 21.3 | 6.65 |
