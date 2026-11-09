// trace: D133, D125
@config(default);
import std;

fn set_first [$xs: Lent.mut std::array<i32>[len: 2 items], $to: i32] -> void:
    change $xs[pos: 1] = $to;
    return void;
end fn set_first

fn.main [] -> void:
    let mut $a: std::array<i32>[len: 2 items] = [1, 2];
    change $a[pos: 2] = 9;
    std::io::fmt_print("{}{newln}", $a[offset.items: 1]);
    set_first(Lend.mut $a, 5);
    std::io::fmt_print("{}{newln}", $a[pos: 1]);
    change $a[offset.items: 0] = $a[pos: 1] + $a[pos: 2];
    std::io::fmt_print("{}{newln}", $a[pos: 1]);
    return void;
end fn.main
