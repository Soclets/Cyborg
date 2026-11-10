// `unit` only applies to a value with a unit.
// trace: D246
@config(default);
import std;

fn.main [] -> void:
    let $n: i32 = 5;
    std::io::fmt_print("{$n, unit}{newln}");
    return void;
end fn.main
