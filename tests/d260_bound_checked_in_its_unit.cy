// A bound converted to the type's unit is checked there, and the type shows it in that unit.
// trace: D260
import std;
import os;

fam.unit length:
    length.meters: base;
    length.kilometers set_to 1000 length.meters;
end fam.unit length

fn.main [none] returns void:
    let mut $out: os::stdout.unbuffered set_to os::stdout.unbuffered();
    let $walked: i32[unit: length.meters] set_to 1500 length.meters;
    let $short: i32[unit: length.meters, min: 0 length.meters, max: 1 length.kilometers] set_to $walked;
    std::io::fmt_print(Lend.mut $out, "{$short, unit}{newln}");
    return void;
end fn.main
