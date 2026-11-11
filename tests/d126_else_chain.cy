// trace: D126
import std;
import os;

fn size [$n: i32] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if $n above 100:
        std::io::fmt_print(Lend.mut $out, "huge{newln}");
    else if $n above 0:
        std::io::fmt_print(Lend.mut $out, "big{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "small{newln}");
    end if
    return void;
end fn size

fn.main [none] -> void:
    size(101);
    size(1);
    size(0);
    return void;
end fn.main
