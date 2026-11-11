// `print_failed` is a placeholder.
// trace: D95, D96
import std;
import os;

fn say [none] -> void fails [print_failed]:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "hi{newln}")?;
    return void;
end fn say

fn.main [none] -> void:
    say();
    return void;
end fn.main
