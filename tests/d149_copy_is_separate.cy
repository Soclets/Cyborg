// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let mut $r: reading set_to reading[celsius set_to 1];
    let $w: wrapped set_to Copy $r as wrapped;
    change $r.celsius set_to 9;
    let $back: reading set_to Give $w as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back.celsius);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $r.celsius);
    return void;
end fn.main
