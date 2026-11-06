// A bad position while baking stops the build.
// trace: D133, D83, D91
import std;

const $primes: std::array<i32>[len: 3 items] = [2, 3, 5];
const $nope: i32 = $primes[pos: 9];

fn.main [] -> void:
    return void;
end fn.main
