// trace: D133
import std;

fn.main [none] -> void:
    let $xs: std::list<i32>[len.max: 8 items] = [10, 20, 30];
    let $a: std::array<i32>[len: 2 items] = [7, 8];
    std::io::fmt_print("{}{newln}", $xs[pos: 1]);
    std::io::fmt_print("{}{newln}", $xs[offset.items: 0]);
    std::io::fmt_print("{}{newln}", $xs[pos: 3]);
    std::io::fmt_print("{}{newln}", $xs[offset.items: 2]);
    std::io::fmt_print("{}{newln}", $a[pos: 2]);
    let $i: i32 = 2;
    std::io::fmt_print("{}{newln}", $xs[pos: $i]);
    std::io::fmt_print("{}{newln}", $xs[offset.items: $i - 1]);
    let $w: i64 = 1;
    std::io::fmt_print("{}{newln}", $xs[pos: $w]);
    std::io::fmt_print("{}{newln}", ($xs[pos: 1] + $a[offset.items: 0]));
    return void;
end fn.main
