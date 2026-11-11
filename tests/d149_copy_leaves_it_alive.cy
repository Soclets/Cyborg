// trace: D149, D3
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 1];
    let $w: wrapped set_to Copy $r as wrapped;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
