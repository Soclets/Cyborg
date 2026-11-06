// trace: D75
import std;

fn x [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn x

struct if:
    n: i32;
end struct if

fn.main [] -> void:
    let $width: i32 = 3;
    let $height: i32 = 4;
    let $area: i32 = $width x $height;
    let $mine: i32 = my::x($width, $height);
    let $thing: my::if = my::if[n = 9];
    std::println($area as str);
    std::println($mine as str);
    std::println($thing.n as str);
    return void;
end fn.main
