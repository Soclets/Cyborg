// A function's header can break inside its `[ ]`, with the `]` on its own line.
// trace: D219
import std;

fn total [
    $a: i32,
    $b: i32
] -> i32:
    return $a + $b;
end fn total

fn.main [none] -> void:
    std::io::fmt_print("{}{newln}", total(2, 3));
    return void;
end fn.main
