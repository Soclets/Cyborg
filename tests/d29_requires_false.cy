// trace: D29, D84
@config(default);
import std;

fn half [$n: i32] -> i32:
    requires $n at_least 0;

    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", half(4));
    std::io::fmt_print("{}{newln}", half(-4));
    return void;
end fn.main
