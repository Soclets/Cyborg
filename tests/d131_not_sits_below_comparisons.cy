// trace: D131
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to 1;
    let $b: i32 set_to 2;
    if not $a below 3:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "a is under 3{newln}");
    end if
    if not $a == $b:
        std::io::fmt_print(Lend.mut $out, "they differ{newln}");
    end if
    if not $a + 1 == $b:
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "the sum matches{newln}");
    end if
    if not not $a == 1:
        std::io::fmt_print(Lend.mut $out, "twice{newln}");
    end if
    return void;
end fn.main
