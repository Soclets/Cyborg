// Items that aren't plain numbers show their hand-off too.
// trace: D102, D3
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: str set_to "a";
    let $b: str set_to "b";
    let $xs: std::list<str>[len.max: 8 items] set_to [Give $a, Copy $b, "c"];
    for $s in $xs:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    end for
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
