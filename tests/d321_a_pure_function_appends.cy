// Appending to a list with a finite `len.max` allocates nothing, but fails when the list is
// full, and whether a pure function may fail isn't decided.
// trace: D321, D86
import std;

fn.pure grow [$a: i32] -> i32:
    let mut $xs: std::coll::list<i32>[len.max: 4 items] = empty;
    $xs.std::%%append(val: $a);
    return $a;
end fn.pure grow

fn.main [none] -> void:
    return void;
end fn.main
