// trace: D149
import std;
import os;

struct reading:
    celsius: i32;
end struct reading

type.new wrapped set_to reading;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $r: reading set_to reading[celsius set_to 7];
    let $w: wrapped set_to Give $r as wrapped;
    let $back: reading set_to Give $w as reading;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $back.celsius);
    return void;
end fn.main
