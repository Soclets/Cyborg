// Which lists are accepted where another is asked for isn't stated.
// trace: D140
import std;

fn f [$xs: std::list<i32>[len.max: inf items]] returns void:
    return void;
end fn f

fn.main [none] returns void:
    let $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    f(Lend $xs);
    return void;
end fn.main
