// A broken := leaves its type unknown, so lines using it say nothing.
// trace: D215, D216
@config(default);
import std;

fn.main [] -> void:
    let $a := 0;
    let $b: i32 = $a + 1;
    let $c: i32 = 1;
    let $d: i64 = $c;
    return void;
end fn.main
