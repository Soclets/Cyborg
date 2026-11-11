// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

struct holder:
    inner: wrapped;
end struct holder

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: reading = reading[celsius = 1];
    let $h: holder = holder[inner = Give $a as wrapped];
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
