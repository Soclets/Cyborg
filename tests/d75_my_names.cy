// trace: D75
import std;
import os;

fn my::x [$a: i32, $b: i32] returns i32:
    return $a + $b;
end fn my::x

struct my::if:
    n: i32;
end struct my::if

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $width: i32 set_to 3;
    let $height: i32 set_to 4;
    let $area: i32 set_to $width x $height;
    let $mine: i32 set_to my::x($width, $height);
    let $thing: my::if set_to my::if[n set_to 9];
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $area);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $mine);
    std::io::fmt_print(Lend.mut $out, "{}{newln}", $thing.n);
    return void;
end fn.main
