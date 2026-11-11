// trace: D151
import std;
import os;

use std::io::fmt_print as "say_2";

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    say_2(Lend.mut $out, "hi{newln}");
    return void;
end fn.main
