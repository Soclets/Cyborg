// trace: D126
import std;
import os;

fn say [$n: i32] -> bool:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "test {}{newln}", $n);
    return $n == 2;
end fn say

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if say(1):
        std::io::fmt_print(Lend.mut $out, "one{newln}");
    else if say(2):
        std::io::fmt_print(Lend.mut $out, "two{newln}");
    else if say(3):
        std::io::fmt_print(Lend.mut $out, "three{newln}");
    else:
        std::io::fmt_print(Lend.mut $out, "none{newln}");
    end if
    return void;
end fn.main
