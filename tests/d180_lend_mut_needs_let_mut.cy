// trace: D180, D2
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        change $n = 0;
    end for
    return void;
end fn.main
