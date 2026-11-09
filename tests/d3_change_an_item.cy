// trace: D3
import std;
@config(default);

fn.main [] -> void:
    let $xs: std::array<i32>[len: 1 items] = [1];
    for $x in $xs:
        change $x = 2;
    end for
    return void;
end fn.main
