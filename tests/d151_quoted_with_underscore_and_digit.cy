// trace: D151
@config(default);
import std;

use std::io::fmt_print as "say_2";

fn.main [] -> void:
    say_2("hi{newln}");
    return void;
end fn.main
