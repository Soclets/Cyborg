// No space follows a unary `-`.
// trace: D213
import std;

fn.main [none] -> void:
    let $x: i32 = 3;
    let $y: i32 = - $x;
    return void;
end fn.main
