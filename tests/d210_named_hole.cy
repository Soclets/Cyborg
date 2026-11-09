// trace: D210
@config(default);
import std;

fn.main [] -> void:
    let $t: i32 = 21;
    let $place: str = "kitchen";
    std::io::fmt_print("{$place}: {$t} C, and {$t} again{newln}");
    return void;
end fn.main
