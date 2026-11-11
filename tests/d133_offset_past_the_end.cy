// trace: D133, D26
import std;

fn.main [none] returns void:
    let $a: std::array<i32>[len: 2 items] set_to [1, 2];
    let $i: i32 set_to 2;
    let $n: i32 set_to $a[offset.items: $i];
    return void;
end fn.main
