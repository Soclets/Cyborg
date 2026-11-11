// trace: D132
import std;
import os;

fn read [none] -> i32 fails [none]:
    fail none;
end fn read

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = 10;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a + read() ?? 1));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", ($a x read() ?? 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (read() ?? 1 + 2));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", (read() ?? 3 x 2));
    if read() ?? 1 below 2:
        std::io::fmt_print(Lend.mut $out, "compared after{newln}");
    end if
    return void;
end fn.main
