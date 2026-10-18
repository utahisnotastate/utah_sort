Times in ms, best of 5 (3 at n = 10M), single-threaded, 8–24-byte string keys. avx2=true avx512f=true

| input | n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | std sort_unstable |
|---|---:|---:|---:|---:|---:|---:|---:|
| random | 100000 | 23.2 | 32.3 | 28.2 | 32.5 | 28.6 | 8.99 |
| random | 1000000 | 338 | 448 | 405 | 458 | 407 | 122 |
| random | 10000000 | 7902 | 10408 | 12561 | 10251 | 11863 | 1839 |
| few unique (4 values) | 1000000 | 153 | 443 | 455 | 350 | 362 | 21.0 |
| sorted | 1000000 | 105 | 337 | 357 | 105 | 107 | 6.85 |
| reversed | 1000000 | 166 | 349 | 365 | 104 | 107 | 7.49 |
| nearly sorted (1% swapped) | 1000000 | 273 | 426 | 392 | 421 | 386 | 134 |
| sorted + 10 random at end | 1000000 | 133 | 349 | 359 | 311 | 318 | 136 |
