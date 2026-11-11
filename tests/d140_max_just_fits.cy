// trace: D140
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::list<i32>[len.max: 255 items, len.type: u8] set_to empty;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $xs.len);
    return void;
end fn.main
