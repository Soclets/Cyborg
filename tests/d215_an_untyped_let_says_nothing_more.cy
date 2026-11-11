// A `let` with no type leaves its type unknown, so lines using it say nothing.
// trace: D215, D277
import std;

fn.main [none] returns void:
    let $a set_to 0;
    let $b: i32 set_to $a + 1;
    let $c: i32 set_to 1;
    let $d: i64 set_to $c;
    return void;
end fn.main
