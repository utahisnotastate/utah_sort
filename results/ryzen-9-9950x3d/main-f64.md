Times in ms, best of 5 (3 at n = 10M), single-threaded, f64 keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 9.39 | 8.35 | 3.73 | 9.21 | 5.29 | 1.12 |
| random | 1000000 | 123 | 100 | 43.5 | 119 | 64.7 | 12.7 |
| random | 10000000 | 1509 | 1228 | 497 | 1441 | 780 | 146 |
| few unique (4 values) | 1000000 | 20.6 | 33.2 | 43.5 | 30.1 | 35.8 | 1.38 |
| sorted | 1000000 | 10.9 | 28.2 | 43.4 | 11.5 | 17.2 | 0.43 |
| reversed | 1000000 | 14.0 | 31.2 | 42.3 | 11.3 | 17.1 | 0.50 |
| nearly sorted (1% swapped) | 1000000 | 90.1 | 79.9 | 43.5 | 89.9 | 56.8 | 12.5 |
| sorted + 10 random at end | 1000000 | 12.4 | 31.0 | 43.9 | 26.8 | 40.7 | 11.4 |
