// A narrowed operator is spaced as a whole.
// trace: D213
import std;

fn.main [none] returns void:
    let $a: i32 set_to 7;
    let $b: i32 set_to $a div.trunc  2;
    return void;
end fn.main
