// `$a  +  $b` is refused, so spaces can't line things up.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a  +  $a;
    return void;
end fn.main
