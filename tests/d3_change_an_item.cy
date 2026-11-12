// trace: D3
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 1 items] = [1];
    for $x in Lend $xs:
        change $x = 2;
    end for
    return void;
end fn.main
