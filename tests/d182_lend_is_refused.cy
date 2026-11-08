// trace: D182, D180
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 2 items] = [1, 2];
    for $n in Lend $xs:
        return void;
    end for
    return void;
end fn.main
