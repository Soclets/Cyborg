// Normal: adding 1 to 100,000,000 in a loop, the way a normal person writes it.
fn main() {
    let mut sum: i64 = 0;
    let mut i: i64 = 1;
    while i <= 100_000_000 {
        sum += i;
        i += 1;
    }
    println!("{}", sum);
}
