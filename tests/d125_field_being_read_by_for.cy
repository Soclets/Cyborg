// trace: D125, D2
import std;

struct bag:
    items: std::coll::array<i32>[len: 2 items];
end struct bag

fn.main [none] -> void:
    let mut $b: bag = bag[items = [1, 2]];
    for $x in $b.items:
        change $b.items = [3, 4];
    end for
    return void;
end fn.main
