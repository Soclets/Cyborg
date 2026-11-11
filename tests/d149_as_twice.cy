// The word is on the whole value, so the second `as` is on something fresh.
// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped = reading;

type.new rewrapped = wrapped;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $r: reading = reading[celsius = 1];
    let $w: rewrapped = Give $r as wrapped as rewrapped;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
