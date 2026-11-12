// A split keeps its list lent to change until the last use of either half.
// trace: D382
import std;
import os;

fn first [$xs: std::coll::list.mut<i64>] -> i64:
    return $xs[pos: 1];
end fn first

fn.main [none] -> void:
    let mut $xs: std::coll::list<i64>[len.max: 8 items] = [1, 2, 3];
    let $h: std::coll::halves.mut<i64> = $xs.%%split.mut(at.items: 1);
    change $xs[pos: 1] = 9;
    let $one: i64 = first(Lend.mut $h.first);
    return void;
end fn.main
