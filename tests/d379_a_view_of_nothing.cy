// A list with no `len.max` is a view of some other list, so it can't be `empty`.
// trace: D11, D379, D103
import std;

fn.main [none] -> void:
    let $xs: std::coll::list<i32> = empty;
    return void;
end fn.main
