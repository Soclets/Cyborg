// trace: D86
import std;
import os;

fn.pure f [$out: Lent os::stdout.unbuffered] -> i32:
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return 1;
end fn.pure f

fn.main [none] -> void:
    return void;
end fn.main
