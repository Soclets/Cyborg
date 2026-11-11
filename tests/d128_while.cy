// trace: D128
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let mut $n: i32 = 3;
    while $n above 0:
        std::io::fmt_print(Lend.mut $out, "{}{newln}", $n);
        change $n = $n - 1;
    end while
    while false:
        std::io::fmt_print(Lend.mut $out, "never{newln}");
    end while
    return void;
end fn.main
