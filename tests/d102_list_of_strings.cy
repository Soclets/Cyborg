// Items that aren't plain numbers show their hand-off too.
// trace: D102, D3
@config(default);
import std;

fn.main [] -> void:
    let $a: str = "a";
    let $b: str = "b";
    let $xs: std::list<str>[len.max: 8 items] = [Give $a, Copy $b, "c"];
    for $s in $xs:
        std::io::fmt_print("{}{newln}", $s);
    end for
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
