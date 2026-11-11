// trace: D133
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 2 items] set_to [1, 2];
    let $n: i32 set_to $a[pos: 1, offset: 0];
    return void;
end fn.main
