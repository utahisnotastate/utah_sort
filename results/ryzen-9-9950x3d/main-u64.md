Times in ms, best of 5 (3 at n = 10M), single-threaded, u64 keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 6.54 | 6.63 | 2.01 | 8.00 | 3.02 | 0.79 |
| random | 1000000 | 82.3 | 82.5 | 24.6 | 97.9 | 39.7 | 9.28 |
| random | 10000000 | 1021 | 1003 | 304 | 1182 | 487 | 110 |
| few unique (4 values) | 1000000 | 15.6 | 19.8 | 27.3 | 20.3 | 18.6 | 1.19 |
| sorted | 1000000 | 8.35 | 15.3 | 24.8 | 5.28 | 8.77 | 0.19 |
| reversed | 1000000 | 9.98 | 16.3 | 24.8 | 5.24 | 8.69 | 0.25 |
| nearly sorted (1% swapped) | 1000000 | 60.7 | 62.4 | 25.1 | 71.8 | 33.2 | 9.75 |
| sorted + 10 random at end | 1000000 | 9.10 | 15.7 | 25.2 | 16.0 | 21.7 | 8.15 |
