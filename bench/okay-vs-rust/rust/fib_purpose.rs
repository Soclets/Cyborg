// Purpose: the cost of a function call. The count comes from the command line, and
// black_box stops the compiler from folding or merging calls.
use std::hint::black_box;

fn fib(n: i64) -> i64 {
    if black_box(n) < 2 { n } else { black_box(fib(n - 1)) + black_box(fib(n - 2)) }
}

fn main() {
    let n: i64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(35);
    println!("{}", fib(n));
}
