// trace: D133, D125
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 2 items] = [1, 2];
    change $a[pos: 1] = 9;
    return void;
end fn.main
