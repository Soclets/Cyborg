// trace: D180, D2
@config(default);
import std;

fn.main [] -> void:
    let mut $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        std::io::fmt_print("{}{newln}", $xs.len);
    end for
    return void;
end fn.main
