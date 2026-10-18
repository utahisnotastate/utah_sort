"""Correctness checks for all three sorts.

Uses a list that refuses negative indices, so Utah Sort's sentinel claim (the
inner insertion loop never needs a `j >= g` bounds check) is actually
tested; a plain Python list would silently wrap arr[-1] to the last element.

    python check_correctness.py
"""

import itertools
import random

from sorts import shaker_sort, shell_sort, utah_sort

SORTS = (shell_sort, shaker_sort, utah_sort)


class NoNegativeIndexList(list):
    def __getitem__(self, i):
        if isinstance(i, int) and i < 0:
            raise IndexError(f"negative index {i}")
        return super().__getitem__(i)

    def __setitem__(self, i, v):
        if isinstance(i, int) and i < 0:
            raise IndexError(f"negative index {i}")
        super().__setitem__(i, v)


def check(data):
    expected = sorted(data)
    for sort in SORTS:
        arr = NoNegativeIndexList(data)
        sort(arr)
        if list(arr) != expected:
            raise AssertionError(f"{sort.__name__} failed on {data!r}: {list(arr)!r}")


def main():
    cases = 0

    # Every permutation up to n = 9.
    for n in range(10):
        for p in itertools.permutations(range(n)):
            check(list(p))
            cases += 1

    # Every sequence over {0,1,2} up to n = 11 (lots of duplicates).
    for n in range(12):
        for p in itertools.product(range(3), repeat=n):
            check(list(p))
            cases += 1

    # Random inputs across many sizes, including sizes just around each gap.
    rng = random.Random(23)
    sizes = list(range(12, 300)) + [
        g + d for g in (4, 10, 23, 57, 132, 301, 701, 1750, 3937, 8858)
        for d in (-2, -1, 0, 1, 2)
    ]
    for n in sizes:
        for _ in range(20):
            check([rng.randrange(n) for _ in range(n)])
            check([rng.randrange(4) for _ in range(n)])
            check(rng.sample(range(n), n))
            cases += 3

    # Structured inputs.
    for n in (1000, 5000, 20000):
        base = list(range(n))
        for data in (
            base,
            base[::-1],
            base[: n // 2] + base[: n // 2][::-1],  # organ pipe
            base[n // 3:] + base[: n // 3],         # rotated
            [i % 37 for i in range(n)],             # sawtooth
        ):
            check(data)
            cases += 1

    names = ", ".join(s.__name__ for s in SORTS)
    print(f"OK: {cases:,} inputs, each sorted correctly by {names}; no negative index used")


if __name__ == "__main__":
    main()
