// trace: D2, D4
import std;
import os;

fn add [$xs: Lent.mut std::list<i32>[len.max: inf items]] returns void:
    $xs.std::%%append(val: 1);
    return void;
end fn add

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $xs: std::list<i32>[len.max: inf items] set_to empty;
    add(Lend.mut $xs);
    add(Lend.mut $xs);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
