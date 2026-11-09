// trace: D24, D25
@config(default);
import std;

fn.main [] -> void:
    let $m: std::map.hash<key: str, val: i32> = empty;
    std::io::fmt_print("made{newln}");
    return void;
end fn.main
