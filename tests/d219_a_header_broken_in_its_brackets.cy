// A function's header can break inside its `[ ]`, with the `]` on its own line.
// trace: D219
import std;
import os;

fn total [
    $a: i32,
    $b: i32
] returns i32:
    return $a + $b;
end fn total

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", total(2, 3));
    return void;
end fn.main
