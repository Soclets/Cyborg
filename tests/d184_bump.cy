// trace: D184
import std;
import os;

fn bump [$n: Lent.mut i32] -> void:
    change $n = $n + 1;
    return void;
end fn bump

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $count: i32 = 1;
    bump(Lend.mut $count);
    bump(Lend.mut $count);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $count);
    return void;
end fn.main
