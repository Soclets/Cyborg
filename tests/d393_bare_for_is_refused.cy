// A loop reads, changes or takes each item, so it says which, even the usual one.
// trace: D393
import std;

fn.main [none] -> void:
    let $xs: std::coll::array<i32>[len: 2 items] = [1, 2];
    for $n in $xs:
    end for
    return void;
end fn.main
