// trace: D134, D120
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: first set_to 3;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    return void;
end fn.main

type.alias first set_to second;
type.alias second set_to third;
type.alias third set_to i32;
