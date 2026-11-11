// trace: D138, D24, D25
import std;
import os;

fn.main [none] -> void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h: std::map.hash<key: str, val: i32> = empty;
    let $o: std::map.ordered<key: str, val: i32> = empty;
    std::io::fmt_print(Lend.mut $out, "made{newln}");
    return void;
end fn.main
