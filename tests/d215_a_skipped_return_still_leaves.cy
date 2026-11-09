// A skipped return still leaves, so nothing says the function can reach its end, and checking
// goes on in the next function.
// trace: D215
@config(default);
import std;

fn half [$n: i32] -> i32:
    return $n x "two";
end fn half

fn.main [] -> void:
    let $h: i32 = half(4);
    let $wrong: i64 = $h;
    return void;
end fn.main
