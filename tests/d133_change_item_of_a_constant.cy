// trace: D133, D83
import std;

const $primes: std::array<i32>[len: 1 items] = [2];

fn.main [none] returns void:
    change $primes[pos: 1] = 3;
    return void;
end fn.main
