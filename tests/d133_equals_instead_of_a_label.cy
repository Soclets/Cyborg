// trace: D133, D13
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 2 items] set_to [1, 2];
    let $n: i32 set_to $a[pos set_to 1];
    return void;
end fn.main
