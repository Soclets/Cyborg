// trace: D131
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $b: bool = true;
    let $c: bool = false == (not $b);
    if $c:
        std::io::fmt_print(Lend.mut $out, "grouped{newln}");
    end if
    return void;
end fn.main
