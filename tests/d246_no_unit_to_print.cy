// `unit` only applies to a value with a unit.
// trace: D246
import std;

fn.main [none] -> void:
    let $n: i32 = 5;
    std::io::fmt_print("{$n, unit}{newln}");
    return void;
end fn.main
