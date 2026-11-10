// An integer going to a bigger unit says how it rounds, as integer `div` does.
// trace: D255, D48
@config(default);
import std;

fam.unit length:
    length.meters: base;
    length.kilometers = 1000 length.meters;
    length.inches: base;
    length.yards = 36 length.inches;
    length.feet = 12 length.inches;
end fam.unit length

fn.main [] -> void:
    let $up: i32[unit: length.meters] = 1500 length.meters;
    let $down: i32[unit: length.meters] = -1500 length.meters;
    let $a: i32[unit: length.kilometers] = $up as.trunc i32[unit: length.kilometers];
    let $b: i32[unit: length.kilometers] = $down as.trunc i32[unit: length.kilometers];
    let $c: i32[unit: length.kilometers] = $down as.floor i32[unit: length.kilometers];
    let $d: i32[unit: length.meters] = 2000 length.meters;
    let $e: i32[unit: length.kilometers] = $d as.exact i32[unit: length.kilometers];
    std::io::fmt_print("{} {} {} {}{newln}", $a, $b, $c, $e);
    return void;
end fn.main
