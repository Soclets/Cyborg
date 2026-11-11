// trace: D131, D117
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $t: bool set_to true;
    let $f: bool set_to false;
    if not $f and $t:
        std::io::fmt_print(Lend.mut $out, "(not f) and t{newln}");
    end if
    if not $t and $t:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end if
    if not $t or $t:
        std::io::fmt_print(Lend.mut $out, "(not t) or t{newln}");
    end if
    if $t and not $f:
        std::io::fmt_print(Lend.mut $out, "t and (not f){newln}");
    end if
    return void;
end fn.main
