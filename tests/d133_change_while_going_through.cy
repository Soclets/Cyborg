// trace: D133, D2
import std;
@config(default);

fn.main [] -> void:
    let mut $a: std::array<i32>[len: 2 items] = [1, 2];
    for $x in $a:
        change $a[pos: 1] = $x;
    end for
    return void;
end fn.main
