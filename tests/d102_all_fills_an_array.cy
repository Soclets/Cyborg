// trace: D102
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $d: std::array<u8>[len: 3 items] set_to all 255;
    for $x in $d:
        let $copy: u8 set_to $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d.len);
    return void;
end fn.main
