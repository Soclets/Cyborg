// The standard's example: `$next` is an `i32`, the type of `$total + 1`.
// trace: D216
@config(default);
import std;

fn.main [] -> void:
    let $total: i32 = 0;
    let $next := $total + 1;
    let $wrong: i64 = $next;
    return void;
end fn.main
