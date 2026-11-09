// A label's `:` is followed by one space.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $a: i32[min:0, max: 9] = 1;
    return void;
end fn.main
