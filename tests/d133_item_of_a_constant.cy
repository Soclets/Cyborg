// trace: D133, D83
import std;

const $primes: std::array<i32>[len: 3 items] = [2, 3, 5];
const $second: i32 = $primes[pos: 2];

fn.main [] -> void:
    std::println($second as str);
    std::println($primes[offset: 2] as str);
    return void;
end fn.main
