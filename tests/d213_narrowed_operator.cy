// A narrowed operator is spaced as a whole.
// trace: D213
import std;

fn.main [none] -> void:
    let $a: i32 = 7;
    let $b: i32 = $a div.trunc  2;
    return void;
end fn.main
