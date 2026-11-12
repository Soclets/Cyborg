// A list view follows a `str` view's borrow rules, so storing one writes its source.
// trace: D379, D367
import std;

fn keep [$xs: std::coll::list<i32>] -> void:
    let $k: std::coll::list<i32> = $xs;
    return void;
end fn keep

fn.main [none] -> void:
    return void;
end fn.main
