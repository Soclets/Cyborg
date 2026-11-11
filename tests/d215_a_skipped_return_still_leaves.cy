// A skipped return still leaves, so nothing says the function can reach its end, and checking
// goes on in the next function.
// trace: D215
import std;

fn half [$n: i32] returns i32:
    return $n x "two";
end fn half

fn.main [none] returns void:
    let $h: i32 set_to half(4);
    let $wrong: i64 set_to $h;
    return void;
end fn.main
