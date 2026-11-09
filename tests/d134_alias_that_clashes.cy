// A name of yours that clashes stays bare where it's defined, and says `my::` where it's used.
// trace: D134, D75
import std;
@config(default);

type.alias str = i32;

fn.main [] -> void:
    let $a: my::str = 5;
    let $b: str = "still a string";
    std::io::fmt_print("{}{newln}", $a);
    std::io::fmt_print("{}{newln}", $b);
    return void;
end fn.main
