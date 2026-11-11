// The word is on the whole value, so the second `as` is on something fresh.
// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

type.new rewrapped set_to wrapped;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 1];
    let $w: rewrapped set_to Give $r as wrapped as rewrapped;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
