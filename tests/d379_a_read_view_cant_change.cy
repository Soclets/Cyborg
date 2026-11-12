// A view with no `.mut` only reads.
// trace: D379
import std;

fn poke [$xs: std::coll::list<i32>] -> void:
    change $xs[pos: 1] = 4;
    return void;
end fn poke

fn.main [none] -> void:
    return void;
end fn.main
