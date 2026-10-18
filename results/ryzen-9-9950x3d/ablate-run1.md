Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. avx2=true avx512f=true

| input | n | Utah, branch-free | − mirror pass | shake+insert every gap, Ciura gaps | shake+insert every gap, 1.7^k gaps | shake, insert at end, Ciura gaps | shaker 1987, branch-free |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 2.99 | 3.26 | 2.90 | 2.85 | 4.87 | 2.00 |
| random | 1000000 | 38.3 | 42.5 | 37.3 | 35.1 | 225 | 24.6 |
| random | 10000000 | 469 | 516 | 454 | 425 | 16764 | 290 |
| few unique (4 values) | 1000000 | 18.2 | 21.5 | 20.4 | 33.1 | 15.5 | 24.3 |
| sorted | 1000000 | 8.43 | 11.3 | 19.5 | 32.8 | 14.1 | 24.4 |
| reversed | 1000000 | 8.41 | 24.8 | 20.0 | 32.9 | 14.5 | 24.2 |
| nearly sorted (1% swapped) | 1000000 | 32.4 | 36.0 | 31.4 | 35.1 | 28.7 | 24.7 |
| sorted + 10 random at end | 1000000 | 21.0 | 24.9 | 19.7 | 33.2 | 14.1 | 24.0 |
