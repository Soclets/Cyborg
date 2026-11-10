// trace: D131, D117
import std;

fn.main [none] -> void:
    let $t: bool = true;
    let $f: bool = false;
    if not $f and $t:
        std::io::fmt_print("(not f) and t{newln}");
    end if
    if not $t and $t:
        std::io::fmt_print("not reached{newln}");
    end if
    if not $t or $t:
        std::io::fmt_print("(not t) or t{newln}");
    end if
    if $t and not $f:
        std::io::fmt_print("t and (not f){newln}");
    end if
    return void;
end fn.main
