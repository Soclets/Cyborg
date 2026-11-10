// trace: D210
import std;

fn.main [none] -> void:
    let $t: i32 = 21;
    let $place: str = "kitchen";
    std::io::fmt_print("{$place}: {$t} C, and {$t} again{newln}");
    return void;
end fn.main
