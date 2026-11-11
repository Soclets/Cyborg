// trace: D149, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn make [none] returns reading:
    return reading[celsius = 3];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $w: wrapped = make() as wrapped;
    let $v: wrapped = reading[celsius = 4] as wrapped;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
