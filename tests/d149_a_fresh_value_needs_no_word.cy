// trace: D149, D2
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn make [none] returns reading:
    return reading[celsius set_to 3];
end fn make

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $w: wrapped set_to make() as wrapped;
    let $v: wrapped set_to reading[celsius set_to 4] as wrapped;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
