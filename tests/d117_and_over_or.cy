// trace: D117, D49
import std;
import os;

fn.pure yes [none] returns bool:
    return 1 below 2;
end fn.pure yes

fn.pure no [none] returns bool:
    return 2 below 1;
end fn.pure no

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if yes() or no() and no():
        std::io::fmt_print(Lend.mut $out, "and binds tighter{newln}");
    end if
    if not no() and yes():
        std::io::fmt_print(Lend.mut $out, "not binds tighter{newln}");
    end if
    return void;
end fn.main
