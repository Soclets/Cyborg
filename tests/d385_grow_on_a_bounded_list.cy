// A list with a finite `len.max` never grows, and `grow:` on one isn't decided.
// trace: D385
import std;

fn.main [none] -> void:
    let $a: std::coll::list<i32>[len.max: 4 items, grow: exact] = empty;
    return void;
end fn.main
