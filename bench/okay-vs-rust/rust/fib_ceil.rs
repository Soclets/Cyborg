// Ceil: as fast as Rust can go. fib(35) by fast doubling.
fn pair(n: u64) -> (i64, i64) {
    if n == 0 { return (0, 1); }
    let (a, b) = pair(n / 2);
    let c = a * (2 * b - a);
    let d = a * a + b * b;
    if n % 2 == 1 { (d, c + d) } else { (c, d) }
}

fn main() {
    println!("{}", pair(35).0);
}
