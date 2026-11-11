// trace: D133, D20
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 1 items] set_to [50];
    let $b: i32[min: 0, max: 9] set_to $a[pos: 1];
    return void;
end fn.main
