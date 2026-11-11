// A line can't break before a binary operator.
// trace: D219
import std;

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to $a
        + 2;
    return void;
end fn.main
