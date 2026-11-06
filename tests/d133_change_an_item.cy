// trace: D133, D125
import std;

fn set_first [$xs: Lent.mut std::array<i32>[len: 2 items], $to: i32] -> void:
    change $xs[pos: 1] = $to;
    return void;
end fn set_first

fn.main [] -> void:
    let mut $a: std::array<i32>[len: 2 items] = [1, 2];
    change $a[pos: 2] = 9;
    std::println($a[offset: 1] as str);
    set_first(Lend.mut $a, 5);
    std::println($a[pos: 1] as str);
    change $a[offset: 0] = $a[pos: 1] + $a[pos: 2];
    std::println($a[pos: 1] as str);
    return void;
end fn.main
