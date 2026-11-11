// Purpose: a loop with an add. The count comes from the command line, and black_box
// stops the compiler from turning the loop into n x (n + 1) / 2.
use std::hint::black_box;

fn main() {
    let n: i64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(100_000_000);
    let mut sum: i64 = 0;
    let mut i: i64 = 1;
    while i <= n {
        sum = black_box(sum + i);
        i += 1;
    }
    println!("{}", sum);
}
