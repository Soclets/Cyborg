// trace: D216
import std;

fn half [$n: i32] -> i32:
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    let $h := half(9);
    std::io::fmt_print("{}{newln}", $h);
    return void;
end fn.main
