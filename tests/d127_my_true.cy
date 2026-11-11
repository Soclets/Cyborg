// trace: D127, D75
import std;
import os;

fn my::true [none] returns i32:
    return 1;
end fn my::true

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered = os::stdout.unbuffered();
    let $a: i32 = my::true();
    let $b: bool = true;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    if $b:
        std::io::fmt_print(Lend.mut $out, "still a bool{newln}");
    end if
    return void;
end fn.main
