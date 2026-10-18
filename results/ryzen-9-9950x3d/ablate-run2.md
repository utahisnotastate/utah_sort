Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Utah, branch-free | − mirror pass | shake+insert every gap, Ciura gaps | shake+insert every gap, 1.7^k gaps | shake, insert at end, Ciura gaps | shaker 1987, branch-free |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 3.00 | 3.35 | 2.92 | 2.87 | 4.79 | 1.99 |
| random | 1000000 | 38.6 | 41.3 | 37.3 | 35.4 | 223 | 24.3 |
| random | 10000000 | 470 | 521 | 461 | 425 | 16765 | 291 |
| few unique (4 values) | 1000000 | 18.6 | 21.5 | 19.8 | 33.0 | 15.3 | 24.4 |
| sorted | 1000000 | 8.42 | 11.0 | 19.6 | 33.1 | 14.0 | 24.2 |
| reversed | 1000000 | 8.44 | 24.9 | 19.8 | 33.3 | 14.1 | 24.2 |
| nearly sorted (1% swapped) | 1000000 | 32.8 | 36.2 | 30.7 | 34.5 | 28.3 | 24.6 |
| sorted + 10 random at end | 1000000 | 21.3 | 23.8 | 19.5 | 33.2 | 14.5 | 24.0 |
