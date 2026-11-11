// Ceil: as fast as Rust can go. The sum of 1 to n is n x (n + 1) / 2.
fn main() {
    let n: i64 = 100_000_000;
    println!("{}", n * (n + 1) / 2);
}
