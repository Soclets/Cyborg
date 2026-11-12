// trace: D2
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $xs: std::coll::array<str>[len: 2 items] = ["a", "b"];
    for $s in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    end for
    return void;
end fn.main
