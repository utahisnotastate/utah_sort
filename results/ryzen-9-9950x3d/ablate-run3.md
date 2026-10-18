Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Utah, branch-free | − mirror pass | shake+insert every gap, Ciura gaps | shake+insert every gap, 1.7^k gaps | shake, insert at end, Ciura gaps | shaker 1987, branch-free |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 2.98 | 3.27 | 2.92 | 2.84 | 5.06 | 1.99 |
| random | 1000000 | 38.7 | 42.5 | 37.1 | 35.5 | 224 | 23.8 |
| random | 10000000 | 473 | 516 | 458 | 431 | 16700 | 289 |
| few unique (4 values) | 1000000 | 18.4 | 21.5 | 20.3 | 34.3 | 15.3 | 24.8 |
| sorted | 1000000 | 8.46 | 11.3 | 19.5 | 33.1 | 14.3 | 24.3 |
| reversed | 1000000 | 8.47 | 24.1 | 19.8 | 33.3 | 14.5 | 24.5 |
| nearly sorted (1% swapped) | 1000000 | 32.9 | 36.5 | 30.5 | 34.1 | 28.7 | 24.2 |
| sorted + 10 random at end | 1000000 | 21.6 | 24.8 | 19.7 | 33.2 | 14.2 | 24.1 |
