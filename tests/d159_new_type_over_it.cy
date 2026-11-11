// trace: D159, D141
import std;
import os;

type.new sensor set_to f64.ieee;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: sensor set_to 0.5;
    let $i: f64.ieee set_to $r as f64.ieee;
    let $clean: f64.stable set_to $i as f64.stable;
    std::io::fmt_print(Lend.mut $out, "ok{newln}");
    return void;
end fn.main
