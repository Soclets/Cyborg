// trace: D133, D83
import std;
@config(default);

const $primes: std::array<i32>[len: 3 items] = [2, 3, 5];
const $second: i32 = $primes[pos: 2];

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", $second);
    std::io::fmt_print("{}{newln}", $primes[offset.items: 2]);
    return void;
end fn.main
