// trace: D29, D84
@config(default);
import std;

fn right [$n: i32] -> i32:
    ensures result at_least $n;

    return $n + 1;
end fn right

fn.main [] -> void:
    std::io::fmt_print("{}{newln}", right(5));
    return void;
end fn.main
