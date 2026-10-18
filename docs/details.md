# Details

The longer version of the README: what each algorithm does, why Utah Sort
is correct, and the full measurements.

## The algorithms

All three are in-place and use O(1) extra memory.

**Shellsort** (Shell 1959). For each gap *g*, largest first, insertion-sort
each of the *g* interleaved subsequences *A[r], A[r+g], A[r+2g], …*
("chains"). The last gap is 1. Gaps here are Ciura's (1, 4, 10, 23, 57, 132,
301, 701, 1750), extended by ×9/4.

**Shaker sort** (Incerpi & Sedgewick 1987). For each gap *h*, do one
*h-shake*: a left-to-right pass that compare-exchanges *(A[i], A[i+h])* for
*i = 0 … n−h−1*, then a right-to-left pass over the same pairs. Finish with
one insertion sort. Gaps are ⌈1.7^k⌉ (1, 2, 3, 5, 9, 15, 25, 42, 70, 119, …),
as recommended in the paper. On each chain, the left-to-right pass carries
the largest element to the right end and the right-to-left pass carries the
smallest to the left end.

**Utah Sort.** This is the variant an AI tool proposed as new. It adds three
things to shaker sort:

```
mirror pass: compare-exchange A[i] with A[n-1-i] for i < n/2

for each gap g (Ciura's gaps), largest first:
    left-to-right g-pass; R = right end of the last exchange
    if no exchange: continue
    right-to-left g-pass over [0, R]; L = left end of the last exchange
    if no exchange: continue
    g-insertion over [L+2g, R], with no j >= g bounds check
```

## Why Utah Sort is correct

Fix a gap *g*.

**Lemma 1.** After the left-to-right pass, every position *k > R* holds a
value at least as large as every element at a smaller index in its chain.
Those positions are already in their final g-sorted places.

*Proof.* In a left-to-right pass, right after the comparison *(k−g, k)*,
*A[k]* holds the largest of the chain's original elements at indices ≤ *k*.
For *k > R*, neither *(k−g, k)* nor *(k, k+g)* exchanged, so *A[k]* keeps
that value. ∎

**Lemma 2.** After the right-to-left pass over *[0, R]*:
(a) every position *k < L* holds a value no larger than anything after it in
its chain, so it is final; and
(b) for every *p* in *[L, L+g−1]*, *A[p]* is the minimum of its chain over
indices ≥ *p*.

*Proof.* (a) is the mirror image of Lemma 1. For (b): right after the
comparison *(p, p+g)*, *A[p]* is the chain minimum over *[p, R]*, and by
Lemma 1 nothing beyond *R* is smaller. The only later comparison that touches
*p* is *(p−g, p)*, and *p−g < L* means it made no exchange. If *p+g > R*, the
claim follows from Lemma 1 directly. ∎

**Consequence.** Only *[L+2g, R]* needs insertion. Positions in
*[L+g, L+2g−1]* are already in place, because their chain predecessor is the
chain minimum. During insertion, the minimum at *[L, L+g−1]* stops every
inner loop, so `j >= g` can be dropped. The last gap is 1, so the result is
sorted.

**Testing.** `python/check_correctness.py` runs all three sorts on 695,129
inputs: every permutation up to n = 9, every sequence over {0,1,2} up to
n = 11, and random and structured inputs up to n = 20,000. It uses a list
type that raises an error on any negative index, so a broken sentinel would
be caught instead of silently wrapping around. The Rust benchmark also checks
every output against `sort_unstable`.

**Running time.** Each gap still makes one full left-to-right pass, so
sorted and reversed inputs cost Θ(n log n), not Θ(n).

## Comparison counts

From `python/count_comparisons.py`, n = 100,000. Ratios are relative to
Shellsort.

| Input | Shellsort | shaker sort | ratio | Utah Sort | ratio |
|---|---:|---:|---:|---:|---:|
| random | 2,563,023 | 4,164,391 | 1.62 | 3,861,629 | 1.51 |
| few unique (4 values) | 1,326,974 | 4,164,391 | 3.14 | 2,981,924 | 2.25 |
| sorted | 1,219,454 | 4,164,391 | 3.41 | 1,269,454 | 1.04 |
| reversed | 1,678,307 | 4,164,391 | 2.48 | 1,269,454 | 0.76 |
| nearly sorted (1% swapped) | 2,075,429 | 4,164,391 | 2.01 | 3,750,475 | 1.81 |
| organ pipe | 1,665,102 | 4,164,391 | 2.50 | 3,564,530 | 2.14 |
| sorted + 10 random at end | 1,370,636 | 4,164,391 | 3.04 | 3,311,124 | 2.42 |

Shaker sort's count is identical on every input. The shakes are
data-oblivious (they compare the same pairs whatever the values), and on
every input here they left the array fully sorted, so the final insertion pass
made exactly n − 1 comparisons. That is the property that lets it run
branch-free at full speed. It is also why it loses on input that is already
sorted: it can't do less work.

Utah Sort makes fewer comparisons than shaker sort on random input, since it
uses fewer, wider gaps. It is still slower, because its insertion pass at
every gap is the same data-dependent loop that slows Shellsort down.

## Timing

From [`results/ryzen-9-9950x3d/main.md`](../results/ryzen-9-9950x3d/main.md)
(commit `d420208`), in ms. "Branch-free" means the compare-exchange is written
as min/max. The current code runs the branch-free rows about 10% slower on
`u32` because of code layout (27 ms at 1M, in
[`main-u32.md`](../results/ryzen-9-9950x3d/main-u32.md)); see
[Other key types](#other-key-types).

| Input, n | Shellsort | shaker 1987 | shaker 1987, branch-free | Utah | Utah, branch-free | `sort_unstable` |
|---|---:|---:|---:|---:|---:|---:|
| random, 100k | 6.51 | 6.72 | 1.98 | 7.84 | 3.02 | 0.68 |
| random, 1M | 82.2 | 81.7 | 24.3 | 96.9 | 38.7 | 7.93 |
| random, 10M | 1011 | 973 | 288 | 1173 | 472 | 90.6 |
| few unique (4 values), 1M | 15.3 | 19.8 | 23.8 | 19.4 | 18.8 | 1.04 |
| sorted, 1M | 8.28 | 12.8 | 24.6 | 4.36 | 8.47 | 0.18 |
| reversed, 1M | 10.3 | 14.0 | 24.0 | 4.25 | 8.50 | 0.22 |
| nearly sorted (1% swapped), 1M | 60.4 | 61.6 | 24.4 | 72.0 | 32.5 | 8.06 |
| sorted + 10 random at end, 1M | 9.06 | 15.0 | 24.0 | 15.0 | 21.3 | 6.65 |

The SIMD-off build ([`main-no-simd.md`](../results/ryzen-9-9950x3d/main-no-simd.md))
gives the same picture. Branch-free shaker sort takes 24 ms at 1M in both.

Three things stand out:

1. **The branch is the whole difference.** With branches, shaker sort ties
   Shellsort on random input (82 vs 82 ms). Branch-free, it is 3.4× faster in
   this build and 3.1× in the current one.
2. **Branch-free shaker sort takes about the same time on every 1M input**
   (24 ms here, 26–27 ms in the current build). It does
   the same work regardless of the data. That makes it fast where branches
   would be unpredictable (random, nearly sorted) and slow where they would be
   predictable (sorted, few unique values).
3. **Utah Sort wins only on sorted and reversed input**, where its windows
   collapse right away.

### SIMD

With `-C target-cpu=native`, LLVM vectorizes the branch-free shake loops with
AVX-512 (`vpminud`/`vpmaxud` on zmm registers). With `-C no-vectorize-loops
-C no-vectorize-slp`, the same function compiles to scalar `cmovb`/`cmova`
and has no vector min/max. Both builds run branch-free shaker sort in about
24 ms at 1M and 290 ms at 10M, so vectorization isn't what makes it fast.

### Ablation

From the three `ablate-run*.md` files, random input, branch-free throughout:

| Variant | 1M | 10M |
|---|---:|---:|
| Utah Sort (mirror + window + insert every gap, Ciura gaps) | 38.3–38.7 | 469–473 |
| without the mirror pass | 41.3–42.5 | 516–521 |
| without the window too (shake + insert every gap, Ciura gaps) | 37.1–37.3 | 454–461 |
| shake + insert every gap, 1.7^k gaps | 35.1–35.5 | 425–431 |
| shake every gap, insert once at the end, Ciura gaps | 223–225 | 16,700–16,765 |
| shaker sort 1987 (shake every gap, insert once at the end, 1.7^k gaps) | 23.8–24.6 | 289–291 |

Within one build, runs agree to about 2%. Across builds, individual variants
have moved by up to 10%. In an earlier build, removing the mirror pass made
Utah Sort faster, not slower. Two conclusions hold in every build:

- **Insert once, at the end.** Inserting after every gap costs about 1.4×.
- **Use dense gaps for shaking.** With Ciura's gaps (ratio about 2.25), one
  shake per gap leaves the array far from sorted, and the final insertion
  pass takes most of the time. With 1.7^k gaps it has almost nothing left to
  do.

## Other key types

`cargo run --release -- --keys u64` (or `f64`, `str`) runs the main table
with other keys. Every key type is built from the same u32 patterns by a map
that keeps order and equality, so all four sort the same permutations with
the same duplicates, and each of the benchmark's own sorts makes the same
comparisons whatever the key type.

- **`u64`**: the u32 value in the high half and a hash of it in the low half,
  so random keys use all 64 bits.
- **`f64`**: values in about ±2.1 × 10⁹ with 20 fraction bits, both signs, no
  NaN. Every algorithm and the reference sort order them by `f64::total_cmp`.
  The branch-free compare-exchange compares the same integers `total_cmp`
  compares (the bits as an `i64`, with the low 63 bits flipped for negative
  numbers) and picks the two values without a branch (in this build, AVX-512
  compares and masked blends). `f64::min`/`max` would be cheaper but wrong
  for NaN, and for −0.0 vs 0.0 can return the same zero twice.
- **`str`**: `&[u8]` slices of 8–24 lowercase letters in one buffer: 7 base-26
  letters that encode the u32 value, then 1–17 letters that depend only on it.
  They are compared with Rust's standard `Ord for [u8]` (a `memcmp` call, then
  the lengths). The branch-free version picks pointer and length with `cmov`
  on the result, but `memcmp` itself still branches.

Random input, in ms, all from one build
([`main-u32.md`](../results/ryzen-9-9950x3d/main-u32.md),
[`main-u64.md`](../results/ryzen-9-9950x3d/main-u64.md),
[`main-f64.md`](../results/ryzen-9-9950x3d/main-f64.md),
[`main-str.md`](../results/ryzen-9-9950x3d/main-str.md)):

| Keys, n | Shellsort | shaker 1987 | shaker 1987, branch-free | Shellsort ÷ branch-free | `sort_unstable` |
|---|---:|---:|---:|---:|---:|
| u32, 1M | 81.4 | 81.4 | 26.6 | 3.1 | 9.40 |
| u32, 10M | 1005 | 984 | 318 | 3.2 | 109 |
| u64, 1M | 82.3 | 82.5 | 24.6 | 3.3 | 9.28 |
| u64, 10M | 1021 | 1003 | 304 | 3.4 | 110 |
| f64, 1M | 123 | 100 | 43.5 | 2.8 | 12.7 |
| f64, 10M | 1509 | 1228 | 497 | 3.0 | 146 |
| string, 1M | 338 | 448 | 405 | 0.83 | 122 |
| string, 10M | 7902 | 10,408 | 12,561 | 0.63 | 1839 |

1. **`u64`: the result holds.** The times are within the build-to-build
   variation of `u32` (see below).
2. **`f64`: it shrinks to about 3×.** `total_cmp` puts a few integer
   operations in front of every comparison, which slows Shellsort down by
   about 1.5× and branch-free shaker sort by 1.6–1.8× compared with `u64`.
   The other patterns look like `u32`: branch-free shaker sort takes about
   43 ms on every 1M input.
3. **Strings: it disappears.** Shellsort beats shaker sort, with or without
   branches, on every input pattern. On random input, branch-free shaker sort
   is 1.2× slower at 1M and 1.6× slower at 10M. A comparison is now a
   `memcmp` call through two pointers, and that, not the branch, is most of
   the cost: removing the branch saves only 10% at 1M (448 → 405 ms). Shaker
   sort makes 1.6× as many comparisons (at n = 100k), so it loses. At 10M the
   strings take about 160 MB, plus 160 MB for the slices being sorted, more
   than the L3 cache. There the branch-free version is slower than shaker
   sort with a branch (12.6 s vs 10.4 s). A likely reason
   is that with a branch the CPU keeps going on its guess and starts the next
   loads early, while with `cmov` each compare-exchange waits for the one
   before it. I haven't checked this with counters.

**The u32 row is slower than in `main.md`** (26.6 vs 24.3 ms branch-free at
1M). `main.md` came from the previous commit (`d420208`). Running that build
again right after these runs reproduced it (24.7 ms), so it isn't the
machine. The inner loop of branch-free shaker sort compiles to the same
instructions in both builds, apart from register names. Built with
`-C llvm-args=-align-loops=64`, both commits take 26.4 ms
(`--only shaker_branchless`), so for that loop the difference is where it
lands in memory, and 24 ms was the lucky layout. The current code gives about
3.1× for u32. Other cells moved between the two builds too, by up to about a
third in either direction: `sort_unstable` 7.9 → 9.4 ms, Shellsort on sorted
input 8.3 → 5.6 ms, branch-free Utah Sort on sorted input 8.5 → 11.5 ms. I
only tested the alignment explanation on branch-free shaker sort.

## Limitations

- One machine, single-threaded. Four key types, and only one kind of string
  (random lowercase letters, 8–24 bytes).
- No hardware performance counters yet, so the branch-misprediction
  explanation is a hypothesis.
- Gap sequences are the ones recommended in each algorithm's literature, not
  tuned here.
- Small in-order cores may behave like the 1987 VAX. That's untested.

## References

- M. Ciura. Best increments for the average case of Shellsort. *FCT 2001*, LNCS 2138, 106–117.
- W. Dobosiewicz. An efficient variation of bubble sort. *Information Processing Letters* 11(1):5–6, 1980.
- S. Edelkamp, A. Weiß. BlockQuicksort: Avoiding branch mispredictions in quicksort. *ESA 2016*; *ACM JEA* 24, 2019.
- J. Incerpi, R. Sedgewick. Practical variations of Shellsort. *Information Processing Letters* 26(1):37–43, 1987. [PDF](https://sedgewick.io/wp-content/themes/sedgewick/papers/1987Practical.pdf)
- D. E. Knuth. *The Art of Computer Programming, Vol. 3: Sorting and Searching.* Addison-Wesley, 1973. §5.2.1 (insertion with a sentinel), §5.2.2 (cocktail-shaker sort).
- D. L. Shell. A high-speed sorting procedure. *Communications of the ACM* 2(7):30–32, 1959.
