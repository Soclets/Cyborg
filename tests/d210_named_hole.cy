// trace: D210
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $t: i32 = 21;
    let $place: str = "kitchen";
    std::io::fmt_print(Lend.mut $out, "{$place}: {$t} C, and {$t} again{newln}");
    return void;
end fn.main
