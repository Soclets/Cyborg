// A line can't break before a binary operator.
// trace: D219
import std;

fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a
        + 2;
    return void;
end fn.main
