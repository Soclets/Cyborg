// trace: D138, D24, D25
import std;
import os;

fn.main [none] -> void:
    let $heap: os::heap = os::heap();
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $h: std::coll::map.hash<key: str, val: i32> = empty;
    defer std::coll::map.done(Give $h, alloc: Lend $heap);
    let $o: std::coll::map.ordered<key: str, val: i32> = empty;
    defer std::coll::map.done(Give $o, alloc: Lend $heap);
    std::io::fmt_print(Lend.mut $out, "made{newln}");
    return void;
end fn.main
