// trace: D134, D120
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: first = 3;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main

type.alias first = second;
type.alias second = third;
type.alias third = i32;
