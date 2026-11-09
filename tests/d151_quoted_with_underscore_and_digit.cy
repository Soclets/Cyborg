// trace: D151
import std;

use std::io::fmt_print as "say_2";
@config(default);

fn.main [] -> void:
    say_2("hi{newln}");
    return void;
end fn.main
