// trace: D2
@config(default);
import std;

fn.main [] -> void:
    let $xs: std::array<str>[len: 2 items] = ["a", "b"];
    for $s in $xs:
        std::io::fmt_print("{}{newln}", $s);
    end for
    return void;
end fn.main
