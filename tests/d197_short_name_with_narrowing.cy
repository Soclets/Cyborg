// trace: D197, D199, D93
@config(default);
import std;
use std::io::fmt_print as "say";

fn.main [] -> void:
    say("hi{newln}");
    say.stderr("oops{newln}");
    return void;
end fn.main
