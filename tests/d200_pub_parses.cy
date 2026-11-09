// One file is one package, so `pub` changes nothing it can see.
// trace: D200
import std;
@config(default);

pub const $max: i32 = 100;

pub struct reading:
    celsius: i32;
end struct reading

pub fn total [$a: i32, $b: i32] -> i32:
    return $a + $b;
end fn total

pub fn.pure area [$w: i32, $h: i32] -> i32:
    return $w x $h;
end fn.pure area

fn.main [] -> void:
    std::io::fmt_print("{} {}{newln}", total(1, 2), area(2, 3));
    return void;
end fn.main
