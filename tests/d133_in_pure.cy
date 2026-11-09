// trace: D133, D86
import std;
@config(default);

fn.pure first [$xs: std::array<i32>[len: 2 items]] -> i32:
    return $xs[pos: 1];
end fn.pure first

fn.main [] -> void:
    let $a: std::array<i32>[len: 2 items] = [6, 7];
    std::io::fmt_print("{}{newln}", first(Lend $a));
    return void;
end fn.main
