"""Count element comparisons for Shellsort, shaker sort and Utah Sort.

    python count_comparisons.py
"""

import random

from sorts import shaker_sort, shell_sort, utah_sort

COUNT = 0


class Counted:
    __slots__ = ("v",)

    def __init__(self, v):
        self.v = v

    def __gt__(self, other):
        global COUNT
        COUNT += 1
        return self.v > other.v


def comparisons(sort, data):
    global COUNT
    arr = [Counted(v) for v in data]
    COUNT = 0
    sort(arr)
    assert [c.v for c in arr] == sorted(data)
    return COUNT


def patterns(n, rng):
    base = list(range(n))
    nearly = base[:]
    for _ in range(max(1, n // 100)):
        i, j = rng.randrange(n), rng.randrange(n)
        nearly[i], nearly[j] = nearly[j], nearly[i]
    return {
        "random": rng.sample(range(n), n),
        "few unique (4 values)": [rng.randrange(4) for _ in range(n)],
        "sorted": base,
        "reversed": base[::-1],
        "nearly sorted (1% swapped)": nearly,
        "organ pipe": base[: n // 2] + base[: n // 2][::-1],
        "sorted + 10 random at end": base[: n - 10] + [rng.randrange(n) for _ in range(10)],
    }


def main():
    rng = random.Random(23)
    for n in (1_000, 10_000, 100_000):
        print(f"\nn = {n:,}  (comparisons; ratio = vs. Shellsort)\n")
        print("| input | Shellsort | shaker sort | ratio | Utah Sort | ratio |")
        print("|---|---:|---:|---:|---:|---:|")
        for name, data in patterns(n, rng).items():
            s = comparisons(shell_sort, data)
            k = comparisons(shaker_sort, data)
            u = comparisons(utah_sort, data)
            print(f"| {name} | {s:,} | {k:,} | {k / s:.2f} | {u:,} | {u / s:.2f} |")


if __name__ == "__main__":
    main()
