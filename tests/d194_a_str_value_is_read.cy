// A print reads its values, so a `str` takes no word.
// trace: D194, D95
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $s: str set_to "hi";
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $s);
    return void;
end fn.main
