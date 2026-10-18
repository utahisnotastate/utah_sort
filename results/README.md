# Results

`comparisons.md` is the output of `python count_comparisons.py`. Comparison
counts don't depend on the machine.

Timings are in one folder per machine. Each timing is the best of 5 runs (3 at
n = 10,000,000), measured with `std::time::Instant` around the sort only. The
input copy happens outside the timed region.

## ryzen-9-9950x3d

Run on 2026-10-17.

| | |
|---|---|
| CPU | AMD Ryzen 9 9950X3D (Zen 5, AVX-512) |
| RAM | 32 GB |
| OS | Windows 11 Pro 10.0.26200 |
| Rust | rustc 1.95.0, `opt-level=3`, LTO, `codegen-units=1`, `-C target-cpu=native` |
| Python | 3.14.0 |

| File | Command (from `rust/`) |
|---|---|
| `main.md` | `cargo run --release` |
| `main-no-simd.md` | same, with `-C no-vectorize-loops -C no-vectorize-slp` added to `RUSTFLAGS` |
| `ablate-run1.md` … `ablate-run3.md` | `cargo run --release -- --ablate`, three separate runs |

The three ablation runs agree to within about 2%. Separate builds of the same
code have differed by up to about 10% on some variants.

## Adding your machine

Make a folder named after your CPU with `main.md`, `ablate-run1.md` and, if you
can, `main-no-simd.md`. Add a short table like the one above, then open a pull
request. An issue with the raw output works too.
