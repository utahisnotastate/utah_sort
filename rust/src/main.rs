//! Shellsort vs. the Incerpi–Sedgewick (1987) shaker sort, plus the
//! "Utah Sort" refinements, timed single-threaded on u32 keys.
//!
//!   cargo run --release                   main comparison
//!   cargo run --release -- --ablate       which ingredient matters
//!   cargo run --release -- --only NAME    one algorithm, 1M random keys, 10 runs
//!                                         (for `perf stat`; NAME from --list)
//!   cargo run --release -- --list         algorithm names
//!
//! The published numbers were built with RUSTFLAGS="-C target-cpu=native".
//! Add "-C no-vectorize-loops -C no-vectorize-slp" to rule out SIMD.
//!
//! No dependencies. Every result is checked against `sort_unstable`.

use std::hint::black_box;
use std::time::Instant;

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
fn cx_branchy(v: &mut [u32], i: usize, j: usize) {
    if v[i] > v[j] {
        v.swap(i, j);
    }
}

/// Branch-free compare-exchange: min/max compile to cmov (or SIMD).
#[inline(always)]
fn cx(v: &mut [u32], i: usize, j: usize) {
    let (a, b) = (v[i], v[j]);
    v[i] = a.min(b);
    v[j] = a.max(b);
}

fn insertion_pass(v: &mut [u32], g: usize) {
    for i in g..v.len() {
        let t = v[i];
        let mut j = i;
        while j >= g && v[j - g] > t {
            v[j] = v[j - g];
            j -= g;
        }
        v[j] = t;
    }
}

/// One h-shake: a left-to-right then a right-to-left pass.
#[inline(always)]
fn h_shake<const BRANCHLESS: bool>(v: &mut [u32], h: usize) {
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
fn shellsort(v: &mut [u32]) {
    for g in ciura_gaps(v.len()) {
        insertion_pass(v, g);
    }
}

/// Shaker sort (Incerpi & Sedgewick 1987), with ordinary branches.
fn shaker(v: &mut [u32]) {
    for h in shaker_gaps(v.len()) {
        h_shake::<false>(v, h);
    }
    insertion_pass(v, 1);
}

/// Shaker sort (Incerpi & Sedgewick 1987), branch-free.
fn shaker_branchless(v: &mut [u32]) {
    for h in shaker_gaps(v.len()) {
        h_shake::<true>(v, h);
    }
    insertion_pass(v, 1);
}

/// Shaker sort, branch-free, but with Ciura's gaps (isolates the gap choice).
fn shaker_branchless_ciura(v: &mut [u32]) {
    for h in ciura_gaps(v.len()) {
        h_shake::<true>(v, h);
    }
    insertion_pass(v, 1);
}

/// Branch-free h-shake followed by a full h-insertion at every gap
/// (Utah Sort without the mirror pass or the window).
fn shake_insert_every_gap(v: &mut [u32]) {
    for g in ciura_gaps(v.len()) {
        h_shake::<true>(v, g);
        insertion_pass(v, g);
    }
}

/// Same, with the 1987 gaps (completes the gaps x insertion-policy grid).
fn shake_insert_every_gap_17(v: &mut [u32]) {
    for g in shaker_gaps(v.len()) {
        h_shake::<true>(v, g);
        insertion_pass(v, g);
    }
}

/// Utah Sort, line for line from the reference implementation (branchy).
fn utah(v: &mut [u32]) {
    utah_impl::<false, true>(v);
}

/// Utah Sort with branch-free passes.
fn utah_branchless(v: &mut [u32]) {
    utah_impl::<true, true>(v);
}

/// Utah Sort with branch-free passes and no mirror pass.
fn utah_branchless_no_mirror(v: &mut [u32]) {
    utah_impl::<true, false>(v);
}

/// Mirror pass, then per gap: forward pass (track the last swap R),
/// backward pass over [0, R] (track the last swap L), then g-insertion over
/// [L + 2g, R] with no bounds check.
fn utah_impl<const BRANCHLESS: bool, const MIRROR: bool>(v: &mut [u32]) {
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
                v[i] = a.min(b);
                v[i + g] = a.max(b);
                r = if a > b { i + g } else { r };
            } else if a > b {
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
                    v[i] = a.min(b);
                    v[i + g] = a.max(b);
                    l = if a > b { i } else { l };
                } else if a > b {
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
                while *v.get_unchecked(j - g) > t {
                    *v.get_unchecked_mut(j) = *v.get_unchecked(j - g);
                    j -= g;
                }
                *v.get_unchecked_mut(j) = t;
            }
        }
    }
}

fn std_sort_unstable(v: &mut [u32]) {
    v.sort_unstable();
}

type Algo = (&'static str, &'static str, fn(&mut [u32]));

/// (name for --only, column header, function)
const ALGOS: &[Algo] = &[
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
];

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
fn best_ms(f: fn(&mut [u32]), data: &[u32], expect: &[u32], reps: usize) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let mut v = data.to_vec();
        let t = Instant::now();
        f(black_box(&mut v));
        let dt = t.elapsed().as_secs_f64() * 1e3;
        assert!(v == expect, "wrong output");
        best = best.min(dt);
    }
    best
}

fn lookup(name: &str) -> &'static Algo {
    ALGOS
        .iter()
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

fn table(names: &[&str]) {
    let algos: Vec<&Algo> = names.iter().map(|n| lookup(n)).collect();
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
        let data = make_input(pattern, n, &mut rng);
        let mut expect = data.clone();
        expect.sort_unstable();
        let reps = if n >= 10_000_000 { 3 } else { 5 };
        print!("| {pattern} | {n} |");
        for a in &algos {
            print!(" {} |", fmt_ms(best_ms(a.2, &data, &expect, reps)));
        }
        println!();
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--list") {
        for a in ALGOS {
            println!("{:<28} {}", a.0, a.1);
        }
        return;
    }

    if let Some(i) = args.iter().position(|a| a == "--only") {
        let a = lookup(args.get(i + 1).expect("--only needs a name"));
        let mut rng = XorShift(0x2323_2323_2323_2323);
        let data = make_input("random", 1_000_000, &mut rng);
        let mut expect = data.clone();
        expect.sort_unstable();
        let ms = best_ms(a.2, &data, &expect, 10);
        println!("{}: best of 10 = {} ms (1M random u32)", a.0, fmt_ms(ms));
        return;
    }

    println!(
        "Times in ms, best of 5 (3 at n = 10M), single-threaded, u32 keys. \
         avx2={} avx512f={}\n",
        cfg!(target_feature = "avx2"),
        cfg!(target_feature = "avx512f")
    );
    if args.iter().any(|a| a == "--ablate") {
        table(ABLATE);
    } else {
        table(MAIN);
    }
}
