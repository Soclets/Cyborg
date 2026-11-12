// trace: D180, D2
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 3 items] = [1, 2, 3];
    for $n in Lend.mut $xs:
        change $n = 0;
    end for
    return void;
end fn.main
