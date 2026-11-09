// `- 1` is a spaced unary `-`.
// trace: D213
@config(default);
import std;

fn.main [] -> void:
    let $y: i32 = - 1;
    return void;
end fn.main
