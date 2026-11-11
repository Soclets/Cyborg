// A spacing error doesn't stop checking.
// trace: D207, D213
import std;

fn.main [none] -> void:
    let $a: i32 = 1;
    let $b: i32 = $a+1;
    let $c: i64 = $a;
    return void;
end fn.main
