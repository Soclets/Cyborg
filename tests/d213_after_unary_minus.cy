// No space follows a unary `-`.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $x: i32 = 3;
    let $y: i32 = - $x;
    return void;
end fn.main
