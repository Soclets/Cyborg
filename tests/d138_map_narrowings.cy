// trace: D138, D24, D25
import std;

fn.main [none] -> void:
    let $h: std::map.hash<key: str, val: i32> = empty;
    let $o: std::map.ordered<key: str, val: i32> = empty;
    std::io::fmt_print("made{newln}");
    return void;
end fn.main
