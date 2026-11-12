// Items that aren't plain numbers show their hand-off too.
// trace: D102, D3
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: str = "a";
    let $b: str = "b";
    let $xs: std::coll::list<str>[len.max: 8 items] = [Give $a, Copy $b, "c"];
    for $s in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
