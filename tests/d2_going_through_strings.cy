// trace: D2
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $xs: std::array<str>[len: 2 items] set_to ["a", "b"];
    for $s in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    end for
    return void;
end fn.main
