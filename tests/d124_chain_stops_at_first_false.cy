// trace: D124
import std;
import os;

fn say [$n: i32] -> i32:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "evaluated {}{newln}", $n);
    return $n;
end fn say

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    if say(2) below say(1) below say(3):
        std::io::fmt_print(Lend.mut $out, "not reached{newln}");
    end if
    return void;
end fn.main
