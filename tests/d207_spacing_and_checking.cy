// A spacing error doesn't stop checking.
// trace: D207, D213
import std;

fn.main [none] returns void:
    let $a: i32 set_to 1;
    let $b: i32 set_to $a+1;
    let $c: i64 set_to $a;
    return void;
end fn.main
