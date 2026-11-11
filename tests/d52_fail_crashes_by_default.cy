// trace: D52, D96
import std;
import os;

fn half [$n: i32] -> i32 fails [odd]:
    if $n mod.trunc 2 != 0:
        fail odd;
    end if
    return $n div.trunc 2;
end fn half

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(4));
    std::io::fmt_print(Lend.mut $out, "{}{newln}", half(3));
    std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    return void;
end fn.main
