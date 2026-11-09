// trace: D3
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    for $x in $xs:
        change $x = 2;
    end for
    return void;
end fn.main
