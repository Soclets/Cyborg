// trace: D127, D75
import std;
import os;

fn my::true [none] returns i32:
    return 1;
end fn my::true

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $a: i32 set_to my::true();
    let $b: bool set_to true;
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $a);
    if $b:
        std::io::fmt_print(Lend.mut $out, "still a bool{newln}");
    end if
    return void;
end fn.main
