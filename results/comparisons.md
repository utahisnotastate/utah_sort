
n = 1,000  (comparisons; ratio = vs. Shellsort)

| input | Shellsort | shaker sort | ratio | Utah Sort | ratio |
|---|---:|---:|---:|---:|---:|
| random | 12,987 | 24,179 | 1.86 | 20,871 | 1.61 |
| few unique (4 values) | 7,855 | 24,179 | 3.08 | 17,147 | 2.18 |
| sorted | 6,771 | 24,179 | 3.57 | 7,271 | 1.07 |
| reversed | 9,354 | 24,179 | 2.58 | 7,271 | 0.78 |
| nearly sorted (1% swapped) | 8,966 | 24,179 | 2.70 | 18,718 | 2.09 |
| organ pipe | 9,539 | 24,179 | 2.53 | 19,541 | 2.05 |
| sorted + 10 random at end | 8,209 | 24,179 | 2.95 | 17,922 | 2.18 |

n = 10,000  (comparisons; ratio = vs. Shellsort)

| input | Shellsort | shaker sort | ratio | Utah Sort | ratio |
|---|---:|---:|---:|---:|---:|
| random | 190,574 | 329,805 | 1.73 | 295,654 | 1.55 |
| few unique (4 values) | 104,929 | 329,805 | 3.14 | 232,211 | 2.21 |
| sorted | 94,226 | 329,805 | 3.50 | 99,226 | 1.05 |
| reversed | 134,535 | 329,805 | 2.45 | 99,226 | 0.74 |
| nearly sorted (1% swapped) | 145,104 | 329,805 | 2.27 | 282,290 | 1.95 |
| organ pipe | 130,016 | 329,805 | 2.54 | 273,504 | 2.10 |
| sorted + 10 random at end | 114,535 | 329,805 | 2.88 | 272,898 | 2.38 |

n = 100,000  (comparisons; ratio = vs. Shellsort)

| input | Shellsort | shaker sort | ratio | Utah Sort | ratio |
|---|---:|---:|---:|---:|---:|
| random | 2,563,023 | 4,164,391 | 1.62 | 3,861,629 | 1.51 |
| few unique (4 values) | 1,326,974 | 4,164,391 | 3.14 | 2,981,924 | 2.25 |
| sorted | 1,219,454 | 4,164,391 | 3.41 | 1,269,454 | 1.04 |
| reversed | 1,678,307 | 4,164,391 | 2.48 | 1,269,454 | 0.76 |
| nearly sorted (1% swapped) | 2,075,429 | 4,164,391 | 2.01 | 3,750,475 | 1.81 |
| organ pipe | 1,665,102 | 4,164,391 | 2.50 | 3,564,530 | 2.14 |
| sorted + 10 random at end | 1,370,636 | 4,164,391 | 3.04 | 3,311,124 | 2.42 |
