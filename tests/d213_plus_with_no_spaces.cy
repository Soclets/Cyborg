// `$a+$b` is refused.
// trace: D213
import std;

fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a+$a;
    return void;
end fn.main
