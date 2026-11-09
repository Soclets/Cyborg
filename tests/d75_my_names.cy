// trace: D75
@config(default);
import std;

fn my::x [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn my::x

struct my::if:
    n: i32;
end struct my::if

fn.main [] -> void:
    let $width: i32 = 3;
    let $height: i32 = 4;
    let $area: i32 = $width x $height;
    let $mine: i32 = my::x($width, $height);
    let $thing: my::if = my::if[n = 9];
    std::io::fmt_print("{}{newln}", $area);
    std::io::fmt_print("{}{newln}", $mine);
    std::io::fmt_print("{}{newln}", $thing.n);
    return void;
end fn.main
