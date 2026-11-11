// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 7];
    let $w: wrapped = Give $r as wrapped;
    let $back: reading = Give $w as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back.celsius);
    return void;
end fn.main
