// trace: D194, D86
import std;
import os;

fn.pure f [$out: os::stdout.unbuffered] returns i32:
    std::io::fmt_print(Lend.mut $out, "hi{newln}");
    return 1;
end fn.pure f

fn.main [none] returns void:
    return void;
end fn.main
