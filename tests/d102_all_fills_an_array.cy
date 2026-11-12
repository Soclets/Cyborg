// trace: D102
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $d: std::coll::array<u8>[len: 3 items] = all 255;
    for $x in Lend $d:
        let $copy: u8 = $x;
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $d.len);
    return void;
end fn.main
