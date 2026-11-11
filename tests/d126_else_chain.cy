// trace: D126
import std;
import os;

fn size [$n: i32] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if $n above 100:
        std::io::fmt_print(Lend.mut $out, "huge{newln}");
    else if $n above 0:
        std::io::fmt_print(Lend.mut $out, "big{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "small{newln}");
    end if
    return void;
end fn size

fn.main [none] returns void:
    size(101);
    size(1);
    size(0);
    return void;
end fn.main
