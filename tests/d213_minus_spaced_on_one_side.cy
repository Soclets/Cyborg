// `$a -1` is a binary `-` spaced on one side.
// trace: D213
import std;

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to $a -1;
    return void;
end fn.main
