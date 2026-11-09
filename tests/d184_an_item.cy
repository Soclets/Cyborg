// trace: D184, D133
@config(default);
import std;

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 2 items] = [1, 2];
    bump(Lend.mut $xs[pos: 2]);
    std::io::fmt_print("{}{newln}", $xs[pos: 2]);
    return void;
end fn.main
