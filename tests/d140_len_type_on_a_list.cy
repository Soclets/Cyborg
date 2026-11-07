// trace: D140
import std;

fn total [$xs: std::list<i32>[len.max: inf items, len.type: u32]] -> u32.items:
    return $xs.len;
end fn total

fn.main [] -> void:
    let mut $xs: std::list<i32>[len.max: inf items, len.type: u32] = empty;
    $xs.std::%append(val: 1);
    let $n: u32.items = $xs.len;
    std::println($n as str);
    std::println(total(Lend $xs) as str);
    let $small: std::list<i32>[len.min: 1 items, len.max: 8 items, len.type: u8] = [1, 2];
    let $s: u8.items = $small.len;
    std::println($s as str);
    return void;
end fn.main
