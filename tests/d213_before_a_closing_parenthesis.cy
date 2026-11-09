// No space comes before a closing bracket.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $x: i32 = 1;
    let $y: i32 = ($x ) + 1;
    return void;
end fn.main
