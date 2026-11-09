// trace: D216
@config(default);
import std;

fn half [$n: i32] -> i32:
    return $n div.trunc 2;
end fn half

fn.main [] -> void:
    let $h := half(9);
    std::io::fmt_print("{}{newln}", $h);
    return void;
end fn.main
