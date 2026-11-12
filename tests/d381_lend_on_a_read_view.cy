// A word on a read-only view hides no mistake, so it's refused with a fix that drops it.
// trace: D381, D368
import std;

fn first [$xs: std::coll::list<i32>] -> i32:
    return $xs[pos: 1];
end fn first

fn through [$xs: std::coll::list<i32>] -> i32:
    return first(Lend $xs);
end fn through

fn.main [none] -> void:
    return void;
end fn.main
