// trace: D133, D83
import std;

const $primes: std::coll::array<i32>[len: 1 items] = [2];

fn.main [none] -> void:
    change $primes[pos: 1] = 3;
    return void;
end fn.main
