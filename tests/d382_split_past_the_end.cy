// A split's place is inside the list or at its end, like a position.
// trace: D382
import std;
import os;

fn.main [none] -> void:
    let mut $xs: std::coll::list<i64>[len.max: 8 items] = [1, 2, 3];
    let $h: std::coll::halves.mut<i64> = $xs.std::%%split.mut(at.items: 4);
    return void;
end fn.main
