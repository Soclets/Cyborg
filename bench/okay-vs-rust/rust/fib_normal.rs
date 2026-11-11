// Normal: recursive fib(35), the way a normal person writes it.
fn fib(n: i64) -> i64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn main() {
    println!("{}", fib(35));
}
