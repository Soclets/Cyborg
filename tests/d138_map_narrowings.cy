// trace: D138, D24, D25
import std;
import os;

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $h: std::map.hash<key: str, val: i32> set_to empty;
    let $o: std::map.ordered<key: str, val: i32> set_to empty;
    std::io::fmt_print(Lend.mut $out, "made{newln}");
    return void;
end fn.main
