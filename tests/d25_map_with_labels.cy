// trace: D24, D25
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $m: std::map.hash<key: str, val: i32> = empty;
    std::io::fmt_print(Lend.mut $out, "made{newln}");
    return void;
end fn.main
