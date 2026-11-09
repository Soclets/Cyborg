// trace: D133, D128
@config(default);
import std;

fn.main [] -> void:
    let $a: std::array<i32>[len: 3 items] = [1, 2, 3];
    let mut $total: i32 = 0;
    for $i in range[from: 1, to: 3]:
        change $total = $total + $a[pos: $i];
    end for
    std::io::fmt_print("{}{newln}", $total);
    return void;
end fn.main
