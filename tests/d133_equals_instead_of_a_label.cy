// trace: D133, D13
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    let $n: i32 = $a[pos = 1];
    return void;
end fn.main
