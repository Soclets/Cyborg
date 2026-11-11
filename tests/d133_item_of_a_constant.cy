// trace: D133, D83
import std;
import os;

const $primes: std::array<i32>[len: 3 items] = [2, 3, 5];
const $second: i32 = $primes[pos: 2];

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $second);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $primes[offset.items: 2]);
    return void;
end fn.main
