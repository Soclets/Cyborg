// A narrowed operator is spaced as a whole.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 7;
    let $b: i32 = $a div.trunc  2;
    return void;
end fn.main
