// trace: D197, D199, D93
import std;
use std::io::fmt_print as "say";
@config(default);

fn.main [] -> void:
    say("hi{newln}");
    say.stderr("oops{newln}");
    return void;
end fn.main
