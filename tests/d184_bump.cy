// trace: D184
import std;
import os;

fn bump [$n: Lent.mut i32] returns void:
    change $n set_to $n + 1;
    return void;
end fn bump

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $count: i32 set_to 1;
    bump(Lend.mut $count);
    bump(Lend.mut $count);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $count);
    return void;
end fn.main
