// A label's `:` is followed by one space.
// trace: D213
import std;

fn.main [none] returns void:
    let $a: i32[min:0, max: 9] set_to 1;
    return void;
end fn.main
