// A name of yours that clashes stays bare where it's defined, and says `my::` where it's used.
// trace: D134, D75
import std;
import os;

type.alias str set_to i32;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: my::str set_to 5;
    let $b: str set_to "still a string";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $b);
    return void;
end fn.main
