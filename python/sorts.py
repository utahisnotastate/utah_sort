"""Reference implementations: Shellsort, shaker sort (1987) and Utah Sort.

All three sort in place and use only `>` on elements, so a wrapper type can
count comparisons (see count_comparisons.py).
"""

from typing import List, MutableSequence, TypeVar

T = TypeVar("T")

# Ciura (2001) gaps, extended geometrically by 9/4 for large n.
CIURA = [1, 4, 10, 23, 57, 132, 301, 701, 1750]


def ciura_gaps(n: int) -> List[int]:
    """Gaps below n, largest first."""
    gaps = list(CIURA)
    while gaps[-1] < n:
        gaps.append((gaps[-1] * 9) >> 2)
    return [g for g in reversed(gaps) if g < n]


def shaker_gaps(n: int) -> List[int]:
    """ceil(1.7^k) below n, largest first (Incerpi & Sedgewick 1987)."""
    gaps: List[int] = []
    x = 1.0
    while True:
        h = int(-(-x // 1))  # ceil
        if h >= n:
            break
        if not gaps or gaps[-1] != h:
            gaps.append(h)
        x *= 1.7
    return gaps[::-1]


def _insertion_pass(arr: MutableSequence[T], g: int) -> None:
    for i in range(g, len(arr)):
        temp = arr[i]
        j = i
        while j >= g and arr[j - g] > temp:
            arr[j] = arr[j - g]
            j -= g
        arr[j] = temp


def shell_sort(arr: MutableSequence[T]) -> MutableSequence[T]:
    """Shellsort (Shell 1959) with Ciura's gaps."""
    for g in ciura_gaps(len(arr)):
        _insertion_pass(arr, g)
    return arr


def shaker_sort(arr: MutableSequence[T]) -> MutableSequence[T]:
    """Shaker sort (Incerpi & Sedgewick 1987).

    At each gap h, one left-to-right and one right-to-left pass of
    compare-exchanges between elements h apart (an "h-shake"), then a single
    insertion sort at the end.
    """
    n = len(arr)
    for h in shaker_gaps(n):
        for i in range(n - h):
            if arr[i] > arr[i + h]:
                arr[i], arr[i + h] = arr[i + h], arr[i]
        for i in range(n - h - 1, -1, -1):
            if arr[i] > arr[i + h]:
                arr[i], arr[i + h] = arr[i + h], arr[i]
    _insertion_pass(arr, 1)
    return arr


def utah_sort(arr: MutableSequence[T]) -> MutableSequence[T]:
    """Utah Sort: an h-shake plus h-insertion at every gap, with a mirror
    pass first and a shrinking window. Correct, but slower than shaker sort
    on random data (see docs/details.md).
    """
    n = len(arr)
    if n < 2:
        return arr

    # Mirror pass: compare-exchange each element with its mirror image.
    for i in range(n >> 1):
        opp = n - 1 - i
        if arr[i] > arr[opp]:
            arr[i], arr[opp] = arr[opp], arr[i]

    for g in ciura_gaps(n):
        # Forward g-pass; remember the right end of the last swap.
        r_max = -1
        for i in range(n - g):
            if arr[i] > arr[i + g]:
                arr[i], arr[i + g] = arr[i + g], arr[i]
                r_max = i + g
        if r_max == -1:
            continue  # already g-sorted

        # Backward g-pass over [0, r_max]; remember the left end of the
        # last swap.
        l_min = -1
        for i in range(r_max - g, -1, -1):
            if arr[i] > arr[i + g]:
                arr[i], arr[i + g] = arr[i + g], arr[i]
                l_min = i
        if l_min == -1:
            continue  # already g-sorted

        # Everything outside [l_min, r_max] is in its final g-sorted place,
        # and arr[l_min .. l_min+g-1] holds the minimum of each g-chain in
        # the active region, so it stops the inner loop without a bounds
        # check.
        for i in range(l_min + 2 * g, r_max + 1):
            temp = arr[i]
            j = i
            while arr[j - g] > temp:
                arr[j] = arr[j - g]
                j -= g
            arr[j] = temp

    return arr
