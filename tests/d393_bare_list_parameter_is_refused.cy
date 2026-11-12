// A list parameter chooses how it's held too.
// trace: D393
import std;

fn first [$xs: std::coll::array<i32>[len: 2 items]] -> i32:
    return $xs[pos: 1];
end fn first

fn.main [none] -> void:
    return void;
end fn.main
