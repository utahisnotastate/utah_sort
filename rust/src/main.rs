//! Shellsort vs. the Incerpi–Sedgewick (1987) shaker sort, plus the
//! "Utah Sort" refinements, timed single-threaded.
//!
//!   cargo run --release                   main comparison, u32 keys
//!   cargo run --release -- --keys TYPE    same, TYPE = u32, u64, f64 or str
//!   cargo run --release -- --ablate       which ingredient matters
//!   cargo run --release -- --only NAME    one algorithm, 1M random keys, 10 runs
//!                                         (for `perf stat`; NAME from --list)
//!   cargo run --release -- --list         algorithm names
//!
//! `--keys` also applies to `--ablate` and `--only`.
//!
//! The published numbers were built with RUSTFLAGS="-C target-cpu=native".
//! Add "-C no-vectorize-loops -C no-vectorize-slp" to rule out SIMD.
//!
//! No dependencies. Every result is checked against `sort_unstable`.

use std::hint::{black_box, select_unpredictable};
use std::time::Instant;

// ---------------------------------------------------------------------------
// Key types
// ---------------------------------------------------------------------------

/// What the sorts need from a key. Every algorithm is written once against
/// this trait, and `--keys` picks the type.
trait Key: Copy + PartialEq {
    /// For the table header.
    const NAME: &'static str;
    /// Turns the u32 test patterns into keys, preserving order and equality,
    /// so every key type sorts the same permutations.
    fn from_u32s(xs: &[u32]) -> Vec<Self>;
    /// `a > b`, used with a branch.
    fn gt(a: Self, b: Self) -> bool;
    /// Branch-free `(min, max, a > b)`.
    fn min_max(a: Self, b: Self) -> (Self, Self, bool);
    /// The reference sort, also timed as "std sort_unstable".
    fn sort_std(v: &mut [Self]);
    /// Output check.
    fn same(a: &[Self], b: &[Self]) -> bool {
        a == b
    }
}

impl Key for u32 {
    const NAME: &'static str = "u32";
    fn from_u32s(xs: &[u32]) -> Vec<u32> {
        xs.to_vec()
    }
    #[inline(always)]
    fn gt(a: u32, b: u32) -> bool {
        a > b
    }
    /// min/max compile to cmov (or SIMD).
    #[inline(always)]
    fn min_max(a: u32, b: u32) -> (u32, u32, bool) {
        (a.min(b), a.max(b), a > b)
    }
    fn sort_std(v: &mut [u32]) {
        v.sort_unstable();
    }
}

impl Key for u64 {
    const NAME: &'static str = "u64";
    /// The pattern in the high half and a hash of it in the low half, so
    /// random keys use all 64 bits.
    fn from_u32s(xs: &[u32]) -> Vec<u64> {
        xs.iter()
            .map(|&x| (x as u64) << 32 | mix(x) as u64)
            .collect()
    }
    #[inline(always)]
    fn gt(a: u64, b: u64) -> bool {
        a > b
    }
    #[inline(always)]
    fn min_max(a: u64, b: u64) -> (u64, u64, bool) {
        (a.min(b), a.max(b), a > b)
    }
    fn sort_std(v: &mut [u64]) {
        v.sort_unstable();
    }
}

/// Ordered by `f64::total_cmp`.
impl Key for f64 {
    const NAME: &'static str = "f64";
    /// Values in about ±2.1e9 with a fractional part, so both signs occur.
    /// 32 integer bits + 20 fraction bits fit in the mantissa exactly.
    fn from_u32s(xs: &[u32]) -> Vec<f64> {
        xs.iter()
            .map(|&x| (x as f64 - 2_147_483_648.0) + (mix(x) >> 12) as f64 / 1_048_576.0)
            .collect()
    }
    #[inline(always)]
    fn gt(a: f64, b: f64) -> bool {
        a.total_cmp(&b).is_gt()
    }
    /// Compares the integers that `total_cmp` compares and picks without a
    /// branch (AVX-512 masked blends with target-cpu=native). That was faster
    /// here than min/max on those integers and converting back.
    /// `f64::min`/`max` would be wrong for NaN and for -0.0 vs 0.0.
    #[inline(always)]
    fn min_max(a: f64, b: f64) -> (f64, f64, bool) {
        let gt = total_key(a) > total_key(b);
        (
            select_unpredictable(gt, b, a),
            select_unpredictable(gt, a, b),
            gt,
        )
    }
    fn sort_std(v: &mut [f64]) {
        v.sort_unstable_by(f64::total_cmp);
    }
    /// Bitwise, so -0.0 vs 0.0 or a NaN payload would count as wrong.
    fn same(a: &[f64], b: &[f64]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
    }
}

/// What `f64::total_cmp` compares: the bits as an i64, with the low 63 bits
/// flipped for negative numbers.
#[inline(always)]
fn total_key(x: f64) -> i64 {
    let b = x.to_bits() as i64;
    b ^ (((b >> 63) as u64) >> 1) as i64
}

/// Byte strings of 8–24 bytes, compared by the standard `Ord for [u8]`
/// (memcmp, then length). memcmp branches internally. The branch-free
/// version only avoids branching on the result.
impl Key for &'static [u8] {
    const NAME: &'static str = "8–24-byte string";
    /// x in base 26 as 7 letters, most significant first, which preserves
    /// order, then 1–17 more letters that depend only on x, so equal x give
    /// equal strings. All strings of one input share one leaked buffer.
    fn from_u32s(xs: &[u32]) -> Vec<&'static [u8]> {
        let mut buf = Vec::with_capacity(xs.len() * 17);
        let mut ends = Vec::with_capacity(xs.len());
        for &x in xs {
            let start = buf.len();
            buf.resize(start + 7, 0);
            let mut y = x;
            for c in buf[start..].iter_mut().rev() {
                *c = b'a' + (y % 26) as u8;
                y /= 26;
            }
            let mut h = mix(x ^ 0x9e37_79b9);
            for _ in 0..=h % 17 {
                h = mix(h);
                buf.push(b'a' + (h % 26) as u8);
            }
            ends.push(buf.len());
        }
        let buf: &'static [u8] = buf.leak();
        let mut start = 0;
        ends.into_iter()
            .map(|end| {
                let s = &buf[start..end];
                start = end;
                s
            })
            .collect()
    }
    #[inline(always)]
    fn gt(a: Self, b: Self) -> bool {
        a > b
    }
    #[inline(always)]
    fn min_max(a: Self, b: Self) -> (Self, Self, bool) {
        let gt = a > b;
        (
            select_unpredictable(gt, b, a),
            select_unpredictable(gt, a, b),
            gt,
        )
    }
    fn sort_std(v: &mut [Self]) {
        v.sort_unstable();
    }
}

/// Integer hash (Wellons' lowbias32), for key bits that don't affect order.
fn mix(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

// ---------------------------------------------------------------------------
// Gap sequences
// ---------------------------------------------------------------------------

/// Ciura (2001), extended by 9/4. Largest first, all < n.
fn ciura_gaps(n: usize) -> Vec<usize> {
    let mut g = vec![1usize, 4, 10, 23, 57, 132, 301, 701, 1750];
    while *g.last().unwrap() < n {
        let last = *g.last().unwrap();
        g.push(last * 9 / 4);
    }
    g.into_iter().rev().filter(|&x| x < n).collect()
}

/// ceil(1.7^k), as recommended for shaker sort in the 1987 paper.
fn shaker_gaps(n: usize) -> Vec<usize> {
    let mut g = vec![];
    let mut x = 1.0f64;
    while (x.ceil() as usize) < n {
        let h = x.ceil() as usize;
        if g.last() != Some(&h) {
            g.push(h);
        }
        x *= 1.7;
    }
    g.reverse();
    g
}

// ---------------------------------------------------------------------------
// Building blocks
// ---------------------------------------------------------------------------

/// Branchy compare-exchange.
#[inline(always)]
fn cx_branchy<T: Key>(v: &mut [T], i: usize, j: usize) {
    if T::gt(v[i], v[j]) {
        v.swap(i, j);
    }
}

/// Branch-free compare-exchange.
#[inline(always)]
fn cx<T: Key>(v: &mut [T], i: usize, j: usize) {
    let (lo, hi, _) = T::min_max(v[i], v[j]);
    v[i] = lo;
    v[j] = hi;
}

fn insertion_pass<T: Key>(v: &mut [T], g: usize) {
    for i in g..v.len() {
        let t = v[i];
        let mut j = i;
        while j >= g && T::gt(v[j - g], t) {
            v[j] = v[j - g];
            j -= g;
        }
        v[j] = t;
    }
}

/// One h-shake: a left-to-right then a right-to-left pass.
#[inline(always)]
fn h_shake<T: Key, const BRANCHLESS: bool>(v: &mut [T], h: usize) {
    let n = v.len();
    for i in 0..n - h {
        if BRANCHLESS {
            cx(v, i, i + h)
        } else {
            cx_branchy(v, i, i + h)
        }
    }
    for i in (0..n - h).rev() {
        if BRANCHLESS {
            cx(v, i, i + h)
        } else {
            cx_branchy(v, i, i + h)
        }
    }
}

// ---------------------------------------------------------------------------
// Algorithms
// ---------------------------------------------------------------------------

/// Shellsort (Shell 1959) with Ciura's gaps.
fn shellsort<T: Key>(v: &mut [T]) {
    for g in ciura_gaps(v.len()) {
        insertion_pass(v, g);
    }
}

/// Shaker sort (Incerpi & Sedgewick 1987), with ordinary branches.
fn shaker<T: Key>(v: &mut [T]) {
    for h in shaker_gaps(v.len()) {
        h_shake::<T, false>(v, h);
    }
    insertion_pass(v, 1);
}

/// Shaker sort (Incerpi & Sedgewick 1987), branch-free.
fn shaker_branchless<T: Key>(v: &mut [T]) {
    for h in shaker_gaps(v.len()) {
        h_shake::<T, true>(v, h);
    }
    insertion_pass(v, 1);
}

/// Shaker sort, branch-free, but with Ciura's gaps (isolates the gap choice).
fn shaker_branchless_ciura<T: Key>(v: &mut [T]) {
    for h in ciura_gaps(v.len()) {
        h_shake::<T, true>(v, h);
    }
    insertion_pass(v, 1);
}

/// Branch-free h-shake followed by a full h-insertion at every gap
/// (Utah Sort without the mirror pass or the window).
fn shake_insert_every_gap<T: Key>(v: &mut [T]) {
    for g in ciura_gaps(v.len()) {
        h_shake::<T, true>(v, g);
        insertion_pass(v, g);
    }
}

/// Same, with the 1987 gaps (completes the gaps x insertion-policy grid).
fn shake_insert_every_gap_17<T: Key>(v: &mut [T]) {
    for g in shaker_gaps(v.len()) {
        h_shake::<T, true>(v, g);
        insertion_pass(v, g);
    }
}

/// Utah Sort, line for line from the reference implementation (branchy).
fn utah<T: Key>(v: &mut [T]) {
    utah_impl::<T, false, true>(v);
}

/// Utah Sort with branch-free passes.
fn utah_branchless<T: Key>(v: &mut [T]) {
    utah_impl::<T, true, true>(v);
}

/// Utah Sort with branch-free passes and no mirror pass.
fn utah_branchless_no_mirror<T: Key>(v: &mut [T]) {
    utah_impl::<T, true, false>(v);
}

/// Mirror pass, then per gap: forward pass (track the last swap R),
/// backward pass over [0, R] (track the last swap L), then g-insertion over
/// [L + 2g, R] with no bounds check.
fn utah_impl<T: Key, const BRANCHLESS: bool, const MIRROR: bool>(v: &mut [T]) {
    let n = v.len();
    if n < 2 {
        return;
    }
    if MIRROR {
        for i in 0..n / 2 {
            if BRANCHLESS {
                cx(v, i, n - 1 - i);
            } else {
                cx_branchy(v, i, n - 1 - i);
            }
        }
    }
    for g in ciura_gaps(n) {
        const NONE: usize = usize::MAX;

        let mut r = NONE;
        for i in 0..n - g {
            let (a, b) = (v[i], v[i + g]);
            if BRANCHLESS {
                let (lo, hi, gt) = T::min_max(a, b);
                v[i] = lo;
                v[i + g] = hi;
                r = if gt { i + g } else { r };
            } else if T::gt(a, b) {
                v.swap(i, i + g);
                r = i + g;
            }
        }
        if r == NONE {
            continue; // already g-sorted
        }

        let mut l = NONE;
        if r >= g {
            for i in (0..=r - g).rev() {
                let (a, b) = (v[i], v[i + g]);
                if BRANCHLESS {
                    let (lo, hi, gt) = T::min_max(a, b);
                    v[i] = lo;
                    v[i + g] = hi;
                    l = if gt { i } else { l };
                } else if T::gt(a, b) {
                    v.swap(i, i + g);
                    l = i;
                }
            }
        }
        if l == NONE {
            continue; // already g-sorted
        }

        for i in (l + 2 * g)..=r {
            let t = v[i];
            let mut j = i;
            // SAFETY: after the backward pass, v[l .. l+g] holds the minimum
            // of each g-chain over the active region (Lemma 2 in
            // docs/details.md), so the loop stops at j >= l + g and j - g
            // never goes below l. The Python version checks this on ~700k
            // inputs with a list that rejects negative indices.
            unsafe {
                while T::gt(*v.get_unchecked(j - g), t) {
                    *v.get_unchecked_mut(j) = *v.get_unchecked(j - g);
                    j -= g;
                }
                *v.get_unchecked_mut(j) = t;
            }
        }
    }
}

fn std_sort_unstable<T: Key>(v: &mut [T]) {
    T::sort_std(v);
}

type Algo<T> = (&'static str, &'static str, fn(&mut [T]));

/// (name for --only, column header, function)
fn algos<T: Key>() -> [Algo<T>; 10] {
    [
        ("shellsort", "Shellsort", shellsort),
        ("shaker", "shaker 1987", shaker),
        (
            "shaker_branchless",
            "shaker 1987, branch-free",
            shaker_branchless,
        ),
        ("utah", "Utah", utah),
        ("utah_branchless", "Utah, branch-free", utah_branchless),
        ("std", "std sort_unstable", std_sort_unstable),
        (
            "utah_branchless_no_mirror",
            "− mirror pass",
            utah_branchless_no_mirror,
        ),
        (
            "shake_insert_every_gap",
            "shake+insert every gap, Ciura gaps",
            shake_insert_every_gap,
        ),
        (
            "shake_insert_every_gap_17",
            "shake+insert every gap, 1.7^k gaps",
            shake_insert_every_gap_17,
        ),
        (
            "shaker_branchless_ciura",
            "shake, insert at end, Ciura gaps",
            shaker_branchless_ciura,
        ),
    ]
}

const MAIN: &[&str] = &[
    "shellsort",
    "shaker",
    "shaker_branchless",
    "utah",
    "utah_branchless",
    "std",
];
const ABLATE: &[&str] = &[
    "utah_branchless",
    "utah_branchless_no_mirror",
    "shake_insert_every_gap",
    "shake_insert_every_gap_17",
    "shaker_branchless_ciura",
    "shaker_branchless",
];

// ---------------------------------------------------------------------------
// Inputs and timing
// ---------------------------------------------------------------------------

struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 32) as u32
    }
}

const INPUTS: &[(&str, usize)] = &[
    ("random", 100_000),
    ("random", 1_000_000),
    ("random", 10_000_000),
    ("few unique (4 values)", 1_000_000),
    ("sorted", 1_000_000),
    ("reversed", 1_000_000),
    ("nearly sorted (1% swapped)", 1_000_000),
    ("sorted + 10 random at end", 1_000_000),
];

/// The test pattern as u32s; `Key::from_u32s` turns it into keys.
fn make_input(pattern: &str, n: usize, rng: &mut XorShift) -> Vec<u32> {
    let base: Vec<u32> = (0..n as u32).collect();
    match pattern {
        "random" => (0..n).map(|_| rng.next()).collect(),
        "few unique (4 values)" => (0..n).map(|_| rng.next() % 4).collect(),
        "sorted" => base,
        "reversed" => base.into_iter().rev().collect(),
        "nearly sorted (1% swapped)" => {
            let mut v = base;
            for _ in 0..(n / 100).max(1) {
                let i = rng.next() as usize % n;
                let j = rng.next() as usize % n;
                v.swap(i, j);
            }
            v
        }
        "sorted + 10 random at end" => {
            let mut v = base;
            for k in 0..10 {
                v[n - 1 - k] = rng.next() % n as u32;
            }
            v
        }
        _ => unreachable!("unknown pattern {pattern}"),
    }
}

/// Best wall-clock time of `reps` runs, in milliseconds. Copies the input
/// outside the timed region and checks every output.
fn best_ms<T: Key>(f: fn(&mut [T]), data: &[T], expect: &[T], reps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let mut v = data.to_vec();
        let t = Instant::now();
        f(black_box(&mut v));
        let dt = t.elapsed().as_secs_f64() * 1e3;
        assert!(T::same(&v, expect), "wrong output");
        best = best.min(dt);
    }
    best
}

fn lookup<T: Key>(name: &str) -> Algo<T> {
    algos::<T>()
        .into_iter()
        .find(|a| a.0 == name)
        .unwrap_or_else(|| panic!("unknown algorithm {name}; try --list"))
}

fn fmt_ms(ms: f64) -> String {
    if ms < 10.0 {
        format!("{ms:.2}")
    } else if ms < 100.0 {
        format!("{ms:.1}")
    } else {
        format!("{ms:.0}")
    }
}

fn table<T: Key>(names: &[&str]) {
    let algos: Vec<Algo<T>> = names.iter().map(|n| lookup(n)).collect();
    let mut rng = XorShift(0x2323_2323_2323_2323);

    print!("| input | n |");
    for a in &algos {
        print!(" {} |", a.1);
    }
    print!("\n|---|---:|");
    for _ in &algos {
        print!("---:|");
    }
    println!();

    for &(pattern, n) in INPUTS {
        let data = T::from_u32s(&make_input(pattern, n, &mut rng));
        let mut expect = data.clone();
        T::sort_std(&mut expect);
        let reps = if n >= 10_000_000 { 3 } else { 5 };
        print!("| {pattern} | {n} |");
        for a in &algos {
            print!(" {} |", fmt_ms(best_ms(a.2, &data, &expect, reps)));
        }
        println!();
    }
}

fn run<T: Key>(args: &[String]) {
    if let Some(i) = args.iter().position(|a| a == "--only") {
        let a = lookup::<T>(args.get(i + 1).expect("--only needs a name"));
        let mut rng = XorShift(0x2323_2323_2323_2323);
        let data = T::from_u32s(&make_input("random", 1_000_000, &mut rng));
        let mut expect = data.clone();
        T::sort_std(&mut expect);
        let ms = best_ms(a.2, &data, &expect, 10);
        println!(
            "{}: best of 10 = {} ms (1M random {} keys)",
            a.0,
            fmt_ms(ms),
            T::NAME
        );
        return;
    }

    println!(
        "Times in ms, best of 5 (3 at n = 10M), single-threaded, {} keys. \
         avx2={} avx512f={}\n",
        T::NAME,
        cfg!(target_feature = "avx2"),
        cfg!(target_feature = "avx512f")
    );
    if args.iter().any(|a| a == "--ablate") {
        table::<T>(ABLATE);
    } else {
        table::<T>(MAIN);
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--list") {
        for a in algos::<u32>() {
            println!("{:<28} {}", a.0, a.1);
        }
        return;
    }

    let keys = match args.iter().position(|a| a == "--keys") {
        Some(i) => args
            .get(i + 1)
            .map(String::as_str)
            .expect("--keys needs u32, u64, f64 or str"),
        None => "u32",
    };
    match keys {
        "u32" => run::<u32>(&args),
        "u64" => run::<u64>(&args),
        "f64" => run::<f64>(&args),
        "str" => run::<&'static [u8]>(&args),
        _ => panic!("unknown key type {keys}; use u32, u64, f64 or str"),
    }
}
