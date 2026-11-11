// How checking carries on past an error in a function's header isn't decided, so it stops there.
// trace: D215
import std;

fn half [$n: nope] returns i32:
    let $a: i64 set_to 1;
    return 1;
end fn half

fn.main [none] returns void:
    let $b: i32 set_to 1;
    let $c: i64 set_to $b;
    return void;
end fn.main
