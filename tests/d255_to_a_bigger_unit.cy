// An integer going to a bigger unit says how it rounds, as integer `div` does.
// trace: D255, D48
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
    length.inches: base;
    length.yards set_to 36 length.inches;
    length.feet set_to 12 length.inches;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $up: i32[unit: length.meters] set_to 1500 length.meters;
    let $down: i32[unit: length.meters] set_to -1500 length.meters;
    let $a: i32[unit: length.kilometers] set_to $up as.trunc i32[unit: length.kilometers];
    let $b: i32[unit: length.kilometers] set_to $down as.trunc i32[unit: length.kilometers];
    let $c: i32[unit: length.kilometers] set_to $down as.floor i32[unit: length.kilometers];
    let $d: i32[unit: length.meters] set_to 2000 length.meters;
    let $e: i32[unit: length.kilometers] set_to $d as.exact i32[unit: length.kilometers];
    std::io::fmt_print(Lend.mut $out, "{} {} {} {}{newln}", $a, $b, $c, $e);
    return void;
end fn.main
