// trace: D117, D124
import std;
import os;

fn say [$n: i32] returns i32:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    std::io::fmt_print(Lend.mut $out, "evaluated {}{newln}", $n);
    return $n;
end fn say

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    if say(1) below say(2) below say(3):
        std::io::fmt_print(Lend.mut $out, "in order{newln}");
    end if
    return void;
end fn.main
