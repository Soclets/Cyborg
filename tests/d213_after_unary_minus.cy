// No space follows a unary `-`.
// trace: D213
import std;

fn.main [none] returns void:
    let $x: i32 set_to 3;
    let $y: i32 set_to - $x;
    return void;
end fn.main
