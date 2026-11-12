// Copying a bare `str` needs no word, and a `let mut` view can be pointed at other text.
// trace: D368
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: str = "first";
    let $b: str = $a;
    let mut $c: str = "a";
    change $c = "b";
    change $c = $b;
    std::io::fmt_print(Lend.mut $out, "{} {} {}{newln}", $a, $b, $c);
    return void;
end fn.main
